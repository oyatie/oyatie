use super::{super::StagedPublication, faults::Fault, support::*};
use foundry_edits::{
    ActionRecord, EditSet, OntologyEdit, WireDataClass, WireProperty, WireTier, WireValue,
    decode_action_record, encode_action_record,
};
use foundry_records_draft::{ActionEnvelope, Receipt, RecordsLog, RecordsLogError, SealedEnvelope};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy)]
enum LogFault {
    Read,
    Head,
    MissingFirst,
    Foreign,
    Gap,
    PrefixActor,
}
struct FaultLog {
    inner: Box<dyn RecordsLog + Send>,
    fault: LogFault,
    writes: Arc<AtomicUsize>,
}
impl RecordsLog for FaultLog {
    fn store_access(&self) -> Option<&foundry_store_access::StoreAccess> {
        self.inner.store_access()
    }
    fn append(&mut self, envelope: ActionEnvelope) -> Result<Receipt, RecordsLogError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        self.inner.append(envelope)
    }
    fn head(&self, tenant: &str) -> Result<u64, RecordsLogError> {
        if matches!(self.fault, LogFault::Head) {
            return Err(RecordsLogError::Storage {
                detail: "injected head failure".into(),
            });
        }
        self.inner.head(tenant)
    }
    fn replay(&self, tenant: &str, from: u64) -> Result<Vec<SealedEnvelope>, RecordsLogError> {
        if matches!(self.fault, LogFault::Read) {
            return Err(RecordsLogError::Storage {
                detail: "injected replay failure".into(),
            });
        }
        let mut entries = self.inner.replay(tenant, from)?;
        match self.fault {
            LogFault::MissingFirst => {
                entries.remove(0);
            }
            LogFault::Foreign => {
                entries[0].envelope.tenant_id = "ten_foreign".into();
            }
            LogFault::Gap => {
                entries[1].receipt.ordinal += 1;
            }
            LogFault::PrefixActor => {
                let mut record = decode_action_record(&entries[0].envelope.payload).unwrap();
                record.principal_id = "prn_changed_prefix".into();
                entries[0].envelope.payload = encode_action_record(&record);
            }
            _ => {}
        }
        Ok(entries)
    }
}
fn seed(files: &Files) {
    let mut log =
        foundry_records_sqlite_draft::SqliteRecordsLog::open(&files.config.action_log).unwrap();
    for ordinal in 1..=3 {
        let key = format!("prefix-{ordinal}");
        let property = WireProperty::new(
            "name",
            WireTier::Scalar,
            WireDataClass::InternalOnly,
            WireValue::String(format!("Name {ordinal}")),
        )
        .unwrap();
        let record = ActionRecord::new(
            "prn_original",
            "decision-original",
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
        log.append(
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
}

#[test]
fn facade_preflight_refuses_log_failures_and_hidden_prefix_drift_without_any_metadata_or_runtime_write()
 {
    for fault in [
        LogFault::Read,
        LogFault::Head,
        LogFault::MissingFirst,
        LogFault::Foreign,
        LogFault::Gap,
        LogFault::PrefixActor,
    ] {
        let files = Files::new();
        seed(&files);
        let (mut app, _) = files.compose(Fault::None);
        let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
        let saved = draft(tenant);
        let before = tenant.projection.clone();
        let metadata = tenant
            .model_store
            .as_ref()
            .unwrap()
            .store
            .history(&caller())
            .unwrap();
        let object = tenant.projection_store.get(TENANT, "ent_record").unwrap();
        let placeholder = Box::new(
            foundry_records_sqlite_draft::SqliteRecordsLog::open(
                &files.directory.join("placeholder.sqlite"),
            )
            .unwrap(),
        );
        let inner = std::mem::replace(&mut tenant.action_log, placeholder);
        let writes = Arc::new(AtomicUsize::new(0));
        tenant.action_log = Box::new(FaultLog {
            inner,
            fault,
            writes: writes.clone(),
        });
        assert_eq!(tenant.observed_model_generation(), Some(1));
        assert!(StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).is_err());
        assert_eq!(writes.load(Ordering::SeqCst), 0);
        assert_eq!(tenant.projection, before);
        assert_eq!(
            tenant.projection_store.get(TENANT, "ent_record").unwrap(),
            object
        );
        assert_eq!(tenant.projection_store.applied_head(TENANT).unwrap(), 3);
        assert_eq!(
            tenant
                .model_store
                .as_ref()
                .unwrap()
                .store
                .history(&caller())
                .unwrap(),
            metadata
        );
        assert!(tenant.publication_unknown.is_none());
    }
}

#[test]
fn facade_preflight_refuses_behind_memory_behind_projection_and_ahead_projection_without_repair() {
    for position in ["memory-behind", "projection-behind", "projection-ahead"] {
        let files = Files::new();
        seed(&files);
        let (mut app, _) = files.compose(Fault::None);
        let tenant = app.tenants.get_mut(TENANT).unwrap().get_mut();
        let saved = draft(tenant);
        match position {
            "memory-behind" => {
                tenant.projection.applied_ordinal = 2;
            }
            "projection-behind" => {
                tenant.projection_store.reset_tenant(TENANT).unwrap();
            }
            _ => {
                tenant
                    .projection_store
                    .apply(
                        foundry_projection_draft::AppliedEntry {
                            tenant_id: TENANT.into(),
                            ordinal: 4,
                            outcome: foundry_projection_draft::EntryOutcome::Poisoned {
                                reason: "injected ahead".into(),
                            },
                        },
                        &Default::default(),
                    )
                    .unwrap();
            }
        }
        let before = tenant.projection.clone();
        let head = tenant.projection_store.applied_head(TENANT).unwrap();
        let metadata = tenant
            .model_store
            .as_ref()
            .unwrap()
            .store
            .history(&caller())
            .unwrap();
        assert!(StagedPublication::prepare(tenant, &caller(), saved.id, &input(&saved)).is_err());
        assert_eq!(tenant.projection, before);
        assert_eq!(tenant.projection_store.applied_head(TENANT).unwrap(), head);
        assert_eq!(tenant.action_log.head(TENANT).unwrap(), 3);
        assert_eq!(
            tenant
                .model_store
                .as_ref()
                .unwrap()
                .store
                .history(&caller())
                .unwrap(),
            metadata
        );
    }
}
