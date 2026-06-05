---
doc_class: MigrationGuide
template_id: TPL-MIGRATION-GUIDE
microservice: messenger
status: Deprecated
deprecation_date: 2026-05-17
removal_target: advisory — HG-MESSENGER accepts at p99 SLOs sustained 30d
related_adrs: [ADR-0135, ADR-0131, ADR-0132, ADR-0133, ADR-0134]
related_specs: [/specs/microservices/messenger.json, /specs/microservices/messenger/messenger.json]
owner_team: axis-messenger
date: 2026-05-17
doc_status: published
---

# Migration: `oya-messenger-*` → `oya-messenger-*`

This document applies the Strangler Pattern from the agent-skills
`deprecation-and-migration` skill to the **messenger** µservice. It is the
consumer-facing companion to ADR-0134 (cross-µservice migration policy) and
ADR-0135 (target topology).

## Status

**Deprecated as of 2026-05-17 — replacement available and production-proven
in dev cluster.**

| Field | Value |
|---|---|
| Replacement | `oya-messenger-*` crate family under `microservices/messenger/src/crates/` |
| Removal date | **Advisory** — concrete target is HG-MESSENGER accepts at p99 SLOs sustained 30d (per ADR-0135 retirement trigger #2) |
| Reason | ADR-0132 no-grouping forward-policy + ADR-0139 per-µservice SLO authority + ADR-0131 per-µservice flat layout + MLS RFC 9420 E2E group messaging is a clean replacement boundary that did not exist in the legacy surface |
| Migration owner (Churn Rule) | axis-messenger |
| Migration window | Phase 2 adapter + Phase 3 canary = ~5 months; Phase 5 removal sweep in month 6 (see ADR-0134) |

## Replacement

The 9 bounded-contexts of the `messenger` µservice live under
`microservices/messenger/src/crates/` per ADR-0131. Each legacy
`oya-messenger-*` crate has a 1:1 replacement under the new prefix.

### Crate import-path map

| Legacy `oya-messenger-*` path | New `oya-messenger-*` path |
|---|---|
| `oya-messenger-domain` | (split per BC; see note below) |
| `oya-messenger-channel-store-kernel` | `oya-messenger-channel-store-kernel` |
| `oya-messenger-channel-store-usecase` | `oya-messenger-channel-store-usecase` |
| `oya-messenger-channel-store-adapter-postgres` | `oya-messenger-channel-store-adapter-postgres` |
| `oya-messenger-channel-store-rest` | `oya-messenger-channel-store-rest` |
| `oya-messenger-channel-store-app` | `oya-messenger-channel-store-app` |
| `oya-messenger-message-stream-kernel` | `oya-messenger-message-stream-kernel` |
| `oya-messenger-message-stream-usecase` | `oya-messenger-message-stream-usecase` |
| `oya-messenger-message-stream-adapter-kafka` | `oya-messenger-message-stream-adapter-kafka` |
| `oya-messenger-message-stream-worker` | `oya-messenger-message-stream-worker` |
| `oya-messenger-message-stream-app` | `oya-messenger-message-stream-app` |
| `oya-messenger-presence-kernel` | `oya-messenger-presence-kernel` |
| `oya-messenger-presence-usecase` | `oya-messenger-presence-usecase` |
| `oya-messenger-presence-adapter-valkey` | `oya-messenger-presence-adapter-valkey` |
| `oya-messenger-presence-worker` | `oya-messenger-presence-worker` |
| `oya-messenger-presence-app` | `oya-messenger-presence-app` |
| `oya-messenger-file-attachment-kernel` | `oya-messenger-file-attachment-kernel` |
| `oya-messenger-file-attachment-usecase` | `oya-messenger-file-attachment-usecase` |
| `oya-messenger-file-attachment-adapter-s3` | `oya-messenger-file-attachment-adapter-s3` |
| `oya-messenger-file-attachment-worker` | `oya-messenger-file-attachment-worker` |
| `oya-messenger-file-attachment-app` | `oya-messenger-file-attachment-app` |
| `oya-messenger-thread-tree-kernel` | `oya-messenger-thread-tree-kernel` |
| `oya-messenger-thread-tree-usecase` | `oya-messenger-thread-tree-usecase` |
| `oya-messenger-thread-tree-app` | `oya-messenger-thread-tree-app` |
| `oya-messenger-mention-router-kernel` | `oya-messenger-mention-router-kernel` |
| `oya-messenger-mention-router-worker` | `oya-messenger-mention-router-worker` |
| `oya-messenger-mention-router-app` | `oya-messenger-mention-router-app` |
| `oya-messenger-read-receipt-kernel` | `oya-messenger-read-receipt-tracker-kernel` |
| `oya-messenger-read-receipt-usecase` | `oya-messenger-read-receipt-tracker-usecase` |
| `oya-messenger-read-receipt-app` | `oya-messenger-read-receipt-tracker-app` |
| `oya-messenger-rest-api` | `oya-messenger-rest-api-surface-rest` |
| `oya-messenger-websocket` | `oya-messenger-websocket-frame-protocol-rest` |
| `oya-messenger-search-cedar-kernel` | `oya-messenger-search-and-cedar-filter-kernel` |
| `oya-messenger-search-cedar-app` | `oya-messenger-search-and-cedar-filter-app` |
| `oya-messenger-huddle-kernel` | `oya-messenger-huddles-livekit-signaling-kernel` |
| `oya-messenger-huddle-app` | `oya-messenger-huddles-livekit-signaling-app` |

> **`oya-messenger-domain` split.** Legacy bundled crate splits per
> ADR-0131 + ADR-0105; consumers must pick the specific replacement BC per
> import-site.

### Net-new boundary — MLS RFC 9420 E2E group messaging

The new µservice introduces **E2E (MLS RFC 9420)** as a first-class capability
that did NOT exist in `oya-messenger-*`. It is therefore not part of
the migration surface — it is a clean replacement-boundary feature. Specifically:

- `oya-messenger-e2e-mls-kernel`, `-usecase`, `-adapter`, `-worker`, `-app` —
  these crates have **no legacy counterpart**.
- The legacy surface offered TLS-in-transit only; the new µservice offers
  TLS-in-transit + MLS E2E group messaging per RFC 9420.
- Consumers wishing to use E2E must adopt the new µservice — there is no
  adapter shim for E2E because the legacy surface had no E2E shape to map to.

This is consistent with the deprecation-and-migration skill SKILL.md §"Step 1:
Build the Replacement" — the replacement must cover all critical use cases of
the legacy system (it does) and may additionally offer new capabilities (E2E
does).

### Concrete import migration recipes

```rust
// BEFORE
use oya_connect_messenger_channel_store_kernel::{Channel, ChannelAcl};
use oya_connect_messenger_message_stream_usecase::SendMessage;
use oya_connect_messenger_presence_kernel::PresenceState;

// AFTER
use oya_messenger_channel_store_kernel::{Channel, ChannelAcl};
use oya_messenger_message_stream_usecase::SendMessage;
use oya_messenger_presence_kernel::PresenceState;
```

```toml
# BEFORE
[dependencies]
oya-messenger-channel-store-kernel  = { workspace = true }
oya-messenger-message-stream-usecase = { workspace = true }

# AFTER
[dependencies]
oya-messenger-channel-store-kernel  = { workspace = true }
oya-messenger-message-stream-usecase = { workspace = true }
```

## Reason

The legacy `oya-messenger-*` family was authored before the no-grouping
forward-policy (ADR-0132) and the per-µservice flat layout (ADR-0131)
crystallised. Specifically:

1. **ADR-0132 no-grouping forward-policy.** A `connect-*` crate prefix encodes
   bundle membership at the architecture layer; bundle membership is a
   brand-layer concept and must not appear in crate names.
2. **ADR-0139 per-µservice SLO authority.** Messenger's persistent-connection
   count, message-stream throughput, presence-replication lag, websocket-frame
   p99, and huddle SFU-degradation MTTR each need independent SLO targets.
3. **ADR-0131 per-µservice flat layout.** Messenger's 10 runbooks, threat-
   model, DPIA, multi-region plan, sdk-plan all need to live under one folder
   (`microservices/messenger/`).
4. **MLS RFC 9420 is a clean replacement boundary.** E2E group messaging is
   net-new and cannot exist within a `connect-*` crate prefix.

## Runbook continuity — operational surface preserved

Per `microservices/messenger/runbooks/`, the new µservice ships 10 runbooks:
attachment-restore, channel-acl-drift, e2e-encryption-key-rotation,
ediscovery-export, huddle-sfu-degraded, mention-storm-throttle,
moderation-classifier-rollback, presence-rebuild, search-index-rebuild,
websocket-storm.

Of these, **6 had direct `oya-messenger-*` predecessors with the same
operational surface** (per this migration prompt's runbook callout):

| New runbook | Legacy operational surface | Semantic preservation |
|---|---|---|
| `search-index-rebuild.md` | Connect-messenger Elasticsearch rebuild procedure | Preserved verbatim: same trigger conditions, same expected rebuild window, same KPI (full-rebuild ≤6h for 1B documents) |
| `presence-rebuild.md` | Connect-messenger presence-cache cold-start | Preserved verbatim: same trigger (Valkey flush + restart), same warmup (~3 min for 1M users) |
| `mention-storm-throttle.md` | Connect-messenger mention fan-out throttle | Preserved verbatim: same per-channel mention-rate ceiling (100 mentions / 60s per channel) |
| `websocket-storm.md` | Connect-messenger websocket reconnect-storm dampening | Preserved verbatim: same exponential-backoff schedule (1s, 2s, 4s, 8s, capped 60s + jitter) |
| `attachment-restore.md` | Connect-messenger S3 attachment restore from versioning | Preserved verbatim: same restore-window (90 days), same chain-of-custody seal requirement |
| `channel-acl-drift.md` | Connect-messenger Cedar policy drift detector | Preserved verbatim: same drift threshold (5% of channels), same auto-page severity |

The remaining 4 runbooks (`e2e-encryption-key-rotation`, `ediscovery-export`,
`huddle-sfu-degraded`, `moderation-classifier-rollback`) are **new** and
have no legacy counterpart:

- `e2e-encryption-key-rotation` — operates on MLS RFC 9420 keys; legacy had
  no E2E.
- `ediscovery-export` — operates on cross-µservice eDiscovery handoff via
  `audit-chain`; legacy had a similar export but for a different bundled
  scope.
- `huddle-sfu-degraded` — operates on the LiveKit-signaling SFU; legacy had
  a less-instrumented huddle path.
- `moderation-classifier-rollback` — operates on the foundry-guardrails
  classifier; legacy had a smaller moderation surface.

## Migration Guide (step-by-step)

Identical 5-step process to mail's `migration-from-connect.md`:

### Step 1 — Add new dependency

### Step 2 — Update imports per the import-path map above

### Step 3 — Verify behavioural parity

```bash
cargo nextest run --features messenger-strangler-canary
```

### Step 4 — Remove the legacy dependency

### Step 5 — Verify zero residual

```bash
cargo tree -e normal -p your-crate | grep oya-messenger   # expect empty
rg "use oya_connect_messenger_" --type rust path/to/your/crate    # expect zero hits
```

## Configuration delta

| Configuration key | Legacy | New |
|---|---|---|
| Feature flag namespace | `connect.messenger.*` | `messenger.*` |
| OpenSLO file | bundled in `Connect.openslo.yaml` | `microservices/messenger/slos/messenger.openslo.yaml` |
| Helm chart values key | `.Values.connect.messenger.*` | `.Values.messenger.*` |
| K8s namespace | `connector` | `messenger` |
| WebSocket public endpoint | `wss://connect.<tenant>.oyatie.com/messenger/ws` | `wss://messenger.<tenant>.oyatie.com/ws` |
| REST API base | `https://connect.<tenant>.oyatie.com/messenger/v1/...` | `https://messenger.<tenant>.oyatie.com/v1/...` |
| Cedar policy fragment path | `policy/connect/messenger/*.cedar` | `microservices/messenger/policy/cedar/*.cedar` |
| Telemetry metric prefix | `oya_connect_messenger_*` | `oya_messenger_*` |

## Dual-context isolation invariant (preserved)

The Personal ↔ Professional context isolation invariant is preserved verbatim;
DM payloads in Personal context never leak to Professional channel surfaces
and vice versa. The kernel-layer `ContextBoundaryGuard` port keeps the same
method signatures across the migration.

## Hyrum's-Law surfaces — explicit callouts

Per the deprecation-and-migration skill SKILL.md §"Hyrum's Law Makes Removal
Hard", the legacy messenger surfaces with potentially-depended-on observable
behaviour, preserved verbatim during the canary:

1. **WebSocket close-code semantics.** Close codes 4000–4999 in the
   application range preserve their legacy mapping (4001 = idle-timeout,
   4002 = auth-expired, 4003 = banned, 4004 = rate-limited).
2. **Message ordering within a channel.** Legacy guaranteed
   per-channel-FIFO; new µservice preserves per-channel-FIFO via
   `oya-messenger-message-stream-adapter-kafka` partition key = channel-id.
3. **Presence transition cadence.** Legacy presence transitions (online →
   idle → offline) had a 5-min idle threshold + 15-min offline threshold;
   new µservice preserves both thresholds.
4. **Mention fan-out latency.** Legacy p99 mention-delivery (sender →
   recipient device-push) was ~250ms; new µservice targets ≤263ms
   (legacy + 5%).
5. **Search ranking tie-break.** Legacy used recency-descending as
   tie-breaker; new µservice preserves recency-descending.
6. **WebSocket frame size cap.** Legacy enforced 1 MiB per frame; new
   µservice preserves 1 MiB.

## Phases (per ADR-0134)

| Phase | Status (messenger) | Exit condition |
|---|---|---|
| 1. Parallel ship | **active** | HG-MESSENGER passes at p99 SLOs in dev cluster sustained 7d |
| 2. Adapter soak | pending | 3-month soak |
| 3. Feature-flagged canary | pending | 100% traffic on new µservice sustained 7d |
| 4. Zero-active-usage verification | pending | Verification commands all exit 0 |
| 5. Code removal sweep | pending | `cargo build --workspace` exits 0 |
| 6. Umbrella retirement | pending | All 8 HG-<MS> gates green sustained 30d |

## Verification checklist (per skill SKILL.md §"Verification")

- [ ] **Replacement is production-proven and covers all critical use cases.**
  ```bash
  buck2 build //:quality-lane-registry-authority-check # lane=hyperscaler-maturity-claims --microservice messenger
  ```
- [ ] **Migration guide exists with concrete steps and examples.**
  ```bash
  test -f microservices/messenger/migration-from-connect.md   # this file
  ```
- [ ] **All active consumers migrated** (per Phase 4):
  ```bash
  cargo tree -e normal -p oya-messenger-domain --invert    | grep -v 'oya-messenger-migration-adapter' | wc -l   # expect 0
  rg "use oya_connect_messenger_" --type rust    | rg -v "migration-adapter|legacy_in_process|tests/"    | wc -l   # expect 0
  ```
- [ ] **Old code, tests, documentation, configuration removed** (per Phase 5):
  ```bash
  find crates -maxdepth 1 -type d -name "oya-messenger-*" | wc -l   # expect 0
  ```
- [ ] **No references to the deprecated system remain in the codebase**:
  ```bash
  rg "oya_connect_messenger" --type rust    | rg -v "docs/decisions/|RETIRED.md|tests/reference/"    | wc -l   # expect 0
  ```
- [ ] **Deprecation notices removed** (per Phase 5):
  ```bash
  test ! -f microservices/messenger/deprecation-notice.md
  test ! -f microservices/messenger/migration-from-connect.md
  ```

## Breaking changes (flagged per `feedback_no_silent_regression`)

Phases 1–4: **not breaking** (adapter preserves the legacy symbol surface).

Phase 5: **breaking** for any consumer that did not migrate. Sunset
schedule advisory: 6 months from 2026-05-17 → target advisory removal
**2026-11-17**. Per `feedback_no_silent_regression`, the axis-messenger
team ships migration ChangeSets for every internal consumer per the Churn
Rule before Phase 5. The MLS E2E surface (`oya-messenger-e2e-mls-*`) is a
net-new addition and does not break anything because the legacy surface
had no E2E to compare against.

## References

- ADR-0135: super-app expansion into 8 flat µservices.
- ADR-0131: Per-microservice flat layout.
- ADR-0132: No-grouping forward-policy.
- ADR-0133: Industry best-practice conformance program.
- ADR-0134: dissolution Strangler migration (operational policy).
- `microservices/messenger/PRD.md` — full target-state product definition.
- `microservices/messenger/PHASE-01-TEAM-CHANNELS-DM-THREADS.md` — phase plan.
- `microservices/messenger/runbooks/*.md` — 10 runbooks.
- `microservices/messenger/deprecation-notice.md` — formal deprecation notice.
- `feedback_no_silent_regression.md` — no-silent-regression principle.
- agent-skills deprecation-and-migration SKILL.md — Strangler Pattern + Adapter Pattern + Churn Rule + Verification.
- RFC 9420 — MLS (Messaging Layer Security) protocol.
