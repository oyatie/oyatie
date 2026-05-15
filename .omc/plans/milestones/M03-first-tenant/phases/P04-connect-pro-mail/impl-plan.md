---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P04-connect-pro-mail
impl_plan_id: IP-P04-connect-pro-mail-full-scaffold
status: pending
owner: council-connect
blocked_by:
- impl_plan: IP-P01-hr-full-scaffold
  reason: EmployeeHired Workflow event required for Connect provisioning-application
    to wire ConnectAccount creation.
acceptance_lanes:
- cargo-check
- cargo-build
- cargo-clippy
- cargo-nextest
- cargo-deny
- lean-a1
- lean-a2
- lean-a3
- lean-a4
- ontology-type-registry
- workflow-event-registry
- audit-chain
- cedar-policy
- k6-smoke
purpose: Auto-backfilled purpose for impl-plan.md
---
# IP-P04-connect-pro-mail-full-scaffold: Connect Professional Mail — SMTP/IMAP/JMAP, tenant DEK encryption, legal hold, eDiscovery, retention, dual-context Cedar policies

## Intent

Scaffolds the `oya-connect-*` µservice: Postgres DDL for mail + legal-hold BCs with Citus sharding + RLS + dual-context schema isolation (`connect_pro` / `connect_personal`); Rust kernel port traits (`MailboxStore`, `LegalHoldStore`); domain entities (`Mailbox`, `Message`, `Thread`, `LegalHold`, `RetentionPolicy`, `ConnectAccount`); adapter implementations (PostgresMailboxStore, OciObjectStorageMessageBody, SmtpIngestAdapter, PstExportAdapter); Cedar dual-context policy pack (ADR-0208/ADR-0215 forbid rules); Protobuf events; ConnectAccount provisioning from EmployeeHired Workflow event; load tests. Personal context scaffold is present but NOT GA.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-connect-mail-kernel/Cargo.toml` | create | deps: `async-trait`, `serde`, `uuid` |
| `crates/oya-connect-mail-kernel/src/types.rs` | create | `MailboxId(Uuid)`, `MessageId(Uuid)`, `ThreadId(Uuid)`, `ContextKind` enum |
| `crates/oya-connect-mail-kernel/src/ports.rs` | create | `MailboxStore`, `LegalHoldStore`, `MessengerStore` sealed port traits |
| `crates/oya-connect-mail-domain/Cargo.toml` | create | deps: mail-kernel + `oya-kms-kernel` (tenant DEK) |
| `crates/oya-connect-mail-domain/src/mailbox.rs` | create | `Mailbox` aggregate + `Message` entity + `Thread` entity |
| `crates/oya-connect-mail-domain/src/dual_context.rs` | create | `ContextKind::Professional | Personal`; `OwnershipPillar::Org | Person`; immutability invariant |
| `crates/oya-connect-legal-hold-kernel/Cargo.toml` | create | deps |
| `crates/oya-connect-legal-hold-kernel/src/types.rs` | create | `HoldId(Uuid)`, `RetentionPolicyId(Uuid)`, `HoldStatus` |
| `crates/oya-connect-legal-hold-kernel/src/ports.rs` | create | `LegalHoldStore`, `ExportPort` sealed port traits |
| `crates/oya-connect-legal-hold-domain/src/legal_hold.rs` | create | `LegalHold` aggregate — hold initiation, scope, release (four-eyes) |
| `crates/oya-connect-legal-hold-domain/src/retention_policy.rs` | create | `RetentionPolicy` — per (artefact_class, pillar, window_months) |
| `crates/oya-connect-legal-hold-domain/src/retention_expiry.rs` | create | Retention window expiry logic; `DeleteAuditLog` entry creation |
| `crates/oya-connect-provisioning-domain/src/connect_account.rs` | create | `ConnectAccount` aggregate; hire → provision, terminate → suspend |
| `crates/oya-connect-provisioning-application/src/provision_on_hire.rs` | create | `ProvisionOnHireUseCase` — subscribes to EmployeeHired Workflow event |
| `crates/oya-connect-workflow-handoff-domain/src/approval_card.rs` | create | `ApprovalCard` entity; action card delivery |
| `crates/oya-connect-personal-domain/src/personal_conversation.rs` | create | `PersonalConversation` stub — NOT GA; person-pillar boundary invariant declared |
| `crates/oya-connect-mail-adapter/src/postgres_mailbox_store.rs` | create | `PostgresMailboxStore` implements `MailboxStore` |
| `crates/oya-connect-mail-adapter/src/oci_object_storage_message_body.rs` | create | Message body → OCI Object Storage; metadata in Postgres |
| `crates/oya-connect-mail-adapter/src/smtp_ingest_adapter.rs` | create | SMTP ingest; SPF/DKIM/DMARC verification; store via `MailboxStore` port |
| `crates/oya-connect-mail-adapter/src/imap_adapter.rs` | create | IMAP server adapter |
| `crates/oya-connect-legal-hold-adapter/src/postgres_legal_hold_store.rs` | create | `PostgresLegalHoldStore` implements `LegalHoldStore` |
| `crates/oya-connect-legal-hold-adapter/src/pst_export_adapter.rs` | create | PST export format with chain of custody |
| `crates/oya-connect-legal-hold-adapter/src/mbox_export_adapter.rs` | create | MBOX export format |
| `crates/oya-connect-app/src/main.rs` | create | DI assembly; SMTP/IMAP servers + REST API + Workflow event consumer |
| `migrations/connect/001_connect_schema.sql` | create | Full DDL (see below) |
| `contracts/connect.openapi.yaml` | create | OpenAPI 3.1 for mail + legal-hold endpoints |
| `proto/connect/events.proto` | create | `ApprovalResponseSubmitted`, `LegalHoldPlaced` |
| `policies/connect/connect.cedar` | create | Dual-context Cedar policy pack |
| `tests/load/smoke-connect-mail-send.js` | create | k6: p99 ≤2s |
| `Cargo.toml` | update | Add all `oya-connect-*` crates |
| `docs/standards/bounded-contexts.md` | update | Register mail/legal-hold/provisioning/workflow-handoff/personal BCs |

---

## Code Shape

### `crates/oya-connect-mail-kernel/src/ports.rs`

```rust
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// Mail store port — implemented in oya-connect-mail-adapter (Postgres + OCI Object Storage)
#[async_trait::async_trait]
pub trait MailboxStore: Send + Sync + sealed::Sealed {
    async fn save_message(
        &self, tenant: &TenantId, msg: &Message
    ) -> Result<MessageId, ConnectError>;

    async fn load_thread(
        &self, tenant: &TenantId, thread_id: &ThreadId
    ) -> Result<Vec<Message>, ConnectError>;

    async fn search(
        &self, tenant: &TenantId, query: &MailSearchQuery
    ) -> Result<Vec<MessageId>, ConnectError>;
}

/// Legal hold store port — implemented in oya-connect-legal-hold-adapter
#[async_trait::async_trait]
pub trait LegalHoldStore: Send + Sync + sealed::Sealed {
    async fn place_hold(
        &self, tenant: &TenantId, hold: &LegalHold
    ) -> Result<HoldId, ConnectError>;

    async fn release_hold(
        &self, tenant: &TenantId, id: &HoldId, approver: &UserId
    ) -> Result<(), ConnectError>;

    async fn export(
        &self, tenant: &TenantId, id: &HoldId, format: ExportFormat
    ) -> Result<ExportStream, ExportError>;
}

/// Export format enum
#[derive(Debug, Clone, Copy)]
pub enum ExportFormat { Pst, Mbox }
```

### `crates/oya-connect-mail-domain/src/dual_context.rs`

```rust
/// Dual-context model — ADR-0208
/// context_kind and ownership_pillar are set at creation and NEVER changed
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "context_kind", rename_all = "snake_case")]
pub enum ContextKind {
    /// Professional (work); wire format: "work"; org-pillar
    Professional,
    /// Personal; wire format: "personal"; person-pillar (NOT GA at M03)
    Personal,
}

/// Data ownership pillar — ADR-0132
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "ownership_pillar", rename_all = "snake_case")]
pub enum OwnershipPillar { Org, Person }

impl ContextKind {
    /// Canonical pillar for a given context
    pub fn ownership_pillar(self) -> OwnershipPillar {
        match self {
            Self::Professional => OwnershipPillar::Org,
            Self::Personal => OwnershipPillar::Person,
        }
    }

    /// Wire format value (matches ADR-0208 PersonaType enum)
    pub fn as_wire_value(self) -> &'static str {
        match self { Self::Professional => "work", Self::Personal => "personal" }
    }
}
```

### `crates/oya-connect-legal-hold-domain/src/legal_hold.rs`

```rust
/// Legal hold aggregate — ADR-0215 Contract 1
/// Org-initiated; covers org-pillar artefacts only
/// Four-eyes approval required for release
pub struct LegalHold {
    pub id: HoldId,
    pub tenant_id: TenantId,
    pub initiated_by: UserId,
    pub scope: HoldScope,
    pub purpose_reason: String,
    pub status: HoldStatus,
    pub initiated_at: DateTime<Utc>,
    pub released_at: Option<DateTime<Utc>>,
    pub released_by: Option<UserId>,
    pub release_approved_by: Option<UserId>,  // four-eyes approver
}

pub enum HoldScope {
    User(UserId),
    Channel(ChannelId),
    DateRange { start: DateTime<Utc>, end: DateTime<Utc> },
    All,
}

pub enum HoldStatus { Active, Released }

impl LegalHold {
    pub fn initiate(
        tenant_id: TenantId,
        initiated_by: UserId,
        scope: HoldScope,
        purpose_reason: String,
    ) -> Result<Self, ConnectError> {
        // Validates purpose_reason non-empty
        // Creates LegalHold with status = Active
    }

    /// Four-eyes release: requires different admin than initiator
    pub fn release(
        &mut self,
        released_by: UserId,
        approved_by: UserId,
    ) -> Result<(), ConnectError> {
        if released_by == self.initiated_by {
            return Err(ConnectError::FourEyesViolation);
        }
        if released_by == approved_by {
            return Err(ConnectError::FourEyesViolation);
        }
        self.status = HoldStatus::Released;
        self.released_by = Some(released_by);
        self.release_approved_by = Some(approved_by);
        self.released_at = Some(Utc::now());
        Ok(())
    }
}
```

---

## Postgres DDL

### migrations/connect/001_connect_schema.sql

```sql
-- Connect µservice — dual-context schema isolation
-- Professional context: schema connect_pro (org-pillar; tenant_id RLS)
-- Personal context: schema connect_personal (person-pillar; NOT GA at M03)
-- ADR-0208 (dual-context), ADR-0215 (retention/legal hold)

CREATE SCHEMA IF NOT EXISTS connect_pro;
CREATE SCHEMA IF NOT EXISTS connect_personal;

CREATE TYPE connect_pro.context_kind AS ENUM ('professional', 'personal');
CREATE TYPE connect_pro.ownership_pillar AS ENUM ('org', 'person');
CREATE TYPE connect_pro.hold_status AS ENUM ('active', 'released');
CREATE TYPE connect_pro.artefact_class AS ENUM ('mail', 'messenger', 'calendar', 'action_card');

-- Connect accounts (provisioned from EmployeeHired Workflow event)
CREATE TABLE connect_pro.connect_accounts (
    account_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    employee_id     uuid NOT NULL,  -- FK to hr.employees (cross-schema; enforced in application)
    email_address   text NOT NULL,  -- corporate domain email
    status          text NOT NULL DEFAULT 'active'
                    CHECK (status IN ('active','suspended','deleted')),
    suspended_at    timestamptz NULL,
    suspension_reason text NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.connect_accounts ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.connect_accounts
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_account_email ON connect_pro.connect_accounts (tenant_id, email_address);
CREATE INDEX idx_account_employee ON connect_pro.connect_accounts (tenant_id, employee_id);
-- SELECT create_distributed_table('connect_pro.connect_accounts', 'tenant_id');

-- Mailboxes
CREATE TABLE connect_pro.mailboxes (
    mailbox_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    account_id      uuid NOT NULL REFERENCES connect_pro.connect_accounts(account_id),
    context_kind    connect_pro.context_kind NOT NULL DEFAULT 'professional',
    ownership_pillar connect_pro.ownership_pillar NOT NULL DEFAULT 'org',
    -- ownership_pillar is IMMUTABLE (ADR-0215 Contract 2)
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.mailboxes ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.mailboxes
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Messages (metadata; body in OCI Object Storage encrypted under tenant DEK)
-- ADR-0208 §5: body_ciphertext_key = OCI Object Storage key for AES-256-GCM encrypted body
CREATE TABLE connect_pro.messages (
    message_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    mailbox_id      uuid NOT NULL REFERENCES connect_pro.mailboxes(mailbox_id),
    thread_id       uuid NULL,
    context_kind    connect_pro.context_kind NOT NULL DEFAULT 'professional',
    ownership_pillar connect_pro.ownership_pillar NOT NULL DEFAULT 'org',
    -- ownership_pillar IMMUTABLE
    from_address    text NOT NULL,
    to_addresses    text[] NOT NULL,
    subject_encrypted text NOT NULL,  -- encrypted under tenant DEK (KMS)
    body_object_key text NOT NULL,    -- OCI Object Storage key for AES-256-GCM body
    dkim_signature  text NULL,
    legal_hold_count int NOT NULL DEFAULT 0,  -- >0 = deletion blocked
    deleted_at      timestamptz NULL,
    deletion_reason text NULL,
    received_at     timestamptz NOT NULL DEFAULT now(),
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.messages
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
-- Prevent deletion of messages under legal hold
CREATE OR REPLACE FUNCTION connect_pro.prevent_held_message_deletion()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    IF OLD.legal_hold_count > 0 AND NEW.deleted_at IS NOT NULL THEN
        RAISE EXCEPTION 'Cannot delete message % under legal hold', OLD.message_id;
    END IF;
    RETURN NEW;
END;
$$;
CREATE TRIGGER prevent_held_message_deletion_trigger
    BEFORE UPDATE ON connect_pro.messages
    FOR EACH ROW EXECUTE FUNCTION connect_pro.prevent_held_message_deletion();

-- Legal holds (ADR-0215)
CREATE TABLE connect_pro.legal_holds (
    hold_id         uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    initiated_by    uuid NOT NULL,
    scope_type      text NOT NULL CHECK (scope_type IN ('user','channel','date_range','all')),
    scope_user_id   uuid NULL,
    scope_channel_id uuid NULL,
    scope_start     timestamptz NULL,
    scope_end       timestamptz NULL,
    purpose_reason  text NOT NULL,
    status          connect_pro.hold_status NOT NULL DEFAULT 'active',
    initiated_at    timestamptz NOT NULL DEFAULT now(),
    released_at     timestamptz NULL,
    released_by     uuid NULL,
    release_approved_by uuid NULL  -- four-eyes approver
);
ALTER TABLE connect_pro.legal_holds ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.legal_holds
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Retention policies (per artefact class and pillar)
CREATE TABLE connect_pro.retention_policies (
    policy_id       uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    ownership_pillar connect_pro.ownership_pillar NOT NULL,
    artefact_class  connect_pro.artefact_class NOT NULL,
    retention_window_months int NOT NULL
                    CHECK (retention_window_months BETWEEN 3 AND 84),  -- 3 months to 7 years
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.retention_policies ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.retention_policies
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_retention_policy ON connect_pro.retention_policies
    (tenant_id, ownership_pillar, artefact_class);

-- Message audit log (append-only; ADR-0208 §5 audit decryption)
CREATE TABLE connect_pro.message_audit_log (
    log_id          bigserial PRIMARY KEY,
    tenant_id       uuid NOT NULL,
    message_id      uuid NOT NULL,
    accessed_by     uuid NOT NULL,
    decrypted_context text NOT NULL CHECK (decrypted_context IN ('legal_hold','audit_discovery','debug')),
    reason          text NOT NULL,
    authorized_by   uuid NOT NULL,
    decrypted_at    timestamptz NOT NULL DEFAULT now()
);
-- Append-only; no UPDATE or DELETE permitted
ALTER TABLE connect_pro.message_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.message_audit_log
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Outbox
CREATE TABLE connect_pro.outbox (
    outbox_id   bigserial PRIMARY KEY,
    tenant_id   uuid NOT NULL,
    topic       text NOT NULL,
    key         text NOT NULL,
    payload     jsonb NOT NULL,
    published_at timestamptz NULL,
    created_at  timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_connect_outbox_unpublished ON connect_pro.outbox (created_at)
    WHERE published_at IS NULL;
```

---

## Cedar Policy Pack (Dual-Context)

```cedar
// connect/connect.cedar — ADR-0208 + ADR-0215

entity Tenant;
entity Organization in [Tenant];
entity Employee in [Organization];
entity Admin in [Tenant] = { ownership_pillar: String };
entity EmployeeUser in [Tenant] = { employee_id: String, ownership_pillar: String };

// --- Contract 2: Dual-context boundary at entity level (ADR-0215) ---

// Reject cross-pillar queries
forbid (
    principal,
    action in [
        Action::"Read", Action::"Update", Action::"Delete",
        Action::"ExportForEDiscovery", Action::"InitiateLegalHold"
    ],
    resource
) when {
    principal.ownership_pillar != resource.ownership_pillar
};

// Org-admin forbidden from person-pillar (403)
forbid (
    principal is Admin,
    action in [
        Action::"Read", Action::"Update", Action::"Delete",
        Action::"ExportForEDiscovery", Action::"InitiateLegalHold"
    ],
    resource
) when {
    resource.ownership_pillar == "person"
};

// Personal context never flows to Workflow (ADR-0215 Contract 4)
forbid (
    principal,
    action == Action::"HandoffToWorkflow",
    resource
) when {
    resource.context_kind == "personal"
};

// --- Legal hold permissions ---
permit (
    principal is Admin,
    action in [Action::"InitiateLegalHold", Action::"ReleaseLegalHold", Action::"ExportForEDiscovery"],
    resource
) when {
    context.tenant_id == principal.tenant_id &&
    resource.ownership_pillar == "org"   // only org-pillar artefacts
};

// Employee can read own mail only
permit (
    principal is EmployeeUser,
    action == Action::"ReadMail",
    resource
) when {
    resource.mailbox_account_id == principal.employee_id
};
```

---

## Acceptance Gates

```bash
cargo check -p oya-connect-app --all-features  # exit 0
cargo nextest run --test test_mail_send_receive  # exit 0; DKIM valid
cargo nextest run --test dual_context_isolation  # exit 0; Professional unreachable from Personal
cargo nextest run --test test_legal_hold_export_100k  # exit 0; ≤5 min
cargo nextest run --test test_provisioning_workflow  # exit 0; EmployeeHired → ConnectAccount
oya gate validate lean-a2 --ms connect  # no imports from hr/payroll/accounting
oya gate validate cedar-policy --ms connect  # dual-context forbid rules pass
oya gate validate audit-chain --ms connect
k6 run tests/load/smoke-connect-mail-send.js  # p(99)<2000
```

---

## Test Plan

| Test | Verifies |
|---|---|
| `test_mail_send_receive` | SMTP ingest → stored under tenant DEK → IMAP retrieve; DKIM signature valid |
| `dual_context_isolation` | Professional data unreachable from Personal context; 403 on cross-pillar query |
| `test_legal_hold_export_100k` | Legal hold placed; 100k messages under hold; PST export complete ≤5 min |
| `test_held_message_deletion_blocked` | Message under legal hold cannot be deleted (DB trigger fires) |
| `test_retention_expiry` | Message older than retention window auto-deleted; `DeletionAuditLog` entry written |
| `test_provisioning_workflow` | `EmployeeHired` event → `ConnectAccount` created; corporate email assigned |
| `test_four_eyes_release` | Hold initiator cannot release own hold; different admin required |
| `test_message_audit_log_append_only` | Audit log entry cannot be updated or deleted |

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent ip-p04-connect-mail \
  --intent "P04-connect-pro-mail: MailboxStore/LegalHoldStore ports, dual-context Cedar policies (ADR-0208/0215), legal hold four-eyes, eDiscovery export, ConnectAccount provisioning from EmployeeHired" \
  --ttl 3600 \
  crates/oya-connect-mail-kernel/src/ports.rs::MailboxStore \
  crates/oya-connect-mail-kernel/src/ports.rs::LegalHoldStore \
  crates/oya-connect-mail-domain/src/dual_context.rs::ContextKind \
  crates/oya-connect-legal-hold-domain/src/legal_hold.rs::LegalHold \
  crates/oya-connect-legal-hold-domain/src/retention_policy.rs::RetentionPolicy \
  crates/oya-connect-provisioning-domain/src/connect_account.rs::ConnectAccount \
  migrations/connect/001_connect_schema.sql::connect_pro.messages \
  policies/connect/connect.cedar::FourEyesRelease
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P04-connect-pro-mail-full-scaffold merged; Connect Professional Mail shipped: SMTP/IMAP, tenant DEK AES-256-GCM (ADR-0111), legal hold + eDiscovery (ADR-0215 Contracts 1-4), dual-context Cedar policies (ADR-0208), ConnectAccount provisioning from EmployeeHired; Personal context scaffolded (NOT GA); next: IP-P05-connect-pro-messenger" \
  -i high \
  -k "M03,P04,IP-P04-connect-pro-mail,connect,mail,legal-hold,dual-context"
```

---

## Halt Conditions

1. `dual_context_isolation` test fails because Cedar policy is not enforced at the DB layer — check that `ownership_pillar` is indexed and Cedar policy evaluation happens before query construction in the application layer.
2. `test_legal_hold_export_100k` exceeds 5 min — investigate streaming export; the export must be chunked via `ExportPort::export() -> ExportStream`, not buffered in memory.
3. LEAN-A2 violation — connect imports `oya-hr-*` — fix: `ProvisionOnHireUseCase` must subscribe to `EmployeeHired` via Workflow event consumer and read Employee via `oya-ontology-entity-kernel::ObjectStore` port only.

---

## Next IP Pointer

`phases/P05-connect-pro-messenger/impl-plan.md`

---

## Cross-References

- PRD: `docs/prds/connect.md`
- Bominal ADR-0208 (dual-context), ADR-0215 (retention/legal hold), ADR-0210 (M3 mail launch), ADR-0132 (pillars), ADR-0111 (tenant DEK), ADR-0028 (audit chain)
- ADR-0056 (BNF v4.1)
