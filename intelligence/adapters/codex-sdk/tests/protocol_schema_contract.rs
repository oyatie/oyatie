use intelligence_codex_sdk::{
    app_server_protocol_definition, app_server_protocol_definition_names,
    app_server_protocol_schema_json, app_server_protocol_schema_summary,
};

#[test]
fn exposes_complete_upstream_app_server_protocol_schema_artifact() {
    let summary = app_server_protocol_schema_summary().unwrap();

    assert_eq!(summary.title.as_deref(), Some("CodexAppServerProtocolV2"));
    assert_eq!(summary.definition_count, 469);
    assert!(app_server_protocol_schema_json().contains("ThreadStartParams"));

    let names = app_server_protocol_definition_names().unwrap();
    assert!(names.binary_search(&"ClientRequest".to_string()).is_ok());
    assert!(
        names
            .binary_search(&"ServerNotification".to_string())
            .is_ok()
    );
    assert!(
        names
            .binary_search(&"ThreadStartParams".to_string())
            .is_ok()
    );
    assert!(names.binary_search(&"TurnStartParams".to_string()).is_ok());

    let turn_start = app_server_protocol_definition("TurnStartParams").unwrap();
    assert_eq!(turn_start["type"], "object");
    assert!(turn_start["properties"].get("threadId").is_some());
    assert!(turn_start["properties"].get("input").is_some());
}
