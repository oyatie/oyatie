---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-010
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community + foundry-guardrails
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010 — foundry-guardrails moderation bridge adapter

## Intent

Ship the bridge adapter that consumes `PostCreated` + `PostEdited` + `VoteCast` events from community and forwards them to foundry-guardrails for spam / abuse / impersonation classification.

## Scope

- Adapter: `oya-community-moderation-queue-adapter-moderation-bridge`.
- Source: NATS JetStream subject `community.<tenant_id>.post.*`.
- Target: foundry-guardrails classifier API.
- Backpressure: dead-letter queue; fallback to rate-limit-only mode on classifier outage.

## Deliverables

- Bridge crate.
- NATS subscription config.
- Fallback policy.
- Per-tenant tunable threshold configuration.

## Acceptance

- Bridge lag p99 ≤ 30 s.
- Dead-letter queue depth alert at > 10 k.
- Fallback mode triggers within 60 s of classifier outage.
- Classifier verdict emits `PostShouldHide` consumed by moderation-queue.

## Owner

axis-community + foundry-guardrails.

## Wave 15 substance conversion

### A. Problem this IP closes

Community moderation needs classifier assistance for spam, abuse, impersonation, child-safety reports, responsible disclosure intake, and workplace-harassment reports, but Foundry cannot become an unreviewed auto-moderator.
The old IP named a bridge but did not define event payload boundaries, fallback behavior, audit evidence, or which community modes need classifier help.
This IP closes the bridge between `community.post.*` events and Foundry guardrails while preserving Cedar and human moderation authority.

### B. Approach

Implement `oya-community-moderation-queue-adapter-moderation-bridge` as an outbound adapter that subscribes to post/reply/edit/vote signals, submits redacted content features to Foundry guardrails, and turns classifier verdicts into moderation queue flags or hide recommendations.
Classifier output is advisory unless Cedar policy and moderation usecase accept the action.
Fallback mode must degrade to rate-limit-only and queue-based manual review when Foundry is unavailable.
All bridge decisions must preserve tenant, space, content hash, model/version, classifier categories, confidence bucket, and audit event reference.

### C. Deliverables

- Implement or specify `oya-community-moderation-queue-adapter-moderation-bridge`.
- Add NATS JetStream subscriptions for `community.post.created`, `community.post.edited`, `community.reply.posted`, and `community.vote.cast`.
- Add Foundry request/response DTOs with redaction of raw PII and anonymous author identity.
- Add fallback policy for classifier outage, backlog, timeout, and model version mismatch.
- Add threshold config per tenant/community mode, including stricter Teamblind workplace and child-safety modes.
- Update `microservices/community/runbooks/coordinated-spam-attack-response.md` and `spam-flood-throttle.md`.

### D. Implementation steps

1. Read AsyncAPI message fields and decide which fields may be sent to Foundry.
2. Hash or redact body content according to data-class policy before adapter submission.
3. Include tenant, space, post kind, tags, language, link count, author reputation bucket, and prior flag count as features.
4. Submit classifier request through a Foundry port, not a hard-coded HTTP client inside usecase code.
5. Map classifier categories to `FlagReason` where possible and to explicit advisory reasons where not possible.
6. Write classifier verdicts to moderation queue as recommendations with model version and confidence bucket.
7. Trigger rate-limit-only fallback within 60 seconds of Foundry outage.
8. Add DLQ depth alert and runbook link for backlog over 10,000 events.
9. Add tests for spam, impersonation, classifier timeout, false-positive appeal, and raw-author leakage.
10. Add audit emission for every hide recommendation and fallback transition.

### E. Acceptance

- Foundry classifier outage does not block post creation; it shifts moderation to rate-limit/manual-review mode.
- Bridge never sends Teamblind deanonymizing identity fields to Foundry.
- Classifier output cannot directly delete content without moderation usecase approval.
- DLQ/backlog alert points to the spam and moderation runbooks.
- Tests cover `PostCreated`, `PostEdited`, `ReplyPosted`, and `VoteCast` input events.

### F. Evidence

- `microservices/community/contracts/asyncapi/community-events.yaml` post, reply, vote, flag, moderation channels.
- `microservices/community/contracts/proto/community.proto` `FlagReason` and `ModerationVerb`.
- `microservices/community/runbooks/coordinated-spam-attack-response.md`.
- `microservices/community/runbooks/spam-flood-throttle.md`.
- `microservices/community/policy/anonymity-mode-*.cedar`.
- `microservices/community/PRD.md` moderation and Teamblind mode purpose.

### G. Counterpart closure

| Counterpart | Guardrail expectation | This IP closure |
|---|---|---|
| Reddit | anti-spam and abuse detection feeding mod queue | advisory classifier flags plus queue actions |
| Teamblind | workplace-sensitive abuse detection without deanonymization | redacted features and stricter thresholds |
| GitHub Discussions | report/abuse processing for developer communities | event-backed classifier to moderation queue |
| ServiceNow | enterprise workflow handoff for safety incidents | audit-backed recommendations and fallback state |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md` matched `asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md` matched `p99`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.

## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/community/IP-010-foundry-guardrails-moderation-bridge.md` matched `emission`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
