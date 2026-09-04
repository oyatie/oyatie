use std::collections::{BTreeMap, BTreeSet};

use cell_region::RegionCode;

use crate::{error::CloudResourceError, kind::ResourceKind};

const RESOURCE_ID_PREFIX_OWNER: &str = "oyatie";
const RESOURCE_ID_PREFIX_SERVICE: &str = "cloud";
const TENANT_ID_PREFIX: &str = "ten_";
const HUMAN_PRINCIPAL_PREFIX: &str = "usr_";
const SERVICE_PRINCIPAL_PREFIX: &str = "sp_";
const POLICY_ID_PREFIX: &str = "pol_";
const RESERVED_TAG_PREFIX: &str = "oyatie:";

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ResourceId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct PrincipalId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct IamPolicyId {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TagKey {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TagValue {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct MeteringTag {
    pub value: String, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResourceIdParts {
    pub(crate) region: RegionCode, // data_class: PUBLIC
    pub(crate) tenant_id: String,  // data_class: INTERNAL_ONLY
    pub(crate) kind_label: String, // data_class: PUBLIC
    pub(crate) name: String,       // data_class: INTERNAL_ONLY
}

impl ResourceId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        parse_resource_id(&value)?;
        Ok(Self { value })
    }

    pub fn tenant_id(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.tenant_id)
    }

    pub fn region(&self) -> Result<RegionCode, CloudResourceError> {
        Ok(self.parts()?.region)
    }

    pub fn kind_label(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.kind_label)
    }

    pub fn resource_name(&self) -> Result<String, CloudResourceError> {
        Ok(self.parts()?.name)
    }

    pub(crate) fn parts(&self) -> Result<ResourceIdParts, CloudResourceError> {
        parse_resource_id(&self.value)
    }
}

impl PrincipalId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if (value.starts_with(HUMAN_PRINCIPAL_PREFIX) && value.len() > HUMAN_PRINCIPAL_PREFIX.len())
            || (value.starts_with(SERVICE_PRINCIPAL_PREFIX)
                && value.len() > SERVICE_PRINCIPAL_PREFIX.len())
        {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidPrincipalId)
        }
    }
}

impl IamPolicyId {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.starts_with(POLICY_ID_PREFIX) && value.len() > POLICY_ID_PREFIX.len() {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidPolicyId)
        }
    }
}

impl TagKey {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.trim().is_empty()
            || value.starts_with(RESERVED_TAG_PREFIX)
            || !value.bytes().all(is_tag_byte)
        {
            return Err(CloudResourceError::InvalidTagKey);
        }
        Ok(Self { value })
    }
}

impl TagValue {
    pub fn new(value: impl Into<String>) -> Result<Self, CloudResourceError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 256 {
            return Err(CloudResourceError::InvalidTagValue);
        }
        Ok(Self { value })
    }
}

impl MeteringTag {
    pub fn new(
        value: impl Into<String>,
        tenant_id: &str,
        kind: ResourceKind,
    ) -> Result<Self, CloudResourceError> {
        let value = value.into();
        let expected = format!("oyatie:metering:{tenant_id}:{}", kind.type_label());
        if value == expected {
            Ok(Self { value })
        } else {
            Err(CloudResourceError::InvalidMeteringTag)
        }
    }
}

fn parse_resource_id(value: &str) -> Result<ResourceIdParts, CloudResourceError> {
    let parts: Vec<&str> = value.split(':').collect();
    if parts.len() != 6
        || parts[0] != RESOURCE_ID_PREFIX_OWNER
        || parts[1] != RESOURCE_ID_PREFIX_SERVICE
        || parts.iter().any(|part| part.trim().is_empty())
    {
        return Err(CloudResourceError::InvalidResourceId);
    }
    let region = RegionCode::new(parts[2]).map_err(|_| CloudResourceError::InvalidResourceId)?;
    validate_tenant_id(parts[3])?;
    validate_canonical_segment(parts[4], CloudResourceError::InvalidResourceId)?;
    validate_canonical_segment(parts[5], CloudResourceError::InvalidResourceId)?;
    Ok(ResourceIdParts {
        region,
        tenant_id: parts[3].to_string(),
        kind_label: parts[4].to_string(),
        name: parts[5].to_string(),
    })
}

pub(crate) fn typed_tags(
    tags: BTreeMap<String, String>,
) -> Result<BTreeMap<TagKey, TagValue>, CloudResourceError> {
    tags.into_iter()
        .map(|(key, value)| Ok((TagKey::new(key)?, TagValue::new(value)?)))
        .collect()
}

pub(crate) fn typed_policy_ids(
    values: Vec<String>,
) -> Result<Vec<IamPolicyId>, CloudResourceError> {
    let mut seen = BTreeSet::new();
    let mut typed = Vec::with_capacity(values.len());
    for value in values {
        let policy_id = IamPolicyId::new(value)?;
        if !seen.insert(policy_id.clone()) {
            return Err(CloudResourceError::DuplicatePolicyId);
        }
        typed.push(policy_id);
    }
    Ok(typed)
}

pub(crate) fn validate_tenant_id(value: &str) -> Result<(), CloudResourceError> {
    if value.starts_with(TENANT_ID_PREFIX) && value.len() > TENANT_ID_PREFIX.len() {
        Ok(())
    } else {
        Err(CloudResourceError::InvalidTenantId)
    }
}

fn validate_canonical_segment(
    value: &str,
    error: CloudResourceError,
) -> Result<(), CloudResourceError> {
    if value.trim().is_empty()
        || value.starts_with('-')
        || value.ends_with('-')
        || value.contains("--")
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(error);
    }
    Ok(())
}

fn is_tag_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'/' | b':')
}
