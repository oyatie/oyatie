---
doc_class: Program-Discovery-Index
doc_status: published
authority_tier: 3
---
# Kubernetes Port Discovery Records
## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | Current baseline for the E0 discovery encode lane. |
| Upstream Kubernetes pin | `v1.36.1` tag object `5b824a493a7ca248b726b6ea09d53842b9b992c2`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; not consumed by discovery records. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for these records. |
| Neutral rule pack | `specs/port-rules/**`, v0 — unauthored or incomplete | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. Discovery records emit no receipt. |
| Program authority | ADR-0704 (live apex). ADR-0637 / ADR-0638 are archived provenance | Discovery lane only; does not amend Accepted apex ADRs. |

## Purpose and authority boundary

Discovery records capture **encodable judgments that are not yet doctrine and are not Accepted apex**.
They preserve Round-2 forever-shape decisions so later founder ADRs (F1) and W0 artifacts can cite a
stable record without pretending those ADRs have already landed.

| This lane is | This lane is not |
|---|---|
| A judgment / decision **record** | Doctrine (see [`../doctrine/`](../doctrine/INDEX.md)) |
| Provisional local memory for the k8s-port program | An Accepted apex ADR |
| Allowed to state proposed direction and hard bans for later encode | A license to edit `scope.json`, the divergence ledger, or the capability registry |
| Allowed to cite Proposed ADRs as direction | A claim that Proposed ADRs are live law |

When a discovery judgment becomes binding outside this program lane, it MUST graduate to doctrine and,
if cross-lane, to an ADR — not by silently promoting this file's authority.

## Required entry schema

| Field | Required content |
|---|---|
| Record identity | Stable discovery ID, title, recording date, owner role, and `status: discovery`. |
| Authority fence | Explicit non-claims: not doctrine, not Accepted apex; zero apex/scope/ledger edits authorized. |
| Judgment | Normative judgment language, intended scope, and explicit non-scope. |
| Round-2 basis | Which Round-2 consensus items the record encodes; banned product nouns must not appear as adopted names. |
| Alternatives | Material alternatives considered and why rejected (comparative prior art may be named only as **not adopted**). |
| Downstream blockers | What F1 / W0 / ledger work is required before encode as doctrine, scope, or ledger. |
| Naming law | Forever nouns used; product-name bans observed. |

Document shape for each entry:

```text
# <stable discovery ID>

## Baseline version header
## Record identity
## Authority fence
## Judgment
## Round-2 basis
## Alternatives
## Downstream blockers
## Naming law
```

## Index (lane-first)

| Discovery ID | Judgment class | Status |
|---|---|---|
| [`DDR-E0-20260810-node-stack-forever-shape`](DDR-E0-20260810-node-stack-forever-shape.md) | Node supervisor / owned runtime / tiers / attestation / surface ratchets | discovery |
| [`DDR-E0-20260810-divergence-adapter-law`](DDR-E0-20260810-divergence-adapter-law.md) | Divergence-as-adapter + fail-closed adapters | discovery |
| [`DDR-E0-20260810-ecosystem-coexistence-contract`](DDR-E0-20260810-ecosystem-coexistence-contract.md) | Ecosystem wire/API coexistence | discovery |
| [`DDR-E0-20260810-vap-cedar-plane-split-rationale`](DDR-E0-20260810-vap-cedar-plane-split-rationale.md) | VAP/CEL admission vs Cedar authz plane-split rationale | discovery |
