//! Additive durable passkey schema, applied through the native migration splitter.

use shared_postgres_command_kernel::split_migration_statements;
use sqlx::{Connection, PgConnection};

/// Apply the additive passkey schema on an explicit administrator connection.
/// The SCIM schema and runtime role must already exist. This is a setup API;
/// serving connections never call it or receive additional privileges.
///
/// # Errors
/// Returns the database error after rolling back all statements on failure.
/// A rollback or commit failure is also returned to the caller.
pub async fn provision_passkey_schema(setup: &mut PgConnection) -> Result<(), sqlx::Error> {
    let mut transaction = setup.begin().await?;
    for statement in split_migration_statements(PASSKEY_AUTHENTICATION_SQL) {
        if let Err(error) = sqlx::query(&statement).execute(&mut *transaction).await {
            transaction.rollback().await?;
            return Err(error);
        }
    }
    transaction.commit().await
}

pub const PASSKEY_AUTHENTICATION_SQL: &str = r#"-- Durable, tenant-scoped WebAuthn challenge, credential, and session records.
-- Applied after the SCIM user tables; every serving read/write sets
-- oyatie.tenant_id in its own transaction before touching these relations.

CREATE TABLE IF NOT EXISTS identity_scim.identity_user_passkey_challenges (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    challenge_id bytea NOT NULL CHECK (octet_length(challenge_id) = 32),
    purpose text NOT NULL CHECK (purpose IN ('enroll', 'authenticate')),
    user_id text,
    state_json jsonb NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, challenge_id),
    CHECK (purpose <> 'enroll' OR user_id IS NOT NULL)
);
ALTER TABLE identity_scim.identity_user_passkey_challenges ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_user_passkey_challenges FORCE ROW LEVEL SECURITY;
DO $$ BEGIN
    CREATE POLICY identity_user_passkey_challenges_tenant_isolation ON identity_scim.identity_user_passkey_challenges AS PERMISSIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    CREATE POLICY identity_user_passkey_challenges_require_tenant_guc ON identity_scim.identity_user_passkey_challenges AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
GRANT SELECT, INSERT, DELETE ON identity_scim.identity_user_passkey_challenges TO identity_scim_runtime;

CREATE TABLE IF NOT EXISTS identity_scim.identity_user_passkeys (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    user_id text NOT NULL,
    credential_id bytea NOT NULL CHECK (octet_length(credential_id) BETWEEN 1 AND 1023),
    credential_json jsonb NOT NULL,
    sign_count bigint NOT NULL CHECK (sign_count >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, credential_id)
);
CREATE INDEX IF NOT EXISTS identity_user_passkeys_by_user ON identity_scim.identity_user_passkeys (tenant_id, user_id);
ALTER TABLE identity_scim.identity_user_passkeys ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_user_passkeys FORCE ROW LEVEL SECURITY;
DO $$ BEGIN
    CREATE POLICY identity_user_passkeys_tenant_isolation ON identity_scim.identity_user_passkeys AS PERMISSIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    CREATE POLICY identity_user_passkeys_require_tenant_guc ON identity_scim.identity_user_passkeys AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
GRANT SELECT, INSERT, UPDATE, DELETE ON identity_scim.identity_user_passkeys TO identity_scim_runtime;

CREATE TABLE IF NOT EXISTS identity_scim.identity_user_sessions (
    tenant_id text NOT NULL CHECK (tenant_id <> ''),
    session_hash bytea NOT NULL CHECK (octet_length(session_hash) = 32),
    user_id text NOT NULL,
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, session_hash)
);
CREATE INDEX IF NOT EXISTS identity_user_sessions_by_user ON identity_scim.identity_user_sessions (tenant_id, user_id);
ALTER TABLE identity_scim.identity_user_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE identity_scim.identity_user_sessions FORCE ROW LEVEL SECURITY;
DO $$ BEGIN
    CREATE POLICY identity_user_sessions_tenant_isolation ON identity_scim.identity_user_sessions AS PERMISSIVE FOR ALL TO identity_scim_runtime USING (tenant_id = current_setting('oyatie.tenant_id', true)) WITH CHECK (tenant_id = current_setting('oyatie.tenant_id', true));
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
DO $$ BEGIN
    CREATE POLICY identity_user_sessions_require_tenant_guc ON identity_scim.identity_user_sessions AS RESTRICTIVE FOR ALL TO identity_scim_runtime USING (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '') WITH CHECK (current_setting('oyatie.tenant_id', true) IS NOT NULL AND current_setting('oyatie.tenant_id', true) <> '');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
GRANT SELECT, INSERT, DELETE ON identity_scim.identity_user_sessions TO identity_scim_runtime;
"#;
