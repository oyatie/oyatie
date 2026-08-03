# Plan: model-routing-usecase-fallback-and-receipt-hardening

**Vertical:** intelligence  
**Crate:** `oya-intelligence-model-routing-usecase`  
**Branch:** `feat/task-model-routing-usecase-fallback-and-receipt-hardening-2026-05-28`  
**Stage:** SPEC → IMPL → VERIFY

---

## Objective

Harden the deterministic model-routing usecase
(`ModelRoutingUsecaseInput` → `ModelRoutingUsecaseReceipt`) with:

1. Ordered candidate-fallback selection when the primary route is denied by a
   recoverable `RouteDenialReason`.
2. A per-candidate denial trail on the `Denied` path (metadata-only refs, no
   secrets or provider payloads).
3. Idempotency replay correctness under the new fallback path.

All changes are confined to `oya-intelligence-model-routing-usecase/src/lib.rs`.
No new crates. No root `Cargo.toml` edits. No durable I/O.

---

## Subtasks

### SUB-1 — Catalog-walk fallback (recoverable vs terminal denial classification)

**What:** When `route_validated_request` denies the top candidate with a
*recoverable* `RouteDenialReason`, iterate the remaining `ProviderRouteProfile`
entries in stable priority order (ascending `priority`, then `provider`, then
`model_id` — matching the kernel's `select_highest_ranked_candidate` tiebreak)
and select the first that validates. Classify denial reasons as recoverable
(candidate-level: `CapabilityUnavailable`, `CredentialModeUnavailable`,
`DataClassNotAllowed`, `AudienceNotAllowed`, `TenantNotAllowed`,
`NoEnabledProvider`) vs terminal (request-level failures surfaced by
`DomainRouteDecision::Invalid`). Terminal failures fail fast without walking
the catalog.

**Implementation steps:**

1. Sort the input catalog snapshot into stable priority order at the start of
   `route()` (before the domain call). The current code passes `input.catalog`
   directly; sorting a local copy preserves the existing fingerprint (which
   already sorts profiles).
2. Replace the single `route_validated_request` call with a catalog-walk loop:
   try `route_validated_request(request, &[candidate])` for each profile in
   priority order; stop on first `Allow` or on a `DomainRouteDecision::Invalid`
   (terminal).
3. Collect per-candidate `RouteDenial` values as the walk proceeds.
4. On exhaustion, produce a `Denied` receipt with the full denial trail.

**Acceptance:**

- `cargo check -p intelligence-model-routing-usecase --all-targets` green.
- `cargo nextest run -p intelligence-model-routing-usecase` green.
- Test (a): recoverable top-candidate denial falls through to next eligible
  candidate → `Routed` receipt naming that candidate.
- Test (b): terminal denial (`DomainRouteDecision::Invalid`) fails fast without
  walking the rest of the catalog.
- Test (c): ordering is deterministic across repeated runs with
  shuffled-but-equivalent catalogs.

---

### SUB-2 — Per-candidate denial trail on the `Denied` path

**What:** Add a `candidate_denials: Vec<CandidateDenial>` field to
`ModelRoutingUsecaseReceipt`. Each `CandidateDenial` pairs a
`ProviderRouteProfile` *reference* (metadata fields only: `provider`,
`model_id`, `priority`) with its `RouteDenialReason` set and evidence refs.
No credential fields, no network-resolved data, no provider secrets.

**New types (in `lib.rs`, no new modules):**

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateDenial {
    pub provider: ModelProvider,           // data_class: PUBLIC
    pub model_id: String,                  // data_class: INTERNAL_ONLY
    pub priority: u16,                     // data_class: INTERNAL_ONLY
    pub reasons: BTreeSet<RouteDenialReason>, // data_class: INTERNAL_ONLY
    pub evidence_refs: Vec<String>,        // data_class: INTERNAL_ONLY
}
```

`ModelRoutingUsecaseReceipt` gains:

```rust
pub candidate_denials: Vec<CandidateDenial>, // data_class: INTERNAL_ONLY
```

On the `Routed` path, `candidate_denials` is the list of profiles tried and
denied before the selected one (may be empty if first candidate matched). On
the fully-`Denied` path, it enumerates every candidate in stable order.

**Acceptance:**

- `cargo nextest run -p intelligence-model-routing-usecase` green.
- Test: all candidates denied → receipt enumerates every candidate with its
  specific `RouteDenialReason` in stable order; `status` is `Denied`; no field
  carries provider secrets or network-resolved data (metadata-only invariant).

---

### SUB-3 — Idempotency replay correctness under fallback

**What:** The canonical fingerprint already covers the full catalog; replay
correctness is preserved because fingerprint computation is unchanged. Verify
explicitly that:

- Re-submitting the same `idempotency_key` returns the identical receipt
  (same chosen candidate + same `candidate_denials`) without re-running the
  catalog walk.
- A conflicting payload on the same key yields
  `ModelRoutingUsecaseDenialKind::IdempotencyConflict`.

No code change needed beyond what SUB-1/SUB-2 introduce — the existing
`receipts_by_idempotency_key` BTreeMap and fingerprint logic already cover
this. This subtask is test-only verification.

**Acceptance:**

- `cargo nextest run -p intelligence-model-routing-usecase` green.
- Test: fallback-resolved request → replay same key → second receipt
  `PartialEq` first (including `candidate_denials` and chosen candidate).
- Test: conflicting payload on same key →
  `ModelRoutingUsecaseDenialKind::IdempotencyConflict`.

---

## Acceptance Gate (overall)

```sh
cargo check -p intelligence-model-routing-usecase --all-targets
cargo nextest run -p intelligence-model-routing-usecase
```

Both must exit 0 with all tests green. No `unwrap`/`expect`/`panic` outside
`#[cfg(test)]`. No `console::log`, `TODO`, `HACK`, or `debugger` residue.
