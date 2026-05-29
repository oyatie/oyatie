---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-007
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0028, ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-007 — moderation-queue

## Intent

Ship the moderation BC with append-only action log, audit-chain seal per action, and two-eyes enforcement for destructive verbs.

## Scope

- Types: `Flag`, `Action`, `QueueItem`, `ModeratorVerdict`, `FlagReason`, `ModerationVerb`.
- Storage: Postgres with append-only trigger on `moderation_actions`.
- Operations: `raise_flag`, `resolve_flag`, `apply_action`, `list_queue`, `escalate_flag`.
- Audit-chain seal per action (Ed25519 signed by moderator JWT-bound key).
- Two-eyes enforcement: `delete_post > 100/day per mod` requires approver.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + adapter-moderation-bridge + worker + sdk.
- Postgres trigger script preventing UPDATE/DELETE on `moderation_actions`.
- Audit-chain integration hook.

## Acceptance

- Moderation action p99 ≤ 200 ms.
- Append-only invariant verified by attempted UPDATE/DELETE (rejected).
- Two-eyes verified by integration test.
- Audit-chain seal latency p99 ≤ 1 s.

## Owner

axis-community + ops-security.

## Wave 15 substance conversion

### A. Problem this IP closes

Community moderation is not a generic queue. It is the audit-sealed control plane for Reddit-style moderators, Teamblind workplace safety, Handshake employer/candidate abuse reports, public KB abuse, and professional-profile impersonation cases.
The earlier IP named append-only actions and two-eyes enforcement, but it did not bind them to the real policy fragments, AsyncAPI events, runbooks, or competitor gaps.
This IP closes the path from `raiseFlag` and `applyModerationAction` to Cedar, audit-chain, queue dashboards, and incident runbooks.

### B. Approach

Represent flags and moderation actions as append-only records with immutable target, actor, verb, reason, two-eyes approver, policy decision ID, and audit seal reference.
Use the chain-of-responsibility doctrine in `microservices/community/decisions/ADR-COMM-0001-moderation-policy-pipeline-architecture.md` if present, plus Cedar fragments under `policy/`.
Queue state changes must emit `community.flag.raised` and `community.moderation.actioned` and update `dashboards/moderation-queue-depth.json`.
Destructive verbs require two-eyes based on threshold and risk, and anonymous workplace content must remain blinded except through explicit deanonymization incident protocol.

### C. Deliverables

- Add crates `oya-community-moderation-queue-kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-moderation-bridge`, `worker`, and `sdk`.
- Add catalog updates for every moderation queue crate.
- Add append-only schema for `flags`, `moderation_actions`, `moderation_action_seals`, and optional `moderation_queue_assignments`.
- Add Cedar action coverage for `raise_flag`, `list_queue`, `apply_action`, `resolve_flag`, and `escalate_flag`.
- Add tests for two-eyes delete, anonymous deanonymization denial, auditor read scope, CI scope, and cross-tenant flag denial.
- Update runbook links to `moderation-queue-clear.md`, `moderator-decision-appeal-protocol.md`, `coordinated-spam-attack-response.md`, and `verified-anonymous-deanonymization-incident.md`.

### D. Implementation steps

1. Map OpenAPI moderation routes to usecase commands and queue queries.
2. Map proto `ModerationQueueService`, `FlagReason`, and `ModerationVerb` to kernel enums.
3. Define immutable `ModerationAction` with policy decision metadata and audit seal reference.
4. Create append-only Postgres protections that reject UPDATE/DELETE for action rows.
5. Implement Cedar decision capture before usecase mutation and store the policy version/hash.
6. Emit CloudEvents matching `FlagRaised` and `ModerationActioned` from AsyncAPI.
7. Add two-eyes policy for destructive verbs and volume thresholds.
8. Add moderation bridge adapter contract for Foundry classifier verdicts.
9. Add queue-depth and action-latency metrics to the existing dashboard and OpenSLO.
10. Add appeal and incident runbook links to the README or catalog metadata.

### E. Acceptance

- Attempted UPDATE/DELETE on `moderation_actions` fails in integration tests.
- Cross-tenant moderation attempt is denied by Cedar and by storage RLS.
- `applyModerationAction` cannot perform destructive action over threshold without approver.
- `community.moderation.actioned` event contains audit-safe fields and seal reference.
- Runbook evidence exists for queue backlog, spam attack, appeal, and deanonymization incident.

### F. Evidence

- `microservices/community/contracts/openapi/community.yaml` moderation routes.
- `microservices/community/contracts/proto/community.proto` `ModerationQueueService`.
- `microservices/community/contracts/asyncapi/community-events.yaml` `FlagRaised` and `ModerationActioned`.
- `microservices/community/policy/*.cedar`.
- `microservices/community/dashboards/moderation-queue-depth.json`.
- `microservices/community/runbooks/moderation-queue-clear.md`.

### G. Counterpart closure

| Counterpart | Moderation expectation | This IP closure |
|---|---|---|
| Reddit | moderator queue, flags, removals, appeals | append-only actions plus queue dashboard |
| Teamblind | workplace safety with anonymity protection | explicit denial of deanonymization except incident protocol |
| Handshake | employer/candidate abuse and fraud reports | scoped flags and two-eyes destructive actions |
| Salesforce Experience Cloud | enterprise community moderation auditability | Cedar decision ID and audit-chain seal reference |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-007-moderation-queue.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-007-moderation-queue.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
