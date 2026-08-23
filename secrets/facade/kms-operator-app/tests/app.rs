use secrets_kms_operator_app::{
    OperatorApp, OperatorConfig, OperatorStartupConfigError, OperatorStateStoreConfig,
    default_operator_backoff,
};
use secrets_kms_operator_k8s::{AdapterError, InMemoryObservedStateProvider, KmsOperatorActuator};
use secrets_kms_operator_kernel::{
    Action, Clock, DataClassLabel, DesiredState, HsmValidation, KeyOrigin, KeyRing, KeyUsage,
    KeyVersionRotationPolicy, ObservedState, ResidencyMode, SealingRoot,
};

#[derive(Clone, Copy)]
struct FixedClock {
    now: u64,
}

impl Clock for FixedClock {
    fn now_epoch_seconds(&self) -> u64 {
        self.now
    }
}

#[derive(Default)]
struct RecordingActuator {
    actions: Vec<Action>,
}

impl KmsOperatorActuator for RecordingActuator {
    fn execute(&mut self, action: &Action) -> Result<(), AdapterError> {
        self.actions.push(action.clone());
        Ok(())
    }
}

#[test]
fn default_config_builds_strict_home_region_desired_state() {
    let config = test_operator_config();

    assert_eq!(config.desired.sealing_roots.len(), 1);
    assert_eq!(config.desired.key_rings.len(), 1);
    assert_eq!(config.desired.key_rings[0].tenant_id, "ten_cloud_kms");
    assert_eq!(config.backoff.base_seconds, 5);
}

#[test]
fn run_once_wires_provider_desired_clock_and_actuator() {
    let config = test_operator_config();
    let provider = InMemoryObservedStateProvider::complete(ObservedState::default());
    let actuator = RecordingActuator::default();
    let mut app = OperatorApp::new(provider, actuator, FixedClock { now: 1_000 }, config);

    let report = app.run_once().expect("run once should reconcile");

    assert_eq!(report.planned_actions, 2);
    assert_eq!(report.executed_actions, 2);
    assert_eq!(app.actuator().actions.len(), 2);
    assert_eq!(report.wide_event.status, "succeeded");
}

#[test]
fn run_once_propagates_fail_closed_adapter_event() {
    let config = test_operator_config();
    let provider = InMemoryObservedStateProvider::partial("watch stream relist gap");
    let actuator = RecordingActuator::default();
    let mut app = OperatorApp::new(provider, actuator, FixedClock { now: 1_000 }, config);

    let failure = app
        .run_once()
        .expect_err("partial observed state should fail closed");

    assert_eq!(
        failure.error,
        AdapterError::PartialObservedState("watch stream relist gap".to_owned())
    );
    assert_eq!(failure.wide_event.status, "failed");
    assert_eq!(app.actuator().actions, Vec::<Action>::new());
}

#[test]
fn state_store_config_requires_explicit_durable_path() {
    let missing = OperatorStateStoreConfig::from_env_pairs(std::iter::empty::<(&str, &str)>())
        .expect_err("production startup must not silently use in-memory KMS state");
    assert_eq!(missing, OperatorStartupConfigError::MissingStatePath);

    let configured = OperatorStateStoreConfig::from_env_pairs([(
        "OYATIE_KMS_OPERATOR_STATE_PATH",
        "/var/lib/secrets-kms-operator/state.json",
    )])
    .expect("explicit state path should be accepted");
    assert_eq!(
        configured.path,
        "/var/lib/secrets-kms-operator/state.json"
    );
}

fn test_operator_config() -> OperatorConfig {
    OperatorConfig {
        desired: DesiredState {
            sealing_roots: vec![SealingRoot {
                name: "cloud-kms-root".to_owned(),
                tenant_id: "ten_cloud_kms".to_owned(),
                region: "us-east-1".to_owned(),
                cell_id: "cell-us-east-1a".to_owned(),
                root_ref: "sealing-root/cloud-kms".to_owned(),
                active_version: 1,
                rotate_after_seconds: 86_400,
            }],
            key_rings: vec![KeyRing {
                name: "cloud-kms-default".to_owned(),
                tenant_id: "ten_cloud_kms".to_owned(),
                region: "us-east-1".to_owned(),
                cell_id: "cell-us-east-1a".to_owned(),
                hsm_partition_ref: "hsm/us-east-1/cell-us-east-1a".to_owned(),
                origin: KeyOrigin::OyatieManaged,
                usage: KeyUsage::EncryptDecrypt,
                hsm_validation: HsmValidation::Fips1403Level3,
                residency: ResidencyMode::StrictHomeRegion,
                data_class: DataClassLabel::InternalOnly,
                rotation_policy: KeyVersionRotationPolicy {
                    rotate_after_seconds: 2_592_000,
                    decrypt_only_grace_seconds: 86_400,
                },
            }],
        },
        backoff: default_operator_backoff(),
    }
}
