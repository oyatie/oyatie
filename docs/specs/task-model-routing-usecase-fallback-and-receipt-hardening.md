# Spec: model-routing-usecase-fallback-and-receipt-hardening

**Vertical:** intelligence  
**Crate:** `intelligence-model-routing-usecase`  
**Task slug:** `model-routing-usecase-fallback-and-receipt-hardening`  
**Branch:** `feat/task-model-routing-usecase-fallback-and-receipt-hardening-2026-05-28`  
**Base:** `origin/dev`

---

## Objective

Harden the deterministic model-routing usecase layer with three coupled
improvements:

1. **Catalog-walk fallback** — when the primary domain `RouteSelection` is
   denied by a *recoverable* `RouteDenialReason`, deterministically walk the
   remaining `ProviderRouteProfile` entries in stable priority order and surface
   either a `Routed` receipt (first eligible candidate) or a `Denied` receipt
   (all candidates exhausted).
2. **Per-candidate denial trail** — on the `Denied` path, enumerate every
   rejected `ProviderRouteProfile` with its per-candidate `RouteDenialReason`
   set in the `ModelRoutingUsecaseReceipt` (metadata refs only; no credential
   or provider payloads).
3. **Idempotency replay under fallback** — re-submitting the same
   `idempotency_key` returns the identical receipt (including the chosen
   fallback candidate and the denial trail) without re-running the catalog walk.

### Non-goals

- No provider calls, credential resolution, network I/O, filesystem access,
  or durable state changes (in-memory only).
- No new crates, no root `Cargo.toml` edits.
- No changes to `intelligence-model-routing-domain` or
  `intelligence-model-routing-kernel`.

---

## Vertical and Layer Context

```
intelligence-model-routing-kernel   (pure value types + decide_route)
        ↑ path-dep
intelligence-model-routing-domain   (validation + route_validated_request)
        ↑ path-dep
intelligence-model-routing-usecase  ← THIS CRATE (usecase orchestration)
```

The usecase crate is the outermost in this vertical slice. It owns:

- Input validation (`validate_input`)
- Idempotency keying (`receipts_by_idempotency_key`)
- Domain delegation (`route_validated_request`)
- Metadata-only audit event emission (`events`)
- Receipt assembly (`receipt_from_input`)

This task extends the domain delegation step with a catalog-walk loop and
extends the receipt type with a denial trail. All changes are within
`src/lib.rs`; no new modules.

---

## Mod Layout (flat-clean-arch within single `src/lib.rs`)

The crate is a single-file library. Logical groupings within `lib.rs`:

| Region | Contents |
|---|---|
| Public types | `ModelRoutingUsecaseInput`, `ModelRoutingUsecaseStatus`, `ModelRoutingUsecaseDenialKind`, `CandidateDenial` (new), `ModelRoutingUsecaseReceipt`, `ModelRoutingAuditEventKind`, `ModelRoutingAuditEvent` |
| Private state | `ModelRouteIntent`, `IntelligenceModelRoutingUsecase` |
| Usecase entry | `IntelligenceModelRoutingUsecase::route` |
| Catalog-walk (new) | `walk_catalog_for_route` (free fn, private) |
| Denial classification (new) | `is_recoverable_denial` (free fn, private) |
| Receipt assembly | `receipt_from_input` |
| Validation helpers | `validate_input`, `validate_request`, `validate_profile` |
| Metadata safety | `safe_metadata`, `safe_tenant`, `safe_ref`, `is_safe_metadata_ref`, `is_safe_opaque_ref`, `contains_whitespace`, `contains_raw_secret_material`, `contains_raw_content_material` |
| Fingerprint | `canonical_fingerprint`, `canonical_profile`, `canonical_entry`, `canonical_request_evidence_refs` |
| Util | `sorted_unique` |
| Tests | `#[cfg(test)] mod tests` |

---

## Type Contracts

### New type: `CandidateDenial`

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDenial {
    pub provider: ModelProvider,                // data_class: PUBLIC
    pub model_id: String,                       // data_class: INTERNAL_ONLY
    pub priority: u16,                          // data_class: INTERNAL_ONLY
    pub reasons: BTreeSet<RouteDenialReason>,   // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,             // data_class: INTERNAL_ONLY
}
```

Carries only metadata fields from `ProviderRouteProfile`. Does not carry
`credential_modes`, `allowed_tenants`, `enabled`, or any resolved credential
material. `evidence_refs` are opaque ref strings only.

### Extended type: `ModelRoutingUsecaseReceipt`

New field added (all others unchanged):

```rust
pub candidate_denials: Vec<CandidateDenial>,  // data_class: INTERNAL_ONLY
```

- On a `Routed` receipt: the list of candidates tried and denied before the
  selected one (empty if the first candidate matched).
- On a `Denied` receipt: every candidate enumerated in stable priority order
  with its specific denial reasons.
- On `InvalidInput` or `IdempotencyConflict` receipts: empty (catalog walk was
  not reached).

### Denial reason classification

```
Recoverable (candidate-level — walk continues):
  CapabilityUnavailable
  CredentialModeUnavailable
  DataClassNotAllowed
  AudienceNotAllowed
  TenantNotAllowed
  NoEnabledProvider

Terminal (request-level — fail fast, surfaced by DomainRouteDecision::Invalid):
  Any DomainRouteDecision::Invalid variant (validation failures:
  EmptyTenantId, InvalidTenantId, EmptyEvidenceRef, DisabledCredentialMode,
  ExternalAudienceSensitiveData)
```

A `DomainRouteDecision::Invalid` result means the request itself is
malformed — no candidate walk is attempted.

### Catalog-walk algorithm

```
sorted_catalog ← sort input.catalog by (priority ASC, provider ASC, model_id ASC)
candidate_denials ← []

for each profile in sorted_catalog:
    match route_validated_request(request, &[profile]):
        DomainRouteDecision::Invalid(denial):
            // terminal — request is malformed regardless of candidate
            return Denied receipt (RouteDenied, denial, candidate_denials so far)
        DomainRouteDecision::Routed(RouteDecision::Allow(selection)):
            return Routed receipt (selection, candidate_denials so far)
        DomainRouteDecision::Routed(RouteDecision::Deny(denial)):
            // recoverable — record and continue
            candidate_denials.push(CandidateDenial from profile + denial)

// all candidates exhausted
return Denied receipt (RouteDenied, aggregate denial, candidate_denials)
```

Sort key `(priority ASC, provider ASC, model_id ASC)` matches the kernel's
`select_highest_ranked_candidate` tiebreak, ensuring the first candidate
selected by a single-pass walk is identical to what the kernel would have
chosen if all were eligible.

---

## Contracts: OpenAPI 3.2.0 (usecase boundary — not a network endpoint)

This usecase crate does not expose an HTTP surface directly. The contracts
below describe the logical I/O boundary for downstream adapter integration.

```yaml
# openapi: 3.2.0
# info: model-routing usecase boundary (logical, not an HTTP server)

components:
  schemas:
    ModelRoutingUsecaseInput:
      type: object
      required:
        - idempotency_key
        - principal_id
        - trace_context_ref
        - policy_decision_ref
        - route_registry_snapshot_ref
        - request
        - catalog
      properties:
        idempotency_key:
          type: string
          description: "Opaque idempotency key. data_class: INTERNAL_ONLY"
        principal_id:
          type: string
          description: "Routing principal. data_class: INTERNAL_ONLY"
        trace_context_ref:
          type: string
          description: "Opaque trace ref (must contain ':'). data_class: INTERNAL_ONLY"
        policy_decision_ref:
          type: string
          description: "Opaque policy decision ref. data_class: INTERNAL_ONLY"
        route_registry_snapshot_ref:
          type: string
          description: "Opaque registry snapshot ref. data_class: INTERNAL_ONLY"
        request:
          $ref: '#/components/schemas/ModelRouteRequest'
        catalog:
          type: array
          items:
            $ref: '#/components/schemas/ProviderRouteProfile'

    CandidateDenial:
      type: object
      required: [provider, model_id, priority, reasons, evidence_refs]
      properties:
        provider:
          type: string
          enum: [Anthropic, AzureOpenAi, Gemini, Local, OpenAi]
          description: "data_class: PUBLIC"
        model_id:
          type: string
          description: "data_class: INTERNAL_ONLY"
        priority:
          type: integer
          format: uint16
          description: "data_class: INTERNAL_ONLY"
        reasons:
          type: array
          items:
            $ref: '#/components/schemas/RouteDenialReason'
          description: "data_class: INTERNAL_ONLY"
        evidence_refs:
          type: array
          items:
            type: string
          description: "Opaque evidence refs only. data_class: INTERNAL_ONLY"

    ModelRoutingUsecaseReceipt:
      type: object
      required:
        - idempotency_key
        - tenant_id
        - principal_id
        - trace_context_ref
        - policy_decision_ref
        - route_registry_snapshot_ref
        - status
        - evidence_refs
        - candidate_denials
      properties:
        status:
          type: string
          enum: [Routed, Denied]
          description: "data_class: PUBLIC"
        denial_kind:
          type: string
          nullable: true
          enum: [IdempotencyConflict, InvalidInput, RouteDenied]
          description: "data_class: INTERNAL_ONLY"
        route_selection:
          nullable: true
          $ref: '#/components/schemas/RouteSelection'
        route_denial:
          nullable: true
          $ref: '#/components/schemas/RouteDenial'
        candidate_denials:
          type: array
          items:
            $ref: '#/components/schemas/CandidateDenial'
          description: "Per-candidate denial trail. data_class: INTERNAL_ONLY"
        evidence_refs:
          type: array
          items:
            type: string
```

---

## Testing Strategy

All tests live in `#[cfg(test)] mod tests` inside `lib.rs`. No separate test
files. Patterns follow existing tests in the crate.

### SUB-1 tests

| Test name | Behaviour |
|---|---|
| `fallback_to_second_candidate_when_first_is_denied` | Catalog: profile-A (priority 1, capability mismatch), profile-B (priority 2, fully eligible). Assert receipt is `Routed` naming profile-B. Assert `candidate_denials` has exactly one entry (profile-A). |
| `terminal_denial_fails_fast_without_walking_catalog` | Request with `ExternalEndUser` + `PiiIdentifying` (triggers `DomainRouteDecision::Invalid`). Assert `Denied` with `InvalidInput` denial kind, `candidate_denials` empty, only one domain call attempted. |
| `fallback_ordering_is_deterministic_with_shuffled_catalog` | Same two profiles, submit twice with catalog order reversed. Assert both receipts are `PartialEq` (same chosen candidate, same `candidate_denials`). |

### SUB-2 tests

| Test name | Behaviour |
|---|---|
| `all_candidates_denied_receipt_enumerates_full_trail_in_stable_order` | Catalog: three profiles, each denied for a distinct recoverable reason. Assert `status == Denied`, `candidate_denials.len() == 3`, each entry has correct `reasons`, stable order matches priority sort. Assert no field in the receipt debug representation carries secret material or raw content. |
| `metadata_only_invariant_no_secrets_in_denial_trail` | Assert `format!("{receipt:?}")` contains none of: `"sk-"`, `"bearer"`, `"raw prompt"`, `"raw output"`. |

### SUB-3 tests

| Test name | Behaviour |
|---|---|
| `fallback_receipt_replays_identically_under_same_key` | Route a request that falls back to the second candidate. Re-submit same `idempotency_key`. Assert second receipt `PartialEq` first (including `candidate_denials` and chosen candidate). Assert `cached_receipt_count() == 1`. |
| `conflicting_payload_on_fallback_key_yields_idempotency_conflict` | Route a fallback request. Re-submit same key with different `capability`. Assert `IdempotencyConflict` denial kind. Assert original receipt still retrievable via clean replay. |

### Regression tests (existing — must remain green)

All five existing tests in the crate must pass without modification:
- `routes_authorized_request_with_metadata_audit_and_idempotency`
- `route_denial_records_fail_closed_metadata_audit`
- `invalid_raw_metadata_denies_before_cache_or_audit_side_effects`
- `idempotency_conflict_denies_without_replacing_original_receipt`
- `domain_sensitive_external_audience_denial_is_preserved`
- `receipts_and_events_never_contain_raw_prompt_output_or_secret_bytes`

---

## Boundaries and Constraints

| Constraint | Detail |
|---|---|
| No durable I/O | All state is in-memory (`BTreeMap`, `Vec`). No file, network, or clock access. |
| No credential resolution | `CandidateDenial` carries only `provider`, `model_id`, `priority`, `reasons`, `evidence_refs`. |
| Metadata-only audit | `ModelRoutingAuditEvent` and `ModelRoutingUsecaseReceipt` carry only opaque refs. |
| Panic-free | No `unwrap`/`expect`/`panic` outside `#[cfg(test)]`. |
| Path-dep inward only | `intelligence-model-routing-usecase` depends only on `intelligence-model-routing-domain`; domain depends only on kernel. |
| std-only | No async, no tokio, no external dependencies beyond the path-dep chain. |
| Single crate | All changes in `intelligence-model-routing-usecase/src/lib.rs`. No new crates or workspace edits. |
| `#![cfg_attr(test, allow(clippy::unwrap_used, ...))]` retained | Existing test-mode clippy allowances unchanged. |

---

## Verification Commands

```sh
cargo check -p intelligence-model-routing-usecase --all-targets
cargo nextest run -p intelligence-model-routing-usecase
```

Both must exit 0. Run from the worktree root
`/tmp/task-model-routing-usecase-fallback-and-receipt-hardening-2026-05-28`.
