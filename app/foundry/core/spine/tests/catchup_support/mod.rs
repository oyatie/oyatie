//! Fixtures for the catch-up law: a registry, and log entries whose
//! content can be varied one field at a time so a divergent log is
//! divergent for exactly one reason.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    ActionTypeDefinition, ActionTypeId, AutonomyTier, EntityTypeDefinition, EntityTypeId,
    EntityTypePropertyDefinition, LinkCardinality, LinkTypeDefinition, LinkTypeId, OntologyEngine,
    PropertyTier,
};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    encode_action_record,
};
use foundry_projection_draft::{
    AppliedEntry, ApplyReceipt, KeyDesignations, MemoryProjectionStore, Page, PageRequest,
    ProjectedLink, ProjectedObject, ProjectionStore, ProjectionStoreError, PropertyPredicate,
};
use foundry_records_draft::{ActionEnvelope, Receipt, SealedEnvelope};

pub(crate) const TENANT: &str = "ten_test";

/// A store whose head cannot be read — the disk is there, the answer is
/// not. Catch-up must refuse rather than read the failure as "empty",
/// which would rebuild the whole log over unknown contents.
pub(crate) struct UnreadableHead {
    pub(crate) inner: MemoryProjectionStore,
}

impl ProjectionStore for UnreadableHead {
    fn apply(
        &mut self,
        entry: AppliedEntry,
        keys: &KeyDesignations,
    ) -> Result<ApplyReceipt, ProjectionStoreError> {
        self.inner.apply(entry, keys)
    }

    fn applied_head(&self, _tenant_id: &str) -> Result<u64, ProjectionStoreError> {
        Err(ProjectionStoreError::Storage {
            detail: "head unreadable".to_owned(),
        })
    }

    fn get(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Option<ProjectedObject>, ProjectionStoreError> {
        self.inner.get(tenant_id, object_ref)
    }

    fn objects_of_type(
        &self,
        tenant_id: &str,
        entity_type: &str,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.objects_of_type(tenant_id, entity_type, page)
    }

    fn filter(
        &self,
        tenant_id: &str,
        entity_type: &str,
        predicate: &PropertyPredicate,
        page: &PageRequest,
    ) -> Result<Page, ProjectionStoreError> {
        self.inner.filter(tenant_id, entity_type, predicate, page)
    }

    fn links_from(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_from(tenant_id, object_ref)
    }

    fn links_to(
        &self,
        tenant_id: &str,
        object_ref: &str,
    ) -> Result<Vec<ProjectedLink>, ProjectionStoreError> {
        self.inner.links_to(tenant_id, object_ref)
    }

    fn poisoned(&self, tenant_id: &str) -> Result<Vec<(u64, String)>, ProjectionStoreError> {
        self.inner.poisoned(tenant_id)
    }
}

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

/// `ety_reading` keys on `name`, so a rebuild that reached the store
/// without the registry's key designation would lose identity law —
/// which is why catch-up folds through the projector rather than
/// copying rows.
pub(crate) fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    let definition = EntityTypeDefinition::new(
        TENANT,
        EntityTypeId::new("ety_reading").unwrap(),
        "Reading",
        vec![
            EntityTypePropertyDefinition::new("name", PropertyTier::Scalar, internal(), true)
                .unwrap(),
        ],
        1,
    )
    .unwrap()
    .with_primary_key_property("name");
    engine.register_entity_type(definition).unwrap();
    engine
        .register_link_type(
            LinkTypeDefinition::new(
                TENANT,
                LinkTypeId::new("lty_measures").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                LinkCardinality::ManyToMany,
                false,
            )
            .unwrap(),
        )
        .unwrap();
    engine
        .register_action_type(
            ActionTypeDefinition::new(
                TENANT,
                ActionTypeId::new("aty_calibrate").unwrap(),
                EntityTypeId::new("ety_reading").unwrap(),
                "ops-console",
                AutonomyTier::T1Assist,
                "reading.calibrated",
            )
            .unwrap(),
        )
        .unwrap();
    engine
}

fn name_property(value: &str) -> WireProperty {
    WireProperty::new(
        "name",
        WireTier::Scalar,
        WireDataClass::InternalOnly,
        WireValue::String(value.into()),
    )
    .unwrap()
}

fn envelope(actor: &str, object_ref: &str, ordinal: u64, name: &str) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    let record = ActionRecord::new(
        actor,
        "dec_1",
        "reading.calibrated",
        &key,
        1_700_000_000_000,
        vec![],
        EditSet::new(vec![
            OntologyEdit::create_object("ety_reading", vec![name_property(name)]).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            TENANT,
            object_ref,
            "aty_calibrate",
            &key,
            1,
            encode_action_record(&record),
            1_700_000_000_000,
        )
        .unwrap(),
        receipt: Receipt {
            ordinal,
            object_sequence: 1,
            deduplicated: false,
        },
    }
}

/// One well-formed entry: object `ent_{ordinal}` named `name`.
pub(crate) fn sealed(ordinal: u64, name: &str) -> SealedEnvelope {
    envelope("prn_alice", &format!("ent_{ordinal}"), ordinal, name)
}

/// The same entry written by a different principal. Differs from
/// [`sealed`] ONLY in `last_actor`, which reaches the store through
/// `ProjectedObject` — so a store built from the other log diverges
/// here without also tripping the primary-key law, and the refusal
/// under test is unambiguous.
pub(crate) fn sealed_by_another_actor(ordinal: u64, name: &str) -> SealedEnvelope {
    envelope("prn_bob", &format!("ent_{ordinal}"), ordinal, name)
}

/// Payload bytes the decoder refuses. Poisons identically on every
/// replay, because a poison derives from (log bytes, registry).
pub(crate) fn corrupt(ordinal: u64) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            TENANT,
            format!("ent_{ordinal}"),
            "aty_calibrate",
            &key,
            1,
            vec![0xFF],
            1_700_000_000_000,
        )
        .unwrap(),
        receipt: Receipt {
            ordinal,
            object_sequence: 1,
            deduplicated: false,
        },
    }
}

/// An entry that registers an outbound edge. FROM is the envelope's own
/// object, per spine law.
pub(crate) fn sealed_link(ordinal: u64, from_ref: &str, to_ref: &str) -> SealedEnvelope {
    let key = format!("idem_{ordinal}");
    let record = ActionRecord::new(
        "prn_alice",
        "dec_1",
        "reading.calibrated",
        &key,
        1_700_000_000_000,
        vec![],
        EditSet::new(vec![
            OntologyEdit::create_link("lty_measures", to_ref).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    SealedEnvelope {
        envelope: ActionEnvelope::new(
            TENANT,
            from_ref,
            "aty_calibrate",
            &key,
            1,
            encode_action_record(&record),
            1_700_000_000_000,
        )
        .unwrap(),
        receipt: Receipt {
            ordinal,
            object_sequence: 1,
            deduplicated: false,
        },
    }
}

/// A three-entry log, dense from ordinal 1.
pub(crate) fn log() -> Vec<SealedEnvelope> {
    vec![sealed(1, "one"), sealed(2, "two"), sealed(3, "three")]
}

/// A log that exercises objects, an edge, and a poison together — so an
/// equivalence claim over it is not vacuous on any of the three.
pub(crate) fn mixed_log() -> Vec<SealedEnvelope> {
    vec![
        sealed(1, "one"),
        sealed(2, "two"),
        sealed_link(3, "ent_1", "ent_2"),
        corrupt(4),
        sealed(5, "five"),
    ]
}
