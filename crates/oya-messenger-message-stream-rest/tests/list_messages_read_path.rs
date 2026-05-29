//! TDD red tests — GET /channels/{channel_id}/messages read path.
//!
//! These tests reference symbols that do not yet exist in the crate:
//!   LIST_MESSAGES_METHOD, LIST_MESSAGES_ROUTE, LIST_MESSAGES_OPERATION_ID,
//!   ListMessagesRestRequest, ListMessagesResponse, MessageSummary,
//!   list_messages(), MessengerReadRouteRequest, MessengerReadRouteResponse,
//!   MessengerReadRouteDispatchError, dispatch_read_route().
//!
//! Expected compile result at this commit: ERROR (unresolved items) — RED.
//! After the impl commit lands these tests must pass — GREEN.

use oya_messenger_message_stream_api::MessengerApiError;
use oya_messenger_message_stream_rest::{
    // ---- route constants (subtask-1) ----------------------------------------
    LIST_MESSAGES_METHOD,
    LIST_MESSAGES_OPERATION_ID,
    LIST_MESSAGES_ROUTE,
    ListMessagesResponse,
    // ---- handler structs (subtask-1) ----------------------------------------
    ListMessagesRestRequest,
    MessageSummary,
    MessengerReadRouteDispatchError,
    // ---- dispatch types + fn (subtask-2) ------------------------------------
    MessengerReadRouteRequest,
    MessengerReadRouteResponse,
    // ---- shared context helpers already present -----------------------------
    MessengerRestContext,
    MessengerRestError,
    OPENAPI_ROUTES,
    RestContextKind,
    RouteDispatchError,
    RouteHandlerStatus,
    dispatch_contract_only_route,
    dispatch_read_route,
    find_openapi_route,
    // ---- handler fn (subtask-1) ---------------------------------------------
    list_messages,
};

fn context() -> MessengerRestContext {
    MessengerRestContext {
        scope_org_id: "tenant:t".into(),
        context_kind: RestContextKind::Professional,
        principal_ref: "user:u".into(),
        idempotency_key: "idem".into(),
        policy_decision_ref: "cedar:allow:message-list".into(),
        request_id: "req".into(),
    }
}

fn list_req(channel_id: &str) -> ListMessagesRestRequest {
    ListMessagesRestRequest {
        channel_id: channel_id.into(),
        page_token: None,
        limit: None,
    }
}

// ---- subtask-1: route constant + struct invariants --------------------------

#[test]
fn list_messages_route_constants_match_openapi_spec() {
    assert_eq!(LIST_MESSAGES_METHOD, "GET");
    assert_eq!(LIST_MESSAGES_ROUTE, "/channels/{channel_id}/messages");
    assert_eq!(LIST_MESSAGES_OPERATION_ID, "messenger.list_messages");
}

#[test]
fn openapi_route_catalog_length_unchanged_at_26() {
    // Flipping ContractOnly -> Implemented must not add or remove entries.
    assert_eq!(OPENAPI_ROUTES.len(), 26);
}

#[test]
fn get_messages_route_resolves_to_implemented() {
    let route = find_openapi_route(LIST_MESSAGES_METHOD, LIST_MESSAGES_ROUTE)
        .expect("GET /channels/{channel_id}/messages must be in OPENAPI_ROUTES");
    assert_eq!(
        route.handler_status,
        RouteHandlerStatus::Implemented,
        "route must be flipped to Implemented"
    );
}

// ---- subtask-1: list_messages() handler -------------------------------------

#[test]
fn list_messages_returns_200_with_typed_empty_page() {
    let response = list_messages(context(), list_req("chan-1")).unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body.channel_id, "chan-1");
    assert!(
        response.body.items.is_empty(),
        "read-only seam returns empty items; no DB I/O"
    );
    assert!(
        response.body.next_page_token.is_none(),
        "no next page on empty seam"
    );
}

#[test]
fn list_messages_rejects_empty_channel_id_with_missing_path_channel() {
    let result = list_messages(context(), list_req(""));
    assert_eq!(
        result,
        Err(MessengerRestError::MissingPathChannel),
        "empty channel_id must return MissingPathChannel before any API call"
    );
}

#[test]
fn list_messages_rejects_whitespace_only_channel_id() {
    let result = list_messages(context(), list_req("   "));
    assert_eq!(result, Err(MessengerRestError::MissingPathChannel));
}

#[test]
fn list_messages_rejects_missing_idempotency_key_via_validate() {
    let mut ctx = context();
    ctx.idempotency_key.clear();
    let result = list_messages(ctx, list_req("c"));
    assert_eq!(
        result,
        Err(MessengerRestError::Api(
            MessengerApiError::MissingIdempotencyKey
        )),
        "validate() must fire before returning a page"
    );
}

#[test]
fn list_messages_rejects_missing_policy_decision_via_validate() {
    let mut ctx = context();
    ctx.policy_decision_ref.clear();
    let result = list_messages(ctx, list_req("c"));
    assert_eq!(
        result,
        Err(MessengerRestError::Api(
            MessengerApiError::MissingPolicyDecision
        ))
    );
}

// ---- subtask-1: ListMessagesResponse struct layout --------------------------

#[test]
fn list_messages_response_channel_id_echoes_request() {
    let resp = list_messages(context(), list_req("echo-chan")).unwrap();
    assert_eq!(resp.body.channel_id, "echo-chan");
}

#[test]
fn list_messages_response_has_correct_type_structure() {
    // Validates that ListMessagesResponse and MessageSummary are the correct
    // public types (not just aliases or unit structs).
    let _: ListMessagesResponse = ListMessagesResponse {
        channel_id: "c".into(),
        items: vec![MessageSummary {
            message_id: "m".into(),
            author_ref: "user:u".into(),
            channel_id: "c".into(),
        }],
        next_page_token: Some("tok".into()),
    };
}

// ---- subtask-2: dispatch_read_route() ---------------------------------------

#[test]
fn dispatch_read_route_routes_implemented_get_messages_path() {
    let response = dispatch_read_route(
        LIST_MESSAGES_METHOD,
        LIST_MESSAGES_ROUTE,
        context(),
        MessengerReadRouteRequest::ListMessages(list_req("c")),
    )
    .unwrap();
    assert_eq!(response.status_code, 200);
    let MessengerReadRouteResponse::ListMessages(page) = response.body;
    assert_eq!(page.channel_id, "c");
    assert!(page.items.is_empty());
}

#[test]
fn dispatch_read_route_refuses_contract_only_read_route() {
    let result = dispatch_read_route(
        "GET",
        "/channels",
        context(),
        MessengerReadRouteRequest::ListMessages(list_req("c")),
    );
    assert_eq!(
        result,
        Err(MessengerReadRouteDispatchError::ContractOnly {
            method: "GET",
            path: "/channels",
        })
    );
}

#[test]
fn dispatch_read_route_returns_unknown_route_for_unregistered_path() {
    let result = dispatch_read_route(
        "GET",
        "/does-not-exist",
        context(),
        MessengerReadRouteRequest::ListMessages(list_req("c")),
    );
    assert_eq!(result, Err(MessengerReadRouteDispatchError::UnknownRoute));
}

#[test]
fn dispatch_read_route_propagates_missing_path_channel_as_handler_error() {
    let result = dispatch_read_route(
        LIST_MESSAGES_METHOD,
        LIST_MESSAGES_ROUTE,
        context(),
        MessengerReadRouteRequest::ListMessages(list_req("")),
    );
    assert_eq!(
        result,
        Err(MessengerReadRouteDispatchError::Handler(
            MessengerRestError::MissingPathChannel
        ))
    );
}

// ---- subtask-2: contract_only invariant for now-Implemented route -----------

#[test]
fn dispatch_contract_only_refuses_now_implemented_get_messages_route() {
    // After the route flip, dispatch_contract_only_route must return
    // TypedHandlerRequired for the GET messages route — not 501.
    assert_eq!(
        dispatch_contract_only_route(LIST_MESSAGES_METHOD, LIST_MESSAGES_ROUTE),
        Err(RouteDispatchError::TypedHandlerRequired {
            method: LIST_MESSAGES_METHOD,
            path: LIST_MESSAGES_ROUTE,
        }),
        "Implemented route must be excluded from the ContractOnly 501 path"
    );
}
