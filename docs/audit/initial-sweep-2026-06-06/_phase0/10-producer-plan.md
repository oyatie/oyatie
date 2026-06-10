# Phase-0 Producer Plan — the LIVE `oya-ci-required` producer

Lane: concrete build plan for the component that posts `oya-ci-required` as a
commit-status / required-context on the candidate SHA, sourced from trusted
controller/trunk state. READ-ONLY audit; no files edited. All paths absolute.
Distinguishes WHAT EXISTS (read from real code) from WHAT TO BUILD.

---

## 0. TL;DR

- A near-complete, idiomatic Rust producer **already exists**: the
  `oya-ci-controller` (bespoke-Prow plank+crier) with a pure kernel
  state-machine, a K8s job-spawner adapter, and a **Forgejo** commit-status
  poster. Its kernel `GATE_CONTEXT` is hard-coded to `"oya-ci-required"`, it
  posts only on the full 40-hex candidate SHA, and the gate Job is
  trunk-sourced (clone `dev`, fetch PR ref as untrusted data, snapshot trusted
  target inventories before candidate checkout). The trust model the founder
  asked for is built and unit-tested.
- It is **NOT a live producer of the enforced gate**, for one decisive reason:
  the **forge-of-record mismatch**. Live branch protection is applied to
  **GitHub** `jason931225/oyatie` (`scripts/branch-protection-apply.sh`,
  baseline `source: gh api repos/jason931225/oyatie/...`), but the only wired
  controller poster targets **Forgejo** (`forgejo.oya-forge.svc.cluster.local`).
  Nothing posts `oya-ci-required` to the GitHub candidate SHA.
- Live GitHub `dev` protection currently requires
  `cargo-fmt, cargo-check, cargo-clippy, cargo-nextest, oya-pr-review` — none of
  which is `oya-ci-required`, and one of which (`oya-pr-review`) is a 501 stub.
  So today's enforcement is the façade the audit found, AND it is wired to the
  wrong producer family.
- WHAT TO BUILD is therefore narrow and mostly **wiring + one adapter**, not a
  green-field service: (a) a GitHub commit-status poster behind the existing
  `ForgejoStatusPoster` seam (or a generic `CommitStatusPoster` seam), (b)
  decide+enforce ONE forge of record, (c) flip live branch protection to
  require exactly `oya-ci-required`, (d) deploy the controller for real, (e)
  a go-live RED/GREEN proof against real PRs.

---

## 1. The producer chain (what posts the gate)

The intended live chain (all components below exist as code):

```
Forge webhook (PR opened/sync)
  └─> ci-webhook-gateway  (verify sig → route → dispatch)
        └─> ControllerDispatcher → POST /gate-run {pr_number, head_sha, base_ref:"dev"}
              └─> oya-ci-controller  POST /gate-run handler
                    └─> K8sJobSpawner.spawn(build_gate_job)  [plank role]
                          └─> batch/v1 Job  (trunk-sourced gate: buck2 build+test)
                    └─> kube-rs Controller watches the Job  [reconcile loop]
                          └─> map_job_to_status(observation) → ReconcileDecision  [pure kernel]
                          └─> ForgejoStatusPoster.post(sha, state, "oya-ci-required", desc)  [crier role]
                                └─> POST /api/v1/repos/<owner>/<repo>/statuses/<sha>
```

This is a faithful Prow decomposition: gateway≈hook, `/gate-run`+JobSpawner≈
plank, the Job≈prowjob pod, the reconcile loop+poster≈crier, TTL GC≈sinker.
`oya-ci-tide` (separate crate) is the Tide merge-pool equivalent and reads the
same `oya-ci-required` context as its merge predicate.

---

## 2. EXISTS vs BUILD — component-by-component

### 2.1 Producer crate (the controller) — EXISTS (strong)

Real crates under `/Users/jasonlee/Developer/source/oya/ci-controller/crates/`:

- `oya-ci-controller-kernel` — pure domain. No I/O, `#![forbid(unsafe_code)]`.
  - `GATE_CONTEXT = "oya-ci-required"`
    (`.../oya-ci-controller-kernel/src/lib.rs:471`). Every terminal/pending
    decision posts with exactly this context.
  - `map_job_to_status(obs, grace_cycles) -> ReconcileDecision` (line 484): the
    TOTAL pure state machine — success/failure/error/pending across Job
    conditions, OOMKilled, Evicted, InvalidImageName (terminal-now),
    ImagePull/CreateContainer (bounded grace), GC-race fail-closed. Fully
    unit-tested (lines 643–985).
  - `ForgejoStatusPoster` + `JobSpawner` trait seams (lines 620, 634) — the I/O
    boundary. NOTE: the post seam is **Forgejo-typed**
    (`fn post(sha, ForgejoState, context, description, target_url)`).
  - It ALSO already contains a Phase-0 **policy evaluator**
    (`evaluate_phase0_ci_policy`, line 285) + fixture-corpus tests (lines
    987–end) that mechanically enforce: required context present, trusted
    producer kind/controller, candidate-bytes-untrusted, trusted gate-definition
    source, override evidence, tenant separation. This is the RED/GREEN policy
    engine and it runs in `cargo test`.

- `oya-ci-controller-app` — kube-rs runtime + axum server.
  - `reconcile()` (`.../oya-ci-controller-app/src/lib.rs:117`): pure-core
    sandwich (observe → kernel decide → act); posts via `spawn_blocking` (the
    blocking-reqwest fix); write-once terminal annotation guard for
    exactly-once posting; fail-closed requeue on poster error.
  - `POST /gate-run` handler (line 468): **rejects non-40-hex SHAs** (line 476)
    and non-`dev` base_ref (line 488) — so the gate evidence binds to the exact
    candidate commit. Idempotent on deterministic Job name.
  - `/healthz`, `/metrics` (lines 445–456).

- `oya-ci-controller-k8s-adapter` — `JobSpawner` + `observe_job`.
  - `build_gate_job` (`.../oya-ci-controller-k8s-adapter/src/lib.rs:91`)
    encodes the TRUST MODEL in the Job command (lines 108–123):
    clone `dev` → `git fetch refs/pull/N/head` as data → snapshot
    `buck2 targets //...` + test targets to trusted files → verify the fetched
    PR ref `git rev-parse` == the exact candidate `{sha}` → only then
    `git checkout --detach {sha}` → run candidate against the trusted target
    lists. `backoffLimit:0` (fail-closed). The runner SA is low-privilege,
    `automountServiceAccountToken:false`, and **FORGEJO_CI_TOKEN is deliberately
    NOT injected** into the runner (lines 167–175): untrusted PR code cannot
    exfiltrate the status-posting credential. Tested at lines 462–520.

- `oya-ci-controller-forgejo-adapter` — `ForgejoCommitStatusPoster` (the live
  poster). `POST <base>/api/v1/repos/<owner>/<repo>/statuses/<sha>` with
  `Authorization: token <FORGEJO_CI_TOKEN>`, accepts 200/201
  (`.../oya-ci-controller-forgejo-adapter/src/lib.rs:72–122`).

- Binary `oya-ci-controller.rs` wires it all from env
  (`.../oya-ci-controller-app/src/bin/oya-ci-controller.rs`): `FORGEJO_CI_TOKEN`
  required; Forgejo base/owner/repo default to in-cluster
  `forgejo.oya-forge.svc.cluster.local:3000 / oya-admin / oyatie`.

**Verdict: the producer crate EXISTS and is the right shape.** What it produces
today is a Forgejo status, not a GitHub status.

### 2.2 The post path (where the status lands) — EXISTS but to the WRONG FORGE

- Controller poster target = **Forgejo** in-cluster.
- Live branch protection target = **GitHub** `jason931225/oyatie`:
  - `/Users/jasonlee/Developer/source/scripts/branch-protection-apply.sh`
    PATCHes `repos/${repo}/branches/${branch}/protection/required_status_checks`
    via `gh api`, default `repo=jason931225/oyatie`.
  - Baseline records live state via
    `gh api repos/jason931225/oyatie/branches/dev/protection`
    (`/Users/jasonlee/Developer/source/specs/phase0-ci-enforcement-baseline.json:82`).

So even if the controller runs perfectly, the `oya-ci-required` it posts goes to
Forgejo where **no branch protection consumes it**, while GitHub — where the
required-context check actually lives — never receives `oya-ci-required` from
any producer. This is the load-bearing gap.

### 2.3 A GitHub poster — EXISTS, but unwired for this context

There is a second, parallel gateway codebase
(`/Users/jasonlee/Developer/source/oya/ci-webhook-gateway/crates/...`, DDD
style) with a real `GitHubStatusPoster`
(`.../oya-ci-webhook-gateway-github-adapter/src/lib.rs`):
`POST https://api.github.com/repos/<owner>/<repo>/statuses/<sha>`,
`Authorization: Bearer`, `X-GitHub-Api-Version: 2022-11-28`. Tested
(`tests/d5_github_status_poster.rs` via httpmock).

BUT its `CommitStatusContext` vocabulary is `cargo-fmt | cargo-check |
cargo-clippy | cargo-nextest | oya-pr-review`
(`tests/d5_commit_status_post.rs:22–28`) — it has **no `oya-ci-required`
variant** — and it is wired into a *different* binary
(`.../oya-ci-webhook-gateway-app/src/bin/ci-webhook-gateway.rs:63`) that posts
gateway-internal statuses, NOT the controller's gate verdict. It is also not
trunk-sourced: it would let the gateway report a status the trusted runner did
not produce.

**So a GitHub-capable poster exists, but (a) it speaks the wrong context set and
(b) it is in the wrong process.** The reusable asset is the HTTP shape, not the
wiring.

### 2.4 The webhook → /gate-run dispatcher — EXISTS

Top-level gateway `/Users/jasonlee/Developer/source/oya/ci-webhook-gateway/src/`:
- `ControllerDispatcher` (`src/dispatch.rs:222`) POSTs the exact `/gate-run`
  body `{pr_number, head_sha, base_ref:"dev"}` and returns a typed transport
  error if `OYA_CI_CONTROLLER_URL` is unset (no silent success). Tested
  (`dispatch.rs:600–709`).
- Selected by `OYA_CI_DISPATCHER=controller` (`src/config.rs:20,87`); **default
  is `jenkins`** (`config.rs:90`, `from_str` line 46). So the controller path is
  opt-in and is NOT the default — a deploy that doesn't set the env var never
  reaches the controller.
- Receiver (`src/receiver.rs`) verifies HMAC/ed25519 on the raw body first
  (fail-closed) then dispatches. Webhook path `/webhook/forgejo`.

**Verdict: the kick path EXISTS** and binds head_sha through to the controller.
It is Forgejo-webhook-shaped and defaults to Jenkins.

### 2.5 The gate logic the runner executes — EXISTS (and is correctly NOT the producer)

`/Users/jasonlee/Developer/source/infra/ci/buck2-affected-gate.sh` is a robust,
fail-closed affected-only buck2 gate (uquery owner/rdeps, fails closed on buck2
errors, FATAL when a `.rs` change maps to no target). It is sound CI logic, but
the design **explicitly forbids it from being the `oya-ci-required` producer**:
fixture `tc-0.0.1-bad-buck2-affected-only-producer.json` expects verdict RED
with violation `untrusted_or_legacy_status_producer`, and the kernel's gate
command asserts `!command.contains("buck2-affected-gate.sh")` (k8s-adapter test
line 511). The required context is the **trunk-sourced full matrix**
(`buck2 build`/`test` over trusted target inventories), with the affected-only
script remaining feedback-only. This distinction is already enforced in code.

### 2.6 Branch-protection apply + verify — EXISTS, but is a check-only bridge

`scripts/branch-protection-apply.sh`: `--check` reads live contexts and diffs
against `infra/branch-protection/dev.json` (which lists exactly
`["oya-ci-required"]`); `--apply` PATCHes GitHub then re-checks. It even refuses
empty contexts (line 96). BUT both `dev.json` and `.github/branch-protection.yaml`
**self-disclaim** as "shadow/target … not Phase-0 exit authority until a trusted
cloud-ci/oya-ci producer is live and applied" (`dev.json:2,25`;
`branch-protection.yaml:1–5`). And the verify step shells to
`cargo run -p oya-dev-cli -- gate validate protection-context-match`
(line 129) — i.e. the legacy oya CLI, which the doctrine bars from being merge
authority. So the apply tooling exists but has never been flipped to live, and
its own verifier rides the legacy CLI.

---

## 3. The minimal-viable producer spec (WHAT TO BUILD)

The goal: ONE trusted process that posts `oya-ci-required` (pending→terminal) to
the candidate SHA on the **forge that branch protection actually reads**,
sourced from trunk/controller state, provable by RED/GREEN.

### 3.1 Decide the forge of record (BLOCKING design decision)

Two coherent end-states; pick one and make every artifact agree:

- **Option A — GitHub is the gate forge.** Branch protection stays on GitHub
  `jason931225/oyatie`. BUILD: a `GitHubCommitStatusPoster` implementing the
  controller's post seam (see 3.2), inject a GitHub token into the controller
  (crier only, never the runner), and point the controller at the PR's GitHub
  SHA. Webhook source can remain GitHub (the gateway already reads
  `X-GitHub-Event`).
- **Option B — Forgejo is the gate forge.** Keep the existing
  `ForgejoCommitStatusPoster` as-is and move branch protection to Forgejo's
  branch-protection API (status-check enforcement on `oya-admin/oyatie`),
  retiring the GitHub `cargo-*`/`oya-pr-review` required set. BUILD: a Forgejo
  branch-protection apply/verify equivalent of `scripts/branch-protection-apply.sh`.

Recommendation: **Option A** is the smaller, lower-risk delta because live
enforcement, `gh`-based tooling, fixtures, and the baseline already assume
GitHub; only a poster adapter + token wiring is net-new. Option B requires
re-homing all branch-protection enforcement and the audit's "live GitHub"
evidence model. Either way, the spec below is forge-parametric.

### 3.2 Generalize the post seam (small, additive)

The kernel seam is currently `ForgejoStatusPoster` with a `ForgejoState` arg
(`oya-ci-controller-kernel/src/lib.rs:620`). Introduce a forge-neutral
`CommitStatusPoster` seam (same signature, generic `state: GateState` mapped to
each forge's vocabulary in the adapter). Then:
- `ForgejoCommitStatusPoster` (exists) implements it unchanged.
- `GitHubCommitStatusPoster` (BUILD) implements it by reusing the proven HTTP
  shape from `oya-ci-webhook-gateway-github-adapter/src/lib.rs` (Bearer auth,
  `X-GitHub-Api-Version`, `POST /repos/<owner>/<repo>/statuses/<sha>`), but
  posting context `"oya-ci-required"` (not `cargo-*`).
This keeps the pure kernel and the entire reconcile loop unchanged — only the
adapter and the binary's poster construction differ.

### 3.3 Producer contract (must satisfy the GREEN fixture)

The producer the policy engine accepts is fixed by
`tc-0.0-good-cloud-ci-required-and-isolated.json`:
- `context = "oya-ci-required"`, `status_posts_on = "candidate_sha"`.
- `controller = "oya-ci-controller"`, `kind = "minimal_rust_bridge_adapter"`
  (or `"oya-ci-controller"` — both accepted by `evaluate_phase0_ci_policy`,
  kernel lines 306–317).
- `candidate_bytes_policy = "untrusted_input_only"`.
- `gate_definition_source = "trusted_dev_or_controller_state"`.
- `full_gate_matrix = "required_backstop_not_affected_only"`.
- `forbidden_authority`: `oya verify`, `oya gate run-all`, `buck2-affected-gate.sh
  only`.
The controller already stamps these as Job annotations
(`k8s-adapter src/lib.rs:140–161`: `ANNOT_CI_PRODUCER_KIND`,
`ANNOT_CI_GATE_DEFINITION_SOURCE`, etc.) — so the producer self-attests in a
machine-checkable way. **What to add:** make these annotations the SOURCE that
`evaluate_phase0_ci_policy` reads from the LIVE Job at deploy time (today the
policy test reads fixtures only).

### 3.4 Trust model (the security spine)

PR-uncontrollable by construction:
1. **Gate command is trunk-sourced.** Job clones `dev`, fetches the PR ref as
   *data*, snapshots trusted build/test target inventories from `dev` BEFORE
   the candidate is checked out, and runs the candidate against those immutable
   lists (k8s-adapter lines 108–123). A PR cannot delete a target to shrink the
   gate, change the context string, or weaken the producer — those bytes are
   never trusted. (Enforced by fixtures `tc-0.0.1a-bad-candidate-mutable-producer`
   and `...-candidate-deletes-trusted-target`.)
2. **Credential isolation.** The status token lives ONLY in the controller
   (crier); it is never injected into the runner Pod
   (k8s-adapter lines 167–175). A malicious PR cannot post its own
   `oya-ci-required=success`.
3. **SHA binding.** Status posts only on the full 40-hex candidate SHA;
   `/gate-run` rejects short/non-hex SHAs (app lines 464–483).
4. **Fail-closed.** `backoffLimit:0`; GC-race → Failure; poster failure →
   requeue, verdict durable on the Job (kernel lines 484–496, app 256–268).
5. **No legacy-CLI authority.** Context must not delegate to `oya …` or the
   affected-only script (k8s-adapter test line 511; policy violation
   `untrusted_or_legacy_status_producer`).

WHAT TO BUILD here is nothing in logic — it is to (a) keep the runner-token
isolation when adding the GitHub token (inject GitHub crier token to the
controller Deployment env / ExternalSecret, NEVER the gate-runner SA), and
(b) ensure the chosen forge's status API call is itself trunk-sourced (the
controller, not the gateway, posts the verdict).

### 3.5 Deployment (mostly EXISTS; flip to live)

Helm chart exists at
`/Users/jasonlee/Developer/source/oya/ci-controller/iac/k8s/helm/` with
Deployment, controller SA (`automountToken:true`), gate-runner SA
(`automountToken:false`), NetworkPolicy (egress to Forgejo:3000, apiserver:6443,
OpenBao, OTel), and an ExternalSecret for `forgejo-ci-token`
(`values.yaml:85–98`). **What to build/change:**
- For Option A: add a GitHub egress rule + a `github-ci-token` ExternalSecret;
  set `OYA_CI_DISPATCHER=controller` + `OYA_CI_CONTROLLER_URL` on the gateway
  Deployment so the controller path is actually taken (today default = jenkins).
- Every IaC template currently carries a `nonClaim:
  static-…-template-only-no-live-…` marker (values.yaml:25,32,36,91) — i.e. the
  chart is admitted as a template, not proven deployed. Go-live must replace
  those nonClaims with real deploy + RBAC + ESO-sync evidence.

### 3.6 Branch-protection flip (the act that makes enforcement REAL)

Run `scripts/branch-protection-apply.sh --apply` (Option A) so live GitHub `dev`
required contexts become exactly `["oya-ci-required"]`, replacing the current
`cargo-*` + `oya-pr-review` set. WHAT TO BUILD: replace the verify step's
`oya-dev-cli` dependency (line 129) with the bespoke policy engine
(`evaluate_phase0_ci_policy`) so the verifier itself carries no legacy-CLI
authority; and remove the self-disclaimer once the producer is live.

---

## 4. Go-live RED/GREEN proof (the robustness bar)

The founder's bar: every gate proven by a RED fixture (known-bad it MUST fail) +
GREEN fixture (known-good it passes) + proof it runs in CI and BLOCKS. Two
layers:

### 4.1 Unit/policy layer — EXISTS, runs in `cargo test`

- GREEN: `tc-0.0-good-cloud-ci-required-and-isolated.json` → verdict GREEN, no
  violations.
- RED corpus (11 fixtures, `phase0-ci-enforcement-baseline.json:130–142`),
  e.g. `tc-0.0.1-bad-missing-required-context`,
  `...-bad-buck2-affected-only-producer` (→ `untrusted_or_legacy_status_producer`),
  `...-bad-legacy-oya-cli-authority`, `...-bad-oya-gate-run-all-required-producer`,
  `...1a-bad-candidate-mutable-producer`, `...1a-bad-candidate-deletes-trusted-target`,
  `...0.2-bad-override-without-ttl-audit`, `...0.3-bad-cross-tenant-shared-cache`.
  All asserted by `phase0_fixture_corpus_executes_red_green_policy`
  (kernel lines 1441–1503), which REQUIRES at least one GREEN and one RED.
- The reconcile state machine has full success/failure/error/pending coverage
  (kernel lines 643–985). **This layer is already a real RED/GREEN gate.**

### 4.2 Live go-live layer — TO BUILD (this is the actual "blocks a merge" proof)

End-to-end against the real forge + real branch protection:

1. **GREEN PR.** Open a trivially-correct PR to `dev`. Webhook → gateway →
   `/gate-run` → Job builds+tests trusted targets → controller posts
   `oya-ci-required=success` on the GitHub HEAD SHA. Assert via
   `gh api repos/<owner>/<repo>/commits/<sha>/statuses` that
   `context==oya-ci-required && state==success`, AND that the PR is
   **mergeable** (branch protection satisfied). `oya-ci-tide`'s `is_mergeable`
   predicate (tide kernel `src/lib.rs:358`) must also return `Ok(())`.
   NOTE: tide's default `required_status_context` is currently
   `"oya-ci-gate"` (tide kernel line 76) — for go-live it MUST be set to
   `"oya-ci-required"` (env `OYA_TIDE_REQUIRED_STATUS_CONTEXT`) so the merge
   predicate reads the same context the controller posts. This mismatch is a
   real go-live bug to fix.

2. **RED PR.** Open a PR whose trusted-target build/test FAILS (e.g. a failing
   test in an affected crate, or a removed trusted target). Controller posts
   `oya-ci-required=failure` on the candidate SHA. Assert the status is
   `failure` AND that GitHub **refuses the merge** (required check not green).
   This is the proof the gate BLOCKS — not just reports.

3. **Tamper PR (trust-model proof).** Open a PR that edits
   `infra/ci/buck2-affected-gate.sh` to `exit 0`, or deletes a trusted target,
   or tries to set its own status. Because the gate is trunk-sourced and the
   runner has no token, the candidate cannot weaken the gate: the verdict must
   still be `failure`/blocked. (This mirrors `tc-0.0.1a-bad-candidate-mutable-producer`
   in live form.)

4. **CI-runs-and-blocks proof.** A presubmit/check that runs
   `branch-protection-apply.sh --check` and the kernel policy test on every PR,
   plus a recorded `gh api …/protection/required_status_checks` showing
   `contexts == ["oya-ci-required"]`. Until step 1–3 produce real status
   receipts on real SHAs, the exit-gate fixture
   `tc-0.12-current-red-p0-0-live-context-missing.json` stays RED
   (`p0_0_green:false`, `AC-0.0_green:false`) — which is the honest current
   state.

---

## 5. Exact exists-vs-build ledger

| Capability | Status | Evidence |
|---|---|---|
| Pure gate state machine → `oya-ci-required` | EXISTS | kernel `src/lib.rs:471,484` |
| `/gate-run` SHA-bound, fail-closed | EXISTS | app `src/lib.rs:468–483` |
| Trunk-sourced gate Job (PR bytes untrusted) | EXISTS | k8s-adapter `src/lib.rs:108–123` |
| Runner token isolation | EXISTS | k8s-adapter `src/lib.rs:167–175` |
| Forgejo status poster | EXISTS | forgejo-adapter `src/lib.rs:72–122` |
| Webhook→/gate-run dispatcher | EXISTS (opt-in, default=jenkins) | dispatch `src/dispatch.rs:222`; config `src/config.rs:90` |
| Phase-0 RED/GREEN policy engine + fixtures | EXISTS, in `cargo test` | kernel `src/lib.rs:285,1441`; `specs/fixtures/phase0-ci-enforcement-baseline/*` |
| Helm chart (controller + 2 SAs + netpol + ESO) | EXISTS as template (nonClaim) | `iac/k8s/helm/values.yaml` |
| Branch-protection apply/verify | EXISTS (check-only, self-disclaimed, legacy-CLI verifier) | `scripts/branch-protection-apply.sh`; `infra/branch-protection/dev.json:2,25` |
| GitHub status poster for `oya-ci-required` | **BUILD** (HTTP shape reusable from gateway github-adapter; wrong context+process today) | github-adapter `src/lib.rs`; `tests/d5_commit_status_post.rs:22` |
| Forge-of-record decision (GitHub vs Forgejo) | **BUILD/DECIDE** | mismatch: poster=Forgejo vs protection=GitHub (`baseline.json:82`) |
| Forge-neutral `CommitStatusPoster` seam | **BUILD** (small refactor of `ForgejoStatusPoster`) | kernel `src/lib.rs:620` |
| Live branch-protection flip to `["oya-ci-required"]` | **BUILD/ACT** | live set still `cargo-*`+`oya-pr-review` (`baseline.json:73–84`) |
| Tide required-context = `oya-ci-required` | **BUILD/FIX** (default is `oya-ci-gate`) | tide kernel `src/lib.rs:76` |
| Real deploy + RBAC + ESO sync evidence (drop nonClaims) | **BUILD/ACT** | `values.yaml:25,32,36,91` |
| Live GREEN/RED/tamper PR receipts + blocked-merge proof | **BUILD** (the go-live proof) | exit-gate fixture `tc-0.12-…` still RED |

---

## 6. Coverage / not-covered (no silent caps)

Read in full: controller kernel (partial — lines 1–1529 of 1970 read directly;
the remaining policy-test tail 1530–1970 was confirmed by structure +
`phase0_*` test names + grep, not line-by-line), controller app, k8s-adapter,
forgejo-adapter, controller binary, helm values; gateway top-level main/config/
dispatch/receiver(head) + crates github-adapter + its d5 tests + crates app bin;
tide kernel; buck2 gate; branch-protection dev.json + .github yaml +
branch-protection-apply.sh; baseline spec (head+live block) + 4 fixtures
(good, buck2-affected-only RED, candidate-mutable RED, exit-gate RED).

NOT read line-by-line (named so the gap is explicit): controller kernel lines
1530–1970 (policy fixture tests, sampled not full); gateway `receiver.rs` lines
121–end; gateway `event.rs`/`signature.rs`/`error.rs`/`replay.rs`; the gateway
DDD `crates/...-app/src/lib.rs` router + integration_webhook_flow test body;
ci-tide app/forgejo-adapter; helm templates (deployment/role/externalsecret/
networkpolicy bodies — only values.yaml read); the full fixture corpus (7 of 11
RED fixtures not opened); `infra/ci/jenkins/*`. The path the task cited
`/Users/jasonlee/Developer/source/oya/infra/ci/buck2-affected-gate.sh` does NOT
exist; the real file is `/Users/jasonlee/Developer/source/infra/ci/buck2-affected-gate.sh`
(read in full). The `.github/branch-protection.yaml` cited as
`/Users/jasonlee/Developer/source/infra/branch-protection/.../branch-protection.yaml`
was read at `/Users/jasonlee/Developer/source/.github/branch-protection.yaml`
(read in full). I did NOT run anything live (`gh api`, kube, buck2) — this is a
read-only audit, so all "live" claims are sourced from the checked-in baseline
evidence, not re-observed.
