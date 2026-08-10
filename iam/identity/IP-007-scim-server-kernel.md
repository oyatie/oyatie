---
doc_class: IP
ip_id: IP-007
microservice: identity
status: ga
related_adrs: [ADR-0190]
related_crates: [oya-shared-scim-server-kernel]
date: 2026-05-18
owner_team: axis-identity
---

# IP-007 — SCIM 2.0 server kernel + RFC 7644 conformance

## Goal

Land `oya-shared-scim-server-kernel`: RFC 7643 / 7644 conformant SCIM 2.0 server kernel with User + Group + EnterpriseExtension + OyatieExtension schemas, filter parser handling the in-the-wild dialect subset used by Okta / Entra / Workspace, PATCH semantics (RFC 7644 §3.5.2), ETag-based concurrency, in-memory stores for tests + reference, and pluggable `UserStore` / `GroupStore` / `IdGen` traits for production.

## Files

| File | Purpose |
|---|---|
| `crates/oya-shared-scim-server-kernel/Cargo.toml` | manifest |
| `crates/oya-shared-scim-server-kernel/src/lib.rs` | trait + types + parser + reference impl |
| `crates/oya-shared-scim-server-kernel/tests/scim_server_kernel.rs` | integration tests |

LoC: ~720 (lib) + ~250 (tests) = ~970 lines.

## Resource schemas

| URN | Resource |
|---|---|
| `urn:ietf:params:scim:schemas:core:2.0:User` | User |
| `urn:ietf:params:scim:schemas:extension:enterprise:2.0:User` | enterprise ext |
| `urn:oyatie:scim:extension:2.0:User` | oyatie ext (`regulatory_pack`, `acr_floor`, `data_residency_jurisdiction`) |
| `urn:ietf:params:scim:schemas:core:2.0:Group` | Group |

## Filter parser scope

Subset of RFC 7644 §3.4.2.2 actually used in the wild:

- Comparison: `eq`, `ne`, `co`, `sw`, `ew`, `pr`
- Logical: `and`, `or`, `not`
- Grouping: `(...)`
- String literals: `"..."` with `\"` escape

Numeric / boolean filters parse but resolve to string comparison (sufficient for `active eq "true"`).

NOT supported (rejected with `InvalidFilter`):
- `gt`, `ge`, `lt`, `le` (numeric ordering rarely needed for users)
- Sub-attribute paths (`emails[type eq "work"].value`) at the filter level — supported only at PATCH path level
- Complex attribute filter expressions (`emails[primary eq "true" and value eq "..."]`)

## PATCH operation semantics

Per RFC 7644 §3.5.2:

- `add` on multi-valued attribute → append.
- `replace` on top-level attribute → set.
- `replace` on multi-valued → replace entire array.
- `remove` on multi-valued with `value` filter → remove matching items.

Bracket-filter on remove (`emails[value eq "..."]`) supported via hand-rolled `parse_filter_value_in_brackets` helper.

## Tests (22 shipped)

1. `create_then_get_user_roundtrips`
2. `create_user_username_uniqueness`
3. `create_user_requires_username`
4. `list_users_paginates`
5. `list_users_caps_items_per_page`
6. `patch_user_replaces_active`
7. `patch_user_unknown_path_returns_invalidpath`
8. `delete_user_then_get_404s`
9. `delete_unknown_user_404s`
10. `create_group_then_patch_members`
11. `replace_user_preserves_created_timestamp`
12. `filter_eq_matches_username`
13. `filter_co_matches_substring`
14. `filter_and_combines_conditions`
15. `filter_pr_present`
16. `filter_parser_rejects_garbage`
17. `filter_parser_handles_or`
18. `scim_error_envelope_serializes_per_rfc_7644`
19. `group_create_requires_displayname`
20. `patch_remove_email_by_value`
21. `tenant_isolation_users_dont_leak_across_tenants`
22. `group_membership_query_does_not_break_users`

## Error envelope

Per RFC 7644 §3.12:

```json
{
  "schemas": ["urn:ietf:params:scim:api:messages:2.0:Error"],
  "status": 409,
  "scimType": "uniqueness",
  "detail": "userName 'alice' already exists"
}
```

## ETag concurrency

`meta.version` stores `W/"<etag>"`; HTTP handler-layer compares against `If-Match`. Mismatch → 412 with `scimType=invalidVers`.

## RFC 7644 conformance test set

Per ADR-0190 §"Verification", `scim2-compliance` (Go tool) runs against the live server in CI; required to pass at GA.

## Evidence

- `evidence/identity/scim-conformance/<date>.json`
- `evidence/identity/scim-dialect-quirks-handled/<vendor>-<date>.md`

## Acceptance — DONE when

- 22 unit + integration tests passing.
- `scim2-compliance` external suite passes.
- Filter parser fuzzed for 100k random inputs without panic.

## Counterpart references - 007-scim-server-kernel

- Counterpart class: workforce lifecycle.
- ServiceNow workforce workflows and GitHub enterprise SSO show the baseline for enterprise identity lifecycle; this IP keeps Oyatie stronger by routing lifecycle changes through SCIM/HRIS contracts, tenant-scoped Cedar, and audit-chain evidence instead of relying on tenant-admin convention.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `iam/identity/PRD.md`, `iam/identity/manifest.json`, and the contract/policy files cited above.

