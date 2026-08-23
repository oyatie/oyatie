# Agent-Execution Controller (pod-runner + work-item + evidence-bundle)

> Extracted from `/Users/jasonlee/Developer/code/docs/source-distillation-cloud-intelligence.md`
> (a now-superseded harness distillation). Everything else in that doc was a thinner,
> partly-stale restatement of artifacts that already exist in source
> (`cloud/cloud-intelligence` PRD/IP-001, `oya/intelligence/*`, ADR-0255/0263/0296/0392/0408).
> **This is the one concept source did not already cover.**

> **STATUS: DECLINED — D-AEC-DECLINE (founder, 2026-06-07; door:one-way).** This layer is **NOT built**. ADR-0116 (retire external agent-coordination tooling) + ADR-0363 (retire the agentic-VCS layer) — both Accepted — deliberately killed the adjacent coordination layer; the missing area is **missing on purpose** (Linus-taste: don't add what isn't load-bearing). The `code` harness repo lapses; ADR-0116/0363 stand unamended. The body below is retained as the **recorded rationale for the decline** (current-truth = declined), not as an open proposal. (Supersedes the open-question framing + the smaller capture in the disjoint `dev`-branch PR #605, which is stale and should be closed.)

## Problem Statement

How might we run an AI agent (Claude Code / Codex / Gemini CLI) as an **ephemeral, policy-gated unit of work** — scheduled, isolated, audited, and handed back as sealed evidence — without trusting a thin status string and without reviving the retired VCS-foundry coordination layer?

This is **agent *execution*, not agent *inference*.** It is explicitly distinct from `cloud/cloud-intelligence`, which is the LLM key-pool egress gateway (`/v1/chat/completions`, SSE passthrough, per-tenant budgets). The gateway never schedules a pod or runs an agent; it brokers tokens. Nothing in `docs/decisions`, `specs/`, `cloud/`, or `oya/` currently owns the *execution* layer — `oya/intelligence` owns the substrate/supervisor/capabilities, but not "run this agent CLI as a K8s Job and give me sealed evidence back."

## Recommended Direction (the idea, distilled)

A flat, single-concern controller (per ADR-0131/0132 — **not** folded into cloud-intelligence) with three contracts and a thin runner:

1. **Work-item lifecycle** — `work-item.v1` + `controller-receipt.v1`, states:
   `queued → claimable → claimed → pod_scheduled → running → {completed | failed | cancelled}`.
   Claiming requires an external Cedar `WorkItemClaim` grant on `work-item:<id>`; both accepted **and** denied claims emit a receipt carrying observability context, policy decision, prev/next state, and worker identity. No in-pod PDP — consume grants.

2. **Pod-schedule plan** — `pod-schedule-plan.v1` (dry-run first, live K8s client later): namespace, runner image, service account, **non-root requirement**, runner args, artifact paths, labels, trace context. Declares its surfaces explicitly: **K8s Job API** for ephemeral runner scheduling, **Talos machine API** for node/cluster lifecycle (never SSH/host-shell), **mTLS gRPC** for internal controller↔runner RPC, **AsyncAPI** for lifecycle fanout, **OpenAPI** northbound/admin.

3. **Evidence-bundle handoff** — the runner's output is a compact `evidence-bundle.v1` receipt, **not** a status string: run identity, provider/auth/workflow, audience/modality, replay state, Cedar policy verdicts, status/error, **a reproduction command**, observability context, and transcript artifact refs. Completion aggregation consumes the bundle; the controller can later upload/seal the same artifact unchanged.

**Thin runner discipline:** accept a versioned task contract → emit substrate plan → validate policy guards *before* provider execution → emit structured JSONL lifecycle events → route to a provider adapter → write evidence → exit. No runner-local retry/checkpoint/session state (durability is the Workflow Engine's job, marked by a workflow profile).

**Provider-native lifecycle preservation (the second sub-idea):** unlike the gateway — which flattens everything to the OpenAI chat-completions wire shape — the execution adapters must **preserve native CLI lifecycle events**: Claude Code stream-json / hook / partial-message / subagent / usage / permission events; Codex CLI noninteractive/subagent events; Gemini `functionCall` parts + thought signatures. Conformance is a fixture-backed Rust contract that keeps unknown future events opaque and rejects raw-secret fixtures.

## Key Assumptions to Validate

- [ ] **Is this layer wanted at all, or retired on purpose?** ADR-0116 (retire external agent-coordination tooling) and ADR-0363 (retire agentic VCS foundry) deliberately killed the adjacent coordination layer. This idea must either (a) be reconciled as a *net-new, narrower* concern that those ADRs did not intend to forbid, or (b) be explicitly declined. **Resolve this before any code.** *Decision owner: founder.*
- [ ] **Boundary holds.** It is a new flat `oyatie-*` (or `cloud-*`) service, single-concern, consuming cloud-intelligence for inference and `oya/intelligence` for substrate/guardrails — never duplicating either. *Test: `depends_on` graph has zero new inference/guardrail logic.*
- [ ] **Credential handles only.** Task/launch contracts carry `credential_ref` handles (ADR-0296 sidecar / OpenBao response-wrapping), never secret material; runner emits redacted dispatch events only.
- [ ] **Talos-native split.** Workloads via K8s Job/Pod; node/cluster lifecycle via Talos API. No host-shell.
- [ ] **Evidence is the only handoff.** Controllers consume `evidence-bundle.v1`, never bespoke runner status files.

## Non-Goals (inherited from the source distillation)

- No NativeLink/RBE, no BXL affected-query, no browser-automation subscription auth, no real provider network calls, no internal-model dependency — until each has green evidence.
- **Not** a re-home of inference proxying, prompt/eval registry, or guardrail classification (those have homes).

## Provenance

Source harness repo: `/Users/jasonlee/Developer/code` (Rust pod-runner/swarm-controller prototype + `examples/work-items/`). If this idea is accepted, audit that repo's `crates/` for salvageable working code before re-authoring. If declined, record the decline here and let the harness repo lapse.
