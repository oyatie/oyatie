---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-foundry-evidence-frontend
impl_plan_id: IP-013-regulator-export-framework-profiles
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: council-privacy + axis-foundry-evidence
acceptance_lanes: [regulator-profile-drill, compliance-claims-citation-check]
---

# IP-013: Regulator-export framework profiles (6 profiles)

## Intent

Author and CI-assert the six regulator-export framework profiles with citation-anchored field selectors. Per `capabilities/regulator-export.yaml` §`framework_profiles`.

## ChangeSet boundary

Pure data + tests; lives in `oya-foundry-evidence-evidence-pack-builder-domain` (per-profile selectors) + `oya-foundry-evidence-regulator-export-usecase` (apply-and-emit).

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/eu-ai-act.yaml` | create | canonical field selector + citation anchors |
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/hipaa.yaml` | create | |
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/gdpr.yaml` | create | |
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/kr-pipa.yaml` | create | |
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/soc2.yaml` | create | |
| `microservices/foundry-evidence/capabilities/regulator-export-profiles/iso-27001.yaml` | create | |
| `crates/oya-foundry-evidence-evidence-pack-builder-domain/src/framework_profiles/*.rs` | edit | wire to YAML profiles via build.rs |
| `crates/oya-foundry-evidence-regulator-export-usecase/tests/profile_field_completeness.rs` | create | golden tests per profile |
| `microservices/foundry-evidence/capabilities/eval/regulator-export-golden.jsonl` | create | golden inputs |

## Acceptance Gates

```bash
cargo nextest run -p oya-foundry-evidence-regulator-export-usecase --test profile_field_completeness
oya gate validate regulator-profile-drill --microservice foundry-evidence
oya gate validate compliance-claims-citation-check --microservice foundry-evidence
```

## Halt Conditions

- Any profile YAML missing citation anchor — block.
- Field selector references a pack-schema field that does not exist — block.
- Golden test fails — block; the framework profile is the regulator-export contract.

## Next IP

[`IP-014-evidence-archive-cascade.md`](IP-014-evidence-archive-cascade.md)

## References

- `compliance.md` (citation cross-walk).
- `capabilities/regulator-export.yaml`.
- ADR-0133.
