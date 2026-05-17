---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-014-observability-slo-branch-protection-hg-cloud-secrets
status: pending
owner: axis-cloud-secrets + axis-observability + axis-governance
acceptance_lanes: [promotion-readiness, authority-cohesion]
---

# IP-014: observability SLO + branch-protection + HG-CLOUD-SECRETS register

## Intent

Author OpenSLO manifests for cloud-secrets hot-path; register HG-CLOUD-SECRETS in authority-cohesion ledger per ADR-0123; wire branch-protection for `microservices/cloud-secrets/**` paths.

## ChangeSet boundary

Pure governance + observability wiring; no Rust code.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/cloud-secrets/slos/secret-resolution.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/rotation-completeness.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/audit-emission-completeness.openslo.yaml` | create |
| `microservices/cloud-secrets/slos/hsm-attestation.openslo.yaml` | create |
| `registry/authority-cohesion/HG-CLOUD-SECRETS.yaml` | create |
| `.github/branch-protection.yaml` | update — require cloud-secrets gates on PRs touching the µservice |

## Acceptance Gates

```bash
cargo run -p oya-dev-cli -- gate validate openslo-manifest --microservice cloud-secrets
cargo run -p oya-dev-cli -- gate validate authority-cohesion
cargo run -p oya-dev-cli -- gate validate promotion-readiness --microservice cloud-secrets
```

## Test Plan

- OpenSLO manifests validate against v1.0 schema.
- HG-CLOUD-SECRETS registered + linked to PRD + threat-model + compliance.
- branch-protection refuses PR if cloud-secrets gates not green.

## Halt Conditions

- HG registration missing post-merge — fail authority-cohesion lane.

## Next IP

`IP-015-lean-a11-raw-secret-emission-lane-wiring.md`
