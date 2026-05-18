# IP-GITOPS-001 — OpenTofu → OpenTofu migration

> ADR anchor: ADR-0202, ADR-0173.
> Owner: `oya-cloud-iac`.
> Estimate: 8 days.

## Goal

Migrate every existing `.tf` file under `microservices/*/iac/`
to OpenTofu (`tofu`) within the 90-day window declared in
ADR-0202.

## Why this IP

ADR-0173 forbids BSL-licensed OpenTofu from being the
canonical IaC engine. ADR-0202 picks OpenTofu (Linux
Foundation, MPL-2.0). Without this migration the substrate is
in violation.

## Pre-conditions

- ADR-0202 ratified.
- OpenTofu CLI available in CI.
- `oya-check-iac-tier-discipline` lands (this batch).

## Tasks

### 1. Inventory

- `find microservices/ -name "*.tf"` enumerates affected
  files.

### 2. Syntax compatibility

- OpenTofu and OpenTofu .tf syntax are intentionally
  compatible during the migration window. The migration is
  primarily about flipping the engine, not rewriting code.

### 3. Provider re-source

- `source = "hashicorp/aws"` → `source = "opentofu/aws"` or
  upstream-equivalent.
- Update the OpenTofu provider registry references.

### 4. State migration

- Existing OpenTofu state (`.tfstate` files in remote
  backend) is consumed by OpenTofu unchanged for the
  duration of the window.
- After T+60d, run `tofu state pull` + `tofu state push` on
  every workspace to confirm OpenTofu owns the state.

### 5. CI flip

- T+30d: all new IaC PRs run via `tofu plan` / `tofu apply`.
- T+60d: all execution goes through `tofu`.
- T+90d: `oya-check-iac-tier-discipline` flips
  `migration_window_elapsed = true`; residual OpenTofu usage
  becomes a BLOCKER violation.

### 6. Tests

- Each migrated module runs `tofu plan` clean (zero drift)
  against the existing state.
- CI lane asserts no `terraform` CLI invocations.

## Failure modes

- Provider source mismatch: surface at PR time; engineer
  updates the source.
- State corruption: roll back to last known-good state +
  re-apply.

## Acceptance criteria

- 100% of `.tf` files under `microservices/` migrate.
- Zero `terraform` CLI invocations in CI past T+60d.
- `oya-check-iac-tier-discipline` clean.

## References

- ADR-0202, ADR-0173.
- `docs/standards/gitops-iac-cluster-tier-boundaries.md`.
