---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-015-hg-network-registration-and-branch-protection
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + council-architecture
acceptance_lanes: [oya-governance-hyperscaler-maturity-claims, oya-governance-authority-cohesion]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015: HG-NETWORK hyperscaler-grade conformance gate registration + branch protection

## Intent

Register the `network` µservice with the HG-NETWORK hyperscaler-grade conformance gate per ADR-0133:

- Add HG-NETWORK entry to `/specs/hyperscaler-gates.json`.
- Add branch-protection pattern `release/network/*` to `.github/branch-protection.yaml`.
- Wire the gate-validator to evaluate all of:
  - `oya-governance-per-microservice-layout --microservice network` green.
  - `oya-governance-professional-context-isolation --microservice network` green.
  - `oya-governance-authority-cohesion --microservice network` green.
  - `oya-governance-shardability --microservice network` green.
  - `oya-governance-statelessness --microservice network` green.
  - `oya-governance-layer-correctness --microservice network` green.
  - `oya-governance-port-location --microservice network` green.
  - `oya-governance-bnf-v4-1 --microservice network` green.
  - `oya-governance-cedar-policy-spec --microservice network` green.
  - `oya-governance-eu-ai-act-employment-conformance --microservice network` green.
  - `oya-governance-endorsement-chain-integrity --microservice network` green.
  - `oya-governance-jobs-handoff-contract --microservice network` green.
  - `oya-governance-inmail-bridge-contract --microservice network` green.
  - `oya-governance-version-pinning-conformance` (LTS pins).
  - `oya-governance-compliance-evidence-recency --microservice network`.

## Concrete File Targets

| Path | Action |
|---|---|
| `/specs/hyperscaler-gates.json` | add HG-NETWORK entry |
| `.github/branch-protection.yaml` | add `release/network/*` pattern |
| `crates/oya-foundry-gate-catalog-domain/src/lib.rs` | register HG-NETWORK gate metadata |
| `.github/workflows/network-gate.yml` | per-µservice CI workflow if not yet auto-generated |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate hyperscaler-maturity-claims --microservice network
cargo run -p oya-dev-cli -- gate validate authority-cohesion --microservice network
```

## Test Plan

- HG-NETWORK gate evaluation against synthetic-tenant deployment: all 15 sub-lanes green.
- Branch-protection `release/network/*` pattern enforced by GitHub Actions.
- gtm-customer-success claims-boundary rules from `competitor-parity-matrix.md` aligned with HG-NETWORK gate.

## Halt Conditions

- Any sub-lane red — fix before merging IP-015.

## Phase Exit Gate

After IP-015 merge, the entire P01 phase exits per `PHASE-01-NETWORK-FOUNDATION.md` Phase Exit Bundle:

1. All 15 IPs merged.
2. All ~165 crates `cargo nextest` green.
3. End-to-end drill in pack-kr cluster: profile-create → connection-request → accept → post → repost → comment → reaction → endorsement → recommendation → InMail-send → jobs-handoff → notification → moderation-verdict → appeal completes within performance envelope.
4. Capacity tier XS deployed: 20 tenants, ~1M Professional MAU, ~500 post/sec sustained, OpenSLO burn-rate green for 7 days.
5. Bias-audit lane green (synthetic golden-set; recruiter-stub OFF in production).
6. Postmortem + sign-off by council-architecture, ops-security, council-privacy, ops-compliance, axis-network lead.

## References

- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0133 (industry best-practice conformance).
- `/specs/hyperscaler-gates.json`.
- `microservices/network/PHASE-01-NETWORK-FOUNDATION.md`.
- `microservices/network/competitor-parity-matrix.md` §"Claim-Boundary Rules".
