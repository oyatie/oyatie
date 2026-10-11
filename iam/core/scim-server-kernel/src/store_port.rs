use super::*;

/// Store-port failure. A real durable backend (Postgres/data) can fail on
/// availability or integrity even for writes that are semantically valid, so
/// the write paths (`put`/`delete`) surface a `Result` rather than pretending
/// to be infallible. Reads keep returning `Option`/`Vec` (absence is not an
/// error); a backend that wants to surface read failures can map them through
/// the server layer in a later slice.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScimStoreError {
    /// The backing store cannot serve the request right now.
    Unavailable { detail: String },
    /// A persisted record failed to decode / violated an invariant.
    Corrupt { detail: String },
    /// The record changed after the server read it; retry against the current value.
    Conflict,
    /// A normal SCIM writer attempted to change a native IAM binding or
    /// reactivate a bound account.
    ProtectedUserChange,
}

impl fmt::Display for ScimStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { detail } => write!(f, "scim store unavailable: {detail}"),
            Self::Corrupt { detail } => write!(f, "scim store record corrupt: {detail}"),
            Self::Conflict => write!(f, "scim user changed concurrently"),
            Self::ProtectedUserChange => write!(f, "native user administration required"),
        }
    }
}

impl std::error::Error for ScimStoreError {}

impl From<ScimStoreError> for ScimError {
    fn from(error: ScimStoreError) -> Self {
        // A store availability/integrity failure is a 500 on the SCIM surface
        // (RFC 7644 §3.12): the request was well-formed; the backend failed.
        match error {
            ScimStoreError::Conflict => ScimError::new(409, None, error.to_string()),
            ScimStoreError::ProtectedUserChange => ScimError::new(403, None, error.to_string()),
            _ => ScimError::new(500, None, error.to_string()),
        }
    }
}

/// Must be evaluated inside the store's atomic compare-and-write operation.
/// A normal SCIM write may still update ordinary fields on an unbound user.
pub fn protected_user_transition(before: &User, after: &User) -> bool {
    let before_native = before
        .oyatie
        .as_ref()
        .and_then(|ext| ext.native_user_id.as_ref());
    let after_native = after
        .oyatie
        .as_ref()
        .and_then(|ext| ext.native_user_id.as_ref());
    let before_roles = before.oyatie.as_ref().and_then(|ext| ext.roles.as_ref());
    let after_roles = after.oyatie.as_ref().and_then(|ext| ext.roles.as_ref());
    before_native != after_native
        || before_roles != after_roles
        || (before_native.is_some() && !before.active && after.active)
}

pub fn protected_user_deletion(user: &User) -> bool {
    user.oyatie
        .as_ref()
        .is_some_and(|ext| ext.native_user_id.is_some() || ext.roles.is_some())
}

/// Pluggable user store. Async (the durable backend performs real I/O),
/// modelled with a return-position boxed future — `core::future::Future` +
/// `core::pin::Pin` + `Box::pin`, no `async-trait` / `futures` dep — so the
/// kernel stays dependency-free.
pub trait UserStore: Send + Sync {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>>;
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>>;
    fn put<'a>(
        &'a self,
        user: &'a User,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
    /// Compare the exact read snapshot, check protected fields, then replace
    /// under one row lock or transaction. No caller-side pre-read may grant.
    fn compare_and_put<'a>(
        &'a self,
        expected: &'a User,
        proposed: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
    fn compare_and_delete<'a>(
        &'a self,
        expected: &'a User,
        tenant: &'a TenantId,
        native_admin: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
    fn find_by_user_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>>;
}

pub trait GroupStore: Send + Sync {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<Group>> + Send + 'a>>;
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<Group>> + Send + 'a>>;
    fn put<'a>(
        &'a self,
        group: &'a Group,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>>;
}
