use super::*;

/// Pluggable ID generator (opaque random IDs in production; monotonic counter for
/// tests).
pub trait IdGen: Send + Sync {
    fn next_id(&self) -> ScimId;
}

pub struct CounterIdGen(std::sync::atomic::AtomicU64);
impl CounterIdGen {
    pub fn new() -> Self {
        Self(std::sync::atomic::AtomicU64::new(1))
    }
}
impl Default for CounterIdGen {
    fn default() -> Self {
        Self::new()
    }
}
impl IdGen for CounterIdGen {
    fn next_id(&self) -> ScimId {
        let n = self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        ScimId(format!("urn:oya:scim:{n:08x}"))
    }
}

pub struct ReferenceScimServer<U: UserStore, G: GroupStore, I: IdGen> {
    pub users: U,
    pub groups: G,
    pub ids: I,
    pub base_url: String, // e.g. "https://identity.oyatie.com/scim/v2"
    pub max_items_per_page: usize,
}

impl<U: UserStore, G: GroupStore, I: IdGen> ReferenceScimServer<U, G, I> {
    pub fn new(users: U, groups: G, ids: I, base_url: impl Into<String>) -> Self {
        Self {
            users,
            groups,
            ids,
            base_url: base_url.into(),
            max_items_per_page: 200,
        }
    }

    pub fn create_user_with_authority<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewUser,
        now_unix: i64,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            validate_native_binding(input.oyatie.as_ref())?;
            if !native_admin
                && input
                    .oyatie
                    .as_ref()
                    .is_some_and(|ext| ext.native_user_id.is_some() || ext.roles.is_some())
            {
                return Err(ScimStoreError::ProtectedUserChange.into());
            }
            if input.user_name.is_empty() {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "userName is required",
                ));
            }
            if self
                .users
                .find_by_user_name(tenant, &input.user_name)
                .await
                .is_some()
            {
                return Err(ScimError::new(
                    409,
                    Some(ScimType::Uniqueness),
                    format!("userName '{}' already exists", input.user_name),
                ));
            }
            let id = self.ids.next_id();
            let etag = format!("{}{}", id.0, now_unix);
            let user = User {
                schemas: build_user_schemas(&input),
                id: id.clone(),
                external_id: input.external_id,
                user_name: input.user_name,
                name: input.name,
                display_name: input.display_name,
                active: input.active,
                emails: input.emails,
                groups: Vec::new(),
                enterprise: input.enterprise,
                oyatie: input.oyatie,
                meta: self.user_meta(tenant, &id, now_unix, &etag),
            };
            self.users.put(&user, tenant).await?;
            Ok(user)
        })
    }

    /// The serving guard passes `native_admin=true` only after both its
    /// scope and Cedar decisions. The store checks the protected transition
    /// under its own lock; an earlier read cannot authorize a later write.
    pub fn replace_user_with_authority<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        input: NewUser,
        now_unix: i64,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            validate_native_binding(input.oyatie.as_ref())?;
            let existing =
                self.users.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("user '{}' not found", id.0))
                })?;
            if input.user_name != existing.user_name
                && self
                    .users
                    .find_by_user_name(tenant, &input.user_name)
                    .await
                    .is_some()
            {
                return Err(ScimError::new(
                    409,
                    Some(ScimType::Uniqueness),
                    format!("userName '{}' already exists", input.user_name),
                ));
            }
            let etag = format!("{}{}", id.0, now_unix);
            let mut user = User {
                schemas: build_user_schemas(&input),
                id: id.clone(),
                external_id: input.external_id,
                user_name: input.user_name,
                name: input.name,
                display_name: input.display_name,
                active: input.active,
                emails: input.emails,
                groups: existing.groups.clone(),
                enterprise: input.enterprise,
                oyatie: input.oyatie,
                meta: self.user_meta(tenant, id, now_unix, &etag),
            };
            user.meta.created = existing.meta.created.clone();
            self.users
                .compare_and_put(&existing, &user, tenant, native_admin)
                .await?;
            Ok(user)
        })
    }

    pub fn patch_user_with_authority<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let existing =
                self.users.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("user '{}' not found", id.0))
                })?;
            let mut user = existing.clone();
            for operation in &op.operations {
                apply_patch_user(&mut user, operation)?;
            }
            validate_native_binding(user.oyatie.as_ref())?;
            if user.oyatie.is_some()
                && !user
                    .schemas
                    .iter()
                    .any(|schema| schema == User::OYATIE_SCHEMA)
            {
                user.schemas.push(User::OYATIE_SCHEMA.to_owned());
            }
            let etag = format!("{}{}", id.0, now_unix);
            user.meta.last_modified = rfc3339(now_unix);
            user.meta.version = format!("W/\"{etag}\"");
            self.users
                .compare_and_put(&existing, &user, tenant, native_admin)
                .await?;
            Ok(user)
        })
    }

    pub fn delete_user_with_authority<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let existing =
                self.users.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("user '{}' not found", id.0))
                })?;
            self.users
                .compare_and_delete(&existing, tenant, native_admin)
                .await?;
            Ok(())
        })
    }

    fn user_meta(&self, tenant: &TenantId, id: &ScimId, now_unix: i64, etag: &str) -> Meta {
        Meta {
            resource_type: "User".into(),
            created: rfc3339(now_unix),
            last_modified: rfc3339(now_unix),
            location: format!("{}/{}/Users/{}", self.base_url, tenant.0, id.0),
            version: format!("W/\"{etag}\""),
        }
    }
    pub(super) fn group_meta(
        &self,
        tenant: &TenantId,
        id: &ScimId,
        now_unix: i64,
        etag: &str,
    ) -> Meta {
        Meta {
            resource_type: "Group".into(),
            created: rfc3339(now_unix),
            last_modified: rfc3339(now_unix),
            location: format!("{}/{}/Groups/{}", self.base_url, tenant.0, id.0),
            version: format!("W/\"{etag}\""),
        }
    }
}
