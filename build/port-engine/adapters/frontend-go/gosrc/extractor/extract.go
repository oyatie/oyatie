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

func extract(
	corpusDir string,
	modulePath string,
	moduleRoot string,
	cfg *buildConfig,
) (*snapshot, error) {
	dirs, err := packageDirs(corpusDir, cfg)
	if err != nil {
		return nil, err
	}
	if len(dirs) == 0 {
		return nil, fmt.Errorf(
			"corpus %s contains no Go package directory for %s",
			corpusDir,
			cfg.describe(),
		)
	}

	// The corpus is its own importer: an intra-corpus import resolves by type-checking the
	// referenced package here, because no module path the stdlib importer knows contains it.
	packages := map[string]string{}
	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		packages[unitIDFor(modulePath, rel)] = dir
	}
	// DEPENDENCIES the type-checker will need, resolved by the Go tool. Merged UNDER the corpus:
	// where both define a path the corpus wins, because the corpus is what is being ported and its
	// sources are the subject rather than a dependency of it.
	resolved, err := dependencyDirs(corpusDir)
	if err != nil {
		return nil, err
	}
	for path, dir := range resolved {
		if _, inCorpus := packages[path]; !inCorpus {
			packages[path] = dir
		}
	}
	resolver := newCorpusImporter(packages, cfg)

	model := &snapshot{
		SchemaVersion: schemaVersion,
		Language:      "go",
		BuildConfig:   cfg.describe(),
		Packages:      make([]pkgNode, 0, len(dirs)),
	}

	facts := []satisfaction{}
	qualifiers := map[string]types.Qualifier{}
	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		unitID := unitIDFor(modulePath, rel)

		decls, observed, tpkg, err := extractPackage(dir, unitID, resolver, cfg)
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
	// cfg is the SAME configuration the extraction walk uses. It has to be: a package reached
	// through an import and the same package reached directly must be one package, and selecting
	// its files by two different rules makes it two.
	cfg *buildConfig
}

func newCorpusImporter(dirs map[string]string, cfg *buildConfig) *corpusImporter {
	fset := token.NewFileSet()
	return &corpusImporter{
		dirs:     dirs,
		resolved: map[string]*types.Package{},
		fallback: importer.ForCompiler(fset, "source", nil),
		fset:     fset,
		cfg:      cfg,
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

	files, err := parsePackage(c.fset, dir, c.cfg)
	if err != nil {
		return nil, err
	}
	conf := types.Config{Importer: c, GoVersion: c.cfg.goVersion()}
	pkg, err := conf.Check(path, c.fset, files, nil)
	if err != nil {
		return nil, fmt.Errorf("import %s: %w", path, err)
	}
	c.resolved[path] = pkg
	return pkg, nil
}

// parsePackage parses the files this CONFIGURATION selects in dir, in sorted order.
func parsePackage(fset *token.FileSet, dir string, cfg *buildConfig) ([]*ast.File, error) {
	names, err := selectDirFiles(dir, cfg)
	if err != nil {
		return nil, err
	}

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
// packageDirs lists the directories that are a package UNDER THIS CONFIGURATION.
//
// A directory holding only `//go:build windows` files is not an empty package on linux, it is not
// a package at all — and reporting it as one would fail the extraction of a corpus that builds
// perfectly well for the declared target.
func packageDirs(root string, cfg *buildConfig) ([]string, error) {
	candidates := map[string]bool{}
	err := filepath.Walk(root, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if info.IsDir() {
			// The Go tool itself ignores these, and so must the walk: `vendor` holds other
			// modules' sources, `testdata` is data rather than code, and a leading `_` or `.`
			// marks a directory the build never considers. Walking them made `chi` fail on an
			// `_examples` program whose imports are not the library's.
			name := info.Name()
			if path != root && (name == "vendor" || name == "testdata" ||
				strings.HasPrefix(name, "_") || strings.HasPrefix(name, ".")) {
				return filepath.SkipDir
			}
			return nil
		}
		if !strings.HasSuffix(path, ".go") || strings.HasSuffix(path, "_test.go") {
			return nil
		}
		candidates[filepath.Dir(path)] = true
		return nil
	})
	if err != nil {
		return nil, fmt.Errorf("walk %s: %w", root, err)
	}
	dirs := make([]string, 0, len(candidates))
	for dir := range candidates {
		selected, err := selectDirFiles(dir, cfg)
		if err != nil {
			return nil, err
		}
		if len(selected) > 0 {
			dirs = append(dirs, dir)
		}
	}
	sort.Strings(dirs)
	return dirs, nil
}

// unitIDFor spells a package's import path from the module path and its directory relative to
// the module root.
//
// The root directory relativises to ".", and joining that naively spells `example.com/mod/.` —
// which is the import path of nothing. A package importing its own module root then fails to
// resolve, and the error it produces ("no required module provides package") names a missing
// dependency rather than the naming bug it actually is. Every fixture package is a subdirectory,
// so the committed corpus could not have shown this; almost every real module puts code at its
// root.
func unitIDFor(modulePath string, rel string) string {
	slashed := filepath.ToSlash(rel)
	if slashed == "." {
		return modulePath
	}
	return modulePath + "/" + slashed
}

// selectDirFiles reads a directory and asks the configuration which of its files are in.
func selectDirFiles(dir string, cfg *buildConfig) ([]string, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("read dir %s: %w", dir, err)
	}
	names := make([]string, 0, len(entries))
	for _, entry := range entries {
		if !entry.IsDir() {
			names = append(names, entry.Name())
		}
	}
	return cfg.selectFiles(dir, names)
}
