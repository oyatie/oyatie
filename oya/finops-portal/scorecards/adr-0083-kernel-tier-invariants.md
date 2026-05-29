---
scorecard_id: finops-portal/adr-0083-kernel-tier-invariants
authored: 2026-05-18
authority: ADR-0083 Tier-A 4-INV kernel-tier invariants
status: ready
---

# Scorecard — ADR-0083 Tier-A 4-INV kernel-tier invariants

ADR-0083 requires that every kernel-tier crate enforces 4
invariants:

- **INV-1**: No `std::io` import (kernel does not touch the world).
- **INV-2**: No `tokio` import (kernel is sync-pure).
- **INV-3**: All public wire-format types are `#[non_exhaustive]`.
- **INV-4**: Total order on types that downstream consumers sort
  (deterministic byte-stability).

## Kernel crates in this µservice

1. `oya-finops-portal-tenant-billing-presentation-kernel` (IP-001).
2. `oya-finops-portal-cost-allocation-policy-kernel` (IP-009).
3. `oya-finops-portal-anomaly-explanation-kernel` (IP-011).
4. `oya-finops-portal-focus-export-kernel` (IP-014).
5. `oya-finops-portal-credit-ledger-kernel` (IP-013).

## Compliance per crate

| Crate                                                | INV-1 | INV-2 | INV-3 | INV-4 | Evidence              |
|------------------------------------------------------|-------|-------|-------|-------|-----------------------|
| tenant-billing-presentation-kernel                   | ✓     | ✓     | ✓     | ✓     | IP-001 acceptance §5  |
| cost-allocation-policy-kernel                        | ✓     | ✓     | ✓     | ✓     | IP-009 acceptance §5  |
| anomaly-explanation-kernel                           | ✓     | ✓     | ✓     | ✓     | IP-011 acceptance §5  |
| focus-export-kernel                                  | ✓     | ✓     | ✓     | ✓     | IP-014 acceptance §5  |
| credit-ledger-kernel                                 | ✓     | ✓     | ✓     | ✓     | IP-013 acceptance §5  |

## CI gate

- `oya gate kernel-tier-invariants` is referenced in each kernel IP's
  Verification section.
- The gate scans Cargo.toml + src/lib.rs for forbidden imports +
  enforces `#[non_exhaustive]` via syn AST inspection.

## Gaps + remediation

- All 5 kernels are planned-ready; actual crate compilation occurs
  when the IPs are implemented (post-seed). Each IP includes
  `cargo build` + `oya gate kernel-tier-invariants` as a
  verification step.

## Verdict

**PASS (planned, ready)**. Verification flips to **PASS (verified)**
once the kernels are implemented.

## References

- ADR-0083 Tier-A 4-INV.
- IPs 001, 009, 011, 013, 014.
