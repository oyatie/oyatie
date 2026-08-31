#[test]
fn functions_invoke_api_maps_payload_data_class_policy_without_masking() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut denied = request(
        "req-compute-functions-payload-denied",
        "idem-functions-payload-denied-001",
        "fninv_payload_denied",
    );
    denied.body.payload_data_class = "PHI".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, denied)
        .expect_err("payload class must be allowed by deployment policy");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::PayloadDataClassNotAllowed)
    );
    assert_eq!(error.invoke_status_code(), 403);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_invocation_at_declared_max_concurrency() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-concurrency",
        "idem-functions-concurrency-001",
        "fninv_concurrency",
    );
    request.body.current_concurrent_invocations = 250;

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("function max_concurrency is enforced by the domain");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::QuotaExceeded)
    );
    assert_eq!(error.invoke_status_code(), 403);
    assert_eq!(ledger.len(), 1);
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_api_rejects_unknown_payload_data_class_before_ledger() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::default();
    let mut request = request(
        "req-compute-functions-data-class",
        "idem-functions-data-class-001",
        "fninv_data_class",
    );
    request.body.payload_data_class = "NOT_A_DATA_CLASS".to_string();

    let error = invoke_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("unknown payload data class label is rejected before invocation");

    assert_eq!(
        error,
        CloudComputeFunctionsApiError::InvalidPayloadDataClassLabel {
            payload_data_class: "NOT_A_DATA_CLASS".to_string(),
        }
    );
    assert_eq!(error.invoke_status_code(), 400);
    assert!(ledger.is_empty());
    assert_eq!(catalog.invocations().count(), 0);
}

#[test]
fn functions_invoke_idempotency_ledger_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::default();
    seed_active_function(&mut catalog);
    let mut ledger = CloudComputeFunctionsInvokeIdempotencyLedger::with_max_entries(1);

    invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-1",
            "idem-functions-bound-1",
            "fninv_bound_1",
        ),
    )
    .expect("first invocation succeeds");
    invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-2",
            "idem-functions-bound-2",
            "fninv_bound_2",
        ),
    )
    .expect("second invocation succeeds");

    assert_eq!(ledger.len(), 1);

    let replay_error = invoke_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request(
            "req-functions-bound-replay",
            "idem-functions-bound-1",
            "fninv_bound_1",
        ),
    )
    .expect_err(
        "evicted idempotency key is no longer replayable and reaches duplicate invocation guard",
    );

    assert_eq!(
        replay_error,
        CloudComputeFunctionsApiError::Compute(CloudComputeError::DuplicateInvocation)
    );
}
