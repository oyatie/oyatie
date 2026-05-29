---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-014
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + council-architecture
related_adrs: [ADR-0105, ADR-0135, ADR-0131, ADR-0132, ADR-0133]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-014 — Hyperscaler maturity gate HG-COMMUNITY

## Intent

Wire the HG-COMMUNITY hyperscaler-maturity claim gate. Per `feedback_quality_performance_scalability_bar.md`, every µservice claims parity vs. industry leaders and that claim is CI-enforced.

## Scope

- Parity matrix: see `competitor-parity-matrix.md`.
- Per-claim test: per row in the matrix, a CI test verifies the claim.
- Per-performance claim: synthetic benchmark.
- Per-policy claim: policy-as-code coverage.
- Quarterly review.

## Deliverables

- Parity-matrix CI lane.
- Per-claim test files at `tests/parity/`.
- Synthetic benchmark set at `tests/perf/`.

## Acceptance

- HG-COMMUNITY gate returns GREEN.
- All parity rows have evidence pointers.
- Performance numbers in matrix match measured.

## Owner

axis-community + council-architecture.

## Wave 15 substance conversion

### A. Problem this IP closes

Community's hyperscaler claim is now broader than a forum. It must be honest across Reddit forums, Teamblind anonymous workplace, Handshake recruiting, and the LinkedIn jobs/profile/recruiter subset that replaced the retired `network` service.
The old IP said "parity matrix CI" but did not specify which claims become blocking, how Big-8/P0 elevation works, or which local artifacts prove or disprove the claim.
This IP closes the HG-COMMUNITY maturity gate with service-specific evidence.

### B. Approach

Treat `microservices/community/competitor-parity-matrix.md`, `feature-parity-matrix-2026-05-20.md`, `PRD.md`, `manifest.json`, contracts, SLOs, policies, runbooks, and benchmarks as the evidence corpus.
The gate must distinguish actual implementation evidence from aspirational design and must downgrade or block claims where only docs exist.
ADR-0328 §D-20 and Big-8 P0 rules mean missing canonical OpenTofu contexts, unsupported OS manifest, Rust-strict violations, or counterpart-critical gaps cannot be waved through as tier polish.

### C. Deliverables

- Add HG-COMMUNITY claim rows for Reddit, Teamblind, Handshake, LinkedIn jobs/profile/recruiter, Discourse, GitHub Discussions, Zendesk Help Center, and Notion/Confluence KB.
- Add machine-checkable evidence pointers for contracts, policies, SLOs, dashboards, runbooks, catalog records, and benchmarks.
- Add claim-boundary rules rejecting LinkedIn engagement-feed parity claims.
- Add P0/P1/P2 severity mapping for Big-8 gaps, ADR-0328 substrate gaps, privacy/anonymity gaps, and product-surface gaps.
- Add CI or governance gate output that reports `green`, `held`, or `blocked` with evidence handles.

### D. Implementation steps

1. Parse `manifest.json` top counterparts and product pillars.
2. Parse `competitor-parity-matrix.md` for explicit counterpart features and forbidden feed boundary.
3. Map each claim to at least one local evidence file; mark missing implementation as `design-only`.
4. Elevate anonymity, employment-sensitive, cross-tenant, and moderation gaps to blocker when they affect Teamblind/Handshake/LinkedIn-subset flows.
5. Elevate missing OpenTofu/supported-OS/Rust-strict violations under ADR-0328 D-20 where applicable.
6. Add negative claims: no sponsored posts, no For-You feed, no influencer monetization via followers.
7. Validate that SLO targets for post, feed, vote, search, moderation, KB, and audit have OpenSLO files.
8. Validate runbooks exist for spam, deletion, search rebuild, moderation queue, vote anomaly, KB restore, and deanonymization incidents.
9. Emit an evidence table for every counterpart row.
10. Fail the gate if evidence pointers do not resolve.

### E. Acceptance

- HG-COMMUNITY report separates built evidence from design intent.
- Reddit, Teamblind, and Handshake top-3 anchors have explicit claim rows.
- LinkedIn jobs/profile/recruiter is present only as a secondary subset and engagement feed claims are forbidden.
- Big-8/P0 elevation rules block privacy, employment, cross-tenant, and moderation gaps.
- Every claim row has a real file pointer or is marked as a gap.

### F. Evidence

- `microservices/community/manifest.json` top and secondary counterparts.
- `microservices/community/competitor-parity-matrix.md`.
- `microservices/community/feature-parity-matrix-2026-05-20.md`.
- `microservices/community/coherence-audit-2026-05-20.md`.
- `microservices/community/contracts/`, `policy/`, `slos/`, `dashboards/`, `runbooks/`.
- `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md`.

### G. Counterpart closure

| Counterpart | Claim risk | This IP closure |
|---|---|---|
| Reddit | forum/vote/moderation parity overstated | claim rows tied to contracts/SLOs/runbooks |
| Teamblind | anonymity and employment-sensitive claims are high risk | P0 elevation for anonymity leaks and workplace policy gaps |
| Handshake | recruiting flow parity can drift into ATS claims | claim boundary limits community to posting/application handoff |
| LinkedIn | engagement feed is explicitly out of scope | forbidden-claim rows block feed superiority claims |

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-014-hyperscaler-maturity-gate.md` matched `SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
