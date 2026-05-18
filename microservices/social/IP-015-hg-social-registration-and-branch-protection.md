---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-015-hg-social-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social + ops-governance
acceptance_lanes: [oya-governance-authority-cohesion, oya-governance-hyperscaler-maturity-claims, branch-protection-validate]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-SOCIAL registration + branch protection wiring

## Intent

Register `HG-SOCIAL` in `/specs/hyperscaler-gates.json` and the
authority-cohesion catalog; wire the four social-specific gates as
required status checks on `dev` + `staging` per `.github/branch-protection.yaml`.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | edit — add HG-SOCIAL record + 4 required lanes |
| `registry/authority-cohesion-catalog.yaml` | edit — add social authorities (Twitter/X, Bluesky, Mastodon, Threads, Facebook, Instagram, LinkedIn, TikTok, Pinterest, Reddit, Lemmy) |
| `.github/branch-protection.yaml` | edit — add 4 social required checks |
| `microservices/social/competitor-parity-matrix.md` | (already authored Slice A) |

## Required Status Checks

- `oya-governance-per-microservice-layout` (social scope)
- `oya-governance-dual-context-isolation` (social scope)
- `oya-governance-authority-cohesion` (HG-SOCIAL)
- `oya-governance-hyperscaler-maturity-claims` (HG-SOCIAL)
- `oya-governance-eu-ai-act-conformance` (social scope; new per ADR-SOC-0003)
- `oya-governance-eu-dsa-conformance` (social scope)
- `oya-governance-pack-aware-age-gate` (social scope; new per IP-013)
- `oya-check-federation-personal-tier-refused` (social scope; new per ADR-SOC-0004 + DCI-08)

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice social
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice social
cargo run -p oya-dev-cli -- gate validate eu-ai-act-conformance --microservice social
cargo run -p oya-dev-cli -- gate validate eu-dsa-conformance --microservice social
oya gate validate branch-protection-config
```

## Test Plan

- Synthetic PR mutating social crate path; branch-protection refuses merge without the 8 required checks green.
- Authority-cohesion: open PR with a non-cited social claim ("Twitter-compatible"); HG-SOCIAL lane refuses.
- Hyperscaler-maturity-claims: open PR with a "faster than X" claim; refuses without published benchmark.
- EU AI Act conformance: missing Statement of Reasons in moderation verdict → refuses.
- Federation personal-tier refused: synthetic PR adding Personal-tier federation path → refuses.

## Halt Conditions

- Branch-protection writer-token missing — engage governance.
- HG-SOCIAL catalog entry rejected — fix names per BNF v4.1.

## Next IP

(end of P01 social slice — proceed to P02 = federation minimum-shippable-tier per ADR-SOC-0004
or to P03 = ML-driven ranking model per PRD Open Question 1.)

## References

- ADR-0123; ADR-0132; ADR-0133.
- `/specs/hyperscaler-gates.json`.
- `microservices/social/competitor-parity-matrix.md`.
- ADR-SOC-0003 (EU AI Act bounds) + ADR-SOC-0004 (federation posture).
