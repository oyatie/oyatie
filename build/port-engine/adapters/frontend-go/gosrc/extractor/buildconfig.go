package main

import (
	"fmt"
	"go/build"
	"sort"
	"strconv"
	"strings"
)

// Which files are IN the package.
//
// A Go package is not "every `.go` file in a directory" — it is the file set a build
// CONFIGURATION selects, and the source says so three different ways: a `//go:build` line, the
// legacy `// +build` line, and the filename itself (`hostid_linux.go`, `sum_amd64.go`). Ignoring
// all three does not produce a bigger package, it produces a file set that no `go build` ever
// emits, and that fails in two ways.
//
// The LOUD way is a redeclaration: `xxhash_asm.go` and `xxhash_other.go` both define `Sum64`, so
// type-checking the union fails outright and the package yields no measurement at all. Three of
// eight surveyed corpora fail exactly here.
//
// The QUIET way is worse and is why this file exists. `pkg/errors/go113.go` is constrained to
// `go1.13` and declares `Is`, `As` and `Unwrap`. Nothing collided, so extraction SUCCEEDED and
// those three entered the snapshot as unconditional declarations of the package. They happen to
// be right for a recent toolchain and would be wrong for a configuration that excludes them, and
// nothing in the snapshot recorded that the question was ever asked.
//
// So the configuration is DECLARED rather than discovered. Reading it from the host's environment
// would make the snapshot depend on the machine that produced it, which is the one thing an
// engine built on a snapshot digest cannot afford: the same upstream commit would extract to two
// identities and the receipt would call an ordinary re-extraction `Unexplained`. Release tags are
// pinned for the same reason — `//go:build go1.13` otherwise resolves against whichever Go
// compiled this extractor.

// Default build configuration. Fixed constants, deliberately not the host's: a default that reads
// `runtime.GOOS` makes identity a property of the machine.
const (
	defaultGOOS    = "linux"
	defaultGOARCH  = "amd64"
	defaultRelease = 21
)

// buildConfig is the configuration a snapshot is extracted FOR. Every field is an input, so two
// configurations of one corpus are two snapshots rather than one snapshot with a hidden variable.
type buildConfig struct {
	goos    string
	goarch  string
	release int      // Go 1.N; selects release tags go1.1 through go1.N.
	tags    []string // Extra build tags, sorted; `purego`, `appengine` and friends.
}

// newBuildConfig parses the declared configuration. `tagList` is comma-separated.
func newBuildConfig(goos, goarch, tagList string, release int) (*buildConfig, error) {
	if goos == "" || goarch == "" {
		return nil, fmt.Errorf("build config: goos and goarch are required")
	}
	if release < 1 {
		return nil, fmt.Errorf("build config: go release must be 1 or greater, got %d", release)
	}
	tags := []string{}
	for _, tag := range strings.Split(tagList, ",") {
		if trimmed := strings.TrimSpace(tag); trimmed != "" {
			tags = append(tags, trimmed)
		}
	}
	// Sorted so one configuration has one spelling, here and in the identity that records it.
	sort.Strings(tags)
	return &buildConfig{goos: goos, goarch: goarch, release: release, tags: tags}, nil
}

// goVersion is the language version to type-check at, spelled as go/types wants it.
//
// Pinned to the same release as the tags: type-checking at whatever version compiled this
// extractor would accept syntax the declared configuration does not have.
func (c *buildConfig) goVersion() string {
	return "go1." + strconv.Itoa(c.release)
}

// describe is the canonical one-line spelling of this configuration.
func (c *buildConfig) describe() string {
	out := c.goos + "/" + c.goarch + " " + c.goVersion()
	if len(c.tags) > 0 {
		out += " tags=" + strings.Join(c.tags, ",")
	}
	return out
}

// context builds the go/build context that answers the selection question.
//
// CgoEnabled is FALSE. A file behind `import "C"` is not portable by this engine under any rule,
// and admitting it would put a declaration in the model whose body the front end cannot read.
// Excluding it deterministically is a refusal the snapshot can state; excluding it because the
// host happened to lack a C compiler is not.
func (c *buildConfig) context() *build.Context {
	ctx := build.Default
	ctx.GOOS = c.goos
	ctx.GOARCH = c.goarch
	ctx.BuildTags = append([]string(nil), c.tags...)
	ctx.CgoEnabled = false
	ctx.Compiler = "gc"
	ctx.UseAllFiles = false
	ctx.ReleaseTags = releaseTags(c.release)
	return &ctx
}

// releaseTags spells go1.1 through go1.N, which is what `//go:build go1.13` matches against.
func releaseTags(release int) []string {
	tags := make([]string, 0, release)
	for minor := 1; minor <= release; minor++ {
		tags = append(tags, "go1."+strconv.Itoa(minor))
	}
	return tags
}

// selectFiles returns the non-test `.go` files this configuration includes, sorted.
//
// Sorted because go/types' object ordering follows parse order, and an unsorted directory listing
// would make the snapshot a property of the filesystem.
func (c *buildConfig) selectFiles(dir string, names []string) ([]string, error) {
	ctx := c.context()
	selected := make([]string, 0, len(names))
	for _, name := range names {
		if !strings.HasSuffix(name, ".go") || strings.HasSuffix(name, "_test.go") {
			continue
		}
		match, err := ctx.MatchFile(dir, name)
		if err != nil {
			// A file this configuration cannot even read its constraints from is not a file to
			// silently skip: the alternative is a package that is quietly one declaration short.
			return nil, fmt.Errorf("match %s: %w", name, err)
		}
		if match {
			selected = append(selected, name)
		}
	}
	sort.Strings(selected)
	return selected, nil
}
