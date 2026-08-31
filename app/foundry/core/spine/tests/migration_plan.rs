//! Migration-plan law: a plan validates only against the registry head it
//! names, every source must exist at the from-revision and every target
//! must be an optional non-key property at the to-revision, transforms are
//! type-checked with no parses, and the plan digest is fixed-width over
//! unbounded inputs so the runner's idempotency key can never overflow the
//! envelope cap.

use data_boundary_kernel::{DataClass, PrivacyDataClass};
use data_ontology_kernel::{
    EntityTypeDefinition, EntityTypeId, EntityTypePropertyDefinition, OntologyEngine, PropertyTier,
    ScalarType, ValueTypeDeclaration,
};
use foundry_spine::{DefaultValue, MigrationPlan, PlanError, UpcastTransform, ValueConversion};

fn internal() -> PrivacyDataClass {
    PrivacyDataClass::try_from(DataClass::InternalOnly).unwrap()
}

fn untyped(name: &str, required: bool) -> EntityTypePropertyDefinition {
    EntityTypePropertyDefinition::new(name, PropertyTier::Scalar, internal(), required).unwrap()
}

fn typed(name: &str, scalar: ScalarType) -> EntityTypePropertyDefinition {
    let mut property =
        EntityTypePropertyDefinition::new(name, PropertyTier::Scalar, internal(), false).unwrap();
    property.value_type = Some(ValueTypeDeclaration::Scalar(scalar));
    property
}

fn rev1_properties() -> Vec<EntityTypePropertyDefinition> {
    vec![
        untyped("serial", true),
        untyped("note", true),
        typed("score", ScalarType::Integer),
        typed("flag", ScalarType::Boolean),
    ]
}

fn rev2_properties() -> Vec<EntityTypePropertyDefinition> {
    let mut properties = rev1_properties();
    properties.push(typed("score_text", ScalarType::String));
    properties.push(typed("score_copy", ScalarType::Integer));
    properties.push(typed("flag_rank", ScalarType::Integer));
    properties.push(untyped("grade", false));
    properties
}

fn definition(
    revision: u32,
    properties: Vec<EntityTypePropertyDefinition>,
) -> EntityTypeDefinition {
    EntityTypeDefinition::new(
        "ten_test",
        EntityTypeId::new("ety_reading").unwrap(),
        "Reading",
        properties,
        revision,
    )
    .unwrap()
    .with_primary_key_property("serial")
}

/// Revision 1 -> revision 2, both retained; head is 2.
fn registry() -> OntologyEngine {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(definition(1, rev1_properties()))
        .unwrap();
    engine
        .evolve_entity_type(definition(2, rev2_properties()))
        .unwrap();
    engine
}

fn plan(transforms: Vec<UpcastTransform>) -> MigrationPlan {
    MigrationPlan {
        tenant_id: "ten_test".into(),
        entity_type: "ety_reading".into(),
        from_revision: 1,
        to_revision: 2,
        action_type: "aty_upcast_reading_2".into(),
        audit_event_type: "reading.upcast_to_2".into(),
        declared_at_epoch_seconds: 1_700_000_000,
        transforms,
    }
}

fn copy(from: &str, to: &str) -> UpcastTransform {
    UpcastTransform::CopyAs {
        from: from.into(),
        to: to.into(),
    }
}

#[test]
fn total_typed_plan_validates_against_the_named_head() {
    let valid = plan(vec![
        copy("score", "score_copy"),
        UpcastTransform::ConvertAs {
            from: "score".into(),
            to: "score_text".into(),
            conversion: ValueConversion::IntegerToString,
        },
        UpcastTransform::ConvertAs {
            from: "flag".into(),
            to: "flag_rank".into(),
            conversion: ValueConversion::BooleanToInteger,
        },
        UpcastTransform::DefaultTo {
            to: "grade".into(),
            value: DefaultValue::String("F".into()),
        },
    ]);
    assert_eq!(valid.validate(&registry()), Ok(()));
}

#[test]
fn revisions_must_ascend() {
    let mut same = plan(vec![copy("score", "score_copy")]);
    same.from_revision = 2;
    assert_eq!(
        same.validate(&registry()),
        Err(PlanError::RevisionsNotAscending)
    );
}

#[test]
fn unknown_entity_type_is_refused() {
    let mut ghost = plan(vec![copy("score", "score_copy")]);
    ghost.entity_type = "ety_ghost".into();
    assert_eq!(
        ghost.validate(&registry()),
        Err(PlanError::UnknownEntityType)
    );
}

#[test]
fn registry_head_must_equal_the_to_revision() {
    let mut engine = registry();
    let mut rev3 = rev2_properties();
    rev3.push(untyped("extra", false));
    engine.evolve_entity_type(definition(3, rev3)).unwrap();
    assert_eq!(
        plan(vec![copy("score", "score_copy")]).validate(&engine),
        Err(PlanError::RegistryHeadMismatch { head: 3 })
    );
}

#[test]
fn skipped_from_revision_is_unretained() {
    let mut engine = OntologyEngine::default();
    engine
        .register_entity_type(definition(1, rev1_properties()))
        .unwrap();
    engine
        .evolve_entity_type(definition(3, rev2_properties()))
        .unwrap();
    let mut skipping = plan(vec![copy("score", "score_copy")]);
    skipping.from_revision = 2;
    skipping.to_revision = 3;
    assert_eq!(
        skipping.validate(&engine),
        Err(PlanError::UnretainedRevision { revision: 2 })
    );
}

#[test]
fn source_must_exist_at_the_from_revision_not_merely_at_head() {
    // `score_text` exists at head revision 2 but not at revision 1.
    assert_eq!(
        plan(vec![copy("score_text", "score_copy")]).validate(&registry()),
        Err(PlanError::SourceAbsent {
            name: "score_text".into()
        })
    );
}

#[test]
fn absent_target_is_refused() {
    assert_eq!(
        plan(vec![copy("score", "ghost")]).validate(&registry()),
        Err(PlanError::TargetAbsent {
            name: "ghost".into()
        })
    );
}

#[test]
fn required_target_is_refused() {
    assert_eq!(
        plan(vec![UpcastTransform::DefaultTo {
            to: "note".into(),
            value: DefaultValue::String("n/a".into()),
        }])
        .validate(&registry()),
        Err(PlanError::TargetRequired {
            name: "note".into()
        })
    );
}

#[test]
fn primary_key_is_untouchable_from_either_side() {
    assert_eq!(
        plan(vec![UpcastTransform::DefaultTo {
            to: "serial".into(),
            value: DefaultValue::String("sn-1".into()),
        }])
        .validate(&registry()),
        Err(PlanError::PrimaryKeyTouched {
            name: "serial".into()
        })
    );
    assert_eq!(
        plan(vec![copy("serial", "grade")]).validate(&registry()),
        Err(PlanError::PrimaryKeyTouched {
            name: "serial".into()
        })
    );
}

#[test]
fn duplicate_targets_are_refused() {
    assert_eq!(
        plan(vec![
            UpcastTransform::DefaultTo {
                to: "grade".into(),
                value: DefaultValue::String("F".into()),
            },
            UpcastTransform::DefaultTo {
                to: "grade".into(),
                value: DefaultValue::String("E".into()),
            },
        ])
        .validate(&registry()),
        Err(PlanError::DuplicateTarget {
            name: "grade".into()
        })
    );
}

#[test]
fn type_incompatibility_is_refused_with_no_parses() {
    // ConvertAs source kind must match the conversion input.
    assert_eq!(
        plan(vec![UpcastTransform::ConvertAs {
            from: "flag".into(),
            to: "score_text".into(),
            conversion: ValueConversion::IntegerToString,
        }])
        .validate(&registry()),
        Err(PlanError::TypeIncompatible {
            target: "score_text".into()
        })
    );
    // A default must satisfy the target's declared scalar.
    assert_eq!(
        plan(vec![UpcastTransform::DefaultTo {
            to: "score_copy".into(),
            value: DefaultValue::String("many".into()),
        }])
        .validate(&registry()),
        Err(PlanError::TypeIncompatible {
            target: "score_copy".into()
        })
    );
    // CopyAs demands identical declarations; typed -> untyped is a retype.
    assert_eq!(
        plan(vec![copy("score", "grade")]).validate(&registry()),
        Err(PlanError::TypeIncompatible {
            target: "grade".into()
        })
    );
}

#[test]
fn digest_is_deterministic_and_fixed_width_over_unbounded_inputs() {
    let a = plan(vec![copy("score", "score_copy")]);
    assert_eq!(a.digest16(), a.digest16());
    assert_eq!(a.digest16().len(), 16);
    assert!(a.digest16().chars().all(|c| c.is_ascii_hexdigit()));

    let mut b = plan(vec![copy("score", "score_copy")]);
    b.audit_event_type = "reading.upcast_to_2_v2".into();
    assert_ne!(a.digest16(), b.digest16());

    let mut long = plan(vec![copy("score", "score_copy")]);
    long.audit_event_type = "x".repeat(4096);
    assert_eq!(long.digest16().len(), 16);
}
