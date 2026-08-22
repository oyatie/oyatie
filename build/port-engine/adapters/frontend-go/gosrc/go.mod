// Hermetic fixture module for the port engine's bootstrap Go front end.
//
// DELIBERATELY DEPENDENCY-FREE. The extractor uses only the Go standard library
// (go/parser, go/token, go/types) and never golang.org/x/tools/go/packages, so this
// module has no `require` block and no go.sum. Nothing here enters the Rust crate
// graph, deps.toml, or deny.toml.
module oyatie.example/portengine-fixture

go 1.24
