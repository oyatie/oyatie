use crate::OntologyReadError;

pub const DEFAULT_PAGE_LIMIT: usize = 100;
pub const MAX_PAGE_LIMIT: usize = 1000;

/// Only literal kinds admitted by the native listing query grammar.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum FilterValue {
    Text(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PropertyFilter {
    Equals {
        property: String,   // data_class: INTERNAL_ONLY
        value: FilterValue, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    },
    Range {
        property: String,  // data_class: INTERNAL_ONLY
        from: FilterValue, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
        to: FilterValue,   // data_class: PROPERTY_VALUE_PRIVACY_CLASS
    },
}

impl PropertyFilter {
    pub fn property(&self) -> &str {
        match self {
            Self::Equals { property, .. } | Self::Range { property, .. } => property,
        }
    }

    pub fn validate(&self) -> Result<(), OntologyReadError> {
        let property = self.property();
        if property.trim().is_empty() || property.trim() != property {
            return Err(OntologyReadError::InvalidQuery);
        }
        if let Self::Range { from, to, .. } = self
            && (std::mem::discriminant(from) != std::mem::discriminant(to) || from > to)
        {
            return Err(OntologyReadError::InvalidQuery);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObjectListQuery {
    pub entity_type: String,            // data_class: INTERNAL_ONLY
    pub revision: u32,                  // data_class: INTERNAL_ONLY
    pub limit: usize,                   // data_class: INTERNAL_ONLY
    pub cursor: Option<String>,         // data_class: TENANT_SCOPED
    pub filter: Option<PropertyFilter>, // data_class: PROPERTY_VALUE_PRIVACY_CLASS
}

impl ObjectListQuery {
    pub fn first(entity_type: impl Into<String>, revision: u32) -> Self {
        Self {
            entity_type: entity_type.into(),
            revision,
            limit: DEFAULT_PAGE_LIMIT,
            cursor: None,
            filter: None,
        }
    }

    /// Local shape checks, never credential or product authorization.
    pub fn validate(&self) -> Result<(), OntologyReadError> {
        if !(1..=MAX_PAGE_LIMIT).contains(&self.limit)
            || self
                .cursor
                .as_ref()
                .is_some_and(|cursor| !cursor.starts_with("ent_"))
        {
            return Err(OntologyReadError::InvalidQuery);
        }
        if let Some(filter) = &self.filter {
            filter.validate()?;
        }
        Ok(())
    }
}
