# IP-GITOPS-006 — Secret bootstrap (Tier B)

> ADR anchor: ADR-0202, ADR-0173.
> Owner: `oya-cloud-iac`.
> Estimate: 3 days.

## Goal

Bootstrap OpenBao initial seed via the `secrets-bootstrap`
OpenTofu module. OpenBao is the canonical secret-storage
substrate per ADR-0173.

## Tasks

### 1. Module surface

- `secrets-bootstrap` module declares:
  - OpenBao initialization (unseal keys, root token).
  - Root token rotation policy.
  - PKI mount creation for cluster-internal TLS.

### 2. Composition

- The module's outputs (root-token reference) feed
  `k8s-namespace-bootstrap` so per-namespace service
  accounts can request short-lived tokens.

### 3. Tests

- Module applies clean against sandbox.
- Root token rotation flow runs end-to-end.

## Acceptance criteria

- OpenBao bootstrap is fully OpenTofu-driven; no manual
  steps.

## References

- ADR-0202, ADR-0173.
