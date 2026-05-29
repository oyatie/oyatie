---
doc_class: Tutorial
microservice: feature-flags
persona: engineer
language: rust
date: 2026-05-20
doc_status: published
---

# Getting started — Rust SDK in 15 minutes

You will: install the SDK, wire one boolean evaluation, observe the result in the local dashboard, and roll the flag to 50 % cohort. Total time ≤ 15 minutes on a fresh laptop with the oyatie monorepo cloned.

## Pre-requisites

- A clone of `oyatie` at the head of `dev` or later.
- `cargo nextest` installed.
- The local cell stack running (`make cell-up` — wait for `feature-flags` to report `ready` in `kubectl get pods -n oyatie-cell-local`).

If `make cell-up` is novel, read `microservices/cell/onboarding/local-cell-first-spin.md` first; it is a 10-minute pre-requisite.

## Step 1 — Generate SDK stubs (≤ 90 s)

```sh
oya codegen feature-flags-sdk \
    --lang rust \
    --out ./crates/tutorial-µservice/feature-flags-client
```

This reads `microservices/feature-flags/contracts/feature-flags-v1.proto` and writes a typed Rust client. The generated `lib.rs` exposes `FFClient::new(endpoint, tenant_id)`, `eval_bool(flag_key, default)`, `eval_string(flag_key, default)`, `eval_number(flag_key, default)`, `eval_json(flag_key, default)`. No hand-edits to the generated file — re-run `oya codegen` on contract changes.

## Step 2 — Define your tutorial flag (≤ 2 min)

Open `microservices/feature-flags/catalog/flags/` and add `tutorial.hello-world.yaml`:

```yaml
flag_key: tutorial.hello-world
intent: First-tutorial flag; demonstrates the eval path end-to-end.
lifecycle: release_toggle
variants:
  on: true
  off: false
default_variant: off
sunset_at: 2026-08-01
audit_required: false
targeting:
  - rule_id: r-001
    cedar_fragment: tutorial-hello-world.cedar
    variant: on
```

Author the Cedar fragment at `policy/tutorial-hello-world.cedar`:

```cedar
permit(principal, action == Action::"evaluate-flag-tutorial.hello-world", resource)
when { principal.persona_tier == "tutorial" };
```

Run `cargo run -p oya-dev-cli -- ff lint --flag tutorial.hello-world` — should green.

## Step 3 — Wire the SDK in your µservice (≤ 3 min)

```rust
use feature_flags_client::FFClient;

let ff = FFClient::new(
    std::env::var("FF_ENDPOINT").unwrap_or_else(|_| "http://localhost:50500".into()),
    tenant_id.clone(),
);

let greeting = if ff.eval_bool("tutorial.hello-world", false).await? {
    "Hello, oyatie!"
} else {
    "Default greeting"
};

println!("{greeting}");
```

Set the persona tier on your local request context (the SDK reads it from the request-bound `Context::persona_tier`); for the tutorial use `tutorial`.

## Step 4 — Observe the result (≤ 2 min)

Run your µservice. Open Grafana at `http://localhost:3000`, dashboard `feature-flags-overview`. The panel `evaluations_total{flag_key="tutorial.hello-world"}` should show traffic. The `variant=on` line moves when your request's persona-tier is `tutorial`; `variant=off` line moves otherwise.

Confirm the audit-chain wrote a `flag_definition_created` event: `cargo run -p oya-dev-cli -- audit query --event flag_definition_created --flag tutorial.hello-world`. Should return one signed event.

## Step 5 — Roll to 50 % (≤ 3 min)

Edit `tutorial.hello-world.yaml` and add the rollout block:

```yaml
rollout:
  type: percentage
  percentage: 50
  salt: hello-world-tutorial-2026-05
```

Run `cargo run -p oya-dev-cli -- ff publish --flag tutorial.hello-world`. Within 60 s the SDK cache refreshes. Generate ten test requests with ten different tenant IDs. Approximately five land in `variant=on`, five in `variant=off`. The mapping is deterministic — re-running the test with the same tenant IDs yields the same split.

## Step 6 — Sunset (≤ 1 min)

You declared `sunset_at: 2026-08-01` in Step 2. On that date the lane `oya-governance-feature-flag-sunset` will BLOCKER any code path still reading the flag. To delete now (since this is a tutorial), run:

```sh
cargo run -p oya-dev-cli -- ff retire --flag tutorial.hello-world --reason tutorial-complete
```

The retire command emits a `flag_retired` audit-chain event and refuses to delete if any production evaluation has been seen in the last 24 h.

## What you learned

- The four-step lifecycle: catalog YAML → Cedar fragment → SDK call → publish.
- The deterministic-hash invariant on rollout percentages (Step 5).
- The audit-chain events surrounding define / publish / evaluate / retire.
- The `sunset_at` discipline that prevents flag-debt accumulation.

Next: read `tutorials/cohort-rollout-with-analytics.md` for the production cohort path; then `tutorials/kill-switch-on-call-runbook.md` for the SRE side of the surface.
