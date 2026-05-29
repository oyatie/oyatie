---
ip_id: IP-009
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/cost-allocation-policy/kernel
related_adrs: [ADR-0083, ADR-0131, ADR-0174, ADR-0199]
depends_on: []
target_lines: 150
---

# IP-009 — `cost-allocation-policy` kernel slice

## Why this slice

The `cost-allocation-policy` BC owns the editable rules that say who
pays for what when resources are shared. The kernel is the pure
typed contract for these rules. Examples:

- "Shared cell capacity is allocated by `tenant_request_share`
  (default), or by `flat_split_among_active_tenants`."
- "Foundry invocation cost is allocated to the invoking tenant
  unless the workflow is marked `internal_use_only=true`."
- "Audit-chain emit cost is split 50/50 between the µservice
  emitting and a fleet-overhead cost-center."

The kernel types make these rules persistable, diff-able, and
audit-emittable. The kernel is **dependency-free** and pure-Rust per
ADR-0083 Tier-A invariants.

## Acceptance criteria

1. New crate
   `crates/oya-finops-portal-cost-allocation-policy-kernel/`.
2. Public types:
   - `CostAllocationPolicy` — name, scope (`Fleet` |
     `RegulatoryPack(Pack)` | `Tenant(TenantId)`), version,
     authored_at, authored_by, rules.
   - `AllocationRule` — enum:
     - `ProportionalToTenantRequestShare { cost_center, denominator: DenominatorKey }`.
     - `FlatSplitAmongActiveTenants { cost_center, active_window: Period }`.
     - `AssignToInvokingTenant { cost_center }`.
     - `SplitBetweenTwoCostCenters { primary, primary_share_pct, secondary }`.
     - `CustomFormula(FormulaExpr)` — closed-set DSL.
   - `FormulaExpr` — abstract syntax tree (whitelisted operators
     only).
   - `PolicyDiff` — additive diff between two policy versions.
   - `PolicyValidationError`.
3. Public trait `PolicyEvaluator`:
   - `fn evaluate(&self, policy: &CostAllocationPolicy, period:
     &InvoicePeriod, raw_cost: &RawCostInput) -> Result<AllocatedCost, PolicyValidationError>`.
4. Reference in-memory evaluator implements `PolicyEvaluator`.
5. Tier-A 4-INV kernel invariants (per ADR-0083) enforced:
   - No `std::io`, no `tokio`, all wire-types `#[non_exhaustive]`,
     `CostAllocationPolicy` impls `Ord` for stable diffing.
6. ≥ 8 unit tests:
   - each `AllocationRule` variant evaluated against a fixture.
   - `PolicyDiff` correctness round-trip.
   - rejects custom-formula with disallowed operator.
   - deterministic ordering of multi-rule evaluation.
7. `cargo test -p oya-finops-portal-cost-allocation-policy-kernel`
   green.

## File-level work plan

1. `Cargo.toml` — no deps beyond `serde`, `thiserror`, `time`.
2. `src/lib.rs` — module roots.
3. `src/policy.rs` — `CostAllocationPolicy` + scope enum.
4. `src/rules.rs` — `AllocationRule` enum + variants.
5. `src/formula.rs` — `FormulaExpr` whitelist + evaluator.
6. `src/diff.rs` — `PolicyDiff` + diff algorithm.
7. `src/eval.rs` — `PolicyEvaluator` trait + reference impl.
8. `src/error.rs` — `PolicyValidationError`.

## Custom-formula safety

The `FormulaExpr` AST allows only:

- Numeric literals (USD cents, integers, percents).
- Variable refs scoped to `period.*` and `tenant.*` fixed namespace.
- Operators: `+`, `-`, `*`, `min`, `max`, `clamp(lo, hi)`.
- No I/O, no recursion, no time travel (`time-now` is not a
  valid var).
- Max AST depth 16 (rejected beyond).

A unit test asserts that any string with unknown operator or
unknown var-ref fails parsing.

## Audit-chain seal mapping

Every `CostAllocationPolicy` version change emits a
`CostAllocationPolicyChanged` audit-chain event. The seal envelope
includes:

- `policy.name`, `policy.scope`, `policy.version`,
  `policy.authored_by`, `PolicyDiff` against the previous version.

The mapping function lives in the usecase layer (IP-010 / future);
the kernel only provides the diff function.

## Risk + mitigation

- **Risk**: editable rules become Turing-complete. **Mitigation**:
  `FormulaExpr` whitelist + depth cap + no recursion.
- **Risk**: rule drift across packs. **Mitigation**: `scope` is
  explicit; policies scoped to one pack do not apply outside.

## Out-of-scope

- Persistence — usecase layer.
- API exposure — usecase + api layers.
- UI editor — app layer.

## References

- ADR-0174 — chargeback formula.
- ADR-0083 — Tier-A invariants.
- ADR-0199 — cost-attribution canonical.

## Verification

- `cargo test -p oya-finops-portal-cost-allocation-policy-kernel`.
- `oya gate kernel-tier-invariants --crate
  oya-finops-portal-cost-allocation-policy-kernel`.
