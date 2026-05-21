---
doc_class: ImplementationPlan
status: Accepted
date: 2026-05-20
related_adrs: [ADR-0253, ADR-0258, ADR-0135, ADR-0263]
acceptance_status: draft
companion_docs:
  - microservices/mail/ARCHITECTURE.md
  - microservices/mail/contracts/openapi/mail.yaml
  - microservices/mail/contracts/proto/mail.proto
  - microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml
inbound_citations: [microservices/mail/manifest.json]
---

# IP-016: JMAP RFC 8620 frontend

## A. Problem

Mail's first-party clients need a protocol that can express mailbox state, email objects, threads, blob upload/download, and push state without forcing the web and mobile clients through IMAP shape. The PRD names JMAP Core and Mail as day-one standards parity, while `contracts/openapi/mail.yaml` currently exposes a REST facade and `contracts/proto/mail.proto` exposes gRPC mail operations. This IP closes the gap between those generic transport surfaces and a JMAP-native edge that can compete with Fastmail and Stalwart without copying Exchange/MAPI.

The key risk is not just route coverage. JMAP must preserve the same `ContextKind` and `X-Tenant-Id` isolation as the rest of mail, advertise capabilities honestly in `/jmap/session`, and route message reads through mailbox-store, Tantivy search, and legal-hold conflict checks. A JMAP client must never discover a Personal mailbox while authenticated to a Professional tenant context.

## B. Approach

Create `oya-mail-jmap-frontend-rest` as a transport adapter over existing mail contracts. The adapter owns RFC 8620 session discovery, request batching, method dispatch, upload/download routes, state tokens, and push subscription projection. It does not own mailbox storage or search. It calls the mailbox-store kernel/domain ports described in IP-002..IP-004, the search-index adapter from IP-009, and the dual-context guard from IP-005.

The implementation maps JMAP methods to existing contract concepts: `Mailbox/get` to `Mailbox`, `Email/query` to `SearchRequest` plus mailbox filters, `Email/get` to `MailMessage`, `Thread/get` to `Thread`, and `EmailSubmission/set` to `SendMessage`. `/jmap/upload/{accountId}` writes MIME blobs through the S3 blob adapter and returns blob ids that `Email/set` can reference. Push uses the existing stream events from `contracts/proto/mail.proto` and emits state changes compatible with RFC 8620 section 7.

## C. Deliverables

| Artifact | Change |
|---|---|
| `microservices/mail/catalog/oya-mail-jmap-frontend-rest.yaml` | tighten catalog row to bind the crate to RFC 8620/8621, HTTP/3, and dual-context guard |
| `microservices/mail/contracts/openapi/mail.yaml` | add JMAP session, batched method-call, upload, download, and push endpoint descriptions if absent |
| `microservices/mail/contracts/proto/mail.proto` | document mapping from `ListMailboxes`, `GetMessage`, `SearchMail`, and `SendMessage` to JMAP methods |
| `microservices/mail/slos/jmap-mailbox-fetch-latency.openslo.yaml` | make P99 mailbox fetch and method-batch latency the acceptance SLO |
| `microservices/mail/iac/edge-waf.yaml` | allow `/jmap/*` while retaining HTTP/3, h2 fallback, ECH, and PQC policy |
| `microservices/mail/benchmarks/gmail-m365-proton-vs-oyatie.md` | add Fastmail/Stalwart JMAP conformance benchmark notes |

## D. Implementation

1. Add a JMAP session resource in the REST contract: `/jmap/session` returns `apiUrl`, `uploadUrl`, `downloadUrl`, `eventSourceUrl`, account ids, max object sizes, and capability ids.
2. Implement method-batch parsing for `/jmap/api`; enforce request-size and method-count ceilings before any mailbox-store call.
3. Route `Mailbox/get`, `Mailbox/query`, `Email/get`, `Email/query`, `Thread/get`, and `EmailSubmission/set` through the existing mailbox, search, and outbound submission ports, carrying `tenant_id`, `ContextKind`, `home_cell`, and audit correlation.
4. Add a state-token ledger keyed by account id and mailbox id; derive invalidation from `StreamMessageReceived` and `ReputationChangedEvent` in `contracts/proto/mail.proto`.
5. Wire upload/download to the S3 MIME blob adapter and reject cross-context blob reads through the IP-005 `ContextBoundaryGuard`.
6. Add HTTP/3 Alt-Svc coverage in `iac/edge-waf.yaml`; verify h2 fallback still reaches the same policy path.
7. Register ADR-0263 audit events for JMAP batch accepted, JMAP method denied, JMAP upload accepted, and JMAP push subscription opened.
8. Add conformance and performance checks: JMAP test suite, malformed method-call tests, Personal-vs-Professional refusal tests, and P99 mailbox fetch under the SLO file.

## E. Acceptance

- `GET /jmap/session` advertises only account ids visible to the caller's `ContextKind`.
- Batched JMAP calls return per-method errors without leaking hidden mailbox ids or message ids.
- `Email/query` and `SearchSnippet` meet `mail-jmap-mailbox-fetch-latency` and do not bypass encrypted-token search rules.
- Upload/download paths reject stale blob ids, wrong-account blob ids, and Professional-to-Personal context crossings.
- CI runs the Fastmail JMAP conformance suite reference plus local tests for request batching, state-token invalidation, and h2 fallback.

## F. Evidence

- `microservices/mail/PRD.md` protocol matrix lists JMAP Core/Mail as a day-one target and Fastmail/Stalwart as JMAP anchors.
- `microservices/mail/contracts/openapi/mail.yaml` already defines mailboxes, messages, search, legal holds, eDiscovery, and DLP surfaces that the JMAP adapter maps onto.
- `microservices/mail/contracts/proto/mail.proto` provides event streams needed for JMAP push state.
- `microservices/mail/competitor-parity-matrix.md` states Fastmail and Stalwart support JMAP while Gmail/Exchange do not.
- RFC 8620 and RFC 8621 are the protocol authorities.

## G. Counterparts

| Counterpart | Gap closed by this IP |
|---|---|
| Fastmail | Matches the strongest JMAP-native commercial precedent while preserving Oyatie dual-context isolation and audit-chain semantics. |
| Stalwart Mail Server | Keeps self-hosted JMAP parity available as an adapter precedent without making Stalwart the product boundary. |
| Gmail / Microsoft Exchange | Exceeds their public JMAP coverage by offering a first-class standards API instead of only IMAP/MAPI-shaped access. |

## H. Non-goals and handoff boundaries

- Do not implement mailbox persistence in the JMAP crate; persistence remains in mailbox-store IP-002..IP-004.
- Do not create an Exchange/MAPI compatibility layer in this IP; ActiveSync/EAS remains roadmap work.
- Do not index plaintext for E2EE accounts to satisfy `SearchSnippet`; metadata-only behavior is acceptable when encryption requires it.
- Do not expose Personal account ids in a Professional `/jmap/session` response even when the same human owns both accounts.
- Do not bypass legal-hold conflicts on `Email/set` destroy operations; conflict responses must match the REST facade.

## I. Fixture set

- `session_personal_only.json` proves Personal account discovery without `X-Tenant-Id`.
- `session_professional_only.json` proves Professional account discovery with tenant id and no Personal leakage.
- `batch_mixed_success_and_denied.json` proves per-method errors inside one JMAP request.
- `upload_wrong_account.json` proves blob account mismatch is rejected.
- `push_state_after_message_received.json` proves state-token invalidation from `StreamMessageReceived`.

## API Versioning (per ADR-0342)
- Carrier: public boundary uses `Oyatie-Version: 2026-05-21`, URL prefix `/v/2026-05-21/`, and proto3 field tag `8001` for `oyatie_version`.
- `declared_version`: `2026-05-21`; support window is `N=3` public date versions for at least `180` days after deprecation.
- Internal-mesh exemption: internal gRPC remains on mesh proto3 compatibility and does not require the public URL/header carrier.
- Surface evidence: `microservices/mail/IP-016-jmap-rfc-8620-frontend.md` matched `openapi, .proto`; contract files `microservices/mail/contracts/openapi/mail.yaml, microservices/mail/contracts/asyncapi/mail-events.yaml, microservices/mail/contracts/proto/mail.proto`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## DR posture (per ADR-0343)
- Manifest target source: `microservices/mail/manifest.json#dr` is missing; `rto_p99_seconds` and `rpo_p99_seconds` are not invented in this IP.
- Applicable compliance-pack floor source: HIPAA-2024(rto=3600,rpo=300,multi_region=true), SOC2-T2(rto=14400,rpo=900,multi_region=false), EU-AI-ACT-2024-HIGH-RISK(rto=1800,rpo=300,multi_region=true), ISO27001-2022(rto=14400,rpo=3600,multi_region=false), KR-PIPA-2023-amendment(rto=14400,rpo=900,multi_region=false) from `specs/compliance-pack-floors.json`.
- Multi-region posture: `multi_region_active_active` is not declared in the manifest; any floor with `multi_region=true` must force active-active before this IP can serve that pack.
- `backup_substrate` enumeration: valkey, valkey_cluster, postgres_wal_g, iceberg_snapshot, object_storage_versioned, seaweedfs_replicated, milvus_snapshot, clickhouse_iceberg_layered, openbao_seal_unseal, audit_chain_merkle_seal.
- Surface evidence: `microservices/mail/IP-016-jmap-rfc-8620-frontend.md` matched `SLO`; anchors `microservices/mail/runbooks/mailbox-restore-from-backup.md, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.
