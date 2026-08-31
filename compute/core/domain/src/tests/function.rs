use data_boundary_kernel::DataClass;

use crate::*;

use super::fixtures::*;

#[test]
fn registers_function_then_invokes_active_function_with_data_class_allowlist() {
    let mut catalog = CloudComputeCatalog::default();
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    let active = catalog
        .activate_function(&function.resource_id.value)
        .expect("function activates");
    assert_eq!(active.state.value, FunctionDeploymentState::Active);

    let receipt = catalog
        .invoke_function(invocation("fninv_001", DataClass::PiiIdentifying))
        .expect("allowed function invocation is recorded");
    assert_eq!(
        receipt.payload_data_class.value.data_class(),
        DataClass::PiiIdentifying
    );
    assert_eq!(receipt.cold_start_budget_ms.value, 750);
    assert_eq!(catalog.invocations().count(), 1);
}

#[test]
fn function_activation_failure_preserves_registered_function() {
    let mut catalog = CloudComputeCatalog::default();
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    catalog
        .activate_function(&function.resource_id.value)
        .expect("first activation succeeds");

    let second_activation = catalog
        .activate_function(&function.resource_id.value)
        .expect_err("active functions cannot be activated twice");

    assert_eq!(second_activation, CloudComputeError::InvalidFunctionState);
    assert_eq!(catalog.functions().count(), 1);
    assert_eq!(
        catalog
            .functions()
            .next()
            .expect("function remains")
            .state
            .value,
        FunctionDeploymentState::Active
    );
}

#[test]
fn rejects_function_budget_payload_class_max_concurrency_duplicate_invocation_and_inactive() {
    let budget_error = FunctionDeployment::new(FunctionDeploymentCreate {
        cold_start_budget_ms: MAX_FUNCTION_COLD_START_BUDGET_MS + 1,
        ..function_create()
    })
    .expect_err("function cold-start budgets are capped");
    assert_eq!(budget_error, CloudComputeError::InvalidFunctionBudget);

    let inactive = FunctionDeployment::new(function_create()).expect("deploying function");
    let inactive_error = inactive
        .invoke(invocation("fninv_inactive", DataClass::Public))
        .expect_err("deploying functions cannot be invoked");
    assert_eq!(inactive_error, CloudComputeError::FunctionNotActive);

    let mut catalog = CloudComputeCatalog::default();
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    catalog
        .activate_function(&function.resource_id.value)
        .expect("function activates");
    let class_error = catalog
        .invoke_function(invocation("fninv_002", DataClass::Phi))
        .expect_err("payload data class must be allowlisted");
    assert_eq!(class_error, CloudComputeError::PayloadDataClassNotAllowed);

    let concurrency_error = catalog
        .invoke_function(FunctionInvocationRequest {
            current_concurrent_invocations: 250,
            ..invocation("fninv_concurrency", DataClass::Public)
        })
        .expect_err("function invocation cannot exceed declared max concurrency");
    assert_eq!(concurrency_error, CloudComputeError::QuotaExceeded);

    catalog
        .invoke_function(invocation("fninv_003", DataClass::Public))
        .expect("first invocation records");
    let duplicate = catalog
        .invoke_function(invocation("fninv_003", DataClass::Public))
        .expect_err("invocation ids are immutable evidence keys");
    assert_eq!(duplicate, CloudComputeError::DuplicateInvocation);
}

#[test]
fn function_invocation_store_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::with_invocation_retention_limit(1);
    let function = catalog
        .register_function(function_create())
        .expect("function registers");
    catalog
        .activate_function(&function.resource_id.value)
        .expect("function activates");

    catalog
        .invoke_function(invocation("fninv_bound_1", DataClass::Public))
        .expect("first invocation records");
    catalog
        .invoke_function(invocation("fninv_bound_2", DataClass::Public))
        .expect("second invocation records");

    assert_eq!(catalog.invocations().count(), 1);
}
