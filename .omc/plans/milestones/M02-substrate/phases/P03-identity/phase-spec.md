---
doc_class: PhaseSpec
template_id: TPL-PHASE-SPEC
milestone: M02-substrate
phase: P03-identity
status: Proposed
acceptance_lanes: []
entry_gate: 'M01-P05 complete; oya-tenancy-kernel ships (TenantId type available);

  Postgres 16 with pgcrypto available; cargo check --workspace exits 0.

  '
exit_gate: "All identity crates compile; migrations/V001__identity_init.sql applied;\n\
  RLS policies verified on all 6 tenant-scoped tables; auth flows (OIDC +\npasskey\
  \ + MFA) compile; Cedar policy lints; Protobuf compiles; k6 smoke\np99\u2264200ms\
  \ on session creation; grit done on all symbols; ICM row emitted.\n"
depends_on:
- milestone: M01
  phase: P05-scaffold-locks
  reason: TenantId type + workspace scaffold prerequisite
owner_team: council-architecture
purpose: "This phase delivers the complete Identity substrate, implementing the 8 bounded contexts that represent the human/organizational layer of oyatie: users (auth principal), persons (human record), organizations (legal entity inside tenant)."
---
# P03-identity: Full Identity substrate — users, persons, organizations, employees, employments, sessions, mfa, passkeys

## Purpose

This phase delivers the complete Identity substrate, implementing the 8 bounded contexts that represent the human/organizational layer of oyatie: users (auth principal), persons (human record), organizations (legal entity inside tenant), employees (Person × Organization junction), employments (effective-dated role snapshots), sessions (JWT + refresh lifecycle), mfa (TOTP/FIDO2 challenge-response), and passkeys (WebAuthn credential store). Identity enforces the critical Bominal ADR-0125 distinctions: Tenant ≠ Organization, User ≠ Person, Person ≠ Employee. Auth flows implement the Bominal ADR-0123 two-cookie + PKCE + nonce cross-product contract. Without Identity no product can authenticate users, assign roles, or enforce per-employee access control. This phase advances the "auth-first" Master Plan principle.

---

## Scope

### In-scope

| µservice | Bounded Contexts | Files / crates affected | BNF v4.1 crate names |
|---|---|---|---|
| `identity` | `users`, `persons`, `organizations`, `employees`, `employments`, `sessions`, `mfa`, `passkeys` | `crates/oya-identity-{users,persons,organizations,employees,employments,sessions,mfa,passkeys}-{kernel,domain,application,adapter}/`, `crates/oya-identity-rest/`, `crates/oya-identity-grpc/`, `crates/oya-identity-worker/`, `crates/oya-identity-app/` | full matrix 8×4 = 32 crates + 3 presentation + 1 app = 36 total |

Naming justification (representative):

```
NAME: oya-identity-users-kernel
JUSTIFICATION:
- microservice = identity: auth + identity substrate; registered in workspace;
  ADR-0056 v4.1 flat BNF
- bc-tokens = users: the auth-principal BC per Bominal ADR-0125 User ≠ Person
  distinction; multiple BCs exist at kernel layer so token is included
- layer = kernel: pure types + UserStore/SessionStore/AuthChallenger port traits;
  zero I/O; ADR-0056 §"Layer semantics"
- exemptions claimed: none

NAME: oya-identity-passkeys-kernel
JUSTIFICATION:
- microservice = identity: same µservice
- bc-tokens = passkeys: WebAuthn credential BC; distinct from mfa BC
  (passkeys = platform authenticator; mfa = TOTP/FIDO2 fallback)
- layer = kernel: PasskeyStore port trait
- exemptions claimed: none
```

### Out-of-scope

- Tenancy-level RLS bootstrap (`oyatie.set_current_tenant`) — owned by oya-tenancy-kernel; imported here.
- LDAP/AD sync — deferred to Wave-C enterprise-directory phase.
- Korean 주민등록번호 / foreigner registration validation — deferred to M03 compliance phase.

---

## Implementation Plans

| IP file | Intent | Status | Owner |
|---|---|---|---|
| [`impl-plan.md`](impl-plan.md) | Full DDL + port traits + Cedar + Proto + REST/gRPC + load test for all 8 BCs | pending | `council-architecture` |

---

## Acceptance Gates

### Cargo / CI gates (exit 0 required)

```bash
cargo check --workspace --all-features               # exit 0
cargo build --workspace --all-features               # exit 0
cargo clippy --workspace --all-features -- -D warnings  # exit 0
cargo nextest run --workspace --all-features         # exit 0; 0 failures
cargo deny check                                     # exit 0
cargo doc --workspace --no-deps                      # exit 0; 0 warnings
```

### Fitness lane gates

```bash
oya gate validate lean-a1 --phase P03-identity
oya gate validate lean-a2 --phase P03-identity
oya gate validate lean-a3 --phase P03-identity
oya gate validate lean-a4 --phase P03-identity
```

### Auth flow gates

```bash
# OIDC PKCE flow integration test
cargo nextest run -p oya-identity-sessions-application --test oidc_pkce_flow  # exit 0
# Passkey registration + assertion round-trip
cargo nextest run -p oya-identity-passkeys-application --test webauthn_round_trip  # exit 0
# MFA TOTP challenge-response
cargo nextest run -p oya-identity-mfa-application --test totp_challenge  # exit 0
# Argon2id password hash verify
cargo nextest run -p oya-identity-users-application --test argon2id_round_trip  # exit 0
```

### Load test gate

```bash
k6 run tests/load/smoke-identity-sessions.js --env BASE_URL=http://localhost:8081
# Pass: p99 ≤200ms on session create + validate; error rate <0.1%
```

---

## Clean Architecture Compliance

### Layer assignments

| Crate (BNF v4.1) | Layer | Port traits in kernel? | Impls in adapter? | Presentation-only? |
|---|---|---|---|---|
| `oya-identity-users-kernel` | `kernel` | Yes — `UserStore`, `AuthChallenger` | N/A | No |
| `oya-identity-sessions-kernel` | `kernel` | Yes — `SessionStore` | N/A | No |
| `oya-identity-passkeys-kernel` | `kernel` | Yes — `PasskeyStore` | N/A | No |
| `oya-identity-mfa-kernel` | `kernel` | Yes — `MfaChallengeStore` | N/A | No |
| `oya-identity-persons-kernel` | `kernel` | Yes — `PersonStore` | N/A | No |
| `oya-identity-organizations-kernel` | `kernel` | Yes — `OrganizationStore` | N/A | No |
| `oya-identity-employees-kernel` | `kernel` | Yes — `EmployeeStore` | N/A | No |
| `oya-identity-employments-kernel` | `kernel` | Yes — `EmploymentStore` | N/A | No |
| `oya-identity-users-adapter` | `adapter` | N/A | Yes — Postgres `UserStore` impl | No |
| `oya-identity-rest` | `rest` | N/A | No direct adapter | Yes |
| `oya-identity-app` | `app` | N/A | Unrestricted inward | No |

### Port traits declared in kernel

```rust
// oya-identity-users-kernel/src/ports.rs
#[doc(hidden)]
mod sealed { pub trait Sealed {} }

#[async_trait::async_trait]
pub trait UserStore: Send + Sync + sealed::Sealed {
    async fn get(&self, user_id: UserId) -> Result<Option<User>, IdentityError>;
    async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError>;
    async fn find_by_phone(&self, phone: &Phone) -> Result<Option<User>, IdentityError>;
    async fn register(&self, draft: UserDraft) -> Result<User, IdentityError>;
    async fn lock(&self, user_id: UserId, reason: LockReason) -> Result<(), IdentityError>;
    async fn unlock(&self, user_id: UserId) -> Result<(), IdentityError>;
}

#[async_trait::async_trait]
pub trait AuthChallenger: Send + Sync + sealed::Sealed {
    async fn challenge(&self, user: &User, method: ChallengeMethod)
        -> Result<ChallengeToken, IdentityError>;
    async fn verify(&self, token: ChallengeToken, response: ChallengeResponse)
        -> Result<MfaSatisfied, IdentityError>;
}

// oya-identity-sessions-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync + sealed::Sealed {
    async fn create(&self, user_id: UserId, tenant_id: Option<TenantId>, mfa_satisfied: bool)
        -> Result<Session, IdentityError>;
    async fn validate(&self, session_id: SessionId) -> Result<Session, IdentityError>;
    async fn revoke(&self, session_id: SessionId) -> Result<(), IdentityError>;
    async fn revoke_all_for_user(&self, user_id: UserId) -> Result<u64, IdentityError>;
    async fn refresh(&self, session_id: SessionId) -> Result<Session, IdentityError>;
}

// oya-identity-passkeys-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait PasskeyStore: Send + Sync + sealed::Sealed {
    async fn register_credential(&self, user_id: UserId, credential: WebAuthnCredential)
        -> Result<CredentialId, IdentityError>;
    async fn get_credentials(&self, user_id: UserId)
        -> Result<Vec<WebAuthnCredential>, IdentityError>;
    async fn verify_assertion(&self, user_id: UserId, assertion: WebAuthnAssertion)
        -> Result<(), IdentityError>;
    async fn revoke_credential(&self, credential_id: CredentialId) -> Result<(), IdentityError>;
}

// oya-identity-employees-kernel/src/ports.rs
#[async_trait::async_trait]
pub trait EmployeeStore: Send + Sync + sealed::Sealed {
    async fn hire(&self, tenant_id: TenantId, person_id: PersonId,
        org_id: OrganizationId, draft: EmploymentDraft) -> Result<Employee, IdentityError>;
    async fn terminate(&self, tenant_id: TenantId, employee_id: EmployeeId,
        effective_date: NaiveDate) -> Result<(), IdentityError>;
    async fn active_for(&self, tenant_id: TenantId, person_id: PersonId)
        -> Result<Vec<Employee>, IdentityError>;
    async fn get(&self, tenant_id: TenantId, employee_id: EmployeeId)
        -> Result<Option<Employee>, IdentityError>;
}
```

### CI lanes that must green before phase exit gate

| Lane | Command | Expected |
|---|---|---|
| `dependency-direction` | `oya gate validate lean-a1 --phase P03-identity` | exit 0 |
| `cross-product-refusal` | `oya gate validate lean-a2 --phase P03-identity` | exit 0 |
| `port-location` | `oya gate validate port-location --phase P03-identity` | exit 0 |
| `statelessness` | `oya gate validate statelessness --phase P03-identity` | exit 0 |
| `shardability` | `oya gate validate shardability --phase P03-identity` | exit 0 |

### New BCs registered in this phase

| BC name | Owner µservice | Registration PR |
|---|---|---|
| `users` | `identity` | pending |
| `persons` | `identity` | pending |
| `organizations` | `identity` | pending |
| `employees` | `identity` | pending |
| `employments` | `identity` | pending |
| `sessions` | `identity` | pending |
| `mfa` | `identity` | pending |
| `passkeys` | `identity` | pending |

---

## Grit Claim Symbols

```
crates/oya-identity-users-kernel/src/ports.rs::UserStore
crates/oya-identity-sessions-kernel/src/ports.rs::SessionStore
crates/oya-identity-passkeys-kernel/src/ports.rs::PasskeyStore
crates/oya-identity-mfa-kernel/src/ports.rs::MfaChallengeStore
crates/oya-identity-employees-kernel/src/ports.rs::EmployeeStore
crates/oya-identity-employments-kernel/src/ports.rs::EmploymentStore
migrations/identity/V001__identity_init.sql::identity_schema
contracts/identity.proto::UserRegistered
```

---

## ICM Rationale Fields

```bash
icm store \
  -t context-oyatie \
  -c "Phase P03-identity started; milestone M02-substrate; scope: 36 crates across 8 BCs (users/persons/orgs/employees/employments/sessions/mfa/passkeys); entry gate: M01-P05 complete" \
  -i high \
  -k "M02,P03,phase-start,identity"

icm store \
  -t context-oyatie \
  -c "Phase P03-identity complete; all identity DDL applied; RLS verified; auth flows compile; ADR-0123 two-cookie contract implemented; k6 p99≤200ms; next: P04-audit-chain" \
  -i high \
  -k "M02,P03,phase-complete,identity"
```

---

## References

- Bominal ADRs inherited: ADR-0123 (auth cookie/redirect contract), ADR-0125 (domain naming), ADR-0126 (employment classification 8 classes)
- oyatie ADRs: ADR-0056 (BNF v4.1)
- depends_on: M01-P05
- unblocks: Wave-B phases (hr, medical, connect, payroll — all need identity BCs)
