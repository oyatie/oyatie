# oya-friction-ledger-merge-driver

Structural three-way Git merge driver for `.omc/ultragoal/friction-ledger.jsonl`
(FRIC-1781370000).

## Why

Three merge-train leader incidents in one session, all in the ledger's trailing-line region:

1. two lanes both authored PRIMARY rows for one FRIC id; the union kept both and the
   friction-accounting gate (ADR-0544) failed closed on `friction_duplicate_primary_row` —
   correct gate, wrong union;
2. a hand-rolled union crashed mid-resolution and committed raw conflict markers;
3. an exact-line dedup pass mangled byte-divergent-but-logically-identical rows.

## Semantics

- **Union, id-aware:** base rows preserved in base order; additions append ours-then-theirs.
  Logical identity is parsed-JSON equality (realized as canonical-byte equality), never raw
  bytes, so byte-divergent twins collapse to one row.
- **Second-author conversion:** an id keeps exactly one PRIMARY. The base primary wins; among
  new primaries the earliest author wins by content (`(seen_at, canonical bytes)` minimum, never
  the merge side). Losers convert to the event-sourced update row
  `{id, seen_at, status_update: <its status>, evidence: <its evidence + enforcement_fix>,
  story/goal carried}`.
- **Canonical output:** rows are serialized through the ADR-0546 canonical-json kernel
  (`oya-cloud-ci-canonical-json-app::canonicalize`, the single owner of escaping, key order, and
  number lexemes; `sort_keys=true`, literal UTF-8, LF) plus this crate's one documented
  projection from the kernel's pretty form to single-line JSONL (sound because the kernel
  escapes every control character inside strings). The first driver-mediated merge normalizes
  the whole file once; afterwards the canonical form is a fixed point.
- **Fail-closed (ADR-0548 D7):** any side that is not a modeled ledger (invalid JSON, non-object
  row, blank id, duplicate object keys, unmodeled row shape, a base already carrying duplicate
  primaries) refuses the merge with a nonzero exit and leaves `%A` untouched — git falls back to
  a normal conflict; the driver never writes garbage. Before writing, the driver self-validates
  its own output: reparse as a modeled ledger, canonical idempotence, single primary per id, row
  conservation, and no new orphan-update ids.

## Known semantics (disclosure) and the authoring rule

- **Committed-history logical-duplicate collapse.** Dedup is logical-set union over the whole
  document, committed history included: canonically-identical update rows for one id collapse
  to one on any merge, even when non-adjacent — an `A,B,A` update interleave collapses to
  `A,B`, silently rewriting the ADR-0544 latest-update fold (effective status `A` becomes `B`;
  e.g. a re-logged `fix-in-flight` after a `RESOLVED` flips the fold to `RESOLVED`). The live
  corpus carries zero such duplicates and the red direction fails closed; the green-to-green
  rewrite is disclosed here and in ADR-0558.
- **Authoring rule.** A re-logged transition — a reopen after a terminal status, or any repeat
  of an earlier status for the same id — must differ in content (a fresh `seen_at` or
  `evidence`) so it is a distinct logical event rather than a dedup target.
- **Orientation note.** Concurrent divergent `status_update` rows for one id have no total
  order across merge orientations: the row set is identical either way, but the gate's
  physical-order fold takes the last update, which depends on which side was ours. Inherent to
  folding parallel appends; no worse than the text union this replaces.

## Activation

Build the driver:

```bash
buck2 build //tools/oya-friction-ledger-merge-driver-app:oya-friction-ledger-merge-driver
```

Point Git at the built binary (per clone — git config is not versioned):

```bash
git config merge.friction-ledger.name "Oyatie friction-ledger structural merge"
git config merge.friction-ledger.driver "oya-friction-ledger-merge-driver %O %A %B"
```

The `.gitattributes` entry is:

```gitattributes
/.omc/ultragoal/friction-ledger.jsonl merge=friction-ledger
```

## Enforcement layering (honest scope)

This driver is the LOCAL automation layer: it only helps actors who configured the
`merge.friction-ledger` git config. Merge authority stays with the cloud-ci gate apps behind the
single required context `oya-ci-required` (ADR-0515); the ADR-0544 friction-accounting gate
remains the canonical backstop that fails closed on whatever an unconfigured actor merges by
hand. The rule holds with the driver absent.

Talos-era successor: under the agentic delivery fabric this becomes server-side merge
intelligence in the ADR-0515 cloud-ci/oya-ci Tide admission path (the merge queue resolves
ledger unions centrally instead of per-clone git config). This crate follows the
`tools/oya-cargo-lock-merge-driver-app` precedent (FRIC-1781069288) — no new doctrine — and is
registered by ADR-0558, the ADR-0555 D2 ownership + justification record, which also pins these
semantics and the two-way door (attribute + crate deleted at Tide cutover); doctrine citations:
FRIC-1781370000, ADR-0544 (ledger fold), ADR-0546 (canonical form), ADR-0548 D7 (fixer
self-validation).
