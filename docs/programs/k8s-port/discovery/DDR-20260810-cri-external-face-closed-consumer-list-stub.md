---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-20260810-cri-external-face-closed-consumer-list-stub
judgment_class: cri-external-compat-face
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-20260810-cri-external-face-closed-consumer-list-stub

## Baseline version header

| Authority | Version this document was authored against | Status at authoring (2026-08-10) |
|---|---|---|
| Repository baseline | `origin/dev` @ `9a56538c74b1fce4d474869956dd278f7fe1981e` | Discovery proposal lane base. |
| Upstream Kubernetes pin | `v1.36.1`, peeled commit `756939600b9a7180fc2df6550a4585b638875e67` | Pinned program input; discovery only — no pin flip. |
| Engine | `build/port-engine/*`, v0 | Not in force as a producer for this record. |
| Neutral rule pack | `specs/port-rules/**`, v0 | Not in force. |
| Corpus rule policy | `specs/k8s-port/rules/**`, v0 — unauthored | Not in force. |
| Go front end | Bootstrap extractor; strategy ruled | Not in force. |
| Reproducibility tuple / receipt schema | `pin`, `snapshot_digest`, `engine_digest`, `rulepack_digest`, `toolchain_digest`, `formatter_digest` | Six required axes; not in force. This record emits no receipt. |
| Program authority | ADR-0701 live apex + ADR-0715 Proposed (F1(e) preconditions) | Discovery **draft/proposal** only — does not Accept ADR-0715. |
## Record identity

- **Stable ID:** `DDR-20260810-cri-external-face-closed-consumer-list-stub`.
- **Judgment class:** CRI as external compatibility face with closed consumer list.
- **Status:** `drafted` discovery stub — **proposal already in ADR-0712**; this record inventories candidate consumers only.
- **Recorded:** 2026-08-10.

## Authority fence

This record **MUST NOT**:

- edit `specs/k8s-port/scope.json` (OWN token and rows blocked on F1 Accept);
- edit divergence-ledger or capability-registry;
- invent a hand mini-CRI product;
- claim containerd product PORT (OVERRULED Round-2).

## Judgment

### J1 — CRI is external-only

Internal components (ported kubelet path, owned runtime libraries, per-sandbox shims) **MUST NOT** route through CRI. CRI survives solely as an external compatibility face.

### J2 — Closed consumer list (candidate stub)

Unlisted consumers = **REFUSE**. Candidate allowlist for later contract tests (not encoded as scope):

| Consumer | Role | Notes |
|---|---|---|
| `crictl` | Operator/debug CLI | Primary soak consumer |
| `node-problem-detector` | Node signal agent | Common external CRI client |
| (future) named first-party debug agents | Explicit add only | Requires same-wave contract test |

This stub is **illustrative**. Ratification of the closed list is a follow-on after F1(b) Accept + OWN vocabulary.

### J3 — OWN disposition (proposal pointer)

ADR-0712 D-2 proposes an `OWN` disposition token so first-party owned runtime is expressible without colliding `SCP-CONSUME-EXTERNAL-RUNTIMES`. **Not applied here.**

## Round-2 basis

Owned runtime libraries + CRI external face; no manager daemon; Go containerd = dated bootstrap CONSUME only.

## Alternatives

| Alternative | Why rejected |
|---|---|
| Encode consumers into `scope.json` now | Blocked on F1 Accept; E0 forbids scope flips |
| Hand mini-CRI | Dead under Round-2 |
| containerd product PORT | OVERRULED |

## Downstream blockers

1. Founder Accept of ADR-0712.
2. Scope vocabulary `OWN` + bootstrap CONSUME row + ledger intent `DVG-OWNED-NODE-RUNTIME` (dated expiry).
3. Contract tests binding the closed consumer list.

## Naming law

Forever nouns: **owned runtime**, **CRI external face**, **closed consumer list**. Ban `asterkube` / `kuberos` product nouns.
