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
	// The engine's MINIMUM SUPPORTED release, raised from 21 deliberately.
	//
	// go1.22 gave each loop iteration its own variables. A corpus written for 1.22 and checked at
	// 1.21 is the same syntax and a different program -- a closure made in a loop captures one
	// shared variable under the old rule and a fresh one under the new -- and nothing downstream
	// can see the difference. Supporting 1.21 would mean modelling BOTH capture rules and deciding
	// per module which applies; making 1.22 the floor deletes the choice instead of answering it.
	//
	// The cost is stated rather than hidden: a module declaring go1.21 or earlier is now refused by
	// the ceiling check, which is the correct outcome for a release whose semantics this engine
	// does not implement.
	defaultRelease = 22
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
	// The TOOL TAGS are stated, because they select files and are not derivable from the rest of
	// this line by a reader who does not have this table.
	if tags := toolTags(c.goarch); len(tags) > 0 {
		out += " tooltags=" + strings.Join(tags, ",")
	}
	return out
}

// toolTags is the microarchitecture baseline for a declared GOARCH.
//
// `go/build` computes these at init FROM THE HOST, and they gate real files -- `arm64.v8.0` and
// `amd64.v1` each select different sources in the standard library and in `x/sys/cpu`. Inheriting
// them means the same commit extracts to a different snapshot on an arm64 machine than on an amd64
// one, which is the failure the receipt exists to prevent and the same one the Go release was.
//
// So they are DECLARED, from the architecture the caller asked for. The goexperiment tags
// `build.Default` also carries are deliberately NOT reproduced: they are properties of the
// toolchain that built the extractor rather than of the configuration being described, and a file
// selected because of how this binary was compiled is not a file the snapshot can account for.
//
// The table is CLOSED. An architecture with no entry gets no tool tags rather than the host's,
// because a wrong tag selects the wrong file and an absent one selects the baseline.
func toolTags(goarch string) []string {
	switch goarch {
	case "amd64":
		return []string{"amd64.v1"}
	case "arm64":
		return []string{"arm64.v8.0"}
	case "386":
		return []string{"386.sse2"}
	case "arm":
		return []string{"arm.5"}
	case "mips", "mipsle":
		return []string{"mips.hardfloat"}
	case "mips64", "mips64le":
		return []string{"mips64.hardfloat"}
	case "ppc64", "ppc64le":
		return []string{"ppc64.power8"}
	case "riscv64":
		return []string{"riscv64.rva20u64"}
	default:
		return nil
	}
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
	// DECLARED rather than inherited. See toolTags: `build.Default` fills these in from the host,
	// and they select files.
	ctx.ToolTags = toolTags(c.goarch)
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
