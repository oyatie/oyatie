---
doc_class: Program-Discovery-Decision-Record
doc_status: discovery
discovery_id: DDR-E0-20260810-divergence-adapter-law
judgment_class: divergence-adapter-law
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-E0-20260810-divergence-adapter-law

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | E0 discovery encode lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; not consumed here. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for this record. |
| Neutral rule pack | `specs/port-rules/**`, v0 | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This record emits no receipt. |
| Program authority | ADR-0704 (live apex); ADR-0638 doctrine-first divergence provenance | Discovery record only. |

## Record identity

- **Stable ID:** `DDR-E0-20260810-divergence-adapter-law`.
- **Judgment class:** divergence-as-adapter law (fail-closed adapters behind upstream seams).
- **Status:** `discovery` — record, **not** doctrine, **not** Accepted apex.
- **Recorded:** 2026-08-10.
- **Owner role:** `council-architecture`.

## Authority fence

This record does **not** add, edit, or expire any row in `specs/k8s-port/divergence-ledger.json`.
It does not ratify empty `test_ids` arrays. It does not amend ADR-0701 / ADR-0704 Accepted text.
Ledger growth remains capped at two new rows per wave **after** the five baseline rows carry
ratified test identifiers (W0 ledger work).

## Judgment

### Law — divergences are separate adapters behind stable upstream seams

Doctrine-first divergences (Cedar authorization seam, owned audit/observability emission,
attestation-context injection, owned OCI executor, runtime-tier class remap, and any future
named divergence) **MUST** be implemented as **separate adapters** composed at the emitted
apiserver / node supervisor composition root — **not** forked into the port-engine projection
corpus.

Rationale: pin-bumps of the mechanical PORT must keep working. Embedding divergence into the
projected tree creates dual-truth forks that silently break on upstream bumps.

### Fail-closed adapter matrix

Adapters inherit the same fail-closed matrix as the seams they attach to:

- An adapter that **silently no-ops** on an unimplemented seam is **dual-truth** and is RED.
- An adapter that cannot decide MUST refuse (deny / UNKNOWN per seam contract), never invent
  success.
- Detached adapter identities spend the W0-E measured detached-control ceiling (ADR-0638 D4);
  ceiling ratification is W0-E work, not claimed complete by this record.

### Ledger coupling (normative intent; no rows authored here)

Each live divergence MUST eventually have a ledger row with rationale, ADR citation, owner,
expiry, review cadence, conformance impact, and **enumerated expected-red test identifiers**.
An unenumerated red is a defect. **E0 does not author those rows.**

Expected adapter attachment points (composition root, not projection internals):

| Divergence class | Adapter posture (intent) |
|---|---|
| Cedar authorization | Owned PEP behind authorizer seam; AdditiveAllow chain first |
| Attestation context | Inject verified result keys into authz context only |
| Owned OCI executor | Trait behind shim; oracles are test artifacts, not product divergences unless escape hatch taken |
| Audit / observability emission | Owned emitters; not silent drop |
| Bootstrap Go containerd CONSUME | Time-boxed CONSUME with dated expiry (future ledger row) |

## Round-2 basis

Round-2 consensus: “divergence-as-adapter” is load-bearing for the greenfield derivation;
adapters get the fail-closed matrix (silent no-op = dual-truth). Oracle adapters for
youki/runc/crun are **test artifacts**, not divergence rows, unless the ledgered escape hatch
is taken.

## Alternatives

| Alternative | Why rejected |
|---|---|
| Fork divergences into port-engine projection | Pin-bump liability; dual-truth |
| Soft / best-effort adapters that pass-through on error | Dual-truth; fails fail-closed bar |
| Bit-compatible abandon of Cedar/audit/observability doctrine | Rejected by ADR-0638 doctrine-first posture |
| Unbounded ledger growth to “encode everything now” | Cap 2/wave; baseline `test_ids` empty — growth banned until ratified |

## Downstream blockers

- Enumerate `test_ids` for the five existing baseline ledger rows (W0).
- W0-E detached-ceiling ratification before adapter count grows.
- Cedar PEP activation card after expected-red IDs exist.
- Any new ledger row (owned-runtime, attestation injection, tier remap, oracle escape) waits for
  F1/W0 sequencing — **not E0**.

## Naming law

Uses Round-2 neutral nouns only. Does not adopt `asterkube` or `kuberos` as product/public names.
