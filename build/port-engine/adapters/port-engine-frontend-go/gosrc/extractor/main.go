// Command extractor is the bootstrap Go front end for the owned deterministic port
// engine (ADR-0638 D3).
//
// It reads a Go corpus with go/parser + go/types and writes a SourceModel snapshot
// envelope as JSON. It runs OUT OF BAND ONLY: the engine's verify() path consumes the
// snapshot artifact and must never invoke a Go toolchain. The Rust side enforces that
// with an architecture test over its own library sources; nothing here is linked into
// the engine.
//
// Only the Go standard library is used. golang.org/x/tools/go/packages would give
// richer package loading and would also give this fixture module a dependency graph,
// a go.sum, and a vendoring question. The corpus is small and hermetic, so stdlib
// parsing is sufficient and buys the module's dependency-freedom.
//
// Usage:
//
//	go run ./extractor -corpus ./corpus -module oyatie.example/portengine-fixture \
//	    -out ../src/fixture-snapshot-v1.json
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
// decoder refuses any other identity during bootstrap admission, so a drift here is a
// red at admission rather than a silent relabel.
const producerBootstrapGo = "bootstrap-go-packages-go-types"

// schemaVersion is the snapshot envelope version this extractor emits. v0 carried unit
// identity only; v1 adds declarations.
const schemaVersion = 1

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
	UnitID       string     `json:"unit_id"`
	Producer     string     `json:"producer"`
	Declarations []declNode `json:"declarations"`
}

// declNode is one package-scope declaration. Go gives every package-scope identifier a
// single shared namespace, so `name` is unique within a package across all kinds — the
// Rust decoder refuses a duplicate on that basis.
type declNode struct {
	Kind     string       `json:"kind"`
	Name     string       `json:"name"`
	Exported bool         `json:"exported"`
	Type     string       `json:"type,omitempty"`
	Fields   []fieldNode  `json:"fields,omitempty"`
	Methods  []methodNode `json:"methods,omitempty"`
	Params   []paramNode  `json:"params,omitempty"`
	Results  []paramNode  `json:"results,omitempty"`
	Variadic bool         `json:"variadic,omitempty"`
}

type fieldNode struct {
	Name     string `json:"name"`
	Type     string `json:"type"`
	Exported bool   `json:"exported"`
	Embedded bool   `json:"embedded,omitempty"`
}

type methodNode struct {
	Name     string      `json:"name"`
	Exported bool        `json:"exported"`
	Params   []paramNode `json:"params,omitempty"`
	Results  []paramNode `json:"results,omitempty"`
	Variadic bool        `json:"variadic,omitempty"`
}

type paramNode struct {
	Name string `json:"name"`
	Type string `json:"type"`
}

// Declaration kinds. These strings are the vocabulary the neutral rule pack's `captures`
// select on, so they are part of the snapshot contract, not an internal detail.
const (
	kindConst     = "const"
	kindVar       = "var"
	kindFunc      = "func"
	kindStruct    = "struct"
	kindInterface = "interface"
	kindAlias     = "alias"
	kindNamed     = "named"
)

// ---------------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------------

func main() {
	corpus := flag.String("corpus", "./corpus", "directory whose subdirectories are Go packages")
	module := flag.String("module", "oyatie.example/portengine-fixture", "module path prefix for unit ids")
	out := flag.String("out", "", "output file; empty writes to stdout")
	flag.Parse()

	model, err := extract(*corpus, *module)
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

func extract(corpusDir string, modulePath string) (*snapshot, error) {
	dirs, err := packageDirs(corpusDir)
	if err != nil {
		return nil, err
	}
	if len(dirs) == 0 {
		return nil, fmt.Errorf("corpus %s contains no Go package directory", corpusDir)
	}

	model := &snapshot{
		SchemaVersion: schemaVersion,
		Language:      "go",
		Packages:      make([]pkgNode, 0, len(dirs)),
	}

	for _, dir := range dirs {
		rel, err := filepath.Rel(corpusDir, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		unitID := modulePath + "/" + filepath.ToSlash(rel)

		decls, err := extractPackage(dir, unitID)
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

// packageDirs returns every directory at or under root that holds at least one .go file,
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

func extractPackage(dir string, unitID string) ([]declNode, error) {
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
		file, err := parser.ParseFile(fset, filepath.Join(dir, name), nil, parser.SkipObjectResolution)
		if err != nil {
			return nil, fmt.Errorf("parse %s: %w", name, err)
		}
		files = append(files, file)
	}

	conf := types.Config{Importer: importer.ForCompiler(fset, "source", nil)}
	tpkg, err := conf.Check(unitID, fset, files, nil)
	if err != nil {
		return nil, fmt.Errorf("type-check: %w", err)
	}

	// Render types relative to the package under extraction: local names stay bare, and
	// anything from elsewhere keeps its full path so the rule pack's type map can tell
	// a local `Point` from an imported one.
	qualify := func(other *types.Package) string {
		if other == tpkg {
			return ""
		}
		return other.Path()
	}

	scope := tpkg.Scope()
	objNames := scope.Names() // already sorted by go/types
	decls := make([]declNode, 0, len(objNames))
	for _, name := range objNames {
		decl, err := declFor(scope.Lookup(name), qualify)
		if err != nil {
			return nil, fmt.Errorf("declaration %s: %w", name, err)
		}
		decls = append(decls, decl)
	}
	return decls, nil
}

func declFor(obj types.Object, qualify types.Qualifier) (declNode, error) {
	base := declNode{Name: obj.Name(), Exported: obj.Exported()}

	switch typed := obj.(type) {
	case *types.Const:
		base.Kind = kindConst
		base.Type = types.TypeString(typed.Type(), qualify)
		return base, nil

	case *types.Var:
		base.Kind = kindVar
		base.Type = types.TypeString(typed.Type(), qualify)
		return base, nil

	case *types.Func:
		sig, ok := typed.Type().(*types.Signature)
		if !ok {
			return base, fmt.Errorf("func object without signature")
		}
		base.Kind = kindFunc
		base.Params = tupleNodes(sig.Params(), qualify)
		base.Results = tupleNodes(sig.Results(), qualify)
		base.Variadic = sig.Variadic()
		return base, nil

	case *types.TypeName:
		return typeDecl(typed, base, qualify)

	default:
		return base, fmt.Errorf("unsupported object kind %T", obj)
	}
}

func typeDecl(obj *types.TypeName, base declNode, qualify types.Qualifier) (declNode, error) {
	if obj.IsAlias() {
		base.Kind = kindAlias
		// Unalias, or the alias renders as its own name: since Go 1.22 an alias is a
		// materialized *types.Alias whose String() is the alias identifier, so
		// `type ID = string` would extract as `ID -> ID` and say nothing. Unalias
		// resolves a chain all the way to the aliased type, which is what a type map
		// needs to answer with. Note this is the alias TARGET; a parameter written as
		// `ID` still extracts as `ID`, because there the alias name is what was written.
		base.Type = types.TypeString(types.Unalias(obj.Type()), qualify)
		return base, nil
	}

	named, ok := obj.Type().(*types.Named)
	if !ok {
		// A non-alias TypeName that is not Named is a builtin (`error`, `any`); the
		// corpus should not surface one at package scope, so refuse rather than guess.
		return base, fmt.Errorf("non-alias type name with unexpected type %T", obj.Type())
	}

	methods := make([]methodNode, 0, named.NumMethods())
	for i := 0; i < named.NumMethods(); i++ {
		method := named.Method(i)
		sig, ok := method.Type().(*types.Signature)
		if !ok {
			return base, fmt.Errorf("method %s without signature", method.Name())
		}
		methods = append(methods, methodNode{
			Name:     method.Name(),
			Exported: method.Exported(),
			Params:   tupleNodes(sig.Params(), qualify),
			Results:  tupleNodes(sig.Results(), qualify),
			Variadic: sig.Variadic(),
		})
	}
	sort.Slice(methods, func(i, j int) bool { return methods[i].Name < methods[j].Name })

	switch underlying := named.Underlying().(type) {
	case *types.Struct:
		base.Kind = kindStruct
		fields := make([]fieldNode, 0, underlying.NumFields())
		for i := 0; i < underlying.NumFields(); i++ {
			field := underlying.Field(i)
			fields = append(fields, fieldNode{
				Name:     field.Name(),
				Type:     types.TypeString(field.Type(), qualify),
				Exported: field.Exported(),
				Embedded: field.Embedded(),
			})
		}
		// Field order is declaration order and is SEMANTIC in Go (layout, composite
		// literals), so it is deliberately not sorted.
		base.Fields = fields
		base.Methods = methods
		return base, nil

	case *types.Interface:
		base.Kind = kindInterface
		ifaceMethods := make([]methodNode, 0, underlying.NumExplicitMethods())
		for i := 0; i < underlying.NumExplicitMethods(); i++ {
			method := underlying.ExplicitMethod(i)
			sig, ok := method.Type().(*types.Signature)
			if !ok {
				return base, fmt.Errorf("interface method %s without signature", method.Name())
			}
			ifaceMethods = append(ifaceMethods, methodNode{
				Name:     method.Name(),
				Exported: method.Exported(),
				Params:   tupleNodes(sig.Params(), qualify),
				Results:  tupleNodes(sig.Results(), qualify),
				Variadic: sig.Variadic(),
			})
		}
		sort.Slice(ifaceMethods, func(i, j int) bool { return ifaceMethods[i].Name < ifaceMethods[j].Name })
		base.Methods = ifaceMethods
		return base, nil

	default:
		base.Kind = kindNamed
		base.Type = types.TypeString(underlying, qualify)
		base.Methods = methods
		return base, nil
	}
}

func tupleNodes(tuple *types.Tuple, qualify types.Qualifier) []paramNode {
	if tuple == nil || tuple.Len() == 0 {
		return nil
	}
	nodes := make([]paramNode, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		v := tuple.At(i)
		nodes = append(nodes, paramNode{Name: v.Name(), Type: types.TypeString(v.Type(), qualify)})
	}
	return nodes
}

// ---------------------------------------------------------------------------------
// Snapshot preimage (mirrored by port_engine_snapshot::snapshot_preimage_v1)
// ---------------------------------------------------------------------------------
//
// Every node is `F(label) F(value) F(itoa(child_count))` followed by its children, where
// `F(s)` is the decimal byte length of s, a `:`, then s. Length prefixes plus an explicit
// arity make the encoding injective: no field value, however it is spelled, can imitate a
// delimiter or absorb a sibling. That is why the digest does not depend on JSON
// canonicalization — and why the same preimage can be computed in Go here and in Rust
// there, with any drift between the two surfacing as a digest mismatch at admission
// rather than as a silently accepted snapshot.

func preimage(model *snapshot) []byte {
	out := make([]byte, 0, 1024)
	node(&out, "snapshot", model.Language, len(model.Packages))
	for _, pkg := range model.Packages {
		node(&out, "package", pkg.UnitID, 1+len(pkg.Declarations))
		node(&out, "producer", pkg.Producer, 0)
		for _, decl := range pkg.Declarations {
			encodeDecl(&out, decl)
		}
	}
	return out
}

func encodeDecl(out *[]byte, decl declNode) {
	children := 3 + len(decl.Fields) + len(decl.Methods) + len(decl.Params) + len(decl.Results)
	node(out, decl.Kind, decl.Name, children)
	node(out, "exported", boolField(decl.Exported), 0)
	node(out, "variadic", boolField(decl.Variadic), 0)
	node(out, "type", decl.Type, 0)
	for _, field := range decl.Fields {
		node(out, "field", field.Name, 3)
		node(out, "exported", boolField(field.Exported), 0)
		node(out, "embedded", boolField(field.Embedded), 0)
		node(out, "type", field.Type, 0)
	}
	for _, method := range decl.Methods {
		encodeMethod(out, method)
	}
	encodeParams(out, decl.Params, decl.Results)
}

func encodeMethod(out *[]byte, method methodNode) {
	children := 2 + len(method.Params) + len(method.Results)
	node(out, "method", method.Name, children)
	node(out, "exported", boolField(method.Exported), 0)
	node(out, "variadic", boolField(method.Variadic), 0)
	encodeParams(out, method.Params, method.Results)
}

func encodeParams(out *[]byte, params []paramNode, results []paramNode) {
	for _, param := range params {
		node(out, "param", param.Name, 1)
		node(out, "type", param.Type, 0)
	}
	for _, result := range results {
		node(out, "result", result.Name, 1)
		node(out, "type", result.Type, 0)
	}
}

func node(out *[]byte, label string, value string, children int) {
	field(out, label)
	field(out, value)
	field(out, strconv.Itoa(children))
}

func field(out *[]byte, value string) {
	*out = append(*out, strconv.Itoa(len(value))...)
	*out = append(*out, ':')
	*out = append(*out, value...)
}

func boolField(value bool) string {
	if value {
		return "1"
	}
	return "0"
}

func digest(preimage []byte) string {
	sum := sha256.Sum256(preimage)
	return "sha256:" + hex.EncodeToString(sum[:])
}
