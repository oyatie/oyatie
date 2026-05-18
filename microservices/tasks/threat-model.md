---
doc_class: ThreatModel
template_id: TPL-THREAT-MODEL
microservice: tasks
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-tasks + ops-security
deciders: council-architecture, ops-security, axis-tasks, council-privacy
methodology: STRIDE + LINDDUN + OWASP Top 10 (2021) + OWASP API Top 10 (2023) + NIST SP 800-154
related_adrs: [ADR-0028, ADR-0056, ADR-0105, ADR-0117, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0140, ADR-TASKS-0001, ADR-TASKS-0002, ADR-TASKS-0005, ADR-TASKS-0006]
review_cadence: quarterly + on every BC architectural change + on every AI capability promotion (T0→T1→T2)
enforced_frameworks:
  - "SOC 2 Type 2: CC6.1-CC6.8, CC7.1-CC7.5, CC8.1"
  - "ISO 27001:2022: A.5.7-A.5.34, A.8.2-A.8.34"
  - "GDPR Arts. 5, 6, 9, 13, 14, 17, 22, 25, 28, 30, 32, 33, 35, 44"
  - "EU AI Act (EU) 2024/1689 Annex III §4 + Art. 14 + Art. 50"
  - "OWASP ASVS v4.0.3"
  - "SLSA L3 + NIST SSDF SP 800-218"
suggested_frameworks_by_pack:
  pack-kr: ["KR PIPA Arts. 15/17/18/22-2/23/24/25/28/29/29-2", "근로기준법 Arts. 23/41 (employment-record retention)", "직장 갑질 protections (harassment-context auto-assign refusal)", "KR-ISMS-P §2.1-2.12"]
  pack-us-healthcare: ["HIPAA 45 CFR §164.308/§164.310/§164.312/§164.314/§164.316/§164.502 (clinical-task assignment)", "EEOC UGESP 1978 + Title VII (auto-assign bias)", "ADA (assignment of accommodation tasks)"]
  pack-eu: ["GDPR Art. 22 (automated decisions) + Art. 25", "EU AI Act Annex III §4 (employment-context)", "eIDAS 910/2014", "NIS2 2022/2555"]
  pack-jp: ["APPI Arts. 17/18/20/21/23/24/26-2/27", "Japanese Labour Standards Act for assignment-context"]
doc_status: published
---

# Threat Model: tasks µservice

## Purpose

Identify, classify, and mitigate threats to the tasks µservice's
confidentiality, integrity, availability, and privacy posture. Tasks
carries dual-context PII (personal + professional task content +
assignee identities + organisational task graphs + custom field values
that may contain sensitive PII), and emits operational decisions about
work assignment that — in employment contexts — fall under EU AI Act
Annex III §4. A compromise here cascades into operational productivity
exposure + potential employment-decision surveillance.

## Scope

### In-scope

All components introduced for the tasks µservice across the seven
bounded contexts (task-store, project-list, view-engine, dependency-
graph, recurrence, search-index, importers), deployed in the tenant
workload cluster:

| Layer-A (adopted OSS) | Layer-B (oyatie-owned) |
|---|---|
| Postgres 16 LTS (task + project + dependency-edge store) | `oya-tasks-task-store-*` (10 crates) |
| Redis 7.2 LTS (view-cache + presence) | `oya-tasks-project-list-*` (8 crates) |
| Meilisearch 0.10.0 LTS (cross-project search) | `oya-tasks-view-engine-*` (8 crates) |
| Loro 1.x CRDT (collab description editing only) | `oya-tasks-dependency-graph-*` (7 crates) |
| `rrule-rs` 0.13.x (recurrence; shared pin with calendar) | `oya-tasks-recurrence-*` (7 crates) |
| Cedar v4.2 (policy) | `oya-tasks-search-index-*` (8 crates) |
| ClamAV (attachment scan; delegated to drive) | `oya-tasks-importers-*` (14 crates) |

### Out-of-scope

- Underlying Kubernetes / IaaS layer (owned by `cloud-k8s`).
- Mail delivery (owned by `mail` µservice).
- Messenger notification (owned by `messenger` µservice).
- Drive attachment storage (owned by `drive` µservice).
- Calendar event bridge (owned by `calendar` µservice).
- Tenancy / identity (owned by `tenancy` µservice).
- Audit-chain seal infrastructure (owned by `audit-chain` µservice).
- Workflow-engine durable-execution (owned by `workflow-engine`).
- Foundry-runtime LLM inference (owned by `foundry-runtime`).
- Observability collectors (owned by `observability` µservice).

## Trust Boundaries

```text
┌─ Internet ─────────────────────────────────────────────────────────────────┐
│                                                                            │
│   Tenant operators        Customer apps        Inbound mail / messenger    │
│         │                       │                       │                  │
│         │ (HTTPS+OIDC+MFA)      │ (per-tenant API key)  │ (Workflow event) │
│         ▼                       ▼                       ▼                  │
│  ┌─ Public ingress (Envoy + WAF + DDoS) ──────────────────────────────┐    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                              │                                             │
└──────────────────────────────│─────────────────────────────────────────────┘
                               ▼
┌─ Tenant workload cluster ──────────────────────────────────────────────────┐
│                                                                            │
│  Trust boundary 1: External → REST ingress (event-store / view / search)   │
│                                                                            │
│  Trust boundary 2: REST → Postgres (per-tenant RLS + tenant-DEK)           │
│                                                                            │
│  Trust boundary 3: REST → Redis (view-cache, per-tenant key prefix)        │
│                                                                            │
│  Trust boundary 4: REST → Meilisearch (per-tenant index + key)             │
│                                                                            │
│  Trust boundary 5: importers REST → subprocess sandbox (cgroup + 5min)     │
│       (Jira/Asana/Trello/Linear/Todoist/CSV parse in isolation)            │
│                                                                            │
│  Trust boundary 6: tasks → workflow-engine (bidirectional Workflow event)  │
│       (per ADR-TASKS-0005; cycle detection at workflow-engine boundary)    │
│                                                                            │
│  Trust boundary 7: tasks → foundry-runtime (T0/T1/T2 capabilities)         │
│       (per-tenant-DEK-wrapped prompt; Cedar-gated EU AI Act §4 refusal)    │
│                                                                            │
│  Trust boundary 8: Workers (retention sweep, recurrence expand, search-    │
│       rebuild, webhook fanout, importer-runner) → DB / Meilisearch         │
│       (SPIFFE-identity bound; not user-callable)                           │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

Eight trust boundaries.

## Assets & Data Classification

Per Bominal ADR-0028 + `oya-check-data-class` LEAN lane.

| Asset | Class | Sensitivity | Retention | Authoritative store |
|---|---|---|---|---|
| Task title + description + custom-field values (Professional context) | `PROFESSIONAL_TASK_CONTENT` (tenant-DEK encrypted) | Critical | per jurisdiction + legal hold | Postgres |
| Task title + description + custom-field values (Personal context) | `PERSONAL_TASK_CONTENT` (E2E where tenant declares) | Critical | per jurisdiction + legal hold | Postgres |
| Assignee + watcher identities | `PII_IDENTIFYING` | High | per task retention | Postgres + audit-chain |
| Task comment threads | `PROFESSIONAL_TASK_CONTENT` / `PERSONAL_TASK_CONTENT` per context | Critical | per task retention | Postgres |
| Dependency-graph edges | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per project retention | Postgres |
| Project + sprint + milestone metadata | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per project retention | Postgres |
| Custom-field schemas | `BEHAVIORAL_TENANT_PRODUCT` | Medium | per project retention | Postgres |
| Cross-project search index | `PROFESSIONAL_TASK_CONTENT` (encrypted at rest) | High | rebuildable | Meilisearch |
| Time-tracking ticks (M02-onward) | `PII_BEHAVIORAL` | High (work-time pattern) | per jurisdiction + employment-record retention | Postgres |
| Webhook payloads in flight | `PROFESSIONAL_TASK_CONTENT` (transient) | High | transient | memory |
| Importer source payloads (Jira/Asana/Trello/Linear/Todoist) | inferred from source | Critical (sandbox) | transient (parsed + dropped) | sandbox tmpfs |
| Legal-hold records | `AUDIT` | Critical | append-only; preserved past retention | Postgres + audit-chain |
| Tenant-DEK | `SECRET` | Critical | OpenBao 90d rotation | OpenBao |
| Meilisearch master key | `SECRET` | Critical | OpenBao 30d rotation | OpenBao |
| Audit-chain seal records | `AUDIT` | High | append-only | audit-chain µservice |
| Foundry-runtime classification telemetry (T1/T2 inputs/outputs) | `AUDIT` + `BEHAVIORAL_TENANT_PRODUCT` | High | per ADR-TASKS-0006 retention | audit-chain + observability |

## Actors

| Actor | Trust | Auth | Capability |
|---|---|---|---|
| Tenant operator (human) | Untrusted external | OIDC + MFA | RW own tenant's tasks / projects |
| Customer app (machine) | Untrusted external | per-tenant API key (30d rotation) | RW own tenant's tasks via SDK / REST |
| Workflow µservice (calling in) | Trusted internal | mTLS + SPIFFE | task-store usecase via WorkflowTrigger event |
| Mail µservice (calling in) | Trusted internal | mTLS + SPIFFE | create-task-from-email via MailReceived event |
| Messenger µservice (calling in) | Trusted internal | mTLS + SPIFFE | create-task-from-message via MessengerMessagePinned event |
| Calendar µservice (calling in/out) | Trusted internal | mTLS + SPIFFE | due-date ↔ event bridge |
| Foundry-runtime (T1/T2 classifier) | Trusted internal | mTLS + SPIFFE + Cedar-gated | classification output materialised into task fields |
| Tenancy µservice | Trusted internal | mTLS + SPIFFE | identity resolution |
| Audit-chain µservice | Trusted internal | mTLS + SPIFFE | seal emission |
| Worker (retention sweep / recurrence expand / search-rebuild / webhook fanout / importer-runner / timer persister) | Trusted internal | SPIFFE + OpenBao SA token | RW on task-store + Meilisearch + outbox |
| Council-architecture / ops-security | Trusted internal | OIDC + MFA + JIT | admin-level access |
| External auditor (SOC 2 / ISO 27001 / EU AI Act conformity) | Read-only time-boxed | OIDC + MFA + JIT ≤ 4h | read-only |
| Attacker (opportunistic / targeted) | Untrusted | none | — |
| Insider (accidental / malicious) | Trusted internal | OIDC + MFA | mitigated via PR review + LEAN gates + audit-chain |

## STRIDE Threat Catalog

Each threat: ID; asset; description; likelihood (L/M/H); impact (L/M/H); risk; mitigations; owner; residual; framework controls.

### Spoofing

**T-S-01 — Importer source forges assignee identity to assign tasks to victim**
- Asset: importer → task-store
- L M / I H / Risk H
- Mitigations:
  - Importer assignee resolution is STRICT (per ADR-TASKS-0001); ambiguous matches refuse with `ImportAssigneeAmbiguous::Refused` (Hyrum #7).
  - Pre-import validator surfaces every assignee mapping for tenant-operator review.
  - LEAN check `oya-check-importer-assignee-strict` ensures strict-only resolver wired.
- Owner: axis-tasks + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1, CC6.2, CC7.1; ISO 27001 A.5.15, A.8.5, A.8.7; GDPR Art. 32(1)(b)

**T-S-02 — Workflow-engine forges task-creation on behalf of victim user**
- Asset: workflow-engine → task-store bridge
- L M / I H / Risk H
- Mitigations:
  - Workflow-creates-task records the workflow run ID + triggering principal in `created_by_workflow_run_id` + `created_by_principal` fields; never anonymous.
  - Cedar policy `workflow-bridge.cedar` refuses cross-tenant workflow-creates-task.
  - LEAN check `oya-check-workflow-bridge-attribution` verifies attribution chain at PR time.
- Owner: axis-tasks
- Residual: L
- Frameworks: SOC 2 CC6.6, CC7.1; ISO 27001 A.5.15, A.8.7

**T-S-03 — Per-tenant API key stolen and used to assign tasks to wrong user**
- Asset: REST API key
- L M / I H / Risk H
- Mitigations:
  - Per-tenant API key bound to `(tenant_id, device_id, scope)`; rotation 30d; revocation on suspicion.
  - Rate-limit + anomaly detection on per-key write patterns; suspicious patterns trigger forced re-auth.
- Owner: ops-security
- Residual: L

### Tampering

**T-T-01 — Custom-field value injection (SQL/JSON via crafted custom-field-typed payload)**
- Asset: custom-field write path
- L H / I H / Risk H
- Mitigations:
  - Custom-field values type-coerced strictly per ADR-TASKS-0001; refusal of mismatched types prevents silent injection.
  - Postgres parameterised statements; jsonb constraints at column level.
  - LEAN check `oya-check-custom-field-type-strict` verifies adapter coerces strictly.
  - Fuzzing: `cargo fuzz` corpus + OWASP injection corpus per custom-field type.
- Owner: axis-tasks + ops-security
- Residual: M (fuzz corpus baseline)
- Frameworks: SOC 2 CC6.7, CC7.1; ISO 27001 A.8.28; GDPR Art. 32; OWASP Top 10 A03:2021 (Injection)

**T-T-02 — Dependency-graph cycle injection (DoS via deeply-nested cycle attempt)**
- Asset: dependency-graph
- L M / I H / Risk H
- Mitigations:
  - Cycle prevention at write time per ADR-TASKS-0002; bounded BFS with 50ms p99 budget.
  - Per-tenant rate limit on dependency-edge writes.
  - Worker queue alarm on cycle-check timeout.
- Owner: axis-tasks
- Residual: L
- Frameworks: SOC 2 CC7.1; ISO 27001 A.8.6, A.8.32; GDPR Art. 32(1)(c)

**T-T-03 — Importer payload-as-malware (Jira/Asana/Trello XML/JSON injection / XXE / billion-laughs)**
- Asset: importer parse path
- L H / I H / Risk H
- Mitigations:
  - Subprocess sandbox + cgroup memory cap + 5min timeout + size limit per file.
  - Parser refuses XXE expansion (XML external-entity attack vector).
  - Per-tenant rate limit on imports: max 10/hour.
  - LEAN check `oya-check-importer-sandbox-config` validates sandbox config at PR time.
- Owner: axis-tasks + ops-security
- Residual: M (fuzz corpus baseline)

**T-T-04 — Search-index cache poisoning leads to wrong tenant seeing wrong tasks**
- Asset: Meilisearch index
- L M / I H / Risk H
- Mitigations:
  - Per-tenant index name + master key prefix; cross-tenant read forbidden by Meilisearch ACL.
  - LEAN check `oya-check-search-index-tenant-prefix` + property test on adapter; full-rebuild idempotency property.
  - Per-tenant API key scope hash includes tenant_id.
- Owner: axis-tasks + ops-security
- Residual: L

**T-T-05 — Audit-chain seal omission for task update**
- Asset: audit emission
- L L / I H / Risk M
- Mitigations:
  - Every task write path emits via `audit-chain` µservice port; LEAN check `oya-check-audit-emission-coverage` refuses build if any mutating usecase skips emission.
  - Audit-chain µservice acks emission; missing acks trigger `held` SLO state via observability.
- Owner: audit-chain + axis-tasks
- Residual: L
- Frameworks: SOC 2 CC4.1, CC7.2, CC8.1; ISO 27001 A.5.28, A.8.15; GDPR Art. 5(2), Art. 30

### Repudiation

**T-R-01 — Task author denies assigning a task to victim**
- Asset: task-assignment chain
- L L / I M / Risk L-M
- Mitigations:
  - Every TaskAssigned emission carries assigner SPIFFE/OIDC subject + Ed25519 audit-chain seal.
  - Task history retained immutably; replayable from audit ledger.
- Owner: axis-tasks
- Residual: L

**T-R-02 — Auto-assign decision repudiated ("I never approved auto-assign")**
- Asset: T2 auto-assign classifier decision
- L M / I H (employment context) / Risk H
- Mitigations:
  - Per ADR-TASKS-0006: T2 auto-assign in employment-context REFUSED at Cedar layer; only proceeds where conformity assessment is complete.
  - When permitted, T2 auto-assign emits per-decision Ed25519-sealed audit record with model id + version + feature vector hash + decision rationale.
  - Per Art. 14 EU AI Act human-oversight: decision reversible within 30s reversibility window.
- Owner: axis-tasks + council-privacy
- Residual: L (when conformity is complete)
- Frameworks: GDPR Art. 22 + EU AI Act Annex III §4

### Information Disclosure

**T-I-01 — Personal-context task leaks into Professional-context query**
- Asset: dual-context isolation
- L M / I H / Risk H
- Mitigations:
  - Context field non-nullable + immutable post-creation; Cedar policy `task-isolation.cedar` refuses cross-context read.
  - Rust kernel types: `PersonalTask` vs `ProfessionalTask` separate structs.
  - LEAN check `oya-check-context-isolation` refuses build on cross-context query.
- Owner: axis-tasks + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.3; GDPR Art. 5(1)(b), 25

**T-I-02 — Cross-tenant search-index leak (tenant A's docs returned for tenant B query)**
- Asset: Meilisearch
- L M / I H / Risk H
- Mitigations:
  - Per-tenant index name + per-tenant master key scope.
  - LEAN check on adapter ensures tenant prefix wired.
  - Penetration test against cross-tenant query annually + on every BC change.
- Owner: axis-tasks + ops-security
- Residual: L
- Frameworks: SOC 2 CC6.1; ISO 27001 A.5.15, A.8.12; GDPR Arts. 5(1)(f), 25, 32; KR PIPA Art. 23

**T-I-03 — Webhook payload contains task fields beyond minimum-necessary**
- Asset: webhook fanout
- L M / I M / Risk M
- Mitigations:
  - Webhook payload schema declared per `contracts/asyncapi/tasks-events.yaml`; LEAN check refuses build on schema drift.
  - Tenant operator selects field projection per webhook subscription.
- Owner: axis-tasks
- Residual: L

**T-I-04 — Custom-field of type "person" leaks identifier outside tenant**
- Asset: custom-field export
- L M / I M / Risk M
- Mitigations:
  - Export filters person-field values by requestor role; non-tenant-admins receive only their own user-id.
  - Cedar policy `export-projection.cedar`.
- Owner: axis-tasks
- Residual: L

**T-I-05 — Time-tracking ticks (M02-onward) reveal employee work patterns**
- Asset: time-tracking telemetry
- L M / I H (employee surveillance risk) / Risk H
- Mitigations:
  - Time-tracking is opt-in per employee; tenant cannot force without explicit consent (EU AI Act + GDPR Art. 22 + 근로기준법 + Title VII).
  - Per-employee retention bounded (default 90d; tenant override up to employment-record retention floor).
  - Aggregation-only reporting; individual ticks accessible only to the employee themselves + 2-person-rule admin.
- Owner: axis-tasks + council-privacy
- Residual: M (employment-context surveillance baseline)
- Frameworks: GDPR Art. 22; EU AI Act Annex III §4; 근로기준법; Title VII; ADA

**T-I-06 — Tenant-DEK leaked via log emission**
- Asset: encryption keys
- L M / I H / Risk H
- Mitigations:
  - DEK wrapped in `Secret<T>` type with stripped `Debug` impl; never serializable.
  - Secret-scanner CI lane scans every commit + log emission.
  - Rotation: 90d for tenant-DEK; rotation event re-encrypts active records.
- Owner: ops-security + cloud-secrets
- Residual: M (human-error baseline)

### Denial of Service

**T-D-01 — Bulk-edit storm: tenant submits 100k-task bulk update**
- Asset: bulk-edit pipeline
- L M / I H / Risk H
- Mitigations:
  - Per-tenant rate limit on bulk-edit; max 100 tasks/sec per tenant baseline.
  - Bulk operations require explicit second-confirmation for >10k tasks.
  - All-or-nothing atomicity bound to per-batch (1000 tasks); rollback on failure.
  - Worker queue depth alarm; backpressure to caller.
- Owner: ops-sre-reliability + axis-tasks
- Residual: L

**T-D-02 — Recurrence storm (deeply-nested RRULE)**
- Asset: recurrence engine
- L H / I H / Risk H
- Mitigations:
  - RRULE complexity bound at API per ADR-TASKS-0003 (aligned with calendar ADR-CAL-0002): max INTERVAL=1, max COUNT=10000, max horizon 5y.
  - Worker rate-limit per tenant: max 100 RRULE expansions/min.
  - Per-tenant cost-meter; excess returns 429.
- Owner: ops-sre-reliability + axis-tasks
- Residual: L

**T-D-03 — Search-index full-rebuild storm (1M-task tenant rebuild)**
- Asset: Meilisearch index
- L M / I H / Risk H
- Mitigations:
  - Per-tenant rate limit on full-rebuild requests; max 1/day.
  - Rebuild runs async with backpressure; degraded mode falls back to direct-Postgres search.
  - Rebuild completes ≤30min for 10M tasks per AC-09.
- Owner: axis-tasks + ops-sre-reliability
- Residual: L

**T-D-04 — Importer storm (concurrent 10-importer-job tenant)**
- Asset: importer worker pool
- L M / I M / Risk M
- Mitigations:
  - Per-tenant concurrent-import-job limit: 3 jobs.
  - Subprocess sandbox + 5min timeout prevents single-job runaway.
  - Worker queue depth alarm.
- Owner: axis-tasks
- Residual: L

**T-D-05 — Webhook fanout flood (tenant subscribes 100+ webhooks)**
- Asset: webhook dispatcher
- L M / I M / Risk M
- Mitigations:
  - Per-tenant webhook subscription cap: 50.
  - Webhook dispatch is async + rate-limited at outbound boundary.
  - Per-webhook circuit-breaker; unhealthy targets stripped from fanout temporarily.
- Owner: axis-tasks
- Residual: L

### Elevation of Privilege

**T-E-01 — Non-admin user creates project + assigns themselves admin role**
- Asset: project membership
- L L / I M / Risk L-M
- Mitigations:
  - Project creation defaults to creator-only admin; adding admins requires tenant-admin OIDC token.
  - Cedar policy refuses self-admin-grant.
- Owner: axis-tasks
- Residual: L

**T-E-02 — Worker SA token leaked → arbitrary task writes**
- Asset: worker ServiceAccount
- L L / I H / Risk M
- Mitigations:
  - SA token bound to pod identity; rotation 24h.
  - Network policy: worker → DB + Meilisearch only; not user-facing.
  - Worker writes are scoped to system-emitted events (audit + retention sweep + recurrence + importer + webhook); user-facing writes go via REST.
- Owner: ops-security + axis-tasks
- Residual: L

**T-E-03 — Legal-hold bypass via raw DB access**
- Asset: legal-hold preservation
- L L / I H / Risk M
- Mitigations:
  - Postgres role for application has no DELETE permission; only soft-delete via row column.
  - Hard-delete restricted to a `purge-with-2-person-rule` admin script audited via audit-chain.
  - Periodic integrity scan: compare hold-set vs Postgres rows; mismatch alerts.
- Owner: ops-security + compliance
- Residual: L

**T-E-04 — Foundry-runtime classifier output writes tasks bypassing Cedar**
- Asset: foundry-runtime → task-store bridge
- L L / I H / Risk M
- Mitigations:
  - Foundry-runtime emits classification via Workflow event; task-store usecase wraps materialisation with Cedar policy admission per ADR-TASKS-0006.
  - LEAN check `oya-check-foundry-bridge-cedar-gate` verifies admission wiring.
- Owner: axis-tasks + ops-security
- Residual: L

## LINDDUN Privacy Catalog

| ID | Category | Asset | Description | Mitigation | Residual |
|---|---|---|---|---|---|
| T-L-01 | Linkability | task assignee + comment history | repeated assignment links employees to projects → social graph + role inference | tenant-DEK + access controls; cross-tenant aggregations refused | M (legitimate use case) |
| T-L-02 | Identifiability | task description content | "Performance review for X" identifies subject | redaction in export per requestor role; Cedar gate | L |
| T-L-03 | Non-repudiation | T2 auto-assign decision | employee disputes auto-assign authorship | per-decision Ed25519 + audit chain (when permitted) | L (when conformity complete) |
| T-L-04 | Detectability | task volume burst | spike in tasks (e.g., M&A diligence) correlates with business events | reasonable disclosure to tenant; no broader mitigation | M |
| T-L-05 | Disclosure | external webhook | tenant misconfigures webhook to public endpoint | per-webhook destination validation + circuit-breaker; LEAN check refuses webhook to public IP without tenant signature | L |
| T-L-06 | Unawareness | end-user (employee) of T2 auto-assign in their employment context | employee may not know AI is assigning their work | EU AI Act Art. 50 user labelling; tenant DPA mandates upstream disclosure | M-H (joint controllership) |
| T-L-07 | Non-compliance | GDPR Art. 17 right-to-erasure | erasure of employee identifier across many tasks + history | DSR cascade: scan tasks for identifier; tombstone the assignee record; preserve task minus identifier; legal hold may override | M (best-effort within hold) |
| T-L-08 | Non-compliance | EU AI Act Annex III §4 (high-risk) | auto-assign affecting employment without conformity assessment | T2 auto-assign REFUSED at Cedar layer for employment-context until ADR-TASKS-0006 conformity-ADR ships | L (refusal is the mitigation) |
| T-L-09 | Non-compliance | Title VII / EEOC UGESP / ADA (US) | auto-assign bias against protected class | T2 auto-assign in employment context refused for pack-us until fairness-audit complete; ADR-TASKS-0006 + T2-auto.yaml `fairness-audit` field | L (refusal mitigation) |

## Mitigations Catalog

| Mitigation | Type | Owner | Verification |
|---|---|---|---|
| Postgres per-tenant RLS | Preventive | axis-tasks | `oya-check-rls-coverage` LEAN lane |
| Tenant-DEK envelope encryption | Preventive | cloud-secrets | DEK binding integrity check |
| Cedar `task-isolation.cedar` | Preventive | ops-security | policy unit-tests |
| Custom-field strict type coercion | Preventive | axis-tasks | LEAN check + adapter property test |
| Dependency-graph cycle prevention at write time | Preventive (correctness) | axis-tasks | `dependency-graph-cycle-prevention` gate + property test |
| Per-tenant Meilisearch index + master key prefix | Preventive | axis-tasks | LEAN check |
| Importer subprocess sandbox (cgroup + 5min + size limit) | Preventive (DoS) | axis-tasks + ops-security | LEAN check on sandbox config |
| Importer strict assignee resolution | Preventive | axis-tasks | LEAN check + test corpus |
| RFC 5545 RRULE bounds enforcement (5y horizon, complexity cap) | Preventive | axis-tasks | LEAN check (shared with calendar) |
| Ed25519 audit-chain seal on every mutation | Detective + non-repudiation | audit-chain | per-event emission |
| Per-tenant rate limits (writes / bulk-edit / search / imports / webhooks) | Preventive (DoS) | ops-sre-reliability | metrics |
| Cedar refusal of T2 auto-assign in employment-context until conformity ADR | Preventive (EU AI Act) | ops-security + council-privacy | LEAN check on Cedar policy admission table |
| Workflow-bridge attribution chain (`created_by_workflow_run_id`) | Preventive (spoofing) | axis-tasks | LEAN check |
| 2-person rule on hard-delete | Preventive (insider) | ops-security | OpenBao JIT |
| DSR cascade runner | Compliance | council-privacy | DSR queue SLO |

## Residual Risk Acceptance

| Risk ID | Residual | Why accepted | Re-review |
|---|---|---|---|
| T-T-01 (custom-field injection) | M | fuzz corpus baseline | Quarterly |
| T-T-03 (importer payload-as-malware) | M | fuzz corpus baseline | Quarterly |
| T-I-05 (time-tracking surveillance) | M | employment-context surveillance baseline; opt-in mitigates partially | Annually |
| T-I-06 (DEK leak via logs) | M | human-error baseline | Quarterly |
| T-L-01 (linkability) | M | legitimate use case | Annually |
| T-L-04 (detectability via burst) | M | tenant business reality | Annually |
| T-L-06 (joint-controllership unawareness) | M-H | tenant-of-tenant disclosure responsibility | Annually |
| T-L-07 (right-to-erasure best-effort) | M | hold-vs-erasure tension | Annually |

Sign-off:
- council-architecture: `pending`
- ops-security: `pending`
- council-privacy: `pending`

## Per-Pack Overlays

### pack-kr (KR PIPA + 근로기준법 + 직장 갑질 + ISMS-P)

Pack-specific threats:

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-KR-01 | I — disclosure | KR PIPA Art. 17 cross-border = SCC-gated; tasks cross-region replication forbidden by default | per-pack data residency pinning at `iac/kustomize/overlays/pack-kr/`; cross-pack ingress refused |
| T-KR-02 | N — non-compliance | 근로기준법 Art. 41 retention floor 3y for employment records — task assignment history is employment-record-adjacent for full-time-employee tenants | retention floor enforced at `task-store-domain`; legal-hold extends past floor |
| T-KR-03 | I — linkability | 직장 갑질 (workplace-harassment) protections — auto-assign of high-workload tasks could be harassment vector | T2 auto-assign REFUSED at Cedar layer for pack-kr employment contexts (mirrors pack-eu EU AI Act refusal) |
| T-KR-04 | T — tampering | 전자문서법 audit-chain integrity for employment-record-adjacent task history | Ed25519 + Merkle audit-chain per Bominal ADR-0028 |

References: KR PIPA Arts. 17/23/28; 근로기준법 Arts. 23/41; 직장 갑질 protections; PIPC Notice 2020-7.

### pack-eu (GDPR + ePrivacy + EU AI Act)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-EU-01 | I — disclosure | GDPR Art. 6(1)(a) lawful-basis | Cedar-gated; consent recorded in audit-chain |
| T-EU-02 | N — non-compliance | GDPR Art. 17 right-to-erasure must reconcile with legal-hold | erasure refused while legal-hold active; tenant comms emitted |
| T-EU-03 | N — non-compliance | **EU AI Act Annex III §4 — auto-assign in employment context = HIGH-RISK** | T2 auto-assign REFUSED at Cedar layer pending ADR-TASKS-0006 conformity-assessment ADR + `T2-auto.yaml` declares classification |
| T-EU-04 | N — non-compliance | GDPR Art. 22 — automated decisions affecting employment require explicit consent + human override | Even when EU AI Act conformity complete, T2 auto-assign carries 30s reversibility per Art. 14; per-decision Ed25519 audit |
| T-EU-05 | T — cross-border | GDPR Chapter V cross-border transfers require SCCs | per-pack data residency; cross-pack transfers SCC-gated |

References: GDPR Regulation (EU) 2016/679; ePrivacy Directive 2002/58/EC; EU AI Act Regulation (EU) 2024/1689 Annex III §4 + Art. 14 + Art. 50 + Art. 22.

### pack-us (CCPA / CPRA / Title VII / EEOC UGESP / ADA)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-US-01 | I — discovery | FRCP Rule 26(b)(1) discovery may compel task export | legal-hold + eDiscovery export per `task-store-usecase` |
| T-US-02 | N — non-compliance | **EEOC UGESP 1978 + Title VII + ADA** — auto-assign could create disparate-impact for protected class | T2 auto-assign in employment-context REFUSED at Cedar layer for pack-us until fairness-audit per `slos/auto-assign-fairness-correctness.openslo.yaml` is green |
| T-US-03 | S — spoofing | CCPA / CPRA right-to-access requires identity verification | OIDC + tenant-API-key + (optional) hardware-token |

References: CCPA / CPRA; SOC 2 TSC 2017+2022; FRCP Rule 26(b)(1); EEOC UGESP 1978 (29 CFR §1607); Title VII Civil Rights Act 1964; ADA 42 USC §12101.

### pack-us-healthcare (HIPAA + ADA + clinical-task assignment)

| Threat | STRIDE/LINDDUN | Rationale | Mitigation |
|---|---|---|---|
| T-HC-01 | I — disclosure | HIPAA 45 CFR §164.502(b) minimum-necessary on clinical-task assignment metadata | data-class `PHI` on every clinical task field; Cedar refuses access outside care-team scope |
| T-HC-02 | N — non-compliance | HIPAA 45 CFR §164.312(a)(2)(iv) encryption | Tenant-DEK envelope at rest; TLS 1.3 in transit |
| T-HC-03 | I — linkability | HIPAA 45 CFR §164.514(b) safe-harbor de-identification doesn't apply | encryption + Cedar; legal-hold for ePHI per BAA |
| T-HC-04 | T — audit-chain | HIPAA 45 CFR §164.312(b) | Ed25519 + Merkle per Bominal ADR-0028 |

References: HIPAA 45 CFR §164.308 + §164.312 + §164.502 + §164.514; FDA 21 CFR Part 11; BAA template per `legal/baa-template.md`.

### pack-jp / pack-sg / pack-au / pack-in / pack-br / pack-ae / pack-ksa

Per-pack overlays at `regional-packs/<pack>/tasks-overlay.md` (scheduled-for-distinct-tracked-work to per-pack activation IPs).

## Re-review Triggers

- Any change to dual-context isolation invariant.
- Any change to dependency-graph cycle prevention algorithm.
- Any AI capability promotion (T0 → T1 → T2).
- Any new EU AI Act Annex III §4 conformity-assessment scope expansion.
- Any new pack activation.
- Quarterly scheduled.
- Post-incident.
- Pen-test or audit finding.

## References

- ADR-0028 (Bominal): Audit chain (Merkle + Ed25519).
- ADR-0056, ADR-0105, ADR-0106, ADR-0117, ADR-0135, ADR-0130, ADR-0131, ADR-0132, ADR-0140.
- ADR-TASKS-0001 (data model + custom fields strict coercion).
- ADR-TASKS-0002 (dependency graph + cycle prevention).
- ADR-TASKS-0005 (workflow-engine bridge).
- ADR-TASKS-0006 (AI auto-assign + EU AI Act Annex III §4 bounds).
- `microservices/tasks/PRD.md`, `dpia.md`, `compliance.md`, `policy/*.cedar`.
- RFC 5545 (iCalendar VTODO).
- Microsoft Threat Modeling (STRIDE), LINDDUN privacy.
- NIST SP 800-154; OWASP ASVS v4.0.3; SLSA L3; NIST SSDF SP 800-218.
- EU AI Act Regulation (EU) 2024/1689 — Art. 14 + Art. 22 + Art. 50 + Annex III §4.
- EEOC UGESP 1978 (29 CFR §1607); Title VII Civil Rights Act 1964; ADA.
- 근로기준법 (Korean Labour Standards Act); 직장 갑질 protections.
- ISO 30414 (HR analytics).
- WCAG 2.2 AA (accessibility).
- `microservices/calendar/threat-model.md` — sibling reference template.
