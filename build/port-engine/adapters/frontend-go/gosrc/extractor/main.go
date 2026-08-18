// Command extractor is the bootstrap Go front end for the owned deterministic port
// engine (ADR-0638 D3).
//
// It reads a Go corpus with go/parser + go/types and writes a SourceModel snapshot
// envelope as JSON. It runs OUT OF BAND ONLY: the engine's verify() path consumes the
// snapshot artifact and must never invoke a Go toolchain. The Rust side enforces that
// with architecture tests over its own library sources; nothing here is linked into the
// engine.
//
// Only the Go standard library is used. golang.org/x/tools/go/packages would give richer
// package loading and would also give this fixture module a dependency graph, a go.sum,
// and a vendoring question. The corpus is small and hermetic, so stdlib parsing is
// sufficient and buys the module's dependency-freedom.
//
// Usage:
//
//	go run ./extractor -corpus ./corpus -module oyatie.example/portengine-fixture \
//	    -out ../../port-engine-snapshot/src/fixture-snapshot-v1.json
package main

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
)

// producerBootstrapGo mirrors port_engine_frontend_go::PRODUCER_BOOTSTRAP_GO. The Rust
// decoder refuses any other identity during bootstrap admission, so drift here is a red
// at admission rather than a silent relabel.
const producerBootstrapGo = "bootstrap-go-packages-go-types"

// schemaVersion is the snapshot envelope version this extractor emits.
//
//	v0 — unit identity only
//	v1 — declaration tree, with types as flat spellings
//	v2 — declaration tree, with types as TREES
//
// v1 is not merely superseded, it is UNACCEPTABLE to the current engine: a v1 artifact cannot
// answer the questions v2 asks, and decoding one by treating each spelling as an opaque name
// would reintroduce exactly the flat-table resolution v2 exists to replace.
const schemaVersion = 2

// ---------------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------------

type snapshot struct {
	SchemaVersion  int       `json:"schema_version"`
	Language       string    `json:"language"`
	SnapshotDigest string    `json:"snapshot_digest"`
	Packages       []pkgNode `json:"packages"`
}

type pkgNode struct {
	UnitID       string `json:"unit_id"`
	Producer     string `json:"producer"`
	Declarations []node `json:"declarations"`
}

// node is one node of the declaration tree, and it is deliberately UNIFORM: a constant, a
// struct field, a function parameter and an interface method are all the same shape, and
// what distinguishes them is `kind` — a value, not a field name and not a variant.
//
// The engine's seam types are language-neutral (`LanguagePair` is data, so a second
// language pair is a second directory of rule data over one engine). A snapshot shaped as
// `fields`/`methods`/`params`/`results` would have pushed Go's declaration taxonomy into
// that seam and made the neutral API answer questions only Go asks. Here `kind`,
// `type`, and `flags` are opaque slugs the engine compares and never interprets; the rule
// pack's `captures` are what give them meaning.
type node struct {
	Kind  string    `json:"kind"`
	Name  string    `json:"name"`
	Type  *typeNode `json:"type,omitempty"`
	Flags []string  `json:"flags,omitempty"`
	// Attrs carries key->value facts that do not fit a set: a constant's value, and whatever a
	// later front end needs to record. Kept separate from Flags because the two answer different
	// questions — Flags is membership, Attrs is a value — and collapsing them would mean encoding
	// "exported" as "exported=1" and losing the distinction between an absent key and an empty one.
	Attrs    map[string]string `json:"attrs,omitempty"`
	Children []node            `json:"children,omitempty"`
}

// typeNode is one node of a type TREE. Same uniform shape as `node`: `kind` is a value, so a
// second source language needs a second rule pack rather than a second seam.
//
// `Package` is what makes a named type addressable. Without it a reference to another package's
// type is indistinguishable from a local one, and two packages declaring the same name are
// indistinguishable from each other — so a resolver silently picks one.
type typeNode struct {
	Kind    string      `json:"kind"`
	Name    string      `json:"name,omitempty"`
	Package string      `json:"package,omitempty"`
	Args    []*typeNode `json:"args,omitempty"`
}

// Type kinds. Part of the snapshot contract: the rule pack answers for each one.
const (
	typeBasic     = "basic"
	typeNamed     = "named"
	typePointer   = "pointer"
	typeSlice     = "slice"
	typeArray     = "array"
	typeMap       = "map"
	typeChan      = "chan"
	typeFunc      = "func"
	typeInterface = "interface"
	typeStruct    = "struct"
	typeTuple     = "tuple"
	typeParam     = "type_param"
	// typeUnsupported keeps the model FAITHFUL where the translator is not: a type shape with no
	// node of its own is recorded as present and refused by name downstream, rather than dropped
	// into a spelling nobody can act on.
	typeUnsupported = "unsupported"
)

// Declaration kinds. These strings are the vocabulary the rule pack's `captures` select
// on, so they are part of the snapshot contract rather than an internal detail.
const (
	kindConst     = "const"
	kindVar       = "var"
	kindFunc      = "func"
	kindStruct    = "struct"
	kindInterface = "interface"
	kindAlias     = "alias"
	kindNamed     = "named"

	kindField  = "field"
	kindMethod = "method"
	kindParam  = "param"
	kindResult = "result"

	// Body vocabulary. A function body is a `body` node whose children are statements, and a
	// statement's children are expressions — the same uniform node all the way down.
	kindBody     = "body"
	kindBlock    = "block"
	kindReturn   = "return"
	kindIf       = "if"
	kindThen     = "then"
	kindElse     = "else"
	kindLet      = "let"
	kindExprStmt = "expr_stmt"

	kindLiteral   = "literal"
	kindIdent     = "ident"
	kindBinary    = "binary"
	kindUnary     = "unary"
	kindParen     = "paren"
	kindSelector  = "selector"
	kindCall      = "call"
	kindIndex     = "index"
	kindComposite = "composite"
	kindKeyed     = "keyed"
	kindZero      = "zero"

	kindAssign = "assign"
	kindFor    = "for"
	kindCond   = "cond"
	kindPost   = "post"
	kindInit   = "init"
	kindRange  = "range"
	kindSwitch = "switch"
	kindCase   = "case"
	kindBreak  = "break"

	// kindUnsupported is how the extractor stays FAITHFUL while the engine stays fail-closed.
	// The snapshot is a model of the source, so a construct the translator cannot yet handle is
	// recorded as present rather than omitted; the transform then refuses it BY NAME. Dropping it
	// here instead would make an untranslatable function look like an empty one.
	kindUnsupported = "unsupported"
)

// Flags. Sorted on emit so the set has one spelling.
const (
	flagExported = "exported"
	flagVariadic = "variadic"
	flagEmbedded = "embedded"
	// flagPointerReceiver records that a method is bound through a pointer receiver. The engine
	// refuses to translate one rather than guess an aliasing mode, so this flag has to be
	// extracted for that refusal to be possible at all — dropping it would silently turn a
	// mutating method into a read-only one.
	flagPointerReceiver = "pointer_receiver"

	// Ownership facts, observed intra-procedurally. See ownershipFacts for what each means and
	// why the third one exists.
	flagMutated       = "mutated"
	flagEscapes       = "escapes"
	flagEffectUnknown = "effect_unknown"
)

// Attribute keys.
const (
	// attrOp is a binary or unary operator, spelled as Go source.
	attrOp = "op"
	// attrGoNode names the Go AST node an `unsupported` placeholder stands for, so a refusal can
	// say what it refused rather than only that it refused.
	attrGoNode = "go_node"
	// attrRef records what an identifier resolves to — a parameter, a constant, a function, a
	// local. Rust cases each of those differently, and an identifier alone cannot say which it is:
	// rendering a reference to `MaxRetries` as `max_retries` would be a dangling name, not a
	// style choice. go/types knows the answer, so it is recorded here rather than guessed there.
	attrRef = "ref"
	// attrDoc is the declaration's documentation block, newline-separated. Recorded because the
	// target emits it: dropping it here is a silent loss of everything the source explained about
	// itself, and no downstream check looks for prose that is simply absent.
	attrDoc = "doc"
	// attrValue is a constant's value, spelled as Go source. It is deliberately the SOURCE
	// spelling rather than a normalized number: the engine emits Rust that must parse, and a
	// Go literal that is not also a valid Rust literal has to fail loudly at the syn parse
	// rather than be silently rounded into something that compiles and means something else.
	attrValue = "value"
)

// ---------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------

func main() {
	corpus := flag.String("corpus", "./corpus", "directory whose subdirectories are Go packages")
	module := flag.String("module", "oyatie.example/portengine-fixture", "module path prefix for unit ids")
	root := flag.String("root", ".", "module root; unit ids are import paths relative to it")
	out := flag.String("out", "", "output file; empty writes to stdout")
	flag.Parse()

	model, err := extract(*corpus, *module, *root)
	if err != nil {
		fmt.Fprintf(os.Stderr, "extractor: %v\n", err)
		os.Exit(1)
	}

	encoded, err := render(model)
	if err != nil {
		fmt.Fprintf(os.Stderr, "extractor: %v\n", err)
		os.Exit(1)
	}

	if *out == "" {
		os.Stdout.Write(encoded)
		return
	}
	if err := os.WriteFile(*out, encoded, 0o644); err != nil {
		fmt.Fprintf(os.Stderr, "extractor: write %s: %v\n", *out, err)
		os.Exit(1)
	}
}

func render(model *snapshot) ([]byte, error) {
	// Indented JSON with a trailing newline, so the committed artifact is reviewable and
	// byte-stable. The digest is computed over the preimage below rather than over these
	// bytes, so JSON formatting is never load-bearing for identity.
	encoded, err := json.MarshalIndent(model, "", "  ")
	if err != nil {
		return nil, fmt.Errorf("marshal snapshot: %w", err)
	}
	return append(encoded, '\n'), nil
}

// ---------------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------------

func extract(corpusDir string, modulePath string, moduleRoot string) (*snapshot, error) {
	dirs, err := packageDirs(corpusDir)
	if err != nil {
		return nil, err
	}
	if len(dirs) == 0 {
		return nil, fmt.Errorf("corpus %s contains no Go package directory", corpusDir)
	}

	// The corpus is its own importer: an intra-corpus import resolves by type-checking the
	// referenced package here, because no module path the stdlib importer knows contains it.
	packages := map[string]string{}
	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		packages[modulePath+"/"+filepath.ToSlash(rel)] = dir
	}
	resolver := newCorpusImporter(packages)

	model := &snapshot{
		SchemaVersion: schemaVersion,
		Language:      "go",
		Packages:      make([]pkgNode, 0, len(dirs)),
	}

	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		unitID := modulePath + "/" + filepath.ToSlash(rel)

		decls, err := extractPackage(dir, unitID, resolver)
		if err != nil {
			return nil, fmt.Errorf("package %s: %w", unitID, err)
		}
		model.Packages = append(model.Packages, pkgNode{
			UnitID:       unitID,
			Producer:     producerBootstrapGo,
			Declarations: decls,
		})
	}

	// Deterministic package order regardless of filesystem walk order.
	sort.Slice(model.Packages, func(i, j int) bool {
		return model.Packages[i].UnitID < model.Packages[j].UnitID
	})

	model.SnapshotDigest = digest(preimage(model))
	return model, nil
}

// corpusImporter resolves an import to a package inside the corpus, and defers to the stdlib
// importer for anything else.
//
// Memoised, and memoised on the PACKAGE rather than on the check: a diamond import would otherwise
// type-check the shared dependency twice and produce two distinct `types.Package` values for one
// package, so a cross-package type would compare unequal to itself.
type corpusImporter struct {
	dirs     map[string]string
	resolved map[string]*types.Package
	fallback types.Importer
	fset     *token.FileSet
}

func newCorpusImporter(dirs map[string]string) *corpusImporter {
	fset := token.NewFileSet()
	return &corpusImporter{
		dirs:     dirs,
		resolved: map[string]*types.Package{},
		fallback: importer.ForCompiler(fset, "source", nil),
		fset:     fset,
	}
}

func (c *corpusImporter) Import(path string) (*types.Package, error) {
	if pkg, ok := c.resolved[path]; ok {
		return pkg, nil
	}
	dir, ok := c.dirs[path]
	if !ok {
		return c.fallback.Import(path)
	}

	files, err := parsePackage(c.fset, dir)
	if err != nil {
		return nil, err
	}
	conf := types.Config{Importer: c}
	pkg, err := conf.Check(path, c.fset, files, nil)
	if err != nil {
		return nil, fmt.Errorf("import %s: %w", path, err)
	}
	c.resolved[path] = pkg
	return pkg, nil
}

// parsePackage reads and parses every non-test Go file in dir, in sorted order.
func parsePackage(fset *token.FileSet, dir string) ([]*ast.File, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("read dir: %w", err)
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
		// ParseComments is REQUIRED for doc extraction: without it every `Doc` field is nil and
		// the documentation is dropped in silence.
		file, err := parser.ParseFile(
			fset,
			filepath.Join(dir, name),
			nil,
			parser.ParseComments|parser.SkipObjectResolution,
		)
		if err != nil {
			return nil, fmt.Errorf("parse %s: %w", name, err)
		}
		files = append(files, file)
	}
	return files, nil
}

// packageDirs returns every directory at or under root holding at least one .go file,
// sorted. Test files are excluded: they are not part of the translatable surface.
func packageDirs(root string) ([]string, error) {
	seen := map[string]bool{}
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			return nil
		}
		if !strings.HasSuffix(path, ".go") || strings.HasSuffix(path, "_test.go") {
			return nil
		}
		seen[filepath.Dir(path)] = true
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("walk %s: %w", root, err)
	}
	dirs := make([]string, 0, len(seen))
	for dir := range seen {
		dirs = append(dirs, dir)
	}
	sort.Strings(dirs)
	return dirs, nil
}

func extractPackage(dir string, unitID string, resolver types.Importer) ([]node, error) {
	fset := token.NewFileSet()

	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("read dir: %w", err)
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
			return nil, fmt.Errorf("parse %s: %w", name, err)
		}
		files = append(files, file)
	}

	conf := types.Config{Importer: resolver}
	info := &types.Info{
		Uses: map[*ast.Ident]types.Object{},
		// Types is what lets a composite literal report WHAT it constructs. Without it the
		// literal's own type would have to be re-derived from its syntax, which is exactly the
		// re-derivation go/types exists to avoid.
		Types: map[ast.Expr]types.TypeAndValue{},
	}
	tpkg, err := conf.Check(unitID, fset, files, info)
	if err != nil {
		return nil, fmt.Errorf("type-check: %w", err)
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
			return nil, fmt.Errorf("declaration %s: %w", name, err)
		}
		decls = append(decls, decl)
	}
	return decls, nil
}

// extractCtx carries what body and doc extraction need alongside the type qualifier.
type extractCtx struct {
	qualify types.Qualifier
	info    *types.Info
	bodies  map[types.Object]*ast.BlockStmt
	docs    map[types.Object]string
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

func declFor(obj types.Object, ctx *extractCtx) (node, error) {
	qualify := ctx.qualify
	base := node{Name: obj.Name(), Flags: flagsFor(obj.Exported(), false, false, false)}
	base.Attrs = withDoc(base.Attrs, ctx.docs[obj])

	switch typed := obj.(type) {
	case *types.Const:
		base.Kind = kindConst
		base.Type = typeTree(typed.Type())
		if value := typed.Val(); value != nil {
			base.Attrs = withAttr(base.Attrs, attrValue, value.String())
		}
		return base, nil

	case *types.Var:
		base.Kind = kindVar
		base.Type = typeTree(typed.Type())
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
		annotateParameterFacts(base.Children, body)
		if body != nil {
			base.Children = append(base.Children, bodyNode(body, ctx))
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
			base.Children = append(base.Children, node{
				Kind:  kindField,
				Name:  field.Name(),
				Type:  typeTree(field.Type()),
				Flags: flagsFor(field.Exported(), false, field.Embedded(), false),
				Attrs: withDoc(nil, ctx.fieldDocs[obj.Name()+"."+field.Name()]),
			})
		}
		base.Children = append(base.Children, methods...)
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
				Kind:  kindMethod,
				Name:  method.Name(),
				Attrs: withDoc(nil, ctx.docs[method]),
				// An interface method has no receiver to be a pointer to; the implementing type
				// decides that, and this node is the requirement rather than the binding.
				Flags:    flagsFor(method.Exported(), sig.Variadic(), false, false),
				Children: signatureChildren(sig, qualify),
			})
		}
		sortNodes(ifaceMethods)
		base.Children = ifaceMethods
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
		if body := ctx.bodies[method]; body != nil {
			// The body walk needs the receiver's NAME: `c.total` becomes `self.total` only if
			// something knows that `c` is the receiver and `other` is not.
			inner := *ctx
			inner.receiver = receiverName
			children = append(children, bodyNode(body, &inner))
		}

		flags := flagsFor(method.Exported(), sig.Variadic(), false, isPointerReceiver(sig))
		flags = append(flags, ownershipFacts(ctx.bodies[method], receiverName)...)
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

// ---------------------------------------------------------------------------------
// Bodies
// ---------------------------------------------------------------------------------
//
// The body walk is deliberately SMALL and deliberately COMPLETE. Small, because only a few
// statement and expression forms have a translation the engine can defend today. Complete,
// because everything else is still recorded — as an `unsupported` node naming the Go AST
// type it stands for — rather than dropped. A dropped construct would make an
// untranslatable function indistinguishable from an empty one, and the engine would emit a
// green, silently wrong body. Recorded, it becomes a refusal the transform can name.

func bodyNode(block *ast.BlockStmt, ctx *extractCtx) node {
	return node{Kind: kindBody, Children: statementNodes(block.List, ctx)}
}

func statementNodes(stmts []ast.Stmt, ctx *extractCtx) []node {
	if len(stmts) == 0 {
		return nil
	}
	out := make([]node, 0, len(stmts))
	for _, stmt := range stmts {
		out = append(out, statementNode(stmt, ctx))
	}
	return out
}

func statementNode(stmt ast.Stmt, ctx *extractCtx) node {
	switch typed := stmt.(type) {
	case *ast.ReturnStmt:
		return node{Kind: kindReturn, Children: expressionNodes(typed.Results, ctx)}

	case *ast.BlockStmt:
		return node{Kind: kindBlock, Children: statementNodes(typed.List, ctx)}

	case *ast.IfStmt:
		// An `if` with an init statement (`if x := f(); x != nil`) scopes a binding to the
		// condition, which Rust has no direct form for. Recorded as unsupported rather than
		// silently hoisted, because hoisting changes the binding's lifetime.
		if typed.Init != nil {
			return unsupportedNode(stmt)
		}
		children := []node{
			{Kind: "cond", Children: []node{expressionNode(typed.Cond, ctx)}},
			{Kind: kindThen, Children: statementNodes(typed.Body.List, ctx)},
		}
		if typed.Else != nil {
			children = append(children, node{
				Kind:     kindElse,
				Children: []node{statementNode(typed.Else, ctx)},
			})
		}
		return node{Kind: kindIf, Children: children}

	case *ast.AssignStmt:
		// Multi-assignment and the op-assign forms each carry a tuple-destructuring or
		// read-modify-write question that needs a rule rather than a default.
		if len(typed.Lhs) != 1 || len(typed.Rhs) != 1 {
			return unsupportedNode(stmt)
		}
		switch typed.Tok {
		case token.DEFINE:
			name, ok := typed.Lhs[0].(*ast.Ident)
			if !ok {
				return unsupportedNode(stmt)
			}
			return node{
				Kind:     kindLet,
				Name:     name.Name,
				Children: []node{expressionNode(typed.Rhs[0], ctx)},
			}
		case token.ASSIGN:
			return node{
				Kind: kindAssign,
				Children: []node{
					expressionNode(typed.Lhs[0], ctx),
					expressionNode(typed.Rhs[0], ctx),
				},
			}
		default:
			return unsupportedNode(stmt)
		}

	case *ast.ExprStmt:
		return node{Kind: kindExprStmt, Children: []node{expressionNode(typed.X, ctx)}}

	case *ast.BranchStmt:
		// `break` maps directly. `continue` does NOT, because a three-clause loop lowers to a
		// `while` whose post-statement a `continue` would skip — a different program. `goto` and
		// labelled breaks have no target form at all.
		if typed.Tok == token.BREAK && typed.Label == nil {
			return node{Kind: kindBreak}
		}
		return unsupportedNode(stmt)

	case *ast.ForStmt:
		return forNode(typed, ctx)

	case *ast.RangeStmt:
		return rangeNode(typed, ctx)

	case *ast.SwitchStmt:
		return switchNode(typed, ctx)

	default:
		return unsupportedNode(stmt)
	}
}

// forNode records a three-clause or condition-only `for`.
//
// The clauses are recorded SEPARATELY rather than pre-lowered, because which target loop they
// deserve is a translation decision: an ascending integer counter is a range, and anything else is
// a `while` whose post-statement has to run on every path.
func forNode(stmt *ast.ForStmt, ctx *extractCtx) node {
	out := node{Kind: kindFor}
	if stmt.Init != nil {
		out.Children = append(out.Children, node{
			Kind:     kindInit,
			Children: []node{statementNode(stmt.Init, ctx)},
		})
	}
	if stmt.Cond != nil {
		out.Children = append(out.Children, node{
			Kind:     kindCond,
			Children: []node{expressionNode(stmt.Cond, ctx)},
		})
	}
	if stmt.Post != nil {
		out.Children = append(out.Children, node{
			Kind:     kindPost,
			Children: []node{statementNode(stmt.Post, ctx)},
		})
	}
	out.Children = append(out.Children, node{
		Kind:     kindThen,
		Children: statementNodes(stmt.Body.List, ctx),
	})
	return out
}

// rangeNode records a `range` loop, with the key and value names it binds.
func rangeNode(stmt *ast.RangeStmt, ctx *extractCtx) node {
	out := node{Kind: kindRange}
	out.Attrs = withAttr(out.Attrs, "key", identName(stmt.Key))
	out.Attrs = withAttr(out.Attrs, "value", identName(stmt.Value))
	out.Children = append(out.Children,
		node{Kind: "over", Children: []node{expressionNode(stmt.X, ctx)}},
		node{Kind: kindThen, Children: statementNodes(stmt.Body.List, ctx)},
	)
	return out
}

// switchNode records an expression switch.
//
// A switch with an init statement, or a TYPE switch, is not recorded as a switch at all: the first
// scopes a binding to the switch and the second dispatches on dynamic type, and neither has a
// target form that a value match reproduces.
func switchNode(stmt *ast.SwitchStmt, ctx *extractCtx) node {
	if stmt.Init != nil {
		return unsupportedNode(stmt)
	}
	out := node{Kind: kindSwitch}
	if stmt.Tag != nil {
		out.Children = append(out.Children, node{
			Kind:     "tag",
			Children: []node{expressionNode(stmt.Tag, ctx)},
		})
	}
	for _, clause := range stmt.Body.List {
		caseClause, ok := clause.(*ast.CaseClause)
		if !ok {
			return unsupportedNode(clause)
		}
		out.Children = append(out.Children, node{
			Kind: kindCase,
			Children: append(
				[]node{{Kind: "patterns", Children: expressionNodes(caseClause.List, ctx)}},
				node{Kind: kindThen, Children: statementNodes(caseClause.Body, ctx)},
			),
		})
	}
	return out
}

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

// compositeNode records a struct literal with every DECLARED field present.
//
// Go fills the fields a literal omits with their type's zero value; the target rejects an
// incomplete literal. Which fields a struct has is a fact go/types holds and the engine does not,
// so the omitted ones are recorded HERE, as `zero` nodes carrying the field's type — leaving the
// target's spelling of that zero to the rule pack.
func compositeNode(lit *ast.CompositeLit, ctx *extractCtx) node {
	fields := compositeStruct(lit, ctx)
	if fields == nil {
		// A slice, map or array literal. It reached here as a composite with no struct behind it,
		// and the previous shape emitted an empty struct literal for it — silently constructing
		// nothing. Recording it as unsupported refuses it by name instead.
		return unsupportedNode(lit)
	}

	written := make(map[string]node, len(lit.Elts))
	for _, element := range lit.Elts {
		keyed, ok := element.(*ast.KeyValueExpr)
		if !ok {
			// A POSITIONAL composite depends on field order, which the target does not reproduce
			// for a named struct — and getting it silently wrong swaps two fields of the same type
			// with no diagnostic anywhere.
			return unsupportedNode(element)
		}
		key, ok := keyed.Key.(*ast.Ident)
		if !ok {
			return unsupportedNode(keyed)
		}
		written[key.Name] = expressionNode(keyed.Value, ctx)
	}

	out := node{Kind: kindComposite, Type: compositeType(lit, ctx)}
	for index := 0; index < fields.NumFields(); index++ {
		field := fields.Field(index)
		value, present := written[field.Name()]
		if !present {
			value = node{Kind: kindZero, Name: field.Name(), Type: typeTree(field.Type())}
		}
		out.Children = append(out.Children, node{
			Kind:     kindKeyed,
			Name:     field.Name(),
			Children: []node{value},
		})
	}
	return out
}

// compositeStruct reports the struct a composite literal constructs, or nil when it constructs
// something else.
func compositeStruct(lit *ast.CompositeLit, ctx *extractCtx) *types.Struct {
	tv, ok := ctx.info.Types[lit]
	if !ok || tv.Type == nil {
		return nil
	}
	underlying := tv.Type.Underlying()
	if pointer, ok := underlying.(*types.Pointer); ok {
		underlying = pointer.Elem().Underlying()
	}
	structured, ok := underlying.(*types.Struct)
	if !ok {
		return nil
	}
	return structured
}

// compositeType records what a composite literal constructs.
func compositeType(lit *ast.CompositeLit, ctx *extractCtx) *typeNode {
	if tv, ok := ctx.info.Types[lit]; ok && tv.Type != nil {
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
		return node{Kind: kindCall, Children: children}

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
func referenceKind(ident *ast.Ident, ctx *extractCtx) string {
	obj := ctx.info.Uses[ident]
	if obj == nil {
		// Not a use of anything the type-checker recorded: a `:=` binding's own name, or the
		// blank identifier. Both are locals as far as casing is concerned.
		return "local"
	}
	switch typed := obj.(type) {
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
func commentText(group *ast.CommentGroup) string {
	if group == nil {
		return ""
	}
	return strings.TrimRight(group.Text(), "\n")
}

// indexGenDeclDocs records documentation for const, var and type declarations.
//
// A GenDecl may carry the comment itself (`// Doc\ntype T struct{}`) or leave it on the single
// spec inside a parenthesised group, so both are checked and the spec's own comment wins — it is
// the more specific of the two.
func indexGenDeclDocs(
	decl *ast.GenDecl,
	tpkg *types.Package,
	docs map[types.Object]string,
	fieldDocs map[string]string,
) {
	groupDoc := commentText(decl.Doc)
	for _, spec := range decl.Specs {
		switch typed := spec.(type) {
		case *ast.TypeSpec:
			if typed.Name == nil {
				continue
			}
			if obj := tpkg.Scope().Lookup(typed.Name.Name); obj != nil {
				if text := firstNonEmpty(commentText(typed.Doc), groupDoc); text != "" {
					docs[obj] = text
				}
			}
			indexStructFieldDocs(typed, fieldDocs)
		case *ast.ValueSpec:
			for _, name := range typed.Names {
				obj := tpkg.Scope().Lookup(name.Name)
				if obj == nil {
					continue
				}
				if text := firstNonEmpty(commentText(typed.Doc), groupDoc); text != "" {
					docs[obj] = text
				}
			}
		}
	}
}

// indexStructFieldDocs keys a struct field's documentation by "TypeName.FieldName". A field is not
// a package-scope object, so there is no types.Object to index it by, and keying by position would
// break the moment a field moves.
func indexStructFieldDocs(spec *ast.TypeSpec, fieldDocs map[string]string) {
	structType, ok := spec.Type.(*ast.StructType)
	if !ok || structType.Fields == nil || spec.Name == nil {
		return
	}
	for _, field := range structType.Fields.List {
		text := firstNonEmpty(commentText(field.Doc), commentText(field.Comment))
		if text == "" {
			continue
		}
		for _, name := range field.Names {
			fieldDocs[spec.Name.Name+"."+name.Name] = text
		}
	}
}

func firstNonEmpty(values ...string) string {
	for _, value := range values {
		if value != "" {
			return value
		}
	}
	return ""
}

func withDoc(attrs map[string]string, text string) map[string]string {
	if text == "" {
		return attrs
	}
	return withAttr(attrs, attrDoc, text)
}

func withAttr(attrs map[string]string, key string, value string) map[string]string {
	if attrs == nil {
		attrs = map[string]string{}
	}
	attrs[key] = value
	return attrs
}

// typeTree renders a go/types type as a tree.
//
// Deliberately does NOT unalias: an alias is a name the source chose, and resolving it here would
// discard the author's vocabulary before the pack ever sees it. The pack can unalias if it wants
// to; it cannot re-alias.
func typeTree(t types.Type) *typeNode {
	switch typed := t.(type) {
	case *types.Basic:
		return &typeNode{Kind: typeBasic, Name: typed.Name()}

	case *types.Alias:
		return namedNode(typed.Obj())

	case *types.Named:
		out := namedNode(typed.Obj())
		for i := 0; i < typed.TypeArgs().Len(); i++ {
			out.Args = append(out.Args, typeTree(typed.TypeArgs().At(i)))
		}
		return out

	case *types.Pointer:
		return &typeNode{Kind: typePointer, Args: []*typeNode{typeTree(typed.Elem())}}

	case *types.Slice:
		return &typeNode{Kind: typeSlice, Args: []*typeNode{typeTree(typed.Elem())}}

	case *types.Array:
		// The length is part of the type. It is carried as a name rather than an argument because
		// it is not a type, and putting a non-type in the argument list would make the arity of
		// every other kind ambiguous.
		return &typeNode{
			Kind: typeArray,
			Name: strconv.FormatInt(typed.Len(), 10),
			Args: []*typeNode{typeTree(typed.Elem())},
		}

	case *types.Map:
		return &typeNode{
			Kind: typeMap,
			Args: []*typeNode{typeTree(typed.Key()), typeTree(typed.Elem())},
		}

	case *types.Chan:
		return &typeNode{
			Kind: typeChan,
			Name: chanDirection(typed.Dir()),
			Args: []*typeNode{typeTree(typed.Elem())},
		}

	case *types.Signature:
		out := &typeNode{Kind: typeFunc}
		out.Args = append(out.Args, tupleTypeNode(typed.Params()), tupleTypeNode(typed.Results()))
		return out

	case *types.Interface:
		return &typeNode{Kind: typeInterface}

	case *types.Struct:
		return &typeNode{Kind: typeStruct}

	case *types.Tuple:
		return tupleTypeNode(typed)

	case *types.TypeParam:
		return &typeNode{Kind: typeParam, Name: typed.Obj().Name()}

	default:
		return &typeNode{Kind: typeUnsupported, Name: strings.TrimPrefix(fmt.Sprintf("%T", t), "*types.")}
	}
}

func namedNode(obj *types.TypeName) *typeNode {
	out := &typeNode{Kind: typeNamed, Name: obj.Name()}
	if pkg := obj.Pkg(); pkg != nil {
		out.Package = pkg.Path()
	}
	return out
}

func tupleTypeNode(tuple *types.Tuple) *typeNode {
	out := &typeNode{Kind: typeTuple}
	if tuple == nil {
		return out
	}
	for i := 0; i < tuple.Len(); i++ {
		out.Args = append(out.Args, typeTree(tuple.At(i).Type()))
	}
	return out
}

func chanDirection(dir types.ChanDir) string {
	switch dir {
	case types.SendOnly:
		return "send"
	case types.RecvOnly:
		return "recv"
	default:
		return "both"
	}
}

// annotateParameterFacts records the ownership facts for each parameter that names something.
//
// Applied to every parameter and not only pointer-typed ones: whether a disposition is meaningful
// for a given type is the ENGINE's question, and deciding it here would put the target language's
// borrow model in the front end.
func annotateParameterFacts(children []node, body *ast.BlockStmt) {
	for i := range children {
		if children[i].Kind != kindParam || children[i].Name == "" {
			continue
		}
		facts := ownershipFacts(body, children[i].Name)
		if len(facts) == 0 {
			continue
		}
		children[i].Flags = append(children[i].Flags, facts...)
		sort.Strings(children[i].Flags)
	}
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

func signatureChildren(sig *types.Signature, qualify types.Qualifier) []node {
	children := make([]node, 0, sig.Params().Len()+sig.Results().Len())
	children = append(children, tupleNodes(kindParam, sig.Params(), qualify)...)
	children = append(children, tupleNodes(kindResult, sig.Results(), qualify)...)
	if len(children) == 0 {
		return nil
	}
	return children
}

// tupleNodes preserves tuple order, which IS semantic: parameters and results are
// positional in both Go and Rust.
func tupleNodes(kind string, tuple *types.Tuple, _ types.Qualifier) []node {
	if tuple == nil || tuple.Len() == 0 {
		return nil
	}
	nodes := make([]node, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		v := tuple.At(i)
		nodes = append(nodes, node{
			Kind: kind,
			Name: v.Name(),
			Type: typeTree(v.Type()),
		})
	}
	return nodes
}

func sortNodes(nodes []node) {
	sort.Slice(nodes, func(i, j int) bool { return nodes[i].Name < nodes[j].Name })
}

// flagsFor returns the set spelling of the boolean facts about a node. Sorted, so the set
// has exactly one encoding; nil when empty, so the JSON omits the key entirely.
func flagsFor(exported bool, variadic bool, embedded bool, pointerReceiver bool) []string {
	flags := make([]string, 0, 4)
	if embedded {
		flags = append(flags, flagEmbedded)
	}
	if exported {
		flags = append(flags, flagExported)
	}
	if pointerReceiver {
		flags = append(flags, flagPointerReceiver)
	}
	if variadic {
		flags = append(flags, flagVariadic)
	}
	if len(flags) == 0 {
		return nil
	}
	sort.Strings(flags)
	return flags
}

// ---------------------------------------------------------------------------------
// Snapshot preimage (mirrored by port_engine_snapshot::snapshot_preimage_v1)
// ---------------------------------------------------------------------------------
//
// `F(s)` is the decimal byte length of s, a `:`, then s. Every node encodes as
//
//	F(kind) F(name) T(type) F(len(flags)) flags...
//	    F(len(attrs)) (F(key) F(value))... F(len(children)) children...
///
// where T(type) is F("0") for an absent type, and otherwise
//
//	F("1") F(kind) F(name) F(package) F(len(args)) args...
//
// Length prefixes plus explicit arity make the encoding injective: no value, however it
// is spelled, can imitate a delimiter or absorb a sibling. That is why the digest does not
// depend on JSON canonicalization — and why the same preimage can be computed in Go here
// and in Rust there, with any drift between the two surfacing as a digest mismatch at
// admission rather than as a silently accepted snapshot.

func preimage(model *snapshot) []byte {
	out := make([]byte, 0, 4096)
	field(&out, "snapshot")
	field(&out, model.Language)
	field(&out, strconv.Itoa(len(model.Packages)))
	for _, pkg := range model.Packages {
		field(&out, "package")
		field(&out, pkg.UnitID)
		field(&out, pkg.Producer)
		field(&out, strconv.Itoa(len(pkg.Declarations)))
		for _, decl := range pkg.Declarations {
			encodeNode(&out, decl)
		}
	}
	return out
}

func encodeNode(out *[]byte, n node) {
	field(out, n.Kind)
	field(out, n.Name)
	encodeType(out, n.Type)
	field(out, strconv.Itoa(len(n.Flags)))
	for _, flag := range n.Flags {
		field(out, flag)
	}
	// Sorted, so the map has exactly one encoding. A map with two orderings is a map with two
	// digests, and the receipt would then attribute a byte-identical corpus to a moved axis.
	attrKeys := make([]string, 0, len(n.Attrs))
	for key := range n.Attrs {
		attrKeys = append(attrKeys, key)
	}
	sort.Strings(attrKeys)
	field(out, strconv.Itoa(len(attrKeys)))
	for _, key := range attrKeys {
		field(out, key)
		field(out, n.Attrs[key])
	}
	field(out, strconv.Itoa(len(n.Children)))
	for _, child := range n.Children {
		encodeNode(out, child)
	}
}

// encodeType covers the type TREE. Leaving it out would put every type outside the snapshot
// identity: change a field's type and `snapshot_digest` would not move, so the receipt would find
// emitted bytes changed with all six axes held and call a fully explainable change Unexplained.
func encodeType(out *[]byte, t *typeNode) {
	if t == nil {
		field(out, "0")
		return
	}
	field(out, "1")
	field(out, t.Kind)
	field(out, t.Name)
	field(out, t.Package)
	field(out, strconv.Itoa(len(t.Args)))
	for _, arg := range t.Args {
		encodeType(out, arg)
	}
}

func field(out *[]byte, value string) {
	*out = append(*out, strconv.Itoa(len(value))...)
	*out = append(*out, ':')
	*out = append(*out, value...)
}

func digest(preimage []byte) string {
	sum := sha256.Sum256(preimage)
	return "sha256:" + hex.EncodeToString(sum[:])
}
