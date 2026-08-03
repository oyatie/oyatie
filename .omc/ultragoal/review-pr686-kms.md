# Security & Architecture Review — PR #686 (G002 slice 2: cloud-kms K8s operator)

**Reviewer:** Fable fresh-context reviewer of record (ultraqa rigor; Torvalds + hyperscaler lens)
**Pinned head:** 9aedb588444874af56db53ffcce20cba68534db7 (branch agent/g002-kms-operator)
**Base:** origin/dev
**Governing decision:** docs/decisions/ADR-0543-cloud-kms-operator-commissioning.md
**Risk Level:** LOW (no HIGH/CRITICAL finding; all findings MEDIUM and below)

---

## Scope & content-assert (verified, not asserted)

`git diff origin/dev..9aedb5884 --name-only` confines to: cloud/cloud-kms/** (3 operator crates + helm + SLO + runbook + domain lib lifecycle ports + OWNERS), docs/decisions/ADR-0543, evidence/multispectrum/g002-kms-operator-slice2-*.json, Cargo.lock, and 4 cloud-ci `*.generated.json` faces. The faces diffs are pure generated artifacts: decision-crosswalk adds ADR-0543 (in_spec:true, decision_count 362→363); scm-facts is commit-time SCM facts (new commit hashes). NO cross-lane source contamination. Scope is clean.

## Build & test truth (leader environment)

- `buck2 build //cloud/cloud-kms/...` → BUILD SUCCEEDED (exit 0). The kube/rustls adapter compiled cleanly; the documented FRIC-1781113000 cold-build constraint did not manifest in the leader env, as predicted.
- `buck2 test //cloud/cloud-kms/...` → Pass 14, Fail 0 (api/oci/openbao unittests; wildcard did not surface the operator rust_test targets in that pass).
- Explicit operator targets `buck2 test //...operator-kernel:..-tests //...operator-k8s-adapter:..-tests //...operator-app:..-tests` → Pass 3 targets, Fail 0. Adapter block shows 25/25 passing incl. fail-closed cases: partial_observed_state_fails_closed_without_side_effects, key_ring_status_missing_health_fails_closed, sealing_root_status_missing_observed_version_fails_closed, domain_actuator_fails_closed_for_quarantined_observed_state, unknown_health_state_is_invalid_crd_object. Kernel (20 fns) + app (11 fns) targets green.

Kernel + app + adapter all build and test green. Build truth satisfied.

## Summary

- Critical Issues: 0
- High Issues: 0
- Medium Issues: 2
- Low Issues: 3

---

## Mandate findings

### 1. Kernel purity (load-bearing claim) — HOLDS
- `oya-cloud-kms-operator-kernel/Cargo.toml`: sole dependency is `serde`. No kube-rs, k8s-openapi, tokio, or clock crate. `BUCK` deps = `third-party//:serde` only. Build-graph confirms purity, not just prose.
- `reconcile(observed, desired, clock) -> Vec<Action>` (src/lib.rs:207) is pure and clock-injected via `trait Clock`. `#![forbid(unsafe_code)]`. Domain types are CRD-shaped (KeyRing/SealingRoot/KeyVersion), zero kube primitives leak into kernel signatures.
- Litmus (would kernel interface change at cloud-k8s cutover?): NO. The kernel is the correctly-drawn cutover-stable seam.

### 2. Security (trust substrate) — CLEAN, no HIGH+
- RBAC (operator-rbac.yaml): namespaced `Role` (NOT ClusterRole), scoped to apiGroup `kms.oyatie.com` resources `kmskeyrings`,`kmssealingroots` (+ `/status`) with get/list/watch/patch/update, plus core `events` create/patch. No `secrets`, no wildcard verbs, no cluster scope. Least-privilege confirmed.
- mTLS: operator-app main.rs validates ca.crt/tls.crt/tls.key exist at startup → ExitCode::FAILURE if missing (fail-closed). Deployment mounts the mTLS secret read-only, `optional:false`; istio ambient dataplane label present.
- No static secrets: secrets scan over the slice returns zero credential literals (hits are Rust identifiers `SecretProvider`/`looks_like_serialized_token` [a defensive reject], `DataClass::Secret`, and k8s API field names `secretName`/`automountServiceAccountToken`). State path required via env (no in-memory fallback in prod — enforced + tested).
- Fail-closed on ambiguous observed state: kernel returns single QuarantineObservedState on non-Complete consistency; run_reconcile_cycle hard-errors before acting; adapter execute() refuses QuarantineObservedState. Three independent layers, all tested.
- cosign/digest posture: values.yaml digest "" pre-release is intended; operator-deployment.yaml FAILS helm render if cosign.required && digest is not a real non-zero sha256 (regexMatch gate). Pod hardening is exemplary: runAsNonRoot, seccompProfile RuntimeDefault, allowPrivilegeEscalation false, readOnlyRootFilesystem true, capabilities drop ALL, kata runtimeClass.

### 3. Correctness — SOUND
- Reconciler actions (CreateSealingRoot/CreateKeyRing/RotateKeyVersion/DemoteKeyVersionToDecryptOnly/QuarantineKeyRing/QuarantineObservedState) match desired-vs-observed logic; rotation gated on age >= rotate_after_seconds AND no newer non-destroyed version (idempotent); multi-active collapses to newest, older demoted to decrypt-only. Idempotency proven by applying_create_rotate_demote_and_quarantine_actions_is_idempotent (re-reconcile yields empty).
- One wide-event per cycle: emit_reconcile_wide_event called on every success and every failure branch; carries status/action_count/executed_count/error_class/convergence_seconds. Verified.
- CRD schemas match kernel/adapter types exactly: spec enums (origin/usage/hsmValidation/residency/dataClass) == parse_* fns; status.health.state enum == project_health; version states == parse_key_version_state. status subresource declared, matching RBAC /status grant and patch_status calls.

### 4. Universality/hermeticity — CLEAN
- No hardcoded cluster/repo/machine assumptions. Helm values parameterize registry, namespace, SA, mTLS paths, state PVC, replicas, resources. Env-driven paths in app (OYA_KMS_OPERATOR_*). The kernel/adapter/app split is a reusable clean-arch pattern (ADR-0510 boundary marker), not oyatie-special.

---

## Findings (ranked)

### [MEDIUM] M1 — Orchestration traits co-located in the transient adapter blur the cutover seam
**Category:** A04 Insecure Design (seam-drawing) / maintainability
**Location:** cloud/cloud-kms/crates/oya-cloud-kms-operator-k8s-adapter/src/lib.rs:215-343 (`ObservedStateProvider`, `KmsOperatorActuator`, `ExponentialBackoff`, `ReconcileCycleReport/Failure`, `ReconcileWideEvent`, `run_reconcile_cycle`, `status_patches_for_actions`); consumed by app lib at oya-cloud-kms-operator-app/src/lib.rs:13-16.
**Issue:** These orchestration types are kube-agnostic (their signatures reference only kernel types + AdapterError, zero kube/k8s-openapi). Yet they live in the crate ADR-0510 marks "transient." The app's core run loop (`OperatorApp::run_once` → `run_reconcile_cycle`) therefore imports its orchestration contract FROM the transient adapter. At cloud-k8s cutover, swapping the adapter would either (a) drag these stable types along with the discarded kube code, or (b) force them to be moved — interface churn the ADR's "kernel and app interfaces are unchanged" consequence does not fully cover.
**Exploitability/Blast radius:** None (not a security exploit). Cutover-friction / seam-purity concern only. The kernel itself (the primary load-bearing seam) is clean; this is the secondary orchestration layer.
**Remediation (Rust):** Extract the kube-agnostic orchestration traits + run_reconcile_cycle + report/backoff types into a pure `oya-cloud-kms-operator-core` (or fold into the kernel), leaving only kube-rs wiring (KubeOperatorRuntime, DynamicObject projection, patch_status) in the adapter. Then the app depends on kernel+core, and the adapter is genuinely replaceable in isolation.

### [MEDIUM] M2 — Observed-state read is hardcoded Complete; a server-paginated list could be silently partial
**Category:** A09 Logging/Integrity (observation integrity)
**Location:** project_observed_state (adapter src/lib.rs:193-197) hardcodes `read_consistency: ReadConsistency::Complete`; list_projected_objects (src/lib.rs:577-590) uses `api.list(&ListParams::default())` (single page, no continue-token loop).
**Issue:** If the API server paginates KmsKeyRing/KmsSealingRoot results (large tenant set / server-side default limit), only the first page is read but the state is still labeled Complete. The kernel then reconciles against a partial view.
**Why NOT High:** desired is derived from observed (`desired_state_for_observed` = `desired_state_from_observed`), so an unread object is simply absent from desired → the kernel takes NO action on it (it cannot create/rotate/quarantine an object it didn't see). Missing-object → no-op, fail-safe by construction; convergence is merely delayed until the next watch event/requeue. No destructive action on key material. Blast radius = delayed convergence, not key loss.
**Remediation (Rust):** Drain pagination explicitly (loop on `metadata.continue` / use `ListParams::default().limit(...)` with continuation) or assert a single-page invariant and set `ReadConsistency::Partial` when a continue token is present, letting the existing fail-closed path engage. Add a RED test feeding a continue-token list.

### [LOW] L1 — `sealing_root_create` sets created_at_epoch_seconds: 0
**Location:** adapter src/lib.rs:1999-2009 (`sealing_root_create`), contrast key_create_from_key_ring which threads the real timestamp.
**Issue:** New sealing roots are persisted with creation epoch 0 rather than the reconcile clock value. Cosmetic/provenance staleness in the state snapshot; no security or convergence impact (sealing-root reconcile keys on observed_version, not created_at).
**Remediation:** Thread the reconcile `now`/requested_at into CreateSealingRoot like the key-ring path.

### [LOW] L2 — Wide-event status field is the only live convergence signal; Prometheus histogram is target-only
**Location:** slos/kms-reconcile-convergence.openslo.yaml:17-19 (annotated exporter_gap); runbook line 23 documents it.
**Issue:** The SLO's thresholdMetric queries `oya_cloud_kms_operator_reconcile_convergence_seconds_bucket`, which is not exported in this slice. Honestly annotated (instrumentation_status: target-only) and the runbook routes operators to the structured wide-event instead — so this is disclosed, not hidden. Tracked as a follow-up, not a blocker.
**Remediation:** Land the OTel histogram exporter mapping in the next slice; until then the candid annotations are acceptable.

### [LOW] L3 — `automountServiceAccountToken: true`
**Location:** operator-rbac.yaml:14 / values.yaml:43.
**Issue:** The SA token is auto-mounted. Required — the operator uses the in-cluster kube client to watch CRDs and patch status, so the token is load-bearing. Flagged for completeness only; not reducible without breaking the operator. The token's blast radius is bounded by the least-privilege Role (M-finding-free RBAC). No change recommended.

---

## Security Checklist
- [x] No hardcoded secrets (slice secrets scan clean)
- [x] All CRD inputs validated (OpenAPI schema enums + tenantId regex at API-server boundary; adapter parse_* reject unknown values as InvalidCrdObject)
- [x] Injection prevention (no SQL/command/string-concat sinks; serde-typed projection; status patches built via serde_json::json!, no string interpolation into JSON)
- [x] Authentication/authorization verified (mTLS files fail-closed at startup; namespaced least-privilege Role; no broader grant)
- [x] Dependencies: no new third-party crates beyond workspace-pinned kube/k8s-openapi/tokio/serde/futures already governed; Cargo.lock refresh only. (Repo uses buck2 reindeer vendoring; no npm/pip/cargo-audit surface applies to this Rust slice — dependency posture is the vendored third-party graph, unchanged in shape.)
- [x] Fail-closed on ambiguous/partial/compromised observed state (3 layers, tested)
- [x] Pod hardening (nonRoot, seccomp, no-priv-esc, ro-rootfs, drop ALL caps, kata)
- [x] Supply chain: cosign-required + digest-gate fails render on missing/zero digest

---

## Verdict rationale
Kernel purity holds (serde-only, pure clock-injected reconcile, cutover-stable). RBAC is least-privilege (namespaced, kms.oyatie.com-only, no secrets/wildcards). No security finding at HIGH or above. buck2 kernel+app+adapter all build and test GREEN in the leader env (adapter compiled — cold-build constraint did not apply). Universality is clean. F3 (PDB maxUnavailable:1) is present and correct in operator-pdb.yaml + values.yaml, letting the single-replica operator drain instead of wedging. The two MEDIUM findings are seam-purity (M1) and observation-pagination (M2); both are non-exploitable, fail-safe-by-construction, and appropriate as fast-follow improvements rather than merge blockers.

VERDICT: APPROVE
