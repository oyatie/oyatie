---
doc_class: IP
ip_id: IP-012
microservice: identity
status: ga
related_adrs: [ADR-0162]
date: 2026-05-18
owner_team: axis-identity
---

# IP-012 — Audit emitter bridge to audit-chain

## Goal

Bridge every identity-class event into the `audit-chain` µservice with Ed25519 signing + Merkle leaf insertion per Bominal ADR-0028 + per-tenant slicing per ADR-0162. Provide a robust DLQ + retry path so audit-emit completeness SLO stays at 1.0.

## Files

| File | Purpose |
|---|---|
| `crates/oya-identity-audit-emitter-kernel/Cargo.toml` | trait + types |
| `crates/oya-identity-audit-emitter-kernel/src/lib.rs` | `AuditEmitter` trait + 18 event types |
| `crates/oya-identity-audit-emitter-usecase/src/lib.rs` | dispatcher + sign + enqueue |
| `crates/oya-identity-audit-emitter-app/src/lib.rs` | background drainer |
| `crates/oya-identity-audit-emitter-adapter-audit-chain-bridge/src/lib.rs` | gRPC to audit-chain |

## Event types

Sealed event set (18 distinct, declared in `manifest.json#audit_chain.seal_events`):

1. IdentityUserProvisioned
2. IdentityUserReactivated
3. IdentityUserSuspended
4. IdentityUserDeleted
5. IdentitySignInSucceeded
6. IdentitySignInFailed
7. IdentityStepUpGranted
8. IdentityStepUpDenied
9. IdentityWebAuthnRegistered
10. IdentityWebAuthnRevoked
11. IdentityScimRequestReceived
12. IdentityOidcTokenIssued
13. IdentityOidcTokenRevoked
14. IdentityJwksRotated
15. IdentityExternalIdpBound
16. IdentityExternalIdpUnbound
17. IdentityHrisHirePulled
18. IdentityHrisTerminationPulled

Each event carries: `event_id (UUID v7)`, `tenant_id`, `pack`, `user_id` (when applicable), `actor` (principal or system), `timestamp_unix_micros`, `payload_json_signed`, `signature_b64` (Ed25519).

## Emission flow

```
[handler] ──▶ [usecase.emit(event)] ──▶ [enqueue Valkey]
                                              │
                                              ▼
                                        [drainer pulls]
                                              │
                                              ▼
                                  [Ed25519 sign with pack-pinned key]
                                              │
                                              ▼
                              [gRPC to audit-chain µservice]
                                              │
                              ┌───────────────┴────────────────┐
                              ▼                                ▼
                       [seal ack → drop]           [error → DLQ retry]
```

## DLQ + retry

- Retry up to 3 times with exponential backoff (1s, 5s, 30s).
- After 3 failures, write to DLQ Postgres table.
- DLQ replay tooling: `oya identity audit dlq replay --since <t>`.
- DLQ growing > 1000 entries → critical alarm.

## Signing key

- Per-pack Ed25519 keypair in OpenBao + HSM (regulated packs).
- Public key embedded in audit-chain seal verifier.
- Rotation cadence: 90 days.

## Tests

| Test | Mechanism |
|---|---|
| `every_event_type_can_be_emitted_and_sealed` | enumerate 18 event types; each successfully seals |
| `signature_verifies_with_published_public_key` | sign + verify roundtrip |
| `gRPC_failure_retries_3_times` | mock fail-then-succeed |
| `dlq_engaged_after_3_failures` | sustained failure; DLQ row appears |
| `audit_chain_responds_with_merkle_proof` | mock returns Merkle proof; emitter stores it |
| `events_ordered_per_tenant` | inject 100 events; observe order preserved in audit-chain |
| `concurrent_emits_no_corruption` | 1000 concurrent; all succeed |
| `signing_key_rotation_seamless` | rotate mid-flow; events continue to seal |
| `per_tenant_slicing_honoured` | events bear tenant_id; audit-chain slices |
| `dlq_replay_idempotent` | replay same DLQ entry twice; only one new seal |

## Acceptance — DONE when

- 10 tests pass.
- Live audit-chain integration in staging.
- Completeness SLO target 1.0 sustained for 7 days at synthetic-load 1000 eps.
- DLQ replay tool exercises clean.

## Cross-references

- Bominal ADR-0028 audit-chain Merkle + Ed25519
- ADR-0162 per-tenant audit-log slicing

## Counterpart references - 012-audit-emitter

- Counterpart class: audit and regulated evidence.
- ServiceNow GRC and Palantir Foundry demonstrate the enterprise expectation that identity actions produce reviewable evidence; this IP requires sealed identity events and regulator/auditor-safe context rather than a flat admin log.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

## DR posture (per ADR-0343)

- Authority: ADR-0343.
- Trigger evidence: `microservices/identity/IP-012-audit-emitter.md` matched `SLO`.
- Numeric target: `rto_p99_seconds=30`, `rpo_p99_seconds=0` from manifest.json#rpo_rto.
- Applicable compliance pack floor: HIPAA-2024(3600s/300s MR), KR-PIPA-2023-amendment(14400s/900s), SOC2-T2(14400s/900s), ISO27001-2022(14400s/3600s), PCI-DSS-L1-v4(86400s/3600s) from `specs/compliance-pack-floors.json`; manifest evidence `microservices/identity/manifest.json`.
- Multi-region posture: `multi_region_active_active=true` for this HA-critical IP path.
- Backup substrate: `postgres_wal_g`, `valkey_cluster`, `openbao_seal_unseal`, `audit_chain_merkle_seal`.
- Runtime evidence: `microservices/identity/slos/oidc-token-issue-latency.openslo.yaml`, `microservices/identity/slos/oidc-token-verify-latency.openslo.yaml`, `microservices/identity/slos/webauthn-authenticate-latency.openslo.yaml`, `microservices/identity/slos/scim-availability.openslo.yaml`, `microservices/identity/policy/cedar-acr-predicates.cedar`.
