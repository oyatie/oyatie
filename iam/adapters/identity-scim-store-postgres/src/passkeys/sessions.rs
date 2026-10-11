use super::*;

impl PgPasskeyStore {
    pub async fn create_session(
        &self,
        tenant_id: &str,
        user_id: &str,
        session_hash: &[u8],
        expires_at_epoch_seconds: i64,
    ) -> Result<(), PasskeyStoreError> {
        if session_hash.len() != 32 {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        sqlx::query(INSERT_SESSION)
            .bind(tenant_id)
            .bind(session_hash)
            .bind(user_id)
            .bind(expires_at_epoch_seconds)
            .execute(&mut *tx)
            .await
            .map_err(map_write_error)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)
    }

    pub async fn session(
        &self,
        tenant_id: &str,
        session_hash: &[u8],
        now_epoch_seconds: i64,
    ) -> Result<Option<StoredSession>, PasskeyStoreError> {
        if session_hash.len() != 32 {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        let row = sqlx::query(FIND_SESSION)
            .bind(tenant_id)
            .bind(session_hash)
            .bind(now_epoch_seconds)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        row.map(|row| {
            Ok(StoredSession {
                user_id: row
                    .try_get("user_id")
                    .map_err(|_| PasskeyStoreError::Corrupt)?,
                expires_at_epoch_seconds: row
                    .try_get("expires_at")
                    .map_err(|_| PasskeyStoreError::Corrupt)?,
            })
        })
        .transpose()
    }

    pub async fn delete_session(
        &self,
        tenant_id: &str,
        session_hash: &[u8],
    ) -> Result<(), PasskeyStoreError> {
        if session_hash.len() != 32 {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        sqlx::query(DELETE_SESSION)
            .bind(tenant_id)
            .bind(session_hash)
            .execute(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)
    }
}
