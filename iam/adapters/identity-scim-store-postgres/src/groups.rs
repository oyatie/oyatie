use super::*;

impl GroupStore for PgScimGroupStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<Group>> + Send + 'a>> {
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
                        op = "group.list",
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
                    op = "group.list",
                    %error,
                    "set tenant GUC failed; returning empty page"
                );
                return Vec::new();
            }
            let rows = match sqlx::query(LIST_GROUPS_SQL)
                .bind(&tenant.0)
                .fetch_all(&mut *tx)
                .await
            {
                Ok(rows) => rows,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "group.list",
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

    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<Group>> + Send + 'a>> {
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
            let row = match sqlx::query(GET_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .fetch_optional(&mut *tx)
                .await
            {
                Ok(row) => row,
                Err(error) => {
                    tracing::warn!(
                        target: "identity_scim_store",
                        op = "group.get",
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

    fn put<'a>(
        &'a self,
        group: &'a Group,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let payload = serde_json::to_value(group).map_err(|e| corrupt(e.to_string()))?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(UPSERT_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&group.id.0)
                .bind(&group.display_name)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(DELETE_GROUP_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }
}
