package main

import (
	"encoding/json"
	"fmt"
	"os"
	"os/exec"
	"runtime"
	"strconv"
	"strings"
)

// Dependency package directories, resolved by the Go tool.
//
// The type-checker needs a source directory for every import a package makes. The corpus supplies
// its own, and the standard library's importer supplies GOROOT's — which leaves every THIRD-PARTY
// import unresolvable, so the front end could only ever load a dependency-free package. That was not
// a stated limitation; it was a selection effect, and it is why the ratchet corpus looked like a set
// of dependency-free byte utilities while the ecosystem it claims to port does not.
//
// Resolved by asking `go list`, not by adding a module dependency. This module is deliberately
// dependency-free — its own doc says so, and licensing policy fail-closes on any new extractor
// dependency until provenance is recorded — and the Go tool is already required to build and run
// this program. It is the module resolver, so it is asked rather than reimplemented: it understands
// the module graph, the vendor directory, workspaces and replace directives, and reimplementing that
// is how a front end acquires a subtly different idea of what a package is than the compiler has.
// listedPackage is what the resolver knows about one package the corpus reaches.
type listedPackage struct {
	// dir is where its source lives.
	dir string
	// goVersion is the `go` directive of the MODULE THAT OWNS IT, empty when the resolver did not
	// report one. A package's language version comes from its own module and from nowhere else --
	// type-checking someone else's module at this corpus's release is a build no Go toolchain
	// performs. It fails false, and worse it can SUCCEED false: a dependency whose module declares
	// go1.22 checked at go1.21 gets the pre-1.22 loop-variable scoping, which is the same syntax
	// and a different program. That is R3d, reintroduced once per dependency.
	goVersion string
	// standard reports whether this package is part of the standard library.
	standard bool
	// export is the compiled export-data file for this package, when the resolver produced one.
	//
	// This is what the COMPILER reads for an import, and reading the same thing is what makes the
	// front end's idea of a dependency's types identical to the toolchain's rather than merely
	// similar. It is also the only way to get types for the DECLARED target: the source importer
	// `go/importer` exposes takes no build context, so it type-checks the standard library for the
	// host -- a corpus declared linux/amd64 was having its stdlib checked as darwin/arm64, and
	// failed on a symbol that exists only on Linux. Export data is produced by the real toolchain,
	// for the target it was asked for, at the release each module declares.
	export string
}

// dependencyPackages resolves every package the corpus reaches, for the DECLARED target.
//
// Resolved by asking `go list`, not by adding a module dependency. This module is deliberately
// dependency-free -- its own doc says so, and licensing policy fail-closes on any new extractor
// dependency until provenance is recorded -- and the Go tool is already required to build and run
// this program. It is the module resolver, so it is asked rather than reimplemented: it understands
// the module graph, the vendor directory, workspaces and replace directives, and reimplementing
// that is how a front end acquires a subtly different idea of what a package is than the compiler.
//
// The QUESTION IT IS ASKED is the declared one. The dependency set is a function of the target:
// `github.com/miekg/dns` reaches `golang.org/x/net/ipv4` on linux and not on darwin, so a resolver
// run with the host's environment returns a set that does not match the files the walk selects, and
// the type-check dies claiming a package cannot be found in GOROOT. The environment is set from the
// same configuration everything else reads.
//
// The STANDARD LIBRARY is included. Its own importer resolves it for the HOST -- `go/importer`'s
// source compiler is `srcimporter` over `build.Default` and takes no build context -- so a corpus
// declared `linux/amd64` was having its stdlib type-checked as darwin/arm64, and `x/sys/unix` failed
// on a symbol that exists only on Linux. Reading the standard library through the same path as
// everything else is what makes the declared configuration mean what it says.
func dependencyPackages(corpus string, cfg *buildConfig) (map[string]listedPackage, error) {
	cmd := exec.Command("go", "list", "-deps", "-json", "-export", "./...")
	cmd.Dir = corpus
	cmd.Env = append(os.Environ(),
		"GOOS="+cfg.goos,
		"GOARCH="+cfg.goarch,
		// The walk excludes cgo files and so must the resolver, or it reports imports that belong
		// to files this configuration never selects.
		"CGO_ENABLED=0",
	)
	if len(cfg.tags) > 0 {
		cmd.Args = append(cmd.Args[:len(cmd.Args)-1], "-tags", strings.Join(cfg.tags, ","), "./...")
	}
	out, err := cmd.Output()
	if err != nil {
		// NOT fatal. A corpus whose dependencies do not resolve still type-checks if its imports
		// are all standard or intra-corpus, which is exactly the case every fixture is in. Failing
		// here would make the fixtures depend on a working module cache to load at all.
		return map[string]listedPackage{}, nil
	}
	packages := map[string]listedPackage{}
	decoder := json.NewDecoder(strings.NewReader(string(out)))
	for decoder.More() {
		var listed struct {
			ImportPath string
			Dir        string
			Export     string
			Standard   bool
			Module     *struct {
				GoVersion string
			}
		}
		if err := decoder.Decode(&listed); err != nil {
			return nil, fmt.Errorf("go list output: %w", err)
		}
		if listed.ImportPath == "" || (listed.Dir == "" && listed.Export == "") {
			continue
		}
		entry := listedPackage{dir: listed.Dir, export: listed.Export, standard: listed.Standard}
		switch {
		case listed.Standard:
			// The STANDARD LIBRARY belongs to the toolchain, not to a module -- `go list` reports
			// no module for it, so there is no directive to read. Its language version is the
			// toolchain's own, because the source being checked is that toolchain's source: it
			// uses whatever the release it shipped with allows, and checking it lower fails inside
			// files nobody in the corpus wrote. The toolchain is already an axis of the receipt.
			entry.goVersion = toolchainVersion()
		case listed.Module != nil:
			entry.goVersion = listed.Module.GoVersion
		}
		packages[listed.ImportPath] = entry
	}
	return packages, nil
}

// corpusRelease is the Go 1.N release the corpus's own module declares, or 0 when it declares none.
//
// The declared release is a CEILING: a corpus whose module says `go 1.25` contains constructs that
// are not legal earlier, and checking it at an earlier release fails somewhere inside whichever
// file happens to use one. `memberlist` surfaced as a syntax error six imports deep in a vendored
// file, which names neither the corpus nor the release nor the mismatch between them.
func corpusRelease(packages map[string]listedPackage, module string) int {
	for path, listed := range packages {
		if listed.standard || (path != module && !strings.HasPrefix(path, module+"/")) {
			continue
		}
		if release := releaseOf(listed.goVersion); release > 0 {
			return release
		}
	}
	return 0
}

// releaseOf reads the minor version out of a `go` directive such as "1.25.0", or 0.
func releaseOf(goVersion string) int {
	rest, ok := strings.CutPrefix(goVersion, "1.")
	if !ok {
		return 0
	}
	if index := strings.IndexByte(rest, '.'); index >= 0 {
		rest = rest[:index]
	}
	minor, err := strconv.Atoi(rest)
	if err != nil {
		return 0
	}
	return minor
}

// toolchainVersion is the `go1.N` the running toolchain is, for type-checking its own source.
//
// `runtime.Version()` reports the toolchain that BUILT this program, which is the same one whose
// GOROOT the resolver points at -- the extractor is run with `go run`, so the two cannot differ.
// A development build reports something that does not parse, and an unparseable version yields the
// empty string rather than a guess: the type-checker then applies its own default, which is the
// same answer this function would have to invent.
func toolchainVersion() string {
	version := runtime.Version()
	rest, ok := strings.CutPrefix(version, "go1.")
	if !ok {
		return ""
	}
	if index := strings.IndexFunc(rest, func(r rune) bool { return r < '0' || r > '9' }); index >= 0 {
		rest = rest[:index]
	}
	if _, err := strconv.Atoi(rest); err != nil {
		return ""
	}
	return "1." + rest
}
