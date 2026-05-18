---
doc_class: PHASE
template_id: TPL-PHASE
phase_id: PHASE-01-FOUNDRY-FOUNDATION
microservice: foundry
status: Accepted
milestone: M01-foundation
related_adrs: [ADR-0123, ADR-0139, ADR-0131, ADR-0132, ADR-0133, ADR-0136, ADR-0137, ADR-0138]
date: 2026-05-18
owner_team: axis-foundry
---

# PHASE-01 — Foundry Foundation

## Purpose

Stand up the foundry µservice as a single product with six internal bounded
contexts (BCs) per ADR-0136 / ADR-0137: runtime, supervisor, eval, evidence,
guardrails, providers. This phase is the consolidated successor to the six
prior PHASE-01 documents owned by the now-retired `foundry-{runtime,
supervisor, eval, evidence, guardrails, providers}` µservices.

The six prior PHASE-01s are preserved verbatim at
`bc-sources/<bc>/PHASE-01-*.md`. This consolidated PHASE-01 supersedes them
as the single phase-of-record for foundry-as-product foundation; the
per-BC PHASE-01s remain authoritative for BC-internal sequencing inside this
phase.

## Phase Scope

| BC | Phase scope (per BC) | bc-sources reference |
|---|---|---|
| runtime | Agent runtime + capability execution substrate | `bc-sources/runtime/PHASE-01-AGENT-RUNTIME-AND-CAPABILITY-EXECUTION.md` |
| supervisor | Control plane landing (fleet + autonomy + capability deploy + kill-switch + event bus) | `bc-sources/supervisor/PHASE-01-CONTROL-PLANE-LANDING.md` |
| eval | Eval harness substrate (runner + parity + replay + golden store) | `bc-sources/eval/PHASE-01-EVAL-HARNESS-SUBSTRATE.md` |
| evidence | Foundry evidence frontend (invocation recorder + pack builder + regulator export + audit-chain bridge) | `bc-sources/evidence/PHASE-01-FOUNDRY-EVIDENCE-FRONTEND.md` |
| guardrails | Guardrails safety + policy enforcement (classifier + validator + autonomy gate + rule engine + jailbreak + AI-slop) | `bc-sources/guardrails/PHASE-01-GUARDRAILS-SAFETY-AND-POLICY-ENFORCEMENT.md` |
| providers | Provider adapter substrate (router + 8 adapters incl. OpenBao credential) | `bc-sources/providers/PHASE-01-PROVIDER-ADAPTER-SUBSTRATE.md` |

## Implementation Plans

This phase ships through **90 sequentially numbered IPs** at
`microservices/foundry/IP-001..IP-090-*.md`, partitioned by BC:

| BC | IP range | Count |
|---|---|---|
| runtime | IP-001 .. IP-015 | 15 |
| supervisor | IP-016 .. IP-030 | 15 |
| eval | IP-031 .. IP-045 | 15 |
| evidence | IP-046 .. IP-060 | 15 |
| guardrails | IP-061 .. IP-075 | 15 |
| providers | IP-076 .. IP-090 | 15 |

Each IP carries a `${bc}` tag in its filename suffix; per ADR-0110, each IP
is one ChangeSet.

## Entry Gates

- ADR-0136 accepted (foundry as single µservice).
- ADR-0137 accepted (six bounded contexts).
- ADR-0138 accepted (six-path deprecation Strangler).
- The 90 consolidated IPs exist at `microservices/foundry/IP-*.md`.
- The bc-sources archive contains all 6 per-BC PRDs + 6 per-BC PHASE-01 + 6
  per-BC threat-models + 6 per-BC compliance.md + ... per `bc-sources/`.

## Exit Gates

- All 90 IPs landed via the agent-coordination Foundry pipeline per
  `docs/AGENTS.md`.
- `oya gate validate per-microservice-layout --microservice foundry` exits 0.
- `oya gate validate authority-cohesion` exits 0; HG-FOUNDRY registered per
  ADR-0123.
- All 6 BC contract surfaces (openapi + asyncapi + proto) lint clean.
- All 4 OpenSLO manifests at `microservices/foundry/slos/` validate.
- Cross-BC e2e tests (AC-X1..AC-X12 from `PRD.md`) green.
- ADR-0138 Phase 5 (six-path code removal) green for any external consumers
  of `microservices/foundry-{runtime,supervisor,eval,evidence,guardrails,
  providers}/` (current state: zero such consumers; soak window 6 months
  from ADR-0138 acceptance).

## Phase Risk Register

| # | Risk | Mitigation | Owner |
|---|---|---|---|
| R1 | Cross-BC operational coupling regresses to monolithic deployment cadence | Per-BC Helm subcharts + per-BC SLO promotion gates per ADR-0139 | ops-sre-reliability |
| R2 | Crate-name collisions across 6 BCs after directory merge | All crate names retain BNF v4.1 `oya-foundry-<bc>-<feature>-<layer>` form; collision-impossible by construction | axis-foundry + tooling-lane |
| R3 | bc-sources archive divergence from canonical top-level docs | CI lane `foundry-bc-source-coherence` enforces no orphan reference + canonical doc lists all 6 BC contributions | axis-foundry |
| R4 | External consumers of old `microservices/foundry-<bc>/` paths break | ADR-0138 Strangler migration + grep verification commands | axis-foundry |
| R5 | Helm chart conflicts when 6 BCs share single chart | One subchart per BC under `iac/helm/<bc>/`; root chart `Chart.yaml` declares 6 dependencies | ops-sre-reliability |

## References

- ADR-0136: Foundry as single µservice.
- ADR-0137: Foundry bounded contexts.
- ADR-0138: Foundry six-path deprecation (Strangler).
- ADR-0131: Per-microservice flat layout.
- ADR-0139: Agentic SLO-gated promotion.
- `microservices/foundry/PRD.md` — consolidated PRD.
- `microservices/foundry/bc-sources/` — per-BC archive (PRDs, PHASE-01s,
  threat-models, etc., preserved verbatim).
