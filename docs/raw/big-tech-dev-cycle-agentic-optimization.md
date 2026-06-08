---
doc_status: published
---

# Big-Tech Dev Cycle and Project Management → Agentic Optimization

**Date:** 2026-05-10
**Status:** Synthesis report; no implementation started.
**Decision target:** Re-shape Bominal/Oyatie's Foundry runtime + Agentic Delivery Fabric so the canonical big-tech dev cycle and PM primitives are inverted into a non-stop autonomous Claude + OpenAI + Gemini (× API + subscription) engine.
**Companion artifacts:** `claude-code-backup-comprehensive-analysis.md` (Claude Code teardown + Foundry P0 patterns), `agentic-delivery-vcs-cicd-report.md` (delivery substrate strategy), `agentic-delivery-fabric-executable-prd.md` (executable PRD with M0–M8).

This document analyzes how Amazon, Google, Meta, Microsoft, and (where useful) Apple actually run their development cycles and project management, and proposes a per-primitive inversion for an autonomous-agent operating model — the "agentic dev cycle." It does **not** assume what hyperscalers do is sacred; it factors out which practices exist because of *human* constraints (synchronous attention, batch ceremonies, escalation latency) and proposes the agentic equivalent that holds the same invariants without the human bottleneck.

---

## 0. Executive synthesis

Big-tech dev cycles converge on a 7-phase loop: **Strategy → Discovery → Decomposition → Implementation → Review → Release → Ops**. Each phase has a canonical artifact (PR-FAQ, design doc, OKR, sprint plan, code review, release plan, postmortem) and a canonical ceremony (planning week, design review, sprint planning, daily standup, code review meeting, release readiness review, retro). The cycle is built around three scarcities that **only apply to humans**: (a) attention is expensive and synchronous; (b) coordination requires shared context windows (meetings, docs); (c) trust is granted in big batches (promotion, OKR sign-off, ADR approval).

Autonomous agents invert all three: attention is cheap and async, coordination happens through machine-readable state, and trust can be granted in narrow continuous slices via policy envelopes. The agentic optimization is therefore not "speed up the existing cycle." It is **re-shape every phase around durable typed objects that agents emit and humans review on dashboards**, replacing batch ceremonies with continuous streaming events.

**The five inversions that matter most for Foundry:**

1. **OKRs → Goal records.** Quarterly cascades become per-task durable Goal objects with quantified acceptance criteria, policy envelope, autonomy ceiling, evidence bundle pointer.
2. **Sprints → continuous flow with budget envelopes.** Two-week boxes dissolve. Token / wall-clock / tool-call / cost budgets bound each goal; agents work continuously inside them.
3. **Code review → adversarial verifier-agent + senior-human gate for high-risk.** Routine review goes to a verifier agent that ingests evidence and runs typed checks. Humans review only what crosses the autonomy ceiling or the policy envelope.
4. **RACI → AgentPodManifest.** Responsible/Accountable/Consulted/Informed is replaced by a typed manifest declaring `owns`, `autonomy.max_tier`, `tpm_required_if`, `escalates_to`. The manifest is policy code, not a wiki page.
5. **Standup / status doc / weekly review → continuous trace stream + dashboards.** Status meetings disappear; humans subscribe to event topics keyed by goal/pod/release-train.

These map cleanly onto the existing Foundry kernels (`oya-intelligence-capability/run/step/evidence/policy/provider`) plus the new Delivery Fabric kernels (`ChangeSet/PatchSet/Stack/EvidenceBundle/PolicyVerdict/AgentPodManifest/LaneDefinition`). The remaining gap is the **upper layer**: Goal, Plan, Verifier, Release-Train, Ops-Loop. Those are what this report formalizes.

---

## 1. The canonical big-tech dev cycle (7 phases)

The same 7 phases recur across every public engineering description from Amazon, Google, Meta, Microsoft, Apple, Netflix, Stripe. Naming differs; structure is identical.

### 1.1 Phase taxonomy

| # | Phase | Core question | Canonical artifact | Canonical ceremony |
|---|---|---|---|---|
| 1 | Strategy | "What are we trying to accomplish, by when, with what success criterion?" | OKR / North-Star metric / PR-FAQ | Planning week, OP1/OP2 (Amazon), DRI-doc (Apple) |
| 2 | Discovery | "What is the right thing to build, and is it possible?" | Design doc / RFC / 6-pager / EngDoc | Design review, Bar Raiser |
| 3 | Decomposition | "How do we break it into shippable units, and who owns each?" | Epic/story/task tree / WBS / project plan | Sprint planning, capacity review, RACI workshop |
| 4 | Implementation | "Build it, with tests, with reviewable diffs." | Code change / PR / patchset / commit stack | Standup, pairing, code review |
| 5 | Review | "Is this safe to ship: correct, compatible, secure, performant?" | Code review verdict + CI evidence + ADR | Code review meeting, readability review, security review |
| 6 | Release | "Deploy progressively, prove safe at scale, roll back if not." | Release plan + canary + bake + rollback artifact | Release readiness review, on-call handoff |
| 7 | Ops | "Watch it in production, learn from incidents, feed back to strategy." | Dashboard + SLO / runbook / postmortem / blameless review | Sev review, weekly metrics review, retro |

### 1.2 Per-phase practice across hyperscalers

**Phase 1 — Strategy / planning**

- *Amazon* uses **OP1 (annual operating plan)** and **OP2 (quarterly tactical plan)**, both authored as 6-page narratives + appendices. Strategy artifacts are **PR-FAQs** ("press release + frequently asked questions") written *before* engineering work starts — the press release describes the launched product as if it shipped today, the FAQ stress-tests internal/external objections. Source: Amazon "working backwards" doctrine.
- *Google* uses **OKRs** (Objectives + Key Results) cascading from company → group → team → individual, originally inherited from Intel via Doerr. Quarterly cycles. KR's are 0.0–1.0 scored. Strategy artifacts are also **EngDocs** / **design docs** with explicit alternatives-considered sections.
- *Meta* uses **half-year H1/H2 planning** with explicit Impact Awards retroactively measuring shipped work; less ceremony than OKRs, more emphasis on shipping speed.
- *Microsoft* uses **semester planning** under "One Engineering System," with documented OKR adoption post-Nadella.
- *Apple* uses **DRI (Directly Responsible Individual)** ownership + secrecy gates; less written, more reviewed in person at small forums.

**Phase 2 — Discovery / design**

- Universal: a **design doc / RFC / EngDoc / 6-pager** is required before non-trivial work. Common sections: context, goals, non-goals, design overview, alternatives considered, security/privacy/operability review, rollout plan.
- *Amazon* enforces **Bar Raisers** for hires *and* design reviews — an empowered objector drawn from outside the proposing team.
- *Google* enforces **readability** as a separate review competency.
- *Meta* uses **paste / Phabricator paste docs** with lighter formality.

**Phase 3 — Decomposition / planning**

- *Amazon/Google* both decompose into **epics → stories → tasks** with the team's PM/EM doing the cut. Typical capacity model: 1.5x story-points-per-sprint with leftover ratcheted forward. Story sizing in points or t-shirt-sizes.
- *Meta* uses task-tracker-driven decomposition with stacked-diff PR culture.
- *Microsoft* uses **VSTS/Azure DevOps work-item hierarchy** (initiative → epic → feature → user story → task).
- All use a **RACI** or **DACI** matrix for cross-team work; RACI = Responsible/Accountable/Consulted/Informed; DACI = Driver/Approver/Contributors/Informed (DACI is more Atlassian/IC, RACI more enterprise).

**Phase 4 — Implementation**

- *Google/Amazon/Meta* are all **trunk-based**: short-lived branches (often <1 day) merged to a main line; long-lived release branches cut at tag.
- *Google* enforces **monorepo** with hermetic Bazel build + Tricorder/Critique presubmit static analysis at scale (sources: Software Engineering at Google chapters 22-25).
- *Meta* invented and uses **stacked diffs** (Phabricator → Sapling) — each conceptual change is a standalone diff in a stack rebased on trunk; a stack of 5–15 small diffs is typical for a feature.
- *Microsoft* uses Git+VFS (Scalar) at scale with BuildXL/CloudBuild distributed cache.

**Phase 5 — Review**

- Universal: **at least one human reviewer** + automated CI gate. Variants:
  - *Google*: 1 owner + 1 readability (if applicable) + 1 LGTM + presubmit-green required to submit. Tricorder findings inline. Critique tool.
  - *Amazon*: code-review-required + CI-required; service-team owns; security review for sensitive surfaces.
  - *Meta*: stacked-diff per-diff review; ship-bot auto-submits when accepted.
- All use **CODEOWNERS-class** files to route review by path.
- All have **release captain** or **on-call** rotations that own ship readiness for a service.

**Phase 6 — Release**

- *Amazon's release-captain pipeline* (CodeDeploy + CodePipeline) is the public canonical model: immutable artifact → one-box deploy → bake → cell deploy → canary 5% → bake → wave rollout → auto-rollback on metric burn. Sources: AWS Builder's Library.
- *Google* uses Spinnaker + Borg + canary-analysis-service.
- *Meta* uses Tupperware + Conveyor with continuous shadow + canary.
- *Microsoft* uses ring deployment (Ring 0/1/2/3 = canary/internal/public-canary/global).

**Phase 7 — Ops**

- Universal: **SLO-driven on-call + blameless postmortem + runbook**. The postmortem is the closing ceremony for any Sev1/Sev2.
- *Google's SRE book* is the canonical doctrine: error budgets, toil reduction, the four golden signals (latency, traffic, errors, saturation).
- *Amazon's COE (Correction of Errors)* doc is structured: timeline → root-cause-five-whys → mitigations (with owners + dates) → testing-of-mitigations.
- *Meta/Microsoft* run similar postmortems with internal "lessons learned" databases.

### 1.3 Where the cycle leaks time, quality, or autonomy (the seven bottlenecks)

| # | Bottleneck | Why it exists for humans | Cost |
|---|---|---|---|
| 1 | Spec ambiguity at Phase 2 | humans synthesize requirements over weeks of meetings; ambiguity not rigorously measured | rebuild loops; 30–50% of work is rework |
| 2 | Capacity rebalancing latency | OP1/OP2/OKR cycles are 6–12 weeks; can't reshape mid-cycle | slow response to opportunity / threat |
| 3 | Review wait | reviewer time is expensive and contended | hours-to-days per diff in industry; weeks at worst |
| 4 | CI duration | builds and tests are minutes-to-hours | per-diff iteration cost; serializes stack work |
| 5 | Approval queues | ADR approval, security review, privacy review are gated on humans | weeks per cross-cutting change |
| 6 | Postmortem latency | RCA is written days-weeks after incident; remediation gated on next sprint | repeats; trust erodes |
| 7 | Coordination tax | RACI / standups / planning meetings consume 15–30% of senior eng time | reduces deep-work time per engineer |

These seven are not bugs. They are the **inevitable cost of running a process around scarce, slow, synchronous human attention**. Removing the human from the path inverts every one.

---

## 2. Project management primitives across big-tech (cross-section)

Independent of the per-phase practice, project management shares a common primitive set across all hyperscalers:

| Primitive | What it does | Canonical implementation |
|---|---|---|
| **Goal** | Declares "what success looks like" with quantified KRs and a horizon | OKR (Google), PR-FAQ (Amazon), Half-plan (Meta), Semester goal (MS) |
| **Decomposition** | Cuts a goal into ownership units | Epic/story/task tree, WBS |
| **Capacity** | Bounds how much goal-volume the org can absorb | Story-points/sprint, FTE-allocated, headcount plan |
| **Dependency** | Records cross-team or cross-system blockers | Project-plan dependency graph, RACI |
| **Risk** | Records what could go wrong + mitigation | Risk register, contingency plan |
| **Decision** | Records why one path was chosen | ADR, RFC, design doc alternatives section, DACI |
| **Quality** | Defines "shippable" | DoD, Bar Raiser, readability, semver compatibility, security review |
| **Communication** | Surfaces state / asks for help | Standup, status doc, dashboard, on-call handoff |
| **Ownership** | Maps surface to owner | CODEOWNERS, service-owner matrix, RACI, DRI |
| **Escalation** | Routes blockers up | TPM ladder, escalation buddy, on-call escalation tree |
| **Evidence** | Proves work was done | Demo, screenshot, test report, metric chart, signed artifact |
| **Promotion / autonomy** | Grants larger scope to trusted contributors | Promotion packet, level ladder, on-call qualification |

Every hyperscaler has all 12 primitives. Names differ; semantics are stable enough to be a checklist.

---

## 3. The agentic inversion — five core ideas

The following table is the backbone of this report. Each row names a primitive that humans need *because* they are humans, and the agent equivalent that holds the same invariant.

| Primitive | Why humans need it | Agent equivalent | Load-bearing artifact |
|---|---|---|---|
| **OKRs / quarterly cascade** | scarce attention forces batching of priorities | durable **Goal** records w/ quantified acceptance criteria, autonomy ceiling, evidence pointer | `oya-intelligence-goal-kernel` Goal struct |
| **Sprint / iteration boundary** | gives slow humans a coherent batch to plan around | continuous flow w/ **budget envelope** (token / wall-clock / cost / retry / child-agent) per Goal | `oya-intelligence-budget-kernel` BudgetEnvelope |
| **Standup / status report** | synchronizes async humans on shared state | continuous **event stream** + dashboards keyed by Goal, Pod, ChangeSet, Release-Train | OpenTelemetry `gen_ai.*` spans + event topics |
| **Sprint planning meeting** | slow humans can't replan continuously | **planner-agent** that converts Goal → Plan DAG with blast-radius classification + write-scope leases | `oya-intelligence-planner-app` |
| **Code review meeting / readability review** | slow human attention is the only quality gate | **verifier-agent** that ingests EvidenceBundle + does typed checks; senior-human gate only when policy verdict crosses ceiling | `oya-intelligence-verifier-app` |
| **Postmortem / COE doc** | humans need narrative to learn | **trace-grading + auto-postmortem agent** that emits root-cause + mitigations + tests for mitigations from typed traces | `oya-intelligence-incident-grader` |
| **RACI / DACI / CODEOWNERS** | human ownership is implicit and needs documentation | **AgentPodManifest** (typed YAML schema) declaring `owns`, `autonomy.max_tier`, `tpm_required_if`, `escalates_to` | `oya.delivery.agent-pod.v1` schema |
| **Release readiness review** | humans are the gate of last resort | **release-captain pipeline** w/ canary/wave/bake/rollback policy declared as code | `oya.delivery.release.v1` |
| **OP1/OP2 capacity planning** | humans allocate FTEs annually | **agent-cohort policy** — N agents × per-agent cost ceiling × per-Goal budget; reshapes continuously | `oya-intelligence-cohort-app` |
| **ADR / RFC / design doc** | humans need to read the why | **same artifact, agent-drafted, human-approved** at policy-defined risk threshold | `docs/consolidated/decisions/ADR-*.md` (unchanged) |
| **Bar Raiser / Readability / Security review** | independent human objector adds skepticism | **adversarial critic agent** whose verdict is required for autonomy uplift; no self-approval | `oh-my-claudecode:critic` pattern (already in repo) |
| **Promotion packet / autonomy uplift** | humans need to prove they've grown | **autonomy-tier ratchet** based on accumulated evidence — eval pass rate, postmortem-clean rate, blast-radius coverage | `oya-intelligence-autonomy-domain` autonomy ratchet rule |

The pattern is consistent: every PM primitive has a typed-record + adversarial-critic-agent equivalent. Where humans rely on narrative judgment, agents rely on typed evidence + adversarial checking.

### 3.1 The five core conceptual moves

1. **Phase boundaries dissolve into typed state transitions.** A "sprint" is just `goal.status: planned → executing → verifying → released`. The boundary lives on the Goal record, not on the calendar.
2. **Ceremonies become subscriptions.** Standups, status meetings, release readiness reviews, retros — all become event topics. Humans subscribe per-role; agents subscribe per-policy.
3. **Documents become executable.** OKRs, ADRs, design docs, runbooks — all carry typed YAML/JSON sidecars that machines parse. The narrative is for humans; the schema is for machines.
4. **Approval becomes envelope.** Per-decision approval is replaced by per-scope policy envelope. Inside the envelope, agents act freely; outside, they escalate.
5. **Postmortem becomes online.** The incident is annotated *during* execution (typed traces, hypothesis state, decision log). The "doc" is generated, not authored.

---

## 4. Per-phase agentic transformation

### 4.1 Phase 1 — Strategy / Goal

**Human shape**: OKR cascade, PR-FAQ, planning week, written 6-pager.

**Agent shape**:
```yaml
schema: oya.foundry.goal.v1
id: goal_01HX...
title: "<short business intent>"
objective: "<measurable goal>"
acceptance_criteria:
  - id: ac_001
    statement: "..."
    verifier: test|static_check|metric|policy_check|human_review
    threshold: "..."
constraints:
  max_wall_clock_minutes: 240
  max_tokens: 5_000_000
  max_cost_usd: 25
  allowed_paths: [...]
  denied_paths: [...]
  allowed_network_hosts: [...]
  autonomy_ceiling: T0|T1|T2|T3|T4
required_evidence:
  - tests
  - diff_summary
  - risk_assessment
  - audit_event
status: queued|planning|executing|verifying|blocked|complete|failed|cancelled
parent_goal: goal_...   # for OKR-style cascade
provenance:
  drafted_by: agent|human
  approved_by: principal|null
  approved_at: ...
```

**Ceremonies replaced**: planning week, OP1/OP2, OKR cascade, PR-FAQ writeup. Goal records carry the same information — quantified objective, KRs, horizon — without the meeting. Humans review goals on a dashboard and approve only those that exceed default policy.

**Open question** (for Phase 4 interview): should Goal carry a `business_value` numeric for prioritization, or is FIFO + autonomy-tier sufficient?

### 4.2 Phase 2 — Discovery / Design

**Human shape**: design doc, RFC, EngDoc, design review meeting, Bar Raiser.

**Agent shape**: deep-interview skill (already exists in this repo) drives ambiguity to ≤20%; spec is emitted as machine-readable + human-readable. Design alternatives are recorded as typed `Decision` records:
```yaml
schema: oya.foundry.decision.v1
id: dec_01HX...
goal_id: goal_...
statement: "..."
rationale: "..."
rejected_alternatives:
  - option: "..."
    reason: "..."
confidence: low|medium|high
drafted_by: agent_run_...
approved_by: principal|null
ad_r_ref: docs/consolidated/decisions/ADR-####.md|null
```

**Bar Raiser equivalent**: an adversarial **critic-agent** is mandatory for any goal with `autonomy_ceiling >= T3` or `blast_radius >= axis_substrate`. The critic is structurally separate (different agent, different prompt, no shared context) and produces an objection list that the planner must address.

**Ceremonies replaced**: design review meeting, RFC discussion. Humans review the spec + decisions + critic verdict on a dashboard.

### 4.3 Phase 3 — Decomposition / Plan

**Human shape**: epic→story→task tree, sprint planning, RACI workshop, capacity review.

**Agent shape**: planner-agent emits a typed **Plan DAG**:
```yaml
schema: oya.foundry.plan.v1
id: plan_01HX...
goal_id: goal_01HX...
version: 1
tasks:
  - task_id: t_001
    title: "Map current implementation"
    type: exploration|design|code_change|verification|release
    dependencies: []
    owner_role: explore|architect|executor|verifier|critic|writer
    write_scope: []
    read_scope: ["src/**"]
    success_signal: file_refs_collected|tests_pass|spec_acc_pass|...
    estimated_budget:
      tokens: 50_000
      wall_clock_minutes: 15
critical_path: [t_001, t_002, t_003]
```

**RACI equivalent**: each `task.owner_role` resolves through the AgentPodManifest registry to a specific Pod; the AgentPodManifest carries the RACI semantics (`owns`, `autonomy`, `tpm_required_if`, `escalates_to`).

**Capacity equivalent**: budget envelopes per goal × per-task, adjusted by an autonomous **cohort-controller** that reshapes per-provider concurrency based on rate-limit telemetry, cost ceilings, and queue depth.

### 4.4 Phase 4 — Implementation

**Human shape**: trunk-based dev, branch + PR or stacked-diff, daily standup, pairing.

**Agent shape**: per-pod ephemeral worktree (git worktree, **not tmux** per Bominal DESIGN.md §3.0); each task gets a **TaskLease**:
```yaml
schema: oya.foundry.lease.v1
id: lease_01HX...
task_id: t_002
agent_id: agent_run_...
role: executor
write_scope: [src/foo/**]
read_scope: [src/**, tests/**]
expires_at: ...
heartbeat_interval_seconds: 30
conflict_policy: fail_on_overlap|allow_readonly_overlap|merge_after_review
```

Implementation produces **PatchSets** (immutable diffs) attached to a **ChangeSet** (logical change identity) per the Delivery Fabric PRD. Stacked work uses explicit `Stack` edges, not branch names.

**Standup equivalent**: continuous live event stream (`agent.task.started`, `agent.task.heartbeat`, `agent.task.tool_invoked`, `agent.task.completed`); humans subscribe to dashboards filtered by Goal/Pod.

**Pairing equivalent**: optional **driver/navigator agent split** — one agent emits actions, another agent reviews each tool call and can veto before execution. Used for high-risk T3+ tasks.

### 4.5 Phase 5 — Review / Verify

**Human shape**: code review meeting, readability review, security review, ADR approval, CI green.

**Agent shape**: **verifier-agent** ingests the EvidenceBundle and produces a typed verdict:
```yaml
schema: oya.foundry.verdict.v1
id: vd_01HX...
change_id: chg_...
patchset_id: ps_...
status: allow|deny|needs_tpm|needs_human|needs_security
reasons:
  - code: missing_lane|failed_lane|stale_evidence|ownership_gap|blast_radius_exceeded|stack_parent_blocked|adversarial_critic_objection
    message: "..."
critic_required: true|false
critic_verdict_ref: vd_critic_...|null
```

**Critical invariant** (from Claude-code-backup-comprehensive-analysis Appendix A): *no agent self-approves*. The executor cannot mark its own high-risk work as complete without verifier evidence; the verifier is structurally separate (different agent, different prompt).

**Senior-human gate**: triggered automatically when verdict is `needs_human`/`needs_security`/`needs_tpm`. Gate consumes a typed **HumanApprovalRequest** record with diff/risk/provenance/evidence pre-rendered; humans tap allow/deny on a dashboard, not in a meeting.

### 4.6 Phase 6 — Release

**Human shape**: release readiness review, on-call handoff, deployment captain.

**Agent shape**: release-captain pipeline (per Delivery Fabric PRD § 8) — immutable artifact + provenance + SBOM → one-box → bake → cell → canary % → bake → wave → auto-rollback on metric burn. Release object is typed (`oya.delivery.release.v1`); the "review" is a pre-deploy policy verdict + evidence-bundle freshness check.

**Release captain equivalent**: the pipeline IS the captain. Humans monitor the release-train dashboard. On-call human is paged only when auto-rollback fires *and* fails to bring SLO back inside threshold within window.

### 4.7 Phase 7 — Ops / Learn

**Human shape**: SLO dashboards, on-call rotation, postmortem doc, retro meeting, runbook.

**Agent shape**:
- **SLO + 4-golden-signals**: same shape as Google SRE; agents own monitoring queries.
- **On-call**: agent first-responder under T2/T3; senior human escalation under T4. Auto-runbook execution via capability registry.
- **Postmortem**: **incident-grader agent** ingests typed traces during the incident (not after) and emits root-cause + five-whys + mitigations + mitigation-tests. Humans approve the verdict.
- **Retro**: dashboard of trace-grades over time; the retro "meeting" is a quarterly review of the dashboard, not a 60-min meeting.

**Loop closure**: ops findings emit `learning` records that become inputs to Phase 1 (strategy) — so the cycle closes. Crucially, the loop is **continuous, not annual**.

---

## 5. Bottleneck-by-bottleneck compression table

For each of the seven bottlenecks identified in §1.3, the agentic optimization and its load-bearing primitive:

| Bottleneck | Human time | Agent time | Compression mechanism | Load-bearing primitive |
|---|---|---|---|---|
| Spec ambiguity | 2–6 weeks | minutes–hours | deep-interview to ≤20% ambiguity, machine-checked acceptance criteria | `oya-intelligence-spec-interview-app` (deep-interview skill kernelized) |
| Capacity rebalancing | 6–12 weeks (OP1/OP2) | continuous | per-goal budget envelope; cohort-controller reshapes provider concurrency | `BudgetEnvelope` + `CohortController` |
| Review wait | hours–days | seconds–minutes | verifier-agent + adversarial critic; senior human only on threshold | `oya-intelligence-verifier-app` |
| CI duration | 30–90 min | 3–10 min affected | Rust affected-graph + nextest sharding + sccache + remote cache | per Delivery Fabric PRD § 5 |
| Approval queues | weeks | seconds–hours | policy envelope evaluated at runtime; human approval only on boundary crossing | `oya-intelligence-policy-domain` runtime gate |
| Postmortem latency | days–weeks | online | typed-trace incident grader emits RCA during execution | `oya-intelligence-incident-grader` |
| Coordination tax | 15–30% senior eng time | ~0% on routine | event-stream subscriptions replace standups/status meetings | OpenTelemetry `gen_ai.*` + event topics |

The total compression across all seven, conservatively estimated: a 4–8 week strategy-to-shipped cycle compresses to **24–72 hours for a low-blast-radius pod-owned change**, gated only by the policy envelope and verifier verdict.

---

## 6. Mapping to Bominal Foundry — what exists, what's missing

### 6.1 What Foundry already has (in-tree as of 2026-05-10)

From `services/agent/daemon/src/foundry/` (~11,700 lines, 17 modules) and `docs/consolidated/products/foundry/PRD.md`:

| Have | File / spec |
|---|---|
| Issue/Run/Step/Workspace types | `domain.rs`, `orchestrator.rs` |
| Codex provider adapter (live) | `app_server.rs`, `runner.rs` |
| Workspace per Issue + after_create/before_run/after_run hooks | `workspace.rs` |
| Token ledger (per-run accounting) | `token_ledger.rs` (1786 lines) |
| Workflow engine (file-backed YAML) | `workflow_engine.rs`, `workflow.rs` |
| HTTP API surface | `http.rs` (axum) |
| Subscription pool (provider session mgmt) | `providers.rs` |
| Tracker (issue source-of-record) | `tracker.rs` |
| Capability/Step/Run/Evidence/Provider/AutonomyCeiling kernel structs (PRD'd, not yet implemented) | PRD § 5 |
| Multi-provider adapter trait (PRD'd) | PRD § 5.1 + DESIGN § 3.0 |

### 6.2 What's missing for the autonomous Claude+OpenAI+Gemini × API+sub engine

Cross-referenced against Claude Code backup analysis Appendix A + Delivery Fabric PRD + this report's §3 inversion table:

| Missing | Source(s) demanding it | Where it goes |
|---|---|---|
| Goal kernel (separate from Run) | Appendix A § A.5.1; this report § 4.1 | `crates/oya-intelligence-goal-kernel` |
| Plan DAG kernel + planner-app | Appendix A § A.5.2; this report § 4.3 | `crates/oya-intelligence-plan-kernel` + `-planner-app` |
| TaskLease kernel + lease-manager | Appendix A § A.5.4 + § A.7.5; this report § 4.4 | `crates/oya-intelligence-lease-kernel` + `-lease-domain` |
| EvidenceBundle kernel + verifier-app | Appendix A § A.5.5; this report § 4.5; Delivery Fabric PRD § 3.4 | `crates/oya-intelligence-evidence-kernel` (PRD'd, not yet) + `-verifier-app` |
| AgentPodManifest schema + registry | Delivery Fabric PRD § 3.6 | `crates/oya-intelligence-pod-kernel` + `-pod-domain` |
| ChangeSet/PatchSet/Stack kernels | Delivery Fabric PRD § 3.1–3.3 | `crates/oya-intelligence-change-kernel` |
| PolicyEnvelope runtime gate (Cedar) | Appendix A § A.5.3; PRD § 13.3.1 #1 (autonomy ceiling as runtime gate, not docs) | `crates/oya-intelligence-policy-domain` (PRD'd) |
| Anthropic Claude adapter (api + subscription) | DESIGN § 3.0 | `crates/oya-intelligence-adapter-claude-{api,subscription}` |
| Google Gemini adapter (api + subscription) | DESIGN § 3.0 | `crates/oya-intelligence-adapter-gemini-{api,subscription}` |
| OpenAI Codex/ChatGPT subscription adapter (subscription mode) | DESIGN § 3.0 (api mode lives now) | `crates/oya-intelligence-adapter-codex-subscription` |
| Stop-hook persistence loop equivalent (or per-provider equivalent) for Codex | Backup analysis Part B § B.5.5 | `crates/oya-intelligence-persistence-domain` |
| Critic / adversarial-verifier agent contract | Appendix A § A.7.2; this report § 3, 4.5 | `crates/oya-intelligence-critic-app` |
| Trace store + grading harness | Appendix A § A.7.13; backup analysis § 26.3 | `crates/oya-intelligence-trace-{kernel,app}` |
| Replay harness | Backup analysis § 26.3; Delivery Fabric PRD § 9 M2 | `crates/oya-intelligence-replay-app` |
| Cross-axis Merkle DAG evidence chain | Foundry top-20 #11; PRD § 5.1 | `crates/oya-platform-audit-chain-kernel` (cross-axis) + `oya-intelligence-evidence-app` |
| OpenTelemetry `gen_ai.*` semconv emission | Foundry top-20 #14 | `crates/oya-intelligence-telemetry-app` |
| Autonomy-tier ratchet (T0..T4 grant policy) | This report § 3, 4.7 | `crates/oya-intelligence-autonomy-domain` |
| Cohort-controller (per-provider concurrency reshape) | This report § 3, 4.3 | `crates/oya-intelligence-cohort-app` |
| Incident-grader / auto-postmortem agent | This report § 4.7 | `crates/oya-intelligence-incident-grader` |
| Release-captain pipeline | Delivery Fabric PRD § 8 | `crates/oya-intelligence-release-app` |
| Rust affected-graph CI lane planner | Delivery Fabric PRD § 5 | `crates/oya-governance-lane-planner` |

### 6.3 The triple alignment

These three artifact stacks now line up coherently:

```
┌──────────────────────────────────────────────────┐
│ This report (agentic dev cycle)                  │  the conceptual frame
│ — Goal / Plan / Lease / Verifier / Release-train │
└──────────────┬───────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────┐
│ agentic-delivery-fabric-executable-prd.md        │  the executable PRD
│ — ChangeSet / PatchSet / Pod / Lane / Evidence   │
└──────────────┬───────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────┐
│ docs/consolidated/products/foundry/PRD.md        │  the kernel struct definitions
│ — Capability / Step / Run / Evidence / Provider  │
└──────────────┬───────────────────────────────────┘
               │
               ▼
┌──────────────────────────────────────────────────┐
│ services/agent/daemon/src/foundry/*.rs           │  the existing Codex scheduler
│ — Issue / Run / Workspace / TokenLedger          │
└──────────────────────────────────────────────────┘
```

The bottom layer is real and live (Codex single-provider). Each higher layer is currently a doc artifact. The agentic optimization is to grow the live implementation **upward** through these layers, not start over at the top.

---

## 7. The non-stop-autonomous Claude + OpenAI + Gemini × API + subscription engine — v0 cut

Per the user's directive: a continuously running engine with three providers and two auth modes each (six adapters total) operating autonomously. This section names the v0 minimum-viable cut.

### 7.1 v0 in-scope

1. **Provider adapter trait + 6 adapters**:
   - `oya-intelligence-provider-kernel::ProviderAdapter` trait + `ProviderAuth` enum (already PRD'd)
   - `oya-intelligence-adapter-codex-{api,subscription}`
   - `oya-intelligence-adapter-claude-{api,subscription}`
   - `oya-intelligence-adapter-gemini-{api,subscription}`
   - SecretProvider binding (OpenBao per ADR-0043)
   - Per-tenant per-capability auth-mode selection
   - Failover policy: `prefer: claude-api → fallback: openai-api → fallback: gemini-subscription`

2. **Goal kernel + planner-app + Plan DAG kernel** (this report § 4.1, 4.3)

3. **AgentPodManifest schema + pod-domain validator** (Delivery Fabric PRD § 3.6)

4. **TaskLease kernel + lease-manager** with write-scope conflict detection (this report § 4.4)

5. **Per-pod ephemeral git worktree** (DESIGN § 3.0.5.2; explicitly **not tmux**) — direct PTY allocation per provider via `openpty`/`forkpty`

6. **Persistence loop**:
   - For Codex: native `Stop` `decision:"block"` + `reason` continuation (per OMX pattern documented in backup analysis Part B § B.5.5)
   - For Claude Code: same pattern via Claude Code's hook system
   - For Gemini: provider-specific equivalent (Gemini's CLI hook surface must be audited; if absent, an outer orchestrator polling loop is required)
   - Crash-safe: state is durable in `.omc/state/` + per-session subdirs; resume after process death without scrollback

7. **EvidenceBundle kernel + submit gate + verifier-agent** (this report § 4.5; Delivery Fabric PRD § 1.2 FR-3, FR-5)

8. **Adversarial critic-agent contract** mandatory for `autonomy_ceiling >= T3` (this report § 4.2)

9. **Stop-conditions and budgets**: token / wall-clock / cost / retry / child-agent budgets per Goal; cohort-controller reshapes per-provider concurrency continuously (this report § 4.3)

10. **OpenTelemetry `gen_ai.*` spans** for model/tool/lane/policy/approval events (Foundry top-20 #14)

### 7.2 v0 out-of-scope (deferred to v1+)

- Cross-axis Merkle DAG evidence chain (real, but spans more than Foundry; v0 emits Evidence locally with per-Run chain root, cross-axis chaining at v1)
- Sapling/Mononoke-style virtualized FS (Delivery Fabric Phase 6 decision)
- Speculative parallel dispatch (DESIGN § 3.0.5.2; nice-to-have v1)
- Macaroons-style capability tokens (Foundry top-20 #17; v1)
- LangGraph-style explicit state machine for agent runs (Foundry top-20 #10; v1)
- In-house model production (W-AI-Model-Substrate wave; long-horizon)
- Tenant-facing capability marketplace (W-Public-GA)

### 7.3 The "non-stop" guarantee — what it actually means

Three sub-guarantees, each with its own primitive:

1. **Per-Goal continuity**: a Goal does not pause on routine human input. The persistence loop (Stop-hook continuation or polling-orchestrator equivalent) ensures the agent keeps producing evidence until `goal.status` reaches a terminal state (`complete | failed | cancelled | escalated`) or the budget envelope is exhausted.
2. **Per-cohort scheduling continuity**: when one provider rate-limits or fails, the cohort-controller reshapes concurrency to other providers within the failover policy. The engine never *stops* because of a single provider outage; at worst it slows.
3. **Per-process crash safety**: the runtime resumes from durable state after process death without reading terminal scrollback. State is event-sourced + checkpointed; the resume protocol reads `goal_id + plan_id + lease_id + last_checkpoint` from disk.

If any one of those three breaks, the engine is *not* non-stop. v0 must close all three.

---

## 8. Recommended sequencing — first 90 days

This sequence assumes the existing `services/agent/daemon/src/foundry/` Codex scheduler is the live floor and the goal is to grow upward through the architecture stack while preserving the live Codex pilot.

### 8.1 Month 1 — Foundation kernels + 3 adapters

Week 1–2:
- ADR for the agent-centric cycle (Goal → Plan → Lease → Evidence → Verdict → Release)
- Schema definitions (frozen at v1) for: Goal, Plan, Lease, EvidenceBundle, PolicyVerdict, AgentPodManifest, ChangeSet, PatchSet, Stack
- `oya-foundry-{goal,plan,lease,evidence,policy,pod}-kernel` crates (no I/O; pure types)

Week 3–4:
- `ProviderAdapter` trait + `ProviderAuth` enum land in `oya-intelligence-provider-kernel`
- Codex API adapter migrated from existing `app_server.rs` to a kernel-conformant trait impl
- Anthropic Claude API adapter (initial)
- Gemini API adapter (initial)
- Adapter trait conformance test fixtures

Exit criteria for Month 1:
- All 9 kernel crates compile with zero I/O
- Codex API adapter passes adapter trait tests
- Claude API adapter passes adapter trait tests
- Gemini API adapter passes adapter trait tests
- Schema fixtures validate

### 8.2 Month 2 — Subscription adapters + persistence loop + pod manifest

Week 5–6:
- Codex subscription adapter (headless session via Codex CLI, isolated `CODEX_HOME` per run)
- Claude subscription adapter (headless session via Claude Code CLI, isolated `CLAUDE_CONFIG_DIR` per run)
- Gemini subscription adapter (audit Gemini CLI hook surface; build polling-orchestrator fallback if needed)
- Subscription token vault binding through OpenBao

Week 7–8:
- Persistence loop: Stop-hook continuation for Codex + Claude; polling-orchestrator fallback for Gemini
- AgentPodManifest validator + ownership coverage check
- TaskLease manager with write-scope conflict detection
- Per-pod ephemeral git worktree allocator (uses `git worktree add` under workspace root)

Exit criteria for Month 2:
- 6 adapters live (3 providers × 2 modes)
- Failover policy switches between providers when rate-limited or unavailable
- One Goal can run end-to-end through Plan → Lease → Patch → Evidence with two adapters concurrently
- Pod manifest blocks out-of-scope writes

### 8.3 Month 3 — Verifier + EvidenceBundle gate + release pilot

Week 9–10:
- Verifier-agent contract + verdict schema
- Adversarial critic-agent for `autonomy_ceiling >= T3`
- EvidenceBundle ingestion into verifier-app
- Submit gate: `repoctl submit check` evaluates verdict; `repoctl submit apply` requires `allow`

Week 11:
- Rust lane engine v0: cargo affected-graph + nextest + sccache metrics; emit EvidenceBundle
- Per-pod budget envelope enforcement (token / wall-clock / cost)
- Cohort-controller reshapes per-provider concurrency under rate-limit telemetry

Week 12:
- Release-captain pipeline pilot for one low-blast service
- Trace store + OpenTelemetry `gen_ai.*` spans
- Auto-postmortem agent stub (records traces; manual trigger; agent draft for first 5 incidents)

Exit criteria for Month 3:
- One pod ships a Rust change end-to-end without human approval (within autonomy ceiling)
- Two pods run concurrently with zero collision
- Critic verdict blocks at least one bad change in the trial window
- Replay-from-trace works for at least one failed run

### 8.4 The conservative scope — what's intentionally NOT in 90 days

- Cross-axis Merkle DAG evidence chain (M4–M5 of the Delivery Fabric PRD)
- jj/Sapling adapter (M4 of Delivery Fabric PRD; deferred until evidence justifies)
- Cluster runner (M7 of Delivery Fabric PRD; v0 runs local + ephemeral on dev box)
- Speculative parallel dispatch
- Tenant-facing capability marketplace
- W-AI-Model-Substrate (in-house models)
- KR/JP/etc regional packs

---

## 9. Open questions (for Phase 4 deep-interview)

The deep-dive's Phase 4 should crystallize these before any code lands:

1. **Goal granularity**: does a "Goal" map 1:1 with a GitHub Issue, or 1:1 with an OKR Key Result, or both? (The existing Foundry scheduler is Issue-keyed; the proposed Goal kernel is more abstract.)

2. **Pod cardinality v0**: how many pods does v0 need? One per crate-family (`pod_foundry_*`, `pod_corp_*`, `pod_logistics_*`, ...) or one global pod with capability-scoped autonomy?

3. **Provider failover policy authoring**: does each Capability declare its provider preference + failover chain, or does the cohort-controller resolve it per-Goal at dispatch time?

4. **Subscription auth UX**: subscription adapters require a one-time human auth (login flow). Where does that auth happen? `repoctl auth login --provider claude --mode subscription` per-machine? Per-tenant? Per-session?

5. **Critic agent's authority**: when the critic objects, can the planner-agent override and proceed (with audit trail), or does the critic's verdict halt the goal until human approval?

6. **Goal-vs-OKR cascade**: does a parent Goal automatically inherit the autonomy ceiling of its child Goals, or does each Goal carry its own ceiling and the runtime checks `min(parent.ceiling, child.ceiling)` at dispatch?

7. **The "non-stop" SLA**: what is the tolerated downtime / latency for the engine? Sub-second mean-time-to-resume? Per-provider 99.5% availability target? These shape failover and persistence-loop design.

8. **Existing in-flight Foundry work**: there is active development in `services/agent/daemon/src/foundry/`. Is the proposed `crates/oya-foundry-*-kernel` migration a fresh build alongside the daemon (with a cutover later), an in-place refactor of the daemon, or a parallel-with-bridge approach?

9. **Where does ChangeSet/PatchSet live**: under `crates/oya-intelligence-change-*` (treats VCS as Foundry domain) or under `crates/oya-platform-delivery-*` (treats VCS as a cross-axis platform concern)?

10. **Postmortem trigger**: is the auto-postmortem agent only for Sev1/Sev2 (current incident model), or for *every* failed Goal? If the latter, it shapes the trace store retention policy.

---

## 10. Risks and what could invalidate this analysis

| Risk | Mitigation |
|---|---|
| The "agentic inversion" is over-stated. Some PM ceremonies exist for *coordination* across humans, not *attention* — and replacing them silently fragments organizational knowledge. | Keep ADRs / design docs / postmortems as **human-readable artifacts even when agent-drafted**. The narrative survives even when the ceremony is automated. |
| The 6-adapter target underestimates subscription-auth UX complexity. Sessions expire, get logged out, hit quotas mid-run, and headless-CLI bot-detection actively fights re-auth. | Design adapters for graceful degradation: if subscription session fails, fall back to API mode for that capability invocation; record the auth failure as evidence; alert the operator. |
| The verifier-agent's quality bar is the only thing between agent autonomy and prod incidents. If the verifier is weak, autonomy ceilings have to stay low and the whole compression collapses. | Treat the verifier as the most-tested component; per-Capability verifier eval set with golden + adversarial cases; verifier-of-verifier independent agent for high-tier work. |
| Provider rate limits invalidate non-stop for any single-provider capability. | Multi-provider failover at v0; per-tenant provider preference declared; the engine never *fails* on a single provider, only slows. |
| The 90-day plan assumes 2–3 senior engineers full-time. Less, and Month 3 slips. | Scope-cut order: drop cohort-controller first, then auto-postmortem, then critic-agent (replace with simple human gate at T3+). Keep adapters + persistence loop + pod manifest + verifier non-negotiable. |
| Monorepo edits to `Cargo.toml [workspace.members]` serialize at the merge queue (per ADR-0015). 23 new kernel crates can't all land in week 1. | Phase the workspace member edits per Delivery Fabric PRD § 9 M0–M3; one PR adds at most 5 new crates; merge-queue serializes naturally. |
| Cedar policy authoring has its own learning curve and ADR overhead. | Start with hand-written allow/deny rules in the policy-domain crate; promote to Cedar at v1 when the policy surface is settled. |

---

## 11. Source anchors

This synthesis pulls from:

- `docs/raw/claude-code-backup-comprehensive-analysis.md` — Claude Code TS architecture teardown (Part A) + 3-repo cross-cutting study with OMC + OMX (Part B); 17 explicit "Foundry lessons"; Appendix A "Optimizing Claude-Code patterns for fully agentic autonomous work" with Goal/Plan/PolicyEnvelope/Lease/EvidenceBundle primitives.
- `docs/raw/agentic-delivery-vcs-cicd-report.md` — Strategic report on Rust-first agentic delivery fabric, hyperscaler benchmark (Amazon/Google/Meta/Microsoft), 7-phase delivery plan.
- `docs/raw/agentic-delivery-fabric-executable-prd.md` — Executable PRD with M0-M8 milestones, 6-plane architecture, typed schemas, CLI/API spec, 10 first implementation issues (ADF-001..010).
- `docs/consolidated/products/foundry/PRD.md` — Foundry agent-runtime + control-plane + foundry-platform PRD; Capability/Step/Run/Evidence/Provider/AutonomyCeiling kernel struct definitions; multi-provider authentication model (Anthropic/OpenAI/Gemini × subscription/API).
- `docs/consolidated/DESIGN.md` — Cohesion thesis, 7-axis layout, plane separation, Foundry-as-accelerator (§ 3), Foundry-internal sequencing (SecretProvider/KMS → adapters → daemon hardening → smoke lane → live pilot), automation-first pipeline (§ 3.0.5), per-agent worktree + merge queue + speculative parallel dispatch, "PTY not tmux" directive.
- `docs/consolidated/CONSTITUTION.md` — Seven axes (SaaS · Workspace · Vertical · Foundry · Cloud · Search · Ads + Analytics).
- `services/agent/daemon/src/foundry/` — Existing Rust scheduler: orchestrator (2049 lines), runner, workspace, providers, tracker, token_ledger, workflow_engine, http, app_server, embed.
- Public engineering doctrine (named in §1.2): Amazon Builder's Library (continuous delivery, hands-off deployments, release-captain pipeline), Software Engineering at Google chapters 22–25 (CI, static analysis, code review), SRE book (golden signals, error budget, blameless postmortem), Sapling/Mononoke docs (Meta), BuildXL / Build Accelerator (Microsoft), 12-Factor Agents doctrine (HumanLayer), OpenTelemetry GenAI semconv.

This document does NOT cite specific public articles inline because the patterns it names are industry consensus across the cited corpora; readers can pull from any of the source anchors to verify a given primitive.

---

## 12. Bottom-line recommendation

Re-shape Foundry into the agent-centric dev cycle described in §3–§4 by growing the existing Rust scheduler upward through the kernel stack in §6.3, with the 90-day cut in §8 producing the non-stop autonomous Claude + OpenAI + Gemini × API + subscription engine in §7. The gating decisions for Phase 4 deep-interview are the 10 questions in §9; everything else is sequencing.

The slogan from `agentic-delivery-vcs-cicd-report.md` already names the principle:

> *Agents do not ask to ship; they prove they are allowed to ship.*

This report's contribution is naming what "allowed" means for every phase of the dev cycle, and naming the Rust crate boundaries where each "allowed" check lives.
