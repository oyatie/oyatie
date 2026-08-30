//! Tuple identity: tenant scope, object and relation references, the
//! subject side of a userset, and the canonical `object#relation@subject` form.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::RebacTupleValidationError;
use super::validate::{
    validate_object_type, validate_relation, validate_tenant_scope, validate_tuple_segment,
};

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
