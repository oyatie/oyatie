use super::*;

pub(super) fn membership_from_payload(
    payload: serde_json::Value,
    tenant_id: &str,
    user_id: &str,
    active_column: bool,
) -> Result<MembershipRecord, MembershipStoreError> {
    let user: User = serde_json::from_value(payload).map_err(|_| MembershipStoreError::Corrupt)?;
    let extension = user.oyatie.ok_or(MembershipStoreError::Corrupt)?;
    if extension.native_user_id.as_deref() != Some(user_id) || user.active != active_column {
        return Err(MembershipStoreError::Corrupt);
    }
    Ok(MembershipRecord {
        user_id: user_id.to_owned(),
        tenant_id: tenant_id.to_owned(),
        active: active_column,
        roles: extension.roles.unwrap_or_default(),
    })
}

impl UserMembershipStore for PgScimUserStore {
    fn candidates<'a>(
        &'a self,
        tenant_id: &'a str,
        user_id: &'a str,
    ) -> Pin<
        Box<dyn Future<Output = Result<Vec<MembershipRecord>, MembershipStoreError>> + Send + 'a>,
    > {
        Box::pin(async move {
            validate_tenant(&TenantId(tenant_id.to_owned()))
                .map_err(|_| MembershipStoreError::Corrupt)?;
            let mut tx = self
                .pool
                .begin()
                .await
                .map_err(|_| MembershipStoreError::Unavailable)?;
            sqlx::query(SET_LOCAL_TENANT_SQL)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
                .map_err(|_| MembershipStoreError::Unavailable)?;
            let rows = sqlx::query(FIND_NATIVE_USER_MEMBERSHIPS_SQL)
                .bind(tenant_id)
                .bind(User::OYATIE_SCHEMA)
                .bind(user_id)
                .fetch_all(&mut *tx)
                .await
                .map_err(|_| MembershipStoreError::Unavailable)?;
            tx.commit()
                .await
                .map_err(|_| MembershipStoreError::Unavailable)?;
            rows.into_iter()
                .map(|row| {
                    let payload = row
                        .try_get("payload_json")
                        .map_err(|_| MembershipStoreError::Corrupt)?;
                    let active = row
                        .try_get("active")
                        .map_err(|_| MembershipStoreError::Corrupt)?;
                    membership_from_payload(payload, tenant_id, user_id, active)
                })
                .collect()
        })
    }
}
