---
doc_class: ImplementationPlan
parent: ./INDEX.md
id: M03-P02-IP-002
title: Cloud Compute K8s + Functions API
status: complete (stable-api-entrypoints-green; app/transport/provider-adapter runtime not claimed)
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
final_shape_compliance: true
dependency_additions: []
purpose: Bring cloud.compute.k8s.cluster.create + cloud.compute.functions.invoke to stable.
---

# M03-P02-IP-002 — Cloud Compute K8s + Functions API

## Purpose
Bring cloud.compute.k8s.cluster.create + cloud.compute.functions.invoke to stable.

## Symbols-to-grit-claim
```
crates/oya-cloud-compute-k8s-api/src/lib.rs::create_cluster
crates/oya-cloud-compute-functions-api/src/lib.rs::invoke
```
(Scaffold-claim per ADR-0054 if any symbol is in a not-yet-existing crate.)

## Agent-prerequisites
Phase INDEX read; parent milestone INDEX read; MASTERPLAN §2 principles understood; M01-P08 ≥ P5 merged.

## Acceptance-test-commands
```
cargo test -p <owning-crate> --all-features
cargo run -p oya-foundry-fitness-cohesion -- <owning-crate-glob>
scripts/check.sh
```

## Done-criteria
- All acceptance-test commands return 0.
- Distroless image built (if IP ships a deployed binary); size < per-binary budget per `docs/standards/image-size-budgets.md`.
- No provider-specific deps outside adapter crates (Directive 4).
- All direct deps current LTS or have ADR-tracked exception (Directive 8).
- PR "good-taste audit" section non-empty (Directive 7).
- Distroless + provider-coupling + LTS-dependency lanes green on PR.

## Rollback-procedure
`grit done` is atomic per-symbol; if a subsequent IP regresses, revert the merge commit. For crates that other IPs depend on, follow per-crate split unwind per ADR-0015 §7.

## Next-IP-pointer
Next IP in this phase's INDEX list (or first IP of next phase if phase complete).

## Icm-store-payload
```
icm store -t context-oyatie -c 'M03-P02-IP-002 Cloud Compute K8s + Functions API shipped; acceptance commands green' -i high -k 'M03-P02-IP-002,complete'
```

## Progress
- 2026-05-21: Stabilized the planned `create_cluster` and `invoke` public entrypoints as thin delegates to the existing API-boundary functions. This removes plan-symbol drift without adding a second validation path.
- 2026-05-21: Targeted K8s/Functions API tests pass with planned entrypoint coverage (`k8s-api` 11/11, `functions-api` 10/10). App/transport wiring and provider adapter runtimes are not claimed by this IP.

## Decision-log (Linus good-taste row)
Special cases eliminated by this IP: plan-facing symbols now resolve to stable public wrappers while the detailed API-boundary functions keep the single validation/idempotency/error-mapping path.
Rejected: duplicating K8s or Functions validation logic in the stable wrappers; a second path would drift from the already-tested API boundary.
Rejected: claiming deployed app/transport/provider adapter runtime readiness from API crate tests; those remain follow-up runtime slices.
