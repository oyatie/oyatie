# Bootstrap Go front end (out-of-band only)

This tree is **not** part of the Rust build. Nothing under `gosrc/` is compiled, linked,
or read by any engine crate, and no engine code path may invoke it. It exists so the
`SourceModel` snapshot the engine consumes is produced from real Go by real Go tooling
rather than hand-written.

ADR-0638 D3 draws the firewall: the bootstrap extractor (`go/parser` + `go/types`) runs
out of band, and the engine's `verify()` path consumes only the resulting artifact.
`port-engine-frontend-go`'s architecture tests enforce the Rust half of that — its library
sources may not spawn `go` or import `std::process::Command`, and may not name this tree.

## Layout

| Path | What it is |
|---|---|
| `go.mod` | Fixture module. **No `require` block, no `go.sum`** — stdlib only, by design. |
| `extractor/main.go` | The bootstrap extractor. Reads a corpus, writes a snapshot envelope. |
| `corpus/basic/` | Constants, variables, a type alias, a named type, functions. |
| `corpus/shapes/` | A struct with fields and methods, an interface with a method set. |
| `corpus-refused/hard/` | A `for` loop and a `defer` — the fixture the engine must REFUSE. |

The corpus is deliberately small, hermetic, and **not Kubernetes**. Kubernetes is the
program's W1 corpus and admitting it is a separate decision; this fixture exists to prove
the pipeline translates Go at all.

## Regenerating the snapshot

```sh
cd build/port-engine/adapters/port-engine-frontend-go/gosrc
go run ./extractor \
    -corpus ./corpus \
    -module oyatie.example/portengine-fixture \
    -out ../../port-engine-snapshot/src/fixture-snapshot-v1.json
```

And the refusal corpus, which is kept separate so the pipeline over `corpus/` stays green while
the refusal path is still exercised against real Go rather than against synthetic nodes:

```sh
go run ./extractor \
    -corpus ./corpus-refused \
    -module oyatie.example/portengine-fixture-refused \
    -out ../../port-engine-snapshot/src/fixture-snapshot-refused-v1.json
```

Both snapshots ADMIT. A model of source the translator cannot handle is not itself invalid — the
extractor records an untranslatable construct as an `unsupported` node naming the AST type it
stands for, and the refusal happens at the transform, where the construct can be named. Dropping
it here instead would make an untranslatable function indistinguishable from an empty one.

Run it twice and diff the two outputs — the artifact is byte-stable by construction
(packages sorted by unit id, declarations in `go/types` scope order, struct fields left in
declaration order because that order is semantic in Go). A non-empty diff is an extractor
defect, and the Rust side refuses a mismatched pair through
`port_engine_snapshot::admit_reproducible_pair` rather than trusting one pass.

## The digest is not the JSON

`snapshot_digest` is SHA-256 over a length-prefixed, explicitly-arity-tagged encoding of
the model — never over the JSON bytes. The encoder is documented at the bottom of
`extractor/main.go` and mirrored by `port_engine_snapshot::snapshot_preimage_v1`.

Mirroring the encoder in two languages is a deliberate trade. The alternative — trusting
whatever digest the extractor claims — would let a front-end defect enter the engine with a
self-consistent receipt. Here, any drift between the Go encoder and the Rust one surfaces
as `AdmitError::DigestMismatch` at admission, so the duplication is checked on every run
instead of being an assumption.
