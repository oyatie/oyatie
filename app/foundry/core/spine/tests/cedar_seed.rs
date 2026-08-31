//! The Cedar seed held to the REAL engine: the schema parses, the
//! policy set passes the STRICT validator, and a permit is present — a
//! policy set that cannot permit anything is a nullified file, not a
//! posture.

use std::str::FromStr;

use cedar_policy::{PolicySet, Schema, ValidationMode, Validator};

const SCHEMA_SRC: &str = include_str!("../../../cedar/foundry.cedarschema");
const POLICIES_SRC: &str = include_str!("../../../cedar/foundry-policies.cedar");

fn schema() -> Schema {
    let (schema, _warnings) =
        Schema::from_cedarschema_str(SCHEMA_SRC).expect("foundry.cedarschema must parse");
    schema
}

fn policy_set() -> PolicySet {
    PolicySet::from_str(POLICIES_SRC).expect("foundry-policies.cedar must parse")
}
/// The seed must pass the STRICT validator — a policy that references a
/// vocabulary the schema does not declare never ships.
#[test]
fn the_seed_strict_validates() {
    let validator = Validator::new(schema());
    let result = validator.validate(&policy_set(), ValidationMode::Strict);
    assert!(
        result.validation_passed(),
        "strict validation failed: {:?}",
        result.validation_errors().collect::<Vec<_>>(),
    );
}

/// No bare forbid: a policy set that cannot permit anything is a
/// nullified file, not a posture.
#[test]
fn the_seed_contains_a_permit() {
    assert!(
        policy_set()
            .policies()
            .any(|policy| { format!("{:?}", policy.effect()).contains("Permit") }),
        "the policy set must carry at least one permit",
    );
}
