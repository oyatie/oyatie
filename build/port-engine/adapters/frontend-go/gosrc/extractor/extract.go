package main

import (
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

// Corpus walk: every package directory, in a deterministic order, through one importer.
//
// The importer is memoised on the PACKAGE rather than on the check, because a diamond import would
// otherwise type-check the shared dependency twice and produce two distinct `types.Package` values
// for one package — after which a cross-package type compares unequal to itself.

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

	facts := []satisfaction{}
	qualifiers := map[string]types.Qualifier{}
	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		unitID := modulePath + "/" + filepath.ToSlash(rel)

		decls, observed, tpkg, err := extractPackage(dir, unitID, resolver)
		if err != nil {
			return nil, fmt.Errorf("package %s: %w", unitID, err)
		}
		facts = append(facts, observed...)
		qualifiers[unitID] = qualifierFor(tpkg)
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

	attributeSatisfactions(model, dedupeSatisfactions(facts), qualifiers)

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
