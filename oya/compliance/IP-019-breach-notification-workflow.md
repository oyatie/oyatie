---
ip_id: IP-019
microservice: compliance
bounded_context: breach-notification-workflow
layer: usecase
status: planned
related_adrs: [ADR-0251, ADR-0263, ADR-0244]
---

# IP-019 — breach-notification workflow

## A. Problem

Compliance needs a concrete workflow for GDPR Article 33/34, KR-PIPA, HIPAA, and US-state breach notification clocks. The brief shell listed a 72-hour clock but did not tie that clock to local runbooks, SLOs, events, pack rules, or tenant-scope evidence. Without a first-class workflow, breach handling becomes a manual incident note that cannot prove timely notification to auditors.

## B. Approach

Build `oya-compliance-breach-notification-workflow-usecase` as the command layer for breach declaration, risk assessment, authority notification, subject notification, state attorney-general fanout, and closure. The workflow reads applicable packs from IP-017, writes evidence through the compliance evidence substrate, emits ADR-0263 events through IP-025, and drives `slos/breach-notify-authority-72h.openslo.yaml`.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/compliance/catalog/oya-compliance-breach-notification-workflow-usecase.yaml` | usecase catalog row |
| `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md` | operational response when authority clock approaches breach |
| `microservices/compliance/dashboards/breach-notification-sla.json` | clock status, risk class, notified authorities, subject fanout |
| `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml` | SLO for authority notification |
| `microservices/compliance/capabilities/breach-declare.cedar` | declaration authority |

## D. Implementation

1. Add `breach.declare(tenant_id, incident_id, affected_data_classes, suspected_start, discovered_at)` and emit `oya.compliance.breach-declared`.
2. Resolve applicable packs: GDPR, HIPAA, KR-PIPA, FedRAMP, PCI-DSS, and local state-law overlays.
3. Compute notification deadlines per pack; store per-authority and per-subject deadlines separately.
4. Run risk assessment for Article 34/high-risk subject notification and HIPAA unsecured PHI.
5. Produce authority packet references from existing evidence artifacts, not raw payload copies.
6. Call IP-025 AsyncAPI emitter for authority and subject notification events.
7. Close only after required packets, notices, audit seals, and post-incident evidence are present.
8. Add tests for 72h deadline math, high-risk subject path, KR-PIPA clock, missing evidence packet, and tenant isolation.

## E. Acceptance

- Declaring a GDPR breach starts a visible 72-hour authority clock.
- A high-risk breach creates a separate subject notification path.
- Missing evidence packets block closure.
- Dashboard, SLO, and runbook use the same event ids.

## F. Evidence

- `microservices/compliance/slos/breach-notify-authority-72h.openslo.yaml` is the local SLO anchor.
- `microservices/compliance/runbooks/breach-notification-72h-clock-at-risk.md` is the operational anchor.
- `microservices/compliance/competitor-parity-matrix.md` lists OneTrust, AuditBoard, and ServiceNow GRC as breach/GRC counterparts.

## G. Counterparts

| Counterpart | Gap closed |
|---|---|
| OneTrust Breach Response | Adds comparable breach workflow while keeping packets in Oyatie-controlled evidence storage. |
| AuditBoard | Brings audit-ready task closure and evidence linkage into breach operations. |
| ServiceNow GRC | Narrows enterprise workflow parity without adopting ServiceNow as the control plane. |

## H. Non-goals and handoff boundaries

- Do not send notification messages directly from the usecase; IP-025 owns AsyncAPI emission.
- Do not copy raw breach payload into authority packets; use evidence refs and scoped export APIs.
- Do not collapse all jurisdictions into the GDPR 72-hour clock; each active pack contributes deadlines.
- Do not close a breach record while required authority or subject notification packets are missing.
- Do not page only Slack for statutory risk; SLO breach paths use incident/on-call routing.

## I. Fixture set

- `gdpr_article_33_72h.json` proves authority deadline math.
- `gdpr_article_34_high_risk_subjects.json` proves subject notification path.
- `kr_pipa_notification_clock.json` proves pack-specific deadline contribution.
- `missing_evidence_packet_blocks_close.json` proves closure safety.
- `tenant_b_cannot_view_tenant_a_breach.json` proves tenant isolation.

## J. Launch blockers

- Breach closure succeeds while an authority packet is missing.
- Subject notification state is conflated with authority notification state.
- Pack-specific deadlines are not visible in dashboard output.
- Notification evidence contains raw breached payload.
- Clock breach fails to page the statutory-risk escalation path.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/compliance/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), PCI-DSS-L1-v4(rto=86400,rpo=3600,multi_region=false), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/compliance/IP-019-breach-notification-workflow.md` matched `SLO, PHI`; anchors `microservices/compliance/runbooks/phi-access-anomaly.md, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/compliance/IP-019-breach-notification-workflow.md` matched `emission`; anchors `microservices/compliance/manifest.json, crates/oya-shared-compliance-evidence-kernel/src/lib.rs`; type anchor `crates/oya-shared-compliance-evidence-kernel/src/lib.rs::EvidenceArtifact`.
