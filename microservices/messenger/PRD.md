---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-messenger
microservice: messenger
status: Accepted
sales_segment: connect-suite-product
tier: hero-product
milestone_first_ship: M02-foundation
bominal_source: [ADR-0208-connect-dual-context-unified-channel-hub.md, ADR-0215-connect-retention-legal-hold-dual-context.md]
related_adrs: [ADR-0008, ADR-0056, ADR-0105, ADR-0106, ADR-0135, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0172]
related_specs: [/specs/microservices/messenger.json, /specs/per-microservice-flat-layout.json, /specs/agentic-slo-gated-promotion.json]
date: 2026-05-17
owner_team: axis-messenger
doc_status: published
---

# PRD-messenger: Team-Channels + DM + Threads + Presence

## Purpose

The `messenger` microservice is the native team-channels-plus-direct-messages surface that ships under ADR-0132 (product-suite-and-bundle dissolution) as a stand-alone hero µservice, factored out of the legacy Connect-suite by parallel-session ADR-0135. It owns **team channels + direct messages + threaded conversation + read receipts + file sharing + inline reactions + @mentions + channel-level RBAC + message search**, across the dual-context model (Personal B2C + Professional B2B) inherited from Bominal ADR-0208 via parallel ADR-0135.

This µservice is **a hero product**, end-user-facing through Workflow Studio shell and standalone messenger clients (web + desktop + mobile). It is also consumable as a shared substrate by other oyatie products via the `messenger.message.v1` Workflow events and the `MessageThread` Ontology object type.

Bominal predecessor: the `connect-messenger` slice of Bominal's unified Connect suite. Per parallel ADR-0135, that monolithic suite is dissolved into per-surface µservices; this PRD is the canonical messenger landing in oyatie.

## Tenant Value

- **Tenant Outcome 1 — Team coordination without app fragmentation.** Tenants get Slack/Teams-class channel + DM + thread workflows in the same shell as mail, calendar, workflow studio, ontology browser — switching personal/professional context without leaving the surface.
- **Tenant Outcome 2 — Dual-context-safe collaboration.** Personal (B2C) DMs never cross into professional (B2B) audit scope per parallel ADR-0135; professional channels carry tenant-DEK encryption + four-eyes audit disclosure inherited from Bominal ADR-0215.
- **Tenant Outcome 3 — Real-time delivery with read-receipt and presence integrity.** p99 message-send ≤ 100ms within region; presence-update p99 ≤ 1s; read-receipt fan-out under WebSocket backpressure remains idempotent.
- **Tenant Outcome 4 — Channel-scoped RBAC + message search that respects Cedar policy.** Search results are filtered server-side by Cedar evaluation; no client-side trust; pack-aware retention bounds (KR PIPA / GDPR / HIPAA when channels carry PHI).
- **Tenant Outcome 5 — Native Workflow + Ontology integration.** Mention-resolution reads from Ontology (`Person`, `Team`, `Channel`); action-cards from `mail` µservice surface inline; channel events feed Workflow Studio engines.

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | end-user | to post a message to a channel I am a member of | the team sees it in real-time | message-stream | Must |
| FR-02 | end-user | to reply in a thread off a parent message | conversation stays scoped | thread-tree | Must |
| FR-03 | end-user | to see read receipts per recipient | I know who has seen my message | read-receipt-tracker | Must |
| FR-04 | end-user | to attach files (≤ 5GB) to messages | rich collaboration works | file-attachment | Must |
| FR-05 | end-user | to react inline with emoji | low-overhead acknowledgement works | message-stream | Must |
| FR-06 | end-user | to @mention people, teams, channels | recipients are notified + linked | mention-router | Must |
| FR-07 | channel-admin | to grant/revoke channel-level RBAC per principal | sensitive channels stay scoped | channel-store | Must |
| FR-08 | end-user | to search messages across channels I can read | I can recover old context | message-stream + search-adapter | Must |
| FR-09 | end-user | to see presence (online/away/dnd) for teammates | I time my comms | presence | Must |
| FR-10 | end-user | to switch personal/professional persona | dual-context isolation is preserved | channel-store | Must |
| FR-11 | compliance-officer | to issue eDiscovery hold on professional channels | regulatory request is satisfied | channel-store + file-attachment | Must |
| FR-12 | tenant-admin | to configure pack-aware retention bounds | per-pack regulatory bounds hold | channel-store | Must |
| FR-13 | Workflow Studio | to consume `MessagePosted` / `MentionEmitted` events | downstream automation works | message-stream | Must |
| FR-14 | mail µservice | to emit action-cards into a channel via Workflow event | inline action delivery works | mention-router | Should |
| FR-15 | tenant-operator | to query channel ACL coverage + message throughput | I can plan capacity | channel-store + message-stream | Must |

## Non-Functional Requirements

### Performance

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Message-send latency (post → fanout-ack) | ≤ 30ms | ≤ 100ms | ≤ 250ms | within-region; cross-region fans out asynchronously |
| Channel-fetch latency (cold list of ≤ 50 channels) | ≤ 60ms | ≤ 200ms | ≤ 500ms | Postgres + Valkey cache |
| Presence-update propagation | ≤ 200ms | ≤ 1s | ≤ 3s | Valkey pub/sub + WebSocket gateway |
| Read-receipt fan-out (per-recipient) | ≤ 50ms | ≤ 150ms | ≤ 500ms | coalesced under 250ms windows |
| File-attachment upload init (5MB chunk) | ≤ 100ms | ≤ 300ms | ≤ 1s | S3 multipart |
| Message-search query (≤ 50 results, single tenant) | ≤ 80ms | ≤ 350ms | ≤ 1s | Tantivy / Elasticsearch |
| @mention resolution + notification | ≤ 80ms | ≤ 250ms | ≤ 1s | Ontology lookup + fanout |

### Security

- Channel-level RBAC enforced server-side via Cedar policy (`policy/channel-scope.cedar`); client never trusted.
- Personal-context DMs end-to-end encrypted; tenant operators + oyatie operators MUST NOT have plaintext access (inherited from Bominal ADR-0208).
- Professional-context channels tenant-DEK encrypted (envelope encryption per Bominal ADR-0111); admin disclosure requires four-eyes audit trail per Bominal ADR-0215.
- File attachments scanned via OPSWAT MetaDefender or ClamAV before persistence (`runbooks/attachment-malware-quarantine.md`).
- All WebSocket connections mTLS-terminated; per-tenant API token bound at OpenBao with rotation 30d.
- Search index excludes redacted PII / PHI per `policy/redaction-phi.md` (pack-us-healthcare overlay).
- Cross-context routing forbidden: a personal DM cannot become a professional channel reply; enforced by `policy/dual-context-isolation.md`.

### Audit + Compliance

- Every channel-create / channel-delete / member-grant / member-revoke / four-eyes-disclosure event writes an audit-chain record (Merkle / Ed25519 per Bominal ADR-0028).
- Professional-context disclosure (admin reads message body for compliance) requires two distinct approving principals + reason code (per Bominal ADR-0215).
- Retention: per-pack bounds in `policy/data-residency.md`. KR PIPA work-mail floor satisfied. GDPR storage-limitation honored. HIPAA pack: PHI retention 6 years where applicable.
- eDiscovery export bundles message + thread + attachment + audit-chain seal under `runbooks/ediscovery-export.md` (Slice B).

### Availability + SLO

- Availability target: 99.95 % monthly for message-send + WebSocket connectivity.
- Read-receipt + presence are best-effort; 99.9 % monthly.
- RTO: ≤ 15 min for message-store. RPO: ≤ 5 min (cross-region replication for professional store).

### Data residency

- Per-tenant pack pinning per ADR-0117. Personal-context user data follows the personal-residency model (per-user); professional follows tenant-residency.
- Cross-pack message routing forbidden by default; explicit federation seam in `multi-region.md`.

### Protocols

The messenger µservice's wire formats are pinned to the following published specifications. Pin versions are mandatory for any release branch; protocol upgrades require an ADR + dual-version-window per `feedback_no_silent_regression`.

| Protocol surface | Spec | Pinned version | Notes |
|---|---|---|---|
| Federated Client-Server | Matrix Client-Server API | **r0.6.1** (matrix.org LTS) | governs the federated client surface; r0.6.1 is the long-term stable line currently mandated; upgrades to v1.x require an ADR per the no-silent-regression rule |
| Federated Server-Server | Matrix Server-Server API | **r0.1.4** (matrix.org LTS) | governs cross-pack federation hop; cross-pack routing remains default-deny per data-residency above, but where a tenant opts in, the Matrix r0.1.4 federation spec is the wire format |
| E2E key agreement | Matrix Olm + Megolm | Olm 3.x line; Megolm 1.x line | personal-context DM end-to-end encryption (`feedback_workflow_objectgraph_adapter_layer (retired per ADR-0145)` carrier exemption permits direct egress only at the transport layer; payloads remain E2E) |
| Real-time transport | WebSocket (RFC 6455) + mTLS | RFC 6455 (final) | client connections; per-tenant API-token-bound at OpenBao per `feedback_no_silent_regression` rotation 30d |
| Action-card carrier | AsyncAPI 2.6 | contracts/asyncapi/action-cards.yaml | mail → messenger action-card ingest contract (channel-mention carrier path) |
| Search index protocol | Tantivy 0.21 / Elasticsearch 8.x | search backend pinned per AdR-MSG-0001 | indexed surface only; Cedar-policy-filtered |

Matrix references: the Matrix Foundation publishes both APIs at `spec.matrix.org`. The r0.6.1 (Client-Server) + r0.1.4 (Server-Server) pin matches the long-term-stable line used by Element / Synapse production deployments; this pin is the canonical-base. Per ADR-0064 canonical-base + localization, per-pack overlays MAY pin a newer minor (e.g., r0.6.1 → r0.6.1+pack-eu-erasure-extension) but MUST NOT drift the major.

Federation with non-Matrix targets (Slack / Teams external adapter): scoped out of P01 per Open Question 3 below; if admitted, the adapter MUST be Matrix-bridged (matrix.org Mattermost / Slack bridges as the reference shape) rather than direct.

## Bounded Contexts

Per ADR-0105 (13-value canonical layer enum) and ADR-0106 (`application` → `usecase` rename). Layers used: `kernel`, `domain`, `usecase`, `api`, `adapter`, `adapter-postgres`, `adapter-redis`, `adapter-s3`, `adapter-websocket`, `adapter-search`, `rest`, `worker`, `sdk`, `app`.

| BC | Crate family | Purpose | Key entities |
|---|---|---|---|
| `channel-store` | `oya-messenger-channel-store-{kernel,domain,usecase,api,adapter,adapter-postgres,rest,sdk,app}` | Channel + DM CRUD; channel-level RBAC; retention policy assignment; eDiscovery hold | `Channel`, `DirectConversation`, `ChannelMember`, `RetentionPolicy`, `Hold` |
| `message-stream` | `oya-messenger-message-stream-{kernel,domain,usecase,api,adapter,adapter-postgres,adapter-search,rest,worker,sdk,app}` | Message append + edit + delete + reactions; real-time fanout via WebSocket; search index emission | `Message`, `Reaction`, `Edit`, `Delete`, `SearchDoc` |
| `thread-tree` | `oya-messenger-thread-tree-{kernel,domain,usecase,api,adapter-postgres,rest,sdk,app}` | Thread reply chains; parent-child traversal; thread participant tracking | `Thread`, `ThreadReply`, `ThreadParticipant` |
| `read-receipt-tracker` | `oya-messenger-read-receipt-tracker-{kernel,domain,usecase,api,adapter-redis,worker,sdk,app}` | Per-recipient last-read-message-id; coalesced fanout; backpressure-idempotent | `ReadReceipt`, `LastReadCursor` |
| `file-attachment` | `oya-messenger-file-attachment-{kernel,domain,usecase,api,adapter,adapter-s3,worker,sdk,app}` | Attachment upload (multipart S3); malware scan; preview generation; encrypted blob refs | `Attachment`, `BlobRef`, `PreviewVariant`, `MalwareScanResult` |
| `mention-router` | `oya-messenger-mention-router-{kernel,domain,usecase,api,adapter,worker,sdk,app}` | @mention parse; Ontology lookup; fanout via notification + WebSocket; action-card ingest from mail | `Mention`, `MentionTarget`, `MentionFanoutPlan` |
| `presence` | `oya-messenger-presence-{kernel,domain,usecase,api,adapter-redis,adapter-websocket,worker,sdk,app}` | Online/away/dnd presence; per-tenant pub/sub; degraded-mode fallback | `Presence`, `PresenceTransition`, `DegradedModeFlag` |

Naming justification — `channel-store`:

```
NAME: oya-messenger-channel-store-<layer>
JUSTIFICATION:
- microservice = messenger: per ADR-0131 per-microservice flat layout.
- bc-tokens = channel-store: primary BC. ADR-0056 v4.1 BC-optionality rule honoured.
- layer = <layer>: ADR-0105 13-value canonical enum; ADR-0106 usecase rename.
- exemptions claimed: -adapter-postgres is canonical *-adapter-<backend> per ADR-0105 Amendment 3.
```

Total crates introduced: **52**.

Port traits declared in each kernel (zero business logic; zero I/O; `data_class` annotated per Bominal ADR-0028):

| Port trait | Kernel crate | Implementation | Data classes touched |
|---|---|---|---|
| `ChannelRepository` | `oya-messenger-channel-store-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `AUDIT` |
| `MessageStore` | `oya-messenger-message-stream-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT`, `PII_IDENTIFYING` (sometimes), `PHI` (pack-us-healthcare) |
| `MessageSearchIndex` | `oya-messenger-message-stream-kernel` | `-adapter-search` (Tantivy + Elasticsearch backends) | `BEHAVIORAL_TENANT_PRODUCT` |
| `RealtimeBroadcaster` | `oya-messenger-message-stream-kernel` | `-adapter-websocket` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ThreadRepository` | `oya-messenger-thread-tree-kernel` | `-adapter-postgres` | `BEHAVIORAL_TENANT_PRODUCT` |
| `ReadReceiptStore` | `oya-messenger-read-receipt-tracker-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `AttachmentBlobStore` | `oya-messenger-file-attachment-kernel` | `-adapter-s3` | `BEHAVIORAL_TENANT_PRODUCT`, sometimes `PII_IDENTIFYING` / `PHI` |
| `MalwareScanner` | `oya-messenger-file-attachment-kernel` | `-adapter` (OPSWAT / ClamAV) | `INTERNAL_ONLY` |
| `MentionResolver` | `oya-messenger-mention-router-kernel` | `-adapter` (Ontology client) | `PII_IDENTIFYING` |
| `PresenceStore` | `oya-messenger-presence-kernel` | `-adapter-redis` | `BEHAVIORAL_TENANT_PRODUCT` |
| `WebSocketGateway` | `oya-messenger-presence-kernel` | `-adapter-websocket` | `BEHAVIORAL_TENANT_PRODUCT` |
| `CedarChannelPolicy` | `oya-messenger-channel-store-kernel` | `-adapter` (Cedar evaluator) | `INTERNAL_ONLY` |
| `AuditChainClient` | (cross-BC) `-kernel` per BC | (cross-BC) `-adapter` to audit-chain µservice | `AUDIT` |

Data-class enforcement: `oya-check-data-class` LEAN lane refuses unannotated fields.

Cross-product rule: `messenger` MUST NOT import any other product µservice crate at any layer. Cross-product flows go through Workflow (events) or Ontology (entity reads/writes). LEAN-A2 lane enforces.

CI lanes that must green:

- `oya gate validate lean-a1 --microservice messenger` — dependency-direction
- `oya gate validate lean-a2 --microservice messenger` — cross-product-refusal
- `oya gate validate port-location --microservice messenger`
- `oya gate validate layer-correctness --microservice messenger`
- `oya gate validate per-microservice-layout --microservice messenger`
- `oya gate validate statelessness --microservice messenger`
- `oya gate validate shardability --microservice messenger`
- `oya gate validate authority-cohesion --microservice messenger` (HG-MESSENGER)
- `oya gate validate dual-context-isolation --microservice messenger` (NEW; per parallel ADR-0135)

## Integration via Workflow + Ontology

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `MessagePosted` | end-user posts message | search-index, mention-router, downstream Workflow engines, audit-chain | append-only |
| `MessageEdited` | end-user edits within edit-window | search-index, audit-chain | append-only delta |
| `MessageDeleted` | end-user / admin deletes | search-index, audit-chain, retention-purge worker | tombstone |
| `MessageReactionAdded/Removed` | end-user reacts | downstream engines | append-only |
| `PresenceChanged` | client heartbeat / disconnect | tenant presence subscribers | append-only |
| `FileAttached` | attachment upload finalised | malware-scanner worker, search-index, audit-chain | append-only |
| `MentionEmitted` | mention-router resolves a mention | notification fanout, action-card consumer (Workflow Studio) | append-only |
| `ChannelCreated / Deleted` | channel-admin action | audit-chain, ontology (`Channel` write) | append-only |
| `ChannelMemberGrantedRevoked` | channel-admin action | audit-chain, ontology | append-only |
| `EDiscoveryHoldOpenedClosed` | compliance-officer action | audit-chain, retention-purge worker | append-only |
| `FourEyesDisclosureExecuted` | tenant-admin pair approves PII read | audit-chain | append-only |

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `OntologyEntityChanged` (Person/Team/Channel) | ontology | mention-router | refresh mention-resolution cache |
| `MailActionCardEmitted` | mail | mention-router | post action-card into target channel |
| `TenantRetentionPolicyUpdated` | tenancy | channel-store | reassign channel retention bounds |
| `AuditChainSealed` | audit-chain | (read-only) | confirm audit-write durability |
| `WorkflowStudioRunStarted/Completed` | workflow-engine | mention-router | post status into bound channel |

### Ontology writes

| Object Type | Written by BC | Audit trail |
|---|---|---|
| `Channel{channel_id, tenant_id, context_kind, members, retention_policy_id}` | `channel-store` | Ed25519 |
| `MessageThread{thread_id, channel_id, parent_message_id, participant_refs}` | `thread-tree` | Ed25519 |
| `MessagePosted{message_id, channel_id, author_ref, ttl, data_class}` (link-event) | `message-stream` | Ed25519 |
| `Mention{message_id, target_ref, mention_kind}` | `mention-router` | Ed25519 |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Person`, `Team`, `Channel` | `mention-router` | `find_by(@-handle, tenant_id)` for mention resolution |
| `RetentionPolicy` | `channel-store` | `lookup(tenant_id, context_kind)` |
| `TenantContext` | every BC | tenant scope + pack jurisdiction lookup |

## Competitive Benchmark

| Competitor | Product / feature | Parity dimensions | Primary source |
|---|---|---|---|
| Slack | Workspaces + channels + DMs + threads + Slack Connect | full feature parity; thread UX; search; reactions; mentions; app integration | `slack.com/help` |
| Microsoft Teams | Channels + chats + meetings + files | enterprise-grade RBAC; eDiscovery; HIPAA | `learn.microsoft.com/microsoftteams` |
| Discord | Voice + text channels + threads + reactions | gaming-oriented; massive scale (200M MAU); real-time presence | `discord.com/developers/docs` |
| Mattermost | OSS Slack-alike; self-hosted | data-sovereignty; channel-level RBAC | `docs.mattermost.com` |
| Rocket.Chat | OSS multi-channel chat | E2E DMs; federation | `docs.rocket.chat` |
| Naver Works Chat | KR-flavored enterprise messenger | KR-first UX; KakaoTalk-style affordances | `naver.worksmobile.com` |
| Line Works | JP/KR enterprise chat | LINE-style UX; mobile-first | `line.worksmobile.com` |

Key parity gaps to close (ordered by priority):

1. **Dual-context isolation by data-model** — none of the competitors enforce personal/professional context as a data-model invariant (Slack/Teams blur via shared identity). Target: compile-time + LEAN-lane enforcement.
2. **Four-eyes admin disclosure on professional reads** — Slack/Teams allow admin discovery without two-party approval. Target: Bominal ADR-0215 four-eyes pattern.
3. **Native Workflow + Ontology integration** — competitors expose webhooks/Bot APIs; oyatie exposes typed Workflow events + Ontology object writes natively.
4. **OpenSLO + agentic gate** — none gate channel feature rollouts on SLO compliance; oyatie does (per ADR-0139).
5. **Multi-pack residency + per-pack regulatory overlays** — competitors are SaaS-region-coarse; oyatie is per-pack jurisdiction-pinned.

## Performance Targets

(See Performance NFR table above.)

Error budget:
- Monthly error budget for message-send: 0.05 % (≈ 22 min/month).
- Burn-rate alarm on `messenger.message-send.availability` is 14.4× burn rate over 1h.
- Error budget policy: `microservices/messenger/runbooks/error-budget-policy.md` (Slice B).

## Horizontal Scalability

**State strategy** (per Bominal ADR-0019 enum): `mixed`. Postgres for message store; Valkey for presence + read-receipts; S3 for attachments; Tantivy/Elasticsearch for search; WebSocket gateway stateless beyond connection registry.

**Active-active compatibility**: stateless WebSocket gateway + Postgres logical-replicated within pack; Valkey primary-replica HA; S3 cross-AZ replication.

Per-cell capacity envelope:

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Active WebSocket connections | 100k | 1M | gateway CPU > 70% or queue depth > 5s |
| Messages/sec | 5k | 50k | Postgres write IOPS > 70% |
| Channels per tenant | 1k | 50k | per-tenant cardinality limit hit |
| Attachments/day | 100k | 1M | S3 PUT rate > 70% provisioned |
| Search index size | 100GB | 5TB | shard count exceeded |

Scale-out policy:
- HPA on WebSocket gateway pods: CPU > 70 %, min 4, max 200 replicas.
- Postgres shard-by-tenant once cell hits 1M messages/sec aggregate.
- Valkey cluster sharding by `(tenant_id, channel_id) mod N`.

Sharding:
- Message store partitions by `(tenant_id, channel_id, year-month)`.
- Read-receipt store partitions by `(tenant_id, user_id)`.
- Presence store partitions by `(tenant_id, user_id)`.
- `oya-check-shardability-cli` lane verifies partition keys are present in every kernel struct.

## Acceptance Criteria

| AC-ID | Criterion | Verification method |
|---|---|---|
| AC-01 | A channel-create + post + thread-reply + reaction roundtrip completes within p99 < 100ms message-send | `microservices/messenger/tests/e2e/channel-post-thread.rs` |
| AC-02 | Personal-context DM cannot become reply on professional channel | `tests/e2e/dual-context-isolation.rs` |
| AC-03 | Professional channel admin read of bodies requires two distinct approving principals + audit-chain seal | `tests/e2e/four-eyes-disclosure.rs` |
| AC-04 | File attachment upload, scan, finalize, then revoke after retention TTL | `tests/e2e/attachment-lifecycle.rs` |
| AC-05 | @mention of Person resolves via Ontology and emits `MentionEmitted` within 250ms | `tests/e2e/mention-emit.rs` |
| AC-06 | Presence transitions propagate to peers within 1s p99 | `tests/e2e/presence-propagation.rs` |
| AC-07 | Message search returns only Cedar-permitted results | `tests/e2e/search-cedar-scope.rs` |
| AC-08 | eDiscovery export bundles message + attachment + audit-chain seal | `tests/e2e/ediscovery-export.rs` |
| AC-09 | `oya gate validate per-microservice-layout --microservice messenger` exit 0 | ADR-0131 lane |
| AC-10 | `oya gate validate authority-cohesion --microservice messenger` exit 0 | ADR-0123 lane; HG-MESSENGER registered |
| AC-11 | `oya gate validate dual-context-isolation --microservice messenger` exit 0 | NEW per parallel ADR-0135 |

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Search backend: Tantivy (self-hosted) only, or Elasticsearch fallback for large tenants? | axis-messenger | ADR after S-tier launch |
| 2 | Voice / video signaling: own µservice or messenger BC? | council-architecture | successor-IP ADR |
| 3 | Federation with external Slack/Teams via adapter — security review owner | ops-security | per-tenant opt-in ADR |
| 4 | Self-observability: messenger emits to observability µservice as one tenant or per-pack? | axis-messenger + axis-observability | resolved in IP-007 |
| 5 | E2E personal-DM key escrow policy — none, or platform-recovery only? | council-privacy | ADR successor-IP |

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| ADR-0008 | Data Use Boundary | personal/professional data-use invariants |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0105 | 13-layer enum | layer authority |
| ADR-0106 | application → usecase | layer naming |
| ADR-0135 | Connect dual-context (parallel) | dual-context isolation source |
| ADR-0139 | Agentic SLO-gated promotion | gates messenger releases |
| ADR-0131 | Per-microservice flat layout | this PRD authored under it |
| ADR-0132 | Suite-and-bundle dissolution | factored Connect into surfaces |
| ADR-0133 | Industry best-practice conformance | HG-MESSENGER under this |
| Bominal ADR-0208 | Connect dual-context unified channel hub | inherited |
| Bominal ADR-0215 | Connect retention legal-hold dual-context | inherited |
| Bominal ADR-0028 | Audit-chain Merkle + Ed25519 | inherited |
| Bominal ADR-0111 | Ciphertext property type + envelope encryption | inherited |
| ADR-0172 | Read replicas + CQRS where appropriate | this µservice's `messenger.search` BC opts in |

## CQRS Read-Replica Addendum — `messenger.search` BC (per ADR-0172)

Per ADR-0172 (2026-05-18), the `messenger.search` bounded context opts in to the read-replica CQRS split as one of the three M02 high-read BCs. Search-as-you-type produces high read traffic at every keystroke; writes happen only at message arrival.

### Declaration

| Field | Value |
|---|---|
| Bounded context | `messenger.search` |
| Command-side primary | `oya-messenger-search-primary` (Postgres 17 LTS + pg_trgm full-text index) |
| Query-side replicas | 5 read replicas + 1 dedicated full-text-search replica via pgpool-II |
| Read-staleness budget | ≤5s p99 |
| Read:write ratio justifying split | ~50×–200× (per ADR-0172 §"Context") |
| Read-after-write mechanism | per-tenant LSN pinning via `oya-read-after-write-lsn` header |

### Migration

Migration follows the ADR-0172 §"Migration / rollout plan" sequenced cutover. The dedicated FTS replica receives the heaviest GIN-index load isolated from the general read replicas; this protects general read latency from FTS query bursts.

### SLO + observability

Read-staleness SLO authored under `microservices/messenger/slos/search-read-staleness.openslo.yaml` (M02 deliverable per ADR-0139). Alert at p99 staleness > 5s.
