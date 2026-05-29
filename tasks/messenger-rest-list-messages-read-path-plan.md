# Plan: messenger-rest-list-messages-read-path

vertical: messenger
crate: oya-messenger-message-stream-rest
branch: feat/task-messenger-rest-list-messages-read-path-2026-05-28
base: origin/dev

## Objective

Promote the GET /channels/{channel_id}/messages route from ContractOnly to a
typed read-path handler. No DB I/O; deterministic in-memory page response.
Mirrors the framework-free contract style of the existing post_message path.

## Subtasks

### [messenger-rest-list-messages-read-path-1] Typed request/response structs + route flip

- Flip the OPENAPI_ROUTES entry for GET /channels/{channel_id}/messages from
  RouteHandlerStatus::ContractOnly to RouteHandlerStatus::Implemented.
- Add constants: LIST_MESSAGES_ROUTE, LIST_MESSAGES_METHOD.
- Add ListMessagesRestRequest { channel_id, page_token, limit } (read-only shape).
- Add MessageSummary { message_id, author_ref, channel_id } (page item).
- Add ListMessagesResponse { channel_id, items: Vec<MessageSummary>, next_page_token: Option<String> }.
- Add list_messages(context: MessengerRestContext, request: ListMessagesRestRequest)
  -> Result<RestResponse<ListMessagesResponse>, MessengerRestError>:
  - Guard: empty channel_id -> MissingPathChannel.
  - Build AuthorizedMessengerContext via messenger_api_context(context).
  - Call api_context.validate() -> map_err(MessengerRestError::Api).
  - Return RestResponse { status_code: 200, body: deterministic empty page }.

Acceptance:
- cargo check -p oya-messenger-message-stream-rest --all-targets green.
- OPENAPI_ROUTES.len() still == 26 (route flipped, not added).
- find_openapi_route("GET", "/channels/{channel_id}/messages").map(|r| r.handler_status)
  == Some(RouteHandlerStatus::Implemented).

### [messenger-rest-list-messages-read-path-2] dispatch_read_route

- Add enum MessengerReadRouteRequest { ListMessages(ListMessagesRestRequest) }.
- Add enum MessengerReadRouteResponse { ListMessages(ListMessagesResponse) }.
- Add enum MessengerReadRouteDispatchError { UnknownRoute, ContractOnly { method, path },
  PayloadMismatch { method, path }, Handler(MessengerRestError) }.
- Add dispatch_read_route(method, path, context, request) mirroring dispatch_write_route:
  - UnknownRoute if find_openapi_route returns None.
  - ContractOnly if handler_status == ContractOnly.
  - Match (method, path, request) arm for LIST_MESSAGES_METHOD + LIST_MESSAGES_ROUTE +
    MessengerReadRouteRequest::ListMessages(req) -> calls list_messages.
  - PayloadMismatch catch-all.

Acceptance:
- cargo check -p oya-messenger-message-stream-rest --all-targets green.
- dispatch_contract_only_route test for typed-handler routes still holds
  (the now-Implemented route is excluded from ContractOnly 501 path).

### [messenger-rest-list-messages-read-path-3] Tests

- list_messages returns 200 with typed page for valid request.
- list_messages rejects empty channel_id with MissingPathChannel.
- list_messages rejects missing idempotency_key with Api(MissingIdempotencyKey).
- list_messages rejects missing policy_decision_ref with Api(MissingPolicyDecision).
- dispatch_read_route routes the implemented GET path and returns 200.
- dispatch_read_route refuses a ContractOnly read route with ContractOnly error.
- dispatch_read_route returns UnknownRoute for an unregistered path.
- Pre-existing openapi_route_catalog_covers_declared_operations and contract-only tests remain green.

Acceptance:
- cargo nextest run -p oya-messenger-message-stream-rest passes including new tests.

## Boundaries

- Touch ONLY: crates/oya-messenger-message-stream-rest/src/lib.rs
- Touch ONLY: docs/specs/task-messenger-rest-list-messages-read-path.md (this spec)
- Touch ONLY: tasks/messenger-rest-list-messages-read-path-plan.md (this plan)
- NEVER: root Cargo.toml, any other crate, any other task's files.
