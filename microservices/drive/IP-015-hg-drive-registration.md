---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-drive-foundation
impl_plan_id: IP-015-hg-drive-registration
status: pending
execution_unit: ChangeSet
owner: axis-drive + council-architecture
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims, oya-governance-authority-cohesion, oya-governance-per-microservice-layout, oya-governance-aggregation-index-generation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-DRIVE registration + per-pack canary cohort + branch-protection

## Intent

Register HG-DRIVE hyperscaler-maturity claim per ADR-0123 + ADR-0133. Wire SLO eligibility (per ADR-0130) + per-pack canary cohort + branch-protection rule.

## Concrete File Targets

| Path | Action |
|---|---|
| `registry/hyperscaler-maturity-claims.json` | append HG-DRIVE entry with axis-1..axis-9 evidence pointers |
| `.github/branch-protection.yaml` | append `oya-governance-hyperscaler-maturity-drive` blocking lane |
| `microservices/drive/specs/canary-cohort.json` | created — per-pack canary weight (10 → 50 → 100% over 6w) |
| `microservices/drive/specs/release-pointer.json` | created — `release/drive/{dev,staging,production}` pointers |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice drive
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice drive
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice drive
cargo run -p oya-dev-cli -- gate validate aggregation-index-generation
```

## Phase exit

This IP completes Phase 1. Phase 2 (adapter soak per ADR-0134) begins after HG-DRIVE passes at p99 SLOs sustained 7d in dev cluster.

## References

- ADR-0123 (hyperscaler maturity claim gate).
- ADR-0130 (SLO-gated promotion).
- ADR-0131 (per-microservice flat layout).
- ADR-0133 (industry conformance).
- ADR-0134 (Strangler migration phase ordering).
