// Hermetic fixture module for the port engine's bootstrap Go front end.
//
// DELIBERATELY DEPENDENCY-FREE. The extractor uses only the Go standard library
// (go/parser, go/token, go/types) and never golang.org/x/tools/go/packages, so this
// module has no `require` block and no go.sum. Nothing here enters the Rust crate
// graph, oya-deps.toml, or deny.toml.
module oyatie.example/portengine-fixture

// The release the extractor is CONFIGURED for, which it treats as a ceiling: a corpus declaring a
// later one is refused rather than silently checked below what its own module requires. This module
// declared go1.24 while extraction ran at the configured release, so `regen-fixtures.sh` had been
// failing since that guard landed, and the declaration is what moved.
//
// go1.22 is the engine's minimum supported release, and the reason is loop-variable scoping: 1.22
// gave each iteration its own variables, so a corpus checked at the wrong release differs in what a
// closure captures -- same syntax, different program.
go 1.22
