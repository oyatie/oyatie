package main

import (
	"fmt"
	"go/ast"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
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
) ([]node, []satisfaction, *types.Package, error) {
	fset := token.NewFileSet()

	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, nil, nil, fmt.Errorf("read dir: %w", err)
	}
	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		name := entry.Name()
		if entry.IsDir() || !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		names = append(names, name)
	}
	// Sorted parse order keeps go/types' object ordering reproducible across filesystems.
	sort.Strings(names)

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

	conf := types.Config{Importer: resolver}
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
				obj := info.Uses[typed.Name]
				if obj == nil {
					// A declaration's own name is a definition, not a use, so it is not in Uses.
					// Resolve it through the package scope or the receiver's method set.
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
		qualify:   qualify,
		info:      info,
		bodies:    bodies,
		docs:      docs,
		fieldDocs: fieldDocs,
	}

	scope := tpkg.Scope()
	objNames := scope.Names() // go/types returns these sorted
	decls := make([]node, 0, len(objNames))
	for _, name := range objNames {
		decl, err := declFor(scope.Lookup(name), ctx)
		if err != nil {
			return nil, nil, nil, fmt.Errorf("declaration %s: %w", name, err)
		}
		decls = append(decls, decl)
	}
	return decls, collectSatisfactions(files, info, unitID), tpkg, nil
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
