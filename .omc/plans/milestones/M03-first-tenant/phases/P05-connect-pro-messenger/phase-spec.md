---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M03-first-paying-tenant
phase: P05-connect-pro-messenger
status: Proposed
entry_gate: |
  M03/P04-connect-pro-mail complete; oya-connect-app binary ships;
  oya-connect-mail-kernel + oya-connect-legal-hold-kernel crates exist;
  tenant DEK KMS operational; dual-context Cedar policies deployed.
exit_gate: |
  All IP acceptance gates green; messenger message round-trip test green;
  PQXDH handshake test green; Signal double-ratchet forward-secrecy test green;
  InternalAuditable thread mode enforced (org-pillar); deep-link to Workflow entity verified;
  `oya gate validate lean-a2 --ms connect` exits 0 (incremental check);
  k6 smoke messenger p99 ≤200ms at 5k concurrent WebSocket sessions;
  grit done on all P05 symbols; ICM phase-handoff row emitted.
depends_on:
  - milestone: M03
    phase: P04-connect-pro-mail
    reason: "Messenger shares oya-connect-app binary and dual-context Cedar policies with Mail; P04 must ship first to avoid re-wiring the composition root."
parallel_wave: 3
owner_team: council-connect
---

# P05-connect-pro-messenger: Connect Professional Messenger — E2E PQXDH, Signal double-ratchet, work-mode threads, Workflow deep-links

## Purpose

Extends the `oya-connect-*` µservice with Professional Messenger: real-time
channels and direct messages using PQXDH key exchange and Signal double-ratchet
for forward secrecy (per Bominal `platform/libs/ratchet/` port). Professional
(work) mode threads are `InternalAuditable` — stored encrypted under tenant DEK,
decryptable via four-eyes audit (ADR-0208). Deep-links to Workflow runs, HR
employment records, and Payroll entries embedded as typed Ontology Object
references inside message payloads.

Personal Messenger (`oya-connect-personal-*`) remains scaffolded but NOT GA;
only `InternalAuditable` (Professional) threads are activated at M03.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Crate family (BNF v4.1) |
|---|---|---|
| `connect` | `messenger` | `oya-connect-messenger-{kernel,domain,application,adapter,rest,grpc}` |

(All other Connect BCs already ship in P04; this phase adds the `messenger` BC only.)

Naming justifications:

```
NAME: oya-connect-messenger-kernel
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger: messenger BC owns Channel + DirectMessage entities + MessengerStore/RatchetKeyStore port-traits + PQXDH session types; ADR-0056 v4.1 BC-optionality (connect has multiple BCs)
- layer = kernel: pure ChannelId/MessageId value types + MessengerStore/RatchetKeyStore port declarations; zero logic; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-messenger-domain
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger; layer = domain: Channel aggregate + DirectMessage entity + InternalAuditable/E2E mode invariants + ratchet session state machine + deep-link ObjectReference validation; calls through MessengerStore; no I/O; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-messenger-application
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger; layer = application: SendMessageUseCase + CreateChannelUseCase + PQXDH handshake coordinator + Workflow deep-link resolver (reads Ontology via ObjectStore port); ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-messenger-adapter
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger; layer = adapter: PostgresMessengerStore (implements MessengerStore), ValKeyRatchetKeyStore (implements RatchetKeyStore), WebSocketPushAdapter; ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-messenger-grpc
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger; layer = grpc: tonic gRPC service for real-time message delivery bus (internal fan-out); ADR-0056 §"Layer semantics"
- exemptions: none

NAME: oya-connect-messenger-rest
JUSTIFICATION:
- microservice = connect; bc-tokens = messenger; layer = rest: Axum HTTP + WebSocket upgrade handlers for /channels, /dms CRUD and real-time feed; ADR-0056 §"Layer semantics"
- exemptions: none
```

### Out-of-scope

- Personal Messenger GA — deferred post-M03 pending crypto audit.
- Disappearing messages (TTL per conversation) — Personal context only; deferred.
- Stories/Status — Personal context only; deferred.
- Community (Reddit-style threaded discussion) — post-M03 feature depth.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Connect Professional Messenger: PQXDH + Signal double-ratchet, InternalAuditable thread mode, WebSocket real-time fan-out, Workflow entity deep-links, tenant DEK message storage, audit trail, load tests | pending | council-connect |

---

## Acceptance Gates

### Cargo / CI gates

```bash
cargo check --workspace --all-features                                           # exit 0
cargo build -p oya-connect-app --all-features                                   # exit 0 (incremental; messenger BC wired)
cargo clippy -p oya-connect-messenger-domain -- -D warnings                     # exit 0
cargo nextest run -p oya-connect-messenger-domain --test pqxdh_handshake        # exit 0; PQXDH key exchange
cargo nextest run -p oya-connect-messenger-domain --test ratchet_forward_secrecy  # exit 0; Signal double-ratchet
cargo nextest run -p oya-connect-messenger-domain --test internal_auditable_mode  # exit 0; InternalAuditable thread enforced
cargo nextest run -p oya-connect-messenger-domain --test deep_link_ontology_ref  # exit 0; Workflow/HR/Payroll object refs valid
cargo deny check                                                                 # exit 0
```

### Fitness lane gates

```bash
oya gate validate lean-a2 --ms connect             # LEAN-A2 still passing after messenger BC addition
oya gate validate lean-a1 --ms connect             # layer ordering
oya gate validate port-location --ms connect       # messenger ports in kernel
oya gate validate audit-chain --ms connect         # Ed25519 seal on messenger messages
```

### Performance gate

```bash
# k6: messenger message p99 ≤200ms at 5k concurrent WebSocket sessions
k6 run tests/load/smoke-connect-messenger-ws.js --env BASE_URL=ws://localhost:8084
# Pass: http_req_duration{p(99)}<200; error rate <0.1%
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate | Layer | Port traits in kernel? | Impls in adapter? |
|---|---|---|---|
| `oya-connect-messenger-kernel` | `kernel` | Yes — `MessengerStore`, `RatchetKeyStore` | N/A |
| `oya-connect-messenger-domain` | `domain` | N/A | N/A |
| `oya-connect-messenger-application` | `application` | N/A | N/A |
| `oya-connect-messenger-adapter` | `adapter` | N/A | Yes — `PostgresMessengerStore`, `ValKeyRatchetKeyStore`, `WebSocketPushAdapter` |
| `oya-connect-messenger-rest` | `rest` | N/A | No direct adapter import |
| `oya-connect-messenger-grpc` | `grpc` | N/A | No direct adapter import |

Cross-product: messenger NEVER imports `oya-hr-*`, `oya-payroll-*`, `oya-workflow-*` directly.
Workflow / HR / Payroll entity deep-links resolved via `oya-ontology-entity-kernel::ObjectStore` port.

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `messenger` | `connect` | pending |

---

## Grit Claim Symbols

```
crates/oya-connect-messenger-kernel/src/ports.rs::MessengerStore
crates/oya-connect-messenger-kernel/src/ports.rs::RatchetKeyStore
crates/oya-connect-messenger-domain/src/channel.rs::Channel
crates/oya-connect-messenger-domain/src/ratchet_session.rs::RatchetSession
crates/oya-connect-messenger-domain/src/deep_link.rs::ObjectReference
contracts/connect.openapi.yaml::sendDirectMessage
contracts/connect.openapi.yaml::createChannel
docs/standards/bounded-contexts.md::connect.messenger
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P05-connect-pro-messenger started; extends P04 Connect binary; PQXDH + Signal double-ratchet for InternalAuditable Professional threads; Workflow/HR/Payroll deep-links via Ontology" \
  -i high \
  -k "M03,P05,phase-start,connect,messenger"

icm store \
  -t context-oyatie \
  -c "Phase P05-connect-pro-messenger complete; Professional Messenger (InternalAuditable mode) shipped; PQXDH + Signal ratchet; WebSocket real-time; Workflow deep-links; tenant DEK storage; next: P06-application-b2b-live" \
  -i high \
  -k "M03,P05,phase-complete,connect,messenger"
```

---

## References

- PRD: `docs/prds/connect.md`
- Bominal ADRs inherited: ADR-0208 (dual-context), ADR-0215 (retention/legal hold), ADR-0111 (tenant DEK), ADR-0047 (deep-links to Workflow + HR/Payroll entities), ADR-0028 (audit chain)
- oyatie ADRs: ADR-0056 (BNF v4.1)
