---
doc_class: Onboarding
microservice: observability
persona: sre-lead
related_adrs: [ADR-0130, ADR-0131, ADR-0329, ADR-0330, ADR-0331]
date: 2026-05-20
doc_status: published
---

# observability — SRE Lead First Week

Audience: an incoming SRE who owns one or more µservices and must wire them into the oyatie observability substrate before any dev→staging→prod promotion.

## Day 1 — environment + access

Morning (3 h):

1. Receive `iam` invite to the `sre-lead` Cedar role; confirm via the cloud IAM control-plane session record that you see `principal_type=human`, `audience_type=corporate`, and the role binding includes `observability::dashboard::read`, `observability::slo::author`, `observability::breach::acknowledge`. Record the native operation ledger id in your onboarding notes.
2. Clone the repo: `git clone https://github.com/oyatie/oyatie.git && cd oyatie`.
3. Create an isolated worktree branch from `dev`. GitHub is the temporary mirror/PR adapter; Buck2/Prow and native operation ledgers remain the authority. Do not install or use retired local command wrappers for onboarding actions.
4. Open Grafana at `https://grafana.<your-cell>.dev.oyatie.io` and verify you see the `Substrate Overview` folder and at least one µservice dashboard rendering live data.

Afternoon (4 h):

5. Read `oya/observability/ARCHITECTURE.md` end-to-end (~ 40 min).
6. Read `oya/observability/PHASE-01-AGENTIC-SLO-GATED-PROMOTION.md` (~ 25 min).
7. Read ADR-0130 + ADR-0131 (~ 30 min combined).
8. List your owned µservices from the service ownership registry/CODEOWNERS projection. GitHub metadata is mirror evidence only; the service-owned source shard is `oya/<service>/`.
9. For each owned µservice, confirm `oya/<service>/slos/` exists and contains at least one `*.openslo.yaml` file. If missing, you cannot promote past dev — file implementation-plan work for the gap.

End of Day 1 deliverable: an annotated `notes/day-1-owned-microservices.md` listing every µservice you own and its current SLO authoring state (none / partial / complete).

## Day 2 — sample-recipe + OTel collector

Morning (4 h):

1. Read IP-030 (`sample-recipe-per-microservice.md`) end-to-end. Understand the per-route sampling envelope.
2. For each owned µservice, copy the canonical sample recipe from a sibling service-owned shard or observability reference implementation into `oya/<service>/observability/sample-recipe.yaml`.
3. Edit the recipe: set `service.name` to your µservice slug; tune `tail_sampling.policies[]` — at minimum keep the `error-spans` (always_sample), `latency-tail` (above-95th-percentile), and `cross-tenant-correlation-tail` (boolean tag-presence) policies.
4. Validate through the native observability operation:

```text
Native operation: observability validate-recipe
Input shard: oya/<service>/observability/sample-recipe.yaml
Required evidence:
- Buck2 target(s) for the recipe/schema validation
- Prow/Kubernetes-native `oya-ci-required` job URL
- native operation ledger id and emitted audit-chain event ids
```

Resolve any schema validation errors before proceeding.

Afternoon (3 h):

5. Ensure your µservice's app crate imports the OTel SDK with the canonical span attributes: `tenant_id`, `pack_id`, `principal_id`, `audience_type`, `request_id`, `span_kind`. If any are missing, file an IP and stop here for the day.
6. Verify with the service-owned Buck2 OTel smoke target and watch the OTel collector logs at `kubectl -n observability logs deploy/otelcol-contrib` for incoming spans tagged with your service name. Attach the Buck2 Build ID and, for PRs, the Prow/Kubernetes-native `oya-ci-required` job URL.

End of Day 2 deliverable: a sample-recipe committed to your branch, validated, emitting recognisable traces to dev-cell ClickHouse within 90 s of root-span emit.

## Day 3 — SLO authoring (OpenSLO)

Morning (4 h):

1. Open `oya/<service>/slos/`. If empty, create starter `.openslo.yaml` files from the native observability SLO template operation. The template output covers availability, p99-latency, and error-rate and records the native operation ledger id.
2. Edit each `.openslo.yaml`. Bind the `objectives[].target` to a number you can actually meet (start conservative — 99.5 % availability for new services, 99.9 % once you have a quarter of data).
3. Bind the SLI to a real PromQL query against the metrics your µservice already emits. If the metric doesn't exist, file an IP for emit instrumentation; do not author an SLO against a metric that doesn't exist.

Afternoon (3 h):

4. Dry-run the SLO through the native observability operation:

```text
Native operation: observability dryrun-slo
Input shard: oya/<service>/slos/availability.openslo.yaml
Window: 7d
Required evidence:
- Buck2 target(s) for OpenSLO validation and replay contract checks
- Prow/Kubernetes-native `oya-ci-required` job URL
- native operation ledger id with SLI value and burn-rate evidence
```

If the SLI is already breaching, lower the objective or fix the µservice — do not ship a known-breaching SLO.
5. For each SLO: write a 1-paragraph rationale in the YAML's `description` field explaining why this number, what user pain it represents, and what action the breach should trigger.

End of Day 3 deliverable: at least 3 SLOs per owned µservice, dryrun-green for the last 7 d, with rationales committed.

## Day 4 — dashboards + runbooks

Morning (3 h):

1. Copy a canonical dashboard from a sibling service-owned shard or observability reference implementation into `oya/<service>/dashboards/<service>-overview.json`. Rename panels, swap PromQL queries, swap variables.
2. Reload Grafana through the native KRM/CUE desired-state PR and controller reconciliation path. Confirm your dashboard appears in `Folders → <service>`.
3. Pin the dashboard to your team's Grafana home page.

Afternoon (4 h):

4. For each SLO, author a runbook at `oya/<service>/runbooks/<slo-name>-breach.md`. Structure: "Symptom → First-glance dashboard panels → Top-3 likely causes with diagnostic commands → Mitigation steps → Escalation contact".
5. Wire the runbook URL into the SLO's `annotations.runbook` field.
6. Trigger a synthetic SLO breach through the native observability operation:

```text
Native operation: observability simulate-breach
Input shard: oya/<service>/slos/availability.openslo.yaml
Required evidence:
- Buck2 target(s) for breach-event contract validation
- Prow/Kubernetes-native `oya-ci-required` job URL
- native operation ledger id and emitted audit-chain event ids
```

Confirm: (a) breach alert arrives at your team's Slack/Discord/Telegram per the `notifications` µservice, (b) the runbook URL is in the alert payload, (c) the dashboard's "Recent SLO breaches" panel shows your synthetic event.

End of Day 4 deliverable: dashboards live, runbooks live, breach simulation green.

## Day 5 — promotion gate dry-run + handoff

Morning (4 h):

1. Read the ADR-0130 promotion-gate ledger through the native observability promotion-ledger operation. Observe the schema — what evidence is required to lift the gate.
2. Dry-run the gate through the native observability eligibility operation. The output enumerates every missing piece (uncommitted recipe, dryrun-failing SLO, missing dashboard, missing runbook, etc.). Resolve every issue or file a follow-up implementation plan.

Afternoon (4 h):

3. Open a PR on your branch with all observability deliverables (recipe + SLOs + dashboards + runbooks). Reference IP-002, IP-003, IP-030 in the PR body.
4. Request review from at least one observability-substrate owner per `.github/CODEOWNERS` and the service ownership registry projection.
5. After PR merge, Prow/Kubernetes-native `oya-ci-required` re-evaluates promotion evidence. If manual re-evaluation is needed, emit the native observability re-evaluate-promotion operation and attach its operation ledger id. The promotion gate should flip to `eligible` if all evidence is present.

End of Week 1 deliverable: your µservice has full observability evidence committed, the PR is open or merged, and you understand the gate semantics well enough to maintain them independently going forward.

## What you should know by end of week 1

- The OpenSLO authoring envelope per microservice.
- The OTel sample-recipe pattern + per-route policies.
- How to read a multi-window burn-rate alert and which window matters when.
- How to read promotion-gate evidence and unblock a stalled promotion.
- Who the substrate owners are and how to escalate when the SLO engine itself is the problem.

## What you should NOT do in week 1

- Don't bypass the SLO-engine and write your own breach detector.
- Don't tune tail-sampling probability below 1 % for any error-class span. The substrate enforces 100 % error sampling; you cannot lower it.
- Don't author SLOs against metrics that don't exist yet. Emit first, then SLO.
- Don't promote a µservice past dev without the full evidence chain. The gate is the contract.
