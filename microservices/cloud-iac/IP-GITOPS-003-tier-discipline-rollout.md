# IP-GITOPS-003 — Tier discipline rollout

> ADR anchor: ADR-0202.
> Owner: `oya-cloud-iac`.
> Estimate: 3 days.

## Goal

Wire `oya-check-iac-tier-discipline` (this batch) into the
CI lane catalog. Phase 1 = advisory; Phase 2 = BLOCKER after
the migration window.

## Tasks

### 1. Lane registration

- Add lane entry to `registry/quality/lanes.yaml` (parent
  wires).
- Lane runs the discipline gate over `microservices/*/iac/`.

### 2. Phase 1 (advisory)

- Violations land as PR comments; no merge block.
- Owner team is paged on > 5 sustained violations.

### 3. Phase 2 (BLOCKER)

- T+90d: lane flips to BLOCKER.
- All PRs touching `iac/` must pass.

### 4. Tests

- Unit test (in `oya-check-iac-tier-discipline` crate, this
  batch) covers the four violation kinds.

## Acceptance criteria

- Lane runs on every PR touching `iac/`.
- T+90d flip happens without false positives.

## References

- ADR-0202.
- `crates/oya-check-iac-tier-discipline/`.
