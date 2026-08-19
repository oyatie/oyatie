package main

import (
	"fmt"
	"go/ast"
	"go/types"
	"strings"
)

// Expressions, and the type facts that make them translatable.
//
// Two nodes carry a TYPE where the syntax alone would not: a selector, because reading a field is
// a copy in Go and a move in Rust, and a composite literal, because Go zero-fills the fields it
// omits and the target must name every one.

func identName(expr ast.Expr) string {
	if ident, ok := expr.(*ast.Ident); ok {
		return ident.Name
	}
	return ""
}

// expressionType reports an expression's type, when go/types recorded one.
func expressionType(expr ast.Expr, ctx *extractCtx) *typeNode {
	if tv, ok := ctx.info.Types[expr]; ok && tv.Type != nil {
		return typeTree(tv.Type)
	}
	return nil
}

func expressionNodes(exprs []ast.Expr, ctx *extractCtx) []node {
	if len(exprs) == 0 {
		return nil
	}
	out := make([]node, 0, len(exprs))
	for _, expr := range exprs {
		out = append(out, expressionNode(expr, ctx))
	}
	return out
}

func expressionNode(expr ast.Expr, ctx *extractCtx) node {
	switch typed := expr.(type) {
	case *ast.BasicLit:
		return node{
			Kind:  kindLiteral,
			Attrs: map[string]string{attrValue: typed.Value, "lit_kind": typed.Kind.String()},
		}

	case *ast.Ident:
		// What the identifier REFERS to is recorded, because the target cases each kind
		// differently and the name alone cannot say which it is — and because the RECEIVER is the
		// one identifier whose target spelling is not its name at all.
		kind := referenceKind(typed, ctx)
		if typed.Name == ctx.receiver && ctx.receiver != "" {
			kind = "receiver"
		}
		return node{
			Kind:  kindIdent,
			Name:  typed.Name,
			Attrs: map[string]string{attrRef: kind},
		}

	case *ast.ParenExpr:
		return node{Kind: kindParen, Children: []node{expressionNode(typed.X, ctx)}}

	case *ast.SelectorExpr:
		// The selector's TYPE is recorded because reading a field by value is a copy in the source
		// and a move in the target: whether that needs a clone depends on the type, and this is
		// where the type is known.
		return node{
			Kind:     kindSelector,
			Name:     typed.Sel.Name,
			Type:     expressionType(typed, ctx),
			Children: []node{expressionNode(typed.X, ctx)},
		}

	case *ast.CallExpr:
		children := []node{expressionNode(typed.Fun, ctx)}
		children = append(children, expressionNodes(typed.Args, ctx)...)
		// The callee's IDENTITY, not its spelling. `errors.New` and a local variable named
		// `errors` are the same text, and only the type-checker can tell them apart — so a rule
		// that keys on the identity answers for the real function and not for whatever shares
		// its name.
		// A CONVERSION is spelled exactly like a call and is not one. Recorded as its own kind,
		// carrying the type it converts TO, because the target has three different forms for it and
		// none of them is a function call.
		if target := conversionTarget(typed, ctx); target != nil {
			return node{
				Kind:     kindConvert,
				Type:     target,
				Children: children[1:],
			}
		}
		attrs := withAttr(nil, attrCallee, calleeIdentity(typed.Fun, ctx))
		if calleeIsMethod(typed.Fun, ctx) {
			attrs = withAttr(attrs, attrCalleeKind, calleeKindMethod)
		}
		return node{
			Kind:     kindCall,
			Attrs:    attrs,
			Children: children,
		}

	case *ast.SliceExpr:
		// A three-index slice sets the CAPACITY of the result, which the target's slicing does not
		// express at all — the capacity of a Rust slice is its length. Recorded as unsupported so
		// it refuses by name rather than silently becoming a two-index slice with a different
		// aliasing story.
		if typed.Slice3 {
			return unsupportedNode(typed)
		}
		return node{
			Kind: kindSlice,
			Children: []node{
				expressionNode(typed.X, ctx),
				sliceBound(typed.Low, ctx),
				sliceBound(typed.High, ctx),
			},
		}

	case *ast.IndexExpr:
		return node{
			Kind: kindIndex,
			Children: []node{
				expressionNode(typed.X, ctx),
				expressionNode(typed.Index, ctx),
			},
		}

	case *ast.CompositeLit:
		return compositeNode(typed, ctx)

	case *ast.BinaryExpr:
		return node{
			Kind:  kindBinary,
			Attrs: map[string]string{attrOp: typed.Op.String()},
			Children: []node{
				expressionNode(typed.X, ctx),
				expressionNode(typed.Y, ctx),
			},
		}

	case *ast.UnaryExpr:
		return node{
			Kind:     kindUnary,
			Attrs:    map[string]string{attrOp: typed.Op.String()},
			Children: []node{expressionNode(typed.X, ctx)},
		}

	default:
		return unsupportedNode(expr)
	}
}

// referenceKind classifies what an identifier resolves to, via go/types.
// calleeIdentity is the package-qualified identity of what a call resolves to, or empty when the
// callee is not a declared function — a value of function type, a conversion, a method value.
func calleeIdentity(fun ast.Expr, ctx *extractCtx) string {
	var name *ast.Ident
	switch typed := fun.(type) {
	case *ast.Ident:
		name = typed
	case *ast.SelectorExpr:
		name = typed.Sel
	default:
		return ""
	}
	obj := ctx.info.Uses[name]
	switch typed := obj.(type) {
	case *types.Builtin:
		return typed.Name()
	case *types.Func:
		// A METHOD is not nameable by package path — it is reached through a receiver, and the
		// target spells that differently from a function call. Recording an identity for it would
		// invite a rule to resolve it as a path, which is the defect this distinction exists to
		// stop; `calleeIsMethod` reports the difference instead.
		if signature, ok := typed.Type().(*types.Signature); ok && signature.Recv() != nil {
			return ""
		}
		if pkg := typed.Pkg(); pkg != nil {
			return pkg.Path() + "." + typed.Name()
		}
		return typed.Name()
	default:
		return ""
	}
}

func referenceKind(ident *ast.Ident, ctx *extractCtx) string {
	obj := ctx.info.Uses[ident]
	if obj == nil {
		// Not a use of anything the type-checker recorded: a `:=` binding's own name, or the
		// blank identifier. Both are locals as far as casing is concerned.
		return "local"
	}
	switch typed := obj.(type) {
	case *types.Nil:
		// The ABSENT value, and it needs its own classification rather than falling through to a
		// local. A failure convention is spelled by comparing against it, so `return x, nil` and
		// `return x, err` are the same shape until this distinguishes them.
		return "nil"
	case *types.Const:
		return "const"
	case *types.Func:
		return "func"
	case *types.TypeName:
		return "type"
	case *types.Builtin:
		return "builtin"
	case *types.Var:
		if typed.IsField() {
			return "field"
		}
		if typed.Parent() != nil && typed.Parent() == typed.Pkg().Scope() {
			return "package_var"
		}
		return "local"
	default:
		return "local"
	}
}

func unsupportedNode(n ast.Node) node {
	return node{
		Kind:  kindUnsupported,
		Attrs: map[string]string{attrGoNode: strings.TrimPrefix(fmt.Sprintf("%T", n), "*ast.")},
	}
}

// commentText renders a comment group as plain text, one line per source line, with the
// comment markers removed. Returns "" when there is no comment, so an undocumented declaration
// carries no attribute rather than an empty one.
