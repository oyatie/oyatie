# 01 — INTERVIEW AGENDA (founder questions raised by the routing + governance verdict)

Synthesizer: workflow-subagent · Date: 2026-06-06 · Source: `00-ROUTING-VERDICT.md` + verified source trees.
Each question states the decision at stake, the options, the evidence, and a recommendation. Founder makes the call.

---

## Q1 — Confirm the foundry-platform home is `oya/intelligence`, not `cloud/cloud-intelligence`

**Decision at stake:** Where the absorbed foundry AI-agent-PLATFORM officially lives.

**Evidence:** 7/7 platform primitives (model-routing-policy, capability registry, MCP gateway, eval harness, autonomy ceiling, guardrails, RAG, attribution, supervisor/kill-switch) are real crates in `oya/intelligence` (~96k LOC, 128 crates). cloud-intelligence has 0/7 and its PRD disclaims them by name (PRD.md:49, :80). `_legacy-foundry/` + 357 `oya-foundry-` references confirm the lineage lives in intelligence.

- **Option A (recommended):** Ratify `oya/intelligence` as the foundry-platform home; cloud-intelligence is NOT the platform.
- **Option B:** Move platform primitives into cloud-intelligence (founder's lean). Cost: would require rebuilding 96k LOC of product substrate into an infra-shaped gateway whose own PRD rejects these surfaces.

**Recommendation: Option A.** The founder's "cloud-intelligence" instinct is correct only for the egress/wire slice (see Q2), not for the platform. This is settled by code, not opinion.

---

## Q2 — When intelligence wires live provider calls, does it go DIRECT or THROUGH cloud-intelligence?

**Decision at stake:** The single real overlap between the two services — who owns the provider wire I/O, and how intelligence's deferred live calls get wired. This is the only place the two services can drift into true duplication.

**Evidence:** intelligence's provider api-adapters are mocks behind a flag (10 deferral-marker files); its CLI drivers spawn subprocesses but stub drain/inject/kill. cloud-intelligence has a REAL wired AnthropicAdapter + a built-but-unwired CodexAdapter (`rest/src/lib.rs:30,369`), and its entire purpose (ADR-0384) is credential pooling + metering + circuit-breaking at one egress chokepoint. Today both carry their own partial Anthropic/Codex adapters.

- **Option A (recommended):** intelligence calls providers THROUGH cloud-intelligence. intelligence's deferred adapters become thin clients of the gateway; all wire I/O, credential pooling, metering, and circuit-breaking happen once, at the cloud-intelligence chokepoint.
- **Option B:** intelligence calls providers DIRECTLY with its own adapters; cloud-intelligence serves only other (non-intelligence) callers. Cost: two parallel provider-adapter stacks, two metering paths, duplicated OAuth/credential logic — the duplication risk made real.

**Recommendation: Option A.** Makes the dependency direction explicit (intelligence = policy plane → cloud-intelligence = egress plane), honors cloud-intelligence's single-chokepoint mandate, and retires intelligence's mock adapters into gateway clients instead of competing implementations.

---

## Q3 — Is `oya/governance` kept, folded, or shelved — and how is it classified in the service inventory?

**Decision at stake:** Governance's status as a service.

**Evidence:** governance is 0 `.rs`, 0 `Cargo.toml`, no `src/crates/`; 41 catalog rows all `scaffolded`/`migrating`; only 6 real Cedar files + contracts/IaC as substance; self-audit says `THIN_IPS_ONLY`. It is a real *design authority* but a *code shell*. ADR-0363 says keep it; ADR-0347 renamed foundry-fitness → governance.

- **Option A (recommended):** Keep as its own service per ADR-0363, but classify it **SPEC-STAGE / not-live** in any service inventory so it is never mistaken for a running gate.
- **Option B:** Fold governance into oya-ci. Cost: collapses the clean authority/runner split; oya-ci becomes both the rule-definer and the rule-runner (self-approval anti-pattern).
- **Option C:** Shelve/archive it. Cost: loses the SSOT for what "admissible / production-ready" means; the Cedar ABAC + contracts are real design assets.

**Recommendation: Option A.** It is distinct-and-needed (authority), not redundant-with-oya-ci (runner) — but it must be labeled honestly as spec-only until built.

---

## Q4 — governance vs oya-ci boundary: build governance out, or formalize it as pure spec that oya-ci implements?

**Decision at stake:** Closing governance's decision-debt — the boundary is correct by design but unproven in code.

**Evidence:** governance DEFINES gates (lanes, 6-axis rules, admission verdicts, signed evidence); oya-ci + GH Actions/Jenkins/ArgoCD RUN them (`oya gate run-all`, ADR-0346/0349). But governance's lane-execution + policy-engine logic is unbuilt, and the 3 bundled `oya-check-*` rows point `prior_path` to a legacy `crates/oya-check-*` location *outside* this service — so live check logic, if any, was never migrated in.

- **Option A:** Build governance out (policy-engine + lane-runtime + evidence-emitter + aggregation-indexer become real crates); it becomes the live authority oya-ci invokes.
- **Option B (recommended near-term):** Keep governance as pure SPEC/contract authority; have oya-ci implement the lanes directly against governance's Cedar + OpenAPI contracts; defer governance's own runtime until there is load-bearing need for signed/replayable auditor evidence (SOC2/ISO/GDPR).
- **Option C:** Source the live check logic from the legacy `oya-check-*` path and wire it under whichever service wins (B → oya-ci, A → governance).

**Recommendation: Option B near-term, with a tripwire to A.** Don't build a second execution engine speculatively. Make governance the authoritative spec; let oya-ci execute. Promote to Option A (governance gets its own runtime) only when an external audit obligation actually requires governance-owned signed/replayable evidence. Resolve the legacy `oya-check-*` source-of-truth either way (Option C).

---

## Q5 — Reconcile the ADR citation mismatch (process integrity)

**Decision at stake:** Trust in the ADR references used to route these services.

**Evidence:** The routing task cited ADR-0389/0390 for cloud-intelligence; those numbers appear nowhere in that service. Its real governing ADRs are ADR-0384/0373/0131/0105/0090. The bedrock-audit / OpenAI-compatible-pipeline *concept* maps cleanly to ADR-0373 — only the identifiers are off.

- **Option A (recommended):** Treat ADR-0389/0390 as renumbered or misattributed; bind the routing to the verified ADRs (0384/0373). Issue a correction so future routing cites the real numbers.
- **Option B:** Assume 0389/0390 refer to a different/newer repo or pending ADRs and locate them before finalizing. Cost: blocks routing on a citation that may not exist.

**Recommendation: Option A.** The concept is verified present under ADR-0373; don't block on a stale identifier. Log the correction so the masterplan/ADR index stays the SSOT (per the SSOT-reachability rule).

---

### Agenda priority order
1. **Q1** (ratify foundry-platform home — foundational, settled by evidence).
2. **Q2** (provider wire seam — the only real duplication risk; highest architectural leverage).
3. **Q3 + Q4** (governance status + boundary — close the decision-debt; B near-term).
4. **Q5** (ADR citation hygiene — process integrity, quick to close).
