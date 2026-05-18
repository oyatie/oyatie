---
doc_class: COMPETITOR-PARITY
microservice: foundry
status: Accepted
date: 2026-05-18
owner_team: axis-foundry + council-product
related_adrs: [ADR-0136, ADR-0137]
---

# Competitor Parity Matrix — foundry (consolidated)

## Scope

Cross-BC competitive analysis. Per-BC parity matrices preserved at
`bc-sources/<bc>/competitor-parity-matrix.md`.

## Foundry shape vs competitors

| Competitor | Product | Internal BCs (their term) | foundry equivalent |
|---|---|---|---|
| AWS Bedrock | Agents | Agent runtime / Knowledge bases / Guardrails / Model catalog / Studio | runtime / (n/a — oyatie reads ontology) / guardrails / providers / supervisor + eval |
| Google Vertex AI | Agent Builder | Agents / Tools / Safety filters / Evals / Deploy | runtime / providers / guardrails / eval / supervisor |
| Microsoft Azure | AI Foundry | Agents / Model catalog / Safety / Evaluation / Deployment | runtime / providers / guardrails / eval / supervisor |
| Anthropic | Console / Claude API | Workbench / System prompts / Tools / Evaluations / Usage analytics | runtime+supervisor / runtime / providers / eval / evidence+observability |
| OpenAI | Assistants API | Threads / Tools / Files / Evals (separate product) | runtime (sessions) / providers / runtime / eval |
| Palantir | AIP | AIP Logic / Threads / Evals / Operator / Tools | supervisor / runtime / eval / supervisor+guardrails / providers |
| LangChain | LangSmith + LangGraph | LangGraph (runtime) / LangSmith (eval+trace) / LangServe (deploy) | runtime / eval+evidence / supervisor |

## Parity dimensions

| Dimension | AWS Bedrock | Vertex AI | Azure Foundry | Anthropic | Palantir | foundry |
|---|---|---|---|---|---|---|
| Hosted agent runtime | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ (runtime BC) |
| Capability/tool registry | ✓ | ✓ | ✓ | partial | ✓ | ✓ (runtime cache + supervisor canonical) |
| Inline safety/guardrails | ✓ | ✓ | ✓ | partial | ✓ | ✓ (guardrails BC) |
| Eval harness with parity replay | partial | ✓ | ✓ | ✓ | ✓ | ✓ (eval BC) |
| Audit-chain Ed25519+Merkle | ✗ | ✗ | partial | ✗ | partial | ✓ (evidence BC) — **differentiator** |
| Autonomy-tier gate (T0–T4 ceiling per principal) | ✗ | ✗ | ✗ | ✗ | partial | ✓ (runtime+guardrails+supervisor) — **differentiator** |
| Subscription-mode provider adapters (e.g. Claude Pro) | ✗ | ✗ | ✗ | ✗ | ✗ | ✓ (providers BC) — **differentiator** |
| Per-pack residency (cross-pack-forbidden default) | partial | partial | partial | ✗ | ✓ | ✓ (all BCs) |
| Self-hosted substrate (operate under your Kubernetes) | ✗ | ✗ | partial | ✗ | ✓ | ✓ (all BCs) |
| OpenSLO-gated capability promotion | ✗ | ✗ | ✗ | ✗ | partial | ✓ (ADR-0139 + supervisor) — **differentiator** |
| Per-tenant kill-switch | partial | partial | ✓ | ✗ | ✓ | ✓ (supervisor BC) |
| OpenBao credential isolation | ✗ | ✗ | ✗ | ✗ | partial | ✓ (providers BC) — **differentiator** |

## Differentiators (foundry-unique)

1. **Audit-chain Ed25519+Merkle on every cross-BC state transition** — none
   of the listed competitors carry first-class cryptographic seal on every
   invocation/supervision/evidence/eval/guardrail/provider event.
2. **Autonomy-tier-gate per principal** with refusal at dispatch — ADR-0022
   is unique to oyatie.
3. **Subscription-mode provider adapters** — invoke Claude Pro / ChatGPT
   Plus / Gemini Advanced subscriptions transparently alongside paid API
   tiers; unique commercial flexibility.
4. **OpenSLO-gated capability promotion** (ADR-0139) — capability versions
   gated through observability runways before tenant-default; competitors
   atomically deploy versions.
5. **OpenBao credential isolation in dedicated BC** — provider credentials
   never resident outside one named BC; cross-BC traffic carries no secret
   material.
6. **Per-pack-forbidden residency invariant** with CI enforcement —
   stricter than competitors' opt-in regional residency.

## Per-BC parity archives

- `bc-sources/runtime/competitor-parity-matrix.md` — runtime-axis competitor
  detail.
- `bc-sources/supervisor/competitor-parity-matrix.md` — supervisor-axis.
- `bc-sources/eval/competitor-parity-matrix.md` — eval-axis (LangSmith,
  RAGAS, OpenAI Evals).
- `bc-sources/evidence/competitor-parity-matrix.md` — audit-chain axis.
- `bc-sources/guardrails/competitor-parity-matrix.md` — Lakera, NVIDIA NeMo
  Guardrails, AWS Bedrock Guardrails, OpenAI Moderation.
- `bc-sources/providers/competitor-parity-matrix.md` — LiteLLM, OpenRouter,
  Bedrock model catalog, Vertex Model Garden.

## References

- ADR-0136 / ADR-0137: foundry topology.
- `microservices/foundry/PRD.md` — competitive benchmark section.
