# AUTH-005 Wave-3 — verified launch-ready plan (2026-06-24)

Source: analyst scope vs **origin/dev `b1c2c2a7`** (live-tree verified, drift-corrected). Gated on the keystone (de-commit `scm-facts`, #141) landing → then Group A first, B–E in parallel build / serial merge. Class-fix gate #818 (dto-authz-trust) is born-blocking, frozen baseline **73 instances**, shrink-only → each fix is gate-validated.

## Already-fixed — DO NOT re-chase
C1 #815 · C3 #816 · C5 #817 · C7/C8/C16/C24/G-C3/G-C4/accounting-idempotency #829 · C9/C10/C11/C15/C18/G-C1/G-C25/G-H25 #824 · CRM(G-C7) #835 · C26 gate · C27 deleted #814 · payroll-money #829 · completion.rs UnsafeCell merged.

## Still-open CRITICALs → groups
**ADR-ID CORRECTION (keystone PR #836 took ADR-0604; origin/dev max = 0603):** the per-group ids written below are OFF BY ONE and collide with the keystone — re-allocate **A=0605, B1=0606, B2=0607, B3=0608, B4=0609, B5=0610, B6=0611, B7=0612, B8=0613, B9=0614, C-cedar=0615, C-itsm=0616, D1=0617, D2=0618, D3=0619, D4=0620, D5=0621, D6=0622, E1=0623, E2=0624, F1=0625, F2=0626.** Re-derive from `max(ADR on origin/dev) + every in-flight branch` and pre-assign DISJOINT before authoring (#138). 0604 = keystone (on #836's branch only until it merges). **THIS banner is AUTHORITATIVE — the per-group ADR ids printed in the bullets below are STALE (off-by-one, collide with the keystone); IGNORE the inline ids and use the A=0605…F2=0626 allocation above.**
- **Group A — IAM keystone (FIRST, serialize):** C2 (`iam/facade/identity-workload-rest/src/lib.rs:926,1110` authorize forges principal from body) + C4 (`identity-workload-app/src/lib.rs:144` repo keyed by `WorkloadId` only → cross-tenant) — one PR, **ADR-0604**. LIVE.
- **Group B — independent caller-authz instances (parallel build / serial merge):**
  - B1 Workspace drive/chat/forms/meet (`oya/application/crates/oya-workspace-*-api`) — 0605
  - B2 Community post-store+social (`oya/community/crates/*`) — 0606
  - B3 Comms mail+messenger (`comms/facade/{mail-mailbox,messenger-stream}-rest`) +mail-DMARC — 0607
  - B4 Workflow exec-engine+event-bus+trigger+webhook-HMAC (`workflow/ports/*`) — 0608
  - B5 Compute vm+k8s+functions+quota (`compute/facade/*`) — 0609
  - B6 Ontology/data api+kernel+query-engine (`data/{ports,core}/*`) — 0610
  - B7 Cell regional-pack+region (`cell/ports/*`) — 0611
  - B8 HR infra-chain+domain-deny-state (`oya/hr/crates/*`) — 0612
  - B9 k8s control-plane-host+tenant-quota+cluster-lifecycle (`k8s/facade/*`, serialize within) — 0613
- **Group C — policy/template class-fixes (one change regenerated):** C-cedar 82× `*/iac/k8s/helm/templates/cedar.yaml` tenant-equality+action-allowlist (ENUMERATE still-wildcard first — don't revert hand-fixes) +tenant-quota-cedar-adapter — 0614 · C-itsm caller-side-eval + ops-dashboard deny-all — 0615.
- **Group D — substrate non-authz (fully parallel):** D1 kernel `wait4` cross-page + VFS `static mut` (`arch-{x86-64,aarch64}-adapter`) — 0616 · D2 cloud-os apid PDP(C20)+impersonation-os(C21) — 0617 · D3 outbox lease/visibility-timeout — 0618 · D4 CI FULL-tier test ratchet (`affected-set-app/src/main.rs:536`) — 0619 · D5 SSRF allowlist (oci+selfhosted+intelligence) — 0620 · D6 intelligence CLI→in-process-HTTPS (C17, ties #90) — 0621.
- **Group E — class-fix-gate-then-instances:** E1 checked-money type+gate → tax-api+accounting-journal (#134) — 0622 · E2 shared idempotency kernel+gate → billing/metering/workflow/moderation — 0623.
- **Group F — DESIGN-first (NOT code fan-out):** F1 C6 HSM/Shamir sealing root (founder call) — 0624 · F2 payments placeholder implement-vs-de-claim — 0625.

## Open questions (need resolution before the relevant group)
1. C6 sealing root: HSM/PKCS#11 non-exportable vs Shamir M-of-N? (blocks F1; rotate-vs-shred existing exportable roots) — **design pass launched 2026-06-24**.
2. C25 payments: implement charge handlers now vs de-claim prod Helm/SLO until implemented?
3. Cedar 82-template: regenerate-all vs enumerate-still-wildcard-first (avoid reverting a hand-fixed chart e.g. cloud-iam)?
4. C17/D6 intelligence CLI: this wave vs defer to #90 (avoid double-work)?
5. Confirm branch-protection required-contexts == only `oya-ci-required` (closes C27 residual).

## Verified gaps (need deeper trace before scoping)
Workflow durability/idempotency HIGHs (exec-engine-usecase commit-ordering) · Cedar 82-file content-uniformity unverified · reconcile frozen-73 baseline keys vs open-CRITICAL list · EDGE-DORMANT composition-root confirmation · SSRF intelligence constructor visibility.
