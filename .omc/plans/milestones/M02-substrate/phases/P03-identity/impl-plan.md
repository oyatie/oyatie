---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-substrate
phase: P03-identity
impl_plan_id: IP-P03-identity-substrate
status: pending
owner: council-architecture
blocked_by: []
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
---

# IP-P03-identity-substrate: Scaffold 36 Identity crates with full DDL, port traits, auth flows, Cedar, Protobuf

## Intent

Delivers the complete Identity substrate across 8 BCs (users, persons, organizations, employees, employments, sessions, mfa, passkeys) — 32 BC-layer crates + 3 presentation + 1 app = 36 total. Implements Bominal ADR-0123 two-cookie PKCE auth flow, WebAuthn passkey registration/assertion, TOTP/FIDO2 MFA, argon2id password hashing, and Bominal ADR-0125 entity distinctions. Full Postgres DDL with RLS; Cedar policy; Protobuf event schema; k6 load test p99≤200ms.

---

## Concrete File Targets

| Path | Action | Description |
|---|---|---|
| `crates/oya-identity-users-kernel/Cargo.toml` | create | UserStore, AuthChallenger port traits |
| `crates/oya-identity-users-kernel/src/types.rs` | create | UserId, Email, Phone, User, UserDraft, LockReason, ChallengeMethod, ChallengeToken |
| `crates/oya-identity-users-kernel/src/ports.rs` | create | UserStore + AuthChallenger sealed traits |
| `crates/oya-identity-sessions-kernel/Cargo.toml` | create | SessionStore trait |
| `crates/oya-identity-sessions-kernel/src/ports.rs` | create | SessionStore sealed trait |
| `crates/oya-identity-passkeys-kernel/Cargo.toml` | create | PasskeyStore trait + WebAuthnCredential types |
| `crates/oya-identity-passkeys-kernel/src/ports.rs` | create | PasskeyStore sealed trait |
| `crates/oya-identity-mfa-kernel/Cargo.toml` | create | MfaChallengeStore trait |
| `crates/oya-identity-persons-kernel/Cargo.toml` | create | PersonStore trait |
| `crates/oya-identity-organizations-kernel/Cargo.toml` | create | OrganizationStore trait |
| `crates/oya-identity-employees-kernel/Cargo.toml` | create | EmployeeStore trait |
| `crates/oya-identity-employments-kernel/Cargo.toml` | create | EmploymentStore trait |
| `crates/oya-identity-users-domain/src/user.rs` | create | User entity; argon2id hash; locked_at logic |
| `crates/oya-identity-users-application/src/register.rs` | create | RegisterUserUseCase: email uniqueness, argon2id hash, emit event |
| `crates/oya-identity-users-application/src/authenticate.rs` | create | AuthenticateUseCase: argon2id verify, challenge-response, session creation |
| `crates/oya-identity-users-adapter/src/postgres.rs` | create | PgUserStore: sqlx queries against identity.users |
| `crates/oya-identity-sessions-domain/src/session.rs` | create | Session entity; idle_timeout; expiry logic |
| `crates/oya-identity-sessions-application/src/oidc_pkce.rs` | create | OIDC PKCE flow per Bominal ADR-0123; two-cookie contract |
| `crates/oya-identity-sessions-adapter/src/postgres.rs` | create | PgSessionStore |
| `crates/oya-identity-passkeys-domain/src/credential.rs` | create | WebAuthnCredential entity; CBOR attestation parsing |
| `crates/oya-identity-passkeys-application/src/register_credential.rs` | create | RegisterCredentialUseCase: webauthn-rs registration ceremony |
| `crates/oya-identity-passkeys-application/src/verify_assertion.rs` | create | VerifyAssertionUseCase: webauthn-rs authentication ceremony |
| `crates/oya-identity-passkeys-adapter/src/postgres.rs` | create | PgPasskeyStore |
| `crates/oya-identity-mfa-domain/src/challenge.rs` | create | MfaChallenge; TOTP via totp-rs; FIDO2 fallback |
| `crates/oya-identity-mfa-application/src/challenge.rs` | create | IssueChallengeUseCase + VerifyChallengeUseCase |
| `crates/oya-identity-mfa-adapter/src/postgres.rs` | create | PgMfaChallengeStore |
| `crates/oya-identity-persons-domain/src/person.rs` | create | Person entity; legal_name per-jurisdiction struct |
| `crates/oya-identity-persons-application/src/create_person.rs` | create | CreatePersonUseCase |
| `crates/oya-identity-persons-adapter/src/postgres.rs` | create | PgPersonStore |
| `crates/oya-identity-organizations-domain/src/organization.rs` | create | Organization entity; kr_entity_kind + tier enums |
| `crates/oya-identity-organizations-application/src/create_org.rs` | create | CreateOrganizationUseCase |
| `crates/oya-identity-organizations-adapter/src/postgres.rs` | create | PgOrganizationStore |
| `crates/oya-identity-employees-domain/src/employee.rs` | create | Employee entity; active flag; Person×Org junction |
| `crates/oya-identity-employees-application/src/hire.rs` | create | HireEmployeeUseCase |
| `crates/oya-identity-employees-application/src/terminate.rs` | create | TerminateEmploymentUseCase |
| `crates/oya-identity-employees-adapter/src/postgres.rs` | create | PgEmployeeStore |
| `crates/oya-identity-employments-domain/src/employment.rs` | create | Employment entity; classification enum (8 classes per ADR-0126) |
| `crates/oya-identity-employments-application/src/create_employment.rs` | create | CreateEmploymentUseCase |
| `crates/oya-identity-employments-adapter/src/postgres.rs` | create | PgEmploymentStore |
| `crates/oya-identity-rest/src/routes.rs` | create | POST /users/register, POST /auth/login, POST /auth/passkey/register, POST /sessions/refresh, DELETE /sessions/{id} |
| `crates/oya-identity-grpc/src/service.rs` | create | tonic: ValidateSession, GetUser, GetEmployee RPCs |
| `crates/oya-identity-worker/src/session_cleanup.rs` | create | periodic expired session purge |
| `crates/oya-identity-app/src/main.rs` | create | composition root; wire all adapters |
| `migrations/identity/V001__identity_init.sql` | create | full DDL (see below) |
| `contracts/identity/identity.proto` | create | Protobuf event schema |
| `policy/identity/identity.cedar` | create | Cedar policy |
| `tests/load/smoke-identity-sessions.js` | create | k6 smoke test |
| `Cargo.toml` | update | add all 36 identity crates |

---

## Crate Naming

```
NAME: oya-identity-users-kernel
JUSTIFICATION:
- microservice = identity: auth + identity substrate; ADR-0056 v4.1
- bc-tokens = users: auth-principal BC; ADR-0125 User ≠ Person distinction
- layer = kernel: UserStore + AuthChallenger port traits + entity types
- exemptions claimed: none
```

---

## Code Shape

### `migrations/identity/V001__identity_init.sql`

```sql
CREATE SCHEMA IF NOT EXISTS identity;

-- Users (auth principals — NOT the same as Person)
CREATE TABLE identity.users (
    user_id           uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    email             citext      UNIQUE,
    phone             citext      UNIQUE,
    password_hash     text        NULL,           -- argon2id; NULL for passwordless-only
    mfa_enrolled      bool        NOT NULL DEFAULT false,
    mfa_methods       jsonb       NOT NULL DEFAULT '[]',
    passkey_credentials jsonb     NOT NULL DEFAULT '[]',   -- WebAuthn credential list (metadata only)
    locked_at         timestamptz NULL,
    lock_reason       text        NULL,
    last_login_at     timestamptz NULL,
    created_at        timestamptz NOT NULL DEFAULT now(),
    updated_at        timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_users_email ON identity.users (email) WHERE email IS NOT NULL;
CREATE INDEX idx_users_phone ON identity.users (phone) WHERE phone IS NOT NULL;

-- Persons (human records — may exist without User account)
CREATE TABLE identity.persons (
    person_id       uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    legal_name      jsonb       NOT NULL,     -- {"given": "Jason", "family": "Lee", "locale": "ko-KR"}
    dob             date        NULL,
    nationality     text        NULL,
    pillar          text        NOT NULL DEFAULT 'person' CHECK (pillar = 'person'),
    user_id         uuid        NULL UNIQUE REFERENCES identity.users(user_id) ON DELETE SET NULL,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz NOT NULL DEFAULT now()
);

-- Organizations (legal entities inside a tenant — NOT the same as Tenant)
CREATE TABLE identity.organizations (
    organization_id       uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id             uuid    NOT NULL,
    parent_organization_id uuid   NULL REFERENCES identity.organizations(organization_id),
    display_name          text    NOT NULL,
    kr_entity_kind        text    NULL CHECK (kr_entity_kind IN ('개인사업자','법인','비영리법인','공공기관')),
    tier                  text    NULL CHECK (tier IN ('5인미만','SME','중견','대기업','공공')),
    pillar                text    NOT NULL DEFAULT 'org' CHECK (pillar = 'org'),
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.organizations ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.organizations FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.organizations
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_orgs_tenant ON identity.organizations (tenant_id, organization_id);

-- Employees (Person × Organization — active employment junction)
CREATE TABLE identity.employees (
    employee_id     uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       uuid    NOT NULL,
    person_id       uuid    NOT NULL REFERENCES identity.persons(person_id),
    organization_id uuid    NOT NULL REFERENCES identity.organizations(organization_id),
    active          bool    NOT NULL DEFAULT true,
    created_at      timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.employees ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.employees FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.employees
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE UNIQUE INDEX idx_employees_person_org
    ON identity.employees (tenant_id, person_id, organization_id);
CREATE INDEX idx_employees_active
    ON identity.employees (tenant_id, organization_id)
    WHERE active = true;

-- Employments (effective-dated role snapshots per Bominal ADR-0126)
CREATE TABLE identity.employments (
    employment_id       uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           uuid    NOT NULL,
    employee_id         uuid    NOT NULL REFERENCES identity.employees(employee_id),
    classification      text    NOT NULL CHECK (classification IN (
        '정규직','계약직','단시간근로자','파견','도급','프리랜서','인턴','임원'
    )),
    title               text    NOT NULL,
    department          text    NULL,
    manager_employee_id uuid    NULL REFERENCES identity.employees(employee_id),
    fte                 numeric(4,3) NOT NULL DEFAULT 1.000 CHECK (fte > 0 AND fte <= 1.000),
    effective_from      date    NOT NULL,
    effective_to        date    NULL,
    statute_citations   jsonb   NOT NULL DEFAULT '[]',  -- ADR-0190 corpus citations
    created_at          timestamptz NOT NULL DEFAULT now()
);
ALTER TABLE identity.employments ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity.employments FORCE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON identity.employments
    USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);
CREATE INDEX idx_employments_active
    ON identity.employments (tenant_id, employee_id, effective_from DESC)
    WHERE effective_to IS NULL;
CREATE INDEX idx_employments_department
    ON identity.employments (tenant_id, department, effective_from DESC)
    WHERE effective_to IS NULL;

-- Sessions
CREATE TABLE identity.sessions (
    session_id          uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id             uuid    NOT NULL REFERENCES identity.users(user_id),
    tenant_id           uuid    NULL,          -- NULL during cross-tenant SSO handshake
    impersonator_user_id uuid   NULL REFERENCES identity.users(user_id),
    issued_at           timestamptz NOT NULL DEFAULT now(),
    idle_timeout_at     timestamptz NOT NULL,
    expires_at          timestamptz NOT NULL,
    mfa_satisfied_at    timestamptz NULL,
    last_seen_at        timestamptz NOT NULL DEFAULT now(),
    user_agent          text    NULL,
    ip_address          inet    NULL,
    refresh_token_hash  text    NULL          -- argon2id of refresh token
);
CREATE INDEX idx_sessions_user_active
    ON identity.sessions (user_id, expires_at DESC)
    WHERE expires_at > now();
CREATE INDEX idx_sessions_tenant
    ON identity.sessions (tenant_id, expires_at DESC)
    WHERE tenant_id IS NOT NULL AND expires_at > now();

-- MFA challenges
CREATE TABLE identity.mfa_challenges (
    challenge_id    uuid    PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         uuid    NOT NULL REFERENCES identity.users(user_id),
    method          text    NOT NULL CHECK (method IN ('totp','fido2','sms','email')),
    challenge_hash  text    NOT NULL,    -- argon2id of challenge value
    expires_at      timestamptz NOT NULL,
    used_at         timestamptz NULL,
    created_at      timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_mfa_challenges_user
    ON identity.mfa_challenges (user_id, expires_at DESC)
    WHERE used_at IS NULL;

-- Identity outbox
CREATE TABLE identity.outbox (
    outbox_id    uuid        PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id    uuid        NULL,
    topic        text        NOT NULL,
    key          text        NOT NULL,
    payload      jsonb       NOT NULL,
    published_at timestamptz NULL,
    created_at   timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX idx_identity_outbox_unpublished
    ON identity.outbox (created_at)
    WHERE published_at IS NULL;
```

### `contracts/identity/identity.proto`

```proto
syntax = "proto3";
package oyatie.identity.v1;

message UserRegistered {
    string user_id      = 1;
    string email        = 2;
    int64  timestamp_ms = 3;
}

message SessionCreated {
    string session_id   = 1;
    string user_id      = 2;
    string tenant_id    = 3;
    bool   mfa_satisfied = 4;
    int64  expires_at_ms = 5;
    int64  timestamp_ms  = 6;
}

message EmployeeHired {
    string employee_id      = 1;
    string tenant_id        = 2;
    string person_id        = 3;
    string organization_id  = 4;
    string employment_id    = 5;
    string classification   = 6;
    int64  timestamp_ms     = 7;
}

message EmploymentTerminated {
    string employee_id    = 1;
    string tenant_id      = 2;
    string employment_id  = 3;
    string effective_date = 4;   // ISO 8601 date
    int64  timestamp_ms   = 5;
}
```

### `policy/identity/identity.cedar`

```cedar
entity Tenant;
entity User   in [Tenant];
entity Session in [Tenant] = { user_id: String };
entity Employee in [Tenant] = { person_id: String, org_id: String };

action ReadUser    appliesTo { principal: [User], resource: User };
action UpdateUser  appliesTo { principal: [User], resource: User };
action CreateSession appliesTo { principal: [User], resource: Session };
action RevokeSession appliesTo { principal: [User], resource: Session };

// Users can only read/update their own record
permit (principal, action == Action::"ReadUser", resource is User)
    when { principal == resource };

permit (principal, action == Action::"UpdateUser", resource is User)
    when { principal == resource };

// Session: user can only revoke their own sessions
permit (principal, action == Action::"RevokeSession", resource is Session)
    when { resource.user_id == principal.id };
```

### `tests/load/smoke-identity-sessions.js`

```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 50, duration: '60s',
  thresholds: {
    http_req_duration: ['p(99)<200'],
    http_req_failed: ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'http://localhost:8081';

export default function () {
  // Session create
  const loginRes = http.post(`${BASE_URL}/identity/v1/auth/login`, JSON.stringify({
    email: `test+${__VU}@example.com`, password: 'test-password-123',
  }), { headers: { 'Content-Type': 'application/json' } });
  check(loginRes, { 'login 200': (r) => r.status === 200 });
  sleep(0.1);
}
```

---

## Acceptance Gates

```bash
cargo check -p oya-identity-users-kernel --all-features    # exit 0
cargo check -p oya-identity-sessions-adapter --all-features # exit 0
cargo clippy --workspace --all-features -- -D warnings      # exit 0
cargo nextest run --workspace --all-features                # exit 0
psql $DATABASE_URL -f migrations/identity/V001__identity_init.sql  # exit 0
# OIDC PKCE flow test
cargo nextest run -p oya-identity-sessions-application --test oidc_pkce_flow  # exit 0
# Passkey round-trip
cargo nextest run -p oya-identity-passkeys-application --test webauthn_round_trip  # exit 0
# Load test
k6 run tests/load/smoke-identity-sessions.js --env BASE_URL=http://localhost:8081
```

---

## Test Plan

### Unit tests

| Test name | What it verifies |
|---|---|
| `test_argon2id_hash_verify` | Password hash + verify round-trip |
| `test_user_lock_unlock` | locked_at set/cleared; locked user cannot authenticate |
| `test_session_expiry_check` | Session expired → validate returns Err |
| `test_employment_classification_enum` | All 8 ADR-0126 classifications compile |
| `test_person_legal_name_structured` | legal_name JSON deserialized correctly |
| `test_org_kr_entity_kind_enum` | Korean entity kind values |
| `test_employee_person_org_junction` | hire + terminate lifecycle |

### Integration tests

| Test name | What it verifies |
|---|---|
| `integration_register_login_session` | Full register → login → session create → validate |
| `integration_passkey_registration_assertion` | WebAuthn registration ceremony + assertion |
| `integration_mfa_totp_challenge` | TOTP challenge issue → verify → session MFA satisfied |
| `integration_rls_org_isolation` | Organization query returns only tenant's orgs |

---

## Clean Architecture Compliance

### Dependency direction check

| Crate | Layer | Imports (layers only) | Forbidden imports |
|---|---|---|---|
| `oya-identity-users-kernel` | `kernel` | nothing project-internal | all layers |
| `oya-identity-users-domain` | `domain` | `users-kernel` | `application`, `adapter`, presentation |
| `oya-identity-users-application` | `application` | `users-domain`, `users-kernel` | `adapter`, presentation |
| `oya-identity-users-adapter` | `adapter` | `users-application`, `users-domain`, `users-kernel` | presentation |
| `oya-identity-rest` | `rest` | all `*-application`, `*-kernel` | direct adapter |
| `oya-identity-app` | `app` | all | none |

---

## Load Test

```bash
k6 run tests/load/smoke-identity-sessions.js --env BASE_URL=http://localhost:8081
# Pass: p99 ≤200ms; 0 errors

echo "POST http://staging.identity/identity/v1/auth/login" \
  | vegeta attack -rate=500/s -duration=60s \
  | vegeta report
# Pass: p99 ≤200ms; success_rate=100%
```

---

## Grit Symbol-Locks

```bash
grit claim \
  --agent m02-wave-a-executor \
  --intent "IP-P03-identity-substrate: 36 crates + DDL + auth flows + load test" \
  --ttl 7200 \
  crates/oya-identity-users-kernel/src/ports.rs::UserStore \
  crates/oya-identity-sessions-kernel/src/ports.rs::SessionStore \
  crates/oya-identity-passkeys-kernel/src/ports.rs::PasskeyStore \
  migrations/identity/V001__identity_init.sql::identity_schema
```

---

## ICM Rows to Emit

```bash
icm store \
  -t context-oyatie \
  -c "IP-P03-identity-substrate merged; 36 crates; ADR-0123 PKCE flow; ADR-0125 distinctions; ADR-0126 8 classifications; DDL+RLS applied; k6 p99≤200ms; next: P04-audit-chain/impl-plan" \
  -i high \
  -k "M02,P03,IP-P03,identity"
```

---

## Next IP Pointer

`phases/P04-audit-chain/impl-plan.md`

---

## Cross-References

- Phase spec: `phase-spec.md`
- Schema foundation: `.omc/plans/M02-substrate-schema-foundation.md §3`
- Bominal ADR-0123 (auth), ADR-0125 (naming), ADR-0126 (employment classification)
