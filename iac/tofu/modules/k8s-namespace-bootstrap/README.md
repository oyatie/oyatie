# `k8s-namespace-bootstrap` OpenTofu module

> ADR anchor: ADR-0202 (Tier B).

Per-µservice namespace + RBAC + NetworkPolicy seed + ArgoCD
AppProject bootstrap.

Note: this module CREATES the namespace + the AppProject. It
does NOT create per-pod manifests (those are Tier A / ArgoCD).
The discipline gate `oya-check-iac-tier-discipline` enforces.
