#[test]
fn vm_create_api_rejects_unknown_data_class_label_before_catalog_mutation() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::default();
    let mut request = request("req-compute-vm-class", "idem-compute-vm-class");
    request.body.data_class = "SECRET".to_string();

    let error = create_vm_with_trusted_verifier(&mut catalog, &mut ledger, request)
        .expect_err("operational markers are not VM API data classes");

    assert_eq!(
        error,
        CloudComputeVmApiError::InvalidDataClassLabel {
            data_class: "SECRET".to_string(),
        }
    );
    assert_eq!(error.vm_create_status_code(), 400);
    assert_eq!(catalog.instances().count(), 0);
}

#[test]
fn vm_create_idempotency_ledger_enforces_bounded_retention() {
    let mut catalog = CloudComputeCatalog::default();
    let mut ledger = CloudComputeVmCreateIdempotencyLedger::with_max_entries(1);

    create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-vm-bound-1", "idem-vm-bound-1"),
    )
    .expect("first create succeeds");
    let mut second = request("req-vm-bound-2", "idem-vm-bound-2");
    second.path_instance_id = "oyatie:cloud:region-home:ten_alpha:instance:app-2".to_string();
    second.body.resource_id = second.path_instance_id.clone();
    create_vm_with_trusted_verifier(&mut catalog, &mut ledger, second)
        .expect("second create succeeds");

    assert_eq!(ledger.len(), 1);

    let replay_error = create_vm_with_trusted_verifier(
        &mut catalog,
        &mut ledger,
        request("req-vm-bound-replay", "idem-vm-bound-1"),
    )
    .expect_err(
        "evicted idempotency key is no longer replayable and reaches duplicate resource guard",
    );

    assert_eq!(
        replay_error,
        CloudComputeVmApiError::Compute(CloudComputeError::DuplicateInstance)
    );
}
