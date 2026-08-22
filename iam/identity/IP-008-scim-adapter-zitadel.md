---
doc_class: IP
ip_id: IP-008
microservice: identity
status: ga
related_adrs: [ADR-0190, ADR-0187]
date: 2026-05-18
owner_team: axis-identity
---

# IP-008 — SCIM Zitadel adapter (lifecycle propagation)

## Goal

Wire the `shared-scim-server-kernel` reference impl to a Zitadel-talking adapter so SCIM operations propagate to Zitadel as the upstream identity authority. Specifically: SCIM POST `/Users` creates a Zitadel User in the corresponding Org; SCIM PATCH `active=false` revokes Zitadel sessions; SCIM DELETE issues a Zitadel User soft-delete + GDPR-DSR cascade trigger.

## Files

| File | Purpose |
|---|---|
| `crates/identity-scim-server-adapter-zitadel/Cargo.toml` | manifest |
| `crates/identity-scim-server-adapter-zitadel/src/lib.rs` | `ZitadelScimBridge` impl of `UserStore` + `GroupStore` |
| `crates/identity-scim-server-adapter-zitadel/src/zitadel_client.rs` | thin Zitadel gRPC admin client |
| `crates/identity-scim-server-adapter-zitadel/tests/bridge.rs` | tests against mock Zitadel |

## Operation mapping

| SCIM | Zitadel admin API |
|---|---|
| POST /Users | `Management.AddHumanUser` |
| PATCH /Users (active=true) | `Management.ReactivateUser` |
| PATCH /Users (active=false) | `Management.DeactivateUser` + `Auth.RevokeAllUserSessions` |
| PATCH /Users (other) | `Management.UpdateHumanUser` |
| DELETE /Users | `Management.RemoveUser` + emit `IdentityUserDeleted` triggering DSR cascade |
| POST /Groups | `Management.AddGroup` |
| PATCH /Groups (members add) | `Management.AddGroupMember` |
| PATCH /Groups (members remove) | `Management.RemoveGroupMember` |
| DELETE /Groups | `Management.RemoveGroup` |

## Failure / partial-failure handling

SCIM operations are idempotent (idempotency key required per ADR-0149). If Zitadel returns an error mid-operation:
- 5xx → retry with exponential backoff up to 3 times; then 503.
- 4xx (logical error like already-exists) → translate to SCIM error envelope.
- Timeout → 504; client retries with same idempotency key (which dedupes server-side).

## Lifecycle event emission

Every SCIM operation that mutates emits an audit event to the audit-emitter:

- `IdentityUserProvisioned(tenant_id, user_id, external_id, source)` where `source` ∈ `scim`, `hris`, `manual`, `migration`.
- `IdentityUserSuspended`, `IdentityUserReactivated`, `IdentityUserDeleted` with reason.
- `IdentityGroupProvisioned`, `IdentityGroupMembershipChanged` with delta.

## Dialect quirks per vendor

| Vendor | Quirk | Adapter handling |
|---|---|---|
| Okta | sends `userName` as email-format | normalise: lowercase + UTF-8 NFKC |
| Microsoft Entra | sends `urn:ietf:params:scim:schemas:extension:enterprise:2.0:User` even when no enterprise data | tolerate; if all sub-fields nil, drop the extension |
| Google Workspace | uses `displayName` instead of `name.formatted` | accept either, prefer `displayName` if both present |
| OneLogin | sends `id` on POST (RFC says server-assigned) | ignore client-supplied `id`; assign server-side |
| JumpCloud | uses non-standard `urn:scim:schemas:extension:enterprise:1.0` (note "1.0" typo) | tolerate; map to enterprise 2.0 |

## Tests

| Test | Mechanism |
|---|---|
| `create_user_via_scim_creates_in_zitadel` | mock Zitadel; assert AddHumanUser called |
| `deactivate_revokes_sessions` | mock Zitadel; assert both DeactivateUser + RevokeAllUserSessions called |
| `delete_triggers_dsr_cascade` | mock cascade event sink; assert event emitted |
| `retries_on_5xx_then_succeeds` | flaky mock fails twice then ok; assert 3 retries observed |
| `idempotency_key_dedupes` | same key twice; only one Zitadel call |
| `okta_username_normalised` | mixed-case email in; lowercase out |
| `entra_empty_extension_dropped` | shape passes |
| `unknown_zitadel_error_maps_to_scim_500` | unrecognised error → InvalidSyntax+500 |
| `cross_tenant_scim_bearer_refused` | bearer for tenant A; ask for tenant B → 403 |
| `concurrent_modify_returns_etag_mismatch_412` | two PATCH; second loses |

## Evidence

- `evidence/identity/scim-zitadel-mapping/<date>.json`
- `evidence/identity/vendor-quirk-conformance/<vendor>-<date>.json` for each of Okta / Entra / Workspace / OneLogin / JumpCloud

## Acceptance — DONE when

- 10 adapter-tests pass.
- Live SCIM conformance test against actual Zitadel passes (preview pack).
- Each vendor quirk has at least one passing test.

## Counterpart references - 008-scim-adapter-zitadel

- Counterpart class: workforce lifecycle.
- ServiceNow workforce workflows and GitHub enterprise SSO show the baseline for enterprise identity lifecycle; this IP keeps Oyatie stronger by routing lifecycle changes through SCIM/HRIS contracts, tenant-scoped Cedar, and audit-chain evidence instead of relying on tenant-admin convention.
- Verification anchor: this row intentionally includes a named counterpart from the Wave 15 grep allowlist while keeping the implementation reference service-local: `microservices/identity/competitor-parity-matrix.md`, `microservices/identity/PRD.md`, `microservices/identity/manifest.json`, and the contract/policy files cited above.

## Sustainability emission (per ADR-0344)

- Authority: ADR-0344.
- Trigger evidence: `microservices/identity/IP-008-scim-adapter-zitadel.md` matched `emission`.
- Per-call audit row fields: `cost_usd_minor_units`, `co2_grams`, `watt_hours`.
- Emission evidence: `microservices/identity/manifest.json` plus this IP's metered trigger text.
- Carbon-aware scheduling: eligible only when ADR-0344 D-9 compliance-pack exclusions do not bar deferral; otherwise the Cedar scheduler rejects delay while still emitting carbon fields.
- finops-portal rollup axes affected: tenant / product / capability / provider / cell.
