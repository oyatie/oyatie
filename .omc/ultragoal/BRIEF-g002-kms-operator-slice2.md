# Worker brief — G002 slice 2: cloud-kms K8s operator + GitOps actuation (one worker, one PR)

Goal context: ultragoal G002 (trust substrate) — slice 1 landed (enclave one-way-door, crypto-shred + attributable cancel, typed root provenance, zero-static-secrets lease; PRs #655/#658). The K8s-native operation depth is missing: NO operator exists for cloud-kms (verified: no kube dep under cloud/cloud-kms). Founder doctrine: cloud-native K8s-native operation — CRDs + operators + reconciliation + GitOps for everything, zero imperative ops. Ports model the OWNED destination (cloud-k8s), transient adapters absorb upstream kube-rs/k8s per ADR-0510.

Work ONLY in a worktree you create: `git -C /Users/jasonlee/Developer/oyatie fetch origin && git -C /Users/jasonlee/Developer/oyatie worktree add /Users/jasonlee/oyatie-worktrees/g002-kms-operator -b agent/g002-kms-operator origin/dev`. NEVER touch the main checkout working tree.

## Study FIRST (mirror conventions, do not invent)
- `oya/ci-controller/crates/oya-ci-controller-k8s-adapter` + `oya-ci-controller-app` — the repo's existing kube-rs operator/adapter split.
- `cloud/cloud-kms/crates/*` — domain, enclave-kernel, api, adapters; the operator actuates THROUGH the existing domain ports, never bypasses them.
- `cloud/cloud-kms/slos/*.openslo.yaml` + threat-models + runbooks dirs — extend, don't duplicate.
- A sibling service's iac/ + catalog wiring for GitOps manifest conventions.

## Deliverables (one PR)
1. **Reconciler kernel (pure, owned-shape)** `cloud/cloud-kms/crates/oya-cloud-kms-operator-kernel`: typed desired-state model (KeyRing, SealingRoot, KeyVersion rotation policy as CRD-shaped Rust structs), pure `reconcile(observed, desired) -> Vec<Action>` decision function with exhaustive unit tests (create/rotate/decrypt-only-demote/quarantine paths, idempotency: reconcile(reconcile(x)) emits no actions; injected clock trait — no system time in logic). The kernel must compile with ZERO kube/k8s deps — review question it must survive: "would this trait change at cloud-k8s cutover?" No unwrap/expect/panic; `#![forbid(unsafe_code)]`.
2. **Transient adapter** `cloud/cloud-kms/crates/oya-cloud-kms-operator-k8s-adapter` (kube-rs, ADR-0510-marked): CRD definitions + watch loop wiring the kernel's Actions to the existing cloud-kms domain API; fail-closed on ambiguous observed state (never act on partial reads); exponential backoff; one structured wide-event per reconcile cycle.
3. **Operator binary** `oya-cloud-kms-operator-app` wiring kernel+adapter; distroless-compatible (no shell-outs).
4. **GitOps manifests** under `cloud/cloud-kms/iac/`: CRDs + operator Deployment (mTLS cert volume per the existing workload-identity pattern, resource limits, PodDisruptionBudget) following sibling iac conventions.
5. **SLO**: add `cloud/cloud-kms/slos/kms-reconcile-convergence.openslo.yaml` (time-to-converge objective) following the existing OpenSLO files' schema exactly.
6. **Runbook**: `cloud/cloud-kms/runbooks/operator-stuck-reconcile.md` console-actionable (no CLI steps — console + API only per cli_surface_policy).
7. Contract/integration tests: kernel golden reconcile sequences; adapter tested against a fake ObservedState provider (no live cluster needed in CI); buck2 wiring for every new target (target-parity gate will fail closed otherwise).

## Rules
- buck2 build + buck2 test = the green signal; cargo supplementary only; lock refresh ONLY via `cargo metadata >/dev/null`.
- SETTLE PROTOCOL (mandatory): all content commits FIRST → `git add` everything → `infra/ci/materialize-cloud-ci-generated-faces.sh .` → FACES-ONLY settle commit LAST. Never hand-edit `*.generated.json`.
- COMMIT-EARLY discipline: commit compiling WIP after each deliverable lands green; never hold >30 min of work uncommitted (worker-death containment, FRIC-1781110000).
- MANDATORY pre-PR adversarial self-review: fresh `codex exec` with `/Users/jasonlee/Developer/oyatie/.omc/ultragoal/RUBRIC-torvalds-review.md` + your branch + this brief; fix all CRITICAL/HIGH; include verdict + findings-fixed in PR body. Leader reviews independently after.
- SSH-signed; push -u origin agent/g002-kms-operator; PR to dev citing G002 + ADR-0510 + the K8s-native founder directive. Final output line: `PR_OPENED: <number>` or `BLOCKED: <reason>`.
