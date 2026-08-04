# CELL-001 topology contract snapshot (t_87f14dd2)

Created: 2026-07-01T09:05:19Z

Review/fix: `t_e4f52e4a` rechecked live descendant dependencies at 2026-07-01T09:13:05Z and corrected the table below so `remaining` notes list only currently non-done parent gates.

This is a contract/spec snapshot for downstream spec and fixture work. It does not unblock or replace the blocked runtime CELL-001 implementation card (`t_06f7e854`) and makes no production-readiness, runtime-ready, deployable-cell, hyperscaler-grade, auto-rebalance, or tenant-migration claim.

Machine-readable companion: `cloud/cloud-iac/cell-topology/cell-001-contract-snapshot.json`.

## Scope boundary

Allowed use:
- Descendant spec/fixture cards may cite this snapshot for explicit topology assumptions.
- The snapshot is safe for `specs/`, `docs/`, `evidence/`, and contract-fixture work under `cloud/cloud-iac/cell-topology/`.

Forbidden use:
- Do not treat this as production cluster/cell runtime evidence.
- Do not edit live deployment manifests, `.github/`, release/governance files, or any `*.generated.json` by hand.
- Do not infer Argo CD, Kubernetes, OpenTofu, provider, live mesh, capacity telemetry, audit-chain persistence, SLO, DR, promotion, drain, decommission, autosharding, auto-rebalance, or tenant-migration readiness.
- Do not extend retired local `oya` CLI merge authority; future enforcement belongs in cloud-ci/Rust gate packets.

## Evidence inspected

- `cloud/cloud-iac/cell-topology/foundation.json:1-315` defines the current local foundation: five contexts/cells, one cloud-iac service tenant fixture, all cells deny cross-cell traffic by default, and the file explicitly non-claims runtime/provider/Kubernetes/OpenTofu/Argo/live-mesh integrations.
- `cloud/cell-lifecycle/manifest.json:82-120` records OpenBao as future-reference-only, no raw secret material, no runtime dependency graph, no provisioning, and no runtime mesh claim.
- `cloud/cell-lifecycle/manifest.json:178-187` repeats the no-runtime/non-claim boundary: no live REST/gRPC/SDK/worker/database/broker/scheduler/dashboard runtime, no measured SLO, no DR drill, no provider/OpenTofu/Kubernetes controller operation, no audit-chain persistence, no autoscaling/autosharding/tenant move execution, and no promotion/drain/decommission lifecycle engine.
- `specs/multi-region-disposition-canonical.json:20-111` is the accepted ADR-0158 canonical source for active-active / active-passive / single-region disposition, sovereign pin overlay, and global control-plane routing shape.
- `specs/multi-region-disposition-canonical.json:112-216` records pack defaults and CI-lane intent; cloud-iac appears as active-passive in the accepted canonical spec.
- `specs/csi-storage-class-canonical.json:20-178` is the accepted ADR-0161 canonical source for `oya-*` storage class names, per-pack CSI matrix, topology-aware provisioning, and workload contract.
- `docs/decisions/ADR-0009-cell-architecture-per-tenant-per-region.md:26-90` is planning context for cells as blast-radius primitives, five cell tiers, edge/mesh/store/event routing, per-cell HSM partition, quarterly isolation evidence, and the data-plane/control-plane boundary. It is still Proposed.
- `docs/decisions/ADR-0049-cross-region-replication-and-residency.md:26-151` is planning context for residency classes, strict KR behavior, immutable residency, cross-region transfer governance, and failover semantics. It is still Proposed.
- `docs/decisions/ADR-0381-kaniko-to-buildkit-and-multinode-talos-cell-topology.md:112-171` is planning context for local multi-node Talos cell topology and cell-boundary enforcement; it is not runtime evidence.

## Snapshot assumptions

| Area | Snapshot assumption | Claim ceiling |
| --- | --- | --- |
| Topology inventory | Current local foundation has 5 contexts/cells: `aws-guest/us-east-1`, `colo/colo-nyc-1`, `oci-guest/us-ashburn-1`, `on-prem/onprem-lab-1`, and `oyatie-cloud-provider/kr-seoul-1`. | Inventory/fixture only. |
| Service tenant fixture | `cloud-iac` fixture uses tenant `ten_cloud_iac_oyatie_cloud_provider` in cell `oyatie-cloud-provider-kr-seoul-1-a-001`. | One service/tenant fixture only. |
| Cell tier | Fixture cell tier is `dedicated`; ADR-0009 tier language remains Proposed planning context until runtime CELL-001 reconciles terminology and enforcement. | Planning/contract value only. |
| Residency | Fixture residency class is `strict_kr`; downstream lanes may reason from KR-pinned strict residency. | No live residency enforcement claim. |
| Multi-region disposition | Fixture region disposition is `active_passive`; accepted canonical spec includes cloud-iac as active-passive. | No failover drill, RPO/RTO, or route-enforcement claim. |
| Storage class | Fixture storage class is `oya-pg-hot`; accepted canonical spec defines `oya-pg-hot` and topology-aware provisioning expectations. | No live PV/CSI/StorageClass claim. |
| Isolation evidence | Fixture contains reference shapes for quarterly `network`, `storage`, `crypto`, `compute`, and `audit` evidence. | Reference-shape only; no evidence payload/pass claim. |
| Cross-cell traffic | Foundation cells set `default_cross_cell_traffic_allowed=false`. | Future runtime contracts still required for any allowed cross-cell call. |
| KMS/secrets | Each cell includes `kms` and `secrets-bootstrap` module refs; cell-lifecycle manifest says OpenBao is future-reference-only and raw secret material is false. | Contract/runbook vocabulary only; no secret runtime. |
| Plane separation | Treat data-plane behavior as cell-local; control and analytics/global-control-plane behavior must be explicitly labelled by descendant specs. | No runtime route or mesh enforcement claim. |

## Reconciliation points after runtime CELL-001

1. Reconcile Proposed ADR-0009 / ADR-0049 names with accepted specs before any binding runtime gate: cell tier casing, residency class names, cross-cell contract shape, and evidence categories.
2. Decide whether `cloud/cloud-iac/cell-topology/foundation.json` remains the source fixture or is replaced by a runtime-discovered topology source after CELL-001 lands.
3. Replace `evidence://.../quarterly-isolation/...` reference shapes with actual evidence records or explicit blocked placeholders for network, storage, crypto, compute, and audit.
4. Prove route-table/API-gateway behavior for sovereign pins and 421 redirects before residency/sovereign lanes claim enforcement.
5. Prove KMS/HSM/OpenBao partition binding with synthetic non-prod rotation evidence before SECRETS-001 claims rotation readiness.
6. Prove topology-aware storage provisioning and per-pack CSI binding in a non-prod cluster before storage assumptions become runtime claims.
7. Keep future enforcement in cloud-ci/Rust gate packets and avoid adding new retired local CLI authority.

## Descendant readiness from this snapshot

The snapshot input is safe for all six descendants listed below, but only within spec/fixture scope and subject to each card's remaining parents and ADR clarification comments.

| Descendant | Snapshot-safe input | Ready candidate after this snapshot? | Remaining parent/gate notes seen live |
| --- | --- | --- | --- |
| `t_bc655724` SECRETS-001 | Use `cell_id`, `strict_kr`, `kms`, `secrets-bootstrap`, and per-cell HSM assumptions for a secrets contract/runbook. | Yes. | Non-snapshot parents `t_646248cb` and `t_68a87026` are done; the card is currently running after this snapshot parent cleared. Keep no-runtime/no-secret-material boundary. |
| `t_4ee683ec` PACK-001 | Use `strict_kr`, `active_passive`, and `oya-pg-hot` as a KR pack fixture seed. | No. | Still has `t_2fc04777`; also preserve ADR clarification/non-mutating scope. |
| `t_215c8b00` RESIDENCY-001 | Use `strict_kr`, `active_passive`, and quarterly evidence ref kinds for residency attestation design. | No. | Still has `t_b70d22bd`; no live cross-border enforcement claim. |
| `t_69f99449` PLANE-001 | Use cell-local data-plane and explicit cross-cell-contract assumptions for plane catalog design. | No. | Still gated by `t_c2adf6cd`; API-001 snapshot parent `t_885677fc` is now done. No runtime route/mesh claim. |
| `t_866b22c0` CLOUD-001 | Use the five-context/five-cell local foundation and module refs as inventory seed. | No. | Still gated by `t_38500376` (`OPS-001`, currently running); other non-snapshot parents observed done. No provider/DCIM/PUE/WUE/SLO/runtime claim. |
| `t_f630eada` SOV-001 | Use the strict KR active-passive fixture for sovereign/air-gap manifest seed. | No. | Still gated by PACK-001 / `t_4ee683ec`; `t_7d27ddbb` and `t_e4361b50` are now done. No signed bundle/no-egress/RTO/SLO claim. |

## Verification summary

Verification for this task should validate the new JSON companion, re-validate the source JSON files read above, and run Kanban stats plus dispatcher dry-run. This Markdown report intentionally has no generated JSON edits and no production/deployment changes.
