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

// schemaVersion is the snapshot envelope version this extractor emits. v0 carried unit
// identity only; v1 adds the declaration tree.
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
	Kind     string   `json:"kind"`
	Name     string   `json:"name"`
	Type     string   `json:"type,omitempty"`
	Flags    []string `json:"flags,omitempty"`
	Children []node   `json:"children,omitempty"`
}

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
)

// Flags. Sorted on emit so the set has one spelling.
const (
	flagExported = "exported"
	flagVariadic = "variadic"
	flagEmbedded = "embedded"
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

func extractPackage(dir string, unitID string) ([]node, error) {
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
	// anything from elsewhere keeps its full path, so the rule pack's type map can tell a
	// local `Point` from an imported one.
	qualify := func(other *types.Package) string {
		if other == tpkg {
			return ""
		}
		return other.Path()
	}

	scope := tpkg.Scope()
	objNames := scope.Names() // go/types returns these sorted
	decls := make([]node, 0, len(objNames))
	for _, name := range objNames {
		decl, err := declFor(scope.Lookup(name), qualify)
		if err != nil {
			return nil, fmt.Errorf("declaration %s: %w", name, err)
		}
		decls = append(decls, decl)
	}
	return decls, nil
}

func declFor(obj types.Object, qualify types.Qualifier) (node, error) {
	base := node{Name: obj.Name(), Flags: flagsFor(obj.Exported(), false, false)}

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
		base.Flags = flagsFor(obj.Exported(), sig.Variadic(), false)
		base.Children = signatureChildren(sig, qualify)
		return base, nil

	case *types.TypeName:
		return typeDecl(typed, base, qualify)

	default:
		return base, fmt.Errorf("unsupported object kind %T", obj)
	}
}

func typeDecl(obj *types.TypeName, base node, qualify types.Qualifier) (node, error) {
	if obj.IsAlias() {
		base.Kind = kindAlias
		// Unalias, or the alias renders as its own name: since Go 1.22 an alias is a
		// materialized *types.Alias whose String() is the alias identifier, so
		// `type ID = string` would extract as `ID -> ID` and say nothing. Unalias
		// resolves the chain to the aliased type, which is what a type map answers with.
		// This is the alias TARGET; a parameter written as `ID` still extracts as `ID`,
		// because there the alias name is what was written.
		base.Type = types.TypeString(types.Unalias(obj.Type()), qualify)
		return base, nil
	}

	named, ok := obj.Type().(*types.Named)
	if !ok {
		// A non-alias TypeName that is not Named is a builtin (`error`, `any`); the corpus
		// should not surface one at package scope, so refuse rather than guess.
		return base, fmt.Errorf("non-alias type name with unexpected type %T", obj.Type())
	}

	methods, err := methodChildren(named, qualify)
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
				Type:  types.TypeString(field.Type(), qualify),
				Flags: flagsFor(field.Exported(), false, field.Embedded()),
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
				Kind:     kindMethod,
				Name:     method.Name(),
				Flags:    flagsFor(method.Exported(), sig.Variadic(), false),
				Children: signatureChildren(sig, qualify),
			})
		}
		sortNodes(ifaceMethods)
		base.Children = ifaceMethods
		return base, nil

	default:
		base.Kind = kindNamed
		base.Type = types.TypeString(underlying, qualify)
		base.Children = methods
		return base, nil
	}
}

// methodChildren returns the methods declared on named, sorted by name. Source order is
// not used: unlike struct fields, method order carries no Go semantics, and sorting keeps
// the snapshot stable against a reordering edit that changes nothing.
func methodChildren(named *types.Named, qualify types.Qualifier) ([]node, error) {
	methods := make([]node, 0, named.NumMethods())
	for i := 0; i < named.NumMethods(); i++ {
		method := named.Method(i)
		sig, ok := method.Type().(*types.Signature)
		if !ok {
			return nil, fmt.Errorf("method %s without signature", method.Name())
		}
		methods = append(methods, node{
			Kind:     kindMethod,
			Name:     method.Name(),
			Flags:    flagsFor(method.Exported(), sig.Variadic(), false),
			Children: signatureChildren(sig, qualify),
		})
	}
	sortNodes(methods)
	return methods, nil
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
func tupleNodes(kind string, tuple *types.Tuple, qualify types.Qualifier) []node {
	if tuple == nil || tuple.Len() == 0 {
		return nil
	}
	nodes := make([]node, 0, tuple.Len())
	for i := 0; i < tuple.Len(); i++ {
		v := tuple.At(i)
		nodes = append(nodes, node{
			Kind: kind,
			Name: v.Name(),
			Type: types.TypeString(v.Type(), qualify),
		})
	}
	return nodes
}

func sortNodes(nodes []node) {
	sort.Slice(nodes, func(i, j int) bool { return nodes[i].Name < nodes[j].Name })
}

// flagsFor returns the set spelling of the boolean facts about a node. Sorted, so the set
// has exactly one encoding; nil when empty, so the JSON omits the key entirely.
func flagsFor(exported bool, variadic bool, embedded bool) []string {
	flags := make([]string, 0, 3)
	if embedded {
		flags = append(flags, flagEmbedded)
	}
	if exported {
		flags = append(flags, flagExported)
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
//	F(kind) F(name) F(type) F(len(flags)) flags... F(len(children)) children...
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
	field(out, n.Type)
	field(out, strconv.Itoa(len(n.Flags)))
	for _, flag := range n.Flags {
		field(out, flag)
	}
	field(out, strconv.Itoa(len(n.Children)))
	for _, child := range n.Children {
		encodeNode(out, child)
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
