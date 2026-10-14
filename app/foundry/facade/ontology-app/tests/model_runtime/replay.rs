use crate::support::*;
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    decode_action_record, encode_action_record,
};
use foundry_ontology_app::{BootError, compose_with_models};
use foundry_projection_draft::ProjectionStore;
use foundry_projection_sqlite_draft::SqliteProjectionStore;
use foundry_records_draft::{ActionEnvelope, RecordsLog};
use foundry_records_sqlite_draft::SqliteRecordsLog;
use foundry_spine::{ProjectionState, catch_up, project_through};

#[test]
fn model_restore_compares_every_stored_prefix_entry_without_resetting_data() {
    let fixture = Fixture::new();
    fixture.initialize();
    let registry = foundry_ontology_app::seed::registry_for(TENANT).unwrap();
    let mut action_log = SqliteRecordsLog::open(&fixture.config.action_log).unwrap();
    for ordinal in 1..=3 {
        let key = format!("record-{ordinal}");
        let property = WireProperty::new(
            "name",
            WireTier::Scalar,
            WireDataClass::InternalOnly,
            WireValue::String(format!("Name {ordinal}")),
        )
        .unwrap();
        let record = ActionRecord::new(
            "prn_recorded",
            "decision-recorded",
            "record.written",
            &key,
            1000,
            vec![],
            EditSet::new(vec![
                OntologyEdit::create_object("ety_record", vec![property]).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap();
        action_log
            .append(
                ActionEnvelope::new(
                    TENANT,
                    "ent_record",
                    "aty_record_write",
                    key,
                    1,
                    encode_action_record(&record),
                    1000,
                )
                .unwrap(),
            )
            .unwrap();
    }
    let entries = action_log.replay(TENANT, 1).unwrap();
    let mut other = entries.clone();
    let mut record = decode_action_record(&other[0].envelope.payload).unwrap();
    record.principal_id = "prn_other_prefix".into();
    other[0].envelope.payload = encode_action_record(&record);
    let mut store = SqliteProjectionStore::open(&fixture.config.projection_store).unwrap();
    let mut state = ProjectionState::new(TENANT, &registry);
    project_through(&mut state, &mut store, &other).unwrap();
    let before = store.get(TENANT, "ent_record").unwrap().unwrap();
    assert_eq!(before.last_actor, "prn_recorded");
    assert_eq!(before.last_ordinal, 3);
    assert!(store.poisoned(TENANT).unwrap().is_empty());
    assert!(
        catch_up(TENANT, &registry, &mut store, &entries).is_ok(),
        "same final entry and prefix poisons hide the earlier changed actor from resumed catch-up"
    );
    drop(store);
    drop(action_log);
    let error = compose_with_models(&fixture.config, Some(&fixture.models)).unwrap_err();
    let BootError::ModelRestoreRefused { detail, .. } = error else {
        panic!("expected model replay refusal: {error:?}");
    };
    assert!(
        detail.contains("DivergentReplay") && detail.contains("ordinal: 1"),
        "{detail}"
    );
    let reopened = SqliteProjectionStore::open(&fixture.config.projection_store).unwrap();
    assert_eq!(reopened.applied_head(TENANT).unwrap(), 3);
    assert_eq!(reopened.get(TENANT, "ent_record").unwrap(), Some(before));
    assert!(reopened.poisoned(TENANT).unwrap().is_empty());
}
