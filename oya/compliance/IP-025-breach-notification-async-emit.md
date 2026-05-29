---
ip_id: IP-025
microservice: compliance
bounded_context: breach-notification-workflow
layer: adapter
status: planned
related_adrs: [ADR-0263, ADR-0253, ADR-0244]
---

# IP-025 — breach-notification AsyncAPI emitter

## A. Problem

Breach notification cannot live only inside the compliance usecase. Notifications must fan out to mail/comms, tenant ops-dashboard, incident-management, and regulator engagement while preserving replay protection and audit seals. The existing shell listed channels but did not bind them to `contracts/asyncapi.yaml`, tenant scope, or the breach SLO.

## B. Approach

Extend `contracts/asyncapi.yaml` with breach workflow channels emitted by an adapter behind IP-019. Events use signed envelopes, tenant ids, pack ids, notification deadlines, evidence packet refs, and audit seal refs. Consumers receive enough metadata to route actions without receiving raw breached data.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/contracts/asyncapi.yaml` | add breach declared/notify authority/notify subjects/closed channels |
| `microservices/compliance/dashboards/breach-notification-sla.json` | consume event lifecycle |
| `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md` | event-driven escalation |
| `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml` | channel freshness and deadline evidence |

## D. Implementation

1. Define envelope fields: `event_id`, `tenant_id`, `incident_id`, `pack_ids`, `deadline_unix_ms`, `evidence_packet_ref`, `audit_chain_seal_hex`, `issued_at`.
2. Add channels `oya.compliance.breach-declared.v1`, `breach-notify-authority.v1`, `breach-notify-subjects.v1`, and `breach-closed.v1`.
3. Sign each event with HMAC or audit-chain envelope key and enforce replay window <=5 minutes at consumers.
4. Keep payload free of raw PHI/PII; consumers fetch scoped packet refs through IP-020 when authorized.
5. Add idempotency key based on tenant, incident, channel, and packet version.
6. Add tests for schema validation, replay rejection, missing seal rejection, duplicate delivery, and no raw PII field names.
7. Wire failure paths to `runbooks/breach-notification-72h-clock-at-risk.md`.

## E. Acceptance

- AsyncAPI contract validates with all breach channels present.
- Events include deadline and evidence packet refs but no raw breached payload.
- Duplicate event delivery is idempotent.
- Replay outside the <=5-minute window is rejected.

## F. Evidence

- `microservices/compliance/contracts/asyncapi.yaml` is the event contract authority.
- `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml` is the deadline proof.
- `microservices/compliance/competitor-parity-matrix.md` anchors OneTrust, AuditBoard, and ServiceNow GRC breach workflow pressure.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| OneTrust | Provides breach workflow fanout without copying sensitive payloads into a SaaS workflow. |
| ServiceNow GRC | Narrows event-driven incident workflow parity with signed AsyncAPI contracts. |
| AuditBoard | Adds audit-ready notification evidence tied to the same breach lifecycle. |

## H. Non-goals and handoff boundaries

- Do not put notification business rules in the AsyncAPI adapter; IP-019 computes deadlines and recipients.
- Do not include raw breached records in events.
- Do not emit unsigned or replayable notification events.
- Do not treat subject and authority notifications as the same channel.
- Do not let downstream mail/comms failures erase the original breach lifecycle event.

## I. Fixture set

- `breach_declared_valid_envelope.json` proves schema and signature fields.
- `breach_notify_authority_deadline.json` proves deadline metadata.
- `breach_notify_subjects_no_raw_pii.json` proves payload minimization.
- `replay_window_expired_rejected.json` proves replay protection.
- `duplicate_delivery_idempotent.json` proves retry behavior.

## J. Launch blockers

- AsyncAPI channels omit tenant id, incident id, or deadline metadata.
- Events include raw PHI/PII payloads.
- Signatures or replay-window checks are optional.
- Subject and authority notifications share one ambiguous channel.
- Downstream delivery failure loses the original breach event.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/compliance/IP-025-breach-notification-async-emit.md` matched `asyncapi`; contract files `microservices/compliance/contracts/openapi.yaml, microservices/compliance/contracts/asyncapi.yaml, microservices/compliance/contracts/compliance.proto`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-025-breach-notification-async-emit.md` matched `SLO, PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
