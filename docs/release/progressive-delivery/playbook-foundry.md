---
doc_class: Playbook
shape: anchor
length_cap: 120
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Rollout strategy for Foundry capability publishes — eval-set + canary + cohort + audit-chain + Cosign-signed.
planned_enforcement_ref:
  - governance-canary-required
  - governance-shadow-diff
  - governance-rollback-evidence
related_adrs: [ADR-0020, ADR-0021, ADR-0024, ADR-0039, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Playbook: Foundry Capability Rollout


## 1. Surface

Foundry capability publishes through `intelligence-capability-kernel` + `intelligence-mcp-gateway-kernel` ([ADR-0021](../../decisions/ADR-0021-intelligence-capability-registry-and-mcp-gateway.md)).

## 2. Default rail

**Canary + dark-launch + eval-set gate.** Blue/green for capability cutovers that replace a published predecessor irreversibly.

Per [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md), capability lifecycle stages mirror branch layers: `dev-draft` → `dev` → `staging` → `prod`. The canary runs on `staging`; `prod-promoter` fires after the 5-gate clears.

## 3. Pre-publish gates

1. Eval-set PASS (per [ADR-0024](../../decisions/ADR-0024-intelligence-eval-harness-and-replay.md)) — capability run against versioned eval-set; pass rate ≥ baseline.
2. Cosign signature ([ADR-0039](../../decisions/ADR-0039-supply-chain-security-trivy-cosign-sbom-signed-commits.md)) — capability artefact signed; SBOM attested.
3. Autonomy-ceiling declaration ([ADR-0022](../../decisions/ADR-0022-autonomy-ceiling-runtime-enforcement.md)) — capability declares max tier (T1–T4).
4. Audit-chain emit hook registered.

## 4. Rollout sequence

| Phase | Cohort | Action | Gate |
|---|---|---|---|
| 1 | `internal` only | Dark-launch (shadow-diff at 100% sample) | shadow-diff verdict pass; ≥ 10,000 pairs |
| 2 | `canary-pioneer` | 1% canary | fast-burn ≤ threshold; eval-set re-run pass |
| 3 | `canary-eligible` | 5% → 25% → 50% → 100% | per [`canary-rail-spec.md`](canary-rail-spec.md) |
| 4 | `stable-enterprise` | 100% on 14-day lag | slow-burn ≤ threshold sustained |
| 5 | `stable-regulated` | 100% on 28-day lag + per-vertical pack approval | per-vertical DPIA refresh ([`playbook-vertical-pack.md`](playbook-vertical-pack.md)) |

## 5. Per-provider canary

If the capability has multiple provider-adapter backings (Claude / OpenAI / Gemini per [ADR-0020](../../decisions/ADR-0020-intelligence-multi-provider-adapter-model.md)), each adapter is canaried independently. Provider-A breach does not block Provider-B promotion.

## 6. Rollback

Per-cell rollback default. Capability-publish rollback re-emits a D14 entry naming the prior published version. The MCP gateway re-routes to the prior version atomically via `intelligence-mcp-gateway-kernel` traffic-shift.

For **replay-affecting** capability changes (output schema, tool-call signature), blue/green is mandatory; rollback re-shifts traffic to the prior version's MCP endpoint.


## 7. Audit-chain artefacts

Per publish, emit:
- capability-id + version + content-hash.
- eval-set version + pass rate.
- Cosign signature + SBOM hash.
- Per-stage cohort sample size + burn-rate observations.
- Rollback re-emit (if applicable).

## 8. Anti-patterns

- Publishing without eval-set baseline → blocked by `governance-cohesion`.
- Publishing without Cosign signature → blocked by supply-chain lane.
- Skipping dark-launch on replay-affecting capability → blocked by `governance-shadow-diff`.
- Canary against a single provider when capability has multiple adapters → blocked by lane (`governance-canary-required`).

## 9. Hyperscaler equivalent

OpenAI's model-card + system-card publication discipline; Anthropic's Acceptable-Use eval; Google DeepMind's red-team eval-set. We adopt the eval-set-as-gate pattern and add the canary rail underneath.

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — capability stages mirror branch layers; `capability-reviewer` re-affirms at staging → prod gate 5.
