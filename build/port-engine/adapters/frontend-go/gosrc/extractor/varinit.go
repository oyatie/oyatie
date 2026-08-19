package main

import (
	"go/ast"
	"go/token"
	"go/types"
)

// What a package-scope `var` is INITIALISED TO.
//
// A `const` records its value and a `var` recorded nothing — type and documentation only. So the
// 67 package variables across the surveyed corpora reached the engine as names with no content,
// and no rule could have emitted anything for them whatever the policy turned out to be. The
// deferral of `var` reads as one decision and is three stacked gaps; this closes the first.
//
// The value is recorded as a CHILD EXPRESSION rather than as a source-text attribute, unlike a
// const's. A constant's value is a literal the target can re-parse; a variable's is arbitrary code
// — `errors.New("...")`, `sync.Pool{...}`, a call into another package — and flattening that to
// text would hand the transform a string no rule can inspect and no resolver can qualify.
//
// ABSENT means the source wrote no initialiser and the zero value applies. That is a different
// fact from an initialiser the front end could not attribute, so the second is recorded as an
// `unsupported` child rather than left absent: `var a, b = f()` gives two names one value, and
// dropping it would make the pair indistinguishable from `var a, b T`. No package in the surveyed
// corpora writes that shape, which is exactly why it must be recorded rather than assumed away.

// varInit is what one name is initialised to.
type varInit struct {
	// expr is the initialising expression, or nil when several names share one value and it
	// cannot be attributed to this name alone.
	expr ast.Expr
	// spec is the declaration the initialiser came from, so a refusal can name its position.
	spec *ast.ValueSpec
}

// indexVarInitializers keys every package-scope variable's initialiser by its object.
//
// Keyed by object rather than by name because that is what the declaration walk holds, and because
// a name alone would not survive a package whose scope and file set disagree.
func indexVarInitializers(files []*ast.File, tpkg *types.Package) map[types.Object]varInit {
	out := map[types.Object]varInit{}
	for _, file := range files {
		for _, decl := range file.Decls {
			gen, ok := decl.(*ast.GenDecl)
			if !ok || gen.Tok != token.VAR {
				continue
			}
			for _, spec := range gen.Specs {
				value, ok := spec.(*ast.ValueSpec)
				if !ok || len(value.Values) == 0 {
					continue
				}
				indexValueSpec(value, tpkg, out)
			}
		}
	}
	return out
}

// indexValueSpec pairs the names of one spec with the values beside them.
func indexValueSpec(
	spec *ast.ValueSpec,
	tpkg *types.Package,
	out map[types.Object]varInit,
) {
	// Go pairs names with values positionally when the counts agree. When they do not, one call's
	// several results fill several names, and no single expression belongs to any one of them.
	paired := len(spec.Values) == len(spec.Names)
	for index, name := range spec.Names {
		obj := tpkg.Scope().Lookup(name.Name)
		if obj == nil {
			continue
		}
		if paired {
			out[obj] = varInit{expr: spec.Values[index], spec: spec}
			continue
		}
		out[obj] = varInit{spec: spec}
	}
}

// initializerNode records what a variable is initialised to, or names the shape it could not read.
func initializerNode(init varInit, ctx *extractCtx) node {
	if init.expr != nil {
		return expressionNode(init.expr, ctx)
	}
	return unsupportedNode(init.spec)
}

// Whether a package variable is ever WRITTEN, anywhere in the package.
//
// The second of the stacked gaps behind the `var` deferral. The pack declines package variables
// because Rust's `static` is immutable, `static mut` is unsafe, and `OnceLock`/`Mutex` each pick a
// synchronization policy the source never stated — but that argument only bites for a variable the
// program actually assigns to. One that is initialised and never written again is a constant with
// a computed value, and the target has an ordinary form for it.
//
// Nothing computed which kind each variable was, so every one of them was deferred on the hardest
// case. This observes it. Package-WIDE rather than intra-procedural, because a package variable is
// visible to every function in the package and a write from any of them counts — which is also why
// the analysis stops at the package boundary and says nothing about an exported variable another
// package may write.

// packageVarWrites reports which package-scope variables some function in the package assigns to.
func packageVarWrites(
	files []*ast.File,
	info *types.Info,
	tpkg *types.Package,
) map[types.Object]bool {
	scope := tpkg.Scope()
	written := map[types.Object]bool{}
	mark := func(expr ast.Expr) {
		ident, ok := expr.(*ast.Ident)
		if !ok {
			return
		}
		obj := info.Uses[ident]
		if obj == nil {
			return
		}
		// Package scope only. A local shadowing the name is a different object, and go/types
		// already told us which one this is.
		if v, ok := obj.(*types.Var); ok && scope.Lookup(v.Name()) == obj {
			written[obj] = true
		}
	}

	for _, file := range files {
		ast.Inspect(file, func(n ast.Node) bool {
			switch typed := n.(type) {
			case *ast.AssignStmt:
				// `=` writes; `:=` at package scope is not legal, and inside a body it binds a
				// local that shadows rather than writing the global.
				if typed.Tok != token.DEFINE {
					for _, lhs := range typed.Lhs {
						mark(lhs)
					}
				}
			case *ast.IncDecStmt:
				mark(typed.X)
			case *ast.UnaryExpr:
				// Taking the address hands out a licence to write through it, and the write may be
				// anywhere. Conservative here costs a synchronization policy; being wrong costs a
				// program that silently stops sharing state.
				if typed.Op == token.AND {
					mark(typed.X)
				}
			}
			return true
		})
	}
	return written
}
