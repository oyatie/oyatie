---
doc_class: Program-Discovery-Decision-Record
doc_status: drafted
discovery_id: DDR-20260810-sonobuoy-licensing-artifact-class-proposal
judgment_class: conformance-licensing-precondition
recorded_at: 2026-08-10
owner: council-architecture
authority_tier: 3
---
# DDR-20260810-sonobuoy-licensing-artifact-class-proposal

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

- **Stable ID:** `DDR-20260810-sonobuoy-licensing-artifact-class-proposal`.
- **Judgment class:** conformance licensing precondition (named `artifact_class` for Sonobuoy).
- **Status:** `drafted` discovery — **not** a live licensing flip; **not** artifact admission.
- **Recorded:** 2026-08-10.
- **Owner role:** `council-architecture`.
- **Honest-ladder phase:** K2 precondition drafting (blocked on F1 Accept + W0 readiness for live encode).

## Authority fence

This record **MUST NOT** be read as:

- an edit to `specs/k8s-port/licensing.json` `artifact_classes` (no live flip);
- admission of any Sonobuoy binary, digest, SBOM, or signature;
- authority to place full CNCF/Sonobuoy suites inside `presubmit`;
- acceptance of any F1 founder ADR.

E0/F1 authorize **proposal text only**. Live encode of a named `artifact_class` requires an explicit follow-on PR after founder Accept and with complete admission fields.

## Judgment

### J1 — Sonobuoy needs a named artifact_class (proposal)

Today `licensing.json` enumerates:

`bootstrap_extractor`, `kubernetes_source`, `ginkgo`, `conformance_test_artifact`, `rule_seed`, `other_external_input`.

Promotion/release gate law (Round-2) requires **full CNCF + Sonobuoy on the promoted commit**, outside `presubmit`. Without a named class + owner, Sonobuoy falls to `other_external_input` (ownerless) and cannot be admitted fail-closed.

**Proposal (not applied):** add named class `sonobuoy` (or `sonobuoy_conformance_suite`) with required admission fields identical to other external artifacts (`source`, `version`, `digest`, `license`, `SBOM`, `signature`, `provenance_verification`, `sandbox_policy`, `owner`).

### J2 — Two gate classes remain binding

| Gate class | Suite | Home |
|---|---|---|
| PR (`presubmit`) | Pinned hermetic conformance **smoke subset** only | Must not absorb multi-hour Sonobuoy |
| Promotion / release | Full CNCF + Sonobuoy on **promoted commit** | After named class admission |

### J3 — Companion licensing notes (proposal only)

Also flagged for later ruled admission (not flipped here):

- **MPL-2.0** (Asterinas) — currently unlisted in forbidden/allowed product-code lists; black-box soak already uses release ISO metadata.
- **LGPL** (crun as differential **oracle**, never shipped product) — oracle artifact rows only after W0 ledger budget allows.

## Round-2 basis

Encodes Round-2 / Round-1 amendment: Sonobuoy/CNCF as promotion gate; named `artifact_class` precondition; PR gate = smoke subset.

## Alternatives

| Alternative | Why rejected here |
|---|---|
| Live-flip `licensing.json` now | Requires founder/W0 path; E0 forbids tip encode of policy flips without Accept |
| Keep Sonobuoy under `other_external_input` | Ownerless; fails closed admission forever |
| Put full Sonobuoy in `presubmit` | Multi-hour suite must not sit in the protected PR context |

## Downstream blockers

1. Founder Accept (or Reject) of F1 set — especially F1(b)/(e) shaping node forever path.
2. W0 `w0_ready()==true` and ledger test-id manifest before conformance expected-red IDs.
3. Follow-on PR that adds the named class **and** admits a pinned artifact with all required fields + owner.

## Naming law

Forever nouns only: promotion gate, smoke subset, named artifact class. **Ban** adopting `asterkube` / `kuberos` as product nouns.
