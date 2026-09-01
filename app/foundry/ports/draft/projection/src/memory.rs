//! The reference in-memory store: the contract's executable meaning.
//! Validation happens before any mutation, so every apply is atomic by
//! construction.

use std::collections::BTreeMap;

use crate::keys::KeyDesignations;
use crate::predicate::PropertyPredicate;
use crate::store::{
    AppliedEntry, ApplyReceipt, EntryOutcome, Page, PageRequest, ProjectedObject, ProjectionCursor,
    ProjectionStore, ProjectionStoreError,
};

#[derive(Debug, Default)]
pub struct MemoryProjectionStore {
    heads: BTreeMap<String, u64>,
    entries: BTreeMap<(String, u64), AppliedEntry>,
    objects: BTreeMap<(String, String), ProjectedObject>,
    poisons: BTreeMap<(String, u64), String>,
}

impl MemoryProjectionStore {
    fn validate(entry: &AppliedEntry) -> Result<(), ProjectionStoreError> {
        require_trimmed(&entry.tenant_id, "blank entry tenant")?;
        match &entry.outcome {
            EntryOutcome::Applied { objects } => {
                for object in objects {
                    if object.entity.tenant_id != entry.tenant_id {
                        return Err(ProjectionStoreError::Entry {
                            detail: "object outside the entry's tenant",
                        });
                    }
                    require_trimmed(&object.last_actor, "blank object actor")?;
                }
            }
            EntryOutcome::Poisoned { reason } => {
                require_trimmed(reason, "blank poison reason")?;
            }
        }
        Ok(())
    }

    /// Refuse an object that cannot be identified, or whose key is
    /// already held — by a stored object OR by an earlier object in the
    /// same entry, which a store-only scan would miss.
    fn check_key(
        &self,
        object: &ProjectedObject,
        keys: &KeyDesignations,
        earlier_in_entry: &[ProjectedObject],
    ) -> Result<(), ProjectionStoreError> {
        let entity_type = object.entity.entity_type.value.as_str();
        let Some(property) = keys.property_for(entity_type) else {
            return Ok(());
        };
        let Some(held) = object.entity.properties.get(property) else {
            return Err(ProjectionStoreError::MissingPrimaryKey {
                property: property.to_owned(),
            });
        };
        let value = &held.value.value;
        if matches!(
            value,
            data_ontology_kernel::PropertyValue::Array(_)
                | data_ontology_kernel::PropertyValue::Struct(_)
        ) {
            return Err(ProjectionStoreError::NonScalarPrimaryKey {
                property: property.to_owned(),
            });
        }

        let clash = earlier_in_entry
            .iter()
            .map(|candidate| (candidate.entity.id.as_str(), candidate))
            .chain(
                self.objects
                    .range((object.entity.tenant_id.clone(), String::new())..)
                    .take_while(|((tenant, _), _)| tenant == &object.entity.tenant_id)
                    .map(|((_, object_ref), candidate)| (object_ref.as_str(), candidate)),
            )
            .find(|(object_ref, candidate)| {
                *object_ref != object.entity.id
                    && candidate.entity.entity_type.value == entity_type
                    && candidate
                        .entity
                        .properties
                        .get(property)
                        .is_some_and(|stored| &stored.value.value == value)
            });
        if let Some((held_by, _)) = clash {
            return Err(ProjectionStoreError::DuplicatePrimaryKey {
                property: property.to_owned(),
                held_by: held_by.to_owned(),
            });
        }
        Ok(())
    }

    fn scan(
        &self,
        tenant_id: &str,
        entity_type: &str,
        predicate: Option<&PropertyPredicate>,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        require_trimmed(tenant_id, "blank tenant")?;
        if page.limit == 0 {
            return Err(ProjectionStoreError::Entry {
                detail: "zero page limit",
            });
        }
        // Kind drift refuses window-independently: the whole type scope
        // is checked before any cursor or limit narrows the walk.
        if let Some(predicate) = predicate
            && let Some((property, kind)) = predicate.range_kind()
        {
            for (_, object) in self
                .objects
                .range((tenant_id.to_owned(), String::new())..)
                .take_while(|((tenant, _), _)| tenant == tenant_id)
            {
                if object.entity.entity_type.value != entity_type {
                    continue;
                }
                if let Some(stored) = object.entity.properties.get(property)
                    && stored.value.value.type_label() != kind
                {
                    return Err(ProjectionStoreError::KindMismatch {
                        property: property.to_owned(),
                    });
                }
            }
        }
        let after = page
            .cursor
            .as_ref()
            .map(|cursor| cursor.after_object_ref.as_str());
        let mut objects = Vec::new();
        let mut next = None;
        for ((_, object_ref), object) in self
            .objects
            .range((tenant_id.to_owned(), String::new())..)
            .take_while(|((tenant, _), _)| tenant == tenant_id)
        {
            if object.entity.entity_type.value != entity_type {
                continue;
            }
            if let Some(after) = after
                && object_ref.as_str() <= after
            {
                continue;
            }
            if let Some(predicate) = predicate
                && !predicate.matches(&object.entity)?
            {
                continue;
            }
            if objects.len() == page.limit {
                next = Some(ProjectionCursor {
                    after_object_ref: objects
                        .last()
                        .map(|last: &ProjectedObject| last.entity.id.clone())
                        .unwrap_or_default(),
                });
                break;
            }
            objects.push(object.clone());
        }
        Ok(Page { objects, next })
    }
}

impl ProjectionStore for MemoryProjectionStore {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        Self::validate(&entry)?;
        let head = self.heads.get(&entry.tenant_id).copied().unwrap_or(0);
        if entry.ordinal <= head {
            let key = (entry.tenant_id.clone(), entry.ordinal);
            return match self.entries.get(&key) {
                Some(stored) if *stored == entry => Ok(ApplyReceipt {
                    ordinal: entry.ordinal,
                    deduplicated: true,
                }),
                _ => Err(ProjectionStoreError::DivergentReplay {
                    ordinal: entry.ordinal,
                }),
            };
        }
        if entry.ordinal != head + 1 {
            return Err(ProjectionStoreError::NonDenseOrdinal {
                expected: head + 1,
                found: entry.ordinal,
            });
        }
        match &entry.outcome {
            EntryOutcome::Applied { objects } => {
                // KEY PASS — every object is cleared before ANY is
                // written, so a refused duplicate leaves nothing behind.
                if !keys.is_empty() {
                    for (index, object) in objects.iter().enumerate() {
                        self.check_key(object, keys, &objects[..index])?;
                    }
                }
                for object in objects {
                    self.objects.insert(
                        (entry.tenant_id.clone(), object.entity.id.clone()),
                        object.clone(),
                    );
                }
            }
            EntryOutcome::Poisoned { reason } => {
                self.poisons
                    .insert((entry.tenant_id.clone(), entry.ordinal), reason.clone());
            }
        }
        self.heads.insert(entry.tenant_id.clone(), entry.ordinal);
        let receipt = ApplyReceipt {
            ordinal: entry.ordinal,
            deduplicated: false,
        };
        self.entries
            .insert((entry.tenant_id.clone(), entry.ordinal), entry);
        Ok(receipt)
    }

    fn applied_head(&self, tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        require_trimmed(tenant_id, "blank tenant")?;
        Ok(self.heads.get(tenant_id).copied().unwrap_or(0))
    }

    fn get(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        require_trimmed(tenant_id, "blank tenant")?;
        Ok(self
            .objects
            .get(&(tenant_id.to_owned(), object_ref.to_owned()))
            .cloned())
    }

    fn objects_of_type(
        &self,
        tenant_id: &str,
        entity_type: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.scan(tenant_id, entity_type, None, page)
    }

    fn filter(
        &self,
        tenant_id: &str,
        entity_type: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.scan(tenant_id, entity_type, Some(predicate), page)
    }

    fn poisoned(&self, tenant_id: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        require_trimmed(tenant_id, "blank tenant")?;
        Ok(self
            .poisons
            .range((tenant_id.to_owned(), 0)..)
            .take_while(|((tenant, _), _)| tenant == tenant_id)
            .map(|((_, ordinal), reason)| (*ordinal, reason.clone()))
            .collect())
    }
}

fn require_trimmed(value: &str, detail: &'static str) -> Result<(), ProjectionStoreError> {
    if value.trim().is_empty() || value.trim() != value {
        return Err(ProjectionStoreError::Entry { detail });
    }
    Ok(())
}
