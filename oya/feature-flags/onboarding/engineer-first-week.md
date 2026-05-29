---
doc_class: Onboarding
microservice: feature-flags
persona: engineer
related_adrs: [ADR-0159, ADR-0114, ADR-0316]
date: 2026-05-20
doc_status: published
---

# Engineer onboarding — first 5 working days

Audience: a new oyatie engineer who joined this week and has been pre-assigned to a workload µservice that already adopts `feature-flags`. By end of Day-5 they will have defined, evaluated, and sunset a flag of their own.

## Day 1 — Read + observe (no commits)

1. Read `PRD.md` end-to-end (≤ 35 min). Skim `ARCHITECTURE.md` for the picture of the three-tier evaluator (in-process SDK cache → side-car evaluator → server-side Cedar).
2. Open the Grafana board `feature-flags-overview` — note the `evaluations_total{flag_key=...}` line for your µservice's three highest-traffic flags.
3. Sit in on one `flag-review` triage call (Wed cadence). Watch how the on-call SRE decides whether a `kill_switch` flag flips for production or whether to escalate to incident-commander instead.

Acceptance: you can name the four variant types (bool, string, number, JSON-object) and the four flag lifecycles (release_toggle, experiment, permission_toggle, kill_switch) without notes.

## Day 2 — Local SDK install

Pick the language your µservice uses (Rust / TypeScript / Python). Vendor the SDK from `microservices/feature-flags/contracts/feature-flags-v1.proto` via `oya codegen feature-flags-sdk --lang rust --out ./crates/your-µservice/feature-flags-client`.

Wire one boolean evaluation in a non-production code path:

```rust
let on = ff_client.bool("yourµservice.beta.dashboard_v2", false).await?;
```

Run `oya verify` locally; the lane `oya-governance-feature-flag-sunset` reads the `sunset_at` declaration; if missing the lane WARNs (advisory) and you fix it.

Acceptance: green local verify, one SDK call wired, the flag visible in the staging dashboard within 30 s of first evaluation.

## Day 3 — Cedar predicate authoring

Move the Day-2 flag from default-only to persona-tier-bounded. Author a Cedar fragment in `microservices/feature-flags/policy/your-flag.cedar`:

```cedar
permit(principal, action == Action::"evaluate-flag-yourµservice.beta.dashboard_v2", resource)
when { principal.persona_tier == "beta-cohort" && resource.pack_id != "kr-pipa" };
```

Pair with the substrate on-call on the predicate before raising the PR — Cedar fragments compose with the µservice's `policy/auditor-scope.cedar` and conflicts surface only in pairing.

Acceptance: PR opened with the predicate; admission-gate is green; you understand why `pack_id != "kr-pipa"` is needed (KR-PIPA cohorting requires `paid compliance_pack` tier, not `paid`).

## Day 4 — Percentage rollout + cohort

Convert the flag to a 5 % cohort rollout. Read `IP-005-rollout-percentage-deterministic-hash.md` for how `(tenant_id, flag_key) → 0-99` bucketing works (xxHash3 with a per-flag salt).

Ship the rollout against the staging tenant cohort; watch `evaluations_total` split into `variant=on` vs `variant=off` in the dashboard. Bump to 10 %, 25 %, 50 % over the day; observe the deterministic-hash invariant — a tenant that landed in the 5 % bucket stays in the 50 % bucket (set-inclusion is monotonic).

Acceptance: the dashboard shows the expected 5 → 10 → 25 → 50 % traffic split; no tenant flipped variant mid-window.

## Day 5 — Sunset

Declare `sunset_at: 2026-09-01` in the flag definition. The lane `oya-governance-feature-flag-sunset` now treats the flag as time-bound; on the sunset date the lane will BLOCKER any code path that still reads the flag.

Open a follow-up issue `tracking-flag-sunset/yourµservice.beta.dashboard_v2` referencing the eventual `cargo run -p oya-dev-cli -- ff retire <flag-key>` invocation.

Acceptance: `sunset_at` set; follow-up issue filed; you can explain the difference between a `kill_switch` (no sunset; lives until the underlying feature is amputated) and a `release_toggle` (sunset within ≤ 90 days of GA).
