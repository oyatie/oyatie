---
doc_class: PRD
template_id: TPL-PRD
microservice: foundry
status: Accepted
date: 2026-05-13
authority_chain: docs/MASTERPLAN.md §2.1 → ADR-0056 → ADR-0058 → this PRD
audience: internal (no external tenants)
owner_team: council-foundry
canonical_base_only: true   # foundry is pack-neutral (internal engine)
adrs_cited:
  - ADR-0053  # sanctioned primitives
  - ADR-0054  # scaffold-claim pattern
  - ADR-0056  # BNF v4.1
  - ADR-0062  # quality/perf/scale bar
  - ADR-0063  # documentation suite coverage (LEAN-A5)
doc_status: published
---

# `oya-foundry-*` — Internal Engineering Engine PRD

## Intent

Foundry is the **internal-only** engineering engine that powers every other µservice. It is not a customer-facing product. Its tenants are oyatie's own engineering agents (and humans during the bootstrap window): they call Foundry to scaffold µservices, claim symbols, run fitness lanes, register capabilities, retrieve evidence, and gate work through the Proof Ladder.

Foundry is the substrate on which M01–M12 milestones depend. Every µservice in the flat catalog (§MASTERPLAN 2.1) consumes Foundry tooling at scaffold-time, build-time, and verify-time. Foundry itself is jurisdiction-neutral and pack-neutral by ADR (§2.5 canonical base rule).

## Bounded Contexts

| BC | Purpose | Lead-phase |
|---|---|---|
| `engine-evidence` | Phase-Spec evidence emission + signed audit record | M02-P01 |
| `engine-eval` | Proof-Ladder L0→L7 evaluation runner | M02-P01 |
| `engine-mcp-gateway` | MCP tool-call dispatch + autonomy-ceiling enforcement | M02-P17 |
| `engine-run` | step-pipeline orchestrator (grit work tracker) | M02-P01 |
| `engine-step` | atomic step record + replay primitive | M02-P01 |
| `engine-catalog` | µservice catalog + capability registry | M02-P17 |
| `engine-cargo-prefix` | BNF v4.1 prefix validator (lib-name parity, layer suffix) | M01-P05 |
| `engine-corpus` | regulatory corpus.lock loader (per pack; Bominal ADR-0190) | M04 |
| `engine-adapter` | adapter-namespace registry + cross-product-refusal gate | M02-P01 |
| `engine-capability` | autonomy-ceiling registry; capability tokens | M02-P17 |
| `engine-bypass` | bootstrap-window carve-out ledger (ADR-0053 §carve-outs) | M01 |
| `engine-mdbook` | mdbook publishing pipeline | M-CC-P02 |
| `engine-openapi` | OpenAPI contract validator | M-CC-P02 |

Plus `oya-check-*` rule binaries (BNF-exempt) for the 14+ fitness lanes.

## Competitive Benchmark

| Capability | Industry reference | Foundry's parity dimension |
|---|---|---|
| Symbol-locked agent work coordination | rtk-ai/grit | grit ↔ ICM ↔ scaffold-claim pattern (ADR-0054); first-class primitive, not a script layer |
| Capability registry + autonomy ceiling | LangChain agent gateway, AWS Bedrock Guardrails | Cedar-policy + capability tokens; deterministic-replay execution per Bominal ADR-0107 |
| Workspace dependency graph + fitness lanes | Bazel (build graph), `cargo-deny` (supply-chain) | cargo + cargo-deny + cargo-semver-checks + 14 LEAN lanes; ADR-0056 inward-only flow CI-enforced |
| Per-PR architectural fitness checks | ArchUnit (Java), Nx workspace lint | `oya-check-architecture` 9 sub-commands; per-PR BLOCKER post M02-P22 |
| Evidence-driven phase progression | OPA + Conftest, Open Policy Framework | Proof Ladder L0→L7 (Bominal ADR-0223 inherited); signed evidence per (tenant, period) |
| Doc-coverage CI enforcement | (no direct competitor; Read.the.Docs is publish-only) | `oya-check-doc-coverage` LEAN-A5 per ADR-0063 |

Primary-source research: Bominal ADRs 0100–0112 (hexagonal-microservice standard family); rtk-ai/grit documentation; AWS Bedrock Guardrails docs; Bazel-vs-cargo workspace comparisons in `docs/architecture/architecture-maps.md`.

## Performance Targets

Foundry is on-host CLI tooling and CI infrastructure; targets are latency + reliability under agent-fleet load (not customer p99).

| Dimension | Target | Notes |
|---|---|---|
| `grit claim` symbol-lock acquire p99 | ≤50ms | local SQLite; lock contention bounded by agent-pool size |
| `grit done` rebase-merge-release p99 | ≤5s | single-PR; pre-commit hooks run inline |
| `oya-check-architecture` per-sub-command (workspace ~140 crates) | ≤300s | cargo metadata + AST scan; cached across PR runs |
| `oya-check-doc-coverage --workspace` | ≤600s | walks `docs/` + pack manifests + milestone dirs |
| `oya-foundry-evidence` segment-seal latency | <1s per (tenant, period) | per Bominal ADR-0028 audit-chain target |
| ICM `store` / `recall` p99 | ≤100ms | local SQLite; fall back to scaffold-locks-oyatie when DB unavailable |
| Agent autonomy-ceiling validation p99 | ≤50ms | Cedar policy eval per Bominal ADR-0107 |
| Capability-token mint p99 | ≤100ms | symmetric key-derivation; bound to (agent, capability, ttl) tuple |

Error budget: 0.1% per phase (one fitness lane failure per 1000 PR runs); SLO burn-rate alarm at 10× monthly budget.

## Horizontal Scalability

Foundry is **mostly stateless** — `grit` uses local SQLite per workspace clone (not cross-machine state); `oya-check-*` binaries are pure functions of repo HEAD; ICM is per-agent local; only `engine-evidence` segment-seal writes to shared storage (RLS-isolated per tenant_id).

| Requirement | Status |
|---|---|
| Stateless services | ✓ `oya-check-*` (pure functions of repo HEAD) |
| Sharded state | n/a (no Postgres in Foundry kernel) |
| Event-driven | n/a (Foundry is synchronous CLI tooling; CI dispatch via GitHub Actions only) |
| Cell architecture | n/a (Foundry runs per-agent; not tenant-bound) |
| Active-active capable | yes — `oya-check-*` binaries replicate trivially per CI runner |
| Cross-region replication | n/a (per-agent state) |
| Auto-scale ready | yes (GitHub Actions runners + per-PR fan-out) |

Per-cell capacity envelope: ~100 concurrent agent-pool PRs per workspace; ~10 simultaneous `cargo nextest run --workspace` on a single 16-core runner.

Scale-out trigger: PR queue depth > 50; spin up additional Actions runners.

Cross-region story: not applicable; Foundry is build-time infrastructure, not runtime.

## Architectural posture

- **No customer-facing surface** — Foundry never receives a tenant request; it exists between agents + CI + the codebase
- **Sanctioned primitives** (per ADR-0053) — Foundry exposes only `grit`, `icm`, `oya-tooling-agent-read`. Direct `git`/`gh` requires ICM rationale logged BEFORE invocation (Directive 12)
- **Composition root only** — `oya-foundry-cli-dev-runtime` is the single binary aggregating Foundry tooling; agents call sub-commands
- **Pack-neutral** — Foundry has zero jurisdiction-specific logic; localization packs do NOT extend Foundry (the localization-pack pluggability rule §ADR-0064 §2 does NOT apply to internal-engine µservices)

## Open dependencies

- Bominal `corpus.lock` schema (ADR-0190 inheritance) for `engine-corpus`
- Bominal Proof Ladder L0..L7 (ADR-0223 inheritance) for `engine-eval`
- rtk-ai/grit binary CLI (external dep; vendored as workspace tool)
- cargo-deny / cargo-semver-checks / cargo-nextest (workspace tooling)

## References

- `docs/MASTERPLAN.md` §2.1 (catalog) §6 (Operating model)
- ADR-0053 sanctioned primitives
- ADR-0054 scaffold-claim pattern
- ADR-0056 BNF v4.1
- ADR-0062 quality/perf/scale bar
- ADR-0063 documentation suite coverage
- Bominal ADRs 0100/0101/0107/0190/0223 (inherited)
