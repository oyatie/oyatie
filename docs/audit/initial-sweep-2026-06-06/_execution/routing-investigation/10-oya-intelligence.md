# oya/intelligence — routing investigation

Read-only audit. Path: `/Users/jasonlee/Developer/source/oya/intelligence`. No source files edited.

## What it is

`intelligence` is the **product AI substrate** µservice (per ADR-0255 two-layer AI Substrate + Consumer Brand Surface, ADR-0220 consumer-intelligence). It is a real, heavily-built service: **128 crates** under `crates/`, **~96,000 lines of Rust**, plus a deep doc corpus (PRD, ARCHITECTURE, 97 IP-* implementation plans, 30+ journey IPs, contracts, IaC, runbooks, 147 catalog records, 26 capability YAMLs). It is NOT a placeholder shell.

Per `README.md` it owns: provider routing, refusal-baseline guardrails, output evaluation, citation attribution, the consumer brand UX surface, credential resolution (BYOK), and audit-tap emission. Embeddings and fine-tuning were promoted OUT to separate µservices (`intelligence-embeddings`, `intelligence-fine-tuning`).

Crates follow a strict hexagonal layering convention (`*-kernel` pure value logic, `*-domain`, `*-usecase`, `*-adapter`, `*-api`/`-rest`/`-sse`/`-websocket`/`-graphql`, `*-worker`, `*-app` composition root).

## Key finding: the foundry IS folded in here

`_legacy-foundry/` is the OLD `foundry` "hosted-agent platform" service, preserved as a docs-only legacy snapshot (its README: `Service: foundry / Business capability: hosted-agent platform`; it lists runtime/supervisor/eval/evidence/guardrails/providers/vector BCs). The agent-PLATFORM primitives that foundry described now live as **real, implemented crates inside `intelligence`**. Capability ids and prompts in the code still carry the `foundry.*` namespace (e.g. `foundry.capability.discover`, `foundry.audit.tail`), and catalog records are still named `oya-foundry-*`, confirming the lineage.

## Does it hold the AI-agent-PLATFORM primitives? YES — substantially.

| Foundry platform primitive | Where in intelligence | Real vs shell |
|---|---|---|
| **Model gateway / router** | `model-routing-{kernel,domain,usecase}`, `route-policy-kernel`, `provider-pool-{kernel,app}` (1758 + 4852 lines) | REAL — `decide_route()` is full deterministic catalog routing w/ denial reasons, priority ranking, data-class/audience/tenant gating + tests |
| **Provider/account adapters** | `adapter-{anthropic,openai,gemini}-{api,subscription}-{kernel,adapter}`, `adapter-{anthropic,openai}-compat-api`, `providers-adapter-openai` | MIXED — kernels/traits REAL; api-adapters are explicit **in-memory mocks** ("Live-network deferred behind feature flag"); `*-compat-api` and `subscription-adapter` are larger (770–1836 lines) |
| **Account / session drivers (Claude, Codex, Gemini CLI)** | `claude-account-adapter`, `codex-account-adapter`, `gemini-account-adapter`, `account-{kernel,domain,adapter-inmemory}`, `oauth-subscription-kernel`, `supervisor-{kernel,app}` | PARTIAL-REAL — drivers actually `spawn` `claude-code`/`codex` subprocesses w/ secret injection + idempotency keys, but `drain_response`/`inject`/`kill` return `"...response placeholder"` stubs |
| **Capability registry** | `capability-registry-{kernel,domain,app}`, `capability-domain` | REAL — typed CapabilityId/EvidenceRef, status state machine, registry views, autonomy classification |
| **MCP gateway** | `mcp-gateway-domain` (528 lines) | REAL — full MCP 2025-11-25 protocol: tenant endpoints, OAuth token validation, scope/tier authorization, rate limiter, tool/prompt projection. Pure domain (no sockets) by design |
| **Eval harness** | `eval-{kernel,domain,usecase,adapter,worker,app}` | REAL — full hexagonal stack (~5k lines across layers); golden-set/parity/replay capabilities in catalog |
| **Guardrails** | `guardrails-{kernel,domain,usecase}`, `route-policy-kernel`, `write-gate-kernel` | REAL — kernel 659 lines; classify-prompt / validate-output / enforce-autonomy capability YAMLs present |
| **RAG / context** | `rag-endpoint-{kernel,domain,app}`, `rag-api`, `context-aware-retrieval-{kernel,domain,usecase,adapter,worker}` | REAL — context-aware-retrieval stack is substantial (~4k lines); rag-endpoint is deliberately thin/internal-only (caller-side-RAG posture: corpus lives elsewhere) |
| **Per-tenant context / credentials** | `credential-resolver-{domain,usecase,adapter}` (893+1433 lines), `account-domain`, SecretReference/SecretStorePort | REAL — BYOK resolution, redacted secret handling, tenant scoping |
| **Autonomy ceiling** | `autonomy-ceiling-{kernel,domain,app}`, `bypass-{domain,ledger-kernel}`, `usage-window-kernel` | REAL — T1Read..T4Actuate tier model, T4 disabled-by-default, tenant ceiling resolution + Cedar mirror |
| **Audit tap** | `audit-tap-{adapter,usecase,worker}` | REAL — ~2.7k lines across layers; audit-chain seal-stream emission per ADR-0263 |
| **Attribution / provenance** | `attribution-{kernel,domain,usecase,adapter,worker,app}` | REAL — full stack ~5k lines |
| **Supervisor / fleet / kill-switch** | `supervisor-{kernel,app,security-adapter}`, `subagent-runtime-{kernel,usecase,app}`, `jsonl-supervisor-adapter`, `pr-review-dispatcher-app` | REAL — session driver trait, fleet lifecycle, kill-switch capabilities in catalog |
| **Brand UX surface** | (doc-described in README/IP-020; assist-draft surface) `assist-draft-{kernel,domain,usecase,adapter,api,rest,worker}` | REAL — advisory-draft surface stack ~6k lines |

## Real-vs-shell verdict (overall)

**Real implementation, not a stale shell.** Of 128 crates, only 2 are near-empty (`oya-codeview-cli` 3 lines, `oya-backbone-workload-live-app` 8 lines). Zero `todo!`/`unimplemented!` markers across the sampled core crates; zero `placeholder`/`stub`/`not-yet-implemented` grep hits in crate Rust. Largest crates are generated/dense surfaces (`openapi-domain` 6332, `architecture-map-kernel` 2528, `provider-pool-app` 4852).

The one honest caveat: the **live network/provider I/O is deliberately deferred.** Provider api-adapters are in-memory mocks gated behind a feature flag, and the CLI session drivers spawn real subprocesses but stub the response drain/inject/kill. So the *domain, routing, policy, registry, MCP, eval, guardrails, autonomy, credential, audit logic is real and tested*; the *outbound provider wire calls and CLI session streaming are the explicit not-yet-wired frontier.* This is consistent with the PRD's stance ("Runtime quality and SLO achievement remain outside this design claim").

## Bottom line for the foundry question

Yes — the AI-agent-platform primitives the foundry had (**model gateway, provider/account adapters, capability registry, MCP gateway, eval harness, guardrails, RAG/context, per-tenant credential context, autonomy ceiling, supervisor/kill-switch, audit/attribution**) all live HERE in `oya/intelligence` as real Rust crates. The legacy foundry service survives only as a docs snapshot under `_legacy-foundry/`; its platform substance was migrated/rebuilt into `intelligence` (with `foundry.*` capability namespacing retained).
