package main

import (
	"go/ast"
	"go/token"
	"go/types"
	"sort"
)

// Ownership facts, observed intra-procedurally.
//
// Go is garbage-collected, so a `*T` says nothing about ownership. These are the facts a front end
// can OBSERVE; what to do about them is the rule pack's decision and the analysis crate's job.

func annotateParameterFacts(
	children []node,
	body *ast.BlockStmt,
	rebound map[string]bool,
	ctx *extractCtx,
) {
	for i := range children {
		if children[i].Kind != kindParam || children[i].Name == "" {
			continue
		}
		facts := ownershipFacts(body, children[i].Name, ctx)
		// The source makes every parameter a mutable local copy and the target makes none of them,
		// so a parameter the body assigns to has to say so. Kept apart from the ownership facts
		// above because it is a claim about the CALLEE's copy, not about the caller's value.
		if rebound[children[i].Name] {
			facts = append(facts, flagRebound)
		}
		// Never mentioned at all. Claimed only where a body exists: without one, "not read" would
		// mean "not looked at".
		if body != nil && children[i].Name != "_" && !mentions(body, children[i].Name) {
			facts = append(facts, flagUnread)
		}
		if len(facts) == 0 {
			continue
		}
		children[i].Flags = append(children[i].Flags, facts...)
		sort.Strings(children[i].Flags)
	}
}

// reboundParameters names the parameters the body assigns to, by name.
//
// Keyed by NAME rather than by object because the caller annotates nodes built from the signature,
// which carry names and no objects; a parameter cannot be shadowed at the top level of its own
// body, so within this scope the name identifies it.
func reboundParameters(assigned map[types.Object]bool) map[string]bool {
	out := map[string]bool{}
	for object := range assigned {
		if v, ok := object.(*types.Var); ok && v.Name() != "" {
			out[v.Name()] = true
		}
	}
	return out
}

// ownershipFacts reports what `name` undergoes inside `body`.
//
// A nil body — an interface method, an external declaration — yields effect_unknown ALONE, which
// is the correct answer rather than an absent one: nothing was proven, and "no facts" must not
// read as "no mutation".
func ownershipFacts(body *ast.BlockStmt, name string, ctx *extractCtx) []string {
	return ownershipFactsSeen(body, name, ctx, map[types.Object]bool{})
}

// ownershipFactsSeen is [ownershipFacts] with the call stack it has already entered.
//
// `seen` is what makes following calls terminate. A callee already on the stack yields
// `effect_unknown` rather than a fixpoint: the honest answer for a pointer whose fate depends on
// itself is that nothing was proven, and iterating to a least fixpoint would claim more than this
// pass can defend.
func ownershipFactsSeen(
	body *ast.BlockStmt,
	name string,
	ctx *extractCtx,
	seen map[types.Object]bool,
) []string {
	if name == "" || name == "_" {
		return nil
	}
	if body == nil {
		return []string{flagEffectUnknown}
	}

	var mutated, escapes, unknown bool

	ast.Inspect(body, func(n ast.Node) bool {
		switch typed := n.(type) {
		case *ast.AssignStmt:
			for _, lhs := range typed.Lhs {
				if rootIdent(lhs) == name && lhs != nil {
					// `x = ..` rebinds the local; `x.f = ..` or `*x = ..` writes THROUGH it.
					if _, plain := lhs.(*ast.Ident); !plain {
						mutated = true
					}
				}
			}
			for _, rhs := range typed.Rhs {
				if _, plain := rhs.(*ast.Ident); plain && rootIdent(rhs) == name {
					// Stored somewhere this pass does not track.
					escapes = true
				}
			}

		case *ast.IncDecStmt:
			if rootIdent(typed.X) == name {
				if _, plain := typed.X.(*ast.Ident); !plain {
					mutated = true
				}
			}

		case *ast.ReturnStmt:
			for _, result := range typed.Results {
				// Only the POINTER escaping counts. `return c` hands the pointer out; `return
				// c.total` returns a copy of a field and the pointer dies with the call, so
				// rooting the check at the identifier would report every reader as an escape.
				if ident, plain := result.(*ast.Ident); plain && ident.Name == name {
					escapes = true
				}
			}

		case *ast.CallExpr:
			// Passing it onward is answered by asking what the CALLEE does to the parameter it
			// lands in. Only where the callee is in this corpus with a body to read: anything else
			// leaves the facts unproven, because nothing was read and "not read" must not become
			// "not mutated".
			// The RECEIVER is where a method call's effect lands, and it is not in `Args` at
			// all — so a `s.helper()` reached the loop below with nothing rooted at `s` and was
			// silently treated as though the pointer had not been passed anywhere.
			if receiverFacts, ok := calleeReceiverFacts(typed, name, ctx, seen); ok {
				for _, fact := range receiverFacts {
					switch fact {
					case flagMutated:
						mutated = true
					case flagEscapes:
						escapes = true
					case flagEffectUnknown:
						unknown = true
					}
				}
			}
			for index, arg := range typed.Args {
				if rootIdent(arg) != name {
					continue
				}
				// A VALUE argument cannot carry an effect back. `d.v1` is a `uint64` -- the call
				// receives a COPY, and nothing it does can reach `d`. Asking the callee about it
				// made `appendUint64(b, d.v1)` poison the receiver of every method that marshals,
				// because that callee passes its own value parameter to a foreign call and the
				// analysis stopped there. What it stopped at could not have mattered.
				//
				// The TYPE decides, not the expression: `d.mem[:d.n]` roots at `d` too and is a
				// slice, which aliases the receiver and is still asked about.
				if !carriesEffect(ctx.info.TypeOf(arg)) {
					continue
				}
				inner, ok := calleeParameterFacts(typed, index, ctx, seen)
				if !ok {
					unknown = true
					continue
				}
				for _, fact := range inner {
					switch fact {
					case flagMutated:
						mutated = true
					case flagEscapes:
						escapes = true
					case flagEffectUnknown:
						unknown = true
					}
				}
			}

		case *ast.FuncLit:
			// A closure that mentions it can outlive the call and can mutate through it, and when
			// it runs is not decidable here.
			if mentions(typed.Body, name) {
				escapes = true
				unknown = true
			}

		case *ast.UnaryExpr:
			// Taking its address hands out an alias this pass cannot follow.
			if typed.Op == token.AND && rootIdent(typed.X) == name {
				escapes = true
			}
		}
		return true
	})

	facts := make([]string, 0, 3)
	if escapes {
		facts = append(facts, flagEscapes)
	}
	if unknown {
		facts = append(facts, flagEffectUnknown)
	}
	if mutated {
		facts = append(facts, flagMutated)
	}
	sort.Strings(facts)
	return facts
}

// rootIdent returns the identifier an expression is rooted at: `x`, `x.f`, `*x`, `x[i]` all root
// at `x`. Empty when the expression is not rooted at a plain identifier.
func rootIdent(expr ast.Expr) string {
	for {
		switch typed := expr.(type) {
		case *ast.Ident:
			return typed.Name
		case *ast.SelectorExpr:
			expr = typed.X
		case *ast.StarExpr:
			expr = typed.X
		case *ast.IndexExpr:
			expr = typed.X
		case *ast.ParenExpr:
			expr = typed.X
		default:
			return ""
		}
	}
}

func mentions(n ast.Node, name string) bool {
	found := false
	ast.Inspect(n, func(node ast.Node) bool {
		if ident, ok := node.(*ast.Ident); ok && ident.Name == name {
			found = true
		}
		return !found
	})
	return found
}

// isPointerReceiver reports whether sig is bound through a pointer receiver.
func isPointerReceiver(sig *types.Signature) bool {
	recv := sig.Recv()
	if recv == nil {
		return false
	}
	_, pointer := types.Unalias(recv.Type()).(*types.Pointer)
	return pointer
}

// carriesEffect reports whether a value of this type can carry an effect back to what it came from.
//
// A copy is all the callee gets, so only a type holding a REFERENCE to something else can let it
// reach the caller's value: a pointer, a slice, a map, a channel, a function, an interface, or an
// unsafe pointer. An integer cannot, and neither can a struct or an array made only of things that
// cannot -- the copy is complete.
//
// Conservative on anything unrecognised: an unknown type is assumed to carry, because being wrong
// that way costs a refusal and being wrong the other way costs a borrow chosen on a fact that was
// never true.
func carriesEffect(t types.Type) bool {
	return carriesEffectSeen(t, map[types.Type]bool{})
}

func carriesEffectSeen(t types.Type, seen map[types.Type]bool) bool {
	if t == nil {
		return true
	}
	if seen[t] {
		// A type reaching itself does so through a pointer, which has already answered yes.
		return false
	}
	seen[t] = true
	switch typed := t.(type) {
	case *types.Basic:
		return typed.Kind() == types.UnsafePointer
	case *types.Named:
		return carriesEffectSeen(typed.Underlying(), seen)
	case *types.Array:
		return carriesEffectSeen(typed.Elem(), seen)
	case *types.Struct:
		for i := 0; i < typed.NumFields(); i++ {
			if carriesEffectSeen(typed.Field(i).Type(), seen) {
				return true
			}
		}
		return false
	case *types.Pointer, *types.Slice, *types.Map, *types.Chan, *types.Signature, *types.Interface:
		return true
	default:
		return true
	}
}
