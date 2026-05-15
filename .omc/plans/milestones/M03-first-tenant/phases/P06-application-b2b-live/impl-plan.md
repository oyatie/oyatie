---
purpose: "P06 — Application B2B Shell Live: Implementation Plan"
---

# P06 — Application B2B Shell Live: Implementation Plan

## Metadata
- phase: P06-application-b2b-live
- milestone: M03-first-tenant
- depends_on: [P01-hr, P02-payroll, P03-accounting, P04-connect-pro-mail, P05-connect-pro-messenger]
- parallel_with: []
- grit_claim_symbols: [m03.p06.application.shell, m03.p06.application.oidc, m03.p06.application.enablement, m03.p06.application.billing]
- icm_topics: [context-oyatie, decisions-oyatie, errors-resolved]
- icm_keywords: [application,b2b,oidc,saml,leptos,ssr,spa,product-enablement,onboarding]

---

## 0. Crate Inventory

```
crates/
  oya-application-shell-kernel/          # port traits: SessionStore, ProductRegistry, TenantActivationPort
  oya-application-shell-domain/          # TenantSession, ProductEnablement, TenantOnboarding aggregates
  oya-application-shell-application/     # use-cases: ActivateTenant, EnableProduct, ProvisionTenantUser
  oya-application-shell-adapter/         # OidcAdapter, SamlAdapter, ProductRegistryAdapter
  oya-application-shell-rest/            # Axum handlers: /auth/*, /api/shell/*
  oya-application-shell-worker/          # EmployeeHired subscriber → ProvisionTenantUser
  oya-application-shell-app/             # composition root: DB pool, OIDC client, Kafka consumer
  oya-application-leptos/                # Leptos SSR+WASM web shell crate
```

---

## 1. Full DDL

```sql
-- migrations/20260513_000001_application_shell.sql

CREATE SCHEMA IF NOT EXISTS application;

-- ── Enums ─────────────────────────────────────────────────────────────
CREATE TYPE application.product_key AS ENUM (
  'hr',
  'payroll',
  'accounting',
  'connect_pro_mail',
  'connect_pro_messenger',
  'workflow_studio'
);

CREATE TYPE application.onboarding_status AS ENUM (
  'pending',
  'identity_configured',
  'products_selected',
  'users_provisioned',
  'active',
  'suspended'
);

CREATE TYPE application.sso_protocol AS ENUM (
  'oidc',
  'saml2'
);

CREATE TYPE application.billing_plan AS ENUM (
  'starter',
  'growth',
  'enterprise'
);

-- ── tenant_shell (one row per tenant) ────────────────────────────────
CREATE TABLE application.tenant_shell (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL UNIQUE,
  display_name        TEXT NOT NULL,
  slug                TEXT NOT NULL UNIQUE CHECK (slug ~ '^[a-z0-9-]{3,63}$'),
  billing_plan        application.billing_plan NOT NULL DEFAULT 'starter',
  onboarding_status   application.onboarding_status NOT NULL DEFAULT 'pending',
  activated_at        TIMESTAMPTZ,
  suspended_at        TIMESTAMPTZ,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('application.tenant_shell', 'tenant_id');
ALTER TABLE application.tenant_shell ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.tenant_shell
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_tenant_shell_slug ON application.tenant_shell (slug);
CREATE INDEX idx_tenant_shell_status ON application.tenant_shell (onboarding_status)
  WHERE onboarding_status != 'active';

-- ── sso_configurations ────────────────────────────────────────────────
CREATE TABLE application.sso_configurations (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL,
  protocol            application.sso_protocol NOT NULL,
  -- OIDC fields
  issuer_url          TEXT,
  client_id           TEXT,
  client_secret_ref   TEXT,           -- SecretReference path in OpenBao
  -- SAML2 fields
  idp_metadata_url    TEXT,
  sp_entity_id        TEXT,
  idp_certificate_ref TEXT,           -- SecretReference path in OpenBao
  -- common
  attribute_map       JSONB NOT NULL DEFAULT '{}',
  is_primary          BOOLEAN NOT NULL DEFAULT false,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  CONSTRAINT sso_config_protocol_fields CHECK (
    (protocol = 'oidc' AND issuer_url IS NOT NULL AND client_id IS NOT NULL)
    OR
    (protocol = 'saml2' AND idp_metadata_url IS NOT NULL AND sp_entity_id IS NOT NULL)
  )
);

SELECT create_distributed_table('application.sso_configurations', 'tenant_id');
ALTER TABLE application.sso_configurations ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.sso_configurations
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE UNIQUE INDEX idx_sso_primary ON application.sso_configurations (tenant_id)
  WHERE is_primary = true;

-- ── product_enablements ───────────────────────────────────────────────
CREATE TABLE application.product_enablements (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL,
  product_key         application.product_key NOT NULL,
  enabled_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  enabled_by          UUID NOT NULL,   -- user_id of TenantAdmin who enabled
  disabled_at         TIMESTAMPTZ,
  config              JSONB NOT NULL DEFAULT '{}',
  UNIQUE (tenant_id, product_key)
);

SELECT create_distributed_table('application.product_enablements', 'tenant_id');
ALTER TABLE application.product_enablements ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.product_enablements
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_product_enabled ON application.product_enablements (tenant_id, product_key)
  WHERE disabled_at IS NULL;

-- ── tenant_users ──────────────────────────────────────────────────────
-- Shell-level user record; downstream from HR EmployeeHired event
CREATE TABLE application.tenant_users (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL,
  external_sub        TEXT,           -- OIDC/SAML subject claim
  employee_id         UUID,           -- nullable until HR link confirmed
  email               TEXT NOT NULL,
  display_name        TEXT NOT NULL,
  is_tenant_admin     BOOLEAN NOT NULL DEFAULT false,
  last_login_at       TIMESTAMPTZ,
  provisioned_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  deprovisioned_at    TIMESTAMPTZ,
  UNIQUE (tenant_id, email)
);

SELECT create_distributed_table('application.tenant_users', 'tenant_id');
ALTER TABLE application.tenant_users ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.tenant_users
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_tenant_users_sub ON application.tenant_users (tenant_id, external_sub)
  WHERE external_sub IS NOT NULL;
CREATE INDEX idx_tenant_users_employee ON application.tenant_users (tenant_id, employee_id)
  WHERE employee_id IS NOT NULL;

-- ── sessions (two-cookie pattern per ADR-0123) ────────────────────────
CREATE TABLE application.sessions (
  id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id           UUID NOT NULL,
  user_id             UUID NOT NULL,
  access_token_hash   BYTEA NOT NULL,   -- SHA-256 of short-lived access token
  refresh_token_hash  BYTEA NOT NULL,   -- SHA-256 of longer-lived refresh token
  pkce_verifier       TEXT,             -- cleared after code exchange
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
  expires_at          TIMESTAMPTZ NOT NULL,
  revoked_at          TIMESTAMPTZ
);

SELECT create_distributed_table('application.sessions', 'tenant_id');
ALTER TABLE application.sessions ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.sessions
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

CREATE INDEX idx_sessions_access ON application.sessions (access_token_hash)
  WHERE revoked_at IS NULL AND expires_at > now();
CREATE INDEX idx_sessions_expiry ON application.sessions (expires_at)
  WHERE revoked_at IS NULL;

-- ── shell_audit_log (append-only) ────────────────────────────────────
CREATE TABLE application.shell_audit_log (
  id                  BIGSERIAL,
  tenant_id           UUID NOT NULL,
  actor_user_id       UUID,
  action              TEXT NOT NULL,
  resource_type       TEXT NOT NULL,
  resource_id         UUID,
  payload             JSONB,
  event_hash          BYTEA NOT NULL,   -- Ed25519 sealed per ADR-0028
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('application.shell_audit_log', 'tenant_id');
ALTER TABLE application.shell_audit_log ENABLE ROW LEVEL SECURITY;
FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON application.shell_audit_log
  USING (tenant_id = current_setting('oyatie.tenant_id')::uuid);

-- Prevent mutation of audit log
CREATE RULE no_update_shell_audit AS ON UPDATE TO application.shell_audit_log DO INSTEAD NOTHING;
CREATE RULE no_delete_shell_audit AS ON DELETE TO application.shell_audit_log DO INSTEAD NOTHING;

-- ── outbox ────────────────────────────────────────────────────────────
CREATE TABLE application.application_outbox (
  id                  BIGSERIAL PRIMARY KEY,
  tenant_id           UUID NOT NULL,
  aggregate_type      TEXT NOT NULL,
  aggregate_id        UUID NOT NULL,
  event_type          TEXT NOT NULL,
  payload             JSONB NOT NULL,
  kafka_topic         TEXT NOT NULL,
  published_at        TIMESTAMPTZ,
  created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

SELECT create_distributed_table('application.application_outbox', 'tenant_id');
CREATE INDEX idx_application_outbox_unpublished
  ON application.application_outbox (tenant_id, created_at)
  WHERE published_at IS NULL;
```

---

## 2. Kernel Port Traits

```rust
// crates/oya-application-shell-kernel/src/ports.rs

use uuid::Uuid;
use async_trait::async_trait;
use crate::sealed;

// ── TenantShellRepository ─────────────────────────────────────────────

pub struct TenantShell {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub slug: String,
    pub billing_plan: BillingPlan,
    pub onboarding_status: OnboardingStatus,
    pub activated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BillingPlan { Starter, Growth, Enterprise }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnboardingStatus {
    Pending,
    IdentityConfigured,
    ProductsSelected,
    UsersProvisioned,
    Active,
    Suspended,
}

#[async_trait]
pub trait TenantShellRepository: Send + Sync + sealed::Sealed {
    async fn find_by_tenant_id(&self, tenant_id: Uuid)
        -> Result<Option<TenantShell>, ShellRepoError>;
    async fn find_by_slug(&self, slug: &str)
        -> Result<Option<TenantShell>, ShellRepoError>;
    async fn upsert(&self, shell: &TenantShell) -> Result<(), ShellRepoError>;
}

// ── SessionStore ──────────────────────────────────────────────────────

pub struct SessionRecord {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub access_token_hash: [u8; 32],
    pub refresh_token_hash: [u8; 32],
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[async_trait]
pub trait SessionStore: Send + Sync + sealed::Sealed {
    async fn create(&self, session: &SessionRecord) -> Result<(), SessionStoreError>;
    async fn find_by_access_hash(&self, hash: &[u8; 32])
        -> Result<Option<SessionRecord>, SessionStoreError>;
    async fn revoke(&self, session_id: Uuid) -> Result<(), SessionStoreError>;
    async fn purge_expired(&self) -> Result<u64, SessionStoreError>;
}

// ── ProductRegistry ───────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProductKey {
    Hr,
    Payroll,
    Accounting,
    ConnectProMail,
    ConnectProMessenger,
    WorkflowStudio,
}

pub struct ProductEnablement {
    pub tenant_id: Uuid,
    pub product_key: ProductKey,
    pub enabled_at: chrono::DateTime<chrono::Utc>,
    pub enabled_by: Uuid,
    pub config: serde_json::Value,
}

#[async_trait]
pub trait ProductRegistry: Send + Sync + sealed::Sealed {
    async fn enabled_products(&self, tenant_id: Uuid)
        -> Result<Vec<ProductEnablement>, RegistryError>;
    async fn enable(
        &self,
        tenant_id: Uuid,
        product: ProductKey,
        enabled_by: Uuid,
        config: serde_json::Value,
    ) -> Result<(), RegistryError>;
    async fn disable(&self, tenant_id: Uuid, product: ProductKey)
        -> Result<(), RegistryError>;
}

// ── TenantUserRepository ──────────────────────────────────────────────

pub struct TenantUser {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub external_sub: Option<String>,
    pub employee_id: Option<Uuid>,
    pub email: String,
    pub display_name: String,
    pub is_tenant_admin: bool,
}

#[async_trait]
pub trait TenantUserRepository: Send + Sync + sealed::Sealed {
    async fn find_by_sub(&self, tenant_id: Uuid, sub: &str)
        -> Result<Option<TenantUser>, UserRepoError>;
    async fn find_by_employee_id(&self, tenant_id: Uuid, employee_id: Uuid)
        -> Result<Option<TenantUser>, UserRepoError>;
    async fn upsert(&self, user: &TenantUser) -> Result<(), UserRepoError>;
    async fn deprovision(&self, id: Uuid) -> Result<(), UserRepoError>;
}

// ── SsoConfigRepository ───────────────────────────────────────────────

pub struct OidcConfig {
    pub issuer_url: String,
    pub client_id: String,
    pub client_secret_ref: String,  // OpenBao path
    pub attribute_map: std::collections::HashMap<String, String>,
}

pub struct Saml2Config {
    pub idp_metadata_url: String,
    pub sp_entity_id: String,
    pub idp_certificate_ref: String, // OpenBao path
    pub attribute_map: std::collections::HashMap<String, String>,
}

pub enum SsoConfig {
    Oidc(OidcConfig),
    Saml2(Saml2Config),
}

#[async_trait]
pub trait SsoConfigRepository: Send + Sync + sealed::Sealed {
    async fn primary_config(&self, tenant_id: Uuid)
        -> Result<Option<SsoConfig>, SsoConfigError>;
    async fn upsert_oidc(&self, tenant_id: Uuid, config: OidcConfig)
        -> Result<(), SsoConfigError>;
    async fn upsert_saml2(&self, tenant_id: Uuid, config: Saml2Config)
        -> Result<(), SsoConfigError>;
}

// ── Errors ────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum ShellRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("slug already taken: {slug}")]
    SlugConflict { slug: String },
}

#[derive(Debug, thiserror::Error)]
pub enum SessionStoreError {
    #[error("database error: {0}")]
    Database(String),
    #[error("session not found")]
    NotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("database error: {0}")]
    Database(String),
    #[error("product already enabled: {0:?}")]
    AlreadyEnabled(ProductKey),
    #[error("product not enabled: {0:?}")]
    NotEnabled(ProductKey),
}

#[derive(Debug, thiserror::Error)]
pub enum UserRepoError {
    #[error("database error: {0}")]
    Database(String),
    #[error("email already registered: {0}")]
    EmailConflict(String),
}

#[derive(Debug, thiserror::Error)]
pub enum SsoConfigError {
    #[error("database error: {0}")]
    Database(String),
    #[error("secret reference resolution failed: {path}")]
    SecretResolution { path: String },
}
```

---

## 3. Domain Aggregates

```rust
// crates/oya-application-shell-domain/src/tenant_onboarding.rs

use uuid::Uuid;
use crate::kernel::{TenantShell, OnboardingStatus, ProductKey};

#[derive(Debug)]
pub struct TenantOnboarding {
    shell: TenantShell,
    selected_products: Vec<ProductKey>,
}

#[derive(Debug, thiserror::Error)]
pub enum OnboardingError {
    #[error("onboarding already complete")]
    AlreadyActive,
    #[error("identity provider not configured before products can be selected")]
    IdentityNotConfigured,
    #[error("no products selected")]
    NoProductsSelected,
    #[error("activation requires users to be provisioned")]
    UsersNotProvisioned,
}

impl TenantOnboarding {
    pub fn new(shell: TenantShell) -> Self {
        Self { shell, selected_products: vec![] }
    }

    pub fn configure_identity(&mut self) -> Result<(), OnboardingError> {
        if self.shell.onboarding_status == OnboardingStatus::Active {
            return Err(OnboardingError::AlreadyActive);
        }
        self.shell.onboarding_status = OnboardingStatus::IdentityConfigured;
        Ok(())
    }

    pub fn select_products(
        &mut self,
        products: Vec<ProductKey>,
    ) -> Result<(), OnboardingError> {
        if self.shell.onboarding_status == OnboardingStatus::Pending {
            return Err(OnboardingError::IdentityNotConfigured);
        }
        if products.is_empty() {
            return Err(OnboardingError::NoProductsSelected);
        }
        self.selected_products = products;
        self.shell.onboarding_status = OnboardingStatus::ProductsSelected;
        Ok(())
    }

    pub fn mark_users_provisioned(&mut self) -> Result<(), OnboardingError> {
        self.shell.onboarding_status = OnboardingStatus::UsersProvisioned;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<chrono::DateTime<chrono::Utc>, OnboardingError> {
        if self.shell.onboarding_status != OnboardingStatus::UsersProvisioned {
            return Err(OnboardingError::UsersNotProvisioned);
        }
        let now = chrono::Utc::now();
        self.shell.onboarding_status = OnboardingStatus::Active;
        self.shell.activated_at = Some(now);
        Ok(now)
    }

    pub fn selected_products(&self) -> &[ProductKey] {
        &self.selected_products
    }

    pub fn shell(&self) -> &TenantShell {
        &self.shell
    }
}
```

---

## 4. Application Use-Cases

```rust
// crates/oya-application-shell-application/src/use_cases.rs

use uuid::Uuid;
use crate::kernel::{
    TenantShellRepository, ProductRegistry, TenantUserRepository,
    SsoConfigRepository, ProductKey, TenantUser,
};

// ── ActivateTenant ────────────────────────────────────────────────────

pub struct ActivateTenantCommand {
    pub tenant_id: Uuid,
    pub slug: String,
    pub billing_plan: crate::kernel::BillingPlan,
    pub display_name: String,
    pub admin_email: String,
    pub admin_name: String,
}

pub struct ActivateTenantOutput {
    pub tenant_shell_id: Uuid,
    pub admin_user_id: Uuid,
    pub activated_at: chrono::DateTime<chrono::Utc>,
}

pub struct ActivateTenantUseCase<S, U> {
    shell_repo: S,
    user_repo: U,
}

impl<S, U> ActivateTenantUseCase<S, U>
where
    S: TenantShellRepository,
    U: TenantUserRepository,
{
    pub fn new(shell_repo: S, user_repo: U) -> Self {
        Self { shell_repo, user_repo }
    }

    pub async fn execute(
        &self,
        cmd: ActivateTenantCommand,
    ) -> Result<ActivateTenantOutput, ActivateTenantError> {
        // Guard: slug uniqueness is enforced by DB UNIQUE constraint.
        // Check tenant_id not already active.
        if let Some(existing) = self.shell_repo.find_by_tenant_id(cmd.tenant_id).await? {
            if existing.onboarding_status == crate::kernel::OnboardingStatus::Active {
                return Err(ActivateTenantError::AlreadyActive);
            }
        }

        let now = chrono::Utc::now();
        let shell_id = Uuid::new_v4();
        let shell = crate::kernel::TenantShell {
            id: shell_id,
            tenant_id: cmd.tenant_id,
            slug: cmd.slug,
            billing_plan: cmd.billing_plan,
            onboarding_status: crate::kernel::OnboardingStatus::Active,
            activated_at: Some(now),
        };
        self.shell_repo.upsert(&shell).await?;

        let admin_id = Uuid::new_v4();
        let admin_user = TenantUser {
            id: admin_id,
            tenant_id: cmd.tenant_id,
            external_sub: None,
            employee_id: None,
            email: cmd.admin_email,
            display_name: cmd.admin_name,
            is_tenant_admin: true,
        };
        self.user_repo.upsert(&admin_user).await?;

        Ok(ActivateTenantOutput {
            tenant_shell_id: shell_id,
            admin_user_id: admin_id,
            activated_at: now,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActivateTenantError {
    #[error("tenant is already active")]
    AlreadyActive,
    #[error("shell repo: {0}")]
    Shell(#[from] crate::kernel::ShellRepoError),
    #[error("user repo: {0}")]
    User(#[from] crate::kernel::UserRepoError),
}

// ── EnableProduct ─────────────────────────────────────────────────────

pub struct EnableProductCommand {
    pub tenant_id: Uuid,
    pub product_key: ProductKey,
    pub enabled_by: Uuid,
    pub config: serde_json::Value,
}

pub struct EnableProductUseCase<P> {
    registry: P,
}

impl<P: ProductRegistry> EnableProductUseCase<P> {
    pub fn new(registry: P) -> Self {
        Self { registry }
    }

    pub async fn execute(
        &self,
        cmd: EnableProductCommand,
    ) -> Result<(), EnableProductError> {
        self.registry
            .enable(cmd.tenant_id, cmd.product_key, cmd.enabled_by, cmd.config)
            .await
            .map_err(EnableProductError::Registry)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EnableProductError {
    #[error("registry: {0}")]
    Registry(#[from] crate::kernel::RegistryError),
}

// ── ProvisionTenantUser (from EmployeeHired Workflow event) ───────────

pub struct ProvisionTenantUserCommand {
    pub tenant_id: Uuid,
    pub employee_id: Uuid,
    pub email: String,
    pub display_name: String,
}

pub struct ProvisionTenantUserUseCase<U> {
    user_repo: U,
}

impl<U: TenantUserRepository> ProvisionTenantUserUseCase<U> {
    pub fn new(user_repo: U) -> Self {
        Self { user_repo }
    }

    pub async fn execute(
        &self,
        cmd: ProvisionTenantUserCommand,
    ) -> Result<Uuid, ProvisionTenantUserError> {
        // Idempotent: if already provisioned by employee_id, return existing.
        if let Some(existing) = self
            .user_repo
            .find_by_employee_id(cmd.tenant_id, cmd.employee_id)
            .await?
        {
            return Ok(existing.id);
        }
        let user_id = Uuid::new_v4();
        let user = TenantUser {
            id: user_id,
            tenant_id: cmd.tenant_id,
            external_sub: None,
            employee_id: Some(cmd.employee_id),
            email: cmd.email,
            display_name: cmd.display_name,
            is_tenant_admin: false,
        };
        self.user_repo.upsert(&user).await?;
        Ok(user_id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ProvisionTenantUserError {
    #[error("user repo: {0}")]
    Repo(#[from] crate::kernel::UserRepoError),
}
```

---

## 5. Adapter Scaffolds

```rust
// crates/oya-application-shell-adapter/src/postgres_shell_repo.rs

use sqlx::PgPool;
use uuid::Uuid;
use crate::kernel::{TenantShell, TenantShellRepository, ShellRepoError, OnboardingStatus, BillingPlan};

pub struct PostgresShellRepository {
    pool: PgPool,
}

impl PostgresShellRepository {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
}

#[async_trait::async_trait]
impl TenantShellRepository for PostgresShellRepository {
    async fn find_by_tenant_id(&self, tenant_id: Uuid)
        -> Result<Option<TenantShell>, ShellRepoError>
    {
        let row = sqlx::query!(
            r#"SELECT id, tenant_id, slug,
                      billing_plan::text AS billing_plan,
                      onboarding_status::text AS onboarding_status,
                      activated_at
               FROM application.tenant_shell
               WHERE tenant_id = $1"#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ShellRepoError::Database(e.to_string()))?;

        Ok(row.map(|r| TenantShell {
            id: r.id,
            tenant_id: r.tenant_id,
            slug: r.slug,
            billing_plan: billing_plan_from_str(&r.billing_plan.unwrap_or_default()),
            onboarding_status: onboarding_status_from_str(&r.onboarding_status.unwrap_or_default()),
            activated_at: r.activated_at,
        }))
    }

    async fn find_by_slug(&self, slug: &str)
        -> Result<Option<TenantShell>, ShellRepoError>
    {
        // same pattern; omitted for brevity
        todo!("find_by_slug")
    }

    async fn upsert(&self, shell: &TenantShell) -> Result<(), ShellRepoError> {
        sqlx::query!(
            r#"INSERT INTO application.tenant_shell
               (id, tenant_id, slug, billing_plan, onboarding_status, activated_at, updated_at)
               VALUES ($1, $2, $3, $4::application.billing_plan,
                       $5::application.onboarding_status, $6, now())
               ON CONFLICT (tenant_id)
               DO UPDATE SET slug = EXCLUDED.slug,
                             billing_plan = EXCLUDED.billing_plan,
                             onboarding_status = EXCLUDED.onboarding_status,
                             activated_at = EXCLUDED.activated_at,
                             updated_at = now()"#,
            shell.id,
            shell.tenant_id,
            shell.slug,
            billing_plan_to_str(&shell.billing_plan),
            onboarding_status_to_str(&shell.onboarding_status),
            shell.activated_at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("unique") && e.to_string().contains("slug") {
                ShellRepoError::SlugConflict { slug: shell.slug.clone() }
            } else {
                ShellRepoError::Database(e.to_string())
            }
        })?;
        Ok(())
    }
}

// ── OIDC Adapter (two-cookie + PKCE — ADR-0123) ───────────────────────

// crates/oya-application-shell-adapter/src/oidc_adapter.rs

use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, IssuerUrl, RedirectUrl,
    PkceCodeChallenge, PkceCodeVerifier, CsrfToken, Nonce,
};
use crate::kernel::{SessionStore, TenantUserRepository, SsoConfigRepository, OidcConfig};

pub struct OidcAdapter<S, U, C> {
    session_store: S,
    user_repo: U,
    sso_config_repo: C,
}

pub struct OidcCallbackParams {
    pub code: AuthorizationCode,
    pub state: CsrfToken,
    pub tenant_id: uuid::Uuid,
}

pub struct OidcCallbackOutput {
    pub user_id: uuid::Uuid,
    pub access_token_cookie: String,
    pub refresh_token_cookie: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OidcError {
    #[error("state mismatch (CSRF)")]
    StateMismatch,
    #[error("token exchange failed: {0}")]
    TokenExchange(String),
    #[error("subject claim missing")]
    MissingSubject,
    #[error("user provisioning failed: {0}")]
    Provisioning(String),
    #[error("sso config: {0}")]
    SsoConfig(String),
}

// Full PKCE + two-cookie implementation uses openidconnect crate;
// cookie names: __Host-oyatie-session (httponly,secure,samesite=strict)
//               __Host-oyatie-refresh (httponly,secure,samesite=strict,path=/api/auth/refresh)

// ── EmployeeHired Kafka Worker ────────────────────────────────────────

// crates/oya-application-shell-worker/src/employee_hired_handler.rs

use oya_workflow_ontology_adapter::events::EmployeeHiredEvent;

pub struct EmployeeHiredHandler<U> {
    provision_use_case: crate::application::ProvisionTenantUserUseCase<U>,
}

impl<U: crate::kernel::TenantUserRepository> EmployeeHiredHandler<U> {
    pub async fn handle(
        &self,
        event: EmployeeHiredEvent,
    ) -> Result<(), crate::application::ProvisionTenantUserError> {
        self.provision_use_case.execute(
            crate::application::ProvisionTenantUserCommand {
                tenant_id: event.tenant_id,
                employee_id: event.employee_id,
                email: event.work_email,
                display_name: event.full_name,
            }
        ).await?;
        Ok(())
    }
}
```

---

## 6. Cedar Policy Fragments

```cedar
// policies/application-shell.cedar

// TenantAdmin can read and update their own tenant shell
permit (
  principal in Role::"TenantAdmin",
  action in [Action::"ReadTenantShell", Action::"UpdateTenantShell",
             Action::"EnableProduct", Action::"DisableProduct",
             Action::"ConfigureSso"],
  resource
)
when {
  principal.tenant_id == resource.tenant_id
};

// TenantAdmin can provision/deprovision users within own tenant
permit (
  principal in Role::"TenantAdmin",
  action in [Action::"ProvisionUser", Action::"DeprovisionUser"],
  resource in Resource::"TenantUser"
)
when {
  principal.tenant_id == resource.tenant_id
};

// TenantUser can read own profile only
permit (
  principal in Role::"TenantUser",
  action == Action::"ReadOwnProfile",
  resource in Resource::"TenantUser"
)
when {
  principal.user_id == resource.id
};

// Auditor: read-only shell audit log for own tenant
permit (
  principal in Role::"Auditor",
  action == Action::"ReadShellAuditLog",
  resource in Resource::"ShellAuditLog"
)
when {
  principal.tenant_id == resource.tenant_id
};

// Forbid cross-tenant access absolutely
forbid (principal, action, resource)
when {
  principal has tenant_id &&
  resource has tenant_id &&
  principal.tenant_id != resource.tenant_id
};

// Platform operator can read any tenant shell (Oyatie support)
permit (
  principal in Role::"PlatformOperator",
  action == Action::"ReadTenantShell",
  resource
);
```

---

## 7. Protobuf Event Schemas + Kafka Topics

```protobuf
// proto/application/v1/events.proto
syntax = "proto3";
package application.v1;

import "google/protobuf/timestamp.proto";

// Kafka topic: oyatie.application.tenant-activated.v1
message TenantActivated {
  string tenant_id       = 1;
  string slug            = 2;
  string billing_plan    = 3;
  string admin_user_id   = 4;
  google.protobuf.Timestamp activated_at = 5;
  string event_id        = 6;
}

// Kafka topic: oyatie.application.product-enabled.v1
message ProductEnabled {
  string tenant_id    = 1;
  string product_key  = 2;
  string enabled_by   = 3;
  google.protobuf.Timestamp enabled_at = 4;
  string event_id     = 5;
}

// Kafka topic: oyatie.application.user-provisioned.v1
message TenantUserProvisioned {
  string tenant_id    = 1;
  string user_id      = 2;
  string employee_id  = 3;   // empty string if not linked
  string email        = 4;
  bool   is_admin     = 5;
  google.protobuf.Timestamp provisioned_at = 6;
  string event_id     = 7;
}

// Kafka topic: oyatie.application.sso-configured.v1
message SsoConfigured {
  string tenant_id = 1;
  string protocol  = 2;   // "oidc" | "saml2"
  google.protobuf.Timestamp configured_at = 3;
  string event_id  = 4;
}
```

---

## 8. OpenAPI Contract (Axum REST)

```yaml
# openapi/application-shell.yaml  (condensed)
openapi: "3.1.0"
info:
  title: Application Shell API
  version: "1.0.0"
paths:
  /api/shell/tenants/{tenantId}/activate:
    post:
      operationId: activateTenant
      summary: Activate a new tenant (sub-5-min onboarding)
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/ActivateTenantRequest'
      responses:
        "201": { description: Tenant activated }
        "409": { description: Tenant already active or slug conflict }

  /api/shell/tenants/{tenantId}/products:
    post:
      operationId: enableProduct
      summary: Enable a product for a tenant
    get:
      operationId: listEnabledProducts
      summary: List enabled products for a tenant

  /api/shell/tenants/{tenantId}/sso:
    put:
      operationId: configureSso
      summary: Configure OIDC or SAML2 identity provider

  /api/shell/tenants/{tenantId}/users:
    get:
      operationId: listTenantUsers
    post:
      operationId: provisionUser

  /auth/oidc/authorize:
    get:
      operationId: oidcAuthorize
      summary: Initiate PKCE OIDC authorization flow
  /auth/oidc/callback:
    get:
      operationId: oidcCallback
      summary: Handle OIDC authorization code callback
  /auth/saml/acs:
    post:
      operationId: samlAcs
      summary: SAML2 Assertion Consumer Service endpoint
  /auth/refresh:
    post:
      operationId: refreshSession
      summary: Rotate access token using refresh cookie
  /auth/logout:
    post:
      operationId: logout
      summary: Revoke session and clear cookies
```

---

## 9. Leptos Web Shell (oya-application-leptos)

```rust
// crates/oya-application-leptos/src/app.rs
// SSR pre-auth pages → hydrated SPA post-auth

use leptos::*;
use leptos_router::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                // Pre-auth (SSR rendered)
                <Route path="/login" view=LoginPage />
                <Route path="/auth/callback" view=OidcCallbackPage />
                // Post-auth SPA shell
                <Route path="/" view=ShellLayout>
                    <Route path="/dashboard" view=DashboardPage />
                    <Route path="/settings/products" view=ProductEnablementPage />
                    <Route path="/settings/sso" view=SsoConfigPage />
                    <Route path="/settings/users" view=UserManagementPage />
                    // Product sub-apps loaded via lazy islands
                    <Route path="/hr/*" view=HrIsland />
                    <Route path="/payroll/*" view=PayrollIsland />
                    <Route path="/mail/*" view=MailIsland />
                </Route>
            </Routes>
        </Router>
    }
}

// Product islands are loaded only when the product is enabled.
// Each island is a separate WASM chunk loaded via <Suspense>.
#[island]
pub fn HrIsland() -> impl IntoView {
    // Checks product_enablement before rendering; 403 if disabled
    view! { <iframe src="/hr/app" /> }
}
```

---

## 10. k6 Load Test

```javascript
// tests/load/application-shell.k6.js
import http from 'k6/http';
import { check, sleep } from 'k6';
import ws from 'k6/ws';

export const options = {
  scenarios: {
    shell_frame_load: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 5000 },
        { duration: '3m', target: 10000 },
        { duration: '1m', target: 0 },
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    // ADR-0210 / PRD-application Performance Targets
    'http_req_duration{name:shell_frame}': ['p(99)<100'],   // ≤100ms @ 10k sessions
    'http_req_duration{name:login_redirect}': ['p(99)<200'],
    'http_req_failed': ['rate<0.001'],
  },
};

const BASE_URL = __ENV.BASE_URL || 'https://app.oyatie.local';

export default function () {
  // Load shell frame (SSR pre-auth)
  const res = http.get(`${BASE_URL}/login`, {
    tags: { name: 'shell_frame' },
  });
  check(res, {
    'shell frame status 200': (r) => r.status === 200,
    'shell frame body contains meta': (r) => r.body.includes('<meta name="oyatie-shell"'),
  });
  sleep(0.1);
}
```

---

## 11. Acceptance Gates

```
GATE APP-01: shell frame SSR p99 ≤ 100ms at 10k concurrent sessions (k6)
GATE APP-02: OIDC PKCE round-trip completes < 2s wall-clock (integration test)
GATE APP-03: SAML2 ACS endpoint processes assertion < 500ms p99
GATE APP-04: Tenant activation flow (slug + admin user + first product) < 5 min wall-clock (E2E Playwright)
GATE APP-05: EmployeeHired → TenantUserProvisioned idempotency: duplicate events produce single user row
GATE APP-06: Cross-tenant Cedar forbid: attempt to read other tenant's shell returns 403 (authorization test)
GATE APP-07: Audit log append-only: UPDATE/DELETE on shell_audit_log returns 0 rows affected (DB invariant test)
GATE APP-08: Two-cookie pattern: access token cookie has HttpOnly+Secure+SameSite=Strict (browser security scan)
GATE APP-09: Session revocation propagates < 1s (logout → subsequent request returns 401)
GATE APP-10: All audit events carry valid Ed25519 seal (audit-chain verification script)
```

---

## 12. Grit Claim Symbols

```
grit session start m03-p06-application-2026-05-13
grit claim m03.p06.application.shell
grit claim m03.p06.application.oidc
grit claim m03.p06.application.enablement
grit claim m03.p06.application.billing
# ... implement ...
grit done --agent m03-p06-application-2026-05-13
```

---

## 13. ICM Payload

```bash
icm store \
  -t context-oyatie \
  -c "M03-P06 Application B2B shell impl-plan complete: two-cookie OIDC/SAML2 SSO, product-enablement console DDL, Leptos SSR/SPA web shell, EmployeeHired→ProvisionTenantUser Workflow event, sub-5-min tenant activation, 10k-session shell-frame p99 ≤100ms k6 load test, Cedar cross-tenant forbid policies, Ed25519 audit chain" \
  -i high \
  -k "application,b2b,oidc,saml,leptos,ssr,spa,product-enablement,onboarding,shell"
```
