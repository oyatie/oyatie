---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-014-recommender-fairness-and-bias-lane
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + axis-foundry-runtime + council-privacy + ops-compliance
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-eu-ai-act-employment-conformance, oya-governance-bias-audit-recency]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014: Recommender fairness + bias audit + EU AI Act employment conformance lane

## Intent

Wire the EU AI Act + EEOC + NYC LL144 + CA AB-331 + CO SB 24-205 conformance pipeline per ADR-NET-0002:

- Per-release 4/5-rule bias audit emitter (`oya.network.recruiter.v1.bias-audit-completed`).
- Continuous post-deployment-monitoring per Art. 72 (drift detector).
- OpenSLO manifest `network-recommender-fairness-correctness` (zero-tolerance).
- Dashboard `recommender-fairness-and-bias.json` wired.
- Auto-rollback on bias-audit failure per `runbooks/recruiter-classifier-rollback.md`.
- LEAN lane `oya-check-eu-ai-act-employment-conformance` validates per-tenant FRIA + LL144 audit + AB-331 impact assessment + CO SB 205 risk-management policy attestation.
- LEAN lane `oya-check-bias-audit-recency` validates bias-audit recorded per release + within rolling 30d window.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-check-eu-ai-act-employment-conformance/` | create LEAN crate |
| `src/crates/oya-check-bias-audit-recency/` | create LEAN crate |
| `crates/oya-dev-cli/src/commands/gate/eu_ai_act_employment.rs` | wire gate command |
| `dashboards/recommender-fairness-and-bias.json` | already created |
| `slos/recommender-fairness-correctness.openslo.yaml` | already created |
| `runbooks/recruiter-classifier-rollback.md` | already created |
| `decisions/ADR-NET-0002-recommender-ai-act-eeoc-bounds.md` | already created |

## Bias-Audit Pipeline

- Per-release pipeline runs golden-set inference over the candidate model version.
- Per protected group (race, gender, age, disability, locale), compute selection rate; compute disparity ratio = minority_rate / majority_rate; threshold 0.8 (4/5-rule).
- Emit `oya.network.recruiter.v1.bias-audit-completed` event with `passes_4_5_rule: bool` + per-group ratio.
- Block model promotion if any group < 0.8.

## Continuous Drift Monitor

- Post-deployment: aggregated decisions emitted; rolling 30d window computes per-group ratio.
- SLO `network-recommender-fairness-correctness` (zero-tolerance) alerts on drop.

## Acceptance Gates

```bash
cargo nextest run -p oya-check-eu-ai-act-employment-conformance
cargo nextest run -p oya-check-bias-audit-recency
cargo run -p oya-dev-cli -- gate validate eu-ai-act-employment-conformance --microservice network
cargo run -p oya-dev-cli -- gate validate bias-audit-recency --microservice network
```

## Test Plan

- Synthetic biased model version: bias-audit lane fails; promotion blocked.
- Synthetic NYC tenant: LL144 audit timestamp check; expired audit blocks tenant-side recruiter-stub activation.
- Synthetic CA tenant: AB-331 impact assessment check.
- Synthetic CO tenant: SB 24-205 risk-management policy check.
- Synthetic EU tenant: FRIA attested check.
- Drill: synthetic 4/5-rule failure injection in shadow model; auto-rollback fires per `runbooks/recruiter-classifier-rollback.md`.

## Halt Conditions

- Bias-audit lane false-positive rate too high (legitimate models blocked) — adjust threshold or sample-size gate; council-privacy approval required.

## Next IP

[`IP-015-hg-network-registration-and-branch-protection.md`](IP-015-hg-network-registration-and-branch-protection.md)

## References

- ADR-NET-0002 (recommender bounds).
- `microservices/network/dashboards/recommender-fairness-and-bias.json`.
- `microservices/network/slos/recommender-fairness-correctness.openslo.yaml`.
- `microservices/network/runbooks/recruiter-classifier-rollback.md`.
- EU AI Act 2024/1689 Annex III §4 + Arts. 9-15, 27, 50, 72, 73.
- EEOC UGESP 29 CFR §1607; NYC LL144; CA AB-331; CO SB 24-205.
