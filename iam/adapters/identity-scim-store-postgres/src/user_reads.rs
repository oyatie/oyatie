use super::*;

impl PgScimUserStore {
    pub(super) fn list_users<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to an
            // empty page before opening the tx / setting the GUC.
            if validate_tenant(tenant).is_err() {
                return Vec::new();
            }
            let mut tx = match self.pool.begin().await {
                Ok(tx) => tx,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.list",
                        %error,
                        "begin tx failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            if let Err(error) = sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
            {
                tracing::warn!(
                    target: "identity_scim_store",
                    op = "user.list",
                    %error,
                    "set tenant GUC failed; returning empty page"
                );
                return Vec::new();
            }
            let rows = match sqlx::query(LIST_USERS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.list",
                        %error,
                        "list query failed; returning empty page"
                    );
                    return Vec::new();
                }
            };
            let _ = tx.commit().await;
            rows.into_iter()
                .filter_map(|row| row.try_get::<serde_json::Value, _>("payload_json").ok())
                .filter_map(decode_payload)
                .collect()
        })
    }

    pub(super) fn get_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to None
            // before opening the tx / setting the GUC.
            validate_tenant(tenant).ok()?;
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = match sqlx::query(GET_USER_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.get",
                        %error,
                        "get query failed; returning None"
                    );
                    return None;
                }
            };
            let _ = tx.commit().await;
            let payload: serde_json::Value = row?.try_get("payload_json").ok()?;
            decode_payload(payload)
        })
    }

    pub(super) fn find_user_by_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            // Fail-closed parity with the write methods: a blank tenant would
            // bind an empty GUC (a silent deny-all), so short-circuit to None
            // before opening the tx / setting the GUC.
            validate_tenant(tenant).ok()?;
            let mut tx = self.pool.begin().await.ok()?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .ok()?;
            let row = match sqlx::query(FIND_USER_BY_NAME_SQL)
                .bind(&tenant.0)
                .bind(user_name)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "user.find_by_user_name",
                        %error,
                        "find-by-user-name query failed; returning None"
                    );
                    return None;
                }
            };
            let _ = tx.commit().await;
            let payload: serde_json::Value = row?.try_get("payload_json").ok()?;
            decode_payload(payload)
        })
    }
}
