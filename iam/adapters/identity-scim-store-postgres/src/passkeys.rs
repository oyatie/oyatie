//! Tenant-scoped durable WebAuthn state. A challenge is removed by one atomic
//! DELETE RETURNING before a ceremony is verified, so a concurrent replay
//! cannot complete the same ceremony twice.

use serde_json::Value;
use shared_postgres_command_kernel::SET_LOCAL_TENANT_SQL;
use sqlx::{PgPool, Postgres, Row, Transaction};

use super::{TenantId, validate_tenant};

const INSERT_CHALLENGE: &str = "INSERT INTO identity_scim.identity_user_passkey_challenges (tenant_id, challenge_id, purpose, user_id, state_json, expires_at) VALUES ($1, $2, $3, $4, $5, to_timestamp($6))";
const TAKE_CHALLENGE: &str = "DELETE FROM identity_scim.identity_user_passkey_challenges WHERE tenant_id = $1 AND challenge_id = $2 AND purpose = $3 AND expires_at > to_timestamp($4) RETURNING user_id, state_json";
const PRUNE_CHALLENGES: &str = "DELETE FROM identity_scim.identity_user_passkey_challenges WHERE tenant_id = $1 AND expires_at <= to_timestamp($2)";
const LIST_CREDENTIALS: &str = "SELECT credential_json FROM identity_scim.identity_user_passkeys WHERE tenant_id = $1 AND user_id = $2 ORDER BY credential_id";
const FIND_CREDENTIAL: &str = "SELECT credential_json, sign_count FROM identity_scim.identity_user_passkeys WHERE tenant_id = $1 AND user_id = $2 AND credential_id = $3";
const INSERT_CREDENTIAL: &str = "INSERT INTO identity_scim.identity_user_passkeys (tenant_id, user_id, credential_id, credential_json, sign_count) VALUES ($1, $2, $3, $4, $5)";
const CAS_CREDENTIAL: &str = "UPDATE identity_scim.identity_user_passkeys SET credential_json = $4, sign_count = $5, updated_at = now() WHERE tenant_id = $1 AND user_id = $2 AND credential_id = $3 AND sign_count = $6";
const INSERT_SESSION: &str = "INSERT INTO identity_scim.identity_user_sessions (tenant_id, session_hash, user_id, expires_at) VALUES ($1, $2, $3, to_timestamp($4))";
const FIND_SESSION: &str = "SELECT user_id, extract(epoch from expires_at)::bigint AS expires_at FROM identity_scim.identity_user_sessions WHERE tenant_id = $1 AND session_hash = $2 AND expires_at > to_timestamp($3)";
const DELETE_SESSION: &str =
    "DELETE FROM identity_scim.identity_user_sessions WHERE tenant_id = $1 AND session_hash = $2";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PasskeyStoreError {
    Unavailable,
    Corrupt,
    Conflict,
}

#[derive(Debug)]
pub struct PendingChallenge {
    pub user_id: Option<String>,
    pub state: Value,
}

#[derive(Debug)]
pub struct StoredCredential {
    pub document: Value,
    pub sign_count: u32,
}

#[derive(Debug)]
pub struct StoredSession {
    pub user_id: String,
    pub expires_at_epoch_seconds: i64,
}

pub struct ChallengeWindow {
    pub issued_at_epoch_seconds: i64,
    pub expires_at_epoch_seconds: i64,
}

#[derive(Clone)]
pub struct PgPasskeyStore {
    pool: PgPool,
}

impl PgPasskeyStore {
    pub fn from_pool(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn tenant_tx(
        &self,
        tenant_id: &str,
    ) -> Result<Transaction<'_, Postgres>, PasskeyStoreError> {
        validate_tenant(&TenantId(tenant_id.to_owned())).map_err(|_| PasskeyStoreError::Corrupt)?;
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        sqlx::query(SET_LOCAL_TENANT_SQL)
            .bind(tenant_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        Ok(tx)
    }
}

mod challenges;
mod credentials;
mod sessions;
fn map_write_error(error: sqlx::Error) -> PasskeyStoreError {
    if let sqlx::Error::Database(database) = &error
        && database.code().as_deref() == Some("23505")
    {
        return PasskeyStoreError::Conflict;
    }
    PasskeyStoreError::Unavailable
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passkey_queries_bind_exact_tenant_and_consume_challenge_atomically() {
        for query in [
            INSERT_CHALLENGE,
            TAKE_CHALLENGE,
            PRUNE_CHALLENGES,
            LIST_CREDENTIALS,
            FIND_CREDENTIAL,
            INSERT_CREDENTIAL,
            CAS_CREDENTIAL,
            INSERT_SESSION,
            FIND_SESSION,
            DELETE_SESSION,
        ] {
            assert!(
                query.contains("tenant_id = $1") || query.contains("(tenant_id,"),
                "{query}"
            );
        }
        assert!(TAKE_CHALLENGE.starts_with("DELETE"));
        assert!(TAKE_CHALLENGE.contains("expires_at > to_timestamp($4) RETURNING"));
        assert!(CAS_CREDENTIAL.contains("AND sign_count = $6"));
    }
}
