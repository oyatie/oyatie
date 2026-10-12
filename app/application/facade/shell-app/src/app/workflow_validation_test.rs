use super::*;
use crate::render_envelope::WorkflowEdge;

fn sample() -> WorkflowSample {
    WorkflowSample::from_envelope(&server_derived_envelope(OperatorContext::TenantAdmin))
}

fn workflow_input_value<'a>(html: &'a str, property: &str) -> &'a str {
    let marker = format!("data-workflow-prop=\"{property}\"");
    let input = html
        .split("<input")
        .skip(1)
        .find(|input| {
            input
                .split('>')
                .next()
                .unwrap_or_default()
                .contains(&marker)
        })
        .expect("selected property input");
    input
        .split_once("value=\"")
        .expect("selected property has a serialized value attribute")
        .1
        .split_once('"')
        .expect("complete value attribute")
        .0
}

#[test]
fn workflow_sample_selected_properties_match_shared_and_static_html() {
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        let state = WorkflowSample::from_envelope(&envelope);
        let selected = state.selected().expect("sample selection").clone();
        let owner = Owner::new();
        let html = owner.with(|| {
            workflow_studio_panel(envelope.clone(), RwSignal::new(state))
                .into_view()
                .to_html()
        });
        assert_eq!(workflow_input_value(&html, "label-ko"), selected.label);
        assert_eq!(workflow_input_value(&html, "type"), selected.kind);
        if context == OperatorContext::TenantAdmin {
            assert!(crate::static_dashboard_html().contains(&html));
        }
        owner.cleanup();
    }

    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    let mut state = WorkflowSample::from_envelope(&envelope);
    state.add(
        "Local review & \"approval\"",
        "Human & \"review\"",
        "Example",
    );
    let owner = Owner::new();
    let html = owner.with(|| {
        workflow_studio_panel(envelope, RwSignal::new(state))
            .into_view()
            .to_html()
    });
    assert_eq!(
        workflow_input_value(&html, "label-ko"),
        "Local review &amp; &quot;approval&quot;"
    );
    assert_eq!(
        workflow_input_value(&html, "type"),
        "Human &amp; &quot;review&quot;"
    );
    owner.cleanup();
}

#[tokio::test]
async fn workflow_sample_served_selected_properties_match_native_selection() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    tokio::time::timeout(std::time::Duration::from_secs(2), async {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("loopback listener");
        let address = listener.local_addr().expect("listener address");
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(crate::server::serve_until_shutdown(listener, async {
            let _ = stopped.await;
        }));
        let mut stream = tokio::net::TcpStream::connect(address)
            .await
            .expect("native SSR connection");
        stream
            .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
            .await
            .expect("HTTP request");
        let mut bytes = Vec::new();
        stream
            .read_to_end(&mut bytes)
            .await
            .expect("served SSR response");
        let _ = stop.send(());
        server.await.expect("server task").expect("server shutdown");
        let html = String::from_utf8(bytes).expect("SSR UTF-8");
        assert!(html.starts_with("HTTP/1.1 200"));
        let state = sample();
        let selected = state.selected().expect("sample selection");
        assert_eq!(workflow_input_value(&html, "label-ko"), selected.label);
        assert_eq!(workflow_input_value(&html, "type"), selected.kind);
    })
    .await
    .expect("native SSR property proof keeps its two-second deadline");
}

#[test]
fn workflow_sample_named_draft_selection_and_label_are_one_current_graph() {
    let mut state = sample();
    let initial = state.nodes.len();
    state.add("HTTP request", "Action", "External HTTP example");
    assert_eq!(state.nodes.len(), initial + 1);
    assert_eq!(state.selected().unwrap().label, "HTTP request");
    assert_eq!(state.selected().unwrap().kind, "Action");
    state.rename_selected("Review request".into());
    assert_eq!(state.nodes.last().unwrap(), state.selected().unwrap());
    assert_eq!(state.selected().unwrap().label, "Review request");
    assert!(validate_workflow_sample(&state.nodes, &state.edges).is_empty());
    state.validate();
    assert!(state.outcome.contains("Sample structure valid"));
    state.preview_path();
    assert!(state.outcome.contains("Review request"));
    assert!(
        state
            .outcome
            .contains("No run, approval, draft, or receipt created")
    );
}

#[test]
fn workflow_sample_validation_refuses_empty_duplicate_missing_reference_and_cycle() {
    assert!(
        validate_workflow_sample(&[], &[])
            .iter()
            .any(|error| error.contains("no nodes"))
    );
    let mut state = sample();
    state.nodes.push(state.nodes[0].clone());
    state.nodes[1].label.clear();
    state.edges.push(WorkflowEdge {
        from: "missing".into(),
        to: state.nodes[0].id.clone(),
        label: String::new(),
    });
    let errors = validate_workflow_sample(&state.nodes, &state.edges);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("duplicate sample node ID"))
    );
    assert!(errors.iter().any(|error| error.contains("empty label")));
    assert!(errors.iter().any(|error| error.contains("missing node")));
    let mut state = sample();
    state.edges.push(WorkflowEdge {
        from: state.nodes.last().unwrap().id.clone(),
        to: state.nodes[0].id.clone(),
        label: String::new(),
    });
    assert!(
        validate_workflow_sample(&state.nodes, &state.edges)
            .iter()
            .any(|error| error.contains("cycle"))
    );
    state.preview_path();
    assert!(state.outcome.contains("Sample path unavailable"));
}

#[test]
fn workflow_sample_complete_native_source_changes_reset_local_state() {
    let envelope = server_derived_envelope(OperatorContext::TenantAdmin);
    let original = WorkflowSample::from_envelope(&envelope);
    let mut edited = original.clone();
    edited.add("HTTP request", "Action", "External HTTP example");
    edited.rename_selected("Local edit".into());
    edited.query = "request".into();
    edited.zoom = 80;
    edited.suggestion_dismissed = true;
    assert_eq!(workflow_sample_for_envelope(&envelope, &edited), edited);
    for field in [
        "/tenant_name",
        "/role_name",
        "/tenant_class",
        "/accreditation/label",
        "/accreditation/healthcare_enabled",
        "/accreditation/explanation",
        "/workflow/name",
        "/workflow/goal",
        "/workflow/nodes/0/id",
        "/workflow/nodes/0/label",
        "/workflow/nodes/0/kind",
        "/workflow/nodes/0/x",
        "/workflow/nodes/0/y",
        "/workflow/nodes/0/explanation",
        "/workflow/edges/0/from",
        "/workflow/edges/0/to",
        "/workflow/edges/0/label",
    ] {
        let mut changed = serde_json::to_value(&envelope).expect("native envelope JSON");
        let value = changed
            .pointer_mut(field)
            .expect("complete native source field");
        *value = match value {
            serde_json::Value::String(text) => serde_json::Value::String(format!("{text} changed")),
            serde_json::Value::Bool(flag) => serde_json::Value::Bool(!*flag),
            serde_json::Value::Number(number) => serde_json::json!(number.as_i64().unwrap() + 1),
            _ => panic!("unexpected native source field: {field}"),
        };
        let changed: TenantRenderEnvelope =
            serde_json::from_value(changed).expect("changed native envelope");
        assert_ne!(
            WorkflowSample::from_envelope(&changed).source,
            original.source,
            "source field {field}"
        );
        assert_eq!(
            workflow_sample_for_envelope(&changed, &edited),
            WorkflowSample::from_envelope(&changed),
            "reset for source field {field}"
        );
    }
    assert_ne!(
        WorkflowSample::from_envelope(&server_derived_envelope(OperatorContext::CorporateOffice))
            .source,
        original.source
    );
    let other_context = server_derived_envelope(OperatorContext::CorporateOffice);
    assert_eq!(
        workflow_sample_for_envelope(&other_context, &edited),
        WorkflowSample::from_envelope(&other_context)
    );
}
