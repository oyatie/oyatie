//! Integration scaffold: differential pairs are stubbed; no binary spawn.

use os_oci_executor_oracle::{
    compare_observations, differential_pair, refuse_oracle_as_product, validate_obligations,
    DiffVerdict, KillSignal, OciExecutor, OciOperation, OracleStub, OwnedExecutorStub,
};

#[test]
fn scaffold_validates_and_pairs_without_spawning() {
    validate_obligations().expect("obligations");
    let (owned, oracle) = differential_pair(OracleStub::runc());
    let owned_obs = owned.start_stub("b1");
    let oracle_obs = oracle.start_stub("b1");
    assert_eq!(owned_obs.operation(), OciOperation::Start);
    assert_eq!(oracle_obs.operation(), OciOperation::Start);
    assert_eq!(
        compare_observations(&owned_obs, &oracle_obs),
        DiffVerdict::Stubbed
    );
    let kill = oracle.kill_stub("b1", KillSignal::Kill);
    assert_eq!(kill.operation(), OciOperation::Kill(KillSignal::Kill));
    assert_eq!(kill.kill_signal(), Some(KillSignal::Kill));
    refuse_oracle_as_product(OwnedExecutorStub.kind()).unwrap();
    assert!(refuse_oracle_as_product(oracle.kind()).is_err());
}
