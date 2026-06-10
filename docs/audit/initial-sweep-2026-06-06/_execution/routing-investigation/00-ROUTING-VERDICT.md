# 00 — ROUTING + GOVERNANCE-VALIDITY VERDICT

Synthesizer: workflow-subagent (routing + governance-validity)
Date: 2026-06-06
Mode: READ-ONLY. No source files edited.
Inputs: `10-oya-intelligence.md`, `10-cloud-intelligence.md`, `10-oya-governance.md` (this dir), each ground-truthed against the real source trees under `/Users/jasonlee/Developer/source/`.

---

## 0. Verification performed before this verdict

All three source roots exist on disk and the load-bearing claims of the three input artifacts were re-checked directly (not taken on trust):

| Claim | Verified evidence |
|---|---|
| intelligence is real, ~96k LOC | `find crates -name '*.rs' \| wc -l` → **95,964 LOC**; **128 crate dirs** |
| intelligence holds the platform primitives | crate dirs physically present for model-routing, provider-pool, mcp-gateway, capability-registry, autonomy-ceiling, eval, guardrails, credential-resolver, rag-endpoint, context-aware-retrieval, supervisor, audit-tap, attribution, claude/codex/gemini-account-adapter |
| foundry folded into intelligence | `_legacy-foundry/` dir exists (docs-only snapshot); **357 files** reference `oya-foundry-`; `foundry.*` capability namespace present in code + tests |
| intelligence's honest frontier | exactly **10 files** carry `response placeholder` / `Live-network deferred` / `in-memory mock` markers |
| cloud-intelligence is real, ~12.6k LOC | `find crates -name '*.rs' \| wc -l` → **12,603 LOC**; **8 crate dirs** |
| cloud-intelligence disclaims platform primitives | `PRD.md:80` "Not a prompt/eval registry…", `PRD.md:49` "does not own prompt templates, eval harnesses, or fine-tune jobs"; codex-adapter TODO at `rest/src/lib.rs:30` and `:369` |
| governance is a code shell | **0 `.rs` files, 0 `Cargo.toml`, `src/crates/` MISSING**; **41 catalog rows all `scaffolded`/`migrating`**; **6 real `.cedar`** files = genuine spec substance |

The three input artifacts are accurate. This verdict builds on verified ground truth.

---

## 1. WHERE the absorbed foundry AI-agent-PLATFORM belongs

### Verdict: the agent-platform substrate belongs in `oya/intelligence`. The founder's lean toward `cloud/cloud-intelligence` is WRONG for the platform substrate — but RIGHT for one specific slice (the egress inference broker), which is exactly what cloud-intelligence already is.

This is not a 51/49 call. It is decisive, and the evidence is asymmetric.

### Head-to-head on the seven platform primitives

| Foundry platform primitive | `oya/intelligence` | `cloud/cloud-intelligence` |
|---|---|---|
| **Model gateway / router** | REAL — `model-routing-{kernel,domain,usecase}`, `route-policy-kernel`, `provider-pool-{kernel,app}`; `decide_route()` full deterministic policy routing | Partial/different — a *reverse-proxy* seat-selection FSM (pool a credential, sign, passthrough). It routes *credentials*, not *models-by-policy*. |
| **Provider/account adapters** | REAL kernels/traits + real CLI subprocess drivers for claude/codex/gemini; live api wire mocked behind a flag | REAL async reqwest AnthropicAdapter (wired) + CodexAdapter (built, NOT wired); this is the genuine **wire I/O** layer |
| **Capability registry** | REAL — `capability-registry-{kernel,domain,app}` | **ABSENT** — PRD disclaims it |
| **MCP gateway** | REAL — `mcp-gateway-domain`, full MCP 2025-11-25 protocol | **ABSENT** |
| **Eval harness** | REAL — `eval-{kernel,domain,usecase,adapter,worker,app}` full stack | **ABSENT** — PRD.md:80 explicitly "no eval harness" |
| **Autonomy ceiling** | REAL — `autonomy-ceiling-*` T1–T4 + bypass ledger + usage-window | **ABSENT** |
| **RAG / context** | REAL — `rag-endpoint-*` + `context-aware-retrieval-*` | **ABSENT** — an embeddings/RAG indexer is a downstream *caller*, not a feature (PRD.md:58) |
| Guardrails / refusal baseline | REAL — `guardrails-*` 659-line kernel + write-gate | ABSENT |
| Supervisor / kill-switch / subagent runtime | REAL — `supervisor-*`, `subagent-runtime-*`, kill-switch capabilities | ABSENT (only an opaque `AgentId` caller-identity for metering) |
| Audit / attribution | REAL — `audit-tap-*`, `attribution-*` | Has metering/audit *events* (`LlmGatewayEvent` → ClickHouse + Valkey), not provenance/attribution |

**Score: 7-of-7 platform primitives live in `oya/intelligence`. 0-of-7 live in `cloud-intelligence`, and cloud-intelligence's own PRD disclaims them by name.** That is dispositive.

### Why the founder's instinct is understandable but mis-targeted

Both services touch "providers, OAuth, Claude/Codex/Gemini, credential pooling," so they *look* like the same thing. They are not. The overlap is real but it is a **seam**, not a duplication:

- `cloud/cloud-intelligence` is the **egress inference broker / request-pipeline gateway**: the single chokepoint that pools provider credentials (ADR-0384 Path B OAuth-subscription-seat pooling + API-key mode) so no agent holds a raw key, meters spend per tenant, and circuit-breaks failing keys. Its IP is the credential-pool FSM + RAII seat lease + SSE passthrough. It is infrastructure-shaped (cloud/* family, OpenBao/ClickHouse/Valkey, Cedar default-deny, axum reverse proxy).
- `oya/intelligence` is the **product AI substrate**: the policy/intelligence brain that decides *which model, under which capability, at which autonomy tier, with which guardrails, producing which evaluated+attributed output*. Its IP is routing policy, capability registry, MCP, eval, autonomy, guardrails, RAG, attribution. It is product-shaped (oya/* family, consumer brand UX surface, ADR-0255 two-layer substrate).

The founder is conflating "the thing that talks to providers over the wire" (cloud-intelligence — correct) with "the agent platform" (intelligence — the actual home of the primitives). cloud-intelligence is the **how-we-reach-providers** layer; the foundry platform is the **what-the-agent-is-allowed-to-do-and-how-good-was-it** layer.

### The crisp boundary

> **`oya/intelligence` owns the agent-platform substrate (routing policy, capability registry, MCP, eval, autonomy, guardrails, RAG, attribution, supervisor/kill-switch) and is the home of the absorbed foundry. `cloud/cloud-intelligence` owns the egress inference broker — credential pooling, OAuth-seat leasing, metering, circuit-breaking, wire I/O — and is a downstream dependency of intelligence, not its peer for platform primitives.**

Direction of dependency: `intelligence` decides → `cloud-intelligence` brokers the call to the provider. intelligence is the policy plane; cloud-intelligence is the data/egress plane.

### The one genuine architectural decision this exposes (the live provider I/O seam)

intelligence's live provider wire I/O is *deliberately deferred* (10 files of mock/placeholder markers; CLI drivers spawn real subprocesses but stub drain/inject/kill). cloud-intelligence's wire I/O is *real and wired* (AnthropicAdapter live). This is the single real overlap and the most important routing question for the founder:

- When intelligence finally wires live provider calls, **should it call providers directly, or call through cloud-intelligence?** The clean answer is: through cloud-intelligence (so credential pooling, metering, and circuit-breaking are enforced at the single egress chokepoint, exactly as cloud-intelligence's PRD intends). That would mean intelligence's deferred provider api-adapters become *thin clients of the cloud-intelligence gateway*, not independent wire callers. This needs an explicit founder decision because today both services carry their own (partial) Anthropic/Codex adapters, which is the only place the two could drift into true duplication.

---

## 2. Is `oya/governance` a VALID live service or stale debt?

### Verdict: VALID as a DESIGN AUTHORITY, but currently a CODE SHELL (spec-stage / decision-debt). It is **distinct-and-needed**, NOT redundant with oya-ci. It must not be deleted — but it must not be counted as a live service either.

### What its crates actually do (today, on disk)

Nothing executes. There is **0 Rust, 0 Cargo.toml, and the `src/crates/` directory the README references does not exist.** The "crates" are **41 catalog YAML descriptors**, every one marked `scaffolded` or `migrating`, across four bounded contexts:

1. **policy-engine** — (designed) rule-pack decision engine, 6-axis conformance aggregation, baseline-pin authority, **admission verdicts** ("is PR #N admissible against `dev`?").
2. **lane-runtime** — (designed) runs the ~50 CI fitness lanes against a working tree.
3. **evidence-emitter** — (designed) signed/replayable Findings + audit-chain seal events for SOC2/ISO27001/GDPR auditor replay.
4. **aggregation-indexer** — (designed) deterministic regen of central indices, refuses hand-edits.

The only genuinely machine-readable substance that exists is **6 real Cedar ABAC policy files** (tenant/auditor/ci/public-read scoping) plus OpenAPI/AsyncAPI/proto contracts and IaC stubs. Its own audit (`AUDIT-FINDINGS-2026-05-18.json`) self-reports `THIN_IPS_ONLY`. The 3 bundled `oya-check-*` rows point `prior_path` at a legacy `crates/oya-check-*` location *outside* this dir — so if any real check logic exists, it predates this service and was not migrated in.

Note: the prompt's example crate names (`abuse-defence`, `acl-enforcement`, `admission`) do **not** exist here, not even as catalog rows. `oya-governance-abuse-defence-ux-floor` appears only as a prose lane-name reference. Do not treat those as governance's real surface.

### Distinct-and-needed vs redundant-with-oya-ci

**Complementary, clean control/execution split — not redundant:**

- **governance = the AUTHORITY**: it *defines* the gates — the ~50 lanes, the ADR-0133 6-axis rules, severities/BLOCKER, baseline citations, the admission permit/forbid verdict semantics, and the signed/replayable evidence trail. It is the SSOT for *what* "production-ready / admissible" means.
- **oya-ci (+ GitHub Actions / Jenkins / ArgoCD) = the EXECUTION SUBSTRATE**: it *runs* the gates (`oya verify --ci-required`, `oya gate run-all` per ADR-0346/0349), mirrors the CI matrix, does GitOps CD. It invokes the authority; it does not define it.

ADR-0363 ("governance stays its own service") + ADR-0347 (foundry-fitness → governance rename) are coherent with this split. The redundancy worry is unfounded: they sit on opposite sides of a policy-authority / execution-substrate line.

**The honest caveat that makes this decision-debt rather than settled fact:** today only the *spec* side of the authority exists. At runtime the governance↔oya-ci boundary is **notional** — the lane-execution logic that would make governance a live authority is unbuilt, and the live check logic (if any) lives at the legacy `oya-check-*` path. So the boundary is correct *by design* but unproven *in code*. This is the central governance decision the founder must close.

---

## 3. Corrected foundry-rename routing

Given all verified evidence, the corrected routing for the absorbed foundry is:

1. **The foundry hosted-agent-PLATFORM → `oya/intelligence`.** This is settled by code: 7/7 platform primitives live there as real crates (~96k LOC), the `_legacy-foundry/` docs snapshot lives there, 357 files carry the `oya-foundry-` lineage namespace, and the `foundry.*` capability namespace is retained as a deliberate lineage marker. The legacy `foundry` service is correctly retired to a docs-only snapshot; its substance was rebuilt into intelligence. **Do not re-route foundry to cloud-intelligence.**

2. **The foundry egress/provider-wire slice → stays as / converges on `cloud/cloud-intelligence`.** The narrow piece the founder is right about — pooled credentials, OAuth-seat leasing, metering, circuit-breaking, real wire I/O to Anthropic/OpenAI — is already cloud-intelligence's whole reason to exist (ADR-0384). intelligence should *consume* this gateway for live calls rather than grow its own parallel wire adapters. This is the convergence point that prevents the only real duplication risk.

3. **`oya/governance` → keep as its own service (ADR-0363), but classify it HONESTLY as spec-stage authority, not live software.** It is not redundant with oya-ci (authority vs runner). It is also not yet alive. Routing action: keep it, do not fold it into oya-ci, but flag it as decision-debt that must either be built out (policy-engine + lane-runtime + evidence-emitter + aggregation-indexer) or have its live-check logic explicitly sourced from the legacy `oya-check-*` path. Until built, it should be marked SPEC-ONLY in any service inventory so it is not mistaken for a running gate.

### One-line routing summary

> foundry-platform → **oya/intelligence** (verified home, 7/7 primitives, 96k LOC) · foundry-egress/wire → **cloud/cloud-intelligence** (the inference broker, intelligence's downstream dependency) · **oya/governance** → keep as distinct policy authority, but classify SPEC-STAGE (0 Rust) not live, and resolve the build-vs-source-from-legacy decision-debt.

---

## Residual risks / things this verdict is NOT claiming

- It does NOT claim intelligence's live provider calls work today — they are deferred (10 mock/placeholder files). The *domain/policy/registry/eval/MCP/autonomy logic* is real and tested; the *wire* is the frontier.
- It does NOT claim cloud-intelligence's full OpenAI-compatible surface is live — only `POST /v1/messages` (Anthropic-shaped) is wired; Codex adapter is built but not wired into the proxy path (`rest/src/lib.rs:30,369`); maturity is self-described "local foundation" (no live deploy / image / measured SLO).
- It does NOT claim governance runs anything — it is 0 Rust today; its validity is as an authoritative spec, not as running software.
- The task prompt cited ADR-0389/0390 for cloud-intelligence; those do not appear in that service — its real governing ADRs are ADR-0384/0373/0131/0105/0090. The bedrock-audit / OpenAI-pipeline *concept* maps to ADR-0373; only the identifiers differ. Flagged for founder reconciliation.
