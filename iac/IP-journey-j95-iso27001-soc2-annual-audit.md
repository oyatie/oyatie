---
doc_class: Implementation-Plan
ip_id: IP-journey-j95-iso27001-soc2-annual-audit
journey_ref: docs/user-journeys/j95-iso-27001-soc-2-annual-audit/
microservice: cloud-iac
status: draft
date: 2026-05-21
---

# IP-journey-j95: ISO 27001 / SOC 2 annual cloud-iac audit packet

## A. Problem
J95 needs auditors to inspect cloud-iac controls without reading every raw plan. The packet must show module provenance, drift coverage, restore drills, registry health, and incident recovery for the annual audit window.

## B. Approach
Assemble a read-only audit packet from the real service artifacts: dashboards under `iac/dashboards/`, SLOs under `iac/slos/`, runbooks under `iac/runbooks/`, and claim boundaries in `iac/competitor-parity-matrix.md`.

## C. Deliverables
- Audit packet examples in `iac/contracts/openapi/cloud-iac.yaml`.
- Evidence-export events in `iac/contracts/asyncapi/cloud-iac-events.yaml`.
- Links to `iac/dashboards/drift-coverage.json` and `iac/runbooks/restore-drill-quarterly.md`.
- Auditor scope reference to `iac/policy/auditor-scope.cedar`.

## D. Implementation
1. Add `audit_window`, `control_family`, and `evidence_bundle_ref` to j95 examples.
2. Include drift, apply success, registry health, and SLSA provenance in the packet.
3. Gate packet reads through auditor-scope Cedar only.
4. Exclude provider secrets and raw OpenBao values from evidence payloads.
5. Rebuild the packet after registry restore to prove projection recovery.
6. Document residual gaps against competitor parity claims.

## E. Acceptance
- Annual packet examples cite dashboards, SLOs, policy, and restore runbook.
- Auditor read is read-only and tenant-scoped.
- The packet can be rebuilt from registry/audit-chain state.
- Claim boundaries do not exceed `iac/competitor-parity-matrix.md`.

## F. Evidence
- Journey: `docs/user-journeys/j95-iso-27001-soc-2-annual-audit/README.md`.
- Dashboard: `iac/dashboards/drift-coverage.json`.
- Policy: `iac/policy/auditor-scope.cedar`.

## G. Counterparts
| Counterpart | Gap closed |
|---|---|
| Terraform Cloud | Adds annual audit packet shape beyond workspace history. |
| Spacelift / Env0 | Matches audit views while adding restore-drill and SLSA proof. |
| ArgoCD / Flux | Adds controls evidence beyond sync and drift status. |

## API Versioning (per ADR-0342)

- Carrier: public contract calls MUST carry `Oyatie-Version: 2026-05-21`, route external HTTP through `/v/2026-05-21/...`, and reserve proto3 field tag `8001` as the `oyatie_version` carrier on public protobuf envelopes.
- Initial declared_version: `iac/manifest.json#api_versioning.declared_version` is absent in this checkout; declared_version is seeded as `2026-05-21`.
- Support window: `N=3` public date versions remain supported for at least `180` days after deprecation notice.
- Internal-mesh exemption: direct internal gRPC over HTTP/3 remains proto3 tag-compatible and is not version-routed at the mesh hop per ADR-0145.
- Surface evidence: `iac/contracts/openapi/cloud-iac.yaml`, `iac/contracts/asyncapi/cloud-iac-events.yaml`, `iac/contracts/proto/cloud-iac.proto`, `iac/IP-journey-j95-iso27001-soc2-annual-audit.md`.

## DR posture (per ADR-0343)

- Target source: `iac/manifest.json#dr` is absent in this checkout; DR numeric targets below use compliance-pack floors only.
- Applicable compliance pack floor: `ISO27001-2022` from `specs/compliance-pack-floors.json` with drill cadence `annual`.
- RTO/RPO target: RTO p99 <= `14400` seconds; RPO p99 <= `3600` seconds.
- Multi-region posture: `active-active` for this HA-critical IP; applicable pack floor `multi_region_required` is `false`, so this declaration is equal to or stronger than the floor.
- backup_substrate: [`object_storage_versioned`, `seaweedfs_replicated`, `postgres_wal_g`].
- Surface evidence: `iac/runbooks/restore-drill-quarterly.md`, `iac/runbooks/seaweedfs-volume-failover.md`, `iac/manifest.json`, `iac/IP-journey-j95-iso27001-soc2-annual-audit.md`.
