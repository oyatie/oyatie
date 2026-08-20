package main

import (
	"fmt"
	"go/ast"
	"go/importer"
	"go/parser"
	"go/token"
	"go/types"
	"io"
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
	packages := map[string]listedPackage{}
	for _, dir := range dirs {
		rel, err := filepath.Rel(moduleRoot, dir)
		if err != nil {
			return nil, fmt.Errorf("relativize %s: %w", dir, err)
		}
		// The corpus is checked at the CONFIGURED release, which is the whole point of configuring
		// one. Its own module's directive is a ceiling checked below, not a substitute.
		packages[unitIDFor(modulePath, rel)] = listedPackage{dir: dir, goVersion: cfg.goVersion()}
	}
	// DEPENDENCIES the type-checker will need, resolved by the Go tool. Merged UNDER the corpus:
	// where both define a path the corpus wins, because the corpus is what is being ported and its
	// sources are the subject rather than a dependency of it.
	resolved, err := dependencyPackages(corpusDir, cfg)
	if err != nil {
		return nil, err
	}
	for path, listed := range resolved {
		if _, inCorpus := packages[path]; !inCorpus {
			packages[path] = listed
		}
	}
	// THE CORPUS'S OWN CEILING, refused by name and here rather than wherever the first construct
	// that needs it happens to sit. A corpus whose module declares a later release than the one
	// configured contains constructs that are not legal at the configured one, and the failure
	// otherwise surfaces as a syntax error inside a vendored file six imports away -- which names
	// neither the corpus, nor the release, nor the mismatch between them.
	if declared := corpusRelease(resolved, modulePath); declared > cfg.release {
		return nil, fmt.Errorf(
			"corpus module %s declares go1.%d and extraction is configured for %s: "+
				"the configured release is a ceiling, and a corpus is not silently checked below "+
				"the release its own module requires",
			modulePath, declared, cfg.goVersion(),
		)
	}
	resolver := newCorpusImporter(packages, cfg)

	model := &snapshot{
		SchemaVersion: schemaVersion,
		Language:      "go",
		BuildConfig:   cfg.describe(),
		Packages:      make([]pkgNode, 0, len(dirs)),
	}
	// Filled in AFTER the walk, because whether the weaker importer was needed is only known once
	// every import has been resolved. See the note on `corpusImporter.source`.
	defer func() {
		if resolver.usedSource {
			model.BuildConfig += " imports=source"
		}
	}()

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
	dirs     map[string]listedPackage
	resolved map[string]*types.Package
	fallback types.Importer
	source   types.Importer
	// usedSource records that at least one import was resolved through the weaker source importer,
	// which resolves the standard library for the host rather than for the declared target.
	usedSource bool
	fset       *token.FileSet
	// cfg is the SAME configuration the extraction walk uses. It has to be: a package reached
	// through an import and the same package reached directly must be one package, and selecting
	// its files by two different rules makes it two.
	cfg *buildConfig
}

func newCorpusImporter(dirs map[string]listedPackage, cfg *buildConfig) *corpusImporter {
	fset := token.NewFileSet()
	return &corpusImporter{
		dirs:     dirs,
		resolved: map[string]*types.Package{},
		// EXPORT DATA, which is what the compiler reads for an import. The source importer
		// `go/importer` offers takes no build context and so resolves the standard library for the
		// HOST, which makes a declared cross-target configuration only half true. The lookup hands
		// back the file the Go tool produced for the DECLARED target, so there is one answer and
		// the toolchain is the one giving it.
		fallback: importer.ForCompiler(fset, "gc", func(path string) (io.ReadCloser, error) {
			listed, ok := dirs[path]
			if !ok || listed.export == "" {
				return nil, fmt.Errorf("no export data for %s", path)
			}
			return os.Open(listed.export)
		}),
		// LAST RESORT, and a WEAKER answer that is recorded as one. A corpus the Go tool cannot
		// build -- an example module with no module cache, a fixture that is not a buildable
		// program -- yields no export data at all, and every fixture in this repository is in that
		// position deliberately. Falling back to source keeps them loadable.
		//
		// The weakness is real and is why the two are not interchangeable: `go/importer`'s source
		// compiler takes no build context, so it resolves the standard library for the HOST. A
		// cross-target configuration served this way is only half true. `usedSource` records that
		// it happened so the snapshot can say so rather than looking like the stronger answer.
		source: importer.ForCompiler(fset, "source", nil),
		fset:   fset,
		cfg:    cfg,
	}
}

func (c *corpusImporter) Import(path string) (*types.Package, error) {
	if pkg, ok := c.resolved[path]; ok {
		return pkg, nil
	}
	// `unsafe` IS NOT A PACKAGE. It is built into the type-checker, and `types.Unsafe` is the only
	// value of it that compares equal to itself. Type-checking GOROOT's `unsafe` directory from
	// source yields a SECOND package whose `Pointer` is a different type from the real one, and
	// then a conversion the language guarantees stops being legal -- `cannot convert
	// &sliceHeader{..} to type unsafe.Pointer`, on a line that is correct Go. The resolver lists it
	// like any other import, so it has to be excluded here.
	if path == "unsafe" {
		return types.Unsafe, nil
	}
	listed, ok := c.dirs[path]
	// FROM SOURCE only where there is no export data, which is exactly the corpus's own packages:
	// they are the subject of the port and are not built by the resolver. Everything else is read
	// the way the compiler reads it. Checking a dependency from source meant checking it at a
	// release its own module never declared, and reading it here means that question no longer
	// exists rather than being answered carefully.
	if !ok || listed.dir == "" || listed.export != "" {
		pkg, err := c.fallback.Import(path)
		if err == nil {
			return pkg, nil
		}
		// No export data for it. Source is the weaker answer and is taken rather than failing,
		// because a corpus the Go tool cannot build still type-checks from source.
		c.usedSource = true
		return c.source.Import(path)
	}

	files, err := parsePackage(c.fset, listed.dir, c.cfg)
	if err != nil {
		return nil, err
	}
	// EACH MODULE AT ITS OWN RELEASE. A package's language version comes from the `go` directive of
	// the module that owns it, and applying the corpus's to everything is a build no Go toolchain
	// performs. It fails false -- `x/sys` declares go1.25 and uses `for range n` -- and it can
	// succeed false, which is worse: a dependency declaring go1.22 checked at go1.21 gets the
	// pre-1.22 loop-variable scoping, the same syntax and a different program. An empty version is
	// left empty rather than filled in with the corpus's, so the type-checker applies its own
	// default instead of a version this program invented.
	conf := types.Config{Importer: c, GoVersion: listed.goVersion}
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
