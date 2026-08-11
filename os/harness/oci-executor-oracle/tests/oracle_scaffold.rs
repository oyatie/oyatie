//! Integration scaffold: differential pairs are stubbed; no binary spawn.

use os_oci_executor_oracle::{
    differential_pair, refuse_oracle_as_product, validate_obligations, DiffVerdict, OciExecutor,
    OracleStub, OwnedExecutorStub,
};

#[test]
fn scaffold_validates_and_pairs_without_spawning() {
    validate_obligations().expect("obligations");
    let (owned, oracle) = differential_pair(OracleStub::runc());
    assert_eq!(owned.start_stub("b1"), DiffVerdict::Stubbed);
    assert_eq!(oracle.start_stub("b1"), DiffVerdict::Stubbed);
    refuse_oracle_as_product(OwnedExecutorStub.kind()).unwrap();
    assert!(refuse_oracle_as_product(oracle.kind()).is_err());
}
