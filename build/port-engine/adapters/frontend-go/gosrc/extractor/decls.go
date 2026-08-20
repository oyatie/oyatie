package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"sort"
)

// Package-scope declarations: the shape of what a unit declares.

func declFor(obj types.Object, ctx *extractCtx) (node, error) {
	qualify := ctx.qualify
	base := node{Name: obj.Name(), Flags: flagsFor(obj.Exported(), false, false, false)}
	base.Attrs = withDoc(base.Attrs, ctx.docs[obj])

	switch typed := obj.(type) {
	case *types.Const:
		base.Kind = kindConst
		// The DEFAULT type, because a declaration in the target must have one. An untyped constant
		// has no type in the source until it is used — `const magic = "xxh"` is `untyped string` —
		// and its type node then matches nothing in the pack, so it emitted `const MAGIC: String`
		// where the constant-position override says `&str`. `types.Default` is the source's own
		// answer to "what type does this take when it must have one", which is exactly the
		// question a target declaration asks.
		// THE TYPE ITS USES AGREE ON, and `types.Default` only when they do not. See
		// packageConstUses: the default is the source's answer to "what type when it must have
		// one", which is right with no other evidence and wrong when every use in the package says
		// something else.
		// SELF-CONTAINED ONLY. Retyping a constant whose initialiser is an EXPRESSION over other
		// constants types only that constant: `const g1582ns100 = g1582 * 10000000` would be
		// declared at the type its uses agree on while `g1582` keeps the type ITS uses agree on,
		// and the target then rejects the multiplication. The source has no such problem because
		// untyped arithmetic is exact and typeless until it lands somewhere.
		//
		// Making the whole constant graph agree is a transitive problem and a different rule, so
		// the narrow one is applied where it is sound and the default stands elsewhere.
		derived := false
		if init, ok := ctx.varInits[typed]; ok {
			derived = init.expr == nil || !isLiteralExpr(init.expr)
		}
		agreed, unanimous := ctx.constUses[typed]
		if unanimous && !derived {
			base.Type = typeTree(agreed)
		} else {
			base.Type = typeTree(types.Default(typed.Type()))
		}
		if value := typed.Val(); value != nil {
			base.Attrs = withAttr(base.Attrs, attrValue, value.String())
		}
		// The author's DERIVATION, beside the folded value. Both are recorded because they answer
		// different questions: the value is what the constant IS, and always correct, while the
		// expression is what the author wrote and is only emittable where the target can spell it.
		// `marshaledSize = len(magic) + 8*5 + 32` folds to `76`, which is right and says nothing.
		if init, ok := ctx.varInits[typed]; ok {
			base.Children = []node{initializerNode(init, ctx)}
		}
		return base, nil

	case *types.Var:
		base.Kind = kindVar
		base.Type = typeTree(typed.Type())
		// ABSENT means the source wrote no initialiser and the zero value applies — a different
		// fact from one the front end could not attribute, which arrives as an `unsupported`
		// child instead of as silence.
		if init, ok := ctx.varInits[typed]; ok {
			base.Children = []node{initializerNode(init, ctx)}
		} else if assigned, ok := ctx.initAssignments[typed]; ok {
			// The package INITIALISER's assignment, which is this variable's value just as a
			// declaration's initialiser is. Recorded the same way so nothing downstream has to know
			// which of the two the source used.
			base.Children = []node{expressionNode(assigned, ctx)}
		}
		// Written somewhere in the package, so the mutability the deferral is about is real here
		// and absent elsewhere. Observed rather than assumed, in both directions.
		if ctx.varWrites[typed] {
			base.Flags = append(base.Flags, flagRebound)
			// WHERE, not only whether. A variable written only by the package initialiser is
			// computed once and never changes; one an ordinary function assigns to is a mutable
			// global. Same write flag, different fact, different target form.
			if ctx.varInitOnly[typed] {
				base.Flags = append(base.Flags, flagInitWritten)
			}
			sort.Strings(base.Flags)
		}
		return base, nil

	case *types.Func:
		sig, ok := typed.Type().(*types.Signature)
		if !ok {
			return base, fmt.Errorf("func object without signature")
		}
		base.Kind = kindFunc
		base.Flags = flagsFor(obj.Exported(), sig.Variadic(), false, false)
		base.Children = signatureChildren(sig, qualify)
		body := ctx.bodies[obj]
		assigned := map[types.Object]bool{}
		if body != nil {
			assigned = assignedLocals(body, ctx)
		}
		annotateParameterFacts(base.Children, body, reboundParameters(assigned), ctx)
		if body != nil {
			inner := *ctx
			inner.assigned = assigned
			inner.reread = rereadBindings(body, ctx)
			base.Children = append(base.Children, bodyNode(body, &inner))
		}
		return base, nil

	case *types.TypeName:
		return typeDecl(typed, base, ctx)

	default:
		return base, fmt.Errorf("unsupported object kind %T", obj)
	}
}

func typeDecl(obj *types.TypeName, base node, ctx *extractCtx) (node, error) {
	qualify := ctx.qualify
	if obj.IsAlias() {
		base.Kind = kindAlias
		// Unalias, or the alias renders as its own name: since Go 1.22 an alias is a
		// materialized *types.Alias whose String() is the alias identifier, so
		// `type ID = string` would extract as `ID -> ID` and say nothing. Unalias
		// resolves the chain to the aliased type, which is what a type map answers with.
		// This is the alias TARGET; a parameter written as `ID` still extracts as `ID`,
		// because there the alias name is what was written.
		base.Type = typeTree(types.Unalias(obj.Type()))
		return base, nil
	}

	if ctx.unsafeOnly[obj] {
		base.Flags = append(base.Flags, flagUnsafeLayoutOnly)
		sort.Strings(base.Flags)
	}

	named, ok := obj.Type().(*types.Named)
	if !ok {
		// A non-alias TypeName that is not Named is a builtin (`error`, `any`); the corpus
		// should not surface one at package scope, so refuse rather than guess.
		return base, fmt.Errorf("non-alias type name with unexpected type %T", obj.Type())
	}

	methods, err := methodChildren(named, ctx)
	if err != nil {
		return base, err
	}

	switch underlying := named.Underlying().(type) {
	case *types.Struct:
		base.Kind = kindStruct
		// Field order is declaration order and is SEMANTIC in Go (memory layout,
		// positional composite literals), so it is deliberately not sorted.
		for i := 0; i < underlying.NumFields(); i++ {
			field := underlying.Field(i)
			attrs := withDoc(nil, ctx.fieldDocs[obj.Name()+"."+field.Name()])
			// THE STRUCT TAG. It is not decoration: a tag is what a serialization library reads to
			// decide the WIRE NAME of a field and whether it may be omitted, so a type carrying one
			// has a wire contract written in its field names. Dropping the tag and casing the name
			// changes the bytes on the wire while the program still compiles -- `memberlist`'s ten
			// message structs are msgpack maps keyed by field name, and the port renamed every key.
			//
			// Recorded rather than interpreted. WHICH library reads a given tag, and what the
			// target's counterpart is, are decisions; that the tag EXISTS is a fact, and the
			// transform cannot refuse what it cannot see.
			if tag := underlying.Tag(i); tag != "" {
				attrs = withAttr(attrs, attrStructTag, tag)
			}
			base.Children = append(base.Children, node{
				Kind:  kindField,
				Name:  field.Name(),
				Type:  typeTree(field.Type()),
				Flags: flagsFor(field.Exported(), false, field.Embedded(), false),
				Attrs: attrs,
			})
		}
		base.Children = append(base.Children, methods...)
		promoted, err := promotedMethods(named, ctx)
		if err != nil {
			return base, err
		}
		base.Children = append(base.Children, promoted...)
		return base, nil

	case *types.Interface:
		base.Kind = kindInterface
		ifaceMethods := make([]node, 0, underlying.NumExplicitMethods())
		for i := 0; i < underlying.NumExplicitMethods(); i++ {
			method := underlying.ExplicitMethod(i)
			sig, ok := method.Type().(*types.Signature)
			if !ok {
				return base, fmt.Errorf("interface method %s without signature", method.Name())
			}
			ifaceMethods = append(ifaceMethods, node{
				Kind: kindMethod,
				Name: method.Name(),
				// An interface method is not a package-scope object, so its documentation is
				// indexed by member name rather than by object.
				Attrs: withDoc(nil, firstNonEmpty(
					ctx.docs[method],
					ctx.fieldDocs[obj.Name()+"."+method.Name()],
				)),
				// An interface method has no receiver to be a pointer to; the implementing type
				// decides that, and this node is the requirement rather than the binding.
				Flags:    flagsFor(method.Exported(), sig.Variadic(), false, false),
				Children: signatureChildren(sig, qualify),
			})
		}
		sortNodes(ifaceMethods)
		// Embeds come FIRST so the supertrait list is in front of the methods it constrains, and
		// they are not sorted with the methods: they are a different kind of child answering a
		// different question.
		base.Children = append(interfaceEmbeds(underlying, ctx), ifaceMethods...)
		return base, nil

	default:
		base.Kind = kindNamed
		base.Type = typeTree(underlying)
		base.Children = methods
		return base, nil
	}
}

// methodChildren returns the methods declared on named, sorted by name. Source order is
// not used: unlike struct fields, method order carries no Go semantics, and sorting keeps
// the snapshot stable against a reordering edit that changes nothing.
func methodChildren(named *types.Named, ctx *extractCtx) ([]node, error) {
	methods := make([]node, 0, named.NumMethods())
	for i := 0; i < named.NumMethods(); i++ {
		method := named.Method(i)
		sig, ok := method.Type().(*types.Signature)
		if !ok {
			return nil, fmt.Errorf("method %s without signature", method.Name())
		}
		receiverName := ""
		if recv := sig.Recv(); recv != nil {
			receiverName = recv.Name()
		}

		children := signatureChildren(sig, ctx.qualify)
		body := ctx.bodies[method]
		assigned := map[types.Object]bool{}
		if body != nil {
			assigned = assignedLocals(body, ctx)
		}
		// A method's parameters were never annotated at all — the function path did this and the
		// method path did not, so a method parameter carried no ownership facts and no rebinding.
		annotateParameterFacts(children, body, reboundParameters(assigned), ctx)
		if body != nil {
			// The body walk needs the receiver's NAME: `c.total` becomes `self.total` only if
			// something knows that `c` is the receiver and `other` is not.
			inner := *ctx
			inner.assigned = assigned
			inner.reread = rereadBindings(body, ctx)
			inner.receiver = receiverName
			children = append(children, bodyNode(body, &inner))
		}

		flags := flagsFor(method.Exported(), sig.Variadic(), false, isPointerReceiver(sig))
		flags = append(flags, ownershipFacts(ctx.bodies[method], receiverName, ctx)...)
		sort.Strings(flags)

		methods = append(methods, node{
			Kind:     kindMethod,
			Name:     method.Name(),
			Flags:    flags,
			Attrs:    withDoc(nil, ctx.docs[method]),
			Children: children,
		})
	}
	sortNodes(methods)
	return methods, nil
}

// isLiteralExpr reports whether this initialiser is a literal, possibly signed.
//
// A literal carries its own value and nothing else's, so retyping the constant it initialises
// cannot disagree with any other declaration. Anything with a NAME in it can: the name has a type
// of its own, decided by its own uses, and the two need not match.
func isLiteralExpr(expr ast.Expr) bool {
	switch typed := expr.(type) {
	case *ast.BasicLit:
		return true
	case *ast.UnaryExpr:
		return typed.Op == token.SUB && isLiteralExpr(typed.X)
	case *ast.ParenExpr:
		return isLiteralExpr(typed.X)
	default:
		return false
	}
}
