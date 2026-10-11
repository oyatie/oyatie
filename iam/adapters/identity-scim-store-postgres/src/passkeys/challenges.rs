use super::*;

impl PgPasskeyStore {
    pub async fn save_challenge(
        &self,
        tenant_id: &str,
        challenge_id: &[u8],
        purpose: &str,
        user_id: Option<&str>,
        state: &Value,
        window: ChallengeWindow,
    ) -> Result<(), PasskeyStoreError> {
        if challenge_id.len() != 32
            || !matches!(purpose, "enroll" | "authenticate")
            || (purpose == "enroll" && user_id.is_none())
            || window.expires_at_epoch_seconds <= window.issued_at_epoch_seconds
        {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        sqlx::query(PRUNE_CHALLENGES)
            .bind(tenant_id)
            .bind(window.issued_at_epoch_seconds)
            .execute(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        sqlx::query(INSERT_CHALLENGE)
            .bind(tenant_id)
            .bind(challenge_id)
            .bind(purpose)
            .bind(user_id)
            .bind(state)
            .bind(window.expires_at_epoch_seconds)
            .execute(&mut *tx)
            .await
            .map_err(map_write_error)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)
    }

    pub async fn take_challenge(
        &self,
        tenant_id: &str,
        challenge_id: &[u8],
        purpose: &str,
        now_epoch_seconds: i64,
    ) -> Result<Option<PendingChallenge>, PasskeyStoreError> {
        if challenge_id.len() != 32 || !matches!(purpose, "enroll" | "authenticate") {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        let row = sqlx::query(TAKE_CHALLENGE)
            .bind(tenant_id)
            .bind(challenge_id)
            .bind(purpose)
            .bind(now_epoch_seconds)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        row.map(|row| {
            Ok(PendingChallenge {
                user_id: row
                    .try_get("user_id")
                    .map_err(|_| PasskeyStoreError::Corrupt)?,
                state: row
                    .try_get("state_json")
                    .map_err(|_| PasskeyStoreError::Corrupt)?,
            })
        })
        .transpose()
    }
}
