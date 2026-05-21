---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-006
microservice: community
phase: PHASE-01-community-substrate
status: Accepted
date: 2026-05-17
owner_team: axis-community
related_adrs: [ADR-0105, ADR-0135, ADR-0131]
doc_status: published
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-006 — voting-engine

## Intent

Ship the voting BC with conflict-free counter (Valkey Lua) and idempotent vote cast.

## Scope

- Types: `Vote`, `Tally`, `Acceptance`, `VoteDirection`.
- Storage: Valkey (live counter) + Postgres (source of truth + audit).
- Operations: `cast_vote(post_id, direction, idempotency_key)`, `read_tally(post_id)`, `accept_answer(question_id, reply_id)`.
- Valkey Lua script for atomic SET NX + INCRBY + audit append.
- Async Postgres flush via worker.

## Deliverables

- Crate set: kernel + domain + usecase + api + adapter + adapter-postgres + worker + sdk.
- Valkey Lua script in `src/lua/vote_cast.lua`.
- Reconciliation worker for Valkey vs. Postgres divergence detection.

## Acceptance

- Vote cast p99 ≤ 100 ms.
- Idempotency verified by integration test.
- Divergence detector runs hourly; alerts at > 0.1 %.
- SLO authored.

## Owner

axis-community.

## Wave 15 substance conversion

### A. Problem this IP closes

Voting is a first-class ranking signal for Reddit-like community forums and Q&A, but it must not become a LinkedIn-style engagement-feed amplification system.
The old IP named Valkey and idempotency but did not tie votes to `castVote`, `VoteCast`, tenant isolation, brigade defense, or counterpart parity.
This IP closes the concrete vote-cast, tally-read, answer-acceptance, and reconciliation gap for community posts and replies.

### B. Approach

Use Postgres as the source of truth for one vote per `(tenant_id, post_id, member_id)` plus Valkey as a low-latency live counter cache.
Use idempotency keys on `castVote` so mobile/web retries cannot double-count.
Keep ranking policy deterministic and auditable: Wilson or bounded score + recency + accepted-answer + moderator endorsement, never sponsored reach or follower-boost mechanics.
Emit `community.vote.cast` and `community.answer.accepted` without exposing raw profile or workplace identity.

### C. Deliverables

- Add crates `oya-community-voting-engine-kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `worker`, and `sdk`.
- Add catalog updates under `microservices/community/catalog/oya-community-voting-engine-*.yaml`.
- Add SQL table `votes` and optional Valkey script file in the adapter crate path, not in docs-only form.
- Add reconciliation worker that compares Valkey counters to Postgres tallies.
- Add tests for idempotent upvote, downvote-to-clear, answer acceptance, vote anomaly, and tenant isolation.
- Bind dashboards `microservices/community/dashboards/vote-rate.json` and SLO `slos/vote-cast-latency.openslo.yaml`.

### D. Implementation steps

1. Map OpenAPI `POST /posts/{post_id}/vote` and `POST /posts/{post_id}/accept-answer` to usecase commands.
2. Map proto `VotingEngineService` request/response types to API crate structs.
3. Define `VoteDirection` exactly from proto: up, down, clear, plus unspecified rejection.
4. Store source-of-truth rows in Postgres with tenant, member, post, direction, idempotency key, and audit timestamp.
5. Update Valkey counters atomically after Postgres commit or via outbox to avoid split-brain.
6. Emit `VoteCast` with `idempotency_key` and low-cardinality direction.
7. Implement hourly reconciliation and alert if divergence exceeds 0.1 percent.
8. Add abuse checks for vote burst, same-workplace brigading, and cross-space coordinated voting.
9. Wire `runbooks/vote-anomaly.md` to the divergence alert and suspicious-burst alert.
10. Keep ranking outputs bounded and explainable for audit review.

### E. Acceptance

- Vote cast p99 target is measured against `microservices/community/slos/vote-cast-latency.openslo.yaml`.
- Idempotency test proves duplicate client retries do not alter the tally.
- Reconciliation test detects a synthetic Valkey/Postgres mismatch.
- Ranking tests prove no sponsored, follower, or engagement-campaign signal can enter the score.
- `VoteCast` events match AsyncAPI fields and include tenant scope.

### F. Evidence

- `microservices/community/contracts/openapi/community.yaml` `castVote` and `acceptAnswer`.
- `microservices/community/contracts/proto/community.proto` `VotingEngineService`, `VoteDirection`, and `VoteTally`.
- `microservices/community/contracts/asyncapi/community-events.yaml` `VoteCast` and `AnswerAccepted`.
- `microservices/community/dashboards/vote-rate.json`.
- `microservices/community/runbooks/vote-anomaly.md`.
- `microservices/community/competitor-parity-matrix.md` voting/ranking parity and forbidden engagement-feed boundary.

### G. Counterpart closure

| Counterpart | Voting expectation | This IP closure |
|---|---|---|
| Reddit | vote-driven ranking and comments | idempotent vote cast and bounded ranking |
| Stack Overflow | accepted answer and score semantics | `acceptAnswer` and answer-specific validation |
| Teamblind | workplace posts with brigading controls | same-workplace burst detection and anonymity-preserving tally |
| LinkedIn | reactions are not the target | explicit rejection of engagement-feed amplification |
| GitHub Discussions | reactions and accepted answers in developer discussions | vote and accepted-answer APIs cover the discussion subset |

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/community/IP-006-voting-engine.md` matched `openapi, asyncapi, .proto`; contract files `microservices/community/contracts/openapi/community.yaml, microservices/community/contracts/asyncapi/community-events.yaml, microservices/community/contracts/proto/community.proto`; type anchor `microservices/community/manifest.json`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/community/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/community/IP-006-voting-engine.md` matched `p99, SLO`; anchors `microservices/community/manifest.json`; type anchor `microservices/community/manifest.json`.
