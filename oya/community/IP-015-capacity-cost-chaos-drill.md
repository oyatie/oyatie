---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-015
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + ops-sre
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-015 — Capacity + cost + chaos drill

## Intent

Quarterly drills covering capacity verification, cost burn validation, and chaos coverage of `failure-modes.md`.

## Scope

- Capacity: 10× nominal traffic for 30 min; SLOs hold; error budget burn < 10 %.
- Cost: per-tier burn validated against `cost-budget.md` forecast; deviation < 15 %.
- Chaos drill rotation:
  - Q1: FM-01 (search rebuild storm), FM-03 (moderation OOM)
  - Q2: FM-05 (spam flood), FM-07 (mass-delete recovery)
  - Q3: FM-04 (S3 outage), FM-15 (cross-tenant bleed)
  - Q4: full P0 + P1 rotation

## Deliverables

- Load drill scripts at `tests/load/`.
- Chaos drill scenarios at `tests/chaos/`.
- Drill outcome report at `evidence/drills/<quarter>.md`.

## Acceptance

- All drills complete.
- All deviations documented in ADRs.
- All findings tracked to closure.

## Owner

axis-community + ops-sre.

## Wave 15 substance conversion

### A. Problem this IP closes

Community's capacity and chaos posture must reflect actual product risks: spam floods, vote brigades, search rebuild storms, moderation queue backlog, KB attachment restore, anonymous identity protection, and mass deletion recovery.
The old IP listed quarterly drills but did not bind them to service runbooks, SLOs, counterpart performance targets, or cost evidence.
This IP closes the repeatable evidence loop for scale, cost, and failure recovery.

### B. Approach

Run quarterly drills that exercise the real paths in contracts and runbooks, not synthetic "traffic" without product semantics.
Capacity drills use post create, feed render, vote cast, search query, moderation action, KB publish, and audit seal SLOs.
Cost drills compare telemetry, search, Postgres/Citus, object storage, and moderation classifier usage against `cost-budget.md`.
Chaos drills rotate through the named community runbooks and must preserve tenant isolation and anonymity guarantees during failure.

### C. Deliverables

- Add load scenarios for post creation, reply traversal, voting, search, moderation queue, KB publish, and public KB read.
- Add chaos scenarios linked to `runbooks/search-rebuild.md`, `spam-flood-throttle.md`, `coordinated-spam-attack-response.md`, `post-mass-deletion.md`, `vote-anomaly.md`, `kb-attachment-restore.md`, and `verified-anonymous-deanonymization-incident.md`.
- Add cost evidence comparing drill resource use to `microservices/community/cost-budget.md`.
- Add evidence reports under a service-local evidence/drills path or the repo's canonical evidence location.
- Add pass/fail thresholds from OpenSLO files and competitor performance rows.

### D. Implementation steps

1. Define a representative tenant mix: B2C personal subreddit, B2B workplace Teamblind board, Handshake employer space, public KB, and developer forum.
2. Generate traffic for `createPost`, `postReply`, `castVote`, `search`, `applyModerationAction`, and `publishKbArticle`.
3. Run 10x nominal traffic for 30 minutes and capture p99 per SLO.
4. Trigger search rebuild storm and prove per-tenant stagger prevents global impact.
5. Trigger spam flood and prove rate limits plus Foundry fallback preserve queue health.
6. Trigger vote anomaly and prove reconciliation plus alert routing.
7. Trigger KB object restore and prove checksum verification.
8. Trigger mass-delete recovery and prove tombstone/restore audit chain.
9. Trigger deanonymization incident protocol and prove default denial outside approved incident path.
10. Write a drill report with SLO, cost, incident, and follow-up evidence.

### E. Acceptance

- Drill reports include exact SLO pass/fail for post, feed, vote, search, moderation, KB, and audit seal.
- Cost variance is compared to `cost-budget.md` and deviations over threshold create follow-up work.
- No chaos scenario permits cross-tenant reads or anonymous identity leakage.
- Search rebuild and spam flood drills cite their runbooks and complete rollback/recovery.
- Quarterly evidence distinguishes measured results from target-only claims.

### F. Evidence

- `microservices/community/cost-budget.md`.
- `microservices/community/slos/*.openslo.yaml`.
- `microservices/community/runbooks/search-rebuild.md`, `spam-flood-throttle.md`, `coordinated-spam-attack-response.md`, `post-mass-deletion.md`, `vote-anomaly.md`, `kb-attachment-restore.md`, `verified-anonymous-deanonymization-incident.md`.
- `microservices/community/performance-benchmark-numbers-2026-05-20.md`.
- `microservices/community/competitor-parity-matrix.md` performance parity rows.

### G. Counterpart closure

| Counterpart | Resilience expectation | This IP closure |
|---|---|---|
| Reddit | survive spam, vote brigades, and large discussion traffic | spam, vote, post, reply, and search drills |
| Teamblind | preserve anonymity under incident pressure | deanonymization incident denial and audit protocol |
| Handshake | keep recruiting spaces usable during bursts | employer/candidate tenant mix in load drills |
| Grafana/Datadog | measured SLO/cost evidence | OpenSLO plus cost-budget comparison |
| GitHub Discussions | keep developer forums searchable and recoverable | search rebuild and mass-deletion drills cover developer-forum mode |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-015-capacity-cost-chaos-drill.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-015-capacity-cost-chaos-drill.md` matched `cost`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
