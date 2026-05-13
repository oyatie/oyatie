---
doc_class: PRD
template_id: TPL-PRD
prd_id: PRD-connect
microservice: connect
status: Accepted
sales_segment: Enterprise
tier: B2B
milestone_first_ship: M03-first-paying-tenant
bominal_source:
  - ADR-0208  # Connect dual-context (Personal + Professional)
  - ADR-0215  # Connect retention / legal hold / dual-context boundary
  - ADR-0210  # M3 KR group mail launch
  - ADR-0132  # data ownership pillars (person-pillar for Personal context)
  - ADR-0123  # cross-product auth cookie + redirect contract
---

# PRD-connect: Connect µservice (dual-context: Personal + Professional)

---

## Purpose

Connect is the communication µservice with two isolated contexts sharing one
codebase: **Connect Professional** (B2B; org-pillar; corporate mail + messenger
+ community; legal-hold/eDiscovery capable) and **Connect Personal** (B2C;
person-pillar; individual identity; post-M03 GA).

M03 ships **Connect Professional only** (per Bominal ADR-0210 KR group mail
launch). Connect Personal context is scaffolded but not GA until post-crypto-audit
(per `feedback_flat_product_catalog.md` §"Deferred").

Inherits Bominal ADR-0208 (dual-context architecture) and ADR-0215 (retention /
legal hold / dual-context boundary) 1:1. Workflow + Ontology are the integration
plane; Connect never calls other µservices directly.

---

## Tenant Value

### Connect Professional (M03)
- **Corporate mail** (Connect-Pro Mail): hosted email for the tenant domain;
  legal-hold and eDiscovery from day one; SMTP/IMAP compatible.
- **Messenger**: team channels + direct messages; threaded; read receipts;
  file sharing.
- **Community**: org-wide announcements; Q&A; knowledge base threads.
- **Workflow integration**: mail-to-workflow handoff; approval notifications
  delivered to inbox; action cards inline in message thread.
- **Compliance**: Korean 전자문서법; retention policies; legal hold; eDiscovery
  export in PST/MBOX; GDPR-ready jurisdiction overlays.

### Connect Personal (post-M03)
- **Individual identity**: person-pillar; not org-owned; Connect Personal data
  never co-mingled with Professional context.
- **Personal messaging**: direct messages between individuals; not employer-accessible.
- Post-M03; deferred until crypto-audit complete.

---

## Functional Requirements

| ID | As a… | I want… | So that… | BC | Priority |
|---|---|---|---|---|---|
| FR-01 | Employee (Professional) | send and receive email on my corporate domain | I have a company email address without separate mail provider | `mail` | Must |
| FR-02 | Employee (Professional) | send and receive direct messages + join team channels | team communication without Slack/Teams dependency | `messenger` | Must |
| FR-03 | HR admin | trigger employee access provisioning on `EmployeeHired` Workflow event | Connect account created automatically without manual IT ticket | `provisioning` | Must |
| FR-04 | IT/Legal admin | place a user mailbox on legal hold; prevent deletion | eDiscovery and litigation hold satisfied per 전자문서법 | `legal-hold` | Must |
| FR-05 | Legal admin | export all communications for a user in a date range (PST/MBOX) | eDiscovery response completed in hours | `legal-hold` | Must |
| FR-06 | IT admin | configure retention policies per mailbox or channel (30d / 90d / 1y / forever) | data lifecycle compliance enforced automatically | `retention` | Must |
| FR-07 | Employee | receive Workflow approval notifications as inline action cards in inbox | approvals completed without leaving mail client | `workflow-handoff` | Should |
| FR-08 | Employee (Personal, post-M03) | use Connect Personal context with personal identity; employer cannot access | personal communication private from employer | `personal` | Deferred |

---

## Non-Functional Requirements

### Performance
- P99 mail send (SMTP ingest to stored): ≤2 s.
- P99 message send (messenger): ≤200 ms.
- P99 mailbox search (full-text, pgroonga): ≤500 ms for 1M messages.
- Legal-hold export 100k messages: ≤5 min.

### Security
- **Dual-context boundary** (ADR-0208/ADR-0215): Professional and Personal
  data physically isolated in separate Postgres schemas/tenants; no join
  possible at DB level; Cedar policy enforces at application layer.
- JWT `tenant_id` enforced for Professional context; `user_id` for Personal
  context (org-pillar vs person-pillar per ADR-0132).
- Mail content encrypted at rest (AES-256-GCM, KMS-wrapped key per tenant).
- Legal-hold: deletion blocked at storage adapter layer; Cedar policy grant
  required to remove hold.
- SMTP inbound: SPF/DKIM/DMARC enforced on inbound relay.

### Audit + Compliance
- Every mail send/receive event Ed25519-sealed per ADR-0028; seal latency ≤1 s.
- 전자문서법 (Korean Electronic Document Act): retention chain integrity provable.
- Jurisdiction overlay `KR` per ADR-0127; GDPR overlay for EU tenants (post-M03).

### Availability + SLO
- 99.9% monthly for mail delivery. 99.5% for messenger (degraded-graceful on cell failure).
- RTO ≤30 s; RPO ≤5 s. Mail delivery durability: zero message loss (outbox-first).

---

## Bounded Contexts

| BC name | Crate family (BNF v4.1) | Purpose | Key entities |
|---|---|---|---|
| `mail` | `oya-connect-mail-{domain,application,infrastructure,rest,grpc}` | Mail store; SMTP/IMAP; mailbox management | `Mailbox`, `Message`, `Thread` |
| `messenger` | `oya-connect-messenger-{domain,application,infrastructure,rest,grpc}` | Real-time channels + DMs; message store | `Channel`, `DirectMessage` |
| `legal-hold` | `oya-connect-legal-hold-{domain,application,infrastructure,rest}` | Hold management; eDiscovery export; retention lock | `LegalHold`, `RetentionPolicy` |
| `provisioning` | `oya-connect-provisioning-{domain,application}` | Account creation/revocation from HR Workflow events | `ConnectAccount` |
| `workflow-handoff` | `oya-connect-workflow-handoff-{domain,application}` | Workflow approval notifications; action cards | `ApprovalCard` |
| `personal` | `oya-connect-personal-{domain,application,infrastructure}` | Personal context (person-pillar; post-M03) | `PersonalConversation` |

### Clean Architecture Layer Map

Dependency direction: strictly inward-only. Per `feedback_clean_architecture_requirements.md`.

```
{mail-rest, messenger-grpc, legal-hold-rest}
        ↑ depends on
   {mail-adapter, messenger-adapter, legal-hold-adapter}  (implements kernel ports)
        ↑ depends on
   {mail-application, messenger-application, legal-hold-application,
    provisioning-application, workflow-handoff-application}
        ↑ depends on
   {mail-domain, messenger-domain, legal-hold-domain}
        ↑ depends on
   {mail-kernel, messenger-kernel, legal-hold-kernel}
        ↑
   connect-app  (composition root)
```

Port traits in kernel — ZERO business logic, ZERO I/O:

```rust
// oya-connect-mail-kernel/src/ports.rs

#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// Mail store port — implemented in oya-connect-mail-adapter (Postgres + OCI Object Storage)
#[async_trait::async_trait]
pub trait MailboxStore: Send + Sync + sealed::Sealed {
    async fn save_message(&self, tenant: &TenantId, msg: &Message) -> Result<MessageId, StoreError>;
    async fn load_thread(&self, tenant: &TenantId, thread_id: &ThreadId)
        -> Result<Vec<Message>, StoreError>;
    async fn search(&self, tenant: &TenantId, query: &MailSearchQuery)
        -> Result<Vec<MessageId>, StoreError>;
}

/// Legal hold store port — implemented in oya-connect-legal-hold-adapter
#[async_trait::async_trait]
pub trait LegalHoldStore: Send + Sync + sealed::Sealed {
    async fn place_hold(&self, tenant: &TenantId, hold: &LegalHold) -> Result<HoldId, StoreError>;
    async fn release_hold(&self, tenant: &TenantId, id: &HoldId) -> Result<(), StoreError>;
    async fn export(&self, tenant: &TenantId, id: &HoldId) -> Result<ExportStream, ExportError>;
}

/// Messenger store port — implemented in oya-connect-messenger-adapter
#[async_trait::async_trait]
pub trait MessengerStore: Send + Sync + sealed::Sealed {
    async fn send_message(&self, tenant: &TenantId, msg: &DirectMessage) -> Result<MessageId, StoreError>;
    async fn load_channel(&self, tenant: &TenantId, channel_id: &ChannelId)
        -> Result<Vec<DirectMessage>, StoreError>;
}
```

Cross-product integration: Connect NEVER imports `oya-hr-*`, `oya-payroll-*`,
or `oya-accounting-*` crates. All integration flows through:
- **Workflow events consumed**: `EmployeeHired`, `EmployeeTerminated`
  (provisioning-application subscribes via Workflow trigger adapter)
- **Ontology reads**: `Employee` Object Type (provisioning reads via
  `oya-ontology-entity-kernel::ObjectTypeStore` port — never direct DB)
- **Workflow events produced**: `ApprovalResponseSubmitted`, `LegalHoldPlaced`

`oya gate validate lean-a2 --ms connect` must exit 0.

```
NAME: oya-connect-mail-domain
JUSTIFICATION:
- microservice = connect: Connect µservice (dual-context communication); flat catalog; ADR-0056 v4.1
- bc-tokens = mail: connect has multiple BCs (mail/messenger/legal-hold/provisioning/workflow-handoff/personal); mail BC owns Mailbox + Message + Thread entities; ADR-0056 v4.1 BC-optionality
- layer = domain: Mailbox entity + Message entity + MailboxRepository port-trait; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none
```

---

## Integration via Workflow + Ontology

### Workflow events consumed

| Event type | Produced by | Handler BC | Action |
|---|---|---|---|
| `EmployeeHired` | `hr` | `provisioning` | Create ConnectAccount; assign corporate email |
| `EmployeeTerminated` | `hr` | `provisioning` | Suspend ConnectAccount; apply retention hold |
| `ApprovalRequested` | `workflow` (state machine) | `workflow-handoff` | Deliver approval action card to inbox |

### Workflow events produced

| Event type | Trigger | Consumed by | State machine |
|---|---|---|---|
| `ApprovalResponseSubmitted` | Employee submits action card | `workflow` | `approval-sm` |
| `LegalHoldPlaced` | IT/Legal admin places hold | `audit-chain` | `legal-hold-sm` |

### Ontology writes

| Object Type | Link Type | Written by BC | Audit trail |
|---|---|---|---|
| `ConnectAccount` | `OwnedBy` → `Employee` | `provisioning` | Ed25519 on create/revoke |
| `Message` | `InThread` → `Thread` | `mail`, `messenger` | Ed25519 on every message |
| `LegalHold` | `Covers` → `ConnectAccount` | `legal-hold` | Ed25519 on place/release |

### Ontology reads

| Object Type | Read by BC | Query shape |
|---|---|---|
| `Employee` | `provisioning` | `filter(tenant_id).where(status=active)` |

---

## Competitive Benchmark

| Competitor | Product | Parity dimensions | Primary source |
|---|---|---|---|
| Google Workspace | Gmail Enterprise + Chat | Mail delivery speed; search quality; legal hold depth; API surface | https://workspace.google.com |
| Microsoft 365 | Outlook + Teams | eDiscovery; retention labels; compliance center; action card integration | https://learn.microsoft.com/en-us/compliance |
| Slack | Slack Enterprise Grid | Channel UX; thread model; message search; workflow notifications | https://slack.com/intl/en-kr |
| Fastmail | Fastmail for Business | SMTP/IMAP compliance; deliverability; privacy model | https://www.fastmail.com |

Key parity gaps:
1. **Legal-hold + eDiscovery** (Microsoft compliance center parity): hold management UI, custodian-based holds, export in PST/MBOX with chain of custody — must reach Microsoft Compliance Center parity before M03 GA for enterprise sales.
2. **SMTP deliverability**: SPF/DKIM/DMARC + IP reputation warm-up plan required before corporate mail GA.
3. **Action cards** (Slack/Teams workflow notification parity): inline approve/reject cards in mail thread — must support JSON card schema renderable in Leptos web client.

---

## Performance Targets

| Metric | p50 | p99 | p999 | Notes |
|---|---|---|---|---|
| Mail send (SMTP ingest → stored) | 200 ms | 2 s | 5 s | Includes DKIM signing + outbox write |
| Messenger message send | 30 ms | 200 ms | 500 ms | Real-time; WebSocket push |
| Mailbox full-text search (1M msgs) | 100 ms | 500 ms | 1 s | pgroonga + Tantivy index |
| Legal-hold export 100k messages | — | 5 min | — | Streaming export |
| Audit chain seal | — | 1 s | — | Per (tenant_id, period); ADR-0028 |

Error budget: 0.1% monthly (mail); 0.5% (messenger). SLO burn-rate: 5× alarm.

---

## Horizontal Scalability

**State strategy**: `postgres` — mail store + messenger store in Postgres + Citus;
`tenant_id` partition key; message content in object-storage (OCI Object Storage)
with metadata in Postgres; legal-hold index in Postgres.

**Active-active compatibility**: `stateless-compatible` for messenger delivery
workers; `single-writer-compatible` for legal-hold state mutations.

| Dimension | Baseline per cell | Max per cell | Scale-out trigger |
|---|---|---|---|
| Max messages/day per cell | 1,000,000 | 100,000,000 | Ingest queue depth > 10k |
| Max concurrent WebSocket connections | 10,000 | 500,000 | Connection pool > 80% |
| Max mailboxes per cell | 50,000 | 5,000,000 | Storage > 80% |

Scale-out: messenger delivery workers HPA on queue depth; SMTP ingest workers
stateless; Postgres Citus horizontal sharding on `tenant_id`.
Cross-region: M03 KR only; cross-region replication required post-M03 (Connect-Pro
mail is high-consequence domain per `feedback_quality_performance_scalability_bar.md`).

---

## Acceptance Criteria

| AC-ID | Criterion | Verification |
|---|---|---|
| AC-01 | Mail send/receive round-trip; DKIM signature valid | integration test `test_mail_send_receive` |
| AC-02 | Dual-context boundary: Professional data unreachable from Personal context | `cargo nextest run --test dual_context_isolation` |
| AC-03 | Legal hold blocks deletion; eDiscovery export complete in ≤5 min | integration test `test_legal_hold_export_100k` |
| AC-04 | `EmployeeHired` → ConnectAccount provisioned; corporate email assigned | integration test `test_provisioning_workflow` |
| AC-05 | LEAN-A2: no direct imports from hr/payroll/accounting | `oya gate validate lean-a2 --ms connect` exits 0 |
| AC-06 | Messenger message p99 ≤200 ms at 5k concurrent users | k6 smoke; `http_req_duration{p(99)}<200` |
| AC-07 | Retention policy enforced; expired messages purged on schedule | `cargo nextest run -p oya-connect-legal-hold-domain` |

---

## Open Questions

| # | Question | Owner | Target |
|---|---|---|---|
| 1 | SMTP relay provider for M03 (self-hosted vs OCI Email Delivery)? | council-infrastructure | M03/P01 |
| 2 | Connect Personal crypto-audit scope and timeline? | council-security | M04 planning |
| 3 | Action card JSON schema: Adaptive Cards (Microsoft) or custom? | council-product | M03/P02 |

---

## Related ADRs

| ADR | Title | Relation |
|---|---|---|
| Bominal ADR-0208 | Connect dual-context (Personal + Professional) | inherited |
| Bominal ADR-0215 | Connect retention / legal hold / dual-context boundary | inherited |
| Bominal ADR-0210 | M3 KR group mail launch | inherited — M03 scope |
| Bominal ADR-0132 | Data ownership pillars | inherited — person-pillar for Personal |
| Bominal ADR-0123 | Cross-product auth cookie + redirect contract | inherited |
| ADR-0056 | BNF v4.1 | naming authority |
| ADR-0103 | Workflow hexagonal | integration plane |
| ADR-0106 | Ontology architecture | information plane |
