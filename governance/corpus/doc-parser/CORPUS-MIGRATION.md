# ADR decision-IR corpus admission plan

Status: **DORMANT / NOT ADMITTED / `HOLD(Planning)`**

This is an evidence-closure plan for the parser foundation. It is not a product
roadmap, an implementation-dispatch authority, a corpus repair authorization,
or a release of `HOLD(Planning)`. The strict parser and IR renderer may land as
inactive foundations, but no current-authority consumer may use a successfully
parsed subset of the ADR corpus.

## Boundary now

- Active inputs are exact repository-relative direct children of
  `docs/decisions/` named `ADR-*.md`. The parser rejects absolute paths,
  subdirectories, and alternate roots.
- Historical material belongs under a path whose name explicitly contains
  `archived`, or only in Git history. It is outside the active input set and
  must not be rediscovered as current authority by a recursive scan.
- Canonical typed fields are `id`, `status`, `date`, `owner`, lifecycle
  relations (`depends_on`, `supersedes`, `superseded_by`, `amends`,
  `amended_by`, `related`), `affected_surfaces`, and `deliverables`.
- Typed relation values are exact `ADR-NNNN` identifiers. Filenames, prose,
  comments, and approximate identifiers are rejected, never coerced.
- Generic scalar, flat-list, and folded/literal block-scalar metadata remains
  available with its exact raw source value and byte span.
- Generic nested maps and sequences of maps are not yet representable. They
  fail closed rather than being dropped, flattened, or misclassified.
- `oya-check-adr-index` retains its legacy record adapter. Its IR renderer is a
  pure dormant candidate; the dev CLI and cross-artifact agreement consumers
  have not been cut over in this lane.
- Generated JSON faces are untouched. The checked-in materializer remains the
  only permitted writer for generated faces.

## Reproducible census receipts

Both receipts below are blockers. Their mismatch is itself an unresolved
evidence gap; neither can support admission.

### Stage-1 coordinator receipt

The coordinator's controlling readiness receipt reports 429 direct current ADR
files, 193 parsed, and 236 rejected. Its parser hashes, complete failing-path
ledger, and exact harness were not supplied to this implementation lane, so the
nine-file difference from the receipt below cannot be truthfully attributed.
The 236 failures remain the controlling minimum blocker until the receipts are
reconciled against identical parser bytes and selection semantics.

### This foundation snapshot

- Base and `origin/dev`: `1cdd04d1687d0f171a1432c8418072456baab6d0`
- `docs/decisions` tree: `17f358e55f65a9f27b848a085e7166c2ca6f19d9`
- Direct `docs/decisions/ADR-*.md` inputs: 429
- `src/adr.rs` SHA-256:
  `0c15ba06b1a0ab8e8b4b2014e34f0804c902e6fb6fa06b0ea472f50377f2a2ae`
- `src/lib.rs` SHA-256:
  `af52d3d97a55cf00e7743e4a0b67f16cab95bd6af08b7f502029b47462b08831`
- Parsed: 184
- Rejected: 245

The harness sorts direct children, reads each file as UTF-8, supplies
`docs/decisions/<filename>` to `parse_adr_decision`, and counts every returned
error as a rejection. The rejection variants are:

| Parser result | Count |
| --- | ---: |
| `MissingRequiredField` | 142 |
| `UnsupportedFrontmatterNesting` | 45 |
| `InvalidAdrReference` | 28 |
| `MissingLeadingFrontmatter` | 26 |
| `InvalidFrontmatter` | 4 |

Named regression fixtures preserve three representative population defects:
missing required metadata, a filename-shaped typed relation, and generic nested
metadata that the IR cannot yet retain losslessly.

## Admission sequence under HOLD

1. Freeze one protected `origin/dev` commit, parser byte hashes, the exact
   direct-child selector, and a path-by-path parse result ledger. Store the
   ledger as immutable evidence; do not hand-edit a generated face.
2. Reconcile the 236- and 245-rejection receipts on those identical inputs.
   Classify every path as parser-shape work, corpus defect, or qualified-owner
   disposition. A count without the path ledger is insufficient.
3. Add RED fixtures before each grammar change. Model legitimate generic nested
   metadata as lossless typed/raw nodes with complete spans, or repair the
   source through a separately owned ADR/corpus lane. Do not relax exact typed
   relation validation to make legacy prose parse.
4. Reach complete current-population closure: every expected active path parses,
   or an explicit qualified authority has moved it to an `archived` path or Git
   history with custody and successor evidence. Partial population rendering is
   prohibited.
5. Migrate the dev CLI and cross-artifact agreement consumers in separately
   owned, regression-first lanes. Remove ad hoc parsing only after each consumer
   proves it consumes the same immutable IR population.
6. Materialize the Markdown and JSON projections only through their canonical
   generator and prove byte-for-byte parity before cutover. No generated face
   may be edited by hand.
7. Obtain independent fresh-context review of the population ledger, source-set
   freshness/archive boundary, parser behavior, consumer closure, projection
   bytes, and evidence custody. This parser lane cannot grant that authority.

## Context-free exit gate

Admission remains blocked unless an independent verifier can prove all of the
following from the packet alone:

- The protected commit, corpus tree, parser hashes, harness, expected active
  path set, and per-path results are exact and reproducible.
- The active decision population is complete: zero missing required fields,
  malformed typed relations, unsupported structures, or silent metadata loss.
- Every retained field has stable semantic value plus exact raw/span
  provenance; source bytes and content hashes are stable and tenant-independent.
- Archived/history-only material is absent from the active direct-child set and
  has explicit custody and successor evidence where required.
- Every current index consumer uses the canonical IR; the transitional record
  adapter has no live producer.
- Canonically materialized Markdown/JSON bytes match the admitted projections,
  Cargo and Buck tests pass, clippy is warning-free, and generated-face policy
  reports no hand edits.
- Qualified authority has independently satisfied the wider Stage-1 legal,
  affected-party, operations, custody, veto, pilot, successor, council,
  fresh-dissent, and context-free gates.

Until then, the only truthful terminal state is **BLOCKED / `HOLD(Planning)`**.
