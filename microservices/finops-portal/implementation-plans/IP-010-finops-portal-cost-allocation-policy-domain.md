---
ip_id: IP-010
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/cost-allocation-policy/domain
related_adrs: [ADR-0131, ADR-0174, ADR-0199]
depends_on: [IP-009]
target_lines: 150
---

# IP-010 — `cost-allocation-policy` domain slice

## Why this slice

The domain layer for `cost-allocation-policy` owns:

- Policy promotion lifecycle (`draft → reviewed → active → retired`).
- Conflict resolution when overlapping scope rules apply
  (precedence: `Tenant > RegulatoryPack > Fleet`).
- Policy review-quorum invariant (per ADR-0174 a policy at
  `Fleet` scope requires two ops-finops reviewers).
- Effective-window computation (when does a newly-active policy
  start applying — calendar-month boundary alignment).

This is the layer where the chargeback-formula correctness for
**shared-resource attribution** is enforced.

## Acceptance criteria

1. New crate
   `crates/oya-finops-portal-cost-allocation-policy-domain/` depends
   on the kernel from IP-009.
2. Public function `promote_policy`:
   ```rust
   pub fn promote_policy(
       current: PolicyLifecycleState,
       proposed: PolicyLifecycleState,
       reviewers: &[Reviewer],
   ) -> Result<PolicyLifecycleState, PromotionError>;
   ```
3. Public function `resolve_overlapping_policies`:
   ```rust
   pub fn resolve_overlapping_policies(
       policies: &[CostAllocationPolicy],
   ) -> Result<EffectivePolicySet, ConflictError>;
   ```
4. Effective-window function:
   ```rust
   pub fn effective_window(
       policy: &CostAllocationPolicy,
       now: OffsetDateTime,
   ) -> InvoicePeriod;
   ```
5. ≥ 7 unit tests:
   - lifecycle transitions: draft → reviewed → active.
   - illegal transitions rejected (e.g. retired → active).
   - quorum-of-2 required for fleet-scope active.
   - tenant overrides regulatory-pack overrides fleet.
   - effective window aligns to calendar-month start.
   - conflict between two same-scope policies returns
     ConflictError.
   - retired policy excluded from EffectivePolicySet.
6. `cargo test -p oya-finops-portal-cost-allocation-policy-domain`
   green.

## File-level work plan

1. `Cargo.toml` — depends on kernel; `time`.
2. `src/lib.rs`.
3. `src/lifecycle.rs` — promotion state machine.
4. `src/resolve.rs` — overlap resolution.
5. `src/window.rs` — effective window.
6. `src/error.rs`.

## Lifecycle state machine

```
draft ──submit── reviewed ──approve(2x)── active ──supersede── retired
  │                  │                          ↑
  │                  └──reject──┐               │
  └─edit─────────────────────────└──draft──────┘
```

Invariants enforced:

- Cannot skip states (e.g. `draft → active` is rejected).
- `active → retired` only via `supersede` (which requires a
  reference to the new active policy).
- Once `retired`, no transition (terminal).

## Precedence rules (per ADR-0174 §shared-resource attribution)

When multiple policies have overlapping scope on the same cost-
center over the same period:

1. **Tenant scope wins** over RegulatoryPack scope.
2. **RegulatoryPack scope wins** over Fleet scope.
3. Within the same scope, the policy with the higher `version`
   number wins.
4. If two policies have the same scope + version, that is a
   `ConflictError` (data corruption; reconciler alerts).

## Effective-window alignment

A policy activated at `2026-05-18T12:34:56Z` starts applying at the
**next calendar-month boundary** in the policy's residency timezone
(per the manifest's regulatory pack). For Fleet scope the timezone
is UTC; for Tenant scope it is the tenant's profile timezone.

This calendar-month alignment is what makes monthly invoice
composition deterministic.

## Risk + mitigation

- **Risk**: clock skew between reviewers causes quorum miscount.
  **Mitigation**: the second reviewer's timestamp must be ≥ first
  reviewer's; otherwise quorum is denied.
- **Risk**: a retired policy is silently re-used. **Mitigation**:
  `EffectivePolicySet` does not include retired policies; the
  reconciler audit-emits any access attempt to a retired policy.

## Out-of-scope

- Persistence — usecase.
- API — separate api crate.

## References

- ADR-0174 — chargeback formula + shared-resource attribution.
- ADR-0199 — cost-attribution canonical.

## Verification

- `cargo test -p oya-finops-portal-cost-allocation-policy-domain`.
- `oya gate domain-tier-invariants --crate
  oya-finops-portal-cost-allocation-policy-domain`.
