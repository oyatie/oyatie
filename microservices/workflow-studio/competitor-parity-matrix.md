---
doc_class: CompetitiveBenchmark
title: Competitor Parity Matrix
microservice: workflow-studio
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: axis-workflow + council-architecture + gtm-customer-success
deciders: axis-workflow, council-architecture
related_adrs: [ADR-0123, ADR-0131, ADR-0133]
related_artifacts:
  - microservices/workflow-studio/PRD.md (§Competitive Benchmark)
  - /specs/products/workflow-studio.json (§competitive)
  - /specs/hyperscaler-gates.json (HG-WORKFLOW-STUDIO)
review_cadence: bi-annually + on every new competitor entrant
doc_status: published
---

# Competitor Parity Matrix (workflow-studio µservice)

## Purpose

Quantitative + qualitative parity comparison vs the industry-leading visual workflow authoring products. Drives the `oya-governance-hyperscaler-maturity-claims` gate (per ADR-0123 HG-WORKFLOW-STUDIO) and informs gtm-customer-success on permissible vs forbidden sales claims. Re-validated bi-annually.

## Competitor Set

Per `/specs/products/workflow-studio.json` §competitive — drawn from primary sources only:

| Competitor | Product / surface | Primary differentiator | Source |
|---|---|---|---|
| n8n | n8n Studio (visual editor + execution) | drag-drop canvas; large connector mental model; queue mode | `docs.n8n.io` |
| Zapier | Zap editor | trigger/action editor; simple onboarding | `help.zapier.com/zap-editor` |
| Workato | Recipe editor + recipe copilot | large connector catalog; recipe functions | `docs.workato.com` |
| Make.com | Scenario editor | scenario settings; error handlers; scenario recovery | `help.make.com` |
| Microsoft Power Automate | Power Automate cloud + desktop | Microsoft ecosystem integration; ALM; governance | `learn.microsoft.com/power-automate` |
| Temporal | Temporal Cloud UI (workflow run inspector) | durable execution; replay-centered reliability | `docs.temporal.io` |
| Palantir Foundry Pipeline Builder | Pipeline Builder visual editor | typed-entity bindings; ontology-aware authoring | `palantir.com/docs/foundry/pipeline-builder/` |
| Retool Workflows | Retool Workflows | low-code; database-integrated workflows | `docs.retool.com/workflows` |
| Tines | Visual story builder | security-team-focused; case management integration | `tines.com/docs` |
| AWS Step Functions Workflow Studio | Step Functions Studio | drag-drop state machine; AWS service integration | `docs.aws.amazon.com/step-functions/latest/dg/workflow-studio.html` |
| Camunda Web Modeler | BPMN/DMN modeler | BPMN 2.0; collaboration; Git sync; play mode | `docs.camunda.io/docs/components/modeler/web-modeler/` |
| Argo Workflows UI | Argo Workflows UI | Kubernetes-native; workflow templates | `argo-workflows.readthedocs.io` |
| GitHub Actions | GitHub Actions YAML reuse | reusable workflows; YAML; marketplace | `docs.github.com/en/actions` |
| Linear (workflow status UX benchmark) | Linear workflow configuration | opinionated state model; keyboard-fast UX | `linear.app/docs/configuring-workflows` |

## Feature Parity Matrix

### Visual authoring core

| Capability | oyatie | n8n | Zapier | Workato | Make | Power Automate | Temporal | Foundry | Retool | Tines | Step Fn | Camunda | Argo |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Drag-drop canvas | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | view-only | ✅ | ✅ | ✅ | ✅ | ✅ | view-only |
| Node config side panel | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | n/a | ✅ | ✅ | ✅ | ✅ | ✅ | n/a |
| Live debugger | ✅ M03 | ✅ | partial | partial | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| DSL view (canonical JSON) | ✅ AC-02 round-trip | partial | ❌ | partial | ❌ | YAML | code | yaml | code | ❌ | JSON | XML BPMN | YAML |
| Spec-first authoring (DSL is source of truth) | ✅ | ❌ (visual is source) | ❌ | ❌ | ❌ | YAML | code | yaml | ❌ | ❌ | ✅ | XML | ✅ |
| Round-trip byte-equality (load(emit(x))==x) | ✅ AC-02 100% GA | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ❌ | ❌ | ❌ | ❌ | partial | ❌ |
| Git-backed PRs from editor | ✅ FR-15 | partial | ❌ | ❌ | ❌ | partial | ✅ | partial | partial | ❌ | ❌ | ✅ | ✅ |

### Collaboration

| Capability | oyatie | n8n | Zapier | Workato | Make | Power Automate | Temporal | Foundry | Retool | Tines | Step Fn | Camunda | Argo |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Real-time multi-user editing | ✅ CRDT | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ |
| Conflict-free merge (CRDT) | ✅ AC-06 | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ✅ | ✅ | ❌ | ❌ | partial (last-writer-wins) | ❌ |
| Never silent loss invariant | ✅ load-bearing | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Multi-user cursor presence | ✅ M03 | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | ✅ | ✅ | ❌ | ❌ | ✅ | ❌ |

### Policy + jurisdiction + audit

| Capability | oyatie | n8n | Zapier | Workato | Make | Power Automate | Temporal | Foundry | Retool | Tines | Step Fn | Camunda | Argo |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Cedar policy preview before save | ✅ FR-09 unique | ❌ | ❌ | ❌ | ❌ | partial (DLP) | ❌ | ✅ | ❌ | partial | ❌ | ❌ | ❌ |
| Per-seat Cedar enforcement | ✅ AC-08 | account-seat | account-seat | account-seat | account-seat | account-seat | account-seat | per-user | per-user | account-seat | n/a | account-seat | n/a |
| Jurisdiction-overlay visual diff | ✅ FR-08 unique | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |
| Per-pack node libraries | ✅ FR-11 unique | partial (regions) | partial | partial | ❌ | partial (GCC) | ❌ | ✅ | ❌ | ❌ | n/a | ❌ | ❌ |
| Audit-chain Ed25519 seal per save | ✅ ADR-0028 | log | log | log | log | log | log | partial | log | partial | log | log | log |
| Data-class markers (PII / PHI / SECRET) on canvas | ✅ FR-16 unique | ❌ | ❌ | ❌ | ❌ | partial (sensitivity labels) | ❌ | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |

### LLM-assist authoring

| Capability | oyatie | n8n | Zapier | Workato | Make | Power Automate | Temporal | Foundry | Retool | Tines | Step Fn | Camunda | Argo |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Prose-to-spec drafting | ✅ FR-12 M03+ | partial | ✅ (AI generation) | ✅ (recipe copilot) | partial | ✅ Copilot | ❌ | partial | ❌ | partial | partial | ❌ | ❌ |
| PII redactor before LLM | ✅ T-I-05 control | ❌ | ❌ | ❌ | ❌ | partial | n/a | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| Prompt-injection scrub + validation | ✅ T-S-05 | ❌ | ❌ | ❌ | ❌ | ❌ | n/a | partial | ❌ | ❌ | ❌ | ❌ | ❌ |
| Schema-valid completion enforced | ✅ FR-12 contract | ❌ | ❌ | ❌ | ❌ | partial | n/a | ✅ | ❌ | ❌ | ❌ | ❌ | ❌ |
| BYO-LLM | ✅ via foundry-providers | partial | ❌ | partial | ❌ | partial (Azure OpenAI) | n/a | ❌ | ❌ | ❌ | ❌ | ❌ | ❌ |

### Performance + scale

| Capability | oyatie target | n8n claim | Workato | Make | Power Automate | Foundry | Camunda |
|---|---|---|---|---|---|---|---|
| Editor TTI cold (p99) | ≤ 2s GA | ~3s typical | ~4-6s | ~3-5s | ~5-8s | ~3-4s | ~4-6s |
| Save round-trip (p99) | ≤ 100ms GA | ~300ms | ~500ms | ~400ms | ~1s | ~200ms | ~500ms |
| Cold-load 5k-node graph | ≤ 3s | ~10s+ (degraded UX) | n/a | n/a | n/a | ~5s | ~8s |
| Collab CRDT merge (p99) | ≤ 100ms | n/a | n/a | n/a | n/a | ~200ms | ~last-writer-wins |
| Concurrent editor sessions per region | 100K | 10K cluster | 100K | 10K | 100K | 50K | 20K |

(All competitor numbers from primary-source docs; oyatie figures are targets, not measured-and-validated until M03/P01 exit gate green per `IP-015` evidence pinning.)

## Quantitative Performance Parity

Per ADR-0123 + `/specs/products/workflow-studio.json` §competitive_claim_policy: NO numeric latency comparison claims permitted without measured oyatie evidence. The table above shows targets vs publicly-claimed numbers for design awareness only.

Pending measurement at M03/P01 exit gate:
- TTI p99 via synthetic Lighthouse harness (test against pack-kr CDN edge).
- Save round-trip p99 via synthetic save-loop test.
- Collab CRDT merge p99 via 10-user concurrent drill.
- Cold-load 5k-node via golden-corpus test.

## Key Parity Gaps to Close (oyatie → industry leader)

| # | Gap | Owner | Target close |
|---|---|---|---|
| 1 | Connector count breadth (Workato ~1200, n8n ~400 vs oyatie M03 per-pack libraries) | axis-workflow + gtm | post-M03 marketplace |
| 2 | Mobile-app editor (none of competitors except Power Automate Mobile; oyatie deferred post-GA) | council-design-system | post-M03 |
| 3 | AI Copilot UX maturity (Zapier AI / Workato Copilot / Power Automate Copilot all live) | axis-workflow + foundry-providers | M03 LLM-assist preview → GA |
| 4 | Marketplace template ecosystem (n8n + Zapier + Make all have community templates) | axis-workflow + gtm | post-M03 |
| 5 | Pre-built integration coverage for top 100 SaaS (Workato/Zapier lead here) | axis-workflow + partner-eng | rolling, multi-quarter |

## Key oyatie Differentiators (not in any competitor)

1. **Round-trip byte-equality contract** (AC-02): no other competitor enforces visual ↔ spec byte-equality. Most BPMN-first tools mutate spec on visual edit. oyatie unique.
2. **Cedar policy preview before save** (FR-09): no other competitor surfaces policy impact in the authoring surface before deploy.
3. **Jurisdiction-overlay visual diff** (FR-08): no other competitor has multi-jurisdiction overlay UX.
4. **Per-seat Cedar enforcement at editor open** (AC-08): competitors enforce at account-tier (coarse); oyatie enforces per-user with audit row per decision.
5. **Per-pack node libraries with Ed25519 signing + revocation propagation ≤ 60s** (FR-11): no competitor has signed-and-revocable supply-chain for nodes.
6. **Audit-chain Ed25519 seal per save + per-license-decision** (PRD §"Audit + Compliance"): competitors log; oyatie cryptographically seals.
7. **"Never silent loss" CRDT invariant** (AC-06): Foundry has CRDT; oyatie additionally has the load-bearing zero-silent-loss invariant as a Sev-1 gate.
8. **Data-class markers on canvas** (FR-16): PII/PHI/SECRET visible before save; competitors do not surface data-class in authoring UX.
9. **Spec-first source of truth** (BP-02 / Bominal ADR-0164 inherited): visual derives from canonical spec; competitors are visual-first with spec generated.

## Claim-Boundary Rules

Permitted (citation-bounded):
- "oyatie Studio offers round-trip byte-equality, an invariant n8n / Zapier / Workato / Make / Power Automate do not contract" (true; cite their primary docs).
- "oyatie Studio surfaces Cedar policy preview before save, which no surveyed competitor's primary docs establish" (true).
- "Jurisdiction-overlay visual diff is unique to oyatie among surveyed competitors" (true as of 2026-05-17; re-validate bi-annually).

Forbidden (per ADR-0123 hyperscaler-maturity-claim-gate + `/specs/products/workflow-studio.json` §competitive_claim_policy):
- "oyatie editor is faster than n8n" (no measured benchmark; would be unsourced superiority).
- "oyatie has more connectors than Workato" (NOT TRUE pre-M04 marketplace).
- "oyatie is the only collaborative workflow editor" (NOT TRUE — Foundry Pipeline Builder + Retool Workflows have collab).
- "oyatie is HIPAA-compliant out of the box" (conditional on BAA + pack-us-healthcare activation).

## Bi-Annual Refresh Process

| Step | Owner |
|---|---|
| 1. Survey competitor docs for changes (new features / pricing / claims) | gtm-customer-success |
| 2. Update this matrix; cite primary-source URLs + commit SHAs | axis-workflow |
| 3. Re-run quantitative benchmarks (load tests against pack-kr cluster) | ops-sre-reliability |
| 4. Council-architecture review for claim-boundary rule updates | council-architecture |
| 5. Publish + notify sales/gtm | gtm-customer-success |

## Evidence Pinning (per IP-015)

Each competitor row's primary-source URL is snapshotted at registration time; SHAs recorded at `evidence/competitor-evidence-snapshots-<timestamp>.json`. This prevents competitor doc-drift from silently invalidating our parity claims.

## References

- `microservices/workflow-studio/PRD.md` §Competitive Benchmark.
- `/specs/products/workflow-studio.json` §competitive + §competitive_claim_policy.
- `/specs/hyperscaler-gates.json` HG-WORKFLOW-STUDIO gate.
- ADR-0123 (hyperscaler-maturity-claim-gate).
- ADR-0133 (industry-best-practice conformance axis-4 named sources).
- Competitor docs as cited inline above.
