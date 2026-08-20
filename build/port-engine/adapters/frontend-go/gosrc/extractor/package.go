package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"path/filepath"
	"sort"
)

// One package: parse, type-check, and index what the declaration walk needs.
//
// Bodies and documentation are indexed by the `types.Object` they belong to, because a declaration
// built from go/types has an object and no syntax, and matching by source position would break the
// moment a declaration moves.

func extractPackage(
	dir string,
	unitID string,
	resolver types.Importer,
	cfg *buildConfig,
) ([]node, []satisfaction, *types.Package, error) {
	fset := token.NewFileSet()

	// The configuration decides which files exist, and it sorts them: go/types' object ordering
	// follows parse order, so an unsorted listing would make the snapshot a property of the
	// filesystem rather than of the source.
	names, err := selectDirFiles(dir, cfg)
	if err != nil {
		return nil, nil, nil, err
	}

	files := make([]*ast.File, 0, len(names))
	for _, name := range names {
		// ParseComments is REQUIRED for the doc extraction below: without it every `Doc` field is
		// nil and the documentation is dropped in silence, which is the exact loss this pass
		// exists to stop.
		file, err := parser.ParseFile(
			fset,
			filepath.Join(dir, name),
			nil,
			parser.ParseComments|parser.SkipObjectResolution,
		)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("parse %s: %w", name, err)
		}
		files = append(files, file)
	}

	// GoVersion is pinned to the same release the constraints resolved against. Left unset,
	// go/types checks at whatever version compiled this extractor — so a corpus using newer
	// syntax than the declared configuration would type-check anyway, and the snapshot would
	// describe a program the declared target cannot build.
	conf := types.Config{Importer: resolver, GoVersion: cfg.goVersion()}
	info := &types.Info{
		Uses: map[*ast.Ident]types.Object{},
		// Defs is what lets a function's own name resolve to its signature, which the
		// satisfaction walk needs in order to attribute a `return` to the result it fills.
		Defs: map[*ast.Ident]types.Object{},
		// Types is what lets a composite literal report WHAT it constructs. Without it the
		// literal's own type would have to be re-derived from its syntax, which is exactly the
		// re-derivation go/types exists to avoid.
		Types: map[ast.Expr]types.TypeAndValue{},
	}
	tpkg, err := conf.Check(unitID, fset, files, info)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("type-check: %w", err)
	}

	// Index every function and method body — and every declaration's documentation — by the
	// object it belongs to, so a declaration built from go/types can find both.
	bodies := map[types.Object]*ast.BlockStmt{}
	docs := map[types.Object]string{}
	fieldDocs := map[string]string{}
	for _, file := range files {
		for _, decl := range file.Decls {
			switch typed := decl.(type) {
			case *ast.FuncDecl:
				if typed.Name == nil {
					continue
				}
				// A declaration's own name is a DEFINITION. Reading `Uses` first and falling back
				// to a package-scope lookup worked for everything addressable and silently missed
				// `init`, which go/types deliberately keeps out of package scope — so its body was
				// never indexed and its code never reached the model at all.
				obj := info.Defs[typed.Name]
				if obj == nil {
					obj = info.Uses[typed.Name]
				}
				if obj == nil {
					obj = lookupFuncObject(tpkg, typed)
				}
				if obj == nil {
					continue
				}
				if typed.Body != nil {
					bodies[obj] = typed.Body
				}
				if text := commentText(typed.Doc); text != "" {
					docs[obj] = text
				}
			case *ast.GenDecl:
				indexGenDeclDocs(typed, tpkg, docs, fieldDocs)
			}
		}
	}

	// Render types relative to the package under extraction: local names stay bare, and
	// anything from elsewhere keeps its full path, so the rule pack's type map can tell a
	// local `Point` from an imported one.
	qualify := func(other *types.Package) string {
		if other == tpkg {
			return ""
		}
		return other.Path()
	}

	ctx := &extractCtx{
		qualify:    qualify,
		info:       info,
		bodies:     bodies,
		docs:       docs,
		fieldDocs:  fieldDocs,
		varInits:   indexVarInitializers(files, tpkg),
		unsafeOnly: indexUnsafeOnlyTypes(files, info, tpkg),
	}
	ctx.varWrites, ctx.varInitOnly = packageVarWrites(files, info, tpkg)
	ctx.initAssignments = indexInitAssignments(files, tpkg)

	scope := tpkg.Scope()
	objNames := scope.Names() // go/types returns these sorted
	// The scope's own order is alphabetical, and a package emitted in it reads like a symbol table
	// rather than like something someone wrote: the constructor lands wherever its name falls, and
	// two reviewers named that as a tell. SOURCE ORDER is what an author chose, and it is just as
	// deterministic here — the files are parsed in sorted order and each file's declarations are
	// walked in the order they appear.
	rank := sourceOrder(files)
	sort.SliceStable(objNames, func(i, j int) bool {
		return rank(objNames[i]) < rank(objNames[j])
	})
	decls := make([]node, 0, len(objNames))
	for _, name := range objNames {
		decl, err := declFor(scope.Lookup(name), ctx)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("declaration %s: %w", name, err)
		}
		decls = append(decls, decl)
	}
	if initializer := packageInit(files, ctx); initializer != nil {
		decls = append(decls, *initializer)
	}
	satisfactions := collectSatisfactions(files, info, unitID)
	satisfactions = append(satisfactions, structuralSatisfactions(tpkg, unitID)...)
	return decls, satisfactions, tpkg, nil
}

// sourceOrder ranks each package-scope name by where the source DECLARES it.
//
// Files first, in the sorted order they were parsed in, then declarations within a file in the
// order they appear. A name the walk never reaches — one go/types synthesises — ranks last, and
// the caller's stable sort leaves those in the alphabetical order they arrived in, so the result is
// total and deterministic either way.
func sourceOrder(files []*ast.File) func(string) int {
	order := map[string]int{}
	next := 0
	record := func(name string) {
		if name == "" || name == "_" {
			return
		}
		if _, seen := order[name]; !seen {
			order[name] = next
			next++
		}
	}
	for _, file := range files {
		for _, decl := range file.Decls {
			switch typed := decl.(type) {
			case *ast.FuncDecl:
				// A METHOD is not a package-scope name; it is carried by the type it is on, and
				// ranking by its own position would move the type.
				if typed.Recv == nil && typed.Name != nil {
					record(typed.Name.Name)
				}
			case *ast.GenDecl:
				for _, spec := range typed.Specs {
					switch s := spec.(type) {
					case *ast.TypeSpec:
						record(s.Name.Name)
					case *ast.ValueSpec:
						for _, name := range s.Names {
							record(name.Name)
						}
					}
				}
			}
		}
	}
	// A name the walk never reached ranks AFTER everything it did. A bare map lookup would answer
	// zero and sort it first, which is the opposite.
	unreached := next
	return func(name string) int {
		if rank, seen := order[name]; seen {
			return rank
		}
		return unreached
	}
}

// packageInit records every `func init()` in the package as ONE declaration.
//
// go/types keeps `init` out of package scope on purpose — it is not addressable, several may exist,
// and only the runtime calls them — so the scope walk above cannot see it and the code was reaching
// the model nowhere. A construct that is neither translated nor refused is the one outcome this
// engine has no answer for.
//
// One declaration rather than several, carrying the bodies in FILE ORDER, because that order is a
// guarantee the source makes: several inits run in the order their files are presented. Splitting
// them into separate declarations would hand that order to a name sort.
func packageInit(files []*ast.File, ctx *extractCtx) *node {
	out := node{Kind: kindPackageInit, Name: "init"}
	for _, file := range files {
		for _, decl := range file.Decls {
			fn, ok := decl.(*ast.FuncDecl)
			if !ok || fn.Recv != nil || fn.Name == nil || fn.Name.Name != "init" || fn.Body == nil {
				continue
			}
			inner := *ctx
			inner.assigned = assignedLocals(fn.Body, ctx)
			inner.reread = rereadBindings(fn.Body, ctx)
			out.Children = append(out.Children, bodyNode(fn.Body, &inner))
		}
	}
	if len(out.Children) == 0 {
		return nil
	}
	return &out
}

// extractCtx carries what body and doc extraction need alongside the type qualifier.
type extractCtx struct {
	qualify types.Qualifier
	info    *types.Info
	bodies  map[types.Object]*ast.BlockStmt
	docs    map[types.Object]string
	// assigned holds every local object the enclosing body ASSIGNS TO after binding it. A binding
	// the body never writes again needs nothing from the target; one it does write needs to say so,
	// and which it is cannot be told from the binding itself.
	assigned map[types.Object]bool
	// receiver is the name the enclosing method binds its receiver to, so an identifier that
	// refers to it can be marked as such. Without it `c.total` and `other.total` are the same
	// shape and only one of them is `self`.
	receiver string
	// varInits is what each package-scope variable is initialised to, keyed by object. A `const`
	// records its value and a `var` recorded nothing, so every package variable reached the engine
	// as a name with no content.
	varInits map[types.Object]varInit
	// unsafeOnly names this package's own types whose every reference is inside the source's
	// `unsafe.Pointer` escape hatch. Such a type describes the source runtime's memory layout,
	// which the target does not share, so it is refused rather than ported. See unsafeuse.go.
	unsafeOnly map[types.Object]bool
	// reread names the bindings the enclosing body reads MORE THAN ONCE. The source copies a
	// value on every read and the target moves it, so a second read of a non-copying binding is a
	// use after move — and a binding read once is moved, which is what someone would write.
	reread map[types.Object]int
	// varWrites names the package-scope variables some function in the package assigns to. A
	// variable that is initialised and never written again is a constant with a computed value;
	// only the ones that ARE written need the synchronization policy the deferral is about.
	varWrites map[types.Object]bool
	// varInitOnly names the subset of those whose every write is in the package initialiser. Such a
	// variable is computed once before anything runs and never changes after, which is not the
	// mutable global the deferral is about — a different fact, and so a different target form.
	varInitOnly map[types.Object]bool
	// initAssignments keys a package variable by what the package INITIALISER assigns it. go/types
	// omits `init` from package scope, so without this the engine could see THAT a variable is
	// computed and never with what.
	initAssignments map[types.Object]ast.Expr
	// fieldDocs is keyed by "TypeName.FieldName": a struct field is not a package-scope object, so
	// it has no types.Object to index by, and matching by position would break the moment a field
	// moves.
	fieldDocs map[string]string
}

// lookupFuncObject finds the types.Object for a declared function or method.
func lookupFuncObject(tpkg *types.Package, fn *ast.FuncDecl) types.Object {
	if fn.Recv == nil || len(fn.Recv.List) == 0 {
		return tpkg.Scope().Lookup(fn.Name.Name)
	}
	recvName := receiverTypeName(fn.Recv.List[0].Type)
	if recvName == "" {
		return nil
	}
	obj := tpkg.Scope().Lookup(recvName)
	if obj == nil {
		return nil
	}
	named, ok := obj.Type().(*types.Named)
	if !ok {
		return nil
	}
	for i := 0; i < named.NumMethods(); i++ {
		if named.Method(i).Name() == fn.Name.Name {
			return named.Method(i)
		}
	}
	return nil
}

func receiverTypeName(expr ast.Expr) string {
	switch typed := expr.(type) {
	case *ast.Ident:
		return typed.Name
	case *ast.StarExpr:
		return receiverTypeName(typed.X)
	case *ast.IndexExpr:
		return receiverTypeName(typed.X)
	case *ast.IndexListExpr:
		return receiverTypeName(typed.X)
	default:
		return ""
	}
}
