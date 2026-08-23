# Spec: messenger-rest-list-messages-read-path

status: spec
vertical: messenger
crate: messenger-message-stream-rest
task_id: messenger-rest-list-messages-read-path
branch: feat/task-messenger-rest-list-messages-read-path-2026-05-28

## Objective

Promote the GET /channels/{channel_id}/messages REST route in
`messenger-message-stream-rest` from `RouteHandlerStatus::ContractOnly` to
`RouteHandlerStatus::Implemented`. Introduce typed request/response structs and
a `list_messages` handler that mirrors the framework-free contract style of the
existing `post_message` path. No DB I/O; the read path returns a deterministic
empty page for the read-only seam.

## Vertical

Messenger vertical. This crate is the HTTP boundary layer; it calls no
downstream I/O and makes no persistence claims. The read-path handler validates
context via `AuthorizedMessengerContext::validate()` (same seam as write path)
and returns a typed page.

## OpenAPI Contract (3.2.0)

Route already declared in `OPENAPI_ROUTES` at index 7 (0-based):

```
GET /channels/{channel_id}/messages
```

Path parameter: `channel_id` (string, required).
Query parameters (optional): `page_token` (string), `limit` (u32).

Response 200:

```json
{
  "channel_id": "string",
  "items": [
    {
      "message_id": "string",
      "author_ref": "string",
      "channel_id": "string"
    }
  ],
  "next_page_token": "string | null"
}
```

Response 400: empty channel_id (MissingPathChannel).
Response 401/403: missing idempotency key or policy decision (Api error).
Response 501: not reached after route flip.

No proto3 surface added in this slice (read path is REST-only; gRPC surface is
out of scope for this task).

## Mod Layout (flat clean-arch)

All code lives in `src/lib.rs` per the flat single-crate pattern (ADR-0509).
No sub-modules are introduced. New items are appended after the existing
`post_message` / `dispatch_write_route` block:

```
constants: LIST_MESSAGES_ROUTE, LIST_MESSAGES_METHOD, LIST_MESSAGES_OPERATION_ID
structs:   ListMessagesRestRequest, MessageSummary, ListMessagesResponse
enums:     MessengerReadRouteRequest, MessengerReadRouteResponse,
           MessengerReadRouteDispatchError
fns:       list_messages, dispatch_read_route
```

Existing items touched:
- `OPENAPI_ROUTES[7].handler_status`: ContractOnly -> Implemented (one field).

## Context Mapping

`list_messages` reuses the private `messenger_api_context(context)` helper
(already used by `post_message_write_plan`) to build `AuthorizedMessengerContext`,
then calls `.validate()`. This is the same scope/idempotency/policy guard as the
write path with no special read-mode relaxation.

## Testing Strategy

Unit tests only, `#[cfg(test)]` block in `src/lib.rs`. No integration fixtures
needed; the handler makes no I/O calls.

Test matrix:
1. Happy path: `list_messages` returns `status_code: 200` with empty items page.
2. Guard: empty `channel_id` returns `Err(MessengerRestError::MissingPathChannel)`.
3. Validate seam: missing `idempotency_key` returns
   `Err(MessengerRestError::Api(MessengerApiError::MissingIdempotencyKey))`.
4. Validate seam: missing `policy_decision_ref` returns
   `Err(MessengerRestError::Api(MessengerApiError::MissingPolicyDecision))`.
5. dispatch_read_route: implemented GET path routes to handler and returns 200.
6. dispatch_read_route: ContractOnly read route returns ContractOnly error.
7. dispatch_read_route: unknown path returns UnknownRoute.
8. Regression: `openapi_route_catalog_covers_declared_operations` (len==26, all Implemented statuses) stays green.
9. Regression: `messenger_contract_only_dispatch_refuses_typed_handler_routes` — after the route flip the GET messages route must also be excluded from ContractOnly 501 path.

## Boundaries

- Single file modified: `crates/messenger-message-stream-rest/src/lib.rs`.
- OPENAPI_ROUTES length stays at 26; no route is added or removed.
- No new crates, no root Cargo.toml edits, no other task's files.
- No async, no I/O, no external dependencies added.
