---
doc_class: Implementation-Plan
ip_id: IP-journey-j83-cell-infra-declarative
journey_ref: docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j83: CN PIPL localization and CAC assessment infrastructure

## A. Problem
J83 requires cloud-iac to prevent an infrastructure plan from crossing a China localization boundary before CAC assessment evidence exists. The prior row flood could not tell an implementer where localization checks belong.

## B. Approach
Put the localization gate in the validator bounded context, not in the applier. `iac/policy/data-residency.md` supplies the residency rule, `iac/capabilities/iac-render.yaml` and `iac/capabilities/iac-apply.yaml` distinguish plan creation from mutation, and `iac/runbooks/state-lock-break.md` defines the locked-state recovery path.

## C. Deliverables
- A j83 validation example in `iac/contracts/openapi/cloud-iac.yaml`.
- A denied-plan event in `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Catalog binding for `iac/catalog/oya-cloud-iac-iac-validator-usecase.yaml`.
- Runbook references to `iac/runbooks/state-lock-break.md` and `iac/runbooks/drift-remediation.md`.

## D. Implementation
1. Add a `residency_assessment_ref` field to j83 examples so plan validation can fail before apply.
2. Treat missing CAC assessment as a validator denial, not an applier runtime exception.
3. Emit a denied validation event with tenant, pack, region, and assessment reference hash.
4. Keep rendered artifacts inspectable but non-applicable until the residency gate passes.
5. Add drift-remediation instructions for plans created before a later pack version update.
6. Validate state-lock recovery without bypassing residency checks.

## E. Acceptance
- A j83 plan without `residency_assessment_ref` is documented as fail-closed.
- The iac-applier acceptance text says it never overrides the validator denial.
- `iac/slos/iac-validator-availability.openslo.yaml` is included as the readiness SLO.
- State-lock recovery does not delete evidence.

## F. Evidence
- Journey: `docs/user-journeys/j83-cn-pipl-data-localization-and-cac-assessment/README.md`.
- Policy: `iac/policy/data-residency.md`.
- Benchmark: `iac/benchmarks/cloud-iac-vs-terraform-cloud-vs-pulumi-cloud-vs-spacelift-vs-env0.md`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds pre-apply localization denial instead of relying on workspace policy alone. |
| Spacelift | Matches policy-gated runs but binds the decision to Cedar and audit-chain evidence. |
| OpenTofu | Keeps OSS plan/apply semantics while adding service-level residency assessment. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j83-cell-infra-declarative.md`.

## DR posture (per ADR-0343)

- Target source: `iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `SOC2-T2` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `900` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/seaweedfs-volume-failover.md`, `iac/manifest.json`, `iac/IP-journey-j83-cell-infra-declarative.md`.
