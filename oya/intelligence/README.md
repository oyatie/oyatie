---
doc_class: ServiceReadme
template_id: TPL-README
microservice: intelligence
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-20
owner_team: axis-intelligence
related_adrs: [ADR-0255, ADR-0263, ADR-0296]
doc_status: published
---

# intelligence — Two-Layer AI Substrate

Library-first dispatch substrate for the oyatie AI surface per ADR-0255 (two-layer AI Substrate +
Consumer Brand Surface) and the ADR-0255 amendment (library-first network-opt-in clarification). The
`intelligence` µservice owns provider routing, refusal-baseline guardrails, output evaluation,
citation attribution, and the consumer brand UX surface used across every product. Embeddings and
fine-tuning are now separate µservices (`intelligence-embeddings`, `intelligence-fine-tuning`) per
ADR-0255 §D promotion and are NOT in scope here.

## Bounded contexts

| BC | Purpose |
|---|---|
| `model-routing` | Provider selection across Anthropic / OpenAI / Google / Bedrock / vLLM / SGLang / Apple Foundation / OpenRouter / Together / Groq / HuggingFace / Replicate. |
| `providers` | Per-provider adapter pool (16 first-class adapters at launch). |
| `guardrails` | Input + output content filtering, refusal policy, classification, EU AI Act Annex III floors. |
| `eval` | Output-quality evaluation, canonicalen-set scoring, online A/B. |
| `attribution` | Citation rendering, source attribution, provenance graph. |
| `brand-ux-surface` | Consumer brand-UX components (sparkle icon, tier badges, streaming text, citation rendering). |
| `credential-resolver` | `SecretReference` resolution per ADR-0255 §D-4 — provider-BYOK by default, platform-default for B2C. |
| `audit-tap` | Audit-event emission per ADR-0263 onto the audit-chain seal stream. |

## Architecture posture

- **Library-first dispatch** per the ADR-0255 amendment. Consumers link the dispatch SDK directly;
  the network surface is the opt-in fallback for cross-language callers and for the
  brand-ux-surface chrome.
- **Multi-modal day one.** Text, image, audio, and video transports share one dispatch envelope.
- **Caller-side RAG.** This µservice never owns the corpus; embeddings live in
  `intelligence-embeddings` and retrieval is the caller's responsibility.
- **Audience-tag-on-every-call.** Every dispatch carries the audience tag (`consumer`, `developer`,
  `internal-foundry`) per ADR-0255 §"Audience surface enumeration".
- **Substrate tier.** `intelligence` is a substrate µservice; substrate dependencies are declared
  in `manifest.json` (`cloud-secrets`, `policy-engine`, `observability`, `audit-chain`, `cell`).
- **provider-BYOK by default.** Provider credentials never live in the substrate; the
  `credential-resolver` BC resolves `SecretReference`s on the tenant's behalf per ADR-0255 §D-4.

## Document map

| Class | Files |
|---|---|
| Strategic | `PRD.md`, `ARCHITECTURE.md`, `PHASE-01-INTELLIGENCE-TWO-LAYER-MVP.md`, `PHASE-02-CONSUMER-BRAND-SURFACE.md`, `threat-model.md`, `dpia.md` |
| Architecture / ops | `capacity-model.md`, `cost-budget.md`, `failure-modes.md`, `multi-region.md`, `incident-response.md`, `backfill-replay.md`, `compliance.md`, `competitor-parity-matrix.md`, `sdk-plan.md` |
| Policy | `policy/*.cedar`, `policy/data-residency.md`, `policy/tenant-isolation.md` |
| Runbooks | `runbooks/*.md` |
| Contracts | `contracts/openapi/intelligence-v1.yaml`, `contracts/asyncapi/intelligence-events-v1.yaml`, `contracts/proto/intelligence-v1.proto`, `contracts/provider-adapter-trait.md` |
| Capabilities | `capabilities/*.yaml` |
| Dashboards | `dashboards/*.json` + `*.md` |
| SLOs | `slos/*.openslo.yaml` |
| Implementation plans | `IP-001-..IP-025-*.md` |
| Catalog records | `catalog/oya-intelligence-*.yaml` |
| IaC | `iac/k8s/*`, `iac/helm/*`, `iac/terraform/*` |
| Manifest + audit | `manifest.json`, `scorecards/overrides.json`, `AUDIT-FINDINGS-2026-05-20.json` |

## Audience served

| Audience tag | Description | Default cost ownership |
|---|---|---|
| `consumer` | Personal-AI brand surface end-user | platform-default (oyatie covers cost float) |
| `developer` | Builder-on-platform calling the dispatch SDK | tenant provider-credential BYOK (ADR-0255 §D-4) |
| `internal-foundry` | Foundry agent caller (planning, review, exec, doubt) | platform-default + tenant-cell shadow |

## References

- ADR-0255 — Intelligence as two-layer AI Substrate (authority).
- ADR-0255 amendment — Library-first network-opt-in clarification.
- ADR-0263 — Audit-tap per-call emission.
- ADR-0296 — Sidecar credential-handle path.
- `docs/standards/documentation-rigor.md`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `oya-governance-oya-verify-ci-mirror-coverage`, `oya-governance-oya-verify-ci-step-exit-semantics`, `oya-governance-oya-verify-skip-flag-allowlist`, `oya-governance-oya-submit-calls-verify`, and `oya-governance-oya-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `oya-governance-*` CI lane prefix RENAMES to `oya-governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `oya-governance-no-foundry-fitness-residue`, `oya-governance-lane-prefix-vocabulary`, and `oya-governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `oya-governance-sharding-automation-coverage`, `oya-governance-autosharding-manual-mode-refusal`, `oya-governance-auto-rebalance-residency-honored`, `oya-governance-dynamic-sharding-threshold-coverage`, `oya-governance-audit-chain-emit-on-automation-events`, and `oya-governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0709-general-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `oya-governance-jenkins-github-actions-parity`, `oya-governance-argocd-application-cosign-verified`, `oya-governance-argocd-tenant-namespace-isolation`, `oya-governance-jenkins-jcasc-only`, and `oya-governance-deploy-audit-chain-emit`.
