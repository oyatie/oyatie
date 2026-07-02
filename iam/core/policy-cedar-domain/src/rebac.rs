//! ReBAC tuple-store port vocabulary.
//!
//! This module models the Zanzibar/OpenFGA-style relationship tuple surface
//! without binding the domain crate to any storage engine or serving path.  A
//! tuple is tenant-scoped and rendered within that scope as
//! `object#relation@subject`; the subject can be either a concrete object
//! (`user:alice`) or another userset (`group:platform#member`).
//! Zookie and snapshot tokens are opaque policy/tuple-store consistency
//! vocabulary: callers may carry and echo them, but ordering belongs to the
//! tuple store implementation.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Validation failure for ReBAC tuple vocabulary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebacTupleValidationError {
    EmptyField { field: &'static str },
    InvalidToken { field: &'static str, value: String },
    InvalidCanonicalTuple { detail: String },
    EmptyRewrite { kind: &'static str },
}

impl fmt::Display for RebacTupleValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(f, "{field} must not be empty"),
            Self::InvalidToken { field, value } => {
                write!(f, "{field} contains invalid characters: {value:?}")
            }
            Self::InvalidCanonicalTuple { detail } => {
                write!(f, "invalid ReBAC canonical tuple: {detail}")
            }
            Self::EmptyRewrite { kind } => write!(f, "{kind} userset rewrite must not be empty"),
        }
    }
}

impl std::error::Error for RebacTupleValidationError {}

/// Explicit tenant/namespace boundary for ReBAC tuples and queries.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RebacTenantScope {
    tenant_id: String, // data_class: TENANT_SCOPED
}

impl RebacTenantScope {
    pub fn new(tenant_id: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let tenant_id = tenant_id.into();
        validate_tenant_scope("rebac_tenant_scope.tenant_id", &tenant_id)?;
        Ok(Self { tenant_id })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.tenant_id
    }
}

impl Serialize for RebacTenantScope {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.tenant_id)
    }
}

impl<'de> Deserialize<'de> for RebacTenantScope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Object reference in a ReBAC tuple (`type:id`).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RebacObjectRef {
    object_type: String, // data_class: INTERNAL_ONLY
    object_id: String,   // data_class: TENANT_SCOPED
}

impl RebacObjectRef {
    pub fn new(
        object_type: impl Into<String>,
        object_id: impl Into<String>,
    ) -> Result<Self, RebacTupleValidationError> {
        let object_type = object_type.into();
        let object_id = object_id.into();
        validate_object_type("rebac_object.object_type", &object_type)?;
        validate_tuple_segment("rebac_object.object_id", &object_id)?;
        Ok(Self {
            object_type,
            object_id,
        })
    }

    pub fn parse(input: &str) -> Result<Self, RebacTupleValidationError> {
        let Some((object_type, object_id)) = input.split_once(':') else {
            return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                detail: "object reference must be type:id".to_owned(),
            });
        };
        Self::new(object_type, object_id)
    }

    #[must_use]
    pub fn object_type(&self) -> &str {
        &self.object_type
    }

    #[must_use]
    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        format!("{}:{}", self.object_type, self.object_id)
    }
}

impl<'de> Deserialize<'de> for RebacObjectRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Wire {
            object_type: String,
            object_id: String,
        }

        let wire = Wire::deserialize(deserializer)?;
        Self::new(wire.object_type, wire.object_id).map_err(serde::de::Error::custom)
    }
}

/// Relation segment in a ReBAC tuple (`viewer`, `member`, `owner`, ...).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct RebacRelation {
    value: String, // data_class: INTERNAL_ONLY
}

impl RebacRelation {
    pub fn new(relation: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let relation = relation.into();
        validate_relation("rebac_relation", &relation)?;
        Ok(Self { value: relation })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl Serialize for RebacRelation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.value)
    }
}

impl<'de> Deserialize<'de> for RebacRelation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Subject side of `object#relation@subject`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RebacSubjectRef {
    Object {
        object: RebacObjectRef, // data_class: TENANT_SCOPED
    },
    Userset {
        object: RebacObjectRef,  // data_class: TENANT_SCOPED
        relation: RebacRelation, // data_class: INTERNAL_ONLY
    },
}

impl RebacSubjectRef {
    #[must_use]
    pub fn object(object: RebacObjectRef) -> Self {
        Self::Object { object }
    }

    #[must_use]
    pub fn userset(object: RebacObjectRef, relation: RebacRelation) -> Self {
        Self::Userset { object, relation }
    }

    pub fn parse(input: &str) -> Result<Self, RebacTupleValidationError> {
        if let Some((object, relation)) = input.split_once('#') {
            if relation.contains('#') {
                return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                    detail: "userset subject must contain exactly one #".to_owned(),
                });
            }
            return Ok(Self::userset(
                RebacObjectRef::parse(object)?,
                RebacRelation::new(relation)?,
            ));
        }
        Ok(Self::object(RebacObjectRef::parse(input)?))
    }

    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        match self {
            Self::Object { object } => object.to_canonical_string(),
            Self::Userset { object, relation } => {
                format!("{}#{}", object.to_canonical_string(), relation.as_str())
            }
        }
    }
}

/// Relationship tuple rendered as `object#relation@subject` inside a tenant scope.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTuple {
    pub tenant: RebacTenantScope, // data_class: TENANT_SCOPED
    pub object: RebacObjectRef,   // data_class: TENANT_SCOPED
    pub relation: RebacRelation,  // data_class: INTERNAL_ONLY
    pub subject: RebacSubjectRef, // data_class: TENANT_SCOPED
}

impl RebacTuple {
    #[must_use]
    pub fn new(
        tenant: RebacTenantScope,
        object: RebacObjectRef,
        relation: RebacRelation,
        subject: RebacSubjectRef,
    ) -> Self {
        Self {
            tenant,
            object,
            relation,
            subject,
        }
    }

    pub fn parse(tenant: RebacTenantScope, input: &str) -> Result<Self, RebacTupleValidationError> {
        let Some((object_relation, subject)) = input.split_once('@') else {
            return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                detail: "tuple must be object#relation@subject".to_owned(),
            });
        };
        if subject.contains('@') {
            return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                detail: "tuple must contain exactly one @".to_owned(),
            });
        }
        let Some((object, relation)) = object_relation.split_once('#') else {
            return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                detail: "tuple object side must be object#relation".to_owned(),
            });
        };
        if relation.contains('#') {
            return Err(RebacTupleValidationError::InvalidCanonicalTuple {
                detail: "tuple object side must contain exactly one #".to_owned(),
            });
        }

        Ok(Self::new(
            tenant,
            RebacObjectRef::parse(object)?,
            RebacRelation::new(relation)?,
            RebacSubjectRef::parse(subject)?,
        ))
    }

    #[must_use]
    pub fn tenant(&self) -> &RebacTenantScope {
        &self.tenant
    }

    #[must_use]
    pub fn to_canonical_string(&self) -> String {
        format!(
            "{}#{}@{}",
            self.object.to_canonical_string(),
            self.relation.as_str(),
            self.subject.to_canonical_string()
        )
    }
}

/// Zanzibar-style consistency token returned by tuple-store writes.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Zookie {
    token: String, // data_class: INTERNAL_ONLY
}

impl Zookie {
    pub fn new(token: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let token = token.into();
        validate_opaque_token("zookie", &token)?;
        Ok(Self { token })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.token
    }
}

impl Serialize for Zookie {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

impl<'de> Deserialize<'de> for Zookie {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Opaque tuple-store snapshot token used for read consistency.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SnapshotToken {
    token: String, // data_class: INTERNAL_ONLY
}

impl SnapshotToken {
    pub fn new(token: impl Into<String>) -> Result<Self, RebacTupleValidationError> {
        let token = token.into();
        validate_opaque_token("snapshot_token", &token)?;
        Ok(Self { token })
    }

    #[must_use]
    pub fn from_zookie(zookie: Zookie) -> Self {
        Self {
            token: zookie.token,
        }
    }

    #[must_use]
    pub fn latest() -> Self {
        Self {
            token: "latest".to_owned(),
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.token
    }
}

impl Serialize for SnapshotToken {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.token)
    }
}

impl<'de> Deserialize<'de> for SnapshotToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Read snapshot requested from a tuple-store port.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RebacReadSnapshot {
    Latest,
    At {
        snapshot: SnapshotToken, // data_class: INTERNAL_ONLY
    },
}

impl RebacReadSnapshot {
    #[must_use]
    pub fn latest() -> Self {
        Self::Latest
    }

    #[must_use]
    pub fn at(snapshot: SnapshotToken) -> Self {
        Self::At { snapshot }
    }

    #[must_use]
    pub fn at_zookie(zookie: Zookie) -> Self {
        Self::at(SnapshotToken::from_zookie(zookie))
    }

    #[must_use]
    pub fn into_snapshot_token(self) -> SnapshotToken {
        match self {
            Self::Latest => SnapshotToken::latest(),
            Self::At { snapshot } => snapshot,
        }
    }
}

/// Tenant-scoped tuple-store query. Any `None` field is a wildcard within the tenant.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTupleQuery {
    pub tenant: RebacTenantScope,         // data_class: TENANT_SCOPED
    pub object: Option<RebacObjectRef>,   // data_class: TENANT_SCOPED
    pub relation: Option<RebacRelation>,  // data_class: INTERNAL_ONLY
    pub subject: Option<RebacSubjectRef>, // data_class: TENANT_SCOPED
}

impl RebacTupleQuery {
    #[must_use]
    pub fn new(tenant: RebacTenantScope) -> Self {
        Self {
            tenant,
            object: None,
            relation: None,
            subject: None,
        }
    }

    #[must_use]
    pub fn object_relation(
        tenant: RebacTenantScope,
        object: RebacObjectRef,
        relation: RebacRelation,
    ) -> Self {
        Self {
            tenant,
            object: Some(object),
            relation: Some(relation),
            subject: None,
        }
    }

    #[must_use]
    pub fn matches(&self, tuple: &RebacTuple) -> bool {
        self.tenant == tuple.tenant
            && self
                .object
                .as_ref()
                .is_none_or(|object| object == &tuple.object)
            && self
                .relation
                .as_ref()
                .is_none_or(|relation| relation == &tuple.relation)
            && self
                .subject
                .as_ref()
                .is_none_or(|subject| subject == &tuple.subject)
    }
}

/// One page returned by the tuple-store port.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RebacTuplePage {
    pub tuples: Vec<RebacTuple>,         // data_class: TENANT_SCOPED
    pub snapshot: SnapshotToken,         // data_class: INTERNAL_ONLY
    pub next_page_token: Option<String>, // data_class: INTERNAL_ONLY
}

/// Tuple-store port: adapters own persistence; the domain crate owns the contract.
pub trait RebacTupleStore {
    fn write_tuple(&mut self, tuple: RebacTuple) -> Result<Zookie, RebacTupleStoreError>;

    fn read_tuples(
        &self,
        query: &RebacTupleQuery,
        snapshot: RebacReadSnapshot,
    ) -> Result<RebacTuplePage, RebacTupleStoreError>;
}

/// Fail-closed tuple-store port errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RebacTupleStoreError {
    InvalidTuple(RebacTupleValidationError),
    InvalidZookie(RebacTupleValidationError),
    StaleSnapshot {
        requested: SnapshotToken,
        current: SnapshotToken,
    },
    Backend(String),
}

impl fmt::Display for RebacTupleStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidTuple(err) => write!(f, "invalid ReBAC tuple: {err}"),
            Self::InvalidZookie(err) => write!(f, "invalid ReBAC zookie: {err}"),
            Self::StaleSnapshot { requested, current } => write!(
                f,
                "stale ReBAC snapshot: requested {} but current is {}",
                requested.as_str(),
                current.as_str()
            ),
            Self::Backend(detail) => write!(f, "ReBAC tuple-store backend error: {detail}"),
        }
    }
}

impl std::error::Error for RebacTupleStoreError {}

/// Zanzibar/OpenFGA-style userset rewrite tree.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum UsersetRewrite {
    This,
    ComputedUserset {
        relation: RebacRelation, // data_class: INTERNAL_ONLY
    },
    TupleToUserset {
        tupleset_relation: RebacRelation, // data_class: INTERNAL_ONLY
        computed_userset_relation: RebacRelation, // data_class: INTERNAL_ONLY
    },
    Union {
        children: Vec<UsersetRewrite>, // data_class: INTERNAL_ONLY
    },
    Intersection {
        children: Vec<UsersetRewrite>, // data_class: INTERNAL_ONLY
    },
    Difference {
        base: Box<UsersetRewrite>,     // data_class: INTERNAL_ONLY
        subtract: Box<UsersetRewrite>, // data_class: INTERNAL_ONLY
    },
}

impl UsersetRewrite {
    #[must_use]
    pub fn this() -> Self {
        Self::This
    }

    #[must_use]
    pub fn computed_userset(relation: RebacRelation) -> Self {
        Self::ComputedUserset { relation }
    }

    #[must_use]
    pub fn tuple_to_userset(
        tupleset_relation: RebacRelation,
        computed_userset_relation: RebacRelation,
    ) -> Self {
        Self::TupleToUserset {
            tupleset_relation,
            computed_userset_relation,
        }
    }

    pub fn union(children: Vec<Self>) -> Result<Self, RebacTupleValidationError> {
        if children.is_empty() {
            return Err(RebacTupleValidationError::EmptyRewrite { kind: "union" });
        }
        let rewrite = Self::Union { children };
        rewrite.validate()?;
        Ok(rewrite)
    }

    pub fn intersection(children: Vec<Self>) -> Result<Self, RebacTupleValidationError> {
        if children.is_empty() {
            return Err(RebacTupleValidationError::EmptyRewrite {
                kind: "intersection",
            });
        }
        let rewrite = Self::Intersection { children };
        rewrite.validate()?;
        Ok(rewrite)
    }

    #[must_use]
    pub fn difference(base: Self, subtract: Self) -> Self {
        Self::Difference {
            base: Box::new(base),
            subtract: Box::new(subtract),
        }
    }

    pub fn validate(&self) -> Result<(), RebacTupleValidationError> {
        match self {
            Self::This | Self::ComputedUserset { .. } | Self::TupleToUserset { .. } => Ok(()),
            Self::Union { children } => validate_children("union", children),
            Self::Intersection { children } => validate_children("intersection", children),
            Self::Difference { base, subtract } => {
                base.validate()?;
                subtract.validate()
            }
        }
    }
}

fn validate_children(
    kind: &'static str,
    children: &[UsersetRewrite],
) -> Result<(), RebacTupleValidationError> {
    if children.is_empty() {
        return Err(RebacTupleValidationError::EmptyRewrite { kind });
    }
    for child in children {
        child.validate()?;
    }
    Ok(())
}

fn validate_object_type(field: &'static str, value: &str) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_tenant_scope(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_relation(field: &'static str, value: &str) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_tuple_segment(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value
        .chars()
        .all(|c| !c.is_whitespace() && c != '#' && c != '@')
    {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_opaque_token(
    field: &'static str,
    value: &str,
) -> Result<(), RebacTupleValidationError> {
    validate_non_empty(field, value)?;
    if value.chars().all(|c| !c.is_whitespace()) {
        Ok(())
    } else {
        Err(RebacTupleValidationError::InvalidToken {
            field,
            value: value.to_owned(),
        })
    }
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), RebacTupleValidationError> {
    if value.is_empty() {
        Err(RebacTupleValidationError::EmptyField { field })
    } else {
        Ok(())
    }
}
