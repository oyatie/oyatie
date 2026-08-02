---
doc_status: drafted
---

# Spec: Data-Classification Fact Layer

> Status: DRAFT — Phase 1 (Specify) of spec-driven development. Not ratified. No ADR yet.

## Objective

Compliance obligations are currently expressed as **prose replicated per service**: `dpia.md` exists
in **87 copies**, one per service, differing by name. Adding EU, US and SOC2 regimes to that shape
means 87 × N documents, none of them queryable and all of them drifting independently.

Replace it with three layers:

| Layer | Content | Cardinality | Authored? |
|---|---|---|---|
| **Facts** | per-field: what the data IS — kind, tier, encryption, retention clock source | O(fields), **jurisdiction-independent** | yes, at the type |
| **Policy** | per-regime rules evaluated against facts | O(regimes) | yes, as data |
| **Projection** | DPIA · RoPA · SOC2 evidence · deletion manifests | O(regimes × services) | **never — derived** |

A field's nature ("this column holds a national ID") is intrinsic and regime-independent. What varies
per regime is the *rule*: PIPA §21③ separated retention, GDPR Art.17 erasure, CCPA deletion — different
clocks, different carve-outs, same underlying fact.

**Success = the DPIA becomes a regenerated projection, and no compliance document is hand-authored.**

## Decisions taken (founder, 2026-08-01)

1. **Facts are ANNOTATED AT THE TYPE and extracted** — not a central registry, not DB catalog comments.
   The fact is born with the field, so drift is impossible by construction. Extraction lifts it into
   the code graph.
2. **All four regimes in scope**: EU GDPR · Korea PIPA/Credit Act · US CCPA/CPRA · SOC2 evidence.
   Designing to GDPR first tends to subsume the others; SOC2 exercises the projection path at lower stakes.
3. **First slice is the COVERAGE GATE** — prove every field carries classification before generating
   any compliance output. Born-advisory, shrink-only. This is the ratchet pattern that has worked
   repeatedly in this repo.

## What already exists (build on, do not rebuild)

- `audit/core/retention-cascade-domain` + `audit/ports/retention-cascade-api` — the retention ENGINE
- **814 `.cedar` policy files** + cloud-iam PDP — the policy EVALUATOR
- `audit/` append-only Merkle audit chain
- Durable-store RLS enforcement, fail-closed on `rolsuper`/`rolbypassrls`
- Compliance capability + `dpia/` dirs per service

**The engine and the evaluator were both built. The fact layer they should read is what is missing.**

## Design

### Facts — a proc-macro attribute on the field

```rust
#[derive(DataClassified)]
pub struct UserRecord {
    #[data_class(kind = pii_national_id, tier = C2,
                 retention = relationship_end + years(5),
                 encryption = per_tenant_dek,
                 exportable, maskable)]
    pub national_id: NationalId,

    #[data_class(kind = non_identifying, tier = C0)]
    pub locale: Locale,
}
```

- `kind` — closed enum. The single most important field; everything downstream keys on it.
- `tier` — storage/isolation class (C0..C6 in the reference model).
- `retention` — a CLOCK SOURCE plus offset, never an absolute date. "5 years from relationship end"
  is a rule; "2031-04-02" is a computed answer and does not belong in source.
- `encryption` — names the key scope, which is what makes crypto-shredding expressible.
- `exportable` / `maskable` — portability and masking capability, distinct from permission.

The derive emits a const fact table per type. Extraction reads that, so facts reach the graph
**without needing a live database**.

### Policy — per-regime packs as data

Cedar, reusing the existing substrate rather than introducing a second evaluator. One pack per regime;
a rule reads facts and emits an obligation (`must_erase_by`, `must_isolate`, `must_not_cross_border`).

### Projection — derived, never authored

DPIA/RoPA/SOC2 evidence generated from facts × policy. Emitted as a CI artifact, **never committed**
— committed generated faces are a merge surface that a mis-invoked materializer corrupts silently
(this repo de-committed 7 of them for exactly that reason).

## Commands

```
build: buck2 build //<capability>/...
test:  buck2 test //ci/facade/data-classification-coverage:ci-data-classification-coverage-{unittest,gate}
gate:  buck2 run  //ci/facade/data-classification-coverage:...-bin -- --repo-root .
```

## Project structure

```
libs/data-classification-kernel/     pure: fact types, enums, serialization. ZERO I/O.
libs/data-classification-derive/     the proc-macro
ci/facade/data-classification-coverage/   the coverage gate + policy JSON + frozen baseline
specs/compliance-regimes/<regime>/   per-regime Cedar policy packs (LATER slice)
```

Kernel purity is enforced by an existing gate — a kernel that does I/O REDs CI.

## Testing strategy

buck2 `rust_test`, colocated. Every gate assertion proven RED before GREEN.

- **kernel**: fact parsing/serialization, byte-stability (canonical output; a freshness gate compares bytes)
- **derive**: a field with no `#[data_class]` fails the derive; an unknown `kind` fails to compile
- **gate**: coverage computed from the live corpus, with a known-positive AND known-negative control —
  a "0 unclassified" result must be provably real, not an empty scan

## Boundaries

**Always** — facts jurisdiction-independent; retention as clock+offset never absolute; kernel I/O-free;
gate born-advisory shrink-only; projections generated, never committed.

**Ask first** — adding a `kind` to the closed enum; anything touching live data paths; any
cluster-scoped action (the Talos cluster is SHARED with the console project).

**Never** — hand-author a DPIA once projection exists; commit a generated projection; classify a field
by guessing its regime treatment (facts describe the DATA, policy describes the REGIME); let the gate
pass on zero observations — zero observations is not evidence of zero violations.

## Success criteria

1. Coverage is COMPUTED as classified-fields / total-persisted-fields, validated on both a
   known-positive and a known-negative control.
2. The gate REDs on a new unclassified persisted field. Proven by adding one.
3. Baseline is shrink-only; a regression cannot be laundered by regenerating it.
4. Facts reach the code graph via a buck2 action (per the everything-in-the-graph northstar).
5. Zero compliance prose authored in this slice — it establishes measurement only.

## Resolved by hyperscaler precedent

These were open; all four are decidable from what Google/AWS actually built.

**1. A persisted field is one that crosses a STORAGE PORT.** Making the port the enforcement point is
how this is made tractable at scale: you cannot persist except through a typed store interface, so the
port's associated types define the universe MECHANICALLY rather than by a scanner guessing which
structs are persisted. This repo already has storage ports per capability.
*Consequence:* SQL-first tables that bypass a port are a SEPARATE finding — **unwired persistence** —
not a classification gap. That keeps the coverage denominator honest instead of silently excluding
what it cannot see, which is the blind-spot class this repo keeps rediscovering.

**2. Proto gets field options; Cedar and Terraform are OUT OF SCOPE by definition.**
Google annotates `.proto` fields directly with data-policy/semantic-type field options — field-level
classification in the schema language is their actual mechanism, not a workaround. So `.proto` carries
the same fact schema via field options, and the extractor reads both.
Cedar and Terraform do not persist user data — they are policy and infrastructure. Out of scope by
what they ARE, not deferred pending effort.

**3. The 87 `dpia.md` retire on projection parity.** Git IS the historical record; retaining the file
to preserve history is redundant with the thing that already preserves it. Nobody maintains
hand-authored compliance documents alongside a generator — the generator becomes the source and the
document becomes output. Parity (projection reproduces the substantive content) is the gate; an ADR
records the retirement, because they are governance artifacts.

**4. GDPR first — because it demands the most FACTS, not because it is most urgent.**
This is the load-bearing reframe: **facts are the expensive layer, policy is cheap.** GDPR requires
lawful basis, purpose limitation, cross-border transfer basis and data-subject rights — a superset of
what PIPA and CCPA need. Build the fact schema to GDPR and the other three regimes require **zero new
facts, only new policy packs**. Sequencing by fact-demand rather than by urgency means the fact layer
is authored once; sequencing by urgency would mean re-opening every annotation when GDPR arrives.

## Remaining open question

**One only:** what is the exact port-crossing predicate the coverage gate keys on? "Crosses a storage
port" is the right rule, but the mechanical test — a trait bound, a marker, an attribute on the port
trait — has to be chosen against the real port definitions in-tree. That is a Phase 2 (Plan) question,
not a Phase 1 one.

## Not doing (this slice)

Regime policy packs · projection generation · retention-engine wiring · touching the 87 `dpia.md` ·
non-Rust store classification. **Measurement only** — the gap must be measurable and non-growing
before any compliance output is generated from it.
