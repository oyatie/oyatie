use super::*;

impl UserStore for PgScimUserStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>> {
        self.list_users(tenant)
    }

    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        self.get_user(tenant, id)
    }

    fn put<'a>(
        &'a self,
        user: &'a User,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let payload = serde_json::to_value(user).map_err(|e| corrupt(e.to_string()))?;
            // external_id is OPTIONAL in SCIM (RFC 7643 §3.1): bind the payload's
            // Option faithfully into the nullable column — NULL when absent,
            // never coerced to "" (which would diverge from the payload).
            let external_id: Option<&str> = user.external_id.as_deref();
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            sqlx::query(INSERT_USER_SQL)
                .bind(&tenant.0)
                .bind(&user.id.0)
                .bind(&user.user_name)
                .bind(external_id)
                .bind(user.active)
                .bind(&payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(user_write_error)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn compare_and_put<'a>(
        &'a self,
        expected: &'a User,
        proposed: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let row = sqlx::query(LOCK_USER_SQL)
                .bind(&tenant.0)
                .bind(&expected.id.0)
                .fetch_optional(&mut *tx)
                .await
                .map_err(store_unavailable)?
                .ok_or(ScimStoreError::Conflict)?;
            let current: User =
                serde_json::from_value(row.try_get("payload_json").map_err(|_| {
                    ScimStoreError::Corrupt {
                        detail: "invalid user payload".to_owned(),
                    }
                })?)
                .map_err(|_| ScimStoreError::Corrupt {
                    detail: "invalid user payload".to_owned(),
                })?;
            if current != *expected {
                return Err(ScimStoreError::Conflict);
            }
            if !native_admin && protected_user_transition(&current, proposed) {
                return Err(ScimStoreError::ProtectedUserChange);
            }
            let payload =
                serde_json::to_value(proposed).map_err(|error| corrupt(error.to_string()))?;
            sqlx::query(UPDATE_USER_SQL)
                .bind(&tenant.0)
                .bind(&expected.id.0)
                .bind(&proposed.user_name)
                .bind(proposed.external_id.as_deref())
                .bind(proposed.active)
                .bind(payload)
                .bind(SCHEMA_VERSION)
                .execute(&mut *tx)
                .await
                .map_err(user_write_error)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn compare_and_delete<'a>(
        &'a self,
        expected: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            validate_tenant(tenant)?;
            let mut tx = self.pool.begin().await.map_err(store_unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(&tenant.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            let row = sqlx::query(LOCK_USER_SQL)
                .bind(&tenant.0)
                .bind(&expected.id.0)
                .fetch_optional(&mut *tx)
                .await
                .map_err(store_unavailable)?
                .ok_or(ScimStoreError::Conflict)?;
            let current: User =
                serde_json::from_value(row.try_get("payload_json").map_err(|_| {
                    ScimStoreError::Corrupt {
                        detail: "invalid user payload".to_owned(),
                    }
                })?)
                .map_err(|_| ScimStoreError::Corrupt {
                    detail: "invalid user payload".to_owned(),
                })?;
            if current != *expected {
                return Err(ScimStoreError::Conflict);
            }
            if !native_admin && protected_user_deletion(&current) {
                return Err(ScimStoreError::ProtectedUserChange);
            }
            sqlx::query(DELETE_USER_SQL)
                .bind(&tenant.0)
                .bind(&expected.id.0)
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
            sqlx::query(DELETE_USER_SQL)
                .bind(&tenant.0)
                .bind(&id.0)
                .execute(&mut *tx)
                .await
                .map_err(store_unavailable)?;
            tx.commit().await.map_err(store_unavailable)?;
            Ok(())
        })
    }

    fn find_by_user_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        self.find_user_by_name(tenant, user_name)
    }
}
