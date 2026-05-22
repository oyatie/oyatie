# Emergency Remediation Notes - 2026-05-21

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:
- microservices/emergency/ARCHITECTURE.md
- microservices/emergency/implementation-plans/IP-002-tracking-board.md
- microservices/emergency/iac/aws-guest/main.tf
- microservices/emergency/iac/oci-guest/main.tf
- microservices/emergency/iac/on-prem/main.tf

Counterpart-fact preservations:
- None.

Files renamed:
- None.
## Wave 15-doctrine-propagation-IPs (2026-05-21)

- Bucket: D4-BUCKET-5.
- Agent: wave-d-d4-bucket-5-codex.
- Scope: trigger-based doctrine propagation only; unmatched IPs were left unchanged.
- IPs scanned: 10.
- Trigger A matched: 2.
- Trigger B matched: 6.
- Trigger C matched: 0.
- Trigger D matched: 0.
- IPs unmatched: 4.

### IP changes
- `microservices/emergency/implementation-plans/IP-001-triage-engine.md` — added API Versioning, DR posture.
- `microservices/emergency/implementation-plans/IP-002-tracking-board.md` — added DR posture.
- `microservices/emergency/implementation-plans/IP-003-protocol-activation.md` — added API Versioning, DR posture.
- `microservices/emergency/implementation-plans/IP-008-disposition-boarding-lwbs.md` — added DR posture.
- `microservices/emergency/implementation-plans/IP-009-metrics-trauma-registry.md` — added DR posture.
- `microservices/emergency/implementation-plans/IP-010-disaster-response-cell-promotion.md` — added DR posture.

### Unmatched IPs
- `microservices/emergency/implementation-plans/IP-004-mci-mode.md` — no trigger match; no doctrine section added.
- `microservices/emergency/implementation-plans/IP-005-ems-handoff.md` — no trigger match; no doctrine section added.
- `microservices/emergency/implementation-plans/IP-006-registration.md` — no trigger match; no doctrine section added.
- `microservices/emergency/implementation-plans/IP-007-order-entry.md` — no trigger match; no doctrine section added.

### Follow-up
- `microservices/emergency/manifest.json#dr` is absent; DR sections use `specs/compliance-pack-floors.json` floors and must be reconciled when the D-2 manifest DR block lands.

## Wave 15-doctrine-propagation-PRD (2026-05-21)

- Bucket: D3-BUCKET-12.
- Scope: PRD doctrine propagation for `emergency`; PRD values match the present `manifest.json` `dr` and `capacity_model` blocks.

### DR posture
- Values: RTO 1800s, RPO 120s, active-active ED cells, failover_runbook `microservices/emergency/runbooks/emergency-board-failover.md`.
- ADR: ADR-0343; HIPAA and EU-AI high-risk floors are looser than the ED safety target.
- Alternative considered: keep the existing generic "disaster tolerance" prose only; rejected because tracking board and MCI behavior need manifest-backed tenant-visible recovery numbers.
- Cost: raises replication, standby capacity, and drill expectations for ED boards and protocol state.

### Capacity model
- Values: 0.45 vCPU, 1024 MiB RAM, 12 GB storage, 6 Postgres connections, 8 Valkey connections, 10 outbound HTTP connections; `per_request` scaling; Tier-3 placement; 3-30 pods per tenant cell.
- ADR: ADR-0340.
- Alternative considered: use bed count alone as the scaling dimension; rejected because board events, protocol activations, and metrics projections are the actual hot operations.
- Cost: reserves Valkey and request headroom for board updates without promoting ED-IS above the manifest's first-party application tier.

### Sustainability + cost attribution
- Values: audit rows carry `cost_usd_minor_units`, `co2_grams`, and `watt_hours`; carbon routing disabled for triage, board, protocol, MCI, and emergency-mode paths but enabled for retrospective metrics and OLAP projection.
- ADR: ADR-0344; ADR-0337 applies to data-warehouse metric projections.
- Alternative considered: carbon-route all data-warehouse traffic; rejected for live registry and emergency-quality feeds that must remain timely.
- Cost: adds per-board/protocol/registry cost segmentation.

### API versioning
- Values: YYYY-MM-DD carrier triplet, SDK semver, last 3 versions for at least 180 days, tenant pinning for EMS and registry integrations, internal-mesh exemption.
- ADR: ADR-0342.
- Alternative considered: rely on hospital-specific integration versions; rejected because EMS and registry contracts need an Oyatie-wide compatibility window.
- Cost: keeps multiple EMS and registry contract versions live during tenant migrations.

## Wave 15-doctrine-propagation (2026-05-21)

### Block 1: capacity_model
- Values: 0.45 vCPU, 1024 MiB RAM, 12 GB storage, and per_request scaling track ED board refreshes, triage mutations, bed changes, and alert acknowledgements.
- ADR: ADR-0340 capacity envelopes and ADR-0340 D-6 pod-runtime/cell-placement covariance.
- Rejected: Rejected Tier-2 cell placement because the service is a product application, and Tier-3 better fits healthcare-family app cells for pod_runtime_tier=2.
- Cost: Commits Emergency to Valkey projection headroom and standby board capacity for surge events.

### Block 2: dr
- Values: RTO 1800s, RPO 120s, active-active true, backup substrates postgres_wal_g, valkey_cluster, object_storage_versioned, audit_chain_merkle_seal.
- ADR: ADR-0343 recoverability floors, with compliance-pack floors treated as minimums.
- Rejected: Rejected 3600s RTO because it would permit a clinically unsafe ED operations gap during a hospital surge.
- Cost: Commits the service to runbook-backed failover drills and evidence capture at runbooks/emergency-board-failover.md.

### Block 3: pod_runtime_tier
- Values: pod_runtime_tier=2; Emergency is a first-party ED operations application handling PHI, triage, patient-flow, and EMTALA-sensitive audit records. It is clinically urgent, but it neither executes tenant-customer code nor owns a shared substrate control plane, so Tier 2 with Tier-3 cell placement is the valid ADR-0340 pairing.
- ADR: ADR-0338 pod runtime tiering and ADR-0340 D-6 covariance.
- Rejected: Rejected Tier 1 because Emergency does not operate shared secrets, keys, audit substrate, or tenant execution infrastructure.
- Cost: Commits placement and scheduling to the declared runtime isolation class rather than cheapest generic app placement.

### Block 4: tenant_version_pinning
- Values: declared version 2026-05-21, default 2026-05-21, three-version support window, 180 day minimum support, per-tenant pinning enabled.
- ADR: ADR-0342 tenant/API version pinning and manifest schema public_surface_files contract map.
- Rejected: Rejected synthetic historical API dates because only one public contract generation is present.
- Cost: Future contract changes need explicit version calendars and migration documents before tenant sunset.

### Block 5: consumes_upstream_oss
- Values: postgresql, valkey, cedar, openbao, opentofu.
- ADR: ADR-0345 OSS stewardship declarations, using registry dep_name strings from specs/oss-stewardship-registry.json.
- Rejected: Rejected non-registry queue names so CVE ownership can resolve through the OSS stewardship registry.
- Cost: CVE response ownership and upgrade stewardship now attach to the declared upstream substrate set.

### Block 6: iac_module_invocations
- Values: aws-guest/tenant-namespace, aws-guest/postgres-wal-g, aws-guest/valkey-cluster, oci-guest/tenant-namespace, oci-guest/valkey-cluster, oci-guest/always-free/tenant-namespace, on-prem/tenant-namespace, colo/tenant-namespace, oyatie-as-cloud-provider/shard-cell.
- ADR: ADR-0339 shared IaC module invocation doctrine and manifest schema authority.
- Rejected: Rejected oyatie-cloud as a manifest context spelling because schema authority names it oyatie-as-cloud-provider.
- Cost: Provider-specific IaC must remain a thin invocation layer over shared module primitives and version pins.
