---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-first-paying-tenant
phase: P05-connect-pro-messenger
impl_plan_id: IP-P05-connect-pro-messenger-full-scaffold
status: pending
owner: council-connect
blocked_by:
- impl_plan: IP-P04-connect-pro-mail-full-scaffold
  reason: oya-connect-app binary + dual-context Cedar policies + oya-connect-mail-kernel
    must exist before messenger BC can be wired into the composition root.
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
- audit-chain
- k6-smoke
purpose: "Extends `oya-connect-app` with the `messenger` BC: Rust kernel port traits (`MessengerStore`, `RatchetKeyStore`); domain entities (`Channel`, `DirectMessage`, `RatchetSession`, `ObjectReference` deep-links)."
---
# IP-P05-connect-pro-messenger-full-scaffold: Connect Professional Messenger — PQXDH, Signal double-ratchet, InternalAuditable threads, WebSocket fan-out, Workflow deep-links

## Intent

Extends `oya-connect-app` with the `messenger` BC: Rust kernel port traits (`MessengerStore`, `RatchetKeyStore`); domain entities (`Channel`, `DirectMessage`, `RatchetSession`, `ObjectReference` deep-links); PQXDH key exchange + Signal double-ratchet (ported from Bominal `platform/libs/ratchet/`); `InternalAuditable` thread mode enforced (Professional context; stored encrypted under tenant DEK); WebSocket real-time fan-out via `WebSocketPushAdapter`; Workflow / HR / Payroll entity deep-links resolved via `ObjectStore` port; adapter implementations; load tests.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-connect-messenger-kernel/Cargo.toml` | create | deps: `async-trait`, `serde`, `uuid` |
| `crates/oya-connect-messenger-kernel/src/types.rs` | create | `ChannelId(Uuid)`, `DirectMessageId(Uuid)`, `RatchetSessionId(Uuid)` |
| `crates/oya-connect-messenger-kernel/src/ports.rs` | create | `MessengerStore`, `RatchetKeyStore` sealed port traits |
| `crates/oya-connect-messenger-domain/Cargo.toml` | create | deps: messenger-kernel + `oya-kms-kernel` |
| `crates/oya-connect-messenger-domain/src/channel.rs` | create | `Channel` aggregate — Professional InternalAuditable mode |
| `crates/oya-connect-messenger-domain/src/direct_message.rs` | create | `DirectMessage` entity — body_ciphertext under tenant DEK |
| `crates/oya-connect-messenger-domain/src/ratchet_session.rs` | create | `RatchetSession` — PQXDH initial key exchange state machine + Signal double-ratchet state |
| `crates/oya-connect-messenger-domain/src/deep_link.rs` | create | `ObjectReference` — typed Ontology Object reference in message payload |
| `crates/oya-connect-messenger-application/Cargo.toml` | create | deps: messenger kernel/domain + `oya-ontology-entity-kernel` |
| `crates/oya-connect-messenger-application/src/send_message.rs` | create | `SendMessageUseCase` — validates InternalAuditable mode; encrypts under tenant DEK; stores via `MessengerStore` |
| `crates/oya-connect-messenger-application/src/create_channel.rs` | create | `CreateChannelUseCase` — PQXDH handshake coordinator |
| `crates/oya-connect-messenger-application/src/deep_link_resolver.rs` | create | Resolves `ObjectReference` via `oya-ontology-entity-kernel::ObjectStore` port |
| `crates/oya-connect-messenger-adapter/Cargo.toml` | create | deps: messenger kernel/application + `sqlx` + `valkey` (formerly Redis) |
| `crates/oya-connect-messenger-adapter/src/postgres_messenger_store.rs` | create | `PostgresMessengerStore` implements `MessengerStore` |
| `crates/oya-connect-messenger-adapter/src/valkey_ratchet_key_store.rs` | create | `ValKeyRatchetKeyStore` implements `RatchetKeyStore` (ratchet state cached in Valkey; persisted to Postgres) |
| `crates/oya-connect-messenger-adapter/src/websocket_push_adapter.rs` | create | `WebSocketPushAdapter` — real-time fan-out via Axum WebSocket upgrade |
| `crates/oya-connect-messenger-rest/Cargo.toml` | create | deps: messenger application + `axum` |
| `crates/oya-connect-messenger-rest/src/routes.rs` | create | `/v1/channels`, `/v1/dms` CRUD + WebSocket upgrade handler |
| `crates/oya-connect-messenger-grpc/Cargo.toml` | create | deps: messenger application + `tonic` |
| `crates/oya-connect-messenger-grpc/src/messenger_service.rs` | create | gRPC service for internal messenger delivery bus |
| `migrations/connect/002_messenger_schema.sql` | create | Messenger DDL (see below) |
| `contracts/connect.openapi.yaml` | update | Add `/v1/channels`, `/v1/dms` endpoints |
| `proto/connect/messenger.proto` | create | gRPC service definition for messenger delivery |
| `tests/load/smoke-connect-messenger-ws.js` | create | k6: p99 ≤200ms at 5k concurrent WebSocket sessions |
| `Cargo.toml` | update | Add all `oya-connect-messenger-*` crates |
| `docs/standards/bounded-contexts.md` | update | Register messenger BC |

---

## Code Shape

### `crates/oya-connect-messenger-kernel/src/ports.rs`

```rust
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

/// Messenger store port — implemented in oya-connect-messenger-adapter
#[async_trait::async_trait]
pub trait MessengerStore: Send + Sync + sealed::Sealed {
    async fn send_message(
        &self, tenant: &TenantId, msg: &DirectMessage
    ) -> Result<DirectMessageId, ConnectError>;

    async fn load_channel(
        &self, tenant: &TenantId, channel_id: &ChannelId
    ) -> Result<Vec<DirectMessage>, ConnectError>;

    async fn create_channel(
        &self, tenant: &TenantId, channel: &Channel
    ) -> Result<ChannelId, ConnectError>;
}

/// Ratchet key store port — implemented in oya-connect-messenger-adapter (Valkey + Postgres)
#[async_trait::async_trait]
pub trait RatchetKeyStore: Send + Sync + sealed::Sealed {
    async fn store_session(
        &self, session_id: &RatchetSessionId, state: &RatchetSessionState
    ) -> Result<(), ConnectError>;

    async fn load_session(
        &self, session_id: &RatchetSessionId
    ) -> Result<Option<RatchetSessionState>, ConnectError>;

    async fn advance_ratchet(
        &self, session_id: &RatchetSessionId, message_key: &MessageKey
    ) -> Result<RatchetSessionState, ConnectError>;
}
```

### `crates/oya-connect-messenger-domain/src/ratchet_session.rs`

```rust
/// PQXDH + Signal double-ratchet session state
/// Ported from Bominal platform/libs/ratchet/
/// Forward secrecy: each message uses a derived message key; compromise of one key
/// does not expose past messages
///
/// Professional (InternalAuditable) mode:
/// - body stored encrypted under tenant DEK (AES-256-GCM, KMS-wrapped)
/// - ratchet ensures per-message forward secrecy within the session
/// - decryptable via four-eyes audit (ADR-0208 §5)
pub struct RatchetSession {
    pub id: RatchetSessionId,
    pub context_kind: ContextKind,  // must be Professional at M03
    pub alice_public_key: PqxdhPublicKey,
    pub bob_public_key: PqxdhPublicKey,
    pub root_key: RootKey,
    pub send_chain_key: ChainKey,
    pub recv_chain_key: ChainKey,
    pub message_counter: u32,
}

impl RatchetSession {
    /// PQXDH initial handshake (X3DH + Kyber post-quantum extension)
    pub fn initiate(
        alice_identity: &IdentityKeyPair,
        bob_prekey_bundle: &PrekeyBundle,
        context_kind: ContextKind,
    ) -> Result<(Self, InitialMessage), ConnectError> {
        // PQXDH: X25519 DH + Kyber512 KEM combined shared secret
        // Professional mode: shared secret used to encrypt per-session DEK
        // Per-session DEK is then wrapped under tenant KMS DEK
    }

    /// Advance ratchet; derive next message key
    pub fn ratchet_send(&mut self) -> Result<MessageKey, ConnectError> {
        // Signal double-ratchet send step
        // Returns MessageKey for encrypting next message body
    }

    pub fn ratchet_receive(
        &mut self, header: &MessageHeader
    ) -> Result<MessageKey, ConnectError> {
        // Signal double-ratchet receive step
    }
}
```

### `crates/oya-connect-messenger-domain/src/deep_link.rs`

```rust
/// Typed Ontology Object reference in message payload
/// Allows Professional messages to deep-link to Workflow runs, HR entities, Payroll entries
/// Resolved via oya-ontology-entity-kernel::ObjectStore port (no direct imports)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjectReference {
    /// Object Type (e.g., "workflow.WorkflowRun", "hr.Employee", "payroll.PayrollEntry")
    pub object_type: String,
    pub object_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    /// Display hint for UI (resolved at render time via Ontology Function)
    pub display_label: Option<String>,
}

impl ObjectReference {
    pub fn validate_type(&self) -> Result<(), ConnectError> {
        // Allow only registered Ontology Object Types
        // Prevents arbitrary object_type injection
        const ALLOWED_TYPES: &[&str] = &[
            "workflow.WorkflowRun",
            "workflow.ApprovalRequest",
            "hr.Employee",
            "hr.Employment",
            "payroll.PayrollEntry",
            "payroll.PayrollRun",
            "accounting.JournalEntry",
        ];
        if !ALLOWED_TYPES.contains(&self.object_type.as_str()) {
            return Err(ConnectError::UnknownObjectType(self.object_type.clone()));
        }
        Ok(())
    }
}
```

---

## Postgres DDL

### migrations/connect/002_messenger_schema.sql

```sql
-- Messenger extension to connect_pro schema

-- Channels (Professional InternalAuditable mode)
CREATE TABLE connect_pro.channels (
    channel_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    name            text NOT NULL,
    context_kind    connect_pro.context_kind NOT NULL DEFAULT 'professional',
    ownership_pillar connect_pro.ownership_pillar NOT NULL DEFAULT 'org',
    -- ownership_pillar IMMUTABLE
    kind            text NOT NULL DEFAULT 'messaging'
                    CHECK (kind IN ('messaging','broadcast','discussion')),
    access          text NOT NULL DEFAULT 'restricted'
                    CHECK (access IN ('open','restricted','private')),
    space_id        uuid NULL,   -- top-level workspace/space
    created_by      uuid NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.channels ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.channels
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_channel_tenant ON connect_pro.channels (tenant_id);

-- Direct messages (InternalAuditable: body encrypted under tenant DEK)
-- forward secrecy via per-message ratchet key; body_object_key = OCI Object Storage
CREATE TABLE connect_pro.direct_messages (
    dm_id           uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    channel_id      uuid NULL REFERENCES connect_pro.channels(channel_id),
    conversation_id uuid NULL,   -- DM conversation (1:1 or small group)
    sender_id       uuid NOT NULL,
    context_kind    connect_pro.context_kind NOT NULL DEFAULT 'professional',
    ownership_pillar connect_pro.ownership_pillar NOT NULL DEFAULT 'org',
    -- ownership_pillar IMMUTABLE
    body_object_key text NOT NULL,  -- OCI Object Storage; AES-256-GCM under tenant DEK
    ratchet_session_id uuid NULL,   -- linked RatchetSession for forward secrecy
    message_counter int NOT NULL DEFAULT 0,
    -- Deep-links to Ontology Object Types
    object_references jsonb NULL,   -- Vec<ObjectReference> JSON
    legal_hold_count  int NOT NULL DEFAULT 0,
    sent_at         timestamptz NOT NULL DEFAULT now(),
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.direct_messages ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.direct_messages
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_dm_channel ON connect_pro.direct_messages (tenant_id, channel_id, sent_at DESC)
    WHERE channel_id IS NOT NULL;
CREATE INDEX idx_dm_conversation ON connect_pro.direct_messages (tenant_id, conversation_id, sent_at DESC)
    WHERE conversation_id IS NOT NULL;

-- Ratchet sessions (PQXDH state; persisted for audit; hot state in Valkey)
CREATE TABLE connect_pro.ratchet_sessions (
    session_id      uuid PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid NOT NULL,
    participant_a   uuid NOT NULL,
    participant_b   uuid NOT NULL,
    context_kind    connect_pro.context_kind NOT NULL DEFAULT 'professional',
    state_encrypted bytea NOT NULL,   -- session state encrypted under tenant DEK
    message_counter int NOT NULL DEFAULT 0,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE connect_pro.ratchet_sessions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON connect_pro.ratchet_sessions
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
```

---

## Acceptance Gates

```bash
cargo check -p oya-connect-app --all-features  # exit 0 (incremental; messenger BC wired)
cargo nextest run -p oya-connect-messenger-domain --test pqxdh_handshake  # exit 0
cargo nextest run -p oya-connect-messenger-domain --test ratchet_forward_secrecy  # exit 0
cargo nextest run -p oya-connect-messenger-domain --test internal_auditable_mode  # exit 0
cargo nextest run -p oya-connect-messenger-domain --test deep_link_ontology_ref   # exit 0
oya gate validate lean-a2 --ms connect  # LEAN-A2 still passing
oya gate validate audit-chain --ms connect
k6 run tests/load/smoke-connect-messenger-ws.js  # p(99)<200 at 5k WS connections
```

---

## Test Plan

| Test | Verifies |
|---|---|
| `pqxdh_handshake` | PQXDH (X25519 + Kyber512) key agreement produces matching shared secret on both sides |
| `ratchet_forward_secrecy` | Compromise of message N key does not expose message N-1 or N+1 |
| `internal_auditable_mode` | Professional channel message stored encrypted under tenant DEK; decryptable via four-eyes audit |
| `deep_link_ontology_ref` | `ObjectReference` with `object_type="workflow.WorkflowRun"` validates correctly; unknown type rejected |
| `cross_product_refusal` | Messenger application never imports `oya-hr-*` or `oya-workflow-*` directly; Ontology port used |
| `websocket_delivery` | Message sent → WebSocket push received by recipient within 200ms |

### Load test

```javascript
// tests/load/smoke-connect-messenger-ws.js
import { WebSocket } from 'k6/experimental/websockets';

export const options = {
  vus: 5000,
  duration: '60s',
  thresholds: {
    ws_session_duration: ['p(99)<200'],
    ws_msgs_received: ['rate>0'],
    http_req_failed: ['rate<0.001'],
  },
};
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent ip-p05-connect-messenger \
  --intent "P05-connect-pro-messenger: PQXDH + Signal ratchet, InternalAuditable threads, WebSocket fan-out, Workflow deep-links" \
  --ttl 3600 \
  crates/oya-connect-messenger-kernel/src/ports.rs::MessengerStore \
  crates/oya-connect-messenger-kernel/src/ports.rs::RatchetKeyStore \
  crates/oya-connect-messenger-domain/src/ratchet_session.rs::RatchetSession \
  crates/oya-connect-messenger-domain/src/deep_link.rs::ObjectReference \
  migrations/connect/002_messenger_schema.sql::connect_pro.direct_messages \
  contracts/connect.openapi.yaml::sendDirectMessage
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P05-connect-pro-messenger-full-scaffold merged; Professional Messenger shipped: PQXDH key exchange + Signal double-ratchet (InternalAuditable mode); WebSocket fan-out; Workflow/HR/Payroll deep-links via Ontology ObjectReference; tenant DEK storage; LEAN-A2 still clean; next: IP-P06-application-b2b-live" \
  -i high \
  -k "M03,P05,IP-P05-connect-pro-messenger,connect,messenger,pqxdh,ratchet"
```

---

## Halt Conditions

1. PQXDH (Kyber512) implementation fails test vectors — do not use a custom implementation; port from Bominal `platform/libs/ratchet/` exactly; escalate if Bominal ratchet library is not available.
2. `ratchet_forward_secrecy` fails — Signal ratchet implementation bug; do not patch test; debug ratchet advance logic.
3. WebSocket fan-out under 5k concurrent connections causes memory exhaustion — investigate `WebSocketPushAdapter` connection pool sizing; escalate if Axum WS handler limit needs architectural change.

---

## Next IP Pointer

`phases/P06-application-b2b-live/impl-plan.md`

---

## Cross-References

- PRD: `docs/prds/connect.md`
- Bominal ADR-0208 (dual-context), ADR-0111 (tenant DEK), ADR-0047 (Workflow deep-links), ADR-0028 (audit chain)
- ADR-0056 (BNF v4.1)
