---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-social-foundation
impl_plan_id: IP-003-user-profile-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-social
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-layer-correctness, oya-governance-dual-context-isolation]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-003: user-profile BC end-to-end (kernel → domain → usecase → api → adapter-postgres → rest → sdk → app)

## Intent

Author the full `user-profile` BC: kernel port traits (`ProfileRepository`,
`CedarSocialPolicy`, `AuditChainClient`), domain entities
(`PersonalProfile`, `ProfessionalProfile`, `Handle`, `VerificationBadge`,
`PersonaContext`), usecase orchestrators (create / update / read / delete),
api types, adapter-postgres implementation with per-tenant RLS + context_kind
CHECK constraint, REST surface, SDK crate, and app composition root.

Lands the `ContextKind` sealed enum per `policy/dual-context-isolation.md`
DCI-01 + DCI-02 invariants.

## ChangeSet boundary

`user-profile` BC across all layers: kernel + domain + usecase + api +
adapter-postgres + rest + sdk + app crates.

## Concrete File Targets

| Path | Action |
|---|---|
| `src/crates/oya-social-user-profile-kernel/src/{ports,entities,context_kind,errors}.rs` | create |
| `src/crates/oya-social-user-profile-domain/src/{personal_profile,professional_profile,handle,verification_badge}.rs` | create |
| `src/crates/oya-social-user-profile-usecase/src/{create,update,read,delete}.rs` | create |
| `src/crates/oya-social-user-profile-api/src/lib.rs` | create — request/response types |
| `src/crates/oya-social-user-profile-adapter-postgres/src/repository.rs` | create |
| `src/crates/oya-social-user-profile-adapter-postgres/migrations/0001_init.sql` | create — RLS + CHECK |
| `src/crates/oya-social-user-profile-rest/src/handlers.rs` | create |
| `src/crates/oya-social-user-profile-sdk/src/client.rs` | create |
| `src/crates/oya-social-user-profile-app/src/main.rs` | create — composition root |
| `tests/dual_context_invariant_profile.rs` | create — UI tests assert cross-context impl-coverage rejected |

## Code Shape

```rust
// kernel/src/context_kind.rs
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContextKind { Personal, Professional }

// kernel/src/ports.rs
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn create(&self, profile: ProfileNew) -> Result<Profile, ProfileError>;
    async fn get_by_handle(&self, tenant_id: &TenantId, handle: &Handle, context: ContextKind)
        -> Result<Option<Profile>, ProfileError>;
    async fn get_by_id(&self, tenant_id: &TenantId, profile_id: &ProfileId)
        -> Result<Option<Profile>, ProfileError>;
    async fn update(&self, tenant_id: &TenantId, profile_id: &ProfileId, patch: ProfilePatch)
        -> Result<Profile, ProfileError>;
}
```

```sql
-- migrations/0001_init.sql
CREATE TABLE social_profiles (
    profile_id        bytea PRIMARY KEY,           -- ULID raw
    tenant_id         text  NOT NULL,
    context_kind      text  NOT NULL CHECK (context_kind IN ('Personal','Professional')),
    handle            text  NOT NULL,
    display_name      text  NOT NULL,
    bio               text,
    avatar_url        text,
    header_url        text,
    verification_badge text  NOT NULL DEFAULT 'none' CHECK (verification_badge IN ('none','blue','organisation','government')),
    is_protected      boolean NOT NULL DEFAULT false,
    federation_opt_in boolean NOT NULL DEFAULT false,
    created_at        timestamptz NOT NULL DEFAULT now(),
    deleted_at        timestamptz,
    UNIQUE (tenant_id, context_kind, handle)
) PARTITION BY HASH (tenant_id);
-- 32 partitions for tenant-id shardability

-- RLS policy
ALTER TABLE social_profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON social_profiles
  USING (tenant_id = current_setting('app.tenant_id'));
```

## Acceptance Gates

```bash
cargo nextest run -p oya-social-user-profile-kernel
cargo nextest run -p oya-social-user-profile-domain
cargo nextest run -p oya-social-user-profile-usecase
cargo nextest run -p oya-social-user-profile-adapter-postgres
cargo nextest run -p oya-social-user-profile-rest
cargo run -p oya-dev-cli -- gate validate port-location --microservice social
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice social
cargo run -p oya-dev-cli -- gate validate dual-context-isolation --microservice social
```

## Test Plan

- Unit tests on each entity invariant (PersonalProfile always Personal; ProfessionalProfile always Professional).
- UI tests: attempting to mix context types fails to compile.
- Integration test: testcontainers Postgres 16; RLS roundtrip (tenant A insert; tenant B read returns 0 rows).
- E2E AC-02: Personal-context profile cannot post under Professional tenant context.

## Halt Conditions

- Any cross-context coercion compiles — bug; fix the type system.
- Any port trait declares I/O dependency — kernel-purity violation.

## Next IP

[`IP-004-follow-graph-bc.md`](IP-004-follow-graph-bc.md)

## References

- `policy/dual-context-isolation.md` DCI-01..DCI-08.
- Bominal ADR-0208; parallel ADR-0135.
