---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-003-professional-profile-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-port-location, oya-governance-layer-correctness, oya-governance-professional-context-isolation]
---

# IP-003: professional-profile BC end-to-end (kernel → domain → usecase → api → adapter-postgres → rest → sdk → app)

## Intent

Author the full `professional-profile` BC: kernel port traits (`ProfileRepository`, `CedarNetworkPolicy`, `AuditChainClient`, `ProfileExportEmitter`), domain entities (`ProfessionalProfile`, `ExperienceEntry`, `EducationEntry`, `SkillEntry`, `Certification`, `Headline`, `Summary`, `VerificationBadge`), usecase orchestrators (create, update, read, delete, export-vcard4, export-jsonresume, export-gdpr-art20), API types, adapter-postgres implementation with per-tenant RLS + `context_kind='Professional'` CHECK + `synchronous_commit = remote_write` (per ADR-NET-0001), REST surface, SDK crate, and app composition root.

Lands the `Professional`-only invariant per `policy/professional-context-isolation.md` PCI-01.

## ChangeSet boundary

`professional-profile` BC across all layers.

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait ProfileRepository: Send + Sync {
    async fn create(&self, profile: ProfileNew) -> Result<ProfessionalProfile, ProfileError>;
    async fn get_by_handle(&self, tenant_id: &TenantId, handle: &Handle)
        -> Result<Option<ProfessionalProfile>, ProfileError>;
    async fn get_by_id(&self, tenant_id: &TenantId, profile_id: &ProfileId)
        -> Result<Option<ProfessionalProfile>, ProfileError>;
    async fn update(&self, tenant_id: &TenantId, profile_id: &ProfileId, patch: ProfilePatch)
        -> Result<ProfessionalProfile, ProfileError>;
}

#[async_trait]
pub trait ProfileExportEmitter: Send + Sync {
    async fn emit_vcard4(&self, profile: &ProfessionalProfile) -> Result<Vec<u8>, ExportError>;
    async fn emit_jsonresume(&self, profile: &ProfessionalProfile) -> Result<Vec<u8>, ExportError>;
    async fn emit_gdpr_art20(&self, profile: &ProfessionalProfile, bundle_signing_key_ref: &KmsKeyRef)
        -> Result<SignedExportBundle, ExportError>;
}
```

```sql
-- migrations/0001_init.sql
CREATE TABLE network_profiles (
    profile_id           bytea PRIMARY KEY,
    tenant_id            text NOT NULL,
    context_kind         text NOT NULL CHECK (context_kind = 'Professional'),
    handle               text NOT NULL,
    display_name         text NOT NULL,
    headline             text,
    summary              text,
    avatar_url           text,
    header_url           text,
    location             jsonb,
    locale               text,
    verification_badge   text NOT NULL DEFAULT 'none' CHECK (verification_badge IN ('none','blue','organisation','government','employer-confirmed')),
    is_open_to_work      boolean NOT NULL DEFAULT false,
    automated_decision_opt_out boolean NOT NULL DEFAULT false,
    minor_protect        boolean NOT NULL DEFAULT false,
    created_at           timestamptz NOT NULL DEFAULT now(),
    deleted_at           timestamptz,
    UNIQUE (tenant_id, handle)
) PARTITION BY HASH (tenant_id);
-- 64 partitions for tenant-id shardability per ADR-NET-0001

ALTER TABLE network_profiles ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON network_profiles
  USING (tenant_id = current_setting('app.tenant_id'));

-- Adjacent tables: experience_entries, education_entries, skill_entries, certifications (FK to profile_id)
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-professional-profile-kernel
cargo nextest run -p oya-network-professional-profile-domain
cargo nextest run -p oya-network-professional-profile-usecase
cargo nextest run -p oya-network-professional-profile-adapter-postgres
cargo nextest run -p oya-network-professional-profile-rest
cargo run -p oya-dev-cli -- gate validate port-location --microservice network
cargo run -p oya-dev-cli -- gate validate layer-correctness --microservice network
cargo run -p oya-dev-cli -- gate validate professional-context-isolation --microservice network
```

## Test Plan

- Unit tests on each entity invariant (Professional-only).
- UI test: attempting to construct a `network::ProfessionalProfile` from a `social::PersonalProfile` MUST fail to compile.
- Integration test: testcontainers Postgres 16; RLS roundtrip (tenant A insert; tenant B read returns 0 rows); `synchronous_commit = remote_write` confirmed via pg_stat_replication.
- Export integration test: vCard 4.0 output validates against RFC 6350 reference parser; JSON Resume against `jsonresume.org/schema/`; GDPR Art. 20 bundle Ed25519 signature verifies.

## Halt Conditions

- Cross-context coercion compiles — bug; fix the type system.
- Any port trait declares I/O dependency — kernel-purity violation.

## Next IP

[`IP-004-professional-graph-and-connection-request-bcs.md`](IP-004-professional-graph-and-connection-request-bcs.md)

## References

- `policy/professional-context-isolation.md` PCI-01..PCI-10.
- ADR-NET-0001 (storage); ADR-NET-0006 (profile portability + export).
- Bominal ADR-0208 (Connect dual-context); parallel ADR-0135.
