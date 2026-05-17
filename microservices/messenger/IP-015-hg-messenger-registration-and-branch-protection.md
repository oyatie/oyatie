---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-015-hg-messenger-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger + ops-governance
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, branch-protection-validate]
---

# IP-015: HG-MESSENGER registration + branch protection wiring

## Intent

Register `HG-MESSENGER` in `/specs/hyperscaler-gates.json` and the
authority-cohesion catalog; wire the four messenger-specific gates as
required status checks on `dev` + `staging` per `.github/branch-protection.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | edit — add HG-MESSENGER record + 4 required lanes |
| `registry/authority-cohesion-catalog.yaml` | edit — add messenger authorities (Slack, Teams, Discord, Matrix, Mattermost) |
| `.github/branch-protection.yaml` | edit — add 4 messenger required checks |
| `microservices/messenger/competitor-parity-matrix.md` | (already authored Slice B) |

## Required Status Checks

- `oya-governance-per-microservice-layout` (messenger scope)
- `oya-governance-dual-context-isolation` (messenger scope)
- `oya-governance-authority-cohesion` (HG-MESSENGER)
- `oya-governance-hyperscaler-maturity-claims` (HG-MESSENGER)

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice messenger
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice messenger
oya gate validate branch-protection-config
```

## Test Plan

- Synthetic PR mutating messenger crate path; branch-protection refuses
  merge without the 4 required checks green.
- Authority-cohesion: open PR with a non-cited messenger claim ("Slack-compatible");
  HG-MESSENGER lane refuses.
- Hyperscaler-maturity-claims: open PR with a "faster than Discord" claim;
  refuses without published benchmark.

## Halt Conditions

- Branch-protection writer-token missing — engage governance.
- HG-MESSENGER catalog entry rejected — fix names per BNF v4.1.

## Next IP

(end of P01 messenger slice — proceed to P02 = thread-tree + read-receipt
landed at MVP scale; or to Slice B = huddles MVP. P02 is referenced by
PRD §"Open Questions" 2.)

## References

- ADR-0123; ADR-0132; ADR-0133.
- `/specs/hyperscaler-gates.json`.
- `microservices/messenger/competitor-parity-matrix.md`.
