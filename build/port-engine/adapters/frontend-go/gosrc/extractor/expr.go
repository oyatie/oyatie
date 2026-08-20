package main

import (
	"fmt"
	"go/ast"
	"go/token"
	"go/types"
	"strconv"
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
		// The TYPE rides along only where it is needed — on a binding this body reads again — so
		// the snapshot does not carry a type on every identifier for the sake of the few that use
		// one.
		var reread []string
		var readType *typeNode
		readCount := ""
		if object := ctx.info.Uses[typed]; object != nil && ctx.reread[object] > 1 {
			reread = []string{flagReread}
			readType = typeTree(object.Type())
			// The COUNT, not just that there was more than one. A reader of this binding can move
			// it when nothing reads it afterwards, and comparing the total against the reads in
			// one construction is how that is known without liveness.
			readCount = strconv.Itoa(ctx.reread[object])
		}
		if typed.Name == ctx.receiver && ctx.receiver != "" {
			kind = "receiver"
		}
		attrs := map[string]string{attrRef: kind}
		// The package's IMPORT PATH, not the local name it was bound to. A rule that keys on a
		// package-qualified call has to key on something stable, and the local name is not: an
		// import may be aliased, so `binary.BigEndian` and `bin.BigEndian` are the same call
		// written two ways. The path is the identity go/types already knows.
		if pkg, ok := ctx.info.Uses[typed].(*types.PkgName); ok {
			attrs[attrPackagePath] = pkg.Imported().Path()
		}
		if readCount != "" {
			attrs[attrReadCount] = readCount
		}
		return node{
			Kind:  kindIdent,
			Name:  typed.Name,
			Type:  readType,
			Flags: reread,
			Attrs: attrs,
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
		// A builtin whose first argument is a TYPE, not a value. `make([]byte, 0, n)` names the
		// thing to allocate, and walking that name as an expression recorded `[]byte` as an
		// unsupported node -- which refused every declaration that allocates. The type is what the
		// call is about, so it is recorded as one.
		if index := typeArgumentIndex(typed, ctx); index >= 0 {
			for at, arg := range typed.Args {
				if at == index {
					children = append(children, node{
						Kind: kindType,
						Type: typeTree(ctx.info.TypeOf(arg)),
					})
					continue
				}
				children = append(children, expressionNode(arg, ctx))
			}
		} else {
			children = append(children, expressionNodes(typed.Args, ctx)...)
		}
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
		// A SPREAD last argument, which the source writes `f(xs...)`. It is not a flourish: it
		// passes the sequence's ELEMENTS where the plain form passes the sequence itself, and the
		// two mean different things to the same callee. Recorded because nothing else in the tree
		// distinguishes them, so `append(b, xs...)` and `append(b, x, y)` arrived identical.
		var flags []string
		if typed.Ellipsis != token.NoPos {
			flags = []string{flagSpread}
		}
		return node{
			Kind:     kindCall,
			Flags:    flags,
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
		// The RESULT TYPE decides how the operation must be spelled, and it is not recoverable
		// from the operator or from the operands' syntax. The source's signed arithmetic is
		// defined to WRAP; the target's panics on overflow in a debug build and wraps in a release
		// one. Those are three different programs, and telling them apart needs to know that this
		// `+` is on integers rather than on floats or strings.
		return node{
			Kind:  kindBinary,
			Type:  typeTree(ctx.info.Types[typed].Type),
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
		// A BODY-SCOPED constant is cased like a local, because that is what it becomes: Go's
		// untyped constant takes its type from each use and so does a target `let`, where a target
		// `const` would have to fix one at the declaration. A package-scoped or predeclared one
		// stays a constant and keeps constant casing.
		if typed.Pkg() != nil && typed.Parent() != typed.Pkg().Scope() {
			return "local"
		}
		return "const"
	case *types.Func:
		return "func"
	case *types.TypeName:
		return "type"
	case *types.Builtin:
		return "builtin"
	case *types.PkgName:
		// A PACKAGE NAME, which is not a value at all. It reached the model as a `local` and was
		// cased and emitted like one, so `binary.LittleEndian.Uint64(b)` came out as
		// `binary.little_endian.uint64(b)` — a path into a crate the emitted output does not have.
		// The engine can only refuse it once it can SEE it, which is what this says.
		return "package"
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

// typeArgumentIndex reports which argument of this call is a TYPE rather than a value, or -1.
//
// The source has a handful of builtins shaped this way: `make` and `new` name what to allocate.
// Everything else takes values, and a type appearing anywhere else is not something this front end
// has met — so it says -1 and the walker records whatever it finds, which is the honest answer.
func typeArgumentIndex(call *ast.CallExpr, ctx *extractCtx) int {
	ident := calleeIdent(call.Fun)
	if ident == nil || ctx == nil || ctx.info == nil {
		return -1
	}
	if _, isBuiltin := ctx.info.Uses[ident].(*types.Builtin); !isBuiltin {
		return -1
	}
	switch ident.Name {
	case "make", "new":
		return 0
	default:
		return -1
	}
}
