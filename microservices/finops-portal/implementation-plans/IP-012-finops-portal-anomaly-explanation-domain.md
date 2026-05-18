---
ip_id: IP-012
ip_status: ready
slice_owner: ops-finops
authored: 2026-05-18
slice: finops-portal/anomaly-explanation/domain
related_adrs: [ADR-0131, ADR-0199]
depends_on: [IP-011]
target_lines: 150
---

# IP-012 — `anomaly-explanation` domain slice

## Why this slice

The domain layer for `anomaly-explanation` wraps the pure kernel
algorithm in the business rules that govern its application:

- Which dimensions to investigate first depends on the tenant's
  past anomaly profile (some tenants are GPU-heavy; some are
  storage-heavy).
- The **action recommendation** layer that maps `Explanation` to a
  set of next-steps (e.g. "open a ticket with the workflow team",
  "page customer-success", "no-op — known seasonal").
- The **suppression** logic for known-safe anomalies (a tenant
  that pre-announced a batch job is suppressed for the announced
  window).

This layer is still I/O-free; it consumes the kernel + a snapshot
of suppression rules.

## Acceptance criteria

1. New crate
   `crates/oya-finops-portal-anomaly-explanation-domain/` depends
   on the kernel from IP-011.
2. Public type `Recommendation`:
   ```rust
   pub enum Recommendation {
       PageCustomerSuccess { reason: String },
       OpenFoundryTicket { capability_id: String, reason: String },
       AwaitConfirmation { tenant_window: TimeRange },
       NoOpKnownSafe { suppression_rule: String },
   }
   ```
3. Public type `SuppressionRule`:
   - `tenant_id`, `window` (start, end), `dimension_subset`,
     `authored_by`, `authored_at`.
4. Public function `recommend`:
   ```rust
   pub fn recommend(
       explanation: &Explanation,
       suppressions: &[SuppressionRule],
       profile: &TenantAnomalyProfile,
   ) -> Recommendation;
   ```
5. ≥ 6 unit tests:
   - suppression match → `NoOpKnownSafe`.
   - GPU-heavy contribution + GPU-heavy profile → `OpenFoundryTicket`.
   - storage-heavy contribution + budget low → `PageCustomerSuccess`.
   - low-confidence explanation → `AwaitConfirmation`.
   - multiple suppressions: first-match wins by `authored_at`.
   - empty suppressions + low confidence → `AwaitConfirmation`.
6. `cargo test -p oya-finops-portal-anomaly-explanation-domain`
   green.

## File-level work plan

1. `Cargo.toml` — depends on kernel; `time`.
2. `src/lib.rs`.
3. `src/recommend.rs` — recommendation logic.
4. `src/suppress.rs` — suppression matching.
5. `src/profile.rs` — `TenantAnomalyProfile` type.
6. `src/error.rs`.

## Recommendation decision tree

```
if any suppression matches → NoOpKnownSafe
else if confidence < 0.5     → AwaitConfirmation
else if top contributor is `Capability`
   → OpenFoundryTicket { capability_id, reason }
else if top contributor is `WorkloadClass=gpu` and profile.gpu_heavy
   → OpenFoundryTicket { capability_id: "gpu-eval", reason }
else if top contributor is `CostCenter=storage` and budget headroom < 10%
   → PageCustomerSuccess { reason }
else
   → AwaitConfirmation { tenant_window: now..now+24h }
```

## Suppression-rule precedence

- Active suppressions are filtered by `now ∈ window`.
- Among active suppressions, the one with the **latest**
  `authored_at` wins (most recent override applies).
- A suppression with `dimension_subset` empty matches all
  explanations for the tenant; non-empty subset must intersect the
  `Explanation.top_contributions`.

## Risk + mitigation

- **Risk**: suppression-rule abuse hides real anomalies.
  **Mitigation**: every `NoOpKnownSafe` recommendation emits an
  audit-chain event `AnomalySuppressed` with the suppression rule
  id + the explanation hash so an auditor can review later.
- **Risk**: profile drift. **Mitigation**: profile is recomputed
  weekly by a separate batch job (not in scope here); domain only
  consumes it.

## Out-of-scope

- Persistence of suppressions — usecase.
- Profile computation — separate batch job µservice.

## References

- ADR-0199 — cost-attribution canonical.
- `slos/anomaly-explanation-latency.openslo.yaml`.

## Verification

- `cargo test -p oya-finops-portal-anomaly-explanation-domain`.
