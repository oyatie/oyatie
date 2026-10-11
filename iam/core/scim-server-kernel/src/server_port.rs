use super::*;

/// The contract every SCIM µservice implements. Async (the durable store
/// behind a server performs real I/O), modelled with return-position boxed
/// futures — `core::future::Future` + `core::pin::Pin` + `Box::pin`, no
/// `async-trait` / `futures` dep — so the kernel stays dependency-free.
pub trait ScimServer: Send + Sync {
    fn list_users<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<User>, ScimError>> + Send + 'a>>;
    fn get_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>>;
    fn create_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>>;
    fn replace_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>>;
    fn patch_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>>;
    fn delete_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>>;

    fn list_groups<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<Group>, ScimError>> + Send + 'a>>;
    fn get_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>>;
    fn create_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewGroup,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>>;
    fn patch_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>>;
    fn delete_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>>;
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListQuery {
    pub start_index: usize,    // 1-indexed per RFC 7644 §3.4.2.4
    pub items_per_page: usize, // bounded by server max
    pub filter: Option<String>,
}

impl Default for ListQuery {
    fn default() -> Self {
        Self {
            start_index: 1,
            items_per_page: 100,
            filter: None,
        }
    }
}

/// Inbound create/replace User payload. Wire shape is RFC 7644 camelCase —
/// real SCIM clients (Okta, Entra, Workspace) send `userName`/`externalId`/
/// `displayName` and the extension URNs, matching what [`User`] responses
/// serialize. The snake_case `alias`es keep pre-RFC-shape callers working
/// (backward compatibility only; new clients must send camelCase).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewUser {
    #[serde(rename = "userName", alias = "user_name")]
    pub user_name: String,
    #[serde(rename = "externalId", alias = "external_id", default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub name: Option<UserName>,
    #[serde(rename = "displayName", alias = "display_name", default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub emails: Vec<Email>,
    #[serde(
        rename = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User",
        alias = "enterprise",
        default
    )]
    pub enterprise: Option<EnterpriseExtension>,
    #[serde(
        rename = "urn:oyatie:scim:extension:2.0:User",
        alias = "oyatie",
        default
    )]
    pub oyatie: Option<OyatieExtension>,
}

/// Inbound create Group payload (RFC 7644 camelCase, snake_case alias kept
/// for backward compatibility — see [`NewUser`]).
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct NewGroup {
    #[serde(rename = "displayName", alias = "display_name")]
    pub display_name: String,
    #[serde(default)]
    pub members: Vec<GroupMembership>,
}
