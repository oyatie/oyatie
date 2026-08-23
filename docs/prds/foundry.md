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
  - ADR-0063  # documentation set coverage (LEAN-A5)
doc_status: published
---

# `foundry-*` — Internal Engineering Engine PRD

## Intent

Foundry is the **internal-only** engineering engine that powers every other µservice. It is not a customer-facing product. Its tenants are oyatie's own engineering agents (and humans during the bootstrap window): they call Foundry to scaffold µservices, claim symbols, run fitness lanes, register capabilities, retrieve evidence, and gate work through the Proof Ladder.

Foundry is the substrate on which M01–M12 milestones depend. Every µservice in the flat catalog (§MASTERPLAN 2.1) consumes Foundry tooling at scaffold-time, build-time, and verify-time. Foundry itself is jurisdiction-neutral and pack-neutral by ADR (§2.5 canonical base rule).

## Bounded Contexts

| BC | Purpose | Lead-phase |
|---|---|---|
| `engine-evidence` | Phase-Spec evidence emission + signed audit record | M02-P01 |
| `engine-eval` | Proof-Ladder L0→L7 evaluation runner | M02-P01 |
| `engine-mcp-gateway` | MCP tool-call dispatch + autonomy-ceiling enforcement | M02-P17 |
| `engine-step` | atomic step record + replay primitive | M02-P01 |
| `engine-catalog` | µservice catalog + capability registry | M02-P17 |
| `engine-cargo-prefix` | BNF v4.1 prefix validator (lib-name parity, layer suffix) | M01-P05 |
| `engine-corpus` | regulatory corpus.lock loader (per pack; Bominal ADR-0190) | M04 |
| `engine-adapter` | adapter-namespace registry + cross-product-refusal gate | M02-P01 |
| `engine-capability` | autonomy-ceiling registry; capability tokens | M02-P17 |
| `engine-bypass` | bootstrap-window carve-out ledger (ADR-0053 §carve-outs) | M01 |
| `engine-mdbook` | mdbook publishing pipeline | M01-P09 |
| `engine-openapi` | OpenAPI contract validator | M01-P09 |

Plus `check-*` rule binaries (BNF-exempt) for the 14+ fitness lanes.

## Competitive Benchmark

| Capability | Industry reference | Foundry's parity dimension |
|---|---|---|
| Capability registry + autonomy ceiling | LangChain agent gateway, AWS Bedrock Guardrails | Cedar-policy + capability tokens; deterministic-replay execution per Bominal ADR-0107 |
| Workspace dependency graph + fitness lanes | Bazel (build graph), `cargo-deny` (supply-chain) | cargo + cargo-deny + cargo-semver-checks + 14 LEAN lanes; ADR-0056 inward-only flow CI-enforced |
| Per-PR architectural fitness checks | ArchUnit (Java), Nx workspace lint | `check-architecture` 9 sub-commands; per-PR BLOCKER post M02-P22 |
| Evidence-driven phase progression | OPA + Conftest, Open Policy Framework | Proof Ladder L0→L7 (Bominal ADR-0223 inherited); signed evidence per (tenant, period) |
| Doc-coverage CI enforcement | (no direct competitor; Read.the.Docs is publish-only) | `check-doc-coverage` LEAN-A5 per ADR-0063 |


## Performance Targets

Foundry is on-host CLI tooling and CI infrastructure; targets are latency + reliability under agent-fleet load (not customer p99).

| Dimension | Target | Notes |
|---|---|---|
| `check-architecture` per-sub-command (workspace ~140 crates) | ≤300s | cargo metadata + AST scan; cached across PR runs |
| `check-doc-coverage --workspace` | ≤600s | walks `docs/` + pack manifests + milestone dirs |
| `intelligence-evidence` segment-seal latency | <1s per (tenant, period) | per Bominal ADR-0028 audit-chain target |
| Agent autonomy-ceiling validation p99 | ≤50ms | Cedar policy eval per Bominal ADR-0107 |
| Capability-token mint p99 | ≤100ms | symmetric key-derivation; bound to (agent, capability, ttl) tuple |

Error budget: 0.1% per phase (one fitness lane failure per 1000 PR runs); SLO burn-rate alarm at 10× monthly budget.

## Horizontal Scalability


| Requirement | Status |
|---|---|
| Stateless services | ✓ `check-*` (pure functions of repo HEAD) |
| Sharded state | n/a (no Postgres in Foundry kernel) |
| Event-driven | n/a (Foundry is synchronous CLI tooling; CI dispatch via GitHub Actions only) |
| Cell architecture | n/a (Foundry runs per-agent; not tenant-bound) |
| Active-active capable | yes — `check-*` binaries replicate trivially per CI runner |
| Cross-region replication | n/a (per-agent state) |
| Auto-scale ready | yes (GitHub Actions runners + per-PR fan-out) |

Per-cell capacity envelope: ~100 concurrent agent-pool PRs per workspace; ~10 simultaneous `cargo nextest run --workspace` on a single 16-core runner.

Scale-out trigger: PR queue depth > 50; spin up additional Actions runners.

Cross-region story: not applicable; Foundry is build-time infrastructure, not runtime.

## Architectural posture

- **No customer-facing surface** — Foundry never receives a tenant request; it exists between agents + CI + the codebase
- **Composition root only** — `intelligence-cli-dev-runtime` is the single binary aggregating Foundry tooling; agents call sub-commands
- **Pack-neutral** — Foundry has zero jurisdiction-specific logic; localization packs do NOT extend Foundry (the localization-pack pluggability rule §ADR-0064 §2 does NOT apply to internal-engine µservices)

## Open dependencies

- Bominal `corpus.lock` schema (ADR-0190 inheritance) for `engine-corpus`
- Bominal Proof Ladder L0..L7 (ADR-0223 inheritance) for `engine-eval`
- cargo-deny / cargo-semver-checks / cargo-nextest (workspace tooling)

## References

- `docs/MASTERPLAN.md` §2.1 (catalog) §6 (Operating model)
- ADR-0053 sanctioned primitives
- ADR-0054 scaffold-claim pattern
- ADR-0056 BNF v4.1
- ADR-0062 quality/perf/scale bar
- ADR-0063 documentation set coverage
- Bominal ADRs 0100/0101/0107/0190/0223 (inherited)
