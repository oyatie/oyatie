---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-meet-foundation
impl_plan_id: IP-015-hg-meet-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-meet + ops-governance
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, branch-protection-validate]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-MEET registration + branch protection wiring

## Intent

Register `HG-MEET` in `/specs/hyperscaler-gates.json` and the authority-cohesion catalog; wire the four meet-specific gates as required status checks on `dev` + `staging` per `.github/branch-protection.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | edit — add HG-MEET record + 4 required lanes |
| `registry/authority-cohesion-catalog.yaml` | edit — add meet authorities (Google Meet, Zoom, Microsoft Teams, Webex, Jitsi, Daily.co, LiveKit) |
| `.github/branch-protection.yaml` | edit — add 4 meet required checks |
| `microservices/meet/competitor-parity-matrix.md` | (already authored Slice B) |

## Required Status Checks

- `oya-governance-per-microservice-layout` (meet scope)
- `oya-governance-recording-consent-coverage` (meet scope; ADR-MEET-0006 + KR PIPA Art. 15)
- `oya-governance-authority-cohesion` (HG-MEET)
- `oya-governance-hyperscaler-maturity-claims` (HG-MEET)

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice meet
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice meet
oya gate validate branch-protection-config
```

## Test Plan

- Synthetic PR mutating meet crate path; branch-protection refuses merge without the 4 required checks green.
- Authority-cohesion: open PR with a non-cited meet claim ("Zoom-compatible API"); HG-MEET lane refuses.
- Hyperscaler-maturity-claims: open PR with a "faster than Google Meet" claim; refuses without published benchmark.

## Halt Conditions

- Branch-protection writer-token missing — engage governance.
- HG-MEET catalog entry rejected — fix names per BNF v4.1.

## Next IP

(end of P01 meet slice — proceed to P02 = mobile SDK polish + PSTN dial-in research per Open Question 1; or to Slice B = whiteboard own-BC vs slides-µservice question per Open Question 5.)

## References

- ADR-0123; ADR-0132; ADR-0133.
- `/specs/hyperscaler-gates.json`.
- `microservices/meet/competitor-parity-matrix.md`.
