# Critical Challenge: The Foundry / Agentic Delivery Doc Stack

**Date:** 2026-05-10
**Status:** Adversarial review; deliberately skeptical.
**Method:** Each doc is read as an advocate, then attacked at its load-bearing claims. The job here is to find what's wrong, not to validate.
**Scope:** Five documents form the current Foundry / Agentic Delivery design corpus —
- D1 `docs/raw/claude-code-backup-comprehensive-analysis.md` (3493 lines)
- D2 `docs/raw/agentic-delivery-vcs-cicd-report.md` (588 lines)
- D3 `docs/raw/agentic-delivery-fabric-executable-prd.md` (1025 lines)
- D4 `.omx/plans/test-spec-agentic-delivery-vcs-cicd.md` (144 lines)
- D5 `docs/raw/big-tech-dev-cycle-agentic-optimization.md` (this author's own synthesis; challenged here in self-defense)

This is intentionally unkind to all five documents, including D5. The goal is to surface every load-bearing assumption that fails before it costs an engineering wave.

---

## 0. The single most important meta-finding

**The corpus is a closed loop.** D1 was written first; D2 cites D1; D3 cites D1+D2; D4 cites D3; D5 cites D1+D2+D3. All five were authored by AI inside the same project window, all pointing at the same target (Foundry), all citing each other or the same upstream slogans. There is **no independent external validation step** anywhere in the loop. When five documents agree, that is not evidence — that is correlation. The corpus needs at least one externally-attested checkpoint (a customer interview, a competitive benchmark with measurements, a postmortem from an analogous system) before any milestone treats the corpus as authoritative.

A second meta-finding: the corpus contains **zero adversarial peer review of itself**. Every doc presents recommendations as inevitabilities. None contains a "we considered approach X and rejected it because Y, but X may turn out to be right" section that survives ratification. This very document is an attempt to fill that gap, but it is also AI-authored — so it is not the external attestation either.

---

## 1. Cross-cutting failure modes (apply to all 5 docs)

These are issues that recur across the corpus. Each is presented with the docs that exhibit it.

### 1.1 Slogan as evidence

> "Agents do not ask to ship; they prove they are allowed to ship." — D2
> "The model should not be the state machine; the model should be a participant in a state machine." — D1 (Appendix A.4.3)
> "One product, one control plane, two persona surfaces." — D3 quoting DESIGN.md
> "Capability records are the source of truth, not ad-hoc tool descriptions." — D1 §A.7.3
> "Foundry is the single highest-leverage investment in the v2 backlog." — quoted by D5

These are inspirational. They are not falsifiable. None of them — across thousands of lines — is grounded in a measurable claim of the form "X is happening today; we project Y; metric Z will move from A to B." The corpus mistakes a memorable phrase for a tested hypothesis. **No slogan should be load-bearing.**

### 1.2 Hyperscaler-as-authority bias

D2 §"Benchmark: union of hyperscaler strengths" names exactly four authorities: Amazon, Google, Meta, Microsoft. D5 inherits this in §1.2. The implicit claim is that the union of these four converges on the right answer.

Counter:
- **Stripe**'s engineering doctrine (consistent SDK, idempotency-first APIs, postmortems-as-changelogs) is widely cited but absent.
- **Shopify**'s monolith→cell architecture and "ShopifyQL" approach to internal tooling is absent.
- **GitLab**'s public handbook (genuinely public, unlike Amazon/Google internal practices) is absent.
- **JetBrains**, **Vercel**, **Netflix**, **Cloudflare**, **Datadog**, **Anduril**, **SpaceX** — all have differentiated practices, especially around small-team velocity, that get no mention.
- **Smaller/younger orgs that operate at adjacent scale to Bominal** (Ghost, Plausible, Linear, Supabase) get no mention either.

Picking the four largest companies as the "right" doctrine is **status-quo bias toward Bay-Area-megacorp practice**. It also imports their tradeoffs uncritically: the four cited companies all run with much higher headcount density, much larger eng-platform teams, and much higher tolerance for build-vs-buy than Bominal can afford for years.

### 1.3 No customer voice

The corpus talks about: agents, pods, TPMs, SREs, security reviewers, internal engineers, ISVs, plugin authors. **It never asks an actual paying tenant what they need from Foundry.** D3 §2 lists tenant personas in a table; their needs are inferred, not interviewed. The danger: Foundry is being designed as the developer-experience-of-Bominal first, and the customer surface is back-fitted. This is a leading indicator of products that ship and don't sell.

### 1.4 No competition section

The 2025–2026 agentic-developer-tools space includes: Devin (Cognition), Replit Agent, Sweep AI, Codegen, Aider, Cursor, GitHub Copilot Workspace, Anthropic's own Claude Code, Anysphere/Cursor's background agents, Continue.dev, Sourcegraph Cody, Tabby, Amazon Q Developer, Google Jules, Stackblitz Bolt. Several have explicit "agent fleet" or "parallel pod" features.

**None are named, benchmarked, or differentiated against in the corpus.** The recommendation is to build a system without measuring what already exists. This is either the right answer (and we should know why) or an expensive mistake (and we should know why).

### 1.5 No threat model

D3 §1.3 has security NFRs. D4 Group G has security tests. **Neither has a threat model with named adversary classes**:
- Compromised pod (insider risk)
- Stolen agent token / session token leakage
- Supply-chain injection in build cache or sccache
- Prompt-injection via tracker issue body
- Capability metadata poisoning (per D1 Part B's MCP-ITP citation, this is real)
- Trace-store leakage (traces contain prompt content, often with PII)
- Cross-tenant evidence-bundle confusion
- Cosign/Rekor key compromise
- Model provider compromise (Claude/OpenAI/Gemini side)

A delivery substrate that mutates code at machine speed is a **high-value target**. The corpus has authentication notes and least-privilege gestures, but no STRIDE/LINDDUN/threat-actor breakdown. v0 should not ship without one.

### 1.6 No cost model

Tokens cost money. Wall-clock costs money. Evidence storage costs money. Trace retention costs money. Build cache (CAS) costs money. Per-tenant cohorts cost money. **The corpus discusses budgets as runtime *limits* but never as *unit economics*.**
- D3 §4.1 lists "max_tokens: 5_000_000" per goal: at $3/MTok input, $15/MTok output that is $15–75 per Goal. Routine.
- D3 §4.4 says "max_parallel_runs: 4" per pod. With 50 pods × 4 parallel × 24 hours non-stop = 4800 concurrent runs. At even $5/run, $24,000/day burn. Sustainable only with revenue.
- Subscription-mode adapters mostly have no per-token billing, but they have **session caps** (Claude Pro has rate limits documented; ChatGPT Plus too). The per-Goal SLA is gated by the consumer plan's cap, not Foundry's logic.

A cost model needs to live next to the runtime model. v0 without a cost ceiling is a runaway-spend bug waiting to fire.

### 1.7 Conway's law unaddressed

Pods imply teams. Pods carry `owners.primary` and `owners.tpm`. The corpus doesn't say:
- How many pods does Bominal have today? (Per Lane 3: zero, since `crates/oya-foundry-*` doesn't exist.)
- How many can it staff?
- Who staffs `pod_foundry_repoctl` vs `pod_foundry_orchestrator` vs `pod_corp_payroll`?
- Who is the TPM today? (D2 talks about "TPM/Orchestrator" as if the role exists.)
- What does "platform-orchestrator" team actually do in this repo right now?

If the org chart can't sustain the proposed pod fleet, the autonomy model collapses to "one human + N agents," at which point the elaborate `tpm_required_if` ceremony is theater. The corpus assumes the org is bigger than it is.

### 1.8 The corpus is biased toward Rust + cargo-specific patterns

D2/D3/D4/D5 all assume `cargo`, `nextest`, `sccache`, `cargo deny`, `cargo machete`, `cargo audit`. Fine for Foundry's Rust crates. But Bominal also has TypeScript (`apps/`, `platform/sites/`), pnpm workspaces, Python, and at least one SvelteKit prototype. The "Rust-first agentic delivery" framing implicitly treats non-Rust code as second-class — which contradicts D5's own observation that Bominal has multi-language surfaces. **What does an "Agentic Delivery Fabric" do for a TypeScript change to `apps/oyatie-eac/`?** The corpus is silent.

### 1.9 "Non-stop autonomous" is a marketing phrase, not a system property

User direction was "non-stop autonomously." The corpus echoes this without operationalizing.

Real systems have:
- Provider rate limits → at peak, no provider may be available
- Subscription-auth session expiration → headless re-login fails detect-bot-traffic checks
- Network partitions
- Trace store backpressure
- Reasonable maintenance windows
- Off-hours scaling for cost

"Non-stop" with these realities means **at most 99.9%-of-clock with graceful degradation** — exactly what every real cloud runtime says. The corpus elevates "non-stop" to a brand attribute and never defines the SLA. v0 must define a tolerable non-availability budget or the term is decorative.

### 1.10 No falsifiable success criteria

What does "Foundry succeeds" mean? "Cycle compresses from 4-8 weeks to 24-72 hours" (D5 §5) — for what kind of change, with what failure rate, at what cost? "Affected verification P95 ≤ 10m" (D3) — fine, but only if measured against a defined workload (which is undefined).

The corpus has milestones (D3 §9 M0–M8), gates (D4 acceptance groups), and exit criteria (D2 phases). None of them is a **success-or-failure-of-the-whole-program** test. There is no answer to "if metric X stays below threshold Y for Z weeks, we kill this and revert to GHA."

---

## 2. Per-document critical analysis

### 2.1 D1 — claude-code-backup-comprehensive-analysis.md

**Strengths**: detailed source-code spelunking with specific file:line anchors; honest about backup branch limitations (no test harness, no package metadata); separates Part A (deep dive) from Part B (cross-cutting).

**Critical challenges**:

**2.1.1 Provenance and IP risk are under-acknowledged.**
D1 line 11: *"the target README describes the repository as leaked source. This report records architectural findings, workflows, guardrails, and clean-room design lessons. It intentionally does not reproduce source bodies, proprietary implementation text, or actionable bypass instructions."*

This disclaimer is doctrinally correct but operationally insufficient:
- Architectural diagrams reverse-engineered from leaked code can still create *clean-room contamination* risk if the same human/agent then implements the recommended Foundry features. The doctrine is to do clean-room implementations behind a wall (one team reads, another implements from a written spec only). Bominal has not declared this wall.
- "Inline source maps are a source-disclosure risk" — D1 §24 mentions this but does not name what Bominal will do if it inadvertently consumed any of those source maps in its analysis pipeline.
- Citation of `src/query.ts:1062` etc. couples Foundry's design language to leaked artifact line numbers. If the source tree is ever DMCA'd or re-private'd, all these anchors become dead links — *and the design rationale becomes uncitable.*

**2.1.2 "Foundry lessons" are pre-shaped, not discovered.**
Every major section ends with `Foundry lesson: …`. This is a template, not an emergent finding. The section structure literally inventories upstream features and converts each into a "lesson." If the same exercise had been done on a different system (e.g., Aider, Devin), 17 different lessons would have emerged. The lessons are not properties of Claude Code; they are the author's **prior architectural opinions, projected onto Claude Code's source.**

This isn't bad — it's a design technique — but it should be labeled as design, not analysis.

**2.1.3 Confidence ratings don't propagate.**
D1 §1 declares: high-confidence on topology, medium on feature-gated paths, low on test/build health. But §28 ("Foundry applicability") and Appendix A treat all recommendations as equally weighted. A reader cannot tell whether "tool-use/tool-result pairing" (high-confidence finding) and "skill-bundled hooks" (medium-confidence finding from feature-flagged code paths) carry the same weight in Foundry's roadmap. They probably don't.

**2.1.4 Counterfactual absence.**
D1 never seriously asks "what if Claude Code is wrong?" The closest is §24 risks ("monolithic files," "feature flag sprawl") — those are minor caveats. Major potential errors that go unexamined:
- *What if conversation-as-state was the right answer for Claude Code, and Foundry's "goal-as-state" inversion will discover the same problems with a different label?* Conversation has implicit context resolution that explicit goal objects must re-implement.
- *What if MCP normalization is over-architected for a 1-tenant runtime, and a simpler local-tool model would be faster?* MCP itself is contested.
- *What if "auto mode is not bypass" was a UX polish for human users, and autonomous systems don't need the distinction?* The whole T0..T5 ladder collapses if so.

**2.1.5 Appendix A's "12-Factor Agents" doctrine is assumed.**
The 12-Factor Agents framing (HumanLayer blog post) is one author's opinion, repackaged. So is "VeRO" and "Natural-Language Agent Harnesses" — these are 2025–2026 arXiv preprints. D1 §28 *itself* warns: *"Caveat: the following are mostly 2025-2026 preprints. Treat them as idea sources and experiment prompts, not settled standards."* But by Appendix A, this caveat is forgotten and the same sources are cited as architectural authority.

**2.1.6 Part B's three-repo cross-cutting study cites a methodology that does not allow reproduction.**
Part B claims 45+65+37 file reads via `gh api` by three Opus agents. The actual files read are not enumerated at file:line granularity (unlike Part A). The conclusions ("hooks-as-prompt-injection is the dominant pattern") are plausible but unverifiable from the doc itself. **A reader cannot tell where summary ends and synthesis begins.**

### 2.2 D2 — agentic-delivery-vcs-cicd-report.md

**Strengths**: clear thesis, named alternatives (Options A–E), explicit recommendation. Good use of phased delivery.

**Critical challenges**:

**2.2.1 "Git is the bottleneck" is asserted before measurement.**
D2 §"Why Git is the bottleneck" lists seven reasons (branch-name-as-coordination, rebase-hazardous, PRs flatten patchset lineage, etc.). These are real properties of Git. **They are not measured against Bominal's actual workflow.**

The corpus expects Phase 0 to "create measurement baseline" — but the decision to replace Git is already made before Phase 0 runs. The phasing is post-hoc rationalization. If Phase 0 finds that Git is responsible for 5% of delivery latency and 95% comes from CI duration, the entire Change Graph build is misallocated capital. That investigation has not been done; the doc treats the decision as obvious.

**2.2.2 jj/Jujutsu is presented as a settled choice.**
Jujutsu is **alpha-status software** as of 2026. Production migration of a 91-crate Rust workspace + ~580 modules+platform+services tree to a jj-as-local-ergonomics layer is a substantial bet. D2 presents it neutrally; it is not neutral. Risks:
- jj's data model is in flux; backward compatibility is not guaranteed.
- The git-compat layer has known edge cases around merges with conflicts.
- Tooling around jj (CI integration, IDE integration) is sparse.
- Most external contributors don't know jj.

A v0 that requires jj for agent ergonomics is a v0 with an alpha dependency. This should be flagged louder.

**2.2.3 Sapling/Mononoke is "long-term reference" — for what scale?**
Sapling/Mononoke was built for Meta's monorepo (tens of millions of files, billions of revisions). Bominal currently has ~580 source directories and a 91-crate target. Citing Meta's stack as Bominal's long-term reference is **scale-cargo-cult**. There is no operational reason a 91-crate workspace ever needs Sapling. Bringing it in as a reference creates an unstated invariant that everything else must be Sapling-compatible.

**2.2.4 "Replace GitHub Actions" is build-vs-buy in disguise.**
GHA is annoying. GHA is also $0.008/CI-minute (per `ubuntu-latest`), MIT-style integrations, and a workforce that knows it. **Replacing it costs:**
- Schedulers (Temporal, durable workflow runner)
- Runner pools (k8s ephemeral, IAM, observability)
- Artifact stores (CAS, retention, GC)
- Tracing infra
- Security operations on every layer above
- Onboarding cost for every external contributor

D2's argument is that GHA is "not the right core substrate for hyperscaler-grade Rust CI/CD velocity." That's a circular argument: it presupposes that Bominal needs hyperscaler-grade infra, which is the very question being decided. **The doc never asks whether GHA's actual marginal cost vs. owned substrate is favorable for Bominal's actual current load.**

**2.2.5 Phase 4 ("Own CI/CD substrate") is a multi-quarter platform engineering project.**
Phase 4 deliverables: durable workflow orchestrator, k8s runner pools, CAS/artifact/evidence store, lane scheduler with resource quotas, GHA status mirror, OTel trace export. Each is its own production-grade subsystem. Realistic estimate: 6–12 person-quarters. The corpus does not surface this honestly.

**2.2.6 "Hyperscaler union" rhetoric.**
"Amazon release safety + Google presubmit/trunk rigor + Meta stacked/monorepo ergonomics + Microsoft distributed deterministic builds." This sentence is the most-quoted in the corpus. It is also **engineering yes-anding**: collecting only the upsides of four companies and proposing a system that has all four. Real systems trade off. There is no hyperscaler union; there are four very different stacks each shaped by different scale, headcount, and acquisition history.

### 2.3 D3 — agentic-delivery-fabric-executable-prd.md

**Strengths**: typed schemas; CLI/API spec; M0–M8 milestone breakdown; first-10-issues list (ADF-001..010).

**Critical challenges**:

**2.3.1 Schema bloat and forward dependencies.**
D3 §3 defines 7 schemas: ChangeSet, PatchSet, Stack, EvidenceBundle, PolicyVerdict, AgentPodManifest, LaneDefinition. D3 §4 adds CiPlan, Release. D4 (test-spec) adds Lease, Event, ReleaseGates — bringing the v0 schema count to **12 typed durable objects**.

Each carries: schema versioning policy, fixture set (valid + invalid), validator, migration story, projection logic, persistence layout, query API, mutation API, idempotency key path, trace path. Conservatively, **each schema is 3–6 person-weeks of careful design**. Twelve schemas in v0 is 36–72 person-weeks of schema work alone, before any business logic. This is not flagged.

**2.3.2 "Independently testable" first-10 issues are sequentially dependent.**
ADF-001 (schemas) → ADF-002 (validate) → ADF-005 (ci plan) → ADF-006 (ci run) → ADF-007 (ChangeSet) → ADF-008 (Evidence) → ADF-009 (PodManifest) → ADF-010 (Git export). The doc claims "Each issue is independently testable and does not require production deployment" — true in unit-test terms, false in integration terms. Without ADF-001, ADF-002 has nothing to validate. Without ADF-007, ADF-008 has no patchset to attach evidence to. The order is implicitly serial; treating it as parallelizable yields a 10× planning overestimate.

**2.3.3 `deliveryd` is internally contradicted.**
D3 §2.2 component map: *"`deliveryd` durable scheduler/controller — later service; v0 can be file-backed/local."*
D3 §4.5 ("`deliveryd` HTTP API v0"): nine HTTP endpoints listed.
A "later service" with a v0 HTTP API listed is a contradiction. The HTTP spec presumes a service that the component map calls out as deferred.

**2.3.4 AgentPodManifest's `can_submit_if` mixes string and structured predicates.**
Sample value: `["blast_radius in [docs, local]", "all_required_lanes_green", "ownership_coverage == full", "no_cross_project_dependency", "no_open_sev1_or_sev2"]`.
These are pseudo-code strings. The PRD does not say:
- Are these parsed as Cedar policy?
- As Rego (Open Policy Agent)?
- As Rhai/Lua?
- As Rust functions registered by name?
- As string-match / SQL predicates?

This is the load-bearing decision for autonomy enforcement. Leaving it unspecified means every reader fills it in with their own preferred policy language, none of which is what ships.

**2.3.5 Rust-first focus excludes the rest of Bominal.**
D3 §1 enumerates "Rust CI lane planner/runner with affected-crate graph." **No analogous TS/pnpm/Python/SvelteKit lane is specified.** If a pod owns `apps/oyatie-eac/**`, Foundry has no lane for it. Either D3 declares non-Rust pods out of scope (and the org accepts a multi-substrate world) or D3 grows. Neither is stated.

**2.3.6 Conflict policy `merge_after_review` is theatre.**
D3 §3.2 PatchSet's `conflicts.status: none|detected|resolved`. D3 §6.4 conflict policy: `Conflict blocks submit, not change creation. Agent may attempt conflict resolution in its workspace. Shared ownership conflicts route to TPM/Orchestrator if ownership scopes overlap materially.`

The third option in the conflict-policy enum (introduced in agentic-delivery-fabric source's lease primitive: `fail_on_overlap | allow_readonly_overlap | merge_after_review`) implies an asynchronous human-mediated merge step. **This is the workflow the rest of D3 says we are escaping** ("agents do not ask to ship"). Either remove the option or flag it as escalation, but don't list it as a routine policy.

**2.3.7 No retention or GC policy.**
- EvidenceBundle: how long? Forever? Per-tenant? Sealed when?
- Trace: how long? Default OTel retention is 7-30 days; D3 says nothing.
- Cache: how long? sccache default?
- Replay fixtures: how long?

These aren't decoration — they are dollar-cost decisions and compliance decisions (PIPA right-to-erasure, GDPR DSR, etc.). D3's silence here is incompatible with Bominal's own constitutional posture on data residency and retention.

### 2.4 D4 — test-spec-agentic-delivery-vcs-cicd.md

**Strengths**: structured by acceptance group; covers schema/lane/policy/security separately.

**Critical challenges**:

**2.4.1 Group P refers to issues that don't exist yet.**
*"ADF-001 through ADF-010 each declare target paths, CLI, schema path where applicable, acceptance group, and dependency order."*
None of those 10 issue-declarations exist as canonical artifacts at the time of this review. Group P's acceptance is a forward reference. Either the issues are filed with full declarations, or Group P is unverifiable.

**2.4.2 No concurrency/throughput tests.**
The whole corpus's *raison d'être* is parallel agent dev. D4 has 16 groups (A–P). **Zero are concurrency tests** of the form: "100 agents in parallel, no collision, throughput ≥ X."
- Group J ("Change Graph consistency and concurrency") tests CAS conflicts on *one* changeset at a time.
- Group I ("Performance and reliability") has latency P95 targets but no concurrency target.

If parallel-agent throughput isn't tested, parallel-agent capability isn't verified.

**2.4.3 No chaos test.**
- No "pod killed mid-run; lease released; replacement worker dispatched" test.
- No "trace store goes offline; goal still progresses" test.
- No "provider hits rate limit; failover to alternate provider; goal still completes" test.
- No "cache poisoning attempt with valid signature but malformed content" test.

For a "non-stop autonomous" system, every fault path needs a test. D4 has security-policy tests (Group G) but no chaos-engineering tests.

**2.4.4 No upgrade/migration tests.**
Schema v1 will need to become v2. Does v0 ship without a migration story? D4 §A says "Reject unknown schema versions unless explicitly allowed" — that's enforcement, not migration. There's no test for *upgrading* a v1 ChangeSet to v2 ChangeSet without data loss.

**2.4.5 Trust roots are unspecified.**
*"Trusted runner cache write records identity/provenance."* What identity provider? What provenance format (in-toto attestation? SLSA L3? Cosign signature? Rekor entry?). The trust root isn't named.

**2.4.6 No human-loop tests.**
The corpus says human approval is needed at autonomy-tier boundary. But no test exercises a human-approval round-trip:
- Agent requests T3 uplift → escalation event emitted → human approves → agent resumes.
- Agent requests T3 uplift → human denies with structured blocker → agent escalates to TPM or terminates with evidence.

These are the **most error-prone interactions** in the system, because they fail asynchronously across days. Untested.

### 2.5 D5 — big-tech-dev-cycle-agentic-optimization.md (self-challenge)

D5 is this author's own work; the sharpest critique should be the most honest.

**Critical challenges**:

**2.5.1 The "agentic inversion" framing is rhetorically inflated.**
D5 §3 claims OKR/sprint/standup/retro exist *because* of human attention scarcity. That's a partial truth at best. They also exist for:
- **Cognitive batching**: stable scope is a quality lever for any actor, including agents. Agents that constantly re-plan may produce thrashier output than agents on a coherent batch.
- **Forward commitment**: standups create public commitment, which affects probability of follow-through. Agents *also* benefit from logged commitments, especially in adversarial-critic settings.
- **Cross-team coordination**: sprint boundaries align dependencies. Agents working continuously across teams risk losing this alignment.
- **Reflection**: retros aren't redundant when memory exists; they aggregate signals that no single trace contains.

Inverting them into "continuous flow with budget envelopes" loses these properties. The inversion table needs to be measured, not asserted.

**2.5.2 90-day plan is wishful.**
D5 §8 lists: 6 provider adapters + Goal/Plan/Lease/Verifier/EvidenceBundle/AgentPodManifest kernels + persistence loop + git worktree pool + lane engine + critic agent + replay harness + release pipeline pilot in 90 days. With three senior engineers, this is 270 person-days. By cross-comparison: Anthropic shipped Claude Code v0 with a much larger team over 12+ months. The plan is not scoped to reality.

**2.5.3 "Continuous flow" undercount of cognitive cost.**
Agents are not cost-free. Each agent run consumes tokens, retains traces, and emits events that humans must filter. Continuous flow at 100 pods × 4-parallel = 400 simultaneous traces is past human comprehension. D5 hand-waves "humans subscribe to dashboards" but doesn't define dashboard cardinality. **The true bottleneck may not be agent throughput — it may be human review throughput.** D5 ignores this entirely.

**2.5.4 The Goal kernel is over-engineered.**
D3's `Run.idempotency_key + initiator + capability_id + acceptance_criteria` could carry everything D5 calls a Goal. Adding a Goal kernel doubles the kernel surface for a marginal expressivity gain that may not be worth it. D5 §6 and §9 (Q1) flag this as a question; **the default answer should be "don't add Goal until you can show a concrete need that Run can't carry."** That's not what D5 recommended.

**2.5.5 D5 cites "industry consensus" without sources.**
Same charge D5 levied at D1: D5 §1.2 makes confident claims about Amazon/Google/Meta/Microsoft practice with no inline citations. "Source anchors" §11 lists categories, not URLs. **Same epistemological crime, this time mine.**

**2.5.6 D5 promises 4–8-week-to-24–72-hours compression with no measurement plan.**
§5's compression table is plausible but unfalsifiable. There is no plan to measure the baseline 4–8 weeks ("what change, on what surface, with what blast radius?"). The 24–72-hour claim is a marketing number. **Either D5 commits to a measurable baseline-to-target metric, or it withdraws the claim.**

**2.5.7 D5 ignores the legal/IP framing.**
The whole corpus is downstream of D1, which is downstream of leaked source. D5 should have called out the clean-room contamination risk that D1 understates. D5 didn't. That's a reviewer-class miss.

---

## 3. The five most consequential challenges (ranked)

Of all the issues in §1–2, these five can change a wave's outcome if mishandled.

**C1 (Severity: maximum). Clean-room IP risk from D1's lineage.** The entire 17-lesson architecture, Appendix-A primitives, and the autonomous-system shape derive from a leaked Anthropic branch. Any future Foundry implementation that rhymes with these structures may carry derivative-work risk. *Mitigation*: a written clean-room protocol where one team reads D1 and writes a Foundry spec from scratch, and another team implements only from the Foundry spec — never from D1.

**C2 (Severity: high). The corpus is a closed loop.** Five docs cite each other in a recursive pattern with no external attestation. *Mitigation*: at least one external benchmark (Devin / Replit Agent / Codegen capability comparison) and at least one customer-side requirements doc before D3 proceeds to implementation.

**C3 (Severity: high). The decision to replace GitHub Actions is unfunded.** D2 phases 4-5 are 6–12 person-quarters of platform engineering, presented as routine milestones. *Mitigation*: Phase 0 baseline must complete before any Phase 4 commitment. If GHA's actual delivery-latency contribution is small, the Phase 4 budget redirects elsewhere.

**C4 (Severity: high). Schema bloat in D3.** Twelve typed durable objects in v0 is 36–72 person-weeks of schema work alone. *Mitigation*: collapse to four (Goal-or-ChangeSet, Lease, Evidence, PolicyVerdict); defer Stack, AgentPodManifest, LaneDefinition, ReleaseGates, Event, CiPlan, Release to v1 or merge into existing kernels.

**C5 (Severity: medium-high). No threat model + no cost model.** Either could become the failure that ships v0. *Mitigation*: STRIDE + cost model are pre-requisites for the M0 milestone. They are not optional sections.

---

## 4. What survives the challenge

Not everything is broken. The following findings hold up under scrutiny:

**S1. Lane 1's capability-runtime gap is real.** No typed CapabilityDefinition, no policy sandwich, no paired tool-result invariant, no autonomy-tier enforcement — these are *absences confirmed by line-level reading* of the existing Foundry source, not corpus echoes. Lane 1's top-5 recommendations (capability-kernel, policy sandwich, autonomy-tier dispatch gate, typed Hook ABI, write-scope lease) are concrete and verifiable.

**S2. Lane 2's orchestration & isolation gap is real.** No git worktree per task, no write-scope lease conflict detection, no DAG-aware scheduler, no working `cancel()`, no blocking verifier gate — also confirmed by line-level reading. Lane 2's 7-plane Rust crate sketch is technically reasonable.

**S3. Lane 3's premise audit is correct.** `crates/oya-foundry-*` does not exist. The user's framing has at least two independent ambiguity axes. Phase 4 cannot proceed without explicit user clarification on (A) which Foundry surface and (B) which optimization target.

**S4. Multi-provider × multi-auth is structurally important** (3 providers × 2 auth modes). PRD § 5.1 already specifies the `ProviderAdapter` trait + `ProviderAuth` enum cleanly. This is the strongest part of the existing Bominal docs and survives all challenges.

**S5. ADR-0023 (Wasmtime/Firecracker sandbox) is structurally stronger than D1's per-shell-language parsers.** Lane 3 caught this; the corpus weakly cites D1's lesson 5; it should be *replaced* by the ADR-0023 posture, not translated.

**S6. Per-agent worktree isolation (DESIGN §3.0.5.2) is the single highest-leverage parallel-dev primitive.** Both Lane 2 and D5 §8 rank this first; the existing Bominal directive (PTY not tmux) is correct.

---

## 5. Recommended adjustments before any code lands

Ordered by reversibility (most-reversible first):

1. **Add a clean-room protocol ADR** before any Foundry kernel crate is authored. This is the cheapest fix and the biggest legal-risk reducer.
2. **Run D2's Phase 0 baseline measurement first**, before committing to Phase 1+. If Git/GHA contribute ≤25% of delivery latency, redirect platform investment.
3. **Collapse D3's 12-schema v0 to 4** (Goal-or-ChangeSet, Lease, Evidence, PolicyVerdict). Defer the rest.
4. **Add a STRIDE threat model and a unit-economics cost model to M0.** Block M0 exit if either is incomplete.
5. **Define the policy DSL.** Choose Cedar, Rego, Rhai, or hand-rolled Rust — pick one before any policy field ships.
6. **Define an SLA for "non-stop autonomous"** (e.g., 99.5% per-cohort availability with N-provider failover). Without it, the term is decorative.
7. **Add at least one external benchmark** (e.g., 100-task Devin/Replit/Cursor head-to-head on a frozen Foundry-relevant task set).
8. **Add at least one tenant interview transcript** to the corpus before D3 hardens further.
9. **Run Lane 3's user-clarification question** (premise audit Q-J1 + Q-J2) before any other Phase 4 work proceeds.
10. **Add concurrency/chaos/upgrade tests to D4** Group I and Group J.

---

## 6. The most uncomfortable single observation

The corpus reads like a startup pitch, not an engineering plan. Pitches optimize for momentum and conviction; engineering plans optimize for falsifiability and graceful degradation. The corpus has the slogan, the chart, the tier model, the milestone list, and the named-hyperscaler doctrine — but it lacks the things engineering reviewers need to refute it: a defined baseline, a measured comparison, a threat model, a cost model, a kill criterion, and a customer voice.

Foundry is too important to ship behind a pitch. The corpus needs a reviewer that says no — repeatedly — until the load-bearing claims are measured. That reviewer should not be inside the same closed loop that authored the corpus.

---

## 7. What this critique does not claim

For honesty: this critique is itself AI-authored, in the same project window, against docs that are AI-authored. **It is not the external attestation the corpus needs.** It is at best a heuristic adversarial pass. The recommended fix in §5 #7 (external benchmark) and §5 #8 (customer interview) cannot be done by any agent; they require human work. Any engineering wave that proceeds without those two pieces is taking the corpus on faith.

The single most useful thing the user can do next is **disambiguate the premise** (Lane 3 Q-J1 + Q-J2) and **approve or deny §5 items 1–10 individually**. The deep-dive Phase 4 interview should drive both.

---

## 8. The validator and the autoresearch result are theatrical

After §1–7 was authored, two additional artifacts came to light that deserve their own challenge: the autoresearch validator script and its `result.json`. Reading them changes the corpus's claim of "validated executable PRD" from *signal* to *theatre*.

### 8.1 The artifacts

- `.omx/specs/autoresearch-agentic-delivery-vcs-cicd/validate-executable-prd.py` (62 lines)
- `.omx/specs/autoresearch-agentic-delivery-vcs-cicd/result.json` (`status: passed`)

The result.json carries `"passed": true` and `"summary": "Executable PRD/spec is complete enough for implementation planning"`. That is a strong claim. The script that produced it is a 62-line Python file whose entire validation logic is:

```python
# excerpted, paraphrased for length
missing_headings = [h for h in required_headings if h not in text]
missing_terms    = [t for t in required_terms if t.lower() not in text.lower()]
issues           = re.findall(r'ADF-\d{3}', text)
test_groups      = re.findall(r'^## Group [A-Z]', test_text, flags=re.M)
urls             = re.findall(r'https?://[^)\s,]+', text)
passed = (
    prd.exists() and test_spec.exists()
    and not missing_headings and not missing_terms
    and len(set(issues)) >= 10
    and len(test_groups) >= 17
    and len(urls) >= 15
)
```

That is the entirety of what the validator does. Five regex checks plus two file-existence checks. **No schema parser. No JSON-Schema/OpenAPI/Cedar/Rego/Rust syntax check. No internal-consistency check. No referential-integrity check. No semantic check at all.**

### 8.2 What this implies about the corpus's "passed" status

The validator passes iff the PRD contains certain headings, certain bag-of-words terms, ≥10 ADF-* IDs, ≥17 `## Group X` lines in the test-spec, and ≥15 URLs. It is a **completeness checklist disguised as a validator**. A doc that contains every required string in random gibberish would pass.

Concretely, the validator does not detect any of the following:
- ADF-001 declaring target paths that do not exist
- ADF-005 referencing a `repoctl ci plan` command whose argument signature contradicts the API spec in §4
- A test group that asserts behavior the PRD does not specify
- A schema's `id` field having a different format in PatchSet than in ChangeSet
- A URL that 404s
- The same field declared with different types across two schemas
- An issue (e.g., ADF-007) listed in §13 but missing from the M0–M8 work breakdown
- A referenced ADR (e.g., ADR-0023) that doesn't exist
- A required heading present once with a typo and once correctly
- The PRD claiming a feature the test-spec doesn't test

Every one of these is the kind of bug a real validator would catch. None is caught.

### 8.3 The result.json is also stale

`result.json` reports `test_group_count: 17`. The current `test-spec-agentic-delivery-vcs-cicd.md` has Groups **A through P, which is 16 groups** (count: A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P). The discrepancy means the result.json was generated against a prior version of the test-spec that has since been edited.

This makes the "passed" claim doubly suspect:
1. It was passed by a validator that checks shape, not substance.
2. It was passed against artifacts that have since changed.

The result.json should be regenerated on every PRD/test-spec edit; it isn't, and the corpus has no convention that says it must be. **A `passed: true` artifact in version control with no freshness guarantee is a lie generator.** Future readers will see the green checkmark and assume validation; they will not see that the underlying validator was a regex grep, and they will not see that the artifacts have drifted.

### 8.4 The required-terms list is itself a leak about over-engineering

The validator's `required_terms` list is informative even outside its validation function — it is a de-facto inventory of vocabulary the PRD must contain to be considered "complete." Read as an inventory:

```
ChangeSet, PatchSet, EvidenceBundle, AgentPodManifest, LaneDefinition,
oya.delivery.event.v1, oya.delivery.lease.v1, oya.delivery.release-gates.v1,
frozen_evidence_id, policy_verdict_id, protected_head_moved,
idempotency-key, expected-sequence, prev-event-hash, lease-id, fencing-token,
repoctl ci plan, repoctl submit check, cargo nextest, sccache, jj,
GitHub status mirror, TPM, rollback, cache hit, affected graph,
POST /delivery/v1/submit/apply
```

This list mandates **at least 11 distinct typed objects** (5 named + Event/Lease/ReleaseGates schema versions + idempotency/sequence/hash/lease/fencing-token primitives that imply Lease + Event aggregates + a frozen-evidence-id flow + a policy-verdict-id flow). It mandates two adapters (jj, GitHub) and one shell tool (sccache). It mandates a specific HTTP route (`POST /delivery/v1/submit/apply`). It mandates `prev-event-hash` (cryptographic chaining of events) and `fencing-token` (distributed-systems lease fencing per Kleppmann's writing).

This is **a lot of vocabulary for a v0**. Each term in this list is a doc requirement (must be mentioned) and an implicit implementation requirement (probably must exist as a typed primitive). Cryptographic event-chain + fencing tokens for distributed leases is **production-grade distributed systems work**, not v0 surface. The vocabulary list locked in by the validator is structurally pre-committing the v0 to be substantially heavier than its description in §0 ("MVP scope") admits.

This is the most-revealing single artifact in the corpus. The validator is not just shape-only; it has **smuggled in a much larger v0 scope through the back door of "required terms."** If the corpus says v0 is small but the validator-of-record requires fencing-token semantics, the corpus is internally inconsistent about its own scope.

### 8.5 The validator script's own quality

Minor but indicative findings about the script itself:
- It hard-codes an `>= 17` test-group count (drift bait).
- It hard-codes an `>= 15` URL count, which incentivizes URL bloat (cite-everything-to-pass).
- It uses `re.findall(r'ADF-\d{3}', text)` and then `len(set(...))`, but does not check ADF IDs are dense (1..10) — `["ADF-007", "ADF-099", ..., "ADF-100"]` would pass the count test with gaps.
- It does not check that referenced ADRs in the PRD body actually exist on disk.
- It is not version-controlled with the artifacts it validates; an artifact edit can pass against an outdated validator (which itself is what produced result.json's stale 17 count).
- It exits 0 on pass and 1 on fail, but is not wired into any CI lane that would block merge on regressions; the result is a check that runs once, freezes its output, and is never re-run.

### 8.6 What this changes about §1–§7

The validator's existence weakens, not strengthens, the corpus:

- C1 (clean-room IP risk) is unchanged.
- C2 (closed-loop authorship) is **strengthened**: the validator was authored by the same loop and "passed" by the same loop, with no external check.
- C3 (replace-GHA cost) is unchanged.
- C4 (schema bloat) is **strengthened**: the validator's `required_terms` list locks in even more schemas/primitives than §2.3.1 inferred.
- C5 (no threat / cost model) is unchanged.

The most important update: the corpus's apparent "validated" status is a single-snapshot regex pass. Anyone reading the corpus and concluding "this is ready for implementation planning because result.json says so" is being misled by green-checkmark theater.

### 8.7 Recommended fixes for the validation pipeline

Ordered by cost-to-implement:

1. **Re-run the validator now** and accept either the new pass or the new fail. Stop carrying a stale result.json.
2. **Add the validator to a CI lane** that re-runs on every PRD/test-spec/script change.
3. **Replace the regex term-list with structural validation**: parse the PRD's YAML schema blocks, verify each schema is valid YAML and conforms to a meta-schema (JSON-Schema-of-schemas), verify cross-references (every `change_id` field in PatchSet references the same type as ChangeSet's `id`, etc.).
4. **Add referential-integrity checks**: every ADF-XXX referenced in §13 must be defined in §13 with target_paths/CLI/schema_path/acceptance_group/dependency_order; every ADR-XXXX referenced must exist on disk; every URL in `Source anchors` must return non-404.
5. **Replace `>= N` count thresholds with semantic checks**: instead of "≥17 test groups," check that every PRD-defined acceptance criterion has at least one matching test group; instead of "≥10 ADF issues," check that the M0–M8 work breakdown's tasks are all covered by ADF-XXX issues.
6. **Add a "scope sanity" lane**: assert that the v0 surface (per §0 MVP) does not exceed the agreed-budget complexity. Today's validator does the opposite — it rewards adding more vocabulary.
7. **Track validator authorship and require an external review** of any new `required_term` or `required_heading`. Without this gate the validator can be edited to pass anything.

### 8.8 The single most uncomfortable observation about the validator

The validator's `required_terms` list contains both `jj` and `cargo nextest` as required strings. This **mandates** that the PRD mention these specific tools to be considered complete. A validator that requires the PRD to name a specific (alpha-status) VCS frontend and a specific test runner has stopped being a validator and has become a **doctrine enforcer**. The corpus has loaded its preferred tools into the validator, and any future PRD that proposes alternative tools will fail validation — not because it's wrong, but because it doesn't say `jj`.

The validator is not a quality gate. It is a doctrine gate. The corpus has gone from "we recommend these tools" to "the PRD must mention these tools to pass" without the doctrine ever being explicitly ratified.

That is the most-consequential finding in §8. **A validator that hard-codes specific tool names into its required-terms list is locking in tool choices outside any ADR.** Every adoption / rejection of `jj`, `sccache`, `cargo nextest`, and the `repoctl` command names should go through ADR review, not be enshrined in a Python regex check.

---

## 9. The second validator: D1 itself is "validated" by URL-count theatre

A second autoresearch directory was found in the same `.omx/specs/` tree: `autoresearch-agentic-dev-best-practices/`. It contains four files: `mission.md` (11 lines), `sandbox.md` (3 paragraphs), `validate.py` (53 lines), and `result.json` (`status: passed`). This validator does not validate the executable PRD; it validates the **original analysis doc D1** itself — specifically D1 §26 ("2026 external best-practice cross-check"). The corpus's most foundational document was passed by a regex grep.

### 9.1 What the second validator actually checks

```python
# excerpted, paraphrased for length
required = [
    '## 26. 2026 external best-practice cross-check',
    '### 26.1 ... no latent subagent channel',
    '### 26.2 Token-saving and context-economy findings',
    '### 26.3 Harness and evaluation findings',
    '### 26.4 Production architecture findings',
    '### 26.5 Guardrail and security findings',
    '### 26.6 Consolidated best-practice checklist',
    '### 26.7 arXiv idea scan: additional leads to mine',
]
required_terms = [
    'prompt caching', 'cache hit', 'context compaction', 'codebooks',
    'trace', 'eval', 'OpenTelemetry', 'capability', 'guardrails',
    'least privilege', 'MCP', 'OWASP', 'NIST', 'CSA', 'CISA', 'arXiv',
]
urls = re.findall(r'https?://[^)\s]+', text[text.find('## 26.'): text.find('\n# Foundry applicability\n')])
arxiv_links = [u for u in urls if 'arxiv.org/abs' in u]
passed = (target.exists() and not missing and not missing_terms
          and len(urls) >= 30 and len(arxiv_links) >= 15)
```

Eight required headings. Sixteen required keywords. ≥30 URLs in section 26. ≥15 arXiv links.

That is the entire validation contract for the doc that grounds the corpus.

### 9.2 What this validator certifies and what it does not

The validator certifies that D1 §26 contains specific section headings, certain keywords, and a sufficient quantity of URLs. **It does not certify any of the following**:

- That a single cited URL contains the claim it is asserted to support.
- That any cited arXiv paper is peer-reviewed (most are preprints, as D1 itself admits).
- That the cited papers actually argue for the position D1 attributes to them.
- That contradictory sources have been considered.
- That sources are recent (the validator does not check publication dates).
- That sources have not been retracted or superseded.
- That summaries are faithful to the originals.
- That the citation density is appropriate (an over-cited paragraph and an under-cited paragraph pass identically as long as the section URL count exceeds 30).

The validator's "passed" claim therefore means: **the doc is shaped like a research review.** It does not mean the research is correct.

### 9.3 The validator REWARDS what it should DISCOURAGE

A useful validator would penalize:
- Single-source claims (over-reliance on one author)
- Stale citations (>3 years old in a fast-moving field)
- Unbalanced citations (only "for" sources, no contrary view)
- Self-citation loops (citing one's own prior work as authority)
- Fashion words ("agentic," "autonomous") used without grounding

This validator does the opposite. It rewards:
- **High URL count** — more URLs = passing. This incentivizes citation bloat.
- **High arXiv-link count** — ≥15 arXiv preprints required. This incentivizes preprint-as-evidence even though D1 itself warns "treat them as idea sources, not settled standards."
- **Specific tribal vocabulary** — `MCP`, `12-Factor` (implied via section structure), `OWASP`, `OpenTelemetry`. A doc that disagrees with this vocabulary cannot pass.

The required-keyword list smuggles in commitments. `MCP` is not an industry-neutral term — it is an Anthropic protocol. By mandating its presence as a passing criterion, the validator pre-commits the corpus to an Anthropic-aligned architecture without an ADR.

### 9.4 The mission.md and sandbox.md do not constitute a research methodology

`mission.md` (11 lines): names four research topics, points at the output file, points at the validator. Does not name a methodology, an evidence standard, a contrarian-source obligation, or a peer review.

`sandbox.md` (3 sentences): *"Sources used include OpenAI, Anthropic, Google ADK, LangGraph, OpenTelemetry, NIST, CSA, OWASP, CISA/NSA/Five Eyes guidance, AWS Strands Evals, HumanLayer, 12-Factor AgentOps, and selected 2026 arXiv harness/cache papers."*

This list flattens authorities of wildly different tiers as if peers:
- **Standards bodies** (NIST, CSA, OWASP, CISA, NSA, Five Eyes) — high authority, peer-reviewed, slow-moving.
- **Vendors** (OpenAI, Anthropic, Google ADK) — first-party authority on their own products only.
- **Open-source projects** (LangGraph, OpenTelemetry) — community authority, evolving.
- **Industry frameworks** (AWS Strands Evals) — vendor-aligned guidance, not standards.
- **Blogs** (HumanLayer's "12-Factor Agents" post) — single-author opinion pieces.
- **Preprints** ("selected 2026 arXiv harness/cache papers") — non-peer-reviewed.

These are six distinct evidence tiers. Treating them as a single "Sources used" pool **launders blog opinions to standards-equivalent authority**. The validator then passes any doc that mentions enough of them — making the laundering self-certifying.

### 9.5 100% pass rate across all autoresearch missions

There are exactly two `result.json` artifacts in `.omx/specs/`:
- `autoresearch-agentic-delivery-vcs-cicd/result.json` → `passed: true`
- `autoresearch-agentic-dev-best-practices/result.json` → `passed: true`

Both are "passed." A validation pipeline that has never produced a failure is not a validation pipeline — it is a stamp. Either every research mission is consistently brilliant (improbable for any process), or the validators are designed to pass. Combined with §8.4's finding that the first validator hard-codes scope-expanding vocabulary, and §9.3's finding that the second validator rewards citation bloat, the pattern is: **these validators measure compliance with the corpus's own preferences, not quality.**

### 9.6 Six unconnected deep-interview specs in the same tree

The same `.omx/specs/` directory contains six deep-interview artifacts that are **not referenced anywhere in the agentic-delivery corpus**:
- `deep-interview-github-issue-priority-pipeline.md`
- `deep-interview-consolidated-ultragoal.md`
- `deep-interview-clean-chatgpt-share.md`
- `deep-interview-prioritized-parallel-roadmap-sequence.md`
- `deep-interview-agentic-issue-pipeline.md`
- `deep-interview-ug-0-authority-status-baseline.md`

Three are topically adjacent: `agentic-issue-pipeline`, `prioritized-parallel-roadmap-sequence`, `consolidated-ultragoal`. None is cited from D2, D3, D4, or the validators. None of the agentic-delivery specs cite these. This means there is **a parallel specification track in the same artifact tree** that has not been reconciled with the corpus. Whatever those specs say — about issue prioritization, parallel roadmaps, or the ultragoal — is potentially in conflict with the D2/D3/D4 stack and silently unaddressed.

A corpus that cannot see its own siblings cannot claim to be a coherent design document.

### 9.7 The compounding theatre

§8 found one theatrical validator. §9 finds a second. The corpus now has:
- D1 (analysis) — passed by URL-count + keyword-salad regex.
- D3 (executable PRD) — passed by required-headings + required-vocabulary regex.
- D2, D4, D5 — not validated at all.
- Six deep-interview specs in the same tree — not cross-referenced, not validated against the corpus.

Composition: a "validated" corpus where two of five docs have ceremonial validators, three have none, and six related specs are floating. **The validation gate is not a gate. It is decoration.**

### 9.8 What §9 changes about §8 and §1–§7

C2 (closed-loop authorship) is now **strongly confirmed**. The validators are written by the same authoring loop, validate against criteria the loop chose, and have a 100% pass rate. There is no external check anywhere in the pipeline.

C5 (no threat / cost model) — strengthened. The autoresearch sandbox.md, which is the closest the corpus comes to a "research provenance" doc, is three sentences long. There is no mention of threat-modeling effort, cost-modeling effort, or even a list of *rejected* sources.

A new finding (C6, severity: high): **the corpus contains a parallel deep-interview track in `.omx/specs/` that the agentic-delivery corpus does not reconcile against**. Whatever the deep-interviews concluded about issue pipelines, parallel roadmaps, or the ultragoal is not present in D2/D3/D4. Either those specs are stale (in which case they should be retired) or they are authoritative (in which case the corpus must reconcile). Both options require human intervention.

### 9.9 Recommended fixes (additional, beyond §5 and §8.7)

11. **Retire both result.json files** until the validators are rewritten to do semantic validation. A green checkmark from a regex grep is worse than no checkmark; it misleads.
12. **Audit `.omx/specs/` for parallel/conflicting specs** and either retire deep-interview-* files that are stale, or fold their content into the agentic-delivery corpus with explicit reconciliation.
13. **Add a "research provenance" template** that includes: sources tier-classified (standards / vendor / OSS / blog / preprint), specific quotes supporting each claim, contrary sources considered, sources rejected with reason, and a peer reviewer name. The 3-sentence sandbox.md is not a research provenance.
14. **Mandate failures**. Add at least one *negative test* per validator — a synthetic input that should fail. If the validator never fails, retire it. The same convention applies to any future validator added in this tree.
15. **Move tool-name terms (`jj`, `sccache`, `cargo nextest`, `MCP`)** out of validator required-term lists and into ADR-of-record. If they're load-bearing tool choices, they need ADRs. If they're casual examples, they don't belong in a validator.

### 9.10 The single most uncomfortable observation about §9

The second validator's `required_terms` includes both `MCP` and `12-Factor` (implied by mandating section 26.7's preprints). The corpus has therefore enshrined **two specific external doctrines** — Anthropic's Model Context Protocol and HumanLayer's 12-Factor Agents blog post — as required vocabulary in a document that grounds Foundry's architecture. Neither is a standard. Neither has had ADR review. **Both are now load-bearing in the corpus by the side door of "validation."**

That is the observation that should disturb most: **the corpus's deepest commitments are not in its ADRs, but in the regex tables of its validators.** When the validators are deleted, the commitments evaporate. When the commitments stay, the corpus has accepted the doctrine without ever ratifying it. Both options are a governance failure.

---

## 10. Final synthesis after all challenges

After §1–§9, the operational picture:

- The corpus has **structural soundness** in places (Lane 1 + Lane 2 findings; multi-provider × multi-auth model; per-agent worktree directive).
- The corpus has **structural unsoundness** in places (closed authorship loop, IP-derivation risk, cost/threat model absence, schema bloat, validator theatre, parallel-spec-track conflict, vendor-doctrine smuggled in via validators).
- The corpus has **no falsifiable success criteria** at the program level.
- The corpus has **a 100% green validation rate** that is mechanically incapable of producing red.

The corpus is not ready to drive an engineering wave as-is. It is ready to drive a **clarification wave** — a focused round of human work that:
1. Disambiguates the premise (Lane 3 Q-J1 + Q-J2).
2. Replaces validator theatre with semantic validation (§9.9).
3. Reconciles `.omx/specs/` deep-interview track with the agentic-delivery corpus (§9.6).
4. Adds threat model, cost model, and customer voice (§5 #4, §5 #7, §5 #8).
5. Establishes a clean-room IP protocol (§5 #1).

If those five steps complete, the corpus's good parts (Lane 1, Lane 2, multi-provider, worktree, lane engine, evidence bundle) survive into a real plan. If they don't, every wave that proceeds from the corpus will be building on validators that cannot fail, slogans without metrics, and citations that may not say what they're claimed to say. **That is the failure mode worth preventing.**
