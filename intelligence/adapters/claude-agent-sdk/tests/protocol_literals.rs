use intelligence_claude_agent_sdk::{
    AccountInfo, ApiKeySource, ClaudeAgentOptions, ConfigScope, ExitReason, HookEvent,
    JsonSchemaOutputFormat, OutputFormat, OutputFormatType, PermissionBehavior,
    PermissionDecisionClassification, PermissionResult, PermissionUpdate,
    PermissionUpdateDestination, SessionEndHookInput, ThinkingAdaptive, ThinkingConfig,
    ThinkingDisabled, ThinkingDisplay, ThinkingEnabled,
};
use serde_json::json;

#[test]
fn exported_permission_update_literals_are_typed_and_drift_preserving() {
    let update = PermissionUpdate {
        update_type: "addRules".into(),
        rules: None,
        behavior: Some(PermissionBehavior::Allow),
        mode: None,
        directories: None,
        destination: Some(PermissionUpdateDestination::UserSettings),
    };
    assert_eq!(
        serde_json::to_value(&update).unwrap(),
        json!({
            "type": "addRules",
            "behavior": "allow",
            "destination": "userSettings"
        })
    );

    let parsed: PermissionUpdate = serde_json::from_value(json!({
        "type": "setMode",
        "behavior": "ask",
        "destination": "cliArg"
    }))
    .unwrap();
    assert_eq!(parsed.behavior, Some(PermissionBehavior::Ask));
    assert_eq!(
        parsed.destination,
        Some(PermissionUpdateDestination::CliArg)
    );

    let drift: PermissionUpdate = serde_json::from_value(json!({
        "type": "future",
        "behavior": "allowOnce",
        "destination": "workspacePolicy"
    }))
    .unwrap();
    assert_eq!(
        drift.behavior,
        Some(PermissionBehavior::Other("allowOnce".into()))
    );
    assert_eq!(
        drift.destination,
        Some(PermissionUpdateDestination::Other("workspacePolicy".into()))
    );
}

#[test]
fn exported_misc_literal_unions_are_typed_and_drift_preserving() {
    let account: AccountInfo = serde_json::from_value(json!({
        "apiKeySource": "oauth"
    }))
    .unwrap();
    assert_eq!(account.api_key_source, Some(ApiKeySource::Oauth));

    let future_account: AccountInfo = serde_json::from_value(json!({
        "apiKeySource": "enterpriseVault"
    }))
    .unwrap();
    assert_eq!(
        future_account.api_key_source,
        Some(ApiKeySource::Other("enterpriseVault".into()))
    );

    let session_end: SessionEndHookInput = serde_json::from_value(json!({
        "hook_event_name": "SessionEnd",
        "session_id": "550e8400-e29b-41d4-a716-446655440010",
        "transcript_path": "/tmp/session.jsonl",
        "cwd": "/tmp/project",
        "reason": "bypass_permissions_disabled"
    }))
    .unwrap();
    assert_eq!(session_end.reason, ExitReason::BypassPermissionsDisabled);

    let scope: ConfigScope = serde_json::from_value(json!("project")).unwrap();
    assert_eq!(scope, ConfigScope::Project);
}

#[test]
fn permission_decision_classification_is_typed_and_drift_preserving() {
    let allow = PermissionResult::allow()
        .with_decision_classification(PermissionDecisionClassification::UserPermanent);
    assert_eq!(
        serde_json::to_value(&allow).unwrap(),
        json!({
            "behavior": "allow",
            "decisionClassification": "user_permanent"
        })
    );

    let deny: PermissionResult = serde_json::from_value(json!({
        "behavior": "deny",
        "message": "blocked",
        "decisionClassification": "future_classifier"
    }))
    .unwrap();
    match deny {
        PermissionResult::Deny {
            decision_classification,
            ..
        } => assert_eq!(
            decision_classification,
            Some(PermissionDecisionClassification::Other(
                "future_classifier".into()
            ))
        ),
        _ => panic!("expected deny permission result"),
    }
}

#[test]
fn hook_event_literals_are_typed_and_drift_preserving() {
    let event: HookEvent = serde_json::from_value(json!("PostToolUseFailure")).unwrap();
    assert_eq!(event, HookEvent::PostToolUseFailure);
    assert_eq!(
        serde_json::to_value(HookEvent::MessageDisplay).unwrap(),
        json!("MessageDisplay")
    );

    let future: HookEvent = serde_json::from_value(json!("FutureHookEvent")).unwrap();
    assert_eq!(future, HookEvent::Other("FutureHookEvent".into()));
    assert_eq!(
        serde_json::to_value(future).unwrap(),
        json!("FutureHookEvent")
    );
}

#[test]
fn output_format_helpers_emit_current_json_schema_shape() {
    let format = OutputFormat::json_schema(json!({
        "type": "object",
        "properties": {
            "answer": { "type": "string" }
        },
        "required": ["answer"]
    }));
    assert_eq!(
        serde_json::to_value(&format).unwrap(),
        json!({
            "type": "json_schema",
            "schema": {
                "type": "object",
                "properties": {
                    "answer": { "type": "string" }
                },
                "required": ["answer"]
            }
        })
    );

    let parsed: JsonSchemaOutputFormat = serde_json::from_value(json!({
        "type": "json_schema",
        "schema": { "type": "object" }
    }))
    .unwrap();
    assert_eq!(parsed.output_type, OutputFormatType::JsonSchema);

    let options = ClaudeAgentOptions::builder().output_format(format).build();
    let args = options.to_cli_args().unwrap();
    let schema_arg = args
        .windows(2)
        .find(|pair| pair[0] == "--json-schema")
        .map(|pair| pair[1].clone())
        .expect("json schema flag should be emitted");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&schema_arg).unwrap(),
        json!({
            "type": "object",
            "properties": {
                "answer": { "type": "string" }
            },
            "required": ["answer"]
        })
    );
}

#[test]
fn thinking_config_helpers_emit_current_display_shape() {
    let adaptive: ThinkingConfig = ThinkingAdaptive {
        display: Some(ThinkingDisplay::Summarized),
    }
    .into();
    assert_eq!(
        serde_json::to_value(&adaptive).unwrap(),
        json!({
            "type": "adaptive",
            "display": "summarized"
        })
    );

    let enabled: ThinkingConfig = ThinkingEnabled {
        budget_tokens: Some(2048),
        display: Some(ThinkingDisplay::Omitted),
    }
    .into();
    assert_eq!(
        serde_json::to_value(&enabled).unwrap(),
        json!({
            "type": "enabled",
            "budgetTokens": 2048,
            "display": "omitted"
        })
    );

    let parsed: ThinkingConfig = serde_json::from_value(json!({
        "type": "enabled",
        "budgetTokens": 128,
        "display": "future_display"
    }))
    .unwrap();
    match parsed {
        ThinkingConfig::Enabled { display, .. } => {
            assert_eq!(
                display,
                Some(ThinkingDisplay::Other("future_display".into()))
            );
        }
        _ => panic!("expected enabled thinking config"),
    }

    let disabled: ThinkingConfig = ThinkingDisabled.into();
    assert_eq!(
        serde_json::to_value(disabled).unwrap(),
        json!({ "type": "disabled" })
    );
}
