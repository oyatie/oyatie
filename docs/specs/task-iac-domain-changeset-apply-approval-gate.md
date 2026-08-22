# Spec: iac-domain-changeset-apply-approval-gate

## Objective

Introduce a pure, deterministic apply-approval gating kernel inside
`cloud-iac-domain` that classifies a `PlanChangeset` into one of three
approval tiers before any `tofu apply` execution.

## Crate boundary

Crate: `cloud-iac-domain` (flat / single-file: `src/lib.rs`).
No new workspace member. No external crate dependencies.

## Contracts

Pure Rust domain function. No I/O, no clocks, no randomness, no HashMap.
Callable from any upstream adapter (REST handler, gRPC service, async runtime)
without blocking.

## New public surface

### `ApplyApprovalVerdict`

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplyApprovalVerdict {
    /// No human review required — proceed automatically.
    AutoApprove,
    /// At least one human reviewer must approve before apply.
    RequiresReview { required_approvals: u32 },
    /// Apply is explicitly blocked regardless of approvals (policy hook).
    Blocked,
}
```

### `PlanChangeset::approval_gate`

```rust
impl PlanChangeset {
    /// Pure tiered apply-approval gate derived from the changeset summary.
    ///
    /// # Verdict rules (evaluated in order)
    ///
    /// 1. `Blocked` — reserved for future policy hooks; never returned by this
    ///    implementation.
    /// 2. `AutoApprove` — all entries are `NoOp` (or the changeset is empty).
    /// 3. `AutoApprove` — only `Create`/`Update` entries AND
    ///    `create_count + update_count < NON_DESTRUCTIVE_THRESHOLD`.
    /// 4. `RequiresReview { required_approvals: 1 }` — only `Create`/`Update`
    ///    AND `create_count + update_count >= NON_DESTRUCTIVE_THRESHOLD`.
    /// 5. `RequiresReview { required_approvals }` — any `Delete` or `Replace`
    ///    present; `required_approvals` scales monotonically on
    ///    `delete_count + replace_count`:
    ///    - 1–5  → 1
    ///    - 6–20 → 2
    ///    - 21+  → 3
    pub fn approval_gate(&self) -> ApplyApprovalVerdict
}
```

## Mod layout (flat clean-arch)

All code lives in `src/lib.rs`.  No new modules or files.

## Testing strategy

Unit tests only, hermetic, no I/O.  Written in-file under `#[cfg(test)]`.

Coverage:
- (a) empty changeset → `AutoApprove`
- (b) no-op-only changeset → `AutoApprove`
- (c) creates-only under threshold → `AutoApprove`
- (d) creates+updates at/over threshold → `RequiresReview { required_approvals: 1 }`
- (e) single delete → `RequiresReview { required_approvals: 1 }`
- (f) 6 destructive → `RequiresReview { required_approvals: 2 }`
- (g) 21 destructive → `RequiresReview { required_approvals: 3 }`
- (h) mixed delete+replace blast radius → higher required_approvals (monotonic)
- (i) determinism: same input called twice → identical verdict

## Observability / SLO

`cloud-iac-domain` is a pure domain crate with no SLO authoring required
(no network/HTTP surface). Upstream adapters are responsible for emitting OTel
spans and recording the `ApplyApprovalVerdict` as a span attribute.

## Constraints

- Does NOT alter `compute_iac_plan_diff`, `summarize`, or `has_destructive_changes`.
- `NON_DESTRUCTIVE_THRESHOLD = 50` (non-destructive create+update count).
- All thresholds are named constants in `lib.rs`.
