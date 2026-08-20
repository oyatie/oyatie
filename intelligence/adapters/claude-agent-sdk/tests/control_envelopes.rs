use intelligence_claude_agent_sdk::{
    SDKControlEnvelopeType, SDKControlErrorResponse, SDKControlRequest, SDKControlResponse,
    SDKControlResponsePayload, SDKControlSuccessResponse,
};
use serde_json::{json, to_value};

#[test]
fn control_request_envelope_serializes_current_wire_shape()
-> intelligence_claude_agent_sdk::Result<()> {
    let request = SDKControlRequest::new("req-1", json!({"subtype": "mcp_status"}));

    assert_eq!(
        to_value(&request)?,
        json!({
            "type": "control_request",
            "request_id": "req-1",
            "request": {"subtype": "mcp_status"}
        })
    );

    let parsed: SDKControlRequest = serde_json::from_value(json!({
        "type": "future_control_request",
        "request_id": "req-2",
        "request": {"subtype": "future_subtype"},
        "future": true
    }))?;
    assert_eq!(
        parsed.envelope_type,
        SDKControlEnvelopeType::Other("future_control_request".into())
    );
    assert_eq!(parsed.request["subtype"], json!("future_subtype"));
    assert_eq!(parsed.extra["future"], json!(true));
    Ok(())
}

#[test]
fn control_response_envelope_parses_success_error_and_unknown_payloads()
-> intelligence_claude_agent_sdk::Result<()> {
    let success = SDKControlResponse::success("req-1", json!({"ok": true}));
    assert_eq!(
        to_value(&success)?,
        json!({
            "type": "control_response",
            "response": {
                "subtype": "success",
                "request_id": "req-1",
                "response": {"ok": true}
            }
        })
    );

    let error = SDKControlResponse::error("req-2", "denied");
    assert!(matches!(
        &error.response,
        SDKControlResponsePayload::Error(SDKControlErrorResponse { error, .. }) if error == "denied"
    ));

    let parsed: SDKControlResponse = serde_json::from_value(json!({
        "type": "control_response",
        "response": {
            "subtype": "success",
            "request_id": "req-3",
            "response": {"models": []},
            "future": 1
        },
        "outerFuture": true
    }))?;
    assert_eq!(
        parsed.envelope_type,
        SDKControlEnvelopeType::ControlResponse
    );
    assert_eq!(parsed.extra["outerFuture"], json!(true));
    assert!(matches!(
        &parsed.response,
        SDKControlResponsePayload::Success(SDKControlSuccessResponse { request_id, response, extra, .. })
            if request_id == "req-3" && response["models"] == json!([]) && extra["future"] == json!(1)
    ));

    let future: SDKControlResponse = serde_json::from_value(json!({
        "type": "control_response",
        "response": {"subtype": "paused", "request_id": "req-4", "reason": "later"}
    }))?;
    assert!(matches!(
        &future.response,
        SDKControlResponsePayload::Other(value)
            if value["subtype"] == json!("paused") && value["reason"] == json!("later")
    ));
    Ok(())
}
