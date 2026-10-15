use crate::{OntologyEngineError, definitions::validate_ontology_tenant};
use data_boundary_kernel::{Classified, DataClass};
use std::{collections::BTreeSet, sync::Arc};

pub const MAX_NAMED_STRING_MEMBERS: usize = 256;
pub const MAX_NAMED_STRING_TEXT_BYTES: usize = 4096;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NamedStringTypeId(String);
impl NamedStringTypeId {
    pub fn new(value: impl Into<String>) -> Result<Self, OntologyEngineError> {
        let value = value.into();
        if !value.starts_with("vty_")
            || value.len() <= 4
            || value.len() > MAX_NAMED_STRING_TEXT_BYTES
        {
            return Err(OntologyEngineError::InvalidNamedStringTypeId);
        }
        Ok(Self(value))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct NamedStringTypeRef {
    tenant_id: String,
    id: NamedStringTypeId,
    revision: u32,
}
impl NamedStringTypeRef {
    pub fn new(
        tenant_id: impl Into<String>,
        id: NamedStringTypeId,
        revision: u32,
    ) -> Result<Self, OntologyEngineError> {
        let tenant_id = tenant_id.into();
        validate_ontology_tenant(&tenant_id)?;
        if revision == 0 {
            return Err(OntologyEngineError::InvalidNamedStringRevision);
        }
        Ok(Self {
            tenant_id,
            id,
            revision,
        })
    }
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    pub fn id(&self) -> &NamedStringTypeId {
        &self.id
    }
    pub const fn revision(&self) -> u32 {
        self.revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NamedStringTypeDefinition(Arc<NamedStringTypeContents>);

#[derive(Debug, Eq, PartialEq)]
struct NamedStringTypeContents {
    reference: NamedStringTypeRef,
    display_name: Classified<String>,
    members: BTreeSet<String>,
}
impl NamedStringTypeDefinition {
    pub fn new(
        reference: NamedStringTypeRef,
        display_name: impl Into<String>,
        members: Vec<String>,
    ) -> Result<Self, OntologyEngineError> {
        let display_name = display_name.into();
        if display_name.trim().is_empty() || display_name.len() > MAX_NAMED_STRING_TEXT_BYTES {
            return Err(OntologyEngineError::EmptyDisplayName);
        }
        if members.is_empty() || members.len() > MAX_NAMED_STRING_MEMBERS {
            return Err(OntologyEngineError::InvalidNamedStringMembers);
        }
        let mut unique = BTreeSet::new();
        for member in members {
            if member.trim().is_empty()
                || member.len() > MAX_NAMED_STRING_TEXT_BYTES
                || !unique.insert(member)
            {
                return Err(OntologyEngineError::InvalidNamedStringMembers);
            }
        }
        Ok(Self(Arc::new(NamedStringTypeContents {
            reference,
            display_name: Classified::new(display_name, DataClass::InternalOnly),
            members: unique,
        })))
    }
    pub fn reference(&self) -> &NamedStringTypeRef {
        &self.0.reference
    }
    pub fn display_name(&self) -> &Classified<String> {
        &self.0.display_name
    }
    pub fn members(&self) -> &BTreeSet<String> {
        &self.0.members
    }
}
