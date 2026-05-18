# IP-007: revocation-kernel + revocation-worker — propagation pipeline scaffolding

- Bounded context: revocation
- Layers: kernel, domain, usecase, api, sdk, worker
- Crates:
  - `oya-consent-graph-revocation-kernel`
  - `oya-consent-graph-revocation-domain`
  - `oya-consent-graph-revocation-usecase`
  - `oya-consent-graph-revocation-api`
  - `oya-consent-graph-revocation-sdk`
  - `oya-consent-graph-revocation-worker`
- Acceptance status: ga
- Authority: ADR-0214 §2.4 (real-time revocation ≤1s), ADR-0105, ADR-0078 (Pulsar substrate).
- Depends on: `oya-consent-graph-agreement-{kernel, sdk}`, `oya-consent-graph-audit-bridge-sdk`,
  `oya-consent-graph-enforcement-sdk` (for cache invalidation).

## 1. Goal

Define the substrate for *originating* a revocation event and *propagating* it to every projection
subscriber within p99 ≤1s. This IP covers everything except the Pulsar adapter itself (IP-008).

## 2. Scope

In:
- `RevocationEvent` and `RevocationReceipt` types (kernel).
- `RevocationStateMachine` (pure transitions).
- `RevocationOriginator` and `RevocationSubscriber` ports.
- `revocation-usecase` orchestration (originate, fan-out, confirm).
- `revocation-worker` background loop that drains the outbox to Pulsar (IP-008 implements the Pulsar
  side; this IP defines the worker shell).

Out:
- Pulsar adapter (→ IP-008).
- Cache invalidation in `enforcement-adapter` (→ IP-006).
- Projection topic destruction (→ IP-009/010).

## 3. Types

```rust
pub struct RevocationEvent {
    pub revocation_id: Ulid,
    pub agreement_id: AgreementId,
    pub grantor: TenantId,
    pub grantee: TenantId,
    pub reason: RevocationReason,
    pub initiated_by: ActorRef,
    pub initiated_at: Timestamp,
    pub deadline_at: Timestamp,   // initiated_at + 1s SLO
}

pub struct RevocationReceipt {
    pub revocation_id: Ulid,
    pub subscriber: SubscriberRef,         // {service, region}
    pub propagated_at: Timestamp,
    pub propagation_latency_ms: u64,
    pub action_taken: PropagationAction,   // CacheInvalidated | ProjectionPaused | TopicDestroyed | RowTombstoned
}

pub enum PropagationAction {
    CacheInvalidated,
    ProjectionPaused,
    TopicDestroyed,
    RowTombstoned,
    NoOp,        // subscriber didn't hold any state for this agreement
}
```

## 4. RevocationStateMachine

```rust
pub enum RevocationLifecycle {
    Originated,            // event committed to outbox
    Published,             // pushed to Pulsar topic
    Propagating { receipts_received: u32, expected_subscribers: u32 },
    Confirmed,             // all expected subscribers receipted within deadline
    PartiallyPropagated,   // deadline passed, <100% receipts
    Failed { reason: String },
}
```

Transition table:
```
∅ → Originated            (originate)
Originated → Published    (worker drains to Pulsar)
Published → Propagating   (first receipt received)
Propagating → Confirmed   (all expected receipts received within deadline)
Propagating → PartiallyPropagated  (deadline elapsed, <100%)
Published → Failed        (Pulsar permanently rejects)
```

Terminal states: `Confirmed`, `PartiallyPropagated`, `Failed`. The `PartiallyPropagated` state is the
escalation path: incident response runbook `revocation-incident.md` is triggered.

## 5. `revocation-usecase`

### 5.1 `OriginateRevocation`
- Input: `OriginateRevocationCommand { agreement_id, actor, reason }`
- Steps:
  1. Verify actor authorization (grantor / grantee / data-subject-if-B2C / system).
  2. Generate `RevocationEvent`.
  3. Insert into `consent_graph_revocations` table + outbox (one tx).
  4. Emit `oya.consent-graph.revocation-originated` audit event.
  5. Return `revocation_id`.
- Latency: p99 ≤100ms.

### 5.2 `ConfirmReceipt`
- Input: `ConfirmReceiptCommand { revocation_id, subscriber, action_taken }`
- Updates the `revocation_receipts` table; recomputes `expected_subscribers - receipts_received`;
  transitions lifecycle if delta reaches 0 or deadline passes.
- Latency: p99 ≤10ms.

### 5.3 `ReconcileDeadlines` (worker-invoked, every 100ms)
- Scans for `Propagating` revocations with `deadline_at < now()`.
- Transitions to `PartiallyPropagated` and fires alert.

## 6. `revocation-worker`

Long-running tokio task pool:
- **OutboxDrainer**: drains `consent_graph_revocations_outbox` → Pulsar via `revocation-adapter-pulsar`
  (IP-008). Latency budget: ≤50ms commit-to-publish.
- **DeadlineReconciler**: runs `ReconcileDeadlines` every 100ms.
- **CacheInvalidator**: per-pod subscriber that consumes the revocation Pulsar topic and calls
  `enforcement-sdk::invalidate_policy(agreement_id)` for every event in this pod's enforcement-app
  sibling.

## 7. Expected-subscriber set

When a revocation is originated, the system must know "who needs to receipt." Sources:
1. `enforcement-app` pods (one per enforcement deployment region).
2. Ontology projection-subscribers for the agreement.
3. Analytics projection-subscribers (if any).
4. Custom subscribers via partner-directory.

The expected set is computed at `OriginateRevocation` time by querying the
`consent_graph_active_subscribers` materialized view (joined from `projection-gateway` and
`partner-directory` state). Stale entries (subscriber went away without unregistering) auto-expire
after 5min. A subscriber missing from the expected set will *also* receipt if it sees the event —
extra receipts are idempotent.

## 8. Tests

| Test | Assertion |
|------|-----------|
| `originate_inserts_outbox_row_atomically` | revocation row + outbox row commit together |
| `receipt_path_transitions_to_confirmed` | N receipts × N subscribers → Confirmed |
| `deadline_reconciler_transitions_partial` | deadline passed → PartiallyPropagated + alert fired |
| `idempotent_originate` | duplicate `revocation_id` → no-op (idempotent key) |
| `actor_authorization_check` | non-party actor → AuthError |
| `b2c_data_subject_can_self_revoke` | data-subject actor permitted only on B2C-mode agreements |
| `stale_subscribers_auto_expire` | subscriber not heartbeating for 5min → removed from expected set |

## 9. Dependencies

- `oya-consent-graph-{agreement-kernel, agreement-sdk}`
- `oya-consent-graph-audit-bridge-sdk`
- `oya-consent-graph-enforcement-sdk`
- `tokio`, `serde`, `thiserror`, `sqlx` (worker only)

## 10. Schema

```sql
CREATE TABLE consent_graph_revocations (
    revocation_id ulid PRIMARY KEY,
    agreement_id ulid NOT NULL,
    grantor_tenant_id uuid NOT NULL,
    grantee_tenant_id uuid NOT NULL,
    reason text NOT NULL,
    initiated_by jsonb NOT NULL,
    initiated_at timestamptz NOT NULL,
    deadline_at timestamptz NOT NULL,
    lifecycle_state text NOT NULL,
    expected_subscribers int NOT NULL,
    receipts_received int NOT NULL DEFAULT 0
);
SELECT create_distributed_table('consent_graph_revocations', 'grantor_tenant_id');

CREATE TABLE consent_graph_revocations_outbox (LIKE consent_graph_agreement_outbox INCLUDING ALL);

CREATE TABLE consent_graph_revocation_receipts (
    revocation_id ulid NOT NULL,
    subscriber_service text NOT NULL,
    subscriber_region text NOT NULL,
    propagated_at timestamptz NOT NULL,
    propagation_latency_ms int NOT NULL,
    action_taken text NOT NULL,
    PRIMARY KEY (revocation_id, subscriber_service, subscriber_region)
);
```

## 11. SLO wiring

This IP feeds the `revocation-propagation-latency` SLO via the metric:
```
oya_consent_graph_revocation_propagation_seconds_bucket{action_taken="...", region="..."}
```

Emitted on every `ConfirmReceipt` call by the receipted subscriber.

## 12. Risk

- **R**: Outbox drainer falls behind during high-revocation burst (e.g., bulk DSAR cascade).
  **M**: Drainer scales with `outbox_unpublished_count_seconds_oldest` gauge; HPA on consumer lag.
  PartiallyPropagated fires alert at 1s deadline.
- **R**: Subscriber pod crashes mid-propagation — no receipt sent.
  **M**: Pulsar `read_compacted=true` + per-subscriber subscription means on restart the pod replays
  missed messages; receipt arrives late but eventually; deadline-passed reconciler still triggers but
  late receipts reconcile after-the-fact.
- **R**: DDoS revocation (malicious actor revokes 1M agreements / s).
  **M**: Per-actor revocation rate limit (1K/min default per tenant); audit emission rate-limited
  separately.

## 13. Verification

- `cargo test` clean.
- Integration test: originate → fan-out to 3 mock subscribers → all confirm within 1s → Confirmed.
- Chaos test: kill 1 of 3 subscribers mid-flight → deadline reconciler fires → PartiallyPropagated +
  alert.
