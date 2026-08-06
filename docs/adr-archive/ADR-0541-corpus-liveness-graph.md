---
id: ADR-0541
title: "Corpus Liveness Graph: one content-addressed corpus graph with per-class decay invariants"
status: Superseded
planning_impact: true
deciders: founder
date: 2026-06-10
door: one-way
owner: council-architecture
supersedes: []
superseded_by: [ADR-0709]
depends_on: [ADR-0139, ADR-0132, ADR-0515, ADR-0538, ADR-0539, ADR-0540]
amends: []
related: [ADR-0083, ADR-0111, ADR-0116, ADR-0363, ADR-0516]
related_specs:
  - /specs/root-hub-pointers.json
milestone: W2
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


> **Disposition light-edit (2026-08-06):** Context re-triage Accept: Corpus liveness graph

# ADR-0541: Corpus Liveness Graph — one content-addressed corpus graph with per-class decay invariants

## Status

**Proposed - 2026-06-10 (authored for founder sign-off; door: one-way).**

## Context

The founder directive of 2026-06-10 names a family of symptoms with no fundamental mechanism:
missed directives from accepted ADRs, directive drift, lossy context across sessions, staleness,
dead code, dead files, and unmaintained docs. The G011 session record shows the point-gate
treadmill this produces: every decay symptom spawns its own gate (manifest-hygiene, bnf-suffix,
staleness-reaper, brand-residue, workspace-glob-coverage, freshness, target-parity, ...), each
with its own enumeration logic, baseline, and registration surface.

The reframe (recommendation record, idea-refine 2026-06-10): every symptom is one graph pathology.
Dead code is an orphan node; a stale doc is a broken edge; directive drift is a missing compliance
edge; unmaintained is a node past its freshness budget. If the whole corpus — code symbols, files,
docs, ADRs, directives, tests, product surfaces — is one content-addressed, AST-derived graph,
then decay control is a small parameterized family of invariants over that graph instead of an
unbounded fleet of bespoke gates.

### Precedent evidence (deep-research corpus, 25/25 claims verified 3-0, persisted at `.omc/research/corpus-liveness-precedents-20260610.json`)

Per the proven-patterns doctrine (founder 2026-06-09: adopt hyperscaler methodology, reimplement
Rust-native, cite precedent per decision), the decision below is split by evidence strength.

**Strong production precedent — adopt directly:**

1. **One corpus-scale semantic graph** over a monorepo, unifying build metadata, compiler
   metadata, and cross-references: Google Kythe (production at google3 scale),
   Meta Glean (production across C++/Python/PHP/JavaScript and five further languages;
   open-sourced 2021), SCIP/LSIF (commodity cross-repo precise indexing).
2. **Typed heterogeneous-asset dependency graph driving automated decay removal**: Meta SCARF —
   one directed graph spanning code symbols AND data assets, 100M+ LOC deleted, 7000+ complex
   subgraph removals; Google Sensenmann — build-target-granularity GC, 1000+ deletion CLs/week,
   ~5% of all C++ deleted.
3. **Continuously re-evaluated deletion invariants with staged quarantine**: SCARF's candidacy
   invariant (no inbound blocking edges, no runtime usage, homogeneous type) must keep holding at
   every system run during the waiting window or removal aborts; quarantine precedes enactment,
   with an engineer revert path.
4. **Mechanical doc-freshness detection**: Google g3doc — docs colocated in the monorepo, owned,
   review-gated, with freshness dates and automated reminder tooling; 30% of doc changelists
   co-change code.

**Documented production failure modes — design around, not into:**

5. **Pure static/AST liveness is UNSOUND for deletion.** SCARF incorrectly deleted live code that
   reached production (reflection, string-built identifiers, factory registration). Meta's
   mitigation: runtime usage telemetry + 300+ hand-written pattern detectors + deliberately
   conservative textual-search edges, trading precision for safety. Any one-shot fail-closed
   deletion gate on AST reachability alone repeats a failure Meta already paid for.
6. **Incremental indexing floors at O(fanout), not O(changes)** (Glean): renaming a widely-used
   symbol re-indexes every dependent. Budget accordingly; content-addressing bounds re-parse, not
   re-derivation of inbound edges.
7. **Opaque numeric node identity was a design failure** (LSIF → SCIP transition): identity must
   be human-readable, stable symbol IDs. The W2 content-addressed AST supplies content identity;
   the graph's public node IDs must additionally be readable and stable across formatting moves.
8. **Fail-open vs fail-closed over the index itself**: Kythe deliberately degrades gracefully on
   incomplete data. A merge-blocking invariant over an incomplete index is a false-red machine.

**No production precedent found (novel risk — staged, never born-blocking):**

9. Docs and directives as first-class NODES in the same dependency graph as code.
10. Fail-closed merge-blocking CI invariants over doc staleness / doc-to-code reference integrity
    (Google's strongest practice is review culture + email nags; their own book concedes tooling
    "has still not caught up").
11. Directive-compliance edges (accepted decisions compiled into required graph edges).

## Decision

Adopt the Corpus Liveness Graph (CLG) as the W2-milestone decay substrate, with the following
binding shape:

### D1. One graph, owned substrate

One content-addressed corpus graph: nodes at every granularity (ADR/doc → file → folder →
module/crate → symbol → statement → token/format projection), edges derived by parse (never
hand-maintained), faces CI-materialized (never hand-rotted; the ADR-0539 freshness discipline
applies to the graph face itself). Substrate = the W2 owned Rust AST (bespoke rowan-style per the
founder decision of 2026-06-09) + the existing registry/ knowledge layers; Kythe/Glean/SCIP are
methodology precedents, reimplemented Rust-native, not dependencies. Node identity: content hash
for change detection AND a human-readable stable symbol ID for reference (the SCIP lesson, #7).

### D2. Invariant families, parameterized — not per-symptom gates

Exactly six conformance classes, each one invariant kind parameterized per node type:
1. liveness/orphan (no inbound reachability edge),
2. reference integrity (broken/hash-moved edge),
3. format conformance (token-form ≠ canonical-format projection),
4. template conformance (structure ≠ declared template shape),
5. freshness (last-validated past the node-class budget),
6. directive compliance (work in a directive's scope without a satisfied compliance edge).

The existing gate fleet (manifest-hygiene, bnf-suffix, format gates, staleness-reaper,
workspace-glob-coverage, target-parity, ...) are special cases; each migrates to a CLG rule-table
row in its own IP with a verified-equivalence step (gate retired only after its CLG invariant
proves byte-equivalent verdicts on the live corpus — the ADR-0363 migration discipline).

### D3. Per-class enforcement posture (the honest core of this decision)

Enforcement follows evidence, not uniform fail-closed:

- **Structural classes on code (reference integrity, format, template):** fail-closed
  born-blocking in `oya-ci-required`. These are precedented as code gates and are already
  enforced today; CLG only unifies their substrate.
- **Liveness/deletion classes:** SCARF posture, never one-shot. Candidacy = continuously
  re-evaluated invariant (static reachability + conservative textual edges + runtime telemetry
  once cloud-observability supplies it per ADR-0139 agentic SLO-gated promotion); staged
  quarantine (visibility revocation
  before removal) with revert path; human review on the removal PR. CI blocks NEW orphans
  (baseline-block-on-new, the proven G011 ratchet machinery) but never auto-deletes.
- **Doc/directive classes (the unprecedented part):** staged rollout per class —
  report-only face → baseline-block-on-new → born-blocking — promoted only on measured
  false-positive rate at each stage. This is the calibration mechanism Google lacked and this
  repo has already proven three times in G011 (#662, #664, #665). Directive-compliance edges
  begin with the mechanizable subset (directives that compile to Cedar policies or gate
  predicates); the semantic residual stays a directive-coverage review axis (the adversarial
  review discipline), explicitly out of mechanical scope.
- **Index integrity:** the graph face carries a completeness attestation; invariants evaluate
  fail-closed ONLY over corpus regions the attestation covers, and fail-open-with-report
  elsewhere (resolves the Kythe tension, #8: never block merges on index gaps, never silently
  skip attested regions).

### D4. Minimal viable slice (first IP, W2 spike)

One service's crate set + its ADRs/docs: build the graph face, implement TWO invariants
end-to-end — reference integrity (doc→code hash edges; class with the clearest precedent gap to
close) and liveness/orphan in report-only — and run the assumption test from the recommendation
record: classify the friction ledger as of the spike's HEAD (56 rows at this ADR's authoring) as
graph-expressible vs needs-judgment.
Exit criteria: O(fanout) incremental cost measured against a threshold the IP declares up front
(recorded in the IP before the spike runs, council sign-off); zero false-red on the attested
region; the classification result documented in the IP.

## Consequences

- The N-gates fragmentation collapses toward one substrate + a rule table; new decay classes
  become rule rows, not new crates (ADR-0132 single-concern is preserved: the CLG substrate is
  one concern — corpus decay — and the per-class rules are configuration, not services).
- Deletion automation arrives staged and telemetry-gated, not at W2; until cloud-observability
  runtime-usage signals exist, liveness invariants ratchet (block new debt) and report, only.
- Doc/directive gating is explicitly experimental: each class earns fail-closed status with
  measured false-positive evidence, or stays a ratchet. If the novel classes prove noisy, the
  fallback posture (ratchet + review axis) is still strictly stronger than today's email-nag
  state of the art.
- Negative: the graph face is a new generated artifact with real build cost (O(fanout) floor);
  the W2 AST becomes load-bearing for governance, raising its quality bar; two sources of truth
  exist during gate migration (mitigated by verified-equivalence retirement per D2).
- Self-referential enforcement: this ADR's own scope declaration makes CLG work surface ADR-0541
  as its governing directive once D3's compliance edges exist — the fix for directive drift is
  itself a ratified, enforced directive.

## Compliance

Until the CLG face exists, compliance = the D4 spike IP lands with its exit criteria; the
decision-crosswalk face registers this ADR mechanically. After D4, CLG invariant promotion
(report-only → ratchet → born-blocking) requires a per-class evidence note appended to the IP —
never a silent disposition flip.

### Addendum 2026-06-30 — WorkAreaTree parser implementation surface

The first Rust-native WorkAreaTree parser slice for the W2 owned AST substrate is governed by
this ADR. The tracked surfaces are:

- `governance/corpus/work-area-rust-parser/BUCK`
- `governance/corpus/work-area-rust-parser/Cargo.toml`
- `governance/corpus/work-area-rust-parser/OWNERS`
- `governance/corpus/work-area-rust-parser/fixtures/minimal_function.rs.txt`
- `governance/corpus/work-area-rust-parser/src/lib.rs`
- `registry/catalog/work-area-rust-parser.yaml`
- `evidence/multispectrum/work-area-rust-parser-20260630-1782840154.json`
### Addendum 2026-06-30 — Markdown document parser implementation surface

The first Rust-native Markdown document parser slice for the W2 owned corpus graph substrate is
governed by this ADR. The tracked surfaces are:

- `governance/corpus/doc-parser/BUCK`
- `governance/corpus/doc-parser/Cargo.toml`
- `governance/corpus/doc-parser/src/lib.rs`
- `governance/corpus/doc-parser/tests/doc_parser_contract.rs`
- `governance/corpus/doc-parser/tests/fixtures/adr-heading-reference.md`
- `governance/corpus/doc-parser/tests/fixtures/adversarial-exfil.md`
