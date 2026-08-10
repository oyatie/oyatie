---
doc_class: IP
ip_id: IP-010
microservice: identity
status: ga
related_adrs: [ADR-0189, ADR-0183]
date: 2026-05-18
owner_team: axis-identity
---

# IP-010 — Step-up ACR orchestrator + Cedar gate

## Goal

Implement the step-up flow per ADR-0189: when a consumer µservice receives a request whose required ACR exceeds the principal's current ACR, return 401 with `X-Step-Up-Required: <level>,reason=<op>`; browser redirects to `/step-up?required_acr=X&return_to=Y`; the orchestrator chooses the correct factor ceremony (Passkey re-tap, hardware key, JIT IT-approval); upon success, mint a new OIDC ID-token with elevated `acr` claim + bumped `acr_event_at`.

## Files

| File | Purpose |
|---|---|
| `crates/oya-identity-step-up-orchestrator-kernel/Cargo.toml` | manifest |
| `crates/oya-identity-step-up-orchestrator-kernel/src/lib.rs` | StepUpOrchestrator trait + state machine |
| `crates/oya-identity-step-up-orchestrator-domain/src/lib.rs` | Cedar PolicyFragment exporter |
| `crates/oya-identity-step-up-orchestrator-rest/src/lib.rs` | axum routes |
| `crates/oya-identity-step-up-orchestrator-adapter/src/lib.rs` | bridges to Zitadel re-auth + WebAuthn + IT-approval API |
| `crates/oya-identity-step-up-orchestrator-kernel/tests/orchestrator.rs` | tests |

## ACR enum

Already in `oya-shared-oidc-client-kernel::AcrLevel`. Re-exported.

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | `/step-up` | render step-up UI; query: `required_acr`, `return_to`, `resource_uri` |
| POST | `/step-up/passkey/start` | begin WebAuthn re-tap; returns assertion challenge |
| POST | `/step-up/passkey/finish` | verify; mint new ID-token if successful |
| POST | `/step-up/hardware/start` | hardware-key ceremony (FIDO-MDS3 L2+ AAGUID) |
| POST | `/step-up/hardware/finish` | finish hardware ceremony |
| POST | `/step-up/it-approval/request` | open JIT IT-approval ticket bound to resource_uri |
| POST | `/step-up/it-approval/finish` | poll for approval token; mint critical session |

## Factor selection per ACR

| ACR | Factor sequence |
|---|---|
| elevated | re-present Passkey (synced or device-bound); time-bound 4h |
| sensitive | re-present Passkey + ONE other (2nd Passkey OR hardware key); time-bound 1h |
| critical | hardware key (FIDO-MDS3 L2+) + Passkey + JIT IT-approval; time-bound 15min |

## State machine

```
[client request denied 401 + X-Step-Up-Required]
       │
       ▼
[browser GET /step-up?required_acr=X&return_to=Y&resource=Z]
       │
       ▼
[orchestrator selects factor sequence per ACR]
       │
       ├──▶ Passkey re-tap (WebAuthn assert)
       │       │
       │       ▼
       │  [factor verified]
       │       │
       │       ▼  (more factors needed?)
       │       ├──▶ next factor
       │       │
       │       ▼ (all factors verified)
       │  [mint new ID-token with elevated acr + acr_event_at=now]
       │       │
       │       ▼
       │  [redirect to return_to]
```

## Cedar gate integration

The orchestrator does NOT enforce policy — Cedar PDP does. The orchestrator:
1. Issues the step-up flow.
2. On success, mints a new ID-token.
3. The consumer retries the original request.
4. Cedar PDP evaluates the new principal.acr_level against the policy.

The Cedar PolicyFragment from the orchestrator domain ships canonical predicates:

```cedar
// Predicate library
permit when { principal.acr_level >= AcrLevel::"elevated" };  // shorthand re-exported
forbid when { principal.acr_event_at + 3600 < context.now };  // sensitive session-age expired
forbid when { resource.tenant_id != principal.tenant_id };  // cross-tenant deny (re-asserted at step-up)
```

These predicates live in `iam/identity/policy/cedar-acr-predicates.cedar` and consumer µservices import them.

## Tests

| Test | Mechanism |
|---|---|
| `routine_to_elevated_via_passkey_retap` | mock WebAuthn; assert new token has acr=elevated |
| `elevated_to_sensitive_requires_two_factors` | first Passkey + second hardware-key |
| `sensitive_to_critical_requires_it_approval` | mock IT-approval endpoint; new token has acr=critical |
| `expired_factor_within_5min_window_rejected` | submit factor presented 6min ago → 401 |
| `it_approval_token_bound_to_resource` | approval for tenant_X resource_Y; reuse for resource_Z → 403 |
| `it_approval_token_one_time_use` | use once → ok; use again → 401 |
| `step_up_loop_rate_limited` | 4 step-ups in 60s → 429 |
| `audit_emitted_per_grant` | observe `IdentityStepUpGranted` event |
| `audit_emitted_per_deny` | observe `IdentityStepUpDenied` event |
| `failed_factor_3x_locks_user_15min` | 3 failed Passkeys → 423 Locked |

## Failure modes

- **Factor ceremony failure**: emit `IdentityStepUpDenied`; allow retry up to 3 attempts.
- **IT-approval timeout** (no approval in 5 min): expire; user must restart.
- **Orchestrator crash mid-flow**: state in Valkey; flow can resume from last completed factor up to 5 min after orchestrator recovery.

## Evidence

- `evidence/identity/step-up-grants/<pack>/<date>.json`
- `evidence/identity/step-up-denies/<pack>/<date>.json`
- `evidence/identity/it-approval-flow/<pack>/<date>.json`

## Acceptance — DONE when

- 10 orchestrator tests pass.
- End-to-end browser test: routine→elevated→sensitive→critical succeeds within 30s.
- `oya-check-step-up-auth-coverage` lane reports ≥80% sensitive-path coverage across the µservice fleet.

## Counterpart references - 010-step-up-orchestrator

- Counterpart class: identity substrate.
- Palantir Foundry and GitHub Enterprise are the counterpart baseline for governed multi-tenant identity surfaces; this IP ties the slice to Oyatie identity contracts, Cedar, and audit-chain evidence rather than leaving the behavior as generic application authentication.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `iam/identity/PRD.md`, `iam/identity/manifest.json`, and the contract/policy files cited above.

