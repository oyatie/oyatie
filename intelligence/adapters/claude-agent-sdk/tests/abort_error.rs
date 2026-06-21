use intelligence_claude_agent_sdk::AbortError;

#[test]
fn abort_error_matches_package_exported_error_shape() {
    let empty = AbortError::default();
    assert_eq!(empty.to_string(), "");
    assert_eq!(empty.message(), "");

    let error = AbortError::new("Connection aborted by user");
    assert_eq!(error.to_string(), "Connection aborted by user");
    assert_eq!(error.message(), "Connection aborted by user");
}
