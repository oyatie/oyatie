---
adr: ADR-0543
title: Commission the cloud-kms K8s operator (G002 slice 2)
status: Superseded
superseded_by: [ADR-0702]
date: 2026-06-10
deciders: founder (in-session sanction 2026-06-10), agent-leader
related:
  - ADR-0510-transitional-substrate-adapters
  - ADR-0131-per-microservice-flat-layout
  - ADR-0541-corpus-liveness-graph
---

> **HISTORICAL / NON-AUTHORITY (2026-08-06):** Not live law. Live source of truth is `docs/decisions/ADR-0700`…`ADR-0709` (see `_disposition/adr-redirect.v1.json`). Frontmatter `status` may still say Accepted for provenance; treat as archived.


# ADR-0543 — Commission the cloud-kms K8s operator (G002 slice 2)

## Status

Proposed (founder ratification pending; lane sanctioned by the founder K8s-native directive of
2026-06-09 and the G002 ultragoal story; delivered via PR #686).

## Context

G002 (trust substrate) slice 1 landed the KMS enclave one-way-door, crypto-shred, typed root
provenance, and zero-static-secrets leasing — but cloud-kms had NO operator: no CRDs, no
reconciliation, no GitOps actuation. The founder doctrine requires cloud-native, K8s-native
operation for every substrate (CRDs + operators + reconciliation + GitOps, zero imperative ops),
with ports modeling the owned cloud-k8s destination and transient kube-rs absorbed in an adapter
(ADR-0510). The founder structure directive (2026-06-10) further requires the crate layout to
encode the clean-architecture seam: kernel crates carry zero transient-tech dependencies.

## Decision

Ship the cloud-kms operator as three single-concern crates plus GitOps surfaces:

- `oya-cloud-kms-operator-kernel` — pure reconciler kernel (typed desired-state for KeyRings,
  SealingRoots, KeyVersion rotation; `reconcile(observed, desired) -> Vec<Action>`; injected
  clock; ZERO kube dependencies — the cutover-stable seam).
- `oya-cloud-kms-operator-k8s-adapter` — ADR-0510 transient adapter: kube-rs CRD watch loop
  wiring kernel Actions to the existing cloud-kms domain API; fail-closed on ambiguous observed
  state; backoff; one wide-event per reconcile cycle.
- `oya-cloud-kms-operator-app` — operator binary (distroless-compatible).
- GitOps surfaces under `cloud/cloud-kms/iac/k8s/helm/` (CRDs, deployment, RBAC, PDB, state PVC),
  convergence SLO, and a console-actionable runbook.

## Governed surfaces

`cloud/cloud-kms/OWNERS`
`secrets/core/kms-domain/src/lib.rs`
`cloud/cloud-kms/iac/k8s/helm/values.yaml`
`secrets/core/kms-operator-kernel/Cargo.toml`
`secrets/core/kms-operator-kernel/BUCK`
`secrets/core/kms-operator-kernel/src/lib.rs`
`secrets/core/kms-operator-kernel/tests/reconcile.rs`
`secrets/adapters/kms-operator-k8s/Cargo.toml`
`secrets/adapters/kms-operator-k8s/BUCK`
`secrets/adapters/kms-operator-k8s/src/lib.rs`
`secrets/adapters/kms-operator-k8s/tests/adapter.rs`
`secrets/facade/kms-operator-app/Cargo.toml`
`secrets/facade/kms-operator-app/BUCK`
`secrets/facade/kms-operator-app/src/lib.rs`
`secrets/facade/kms-operator-app/src/main.rs`
`secrets/facade/kms-operator-app/tests/app.rs`
`cloud/cloud-kms/iac/k8s/helm/crds/kmskeyrings.kms.oyatie.com.yaml`
`cloud/cloud-kms/iac/k8s/helm/crds/kmssealingroots.kms.oyatie.com.yaml`
`cloud/cloud-kms/iac/k8s/helm/templates/operator-deployment.yaml`
`cloud/cloud-kms/iac/k8s/helm/templates/operator-rbac.yaml`
`cloud/cloud-kms/iac/k8s/helm/templates/operator-pdb.yaml`
`cloud/cloud-kms/iac/k8s/helm/templates/operator-state-pvc.yaml`
`secrets/observability/slos/cloud-kms/kms-reconcile-convergence.openslo.yaml`
`cloud/cloud-kms/runbooks/operator-stuck-reconcile.md`
`evidence/multispectrum/g002-kms-operator-slice2-20260610-1781111229.json`

## Consequences

- Cutover litmus holds by construction: swapping kube-rs for the owned cloud-k8s substrate
  replaces the adapter crate only; kernel and app interfaces are unchanged.
- Known constraint at delivery: the kube/rustls adapter stack fails buck2 cold-builds under
  codex-spawned daemon environments (FRIC-1781113000); kernel + projection seams are
  buck2-verified, adapter compile coverage verified from leader-environment daemons.
- The operator's lane history (worker death at provider usage limit, leader salvage incl.
  reflog-recovered hardening commits) is recorded in the dispatch ledger.
