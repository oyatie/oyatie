---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-010-regulator-export-stack
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-foundry-evidence + council-privacy
acceptance_lanes: [cargo-clippy, lean-layer-correctness, regulator-export-2pr-drill, regulator-profile-drill]
---

# IP-010: Regulator-export stack

## Intent

`oya-foundry-evidence-regulator-export-{kernel,domain,usecase,api,adapter,rest,worker}`: framework-filtered, signed evidence-pack bundle assembly. 2-person rule enforced. Per PRD.md FR-06 + ADR-0131.

## ChangeSet boundary

7 Rust crates.

## Concrete File Targets

| Crate | Layer | Purpose |
|---|---|---|
| `oya-foundry-evidence-regulator-export-kernel` | kernel | bundle-assembly port traits |
| `oya-foundry-evidence-regulator-export-domain` | domain | bundle assembly logic; framework-profile field selection; Merkle-bundling logic via substrate SDK types |
| `oya-foundry-evidence-regulator-export-usecase` | usecase | 2-person-rule check (Cedar) + Postgres range scan + framework-profile apply + bundle seal request through audit-chain bridge + S3 export-bucket upload |
| `oya-foundry-evidence-regulator-export-api` | api | re-exports |
| `oya-foundry-evidence-regulator-export-adapter` | adapter | Postgres range-scan reader; S3 export-bucket writer; audit-chain bundle-seal bridge |
| `oya-foundry-evidence-regulator-export-rest` | rest | axum router; reissue endpoint |
| `oya-foundry-evidence-regulator-export-worker` | worker | leader-elected bundle-assembly daemon |

## Acceptance Gates

```bash
cargo check -p oya-foundry-evidence-regulator-export-kernel
cargo check -p oya-foundry-evidence-regulator-export-domain
cargo check -p oya-foundry-evidence-regulator-export-usecase
cargo check -p oya-foundry-evidence-regulator-export-api
cargo check -p oya-foundry-evidence-regulator-export-adapter
cargo check -p oya-foundry-evidence-regulator-export-rest
cargo check -p oya-foundry-evidence-regulator-export-worker
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test eu_ai_act_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test hipaa_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test gdpr_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test kr_pipa_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test soc2_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test iso_27001_profile
cargo nextest run -p oya-foundry-evidence-regulator-export-rest --test two_person_rule
oya gate validate regulator-export-2pr-drill --microservice foundry-evidence
oya gate validate regulator-profile-drill --microservice foundry-evidence
```

## Halt Conditions

- Any framework profile fails field-completeness — block.
- 2-person rule bypass detected (approver == requester) — block.
- Bundle assembly p99 > 30 s per 10k packs — block.
- Pre-signed URL TTL > 5 min — block (Cedar `regulator-export-scope.cedar`).

## Next IP

[`IP-011-audit-chain-bridge.md`](IP-011-audit-chain-bridge.md)

## References

- `policy/regulator-export-scope.cedar`.
- `runbooks/regulator-export-reissue.md`.
- ADR-0133 (honest-claim posture).
