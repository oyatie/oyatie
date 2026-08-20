package main

import (
	"encoding/json"
	"fmt"
	"os/exec"
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
func dependencyDirs(corpus string) (map[string]string, error) {
	cmd := exec.Command("go", "list", "-deps", "-json", "./...")
	cmd.Dir = corpus
	out, err := cmd.Output()
	if err != nil {
		// NOT fatal. A corpus whose dependencies do not resolve still type-checks if its imports
		// are all standard or intra-corpus, which is exactly the case every fixture is in. Failing
		// here would make the fixtures depend on a working module cache to load at all.
		return map[string]string{}, nil
	}
	dirs := map[string]string{}
	decoder := json.NewDecoder(strings.NewReader(string(out)))
	for decoder.More() {
		var listed struct {
			ImportPath string
			Dir        string
			Standard   bool
		}
		if err := decoder.Decode(&listed); err != nil {
			return nil, fmt.Errorf("go list output: %w", err)
		}
		// The STANDARD library is left to its own importer, which reads it faster and is the one
		// the compiler uses. Only what that importer cannot reach is recorded here.
		if listed.Standard || listed.Dir == "" || listed.ImportPath == "" {
			continue
		}
		dirs[listed.ImportPath] = listed.Dir
	}
	return dirs, nil
}
