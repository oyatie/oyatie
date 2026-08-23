//! SCIM 2.0 (RFC 7643/7644) server kernel.
//!
//! Authority: ADR-0190 (SCIM 2.0 inbound provisioning), ADR-0187 (Zitadel
//! IdP via adapter), ADR-0145 (inter-microservice OIDC bearer).
//!
//! This kernel exposes:
//!
//! - [`ScimServer`] trait — the contract every µservice handler implements.
//! - [`ReferenceScimServer`] — a fully functional in-memory reference impl
//!   suitable for tests + reference. Production wires a Postgres-backed
//!   [`UserStore`] / [`GroupStore`].
//! - SCIM resource types: [`User`], [`Group`], [`PatchOp`], [`ListResponse`].
//! - Filter parser ([`parse_filter`]) covering the RFC 7644 §3.4.2.2
//!   subset used by Okta / Entra / Workspace SCIM clients in practice
//!   (`eq`, `co`, `sw`, `pr`, `ne`, `gt`, `ge`, `lt`, `le`, `and`, `or`).
//! - Error envelope per RFC 7644 §3.12 (HTTP-status + scimType).
//!
//! The kernel does NOT bind to a particular HTTP framework. Handler
//! adapters convert from `axum::extract`/`hyper::Request` into the kernel's
//! request shape; the kernel returns a [`ScimResponse`] which the adapter
//! serialises into the wire format.

#![forbid(unsafe_code)]

use core::future::Future;
use core::pin::Pin;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Tenant identifier — every SCIM request is tenant-scoped (URI segment
/// `/scim/v2/{tenant}/Users` per ADR-0190).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct TenantId(pub String);

/// Externally-issued opaque resource ID (server-assigned).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct ScimId(pub String);

/// SCIM `Meta` envelope per RFC 7643 §3.1.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Meta {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub created: String, // RFC 3339
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub location: String,
    pub version: String, // ETag value
}

#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct UserName {
    #[serde(rename = "givenName", default, skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(
        rename = "familyName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub family_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(
        rename = "middleName",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub middle_name: Option<String>,
    #[serde(
        rename = "honorificPrefix",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub honorific_prefix: Option<String>,
    #[serde(
        rename = "honorificSuffix",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub honorific_suffix: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Email {
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>, // data_class: SENSITIVE_PIPA_ART23
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GroupMembership {
    pub value: ScimId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(rename = "$ref")]
    pub r#ref: Option<String>, // data_class: SENSITIVE_PIPA_ART23
}

/// SCIM Enterprise extension per RFC 7643 §4.3.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct EnterpriseExtension {
    #[serde(
        rename = "employeeNumber",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub employee_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    #[serde(
        rename = "costCenter",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub cost_center: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub manager: Option<ScimId>,
}

/// Oyatie SCIM extension `urn:oyatie:scim:extension:2.0:User`.
#[derive(Clone, Debug, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct OyatieExtension {
    pub regulatory_pack: Option<String>,
    pub acr_floor: Option<String>,
    pub data_residency_jurisdiction: Option<String>,
}

/// Core User resource per RFC 7643 §4.1 + extensions.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub schemas: Vec<String>,
    pub id: ScimId,
    #[serde(
        rename = "externalId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub external_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<UserName>,
    #[serde(
        rename = "displayName",
        alias = "display_name",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub display_name: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<Email>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<GroupMembership>,
    #[serde(
        rename = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub enterprise: Option<EnterpriseExtension>,
    #[serde(
        rename = "urn:oyatie:scim:extension:2.0:User",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub oyatie: Option<OyatieExtension>,
    pub meta: Meta,
}

impl User {
    pub const CORE_SCHEMA: &'static str = "urn:ietf:params:scim:schemas:core:2.0:User";
    pub const ENTERPRISE_SCHEMA: &'static str =
        "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User";
    pub const OYATIE_SCHEMA: &'static str = "urn:oyatie:scim:extension:2.0:User";
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub schemas: Vec<String>,
    pub id: ScimId,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<GroupMembership>,
    pub meta: Meta,
}

impl Group {
    pub const CORE_SCHEMA: &'static str = "urn:ietf:params:scim:schemas:core:2.0:Group";
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ListResponse<T> {
    pub schemas: Vec<String>,
    #[serde(rename = "totalResults")]
    pub total_results: usize,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: usize,
    #[serde(rename = "startIndex")]
    pub start_index: usize,
}

impl<T> ListResponse<T> {
    pub const SCHEMA: &'static str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
}

/// PATCH operation per RFC 7644 §3.5.2.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchOp {
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<PatchOperation>,
}

impl PatchOp {
    pub const SCHEMA: &'static str = "urn:ietf:params:scim:api:messages:2.0:PatchOp";
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PatchOperation {
    pub op: PatchOpKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PatchOpKind {
    Add,
    Replace,
    Remove,
}

/// SCIM error envelope per RFC 7644 §3.12.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub status: u16,
    #[serde(rename = "scimType", skip_serializing_if = "Option::is_none")]
    pub scim_type: Option<ScimType>,
    pub detail: String,
}

impl ScimError {
    pub const SCHEMA: &'static str = "urn:ietf:params:scim:api:messages:2.0:Error";

    pub fn new(status: u16, scim_type: Option<ScimType>, detail: impl Into<String>) -> Self {
        Self {
            schemas: vec![Self::SCHEMA.to_owned()],
            status,
            scim_type,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScimType {
    InvalidFilter,
    InvalidPath,
    InvalidSyntax,
    InvalidValue,
    InvalidVers,
    Mutability,
    NoTarget,
    Sensitive,
    TooMany,
    Uniqueness,
}

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
}

impl fmt::Display for ScimStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable { detail } => write!(f, "scim store unavailable: {detail}"),
            Self::Corrupt { detail } => write!(f, "scim store record corrupt: {detail}"),
        }
    }
}

impl std::error::Error for ScimStoreError {}

impl From<ScimStoreError> for ScimError {
    fn from(error: ScimStoreError) -> Self {
        // A store availability/integrity failure is a 500 on the SCIM surface
        // (RFC 7644 §3.12): the request was well-formed; the backend failed.
        ScimError::new(500, None, error.to_string())
    }
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

#[derive(Default)]
pub struct InMemoryUserStore {
    inner: std::sync::Mutex<Vec<(TenantId, User)>>,
}

impl UserStore for InMemoryUserStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .map(|g| {
                    g.iter()
                        .filter(|(t, _)| t == tenant)
                        .map(|(_, u)| u.clone())
                        .collect()
                })
                .unwrap_or_default()
        })
    }
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, u)| t == tenant && u.id == *id)
                .map(|(_, u)| u.clone())
        })
    }
    fn put<'a>(
        &'a self,
        user: &'a User,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            if let Some(slot) = g.iter_mut().find(|(t, u)| t == tenant && u.id == user.id) {
                slot.1 = user.clone();
            } else {
                g.push((tenant.clone(), user.clone()));
            }
            Ok(())
        })
    }
    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "user store lock poisoned".to_owned(),
            })?;
            g.retain(|(t, u)| !(t == tenant && u.id == *id));
            Ok(())
        })
    }
    fn find_by_user_name<'a>(
        &'a self,
        tenant: &'a TenantId,
        user_name: &'a str,
    ) -> Pin<Box<dyn Future<Output = Option<User>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, u)| t == tenant && u.user_name == user_name)
                .map(|(_, u)| u.clone())
        })
    }
}

#[derive(Default)]
pub struct InMemoryGroupStore {
    inner: std::sync::Mutex<Vec<(TenantId, Group)>>,
}

impl GroupStore for InMemoryGroupStore {
    fn list<'a>(
        &'a self,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Vec<Group>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .map(|g| {
                    g.iter()
                        .filter(|(t, _)| t == tenant)
                        .map(|(_, gr)| gr.clone())
                        .collect()
                })
                .unwrap_or_default()
        })
    }
    fn get<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Option<Group>> + Send + 'a>> {
        Box::pin(async move {
            self.inner
                .lock()
                .ok()?
                .iter()
                .find(|(t, gr)| t == tenant && gr.id == *id)
                .map(|(_, gr)| gr.clone())
        })
    }
    fn put<'a>(
        &'a self,
        group: &'a Group,
        tenant: &'a TenantId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "group store lock poisoned".to_owned(),
            })?;
            if let Some(slot) = g
                .iter_mut()
                .find(|(t, gr)| t == tenant && gr.id == group.id)
            {
                slot.1 = group.clone();
            } else {
                g.push((tenant.clone(), group.clone()));
            }
            Ok(())
        })
    }
    fn delete<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimStoreError>> + Send + 'a>> {
        Box::pin(async move {
            let mut g = self.inner.lock().map_err(|_| ScimStoreError::Corrupt {
                detail: "group store lock poisoned".to_owned(),
            })?;
            g.retain(|(t, gr)| !(t == tenant && gr.id == *id));
            Ok(())
        })
    }
}

/// Pluggable ID generator (UUIDv7 in production; monotonic counter for
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

    fn user_meta(&self, tenant: &TenantId, id: &ScimId, now_unix: i64, etag: &str) -> Meta {
        Meta {
            resource_type: "User".into(),
            created: rfc3339(now_unix),
            last_modified: rfc3339(now_unix),
            location: format!("{}/{}/Users/{}", self.base_url, tenant.0, id.0),
            version: format!("W/\"{etag}\""),
        }
    }
    fn group_meta(&self, tenant: &TenantId, id: &ScimId, now_unix: i64, etag: &str) -> Meta {
        Meta {
            resource_type: "Group".into(),
            created: rfc3339(now_unix),
            last_modified: rfc3339(now_unix),
            location: format!("{}/{}/Groups/{}", self.base_url, tenant.0, id.0),
            version: format!("W/\"{etag}\""),
        }
    }
}

fn rfc3339(unix_seconds: i64) -> String {
    // Minimal RFC 3339 formatter — avoids a chrono dependency in the kernel.
    // For dates outside [1970-01-01, 9999-12-31] returns "1970-01-01T00:00:00Z".
    let secs = unix_seconds.max(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

fn days_to_ymd(mut days: i64) -> (i64, i64, i64) {
    let mut year = 1970_i64;
    loop {
        let dly = if is_leap(year) { 366 } else { 365 };
        if days < dly {
            break;
        }
        days -= dly;
        year += 1;
    }
    let dim = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut mo = 0;
    while days >= dim[mo] {
        days -= dim[mo];
        mo += 1;
    }
    (year, (mo + 1) as i64, days + 1)
}

fn is_leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

impl<U: UserStore, G: GroupStore, I: IdGen> ScimServer for ReferenceScimServer<U, G, I> {
    fn list_users<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<User>, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let all = self.users.list(tenant).await;
            let filtered = apply_user_filter(&all, q.filter.as_deref())?;
            let total = filtered.len();
            let per_page = q.items_per_page.min(self.max_items_per_page).max(1);
            let start = q.start_index.saturating_sub(1);
            let slice: Vec<User> = filtered.into_iter().skip(start).take(per_page).collect();
            Ok(ListResponse {
                schemas: vec![ListResponse::<User>::SCHEMA.to_owned()],
                total_results: total,
                resources: slice,
                items_per_page: per_page,
                start_index: q.start_index,
            })
        })
    }

    fn get_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            self.users
                .get(tenant, id)
                .await
                .ok_or_else(|| ScimError::new(404, None, format!("user '{}' not found", id.0)))
        })
    }

    fn create_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
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

    fn replace_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        input: NewUser,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let existing =
                self.users.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("user '{}' not found", id.0))
                })?;
            // Uniqueness only enforced if userName actually changed.
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
                groups: existing.groups, // preserve group memberships
                enterprise: input.enterprise,
                oyatie: input.oyatie,
                meta: self.user_meta(tenant, id, now_unix, &etag),
            };
            // Preserve original created timestamp.
            user.meta.created = existing.meta.created;
            self.users.put(&user, tenant).await?;
            Ok(user)
        })
    }

    fn patch_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<User, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let mut user =
                self.users.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("user '{}' not found", id.0))
                })?;
            for o in &op.operations {
                apply_patch_user(&mut user, o)?;
            }
            let etag = format!("{}{}", id.0, now_unix);
            user.meta.last_modified = rfc3339(now_unix);
            user.meta.version = format!("W/\"{etag}\"");
            self.users.put(&user, tenant).await?;
            Ok(user)
        })
    }

    fn delete_user<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>> {
        Box::pin(async move {
            if self.users.get(tenant, id).await.is_none() {
                return Err(ScimError::new(
                    404,
                    None,
                    format!("user '{}' not found", id.0),
                ));
            }
            self.users.delete(tenant, id).await?;
            Ok(())
        })
    }

    fn list_groups<'a>(
        &'a self,
        tenant: &'a TenantId,
        q: &'a ListQuery,
    ) -> Pin<Box<dyn Future<Output = Result<ListResponse<Group>, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let all = self.groups.list(tenant).await;
            let total = all.len();
            let per_page = q.items_per_page.min(self.max_items_per_page).max(1);
            let start = q.start_index.saturating_sub(1);
            let slice: Vec<Group> = all.into_iter().skip(start).take(per_page).collect();
            Ok(ListResponse {
                schemas: vec![ListResponse::<Group>::SCHEMA.to_owned()],
                total_results: total,
                resources: slice,
                items_per_page: per_page,
                start_index: q.start_index,
            })
        })
    }
    fn get_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            self.groups
                .get(tenant, id)
                .await
                .ok_or_else(|| ScimError::new(404, None, format!("group '{}' not found", id.0)))
        })
    }
    fn create_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        input: NewGroup,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            if input.display_name.is_empty() {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "displayName is required",
                ));
            }
            let id = self.ids.next_id();
            let etag = format!("{}{}", id.0, now_unix);
            let group = Group {
                schemas: vec![Group::CORE_SCHEMA.to_owned()],
                id: id.clone(),
                display_name: input.display_name,
                members: input.members,
                meta: self.group_meta(tenant, &id, now_unix, &etag),
            };
            self.groups.put(&group, tenant).await?;
            Ok(group)
        })
    }
    fn patch_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
        op: &'a PatchOp,
        now_unix: i64,
    ) -> Pin<Box<dyn Future<Output = Result<Group, ScimError>> + Send + 'a>> {
        Box::pin(async move {
            let mut group =
                self.groups.get(tenant, id).await.ok_or_else(|| {
                    ScimError::new(404, None, format!("group '{}' not found", id.0))
                })?;
            for o in &op.operations {
                apply_patch_group(&mut group, o)?;
            }
            let etag = format!("{}{}", id.0, now_unix);
            group.meta.last_modified = rfc3339(now_unix);
            group.meta.version = format!("W/\"{etag}\"");
            self.groups.put(&group, tenant).await?;
            Ok(group)
        })
    }
    fn delete_group<'a>(
        &'a self,
        tenant: &'a TenantId,
        id: &'a ScimId,
    ) -> Pin<Box<dyn Future<Output = Result<(), ScimError>> + Send + 'a>> {
        Box::pin(async move {
            if self.groups.get(tenant, id).await.is_none() {
                return Err(ScimError::new(
                    404,
                    None,
                    format!("group '{}' not found", id.0),
                ));
            }
            self.groups.delete(tenant, id).await?;
            Ok(())
        })
    }
}

fn build_user_schemas(input: &NewUser) -> Vec<String> {
    let mut s = vec![User::CORE_SCHEMA.to_owned()];
    if input.enterprise.is_some() {
        s.push(User::ENTERPRISE_SCHEMA.to_owned());
    }
    if input.oyatie.is_some() {
        s.push(User::OYATIE_SCHEMA.to_owned());
    }
    s
}

fn apply_patch_user(user: &mut User, op: &PatchOperation) -> Result<(), ScimError> {
    let path = op.path.as_deref().unwrap_or("");
    match (op.op, path) {
        (PatchOpKind::Replace, "active") => {
            let v = op.value.as_ref().and_then(|v| v.as_bool()).ok_or_else(|| {
                ScimError::new(400, Some(ScimType::InvalidValue), "active expects bool")
            })?;
            user.active = v;
            Ok(())
        }
        (PatchOpKind::Replace, "displayName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).map(String::from);
            user.display_name = v;
            Ok(())
        }
        (PatchOpKind::Replace, "userName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).ok_or_else(|| {
                ScimError::new(400, Some(ScimType::InvalidValue), "userName expects string")
            })?;
            user.user_name = v.to_owned();
            Ok(())
        }
        (PatchOpKind::Add, "emails") => {
            let arr = op
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    ScimError::new(400, Some(ScimType::InvalidValue), "emails expects array")
                })?;
            for e in arr {
                let parsed: Email = serde_json::from_value(e.clone()).map_err(|err| {
                    ScimError::new(400, Some(ScimType::InvalidSyntax), err.to_string())
                })?;
                user.emails.push(parsed);
            }
            Ok(())
        }
        (PatchOpKind::Remove, p) if p.starts_with("emails[") => {
            // Best-effort: only support emails[value eq "..."]; remove matching.
            if let Some(needle) = parse_filter_value_in_brackets(p) {
                user.emails.retain(|e| e.value != needle);
            }
            Ok(())
        }
        (op_kind, p) => Err(ScimError::new(
            400,
            Some(ScimType::InvalidPath),
            format!("unsupported {op_kind:?} on path '{p}'"),
        )),
    }
}

fn apply_patch_group(group: &mut Group, op: &PatchOperation) -> Result<(), ScimError> {
    let path = op.path.as_deref().unwrap_or("");
    match (op.op, path) {
        (PatchOpKind::Replace, "displayName") => {
            let v = op.value.as_ref().and_then(|v| v.as_str()).ok_or_else(|| {
                ScimError::new(
                    400,
                    Some(ScimType::InvalidValue),
                    "displayName expects string",
                )
            })?;
            group.display_name = v.to_owned();
            Ok(())
        }
        (PatchOpKind::Add, "members") => {
            let arr = op
                .value
                .as_ref()
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    ScimError::new(400, Some(ScimType::InvalidValue), "members expects array")
                })?;
            for m in arr {
                let parsed: GroupMembership = serde_json::from_value(m.clone()).map_err(|err| {
                    ScimError::new(400, Some(ScimType::InvalidSyntax), err.to_string())
                })?;
                group.members.push(parsed);
            }
            Ok(())
        }
        (PatchOpKind::Remove, p) if p.starts_with("members[") => {
            if let Some(needle) = parse_filter_value_in_brackets(p) {
                group.members.retain(|m| m.value.0 != needle);
            }
            Ok(())
        }
        (op_kind, p) => Err(ScimError::new(
            400,
            Some(ScimType::InvalidPath),
            format!("unsupported {op_kind:?} on path '{p}'"),
        )),
    }
}

/// Extract the bare value out of `members[value eq "abc"]` → `Some("abc")`.
fn parse_filter_value_in_brackets(p: &str) -> Option<String> {
    let lb = p.find('[')?;
    let rb = p.rfind(']')?;
    let inner = &p[lb + 1..rb];
    // Look for `eq` and quoted value.
    let q1 = inner.find('"')?;
    let q2 = inner.rfind('"')?;
    if q1 < q2 {
        Some(inner[q1 + 1..q2].to_owned())
    } else {
        None
    }
}

/// Apply a SCIM filter expression to a list of users. Returns the filtered
/// list. Supports the practical RFC 7644 §3.4.2.2 subset used by Okta /
/// Entra / Workspace clients in the wild.
fn apply_user_filter(users: &[User], filter: Option<&str>) -> Result<Vec<User>, ScimError> {
    let Some(f) = filter else {
        return Ok(users.to_vec());
    };
    let f = f.trim();
    if f.is_empty() {
        return Ok(users.to_vec());
    }
    let expr = parse_filter(f)?;
    Ok(users
        .iter()
        .filter(|u| eval_user(u, &expr))
        .cloned()
        .collect())
}

#[derive(Clone, Debug)]
pub enum FilterExpr {
    Eq(String, String),
    Ne(String, String),
    Co(String, String),
    Sw(String, String),
    Ew(String, String),
    Pr(String),
    And(Box<FilterExpr>, Box<FilterExpr>),
    Or(Box<FilterExpr>, Box<FilterExpr>),
    Not(Box<FilterExpr>),
}

/// Minimal recursive-descent filter parser. Sufficient for the in-the-wild
/// dialects used by the canonical SCIM clients.
pub fn parse_filter(input: &str) -> Result<FilterExpr, ScimError> {
    let mut p = Parser {
        src: input.as_bytes(),
        pos: 0,
    };
    let e = p.parse_or()?;
    p.skip_ws();
    if p.pos < p.src.len() {
        return Err(ScimError::new(
            400,
            Some(ScimType::InvalidFilter),
            format!("unexpected trailing input at byte {}", p.pos),
        ));
    }
    Ok(e)
}

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
}
impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_whitespace() {
            self.pos += 1;
        }
    }
    fn peek_keyword(&mut self, kw: &str) -> bool {
        self.skip_ws();
        let bytes = kw.as_bytes();
        if self.src.len() < self.pos + bytes.len() {
            return false;
        }
        let slice = &self.src[self.pos..self.pos + bytes.len()];
        if !slice.eq_ignore_ascii_case(bytes) {
            return false;
        }
        // Must be followed by a delimiter (whitespace, paren, eof).
        let next = self.pos + bytes.len();
        if next == self.src.len() {
            return true;
        }
        let c = self.src[next];
        c.is_ascii_whitespace() || c == b'(' || c == b')'
    }
    fn eat_keyword(&mut self, kw: &str) -> bool {
        if self.peek_keyword(kw) {
            self.pos += kw.len();
            true
        } else {
            false
        }
    }
    fn parse_or(&mut self) -> Result<FilterExpr, ScimError> {
        let mut left = self.parse_and()?;
        loop {
            self.skip_ws();
            if !self.eat_keyword("or") {
                break;
            }
            let right = self.parse_and()?;
            left = FilterExpr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_and(&mut self) -> Result<FilterExpr, ScimError> {
        let mut left = self.parse_not()?;
        loop {
            self.skip_ws();
            if !self.eat_keyword("and") {
                break;
            }
            let right = self.parse_not()?;
            left = FilterExpr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn parse_not(&mut self) -> Result<FilterExpr, ScimError> {
        self.skip_ws();
        if self.eat_keyword("not") {
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b'(' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "not requires (...)",
                ));
            }
            self.pos += 1;
            let inner = self.parse_or()?;
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b')' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "missing ) after not",
                ));
            }
            self.pos += 1;
            return Ok(FilterExpr::Not(Box::new(inner)));
        }
        self.parse_atom()
    }
    fn parse_atom(&mut self) -> Result<FilterExpr, ScimError> {
        self.skip_ws();
        if self.pos >= self.src.len() {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "unexpected eof",
            ));
        }
        if self.src[self.pos] == b'(' {
            self.pos += 1;
            let e = self.parse_or()?;
            self.skip_ws();
            if self.pos >= self.src.len() || self.src[self.pos] != b')' {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    "missing )",
                ));
            }
            self.pos += 1;
            return Ok(e);
        }
        let attr = self.parse_attr()?;
        self.skip_ws();
        if self.eat_keyword("pr") {
            return Ok(FilterExpr::Pr(attr));
        }
        let op = self.parse_op_word()?;
        self.skip_ws();
        let val = self.parse_string_literal()?;
        Ok(match op.as_str() {
            "eq" => FilterExpr::Eq(attr, val),
            "ne" => FilterExpr::Ne(attr, val),
            "co" => FilterExpr::Co(attr, val),
            "sw" => FilterExpr::Sw(attr, val),
            "ew" => FilterExpr::Ew(attr, val),
            other => {
                return Err(ScimError::new(
                    400,
                    Some(ScimType::InvalidFilter),
                    format!("unsupported op {other}"),
                ));
            }
        })
    }
    fn parse_attr(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() {
            let c = self.src[self.pos];
            if c.is_ascii_alphanumeric() || c == b'.' || c == b':' || c == b'_' || c == b'-' {
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == start {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected attribute name",
            ));
        }
        Ok(std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 in attr"))?
            .to_owned())
    }
    fn parse_op_word(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        let start = self.pos;
        while self.pos < self.src.len() && self.src[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected op",
            ));
        }
        Ok(std::str::from_utf8(&self.src[start..self.pos])
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 in op"))?
            .to_ascii_lowercase())
    }
    fn parse_string_literal(&mut self) -> Result<String, ScimError> {
        self.skip_ws();
        if self.pos >= self.src.len() || self.src[self.pos] != b'"' {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "expected \"..\" string literal",
            ));
        }
        self.pos += 1;
        let mut out = Vec::new();
        while self.pos < self.src.len() && self.src[self.pos] != b'"' {
            if self.src[self.pos] == b'\\' && self.pos + 1 < self.src.len() {
                out.push(self.src[self.pos + 1]);
                self.pos += 2;
            } else {
                out.push(self.src[self.pos]);
                self.pos += 1;
            }
        }
        if self.pos >= self.src.len() {
            return Err(ScimError::new(
                400,
                Some(ScimType::InvalidFilter),
                "unterminated string",
            ));
        }
        self.pos += 1;
        String::from_utf8(out)
            .map_err(|_| ScimError::new(400, Some(ScimType::InvalidFilter), "non-utf8 string"))
    }
}

fn eval_user(u: &User, expr: &FilterExpr) -> bool {
    match expr {
        FilterExpr::And(a, b) => eval_user(u, a) && eval_user(u, b),
        FilterExpr::Or(a, b) => eval_user(u, a) || eval_user(u, b),
        FilterExpr::Not(inner) => !eval_user(u, inner),
        FilterExpr::Pr(attr) => attr_present_user(u, attr),
        FilterExpr::Eq(attr, v) => {
            attr_value_user(u, attr).is_some_and(|s| s.eq_ignore_ascii_case(v))
        }
        FilterExpr::Ne(attr, v) => {
            attr_value_user(u, attr).is_none_or(|s| !s.eq_ignore_ascii_case(v))
        }
        FilterExpr::Co(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().contains(&v.to_ascii_lowercase())),
        FilterExpr::Sw(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().starts_with(&v.to_ascii_lowercase())),
        FilterExpr::Ew(attr, v) => attr_value_user(u, attr)
            .is_some_and(|s| s.to_ascii_lowercase().ends_with(&v.to_ascii_lowercase())),
    }
}

fn attr_value_user(u: &User, attr: &str) -> Option<String> {
    match attr {
        "userName" | "username" => Some(u.user_name.clone()),
        "id" => Some(u.id.0.clone()),
        "externalId" => u.external_id.clone(),
        "displayName" => u.display_name.clone(),
        "active" => Some(if u.active {
            "true".to_owned()
        } else {
            "false".to_owned()
        }),
        "emails.value" => u.emails.first().map(|e| e.value.clone()),
        "name.givenName" => u.name.as_ref().and_then(|n| n.given_name.clone()),
        "name.familyName" => u.name.as_ref().and_then(|n| n.family_name.clone()),
        _ => None,
    }
}

fn attr_present_user(u: &User, attr: &str) -> bool {
    attr_value_user(u, attr).is_some_and(|s| !s.is_empty())
}
