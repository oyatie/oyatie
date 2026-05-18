---
doc_class: FailureModes
title: notes µservice — Failure Modes + Effects Analysis
microservice: notes
status: Accepted
date: 2026-05-17
owner_team: axis-notes + ops-sre-reliability
doc_status: published
---

# Failure Modes + Effects Analysis — notes µservice

## Methodology

FMEA pattern: enumerate fault by BC × layer × cause; describe symptom; mitigation; runbook link; observability anchor. Risk-priority = Likelihood × Impact × Detectability (1-5 each; RPN = product).

## BC-Level Failure Modes

### note-store

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-NS-01 | Postgres primary down | infra | note-open + note-create fail | 5×5×2 = 50 | warm-standby promotion; HPA replicas surface 5xx | runbooks/sync-conflict-resolution.md |
| F-NS-02 | per-tenant Cedar evaluation slow | policy size or evaluator bug | latency p99 spike | 3×3×3 = 27 | Cedar policy size budget + caching | runbooks/sync-conflict-resolution.md |
| F-NS-03 | context_kind invariant violation attempt | bug | DCI lane fail | 1×5×5 = 25 | runtime metric + audit-chain | policy/dual-context-isolation.md |

### tag-graph

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-TG-01 | adjacency table corruption | partial write | tag-search wrong results | 3×4×3 = 36 | rebuild from note_tag truth; consistency check | runbooks/tag-graph-corruption.md |
| F-TG-02 | tag-graph cardinality bomb | runaway script | DB CPU + storage spike | 4×3×4 = 48 | per-tenant rate-limit + cardinality cap | (runbook auto-recovers) |
| F-TG-03 | tag rename race | concurrent rename | dangling tag refs | 2×3×3 = 18 | tx-locked rename with audit-chain | runbooks/tag-graph-corruption.md |

### backlink-graph

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-BL-01 | wikilink parse drift between client + server | divergent parser | broken backlinks | 3×3×3 = 27 | shared `pulldown-cmark` parser version pin | (rebuild via backfill) |
| F-BL-02 | backlink rebuild OOM on Enterprise vault | unbounded chunk | worker crash | 3×3×3 = 27 | streaming chunked rebuild; 10k-batch cap | (backfill) |

### daily-note

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-DN-01 | duplicate daily-note created on first-access | race | duplicate "2026-05-17" notes per user | 3×2×3 = 18 | per-user-per-day UNIQUE constraint | (automatic) |
| F-DN-02 | wrong timezone attribution | server-tz vs user-tz | wrong-day note | 3×2×4 = 24 | user-local-tz from JWT claim; fallback to UTC + correction job | (Open Q #4) |

### web-clipper-bridge

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-WC-01 | installation token leak via DOM | extension bug | replay attacks | 2×4×3 = 24 | rotate tokens; MV3 isolated world | runbooks/web-clipper-degraded.md |
| F-WC-02 | clip capture > 500ms p95 | network or large HTML | UX degradation | 4×2×3 = 24 | client-side trim + retry | runbooks/web-clipper-degraded.md |
| F-WC-03 | extension version drift breaks API | mismatch | upload 422 | 3×2×4 = 24 | extension carries API version header; reject mismatches with upgrade banner | runbooks/web-clipper-degraded.md |

### share-link

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-SL-01 | brute-force share-link enumeration | adversary | invalid-access spike | 2×3×4 = 24 | rate-limit + CAPTCHA + token entropy 128-bit | (incident-response IR-05) |
| F-SL-02 | OG metadata leaks body snippet | misconfig | privacy leak | 1×4×3 = 12 | OG renders title only by default (Open Q #5) | (policy/data-residency.md) |

### embed

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-EM-01 | drive attachment revoked but embed still references | sync gap | broken embed | 3×2×3 = 18 | `DriveAttachmentRevoked` event handler marks embed broken | runbooks/attachment-loss-recovery.md |
| F-EM-02 | drive attachment ACL drift | cross-µservice | non-member sees embed | 1×4×3 = 12 | drive resolves under requester Cedar scope | (drive runbook) |

### checklist

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-CK-01 | `ChecklistItemEmitted` event lost | workflow-engine backpressure | task not created | 3×3×3 = 27 | at-least-once delivery + dedupe by checklist_id | (workflow runbook) |
| F-CK-02 | duplicate task on retry | dedupe fail | duplicate tasks | 2×2×3 = 12 | idempotency-key on emit | (workflow runbook) |

### version-history

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-VH-01 | restore-to-version overwrites concurrent edits | race | data loss | 2×4×3 = 24 | transactional restore + version-pointer-fence | (incident-response IR) |

### search-index

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-SI-01 | Meilisearch shard loss | infra | search degraded | 3×3×2 = 18 | rebuild from Postgres (see backfill-replay.md) | runbooks/sync-conflict-resolution.md |
| F-SI-02 | cross-tenant search bleed | misconfig | privacy leak | 1×5×3 = 15 | per-tenant namespace + weekly audit | (threat-model T-I-09b) |
| F-SI-03 | search slow on large tenant | shard imbalance | latency p99 spike | 3×3×3 = 27 | shard-by-tenant > 1TB | runbooks/sync-conflict-resolution.md |

### graph-view-data

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-GV-01 | graph render > 1s p95 for 5k vault | client perf | slow UX | 3×2×3 = 18 | client-side WebGL force-directed; serverside pre-binning | (client perf) |
| F-GV-02 | adjacency snapshot stale | cache | wrong graph | 3×2×3 = 18 | 1m TTL + invalidation on BacklinkResolved | (auto) |

### collab-edit

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-CE-01 | Loro op-log divergence | broker bug | content drift | 1×4×3 = 12 | Loro 1.x deterministic merge; reference-implementation test | runbooks/sync-conflict-resolution.md |
| F-CE-02 | Personal-tier collab requested | misuse | 403 | 1×2×5 = 10 | E2E refusal Cedar policy; collab is Professional-only | (auto) |

### import-pipeline

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-IP-01 | malicious ENEX with script payload | adversarial | XSS / exfiltration | 2×4×3 = 24 | sandboxed worker + CSP + Markdown sanitiser | runbooks/import-pipeline-failure.md |
| F-IP-02 | import-OOM on large Obsidian vault | infra | failed import | 3×3×3 = 27 | chunked import + restart-from-checkpoint | runbooks/import-pipeline-failure.md |
| F-IP-03 | partial import leaves dangling state | crash mid-import | inconsistent vault | 2×3×3 = 18 | transactional batching + per-batch checkpoint | runbooks/import-pipeline-failure.md |

### export-pipeline

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-EP-01 | export sees ciphertext for Personal-tier | misuse | unusable export | 1×3×5 = 15 | export bound to user's session; Personal-tier requires client SDK decrypt + re-emit | (SDK) |
| F-EP-02 | export job OOM on Enterprise vault | infra | failed export | 3×2×3 = 18 | chunked streaming output + S3 multipart | (auto) |

### ai-assist

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-AI-01 | AI call on E2E content (near-miss) | bug | refused by Cedar | 1×5×5 = 25 | Cedar `e2e-ai-refusal.cedar` + CI lane + runtime metric alarm | runbooks/ai-classifier-rollback-e2e-respect.md |
| F-AI-02 | AI assist provider drift in output | model upgrade | quality regression | 3×2×3 = 18 | golden eval set + canary | runbooks/ai-classifier-rollback-e2e-respect.md |
| F-AI-03 | AI call timeout | foundry-runtime slow | UX delay | 3×2×3 = 18 | 8s timeout + degraded-mode banner | (auto) |

### e2e-key-management

| ID | Failure | Cause | Symptom | RPN | Mitigation | Runbook |
|---|---|---|---|---|---|---|
| F-KM-01 | KeyPackage signing-cert revocation list stale | cache | revoked devices still accepted | 1×4×3 = 12 | 1m TTL on revocation list + force refresh on Sev-1 | runbooks/e2e-key-rotation-and-recovery.md |
| F-KM-02 | epoch advance race split-brain | client coordination | encrypted but partitioned chat-history | 1×4×3 = 12 | RFC 9420 epoch fallback; KeyPackage-bundle dual-encryption for catch-up | runbooks/e2e-key-rotation-and-recovery.md |
| F-KM-03 | user loses all devices + paper seed | user behaviour | data destruction | 3×5×1 = 15 | documented Personal-tier tradeoff; double-confirmation UX | runbooks/e2e-key-rotation-and-recovery.md |

## Cross-BC Failures

| ID | Failure | Cause | Mitigation |
|---|---|---|---|
| F-X-01 | Postgres logical-replication lag > 5 min for AUDIT | infra | sync replica for AUDIT; alarm |
| F-X-02 | OpenBao mount unavailable | infra | retry + fail-closed; Sev-1 |
| F-X-03 | audit-chain seal latency > 1s | infra | retry + queue + Sev-3 |
| F-X-04 | observability emission down | infra | retry + Sev-3 |
| F-X-05 | ontology resolution down | cross-µservice | local cache fallback; degraded UX banner |
| F-X-06 | drive-µservice down | cross-µservice | embed-broken state; user notified |
| F-X-07 | tasks-µservice down | cross-µservice | checklist-emit queue; Sev-3 |

## References

- `runbooks/*`.
- `incident-response.md`.
- `threat-model.md`.
- `policy/dual-context-isolation.md`.
