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

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

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
buck2 build //:quality-lane-registry-authority-check # lane=authority-cohesion --microservice messenger
buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity-claims --microservice messenger
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
landed at minimum-shippable-tier scale; or to Slice B = huddles minimum-shippable-tier. P02 is referenced by
PRD §"Open Questions" 2.)

## References

- ADR-0123; ADR-0132; ADR-0133.
- `/specs/hyperscaler-gates.json`.
- `microservices/messenger/competitor-parity-matrix.md`.

## Wave 15 substance conversion — HG-MESSENGER gate wiring

### §A Problem

Messenger can only make Slack/Teams/Discord/Matrix parity claims if those claims are gated in CI and branch
protection.
This IP closes the governance gap between the parity matrix and merge-time enforcement.

### §B Approach

Register `HG-MESSENGER` in the hyperscaler gate registry, authority-cohesion catalog, and branch protection config.
The required checks make unsupported marketing or implementation claims block before merge.

### §C Deliverables

- `/specs/hyperscaler-gates.json` HG-MESSENGER record
- `registry/authority-cohesion-catalog.yaml` messenger authorities
- `.github/branch-protection.yaml` messenger required checks
- validated reference to `microservices/messenger/competitor-parity-matrix.md`

### §D Implementation

1. Add Slack, Teams, Discord, Matrix, and Mattermost authority rows.
2. Bind required checks to `dev` and `staging`.
3. Include dual-context isolation and per-microservice layout gates.
4. Add a synthetic claim test for "faster than Discord" denial.
5. Add a branch-protection config validation check.
6. Keep forbidden claims synchronized with the parity matrix claim-boundary section.

### §E Acceptance

Governance gates must reject uncited messenger claims and branch protection must require the four named checks for
messenger path mutations.

### §F Evidence

Local anchors: `/specs/hyperscaler-gates.json`, `competitor-parity-matrix.md`, `.github/branch-protection.yaml`,
ADR-0123.

### §G Counterparts

Slack, Microsoft Teams, Discord, Matrix/Element, and Mattermost are the explicit benchmark set; this IP turns those
counterparts into enforced claim boundaries instead of prose.
