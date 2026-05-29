---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-team-channels-dm-threads
impl_plan_id: IP-009-thread-tree-and-mention-router
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-messenger
acceptance_lanes: [cargo-nextest, ontology-integration-smoke]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-009: thread-tree + mention-router BCs

## Intent

Thread reply chains with parent-child traversal + participant tracking;
mention parser + Ontology-backed identity resolution for Person/Team/Channel;
fanout to notification + WebSocket; ingest action-cards from mail µservice.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-messenger-thread-tree-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}/...` | create |
| `src/crates/oya-messenger-mention-router-{kernel,domain,usecase,api,adapter,worker,sdk,app}/...` | create |
| `tests/mention_resolve_e2e.rs` | create |

## Acceptance Gates

```bash
cargo nextest run -p oya-messenger-thread-tree-kernel
cargo nextest run -p oya-messenger-mention-router-kernel
cargo nextest run --test mention_resolve_e2e
```

## Test Plan

- Thread tree: 3-deep nesting; parent-only traversal; participant set correct.
- Mention resolve: `@alice` resolves via Ontology Person lookup; emits MentionEmitted ≤ 250ms p99.
- Mail action-card ingest: action-card → mention-router → post into target channel.

## Next IP

[`IP-010-read-receipt-tracker.md`](IP-010-read-receipt-tracker.md)

## Wave 15 substance conversion — thread tree and mention router

### §A Problem

Messenger parity with Slack, Teams, Discord, and Matrix depends on replies, mentions, and action-card routing being
first-class domain objects rather than UI conventions.
This IP closes the gap between `Channel`, `Message`, Ontology person lookup, and fanout audit evidence.

### §B Approach

Create thread-tree and mention-router bounded contexts that consume message-stream events and emit typed mention
fanout events.
The implementation must preserve dual-context isolation: a personal DM mention cannot leak into a professional
channel, and work mentions must respect `policy/channel-scope.cedar`.

### §C Deliverables

- `src/crates/oya-messenger-thread-tree-{kernel,domain,usecase,adapter-postgres,worker}/...`
- `src/crates/oya-messenger-mention-router-{kernel,domain,usecase,worker}/...`
- `tests/mention_resolve_e2e.rs`
- SLO proof against `slos/mention-fanout.openslo.yaml`

### §D Implementation

1. Model `ThreadNode` with tenant, channel, root message, parent message, depth, and participant set.
2. Reject thread parentage that crosses tenant, channel, or `ContextKind`.
3. Resolve `@user`, `@team`, and `@channel` through Ontology/person and channel membership projections.
4. Evaluate Cedar before emitting notification fanout.
5. Emit `MentionEmitted` and audit-chain evidence with no message body leakage.
6. Backpressure fanout workers so mention storms trigger the runbook, not gateway collapse.

### §E Acceptance

E2E tests must cover three-deep replies, cross-context mention denial, p99 fanout within `mention-fanout` SLO, and
mail action-card ingestion into a target work channel.

### §F Evidence

Local anchors: `PRD.md` threads/mentions matrix, `policy/dual-context-isolation.md`,
`policy/channel-scope.cedar`, `runbooks/mention-storm-throttle.md`.

### §G Counterparts

Slack and Teams anchor enterprise mentions, Discord anchors high-scale channels, and Matrix anchors federated event
shape; oyatie closes parity with Cedar-scoped, dual-context-safe fanout.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/messenger/IP-009-thread-tree-and-mention-router.md` matched `SLO, p99`.
- Numeric target: `rto_p99_seconds=3600`, `rpo_p99_seconds=300` from manifest-declared pack floor via specs/compliance-pack-floors.json.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), KR-CSAP-v3.1(3600s/900s MR) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/messenger/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `object_storage_versioned`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/messenger/slos/attachment-scan-freshness.openslo.yaml`, `microservices/messenger/slos/mention-fanout.openslo.yaml`, `microservices/messenger/slos/message-send-availability.openslo.yaml`, `microservices/messenger/slos/message-send-latency.openslo.yaml`, `microservices/messenger/policy/auditor-scope.cedar`.
