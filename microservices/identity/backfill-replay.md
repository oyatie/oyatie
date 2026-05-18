---
doc_class: BackfillReplay
template_id: TPL-BACKFILL-REPLAY
microservice: identity
status: Accepted
date: 2026-05-18
owner_team: axis-identity + ops-data-platform
---

# Backfill + Replay — identity µservice

Three classes of replay/backfill the identity µservice supports.

## Class 1 — Audit-chain replay

**Purpose**: Compliance audit (SOC 2 evidence, GDPR Art. 30 records of processing, KR-FSS sector audit), forensic incident investigation, and downstream consumer state reconstruction.

**Mechanism**: Every identity event seals into `audit-chain` µservice (Merkle + Ed25519 per Bominal ADR-0028). The audit-chain µservice exposes a `replay` capability returning events filtered by `(microservice, event_type, tenant_id, time_window)`.

**Tooling**:
- `oya identity audit replay --tenant <t> --since 2026-01-01 --until 2026-03-31 --kinds 'IdentityOidcTokenIssued,IdentityStepUpGranted'`
- Output is an ordered stream of sealed events, each with Merkle proof; receivers verify the proof end-to-end.

**Performance**:
- Sequential replay throughput: 50K events/sec per replay worker.
- Random access (by event hash): O(log N) per Merkle tree.

**Retention horizon**:
- KR PIPA Enforcement Decree Art. 30: ≥1 year (KR-FSS sector ≥5 years).
- HIPAA §164.316(b)(2): 6 years.
- GDPR Art. 30: purpose-bounded (typical: 7 years).
- PCI-DSS v4.0 §10.5.1: ≥1 year (3 months immediately available).

The identity µservice keeps a 24h hot index of its own emissions for performant queries; older lookups go through audit-chain replay API.

## Class 2 — SCIM event replay

**Purpose**: 
- (a) Restore tenant state after rollback / accidental mass mutation.
- (b) Bootstrap a new consumer-side cache from authoritative SCIM history.
- (c) Verify SCIM idempotency claims (re-apply old events; assert no double-mutation).

**Mechanism**: SCIM operations are logged in a per-tenant append-only Postgres table `scim_request_log` with columns `(timestamp, tenant_id, http_method, path, body_hash, scim_bearer_id, response_status)`. The body is NOT stored verbatim (PII concern); a content-addressable lookup into a short-retention blob store retrieves the body for the 30-day window.

**Tooling**:
- `oya identity scim replay --tenant <t> --since 2026-04-01 --until 2026-04-15 --dry-run`
- Dry-run mode prints what WOULD happen; absence of `--dry-run` re-applies.
- Replay honours idempotency: re-applying an already-applied PATCH produces no state change (per ADR-0149).

**Conflict resolution**: If the current state diverges from what replay expects (because another mutation has happened since), the replay engine SURFACES the conflict (per-resource diff) for operator decision. Default: skip the conflicting record, log it; operator chooses to apply or discard.

## Class 3 — Passkey credential migration

**Purpose**: 
- (a) Migrate user credentials from a prior IdP (e.g., a tenant moves from Auth0 to oyatie).
- (b) Restore from a backup if the credential database corruption requires it.
- (c) Phase-2 migration from Zitadel to `oya-identity-server` (per ADR-0187 §In-house roadmap).

**Mechanism**: 
- WebAuthn credentials are PORTABLE (public key + AAGUID + transports + sign_count are the meta we store; the private key never moves and stays on the user's authenticator).
- Migration is an offline transform: source format → SCIM-shaped export → SCIM POST `Users` with `urn:oyatie:scim:extension:2.0:User` carrying credential refs.
- Sign-count starts at 0 (per W3C WebAuthn §6.1.1) after import; the first assertion after import is accepted at sign_count=0 and the counter starts fresh.

**Operator runbook**: `runbooks/passkey-credential-migration.md`.

**Verification**:
- Post-migration: every user's `IdentityWebAuthnRegistered` event is sealed in audit-chain with `source=migration`.
- Sample 5% of migrated credentials with end-to-end test (real assertion ceremony against the imported credential).

## Class 4 — Bootstrap a new pack

**Purpose**: Launching a new regulatory pack (e.g., pack-mx for Mexico) requires a Zitadel Instance deploy + an initial tenant + initial admin set.

**Procedure**:
1. Helm install Zitadel chart with pack-mx values.
2. Postgres event-store migration to current schema.
3. Provision OpenBao path `secret/identity/mx/*`.
4. Provision SCIM bearer for `tenant_root_mx`.
5. Run `oya identity bootstrap-pack --pack mx --admin-email <ops-on-call>`.
6. Verify JWKS endpoint reachable; OIDC discovery responds; SCIM endpoint responds.
7. First sign-in by ops-on-call (Passkey registration).

**Time budget**: ≤4h end-to-end for a new pack.

## Storage characteristics

| Class | Hot storage | Cold storage | Cost (year-5 / pack) |
|---|---|---|---|
| Audit-chain hot index | Postgres, 24h | audit-chain µservice ≥1y | $30/mo |
| SCIM request log | Postgres, 30d | tape S3 ≥7y | $50/mo |
| WebAuthn credentials | Postgres, active | (no archive — credentials are revoked, not archived) | included in DB |
| Migration staging blob | S3, 90d | (purged) | $10/mo |

## Audit cascades on replay

Any replay operation itself emits audit events:
- `IdentityAuditReplayInitiated(operator, tenant, kind, window)`
- `IdentityScimReplayApplied(operator, tenant, count, conflicts)`
- `IdentityCredentialMigrated(operator, source_idp, count)`

These are sealed in audit-chain like any other event, maintaining the chain-of-custody for replay actions themselves.
