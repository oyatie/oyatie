//! Explicit sample data only: no I/O, credentials, grants or live fallback.

#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use application_ontology_read::{
    AuditRow, EntitySchema, EntityType, EntityTypeLabel, HistoryRow, LinkSchema, ObjectDetail,
    ObjectListQuery, ObjectPage, OntologyReadError, OntologyReadSource, UpcastState,
};

mod filtering;
mod fixture;

#[derive(Clone, Copy, Debug)]
pub struct FixtureOntologyReadSource {
    refusal: Option<OntologyReadError>,
}

impl FixtureOntologyReadSource {
    pub const TENANT: &'static str = "ten_fixture_ontology";
    pub const ENTITY_TYPE: &'static str = "ety_fixture_record";

    pub const fn sample() -> Self {
        Self { refusal: None }
    }

    pub const fn refusing(error: OntologyReadError) -> Self {
        Self {
            refusal: Some(error),
        }
    }

    fn check(&self, tenant: &str) -> Result<(), OntologyReadError> {
        if let Some(error) = self.refusal {
            return Err(error);
        }
        if tenant != Self::TENANT {
            return Err(OntologyReadError::Forbidden);
        }
        Ok(())
    }
}

fn check_revision(revision: u32) -> Result<(), OntologyReadError> {
    if !matches!(revision, 1 | 2) {
        return Err(OntologyReadError::RevisionConflict);
    }
    Ok(())
}

fn pin(mut object: application_ontology_read::PinnedObject, revision: u32) -> ObjectDetail {
    let properties = fixture::properties(revision);
    object
        .properties
        .retain(|name, _| properties.contains(name));
    object.upcast_state = if object.written_revision < revision {
        UpcastState::UpcastPending
    } else {
        UpcastState::Current
    };
    ObjectDetail {
        requested_revision: revision,
        object,
    }
}

impl OntologyReadSource for FixtureOntologyReadSource {
    async fn type_labels(&self, tenant: &str) -> Result<Vec<EntityTypeLabel>, OntologyReadError> {
        self.check(tenant)?;
        Ok(vec![EntityTypeLabel {
            entity_type: Self::ENTITY_TYPE.to_owned(),
            revision: 2,
            display_name: "Fixture record".to_owned(),
        }])
    }

    async fn link_types(&self, tenant: &str) -> Result<Vec<LinkSchema>, OntologyReadError> {
        self.check(tenant)?;
        Ok(vec![])
    }

    async fn schemas(&self, tenant: &str) -> Result<Vec<EntitySchema>, OntologyReadError> {
        self.check(tenant)?;
        Ok(vec![fixture::schema()])
    }

    async fn types(&self, tenant: &str) -> Result<Vec<EntityType>, OntologyReadError> {
        self.check(tenant)?;
        Ok(vec![EntityType {
            entity_type: Self::ENTITY_TYPE.to_owned(),
            revision: 2,
            properties: fixture::properties(2),
        }])
    }

    async fn objects(
        &self,
        tenant: &str,
        query: &ObjectListQuery,
    ) -> Result<ObjectPage, OntologyReadError> {
        self.check(tenant)?;
        query.validate()?;
        if query.entity_type != Self::ENTITY_TYPE {
            return Err(OntologyReadError::InvalidQuery);
        }
        check_revision(query.revision)?;
        let mut objects = fixture::objects();
        filtering::apply(&mut objects, query)?;
        objects.retain(|object| {
            query
                .cursor
                .as_ref()
                .is_none_or(|cursor| &object.object_ref > cursor)
        });
        let has_more = objects.len() > query.limit;
        objects.truncate(query.limit);
        let next = has_more
            .then(|| objects.last().map(|object| object.object_ref.clone()))
            .flatten();
        Ok(ObjectPage {
            requested_revision: query.revision,
            objects: objects
                .into_iter()
                .map(|object| pin(object, query.revision).object)
                .collect(),
            next,
        })
    }

    async fn object(
        &self,
        tenant: &str,
        object_ref: &str,
        revision: u32,
    ) -> Result<ObjectDetail, OntologyReadError> {
        self.check(tenant)?;
        let object = fixture::objects()
            .into_iter()
            .find(|object| object.object_ref == object_ref)
            .ok_or(OntologyReadError::AbsentObject)?;
        check_revision(revision)?;
        Ok(pin(object, revision))
    }

    async fn history(
        &self,
        tenant: &str,
        object_ref: &str,
    ) -> Result<Vec<HistoryRow>, OntologyReadError> {
        self.check(tenant)?;
        Ok(fixture::history(object_ref))
    }

    async fn audit(&self, tenant: &str) -> Result<Vec<AuditRow>, OntologyReadError> {
        self.check(tenant)?;
        Ok(fixture::audit())
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod label_tests;
