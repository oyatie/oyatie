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

// indexInitAssignments keys a package variable by the expression the package INITIALISER assigns it.
//
// `go/types` omits `init` from package scope, so the engine could see THAT such a variable is
// computed and never WITH WHAT -- the third of three stacked gaps the plan names for package-scope
// variables, after the missing initialiser and the missing write analysis.
//
// Only where the initialiser assigns it EXACTLY ONCE, and only a plain `=`. A variable the
// initialiser writes twice has no single expression that is its value, and a read-modify-write reads
// a value from before the initialiser ran, which is the zero and is a different question.
func indexInitAssignments(files []*ast.File, tpkg *types.Package) map[types.Object]ast.Expr {
	found := map[types.Object]ast.Expr{}
	twice := map[types.Object]bool{}
	scope := tpkg.Scope()
	for _, file := range files {
		for _, decl := range file.Decls {
			fn, isFunc := decl.(*ast.FuncDecl)
			if !isFunc || !isPackageInit(fn) {
				continue
			}
			ast.Inspect(fn.Body, func(n ast.Node) bool {
				assign, ok := n.(*ast.AssignStmt)
				if !ok || assign.Tok != token.ASSIGN || len(assign.Lhs) != len(assign.Rhs) {
					return true
				}
				for at, lhs := range assign.Lhs {
					ident, ok := lhs.(*ast.Ident)
					if !ok {
						continue
					}
					obj := scope.Lookup(ident.Name)
					if obj == nil || obj != tpkg.Scope().Lookup(ident.Name) {
						continue
					}
					if _, seen := found[obj]; seen {
						twice[obj] = true
						continue
					}
					found[obj] = assign.Rhs[at]
				}
				return true
			})
		}
	}
	for obj := range twice {
		delete(found, obj)
	}
	return found
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
			// CONSTANTS too, and for a reason that is not the variable's. A constant's VALUE is
			// already known -- go/types folded it -- so the engine could always emit it and be
			// right. What the folding throws away is the author's DERIVATION: `marshaledSize =
			// len(magic) + 8*5 + 32` becomes `76`, and two reviewers in a row named that literal as
			// evidence a translator had evaluated an expression a person would have kept. The
			// expression is the better emit where the target can spell it, and the folded value
			// remains correct where it cannot.
			if !ok || (gen.Tok != token.VAR && gen.Tok != token.CONST) {
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

// packageVarWrites reports which package-scope variables some function assigns to, and which of
// those are assigned ONLY by the package initialiser.
func packageVarWrites(
	files []*ast.File,
	info *types.Info,
	tpkg *types.Package,
) (map[types.Object]bool, map[types.Object]bool) {
	scope := tpkg.Scope()
	written := map[types.Object]bool{}
	initOnly := map[types.Object]bool{}

	// WHERE the write happens, not only whether. A variable every write to which is a package
	// initialiser is not a mutable global at all -- it is computed once before anything runs and
	// never changes after -- and that is a different target form from one an ordinary function
	// assigns to at run time. Walking per function declaration is the only way to tell them apart;
	// walking the file says both happened somewhere.
	inits := map[types.Object]bool{}
	outside := map[types.Object]bool{}
	for _, file := range files {
		for _, decl := range file.Decls {
			fn, isFunc := decl.(*ast.FuncDecl)
			into := outside
			if isFunc && isPackageInit(fn) {
				into = inits
			}
			mark := marker(scope, info, written, into)
			ast.Inspect(decl, func(n ast.Node) bool {
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
					// Taking the address hands out a licence to write through it, and the write may
					// be anywhere. Conservative here costs a synchronization policy; being wrong
					// costs a program that silently stops sharing state.
					if typed.Op == token.AND {
						mark(typed.X)
					}
				case *ast.SliceExpr:
					// A SLICE of a package array is the same licence by another spelling: it is a
					// mutable view, and `io.ReadFull(r, pool[:])` fills the array through it without
					// ever assigning to the name. That is how a randomness pool read as a constant.
					mark(typed.X)
				}
				return true
			})
		}
	}
	for obj := range inits {
		if !outside[obj] {
			initOnly[obj] = true
		}
	}
	return written, initOnly
}

// isPackageInit reports whether this declaration is the package initialiser Go runs before main.
//
// Named `init`, no receiver, no parameters and no results -- all four, because a method called
// `init` is an ordinary method and a function called `init` that takes an argument is not the
// initialiser either. go/types omits it from package scope, so it can only be recognised here.
func isPackageInit(fn *ast.FuncDecl) bool {
	if fn.Name == nil || fn.Name.Name != "init" || fn.Recv != nil || fn.Body == nil {
		return false
	}
	params := fn.Type.Params != nil && len(fn.Type.Params.List) > 0
	results := fn.Type.Results != nil && len(fn.Type.Results.List) > 0
	return !params && !results
}

// marker records a write into both the overall set and the set for where it was found.
func marker(
	scope *types.Scope,
	info *types.Info,
	written map[types.Object]bool,
	into map[types.Object]bool,
) func(ast.Expr) {
	return func(expr ast.Expr) {
		// THROUGH the expression to the variable it reaches. `pool[i] = x` writes `pool`, and
		// `cfg.field = x` writes `cfg`; stopping at the outermost node saw neither, so a package
		// array something fills element by element read as never written -- and a never-written
		// variable becomes a constant, which cannot be written at all.
		ident, ok := baseIdent(expr).(*ast.Ident)
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
			into[obj] = true
		}
	}
}

// baseIdent walks an lvalue down to the variable it ultimately reaches.
//
// An assignment's target is rarely a bare name: `pool[i]`, `cfg.field`, `*p`, and any nesting of
// them all write the variable at the bottom. Anything else -- a call, a literal -- reaches no
// variable, and the caller's type assertion rejects it.
func baseIdent(expr ast.Expr) ast.Expr {
	for {
		switch typed := expr.(type) {
		case *ast.IndexExpr:
			expr = typed.X
		case *ast.SelectorExpr:
			expr = typed.X
		case *ast.StarExpr:
			expr = typed.X
		case *ast.ParenExpr:
			expr = typed.X
		default:
			return expr
		}
	}
}

// packageConstUses names, for each UNTYPED package constant, the single type its uses agree on.
//
// An untyped constant has no type in the source until it is used, and it takes a DIFFERENT one at
// each use: `const offset = 10` is a byte in `offset + (digit - 'A')` and an int in `n + offset`.
// The target has no such thing — a `const` must state one type at its declaration.
//
// `types.Default` answers "what type does this take when it must have one", which is the right
// answer when there is no other evidence. Where the uses AGREE it is the wrong one: ksuid's
// `offsetUppercase` defaulted to the source's `int`, and every use of it was byte arithmetic, so
// the emitted constant could not be added to anything it was declared to serve.
//
// Only unanimity counts. A constant used at two different types has no single target declaration —
// the engine already records that cost for body-scoped constants — so disagreement yields nothing
// here and the default stands, which fails to compile rather than silently picking one use over
// another.
func packageConstUses(files []*ast.File, info *types.Info, tpkg *types.Package) map[types.Object]types.Type {
	scope := tpkg.Scope()
	agreed := map[types.Object]types.Type{}
	conflicted := map[types.Object]bool{}
	for _, file := range files {
		ast.Inspect(file, func(n ast.Node) bool {
			ident, ok := n.(*ast.Ident)
			if !ok {
				return true
			}
			konst, ok := info.Uses[ident].(*types.Const)
			if !ok || scope.Lookup(konst.Name()) != konst {
				return true
			}
			basic, ok := konst.Type().(*types.Basic)
			if !ok || basic.Info()&types.IsUntyped == 0 {
				return true
			}
			at := info.Types[ident].Type
			if at == nil {
				return true
			}
			if used, isBasic := at.(*types.Basic); !isBasic || used.Info()&types.IsUntyped != 0 {
				return true
			}
			// A use at the constant's DEFAULT type is not evidence of its type. Go applies the
			// default exactly where the context constrains nothing -- an `interface{}` parameter
			// being the common case, and `fmt.Errorf("%d", ProtocolVersionMax)` being the one that
			// cost a real answer here. Counting it made `ProtocolVersionMax` disagree with
			// `ProtocolVersionMin uint8` and fall back to the default anyway, so the pair that
			// bounds one inclusive range came out `i64` and `u8` and could not be compared.
			//
			// Excluding it can only change the answer when some OTHER use constrains the value,
			// and that use is the one carrying information. Where every use is the default, there
			// is no agreement to find and the default still stands.
			if types.Identical(at, types.Default(konst.Type())) {
				return true
			}
			switch previous, seen := agreed[konst]; {
			case !seen:
				agreed[konst] = at
			case !types.Identical(previous, at):
				conflicted[konst] = true
			}
			return true
		})
	}
	for obj := range conflicted {
		delete(agreed, obj)
	}
	return agreed
}
