use super::*;

impl PgPasskeyStore {
    pub async fn credentials_for_user(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<Vec<Value>, PasskeyStoreError> {
        let mut tx = self.tenant_tx(tenant_id).await?;
        let rows = sqlx::query(LIST_CREDENTIALS)
            .bind(tenant_id)
            .bind(user_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        rows.into_iter()
            .map(|row| {
                row.try_get("credential_json")
                    .map_err(|_| PasskeyStoreError::Corrupt)
            })
            .collect()
    }

    pub async fn credential(
        &self,
        tenant_id: &str,
        user_id: &str,
        credential_id: &[u8],
    ) -> Result<Option<StoredCredential>, PasskeyStoreError> {
        let mut tx = self.tenant_tx(tenant_id).await?;
        let row = sqlx::query(FIND_CREDENTIAL)
            .bind(tenant_id)
            .bind(user_id)
            .bind(credential_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        row.map(|row| {
            let count: i64 = row
                .try_get("sign_count")
                .map_err(|_| PasskeyStoreError::Corrupt)?;
            Ok(StoredCredential {
                document: row
                    .try_get("credential_json")
                    .map_err(|_| PasskeyStoreError::Corrupt)?,
                sign_count: u32::try_from(count).map_err(|_| PasskeyStoreError::Corrupt)?,
            })
        })
        .transpose()
    }

    pub async fn insert_credential(
        &self,
        tenant_id: &str,
        user_id: &str,
        credential_id: &[u8],
        document: &Value,
        sign_count: u32,
    ) -> Result<(), PasskeyStoreError> {
        if credential_id.is_empty() || credential_id.len() > 1023 {
            return Err(PasskeyStoreError::Corrupt);
        }
        let mut tx = self.tenant_tx(tenant_id).await?;
        sqlx::query(INSERT_CREDENTIAL)
            .bind(tenant_id)
            .bind(user_id)
            .bind(credential_id)
            .bind(document)
            .bind(i64::from(sign_count))
            .execute(&mut *tx)
            .await
            .map_err(map_write_error)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)
    }

    pub async fn compare_and_set_counter(
        &self,
        tenant_id: &str,
        user_id: &str,
        credential_id: &[u8],
        document: &Value,
        previous: u32,
        next: u32,
    ) -> Result<bool, PasskeyStoreError> {
        let mut tx = self.tenant_tx(tenant_id).await?;
        let result = sqlx::query(CAS_CREDENTIAL)
            .bind(tenant_id)
            .bind(user_id)
            .bind(credential_id)
            .bind(document)
            .bind(i64::from(next))
            .bind(i64::from(previous))
            .execute(&mut *tx)
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        tx.commit()
            .await
            .map_err(|_| PasskeyStoreError::Unavailable)?;
        Ok(result.rows_affected() == 1)
    }
}
