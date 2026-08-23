---
doc_status: published
---

# Team: Axis — Foundry (Agent Runtime + Foundry)

## Mission
This team owns Oyatie's AI agent runtime (Foundry) **and** the Foundry engineering platform — two surfaces that share a single capability registry and a single autonomy-ceiling policy, and therefore cannot be split without diverging their ground truth. Foundry is the force-multiplier axis: it accelerates every other axis exponentially by running their workflows, operating their control planes, populating their search indexes, and executing their ad-auction ML loops — all under one autonomy ceiling, one evidence chain, one capability catalog. Foundry is the discipline surface that ensures every engineer (human or agent) builds to the same quality bar via repoctl, catalog, fitness functions, and CI lane gates. Together they are the substrate that makes the rest of Oyatie coherent at scale.

This team does **not** own per-axis business logic (each axis owns its domain); it owns the runtime, the gates, and the trust infrastructure that every axis runs on.

## Owned axes / surfaces / contracts

### Agent Runtime (Foundry — Axis 3)
- `intelligence-capability-kernel` — `Capability`, `CapabilityId`, `CapabilitySpec`, `AutonomyTier`
- `intelligence-evidence-kernel` — `EvidenceRecord`, `EvidenceChainRef`, `StepTrace`
- `intelligence-policy-kernel` — `AutonomyCeilingPolicy`, `BreakGlass`, `PolicySet`
- `intelligence-domain-*` — capability lifecycle, run orchestration, autonomy-ceiling enforcement use-cases
- `intelligence-adapter-codex` — Codex provider adapter (isolated `CODEX_HOME` per run)
- `intelligence-adapter-claude` — Claude provider adapter (isolated `CLAUDE_CONFIG_DIR` per run)
- `foundation-app` — bootstrap capability invocation REST contract source (`contracts/openapi/foundry/capability-v1.yaml`); future `intelligence-api` owns the deployable REST/gRPC surface
- `intelligence-registry` — capability registry projection (reads from `registry/catalog/`)
- `intelligence-rag` — RAG endpoint (Foundry + Search cross-axis contract)
- `intelligence-runtime-*` — planned flat agent-daemon composition roots and hardening work (#1266 hook_bus, #1267 credential shadowing, #1268 shutdown checkpoint)
- Provider authentication: subscription mode (Claude Pro / OpenAI Plus / Gemini Advanced) + API mode (Anthropic API / OpenAI API / Google Gemini API)
- PTY/process launch backend: direct `openpty`/`forkpty` per spawned provider (not tmux for production; tmux optional for developer-attached debug only)
- SecretProvider / KMS integration (Issue #1315 — P0 blocker for live-provider execution)
- Smoke lane: live provider smoke tests (Issue #1316, env-flag gated)

### Foundry (Axis 4 — consolidated into Foundry)
- `intelligence-catalog-kernel` — `CatalogRecord`, `CrateTarget`, `PlaneDecloration`, `LaneClass`
- `governance-gate-kernel` — `Gate`, `ClaimCeiling`, `Bypass`, `BypassExpiry`
- `governance-*` — all fitness-function crates (one per domain: tenant-shape, audit-emission, data-use-boundary, eventing-topic, architecture-boundaries, doc-catalog, product-prd, horizontal-scale, contract-orphan)
- `governance-scorecard-*` — quality scorecard rollup surfaces
- `tooling-cli-dev-runtime` — current `repoctl` compatibility binary; persona split planned under `crates/tooling-cli-*` per ADR-0015/ROADMAP
- `registry/catalog/` — live catalog path; any future relocation requires a catalog protocol update
- Plane-gated CI lanes (every PR class is routed to the correct lane)
- ADR templates (`decisions/_template.md`)
- Branch-protection-as-code (#239, #1295)
- Signed commits enforcement (#1299)
- Supply-chain: Trivy 4-layer scan (ADR-0039), Cosign image signing, SBOM generation (#614)
- License-policy gate: AGPL/GPL hard-fail in product code; SSPL/BUSL require ADR review
- Foundation-bypass ledger: create, expiry, ledger publication
- Claim-ceiling validator: every new crate's claim is validated against what the foundation has shipped
- Plugin-substrate trust gates (co-owned with `axis-saas` for marketplace listing; Foundry owns signing and sandbox enforcement)
- Vertical-pack authoring surface: Foundry agents author vertical-pack PRDs and regulatory-evidence collections

### Customer-facing builder surfaces
- Workflow Studio agent authoring (agent-authored workflows surface — the runtime is Foundry; the UX shell is `axis-saas` but the agent authoring capability is Foundry)
- Capability authoring SDK for external developers and ISVs
- Foundry capability marketplace (distinct from SaaS plugin marketplace: Foundry capabilities are agent-executable; SaaS plugins are tenant-installable UX extensions)

- **Products owned:** `products/foundry/PRD.md`, `products/foundry/PRD.md`
- **Cross-axis contracts (DESIGN §10):**
  - `Capability invocation` (owner) — every axis is a consumer
  - `Autonomy ceiling policy` (owner, with ADR-0050 governance) — every regulated capability
  - `Capability registry record` (owner) — Foundry + all axes as consumers
  - `Plane class` (owner) — all surfaces declare plane via `registry/catalog/<crate>.yaml: plane:`
  - `Marketplace listing` (co-owner with `axis-saas`) — plugin signing/sandbox gate is Foundry
  - `RAG endpoint` (owner) — Foundry + Search cross-axis contract
- **Catalog records:** `crates/foundry-*`, `crates/tooling-cli-dev-runtime`, `registry/catalog/`
- **Runbooks:** `runbooks/foundry-agent-daemon.md`, `runbooks/autonomy-ceiling-breach-response.md`, `runbooks/capability-rollback.md`, `runbooks/claim-ceiling-bypass-expiry.md`, `runbooks/supply-chain-trivy-alert.md`
- **ADRs:** ADR-0015 (repo structure), ADR-0015 (flat crates), ADR-0050 (AI/ML governance + autonomy ceiling), ADR-0001 (foundry gates), ADR-0039 (supply-chain: Trivy + Cosign)

## In-scope work

### Agent runtime
- SecretProvider + KMS wiring (P0 — Issue #1315; required before any live-provider execution)
- Codex provider adapter: isolated `CODEX_HOME` per run, process lifecycle, stdout/stderr capture
- Claude provider adapter: isolated `CLAUDE_CONFIG_DIR` per run, turn-proof app-server, eval harness
- PTY/process launch backend: `openpty`/`forkpty`, structured output capture, sandbox boundary
- Daemon hardening: hook_bus stale anchor (#1266), credential shadowing (#1267), shutdown checkpoint (#1268)
- AutonomyCeiling policy enforcement (#1279): Cedar policy evaluation per capability invocation
- Capability registry: read from `registry/catalog/`, serve to all axes, maintain projection
- Evidence chain emission: every agent step emits via `intelligence-evidence` → `platform-audit-evidence` chain
- RAG endpoint: expose search to Foundry; per-class consent gate (Data Use Boundary)
- Live provider smoke lane (Issue #1316): env-flag gated CI lane for real provider tests
- Multi-provider authentication: subscription-mode handshake (Claude Pro / OpenAI Plus / Gemini Advanced); API-mode key management (Anthropic API, OpenAI API, Google Gemini API)
- Agent telemetry, eval runs, model performance analytics (analytics plane)
- Agentic buying under autonomy ceiling for `axis-ads-analytics` (ads-axis smart-bidding agent loops)

### Foundry
- repoctl: `pre-push`, `check`, `quality-check`, `release-verify`, `parked` subcommands
- Catalog record authorship tooling: agent-assisted YAML generation + human review gate
- Claim-ceiling validator: ratchet WARN→BLOCK per wave (PRD §4.1 target: ≥ 1 block per 100 PRs → every wave promotes one WARN→BLOCK)
- Foundation-bypass ledger: every bypass has owner, expiry, rationale; 100% retire within declared expiry (PRD §4.2)
- Plane-gated CI lanes: every PR class (`rust-*`, `typescript-*`, `database-*`, `security-*`, `cross-axis-*`) routes to the correct lane
- Fitness functions: author and maintain all `governance-*` crates; hard-fail on violations
- Scorecards: per-axis quality rollup, per-crate health score
- Branch-protection-as-code: every branch rule is code, not console click
- Signed commits: enforce GPG / SSH signing (#1299)
- Supply-chain: Trivy SARIF upload (ADR-0039), Cosign image signing, SBOM per release artifact
- License-policy gate: AGPL/GPL hard-fail; SSPL/BUSL ADR-review gate; Apache-2/MIT/BSD/Mozilla-2 allow-list
- ADR template maintenance: `decisions/_template.md`, supersession graph tooling

### Vertical-pack & capability authoring (customer-facing)
- Foundry capability SDK for external developers
- Agent-authored vertical-pack PRD drafts (human review required before acceptance)
- Agent-assisted regulatory-evidence collection for vertical teams

## Out-of-scope (anti-scope)
- Per-axis domain business logic (each axis owns its domain; Foundry runs it)
- Search index implementation (→ `axis-search` — Foundry consumes RAG endpoint; search team builds it)
- Cloud infrastructure (→ `axis-cloud` — Foundry runs on cloud cells but doesn't provision them)
- SaaS workflow engine (→ `axis-saas` — workflows invoke capabilities via Foundry; engine is `axis-saas`)
- Audit chain infrastructure (→ `platform-audit-evidence` — Foundry emits; audit team owns the chain)
- Data Use Boundary ADR (→ `platform-privacy-dub` — RAG endpoint respects it; Foundry doesn't author it)
- Per-regulator compliance matrix (→ `ops-compliance`)

## Key dependencies on other teams
| Depends on | What we need | Cadence |
|---|---|---|
| `platform-tenancy-identity` | `TenantId`, `AutonomyTier`, Cedar policy evaluation surface | Per-release |
| `platform-audit-evidence` | Audit chain append endpoint for evidence emission | Per-release |
| `platform-privacy-dub` | Data Use Boundary consent gate for RAG endpoint | ADR lifecycle |
| `platform-eventing-og` | Event envelopes for capability telemetry | Per-release |
| `axis-cloud` | Compute cells for daemon hosting, KMS for SecretProvider | Wave gate |
| `axis-search` | Search index read for RAG endpoint | Wave gate |
| `crew-adr-promotion` | ADR-0015, ADR-0015, ADR-0050, ADR-0001, ADR-0039 promotion | ADR batch |
| `ops-security` | Supply-chain Trivy/Cosign threat-model review | Quarterly |

## Teams that depend on us
| Consumer | What they need | Cadence |
|---|---|---|
| `axis-saas` | Capability invocation for agent-authored workflows; Foundry catalog + fitness gates | Every PR + every workflow |
| `axis-cloud` | Capability invocation for control-plane agent operators; catalog gate for cloud crates | Every PR + wave gate |
| `axis-search` | Capability invocation for index-lifecycle agent ops; fitness function gates | Every PR + index lifecycle |
| `axis-ads-analytics` | Capability invocation for smart-bidding ML loops (autonomy-ceiling-gated); catalog gate | Every PR + auction loop |
| All vertical teams | Capability invocation for regulatory-evidence collection; fitness functions; catalog records | Every PR + vertical onboard |
| `platform-tenancy-identity` | Fitness function `governance-tenant-shape` | Every Tenant-shape PR |
| `platform-audit-evidence` | Fitness function `governance-audit-emission` | Every regulated surface PR |
| `platform-privacy-dub` | Fitness function `governance-data-use-boundary` | Every data-class PR |
| `platform-eventing-og` | Fitness function `governance-eventing-topic` | Every topic-shape PR |
| `platform-api-sdk` | API stability gate (ADR-0040 fitness function) | Every public API PR |
| `ops-sre-reliability` | repoctl `pre-push`, `release-verify`; CI lane routing | Every pre-push + release |
| `ops-security` | Supply-chain SARIF upload, Cosign attestation | Every release |
| `crew-adr-promotion` | ADR supersession graph tooling | Every ADR batch |
| `council-architecture` | Catalog + fitness function as governance substrate | Continuous |

## Success metrics
- **Foundry agent runs in production:** ≥ 50K/week, ≥ 99.5% success (PRD §4.1)
- **Audit-chain evidence emission coverage:** 100% of regulated capability invocations (PRD §4.1)
- **Capability namespace count under autonomy ceiling:** ≥ 80% of regulated capabilities (PRD §4.1)
- **Claim-ceiling validator ratchet:** every wave promotes ≥ 1 WARN→BLOCK (PRD §4.1)
- **Foundation-bypass ledger expiry SLA:** 100% of bypasses retire within declared expiry (PRD §4.2)
- **Cross-axis contract violations on `main`:** 0 per quarter (PRD §4.2)
- **Plane-gated CI lane block rate:** ≥ 1 block per 100 PRs (PRD §4.1 — proves gates are non-vacuous)
- **Supply-chain Trivy critical CVEs unpatched > 7 days:** 0
- **License-policy violations in product code:** 0 (hard-fail gate)
- **SecretProvider + KMS wired (Issue #1315):** completion unblocks W-Foundry-Preview gate

## Escalation path
- Internal: tech lead → team manager
- Cross-team: architecture council (`teams/council-architecture/CHARTER.md`) — autonomy-ceiling policy disputes, catalog contract disputes
- Privacy: privacy council (`teams/council-privacy/CHARTER.md`) — RAG endpoint consent gate disputes
- Security: `ops-security` — supply-chain incidents, credential shadowing, autonomy-ceiling breach
- Founder: as last resort

## Communication cadence
- Stand-up: daily async (Slack thread — split by runtime vs foundry sub-tracks)
- Weekly: 60-min sync — capability registry state, daemon hardening progress, fitness-function coverage, bypass-ledger audit
- Cross-team review: monthly cross-axis contract audit; quarterly supply-chain review with `ops-security`
- ADR batch: monthly; ADR-0050 amendments require governance quorum

## Bandwidth + hiring
- Current FTE: TBD (Foundry + Agent Runtime combined; may split into sub-teams at scale)
- Target FTE: TBD per axis-wave (PRD §3.1)
- Open requisitions: link to `HIRING-CAPACITY-PLAN.md`

## Operating norms
- Code review: per CLAUDE.md `## Code Review` rules; autonomy-ceiling policy PRs require security-reviewer agent; supply-chain PRs require `ops-security` co-review
- PR shape: 5-section H2 template
- Pre-push: `repoctl check`
- ADR proposal cadence: monthly batch; ADR-0050 (autonomy ceiling) amendments are P0

## Slice of risk register
| Risk | Severity | Mitigation |
|---|---|---|
| Agent runtime escaping autonomy ceiling for a regulated capability | Catastrophic | Cedar policy + runtime enforcement (#1279) + audit emission + per-capability break-glass + automated revoke |
| SecretProvider stall blocks W-Foundry-Preview gate indefinitely | High | Issue #1315 is P0; weekly progress check; founder escalation if >2 weeks blocked |
| Fitness function false-positive blocks productive PRs | Medium | Fitness function PRs require both `axis-foundry` and the consuming team's sign-off |
| Supply-chain compromise via upstream dependency | High | Trivy 4-layer scan every release; Cosign attestation; SBOM published; AGPL/GPL hard-gate |
| Flat-crates migration stall breaks v2 backlog derivations | High | Per ADR-0015 forward-only assumption; stall triggers ROADMAP blast-radius re-rank |
| Credential shadowing in daemon (#1267) leaks provider keys | Catastrophic | Issue #1267 in daemon hardening track; security-reviewer on every secrets-adjacent PR |
| Claim-ceiling bypass becomes permanent | Medium | 100% expiry SLA enforced by automated ledger monitor; alert at 80% of expiry window |

## Sources scanned
PRD.md §3.1 (W-Foundry-Preview, W-Foundry-Preview gates), §4.1 (Foundry metrics), §4.2 (structural metrics), DESIGN.md §3 (Foundry-as-accelerator, internal sequencing), §4 (bounded context), §10 (capability invocation, autonomy ceiling, capability registry, plane class, marketplace, RAG endpoint rows), ADR-0015, ADR-0050, ADR-0001, ADR-0039, products/foundry/PRD.md (draft), flat `crates/foundry-*` and `crates/tooling-cli-dev-runtime/` implementation surfaces.
