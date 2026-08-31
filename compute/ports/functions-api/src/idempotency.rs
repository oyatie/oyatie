fn parse_api_data_class(label: String) -> Result<DataClass, CloudComputeFunctionsApiError> {
    parse_data_class_label(&label).ok_or(
        CloudComputeFunctionsApiError::InvalidPayloadDataClassLabel {
            payload_data_class: label,
        },
    )
}

fn idempotency_key_for(
    boundary: &CloudComputeFunctionsApiBoundaryContext,
    principal: &CloudComputeFunctionsApiPrincipal,
    surface: &str,
) -> CloudComputeFunctionsIdempotencyLedgerKey {
    CloudComputeFunctionsIdempotencyLedgerKey {
        tenant_id: boundary.tenant_id.clone(),
        principal_id: principal.principal_id.clone(),
        surface: surface.to_string(),
        idempotency_key: boundary.idempotency_key.clone(),
    }
}

fn function_invoke_fingerprint_for(
    path_function_id: &str,
    input: &FunctionInvocationRequest,
) -> CloudComputeFunctionsRequestFingerprint {
    CloudComputeFunctionsRequestFingerprint {
        canonical: canonical_fields(&[
            ("path.function_id", path_function_id.to_string()),
            ("body.invocation_id", input.invocation_id.clone()),
            ("body.tenant_id", input.tenant_id.clone()),
            ("body.function_id", input.function_id.clone()),
            ("body.region", input.region.clone()),
            (
                "body.payload_data_class",
                input.payload_data_class.label().to_string(),
            ),
            (
                "body.current_concurrent_invocations",
                input.current_concurrent_invocations.to_string(),
            ),
            (
                "body.requested_at_epoch_seconds",
                input.requested_at_epoch_seconds.to_string(),
            ),
        ]),
    }
}

fn canonical_fields(fields: &[(&str, String)]) -> String {
    fields
        .iter()
        .map(|(name, value)| format!("{}:{}={}:{}", name.len(), name, value.len(), value))
        .collect::<Vec<_>>()
        .join("")
}

fn invocation_receipt(
    receipt: FunctionInvocationReceipt,
) -> CloudComputeFunctionsInvocationReceipt {
    CloudComputeFunctionsInvocationReceipt {
        invocation_id: receipt.invocation_id.value.value,
        tenant_id: receipt.tenant_id.value,
        function_id: receipt.function_id.value.value,
        region: receipt.region.value.value,
        payload_data_class: receipt.payload_data_class.value.label().to_string(),
        cold_start_budget_ms: receipt.cold_start_budget_ms.value,
        accepted_at_epoch_seconds: receipt.accepted_at_epoch_seconds.value,
        schema_version: receipt.schema_version.value,
    }
}
