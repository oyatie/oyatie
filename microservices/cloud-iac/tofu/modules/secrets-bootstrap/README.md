# `secrets-bootstrap` OpenTofu module

> ADR anchor: ADR-0202 (Tier B), ADR-0173.

OpenBao initial seed: unseal keys, root token, PKI mount.
Consumed by `k8s-namespace-bootstrap` for per-namespace SA
tokens.
