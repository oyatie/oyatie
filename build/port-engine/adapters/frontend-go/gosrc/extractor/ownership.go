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

func annotateParameterFacts(children []node, body *ast.BlockStmt, rebound map[string]bool) {
	for i := range children {
		if children[i].Kind != kindParam || children[i].Name == "" {
			continue
		}
		facts := ownershipFacts(body, children[i].Name)
		// The source makes every parameter a mutable local copy and the target makes none of them,
		// so a parameter the body assigns to has to say so. Kept apart from the ownership facts
		// above because it is a claim about the CALLEE's copy, not about the caller's value.
		if rebound[children[i].Name] {
			facts = append(facts, flagRebound)
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
func ownershipFacts(body *ast.BlockStmt, name string) []string {
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
			// Passing it onward makes every fact about it UNPROVEN: the callee may mutate through
			// it, may retain it, and this pass does not follow calls.
			for _, arg := range typed.Args {
				if rootIdent(arg) == name {
					unknown = true
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
