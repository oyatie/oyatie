---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-workspace-preview
phase: P01-slides-foundation
impl_plan_id: IP-015-hg-slides-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-workspace + council-architecture + ops-sre-reliability
acceptance_lanes: [authority-cohesion, hg-slides-green]
depends_on: [IP-014]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-SLIDES registration + branch protection + competitor-parity-matrix evidence pinning

## Intent

Final IP: register HG-SLIDES hyperscaler gate per ADR-0123; add slides-specific CI lanes to `.github/branch-protection.yaml` required_status_checks on `dev` + `staging`; create `release/slides/{staging,production}` pattern protection; pin competitor-parity-matrix evidence per ADR-0133; end-to-end Studio launch verification.

## ChangeSet boundary

Repo-wide files plus slides-side wrap-up artifacts.

## Concrete File Targets

| Path | Action |
|---|---|
| `.github/branch-protection.yaml` | edit |
| `/specs/hyperscaler-gates.json` | edit (add HG-SLIDES) |
| `microservices/slides/evidence/competitor-parity-pinning.json` | create |
| `microservices/slides/evidence/launch-verification.json` | create |

## Code Shape

`.github/branch-protection.yaml` (additions to required_status_checks for `dev` and `staging`):

```yaml
- oya-governance-slides-pptx-roundtrip-subset
- oya-governance-ai-act-risk-class-stamp
- oya-governance-reduced-motion-fallback-mandatory
- oya-governance-flashing-policy
- oya-governance-cedar-preview-required
- oya-governance-per-slide-acl-no-deck-bypass
- oya-governance-named-block-no-bypass
- oya-governance-chart-revocation-cascade-bounded
- oya-governance-broadcast-livekit-types-not-leaked
- oya-governance-broadcast-speaker-notes-isolation
- oya-governance-mp4-determinism
- oya-governance-pdf-a-conformance
- oya-governance-wasm-bundle-sri
- oya-governance-present-mode-frame-budget
- oya-governance-ai-provenance-watermark-preserved
```

`/specs/hyperscaler-gates.json` HG-SLIDES entry:

```json
{
  "gate_id": "HG-SLIDES",
  "microservice": "slides",
  "authority_holder": "axis-workspace",
  "claim_path": "microservices/slides/PRD.md",
  "required_lanes": [
    "oya-governance-slides-pptx-roundtrip-subset",
    "oya-governance-ai-act-risk-class-stamp",
    "oya-governance-reduced-motion-fallback-mandatory",
    "oya-governance-cedar-preview-required",
    "oya-governance-wasm-bundle-sri",
    "oya-governance-present-mode-frame-budget"
  ],
  "claim_summary": "Slides — Google-Slides/PowerPoint-Web/Keynote/Pitch/Beautiful.ai-class collaborative presentation product; Loro CRDT; Leptos WASM; LiveKit broadcast reuse; 11 packs; EU AI Act risk-class enforced.",
  "evidence": "microservices/slides/evidence/"
}
```

## Acceptance Gates

```bash
oya gate validate authority-cohesion
oya gate validate hg-slides-green
```

## Test Plan

| Verification | Method |
|---|---|
| Branch protection has all slides lanes | review `.github/branch-protection.yaml` |
| HG-SLIDES registered in /specs/hyperscaler-gates.json | review |
| Competitor-parity evidence pinned | review evidence file |
| End-to-end launch drill (10 tenants in dev) | tests/e2e/launch-drill.sh |
| All AC-01 through AC-19 green | review evidence/launch-verification.json |

## Halt Conditions

- HG-SLIDES gate fails registration — STOP.
- Any of AC-01..AC-19 lane red — STOP.

## Next

Phase exit_gate declared.
