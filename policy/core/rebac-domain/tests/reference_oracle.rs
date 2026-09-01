//! A deliberately small, independent finite evaluator compared with the ReBAC engine.
#![allow(clippy::expect_used, clippy::panic, clippy::unwrap_used)]

mod reference_oracle_support;

#[path = "reference_oracle_tests/bounds.rs"]
mod bounds;
#[path = "reference_oracle_tests/cycles.rs"]
mod cycles;
#[path = "reference_oracle_tests/exhaustive.rs"]
mod exhaustive;
#[path = "reference_oracle_tests/model_contract.rs"]
mod model_contract;
#[path = "reference_oracle_tests/ordering.rs"]
mod ordering;
#[path = "reference_oracle_tests/scope.rs"]
mod scope;
