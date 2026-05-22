# Wave 15-ZH-v2 Category C empty-substance cleanup plan

Scope guard: candidates must be literal empty-substance `TBD`, `TODO`, `placeholder`, or `scaffold` files after verification.

## Delete files (0)


## Explicit keeps / exclusions (2)

- `microservices/cloud-iac/tofu/modules/kms/README.md` — KEEP: small file has substantive content: # `kms` OpenTofu module > ADR anchor: ADR-0202 (Tier B). Per-tenant per-region KMS keys with annual rotation by default.
- `microservices/cloud-iac/tofu/modules/vpc/README.md` — KEEP: small file has substantive content: # `vpc` OpenTofu module > ADR anchor: ADR-0202 (Tier B). Per-region VPC + subnets + routing. Consumed by `k8s-namespace-bootstrap` downstream. See `main.tofu` f
