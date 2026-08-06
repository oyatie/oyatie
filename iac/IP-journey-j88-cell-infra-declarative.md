---
doc_class: Implementation-Plan
ip_id: IP-journey-j88-cell-infra-declarative
journey_ref: docs/user-journeys/j88-au-irap-protected-tenant/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j88: AU IRAP protected tenant infrastructure

## A. Problem
J88 needs cloud-iac to prove protected-tenant isolation for Australian IRAP posture: scoped namespace, network policy, state backend, restore drill, and drift evidence. The stamped IP had repeated row labels without naming the local proof artifacts.

## B. Approach
Treat protected tenant provisioning as a registry-backed apply with explicit rollback. The IP binds `iac/policy/iac-isolation.md`, `iac/dashboards/apply-success-rate.json`, `iac/dashboards/drift-coverage.json`, and `iac/runbooks/restore-drill-quarterly.md`.

## C. Deliverables
- Protected tenant fields in `iac/contracts/openapi/cloud-iac.yaml`.
- Apply and drift events in `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Isolation policy references from `iac/policy/iac-isolation.md`.
- Restore evidence from `iac/runbooks/restore-drill-quarterly.md`.

## D. Implementation
1. Add `protected_tenant=true`, `irap_profile`, and `restore_drill_ref` to j88 examples.
2. Validate the namespace and network-policy plan before apply.
3. Require drift coverage to be healthy before marking the tenant infrastructure ready.
4. Record registry state with tenant and cell identifiers only; do not expose raw provider account details.
5. Exercise restore-drill evidence after initial apply.
6. Fail closed if isolation policy or audit emission is absent.

## E. Acceptance
- Isolation policy and drift dashboard are both cited in the plan.
- Protected tenant readiness requires apply success, drift coverage, and restore evidence.
- Rollback text references `iac/runbooks/rollback-orchestration.md`.
- No IRAP claim is made without audit-chain evidence.

## F. Evidence
- Journey: `docs/user-journeys/j88-au-irap-protected-tenant/README.md`.
- Policy: `iac/policy/iac-isolation.md`.
- Dashboard: `iac/dashboards/drift-coverage.json`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Spacelift | Matches multi-tenant IaC orchestration while adding protected-tenant restore proof. |
| Terraform Cloud | Adds IRAP-specific isolation readiness instead of generic workspace separation. |
| ArgoCD / Flux | Adds infrastructure evidence beyond sync health. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j88-cell-infra-declarative.md`.

## Sustainability emission (per ADR-0344)

- Per-call audit row emission MUST include `cost_usd_minor_units`, `co2_grams`, and `watt_hours` on the same metering/audit event.
- Carbon-aware scheduling eligibility: eligible only when the workload is not Tier 0/Tier 1 and not one of `eu-ai-act-annex-iii`, `hipaa-em-incident-response`, or `pci-dss-realtime-fraud-detection`; excluded calls emit `defer_rejected`.
- finops-portal rollup axes affected: `tenant`, `product`, `capability`, `provider`, `cell`.
- Cost source: `iac/manifest.json#paid_billing_components_emitted` declares `["per_usage"]`.
- Surface evidence: `iac/manifest.json`, `iac/IP-journey-j88-cell-infra-declarative.md`.
