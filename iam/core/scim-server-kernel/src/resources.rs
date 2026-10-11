use super::*;

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
    #[serde(
        rename = "nativeUserId",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub native_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
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
