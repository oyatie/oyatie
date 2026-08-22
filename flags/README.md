---
doc_class: README
microservice: feature-flags
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0105
  - ADR-0131
  - ADR-0159
  - ADR-0160
  - ADR-0242
  - ADR-0243
  - ADR-0248
companion_docs:
  - microservices/feature-flags/ARCHITECTURE.md
  - microservices/feature-flags/PRD.md
  - microservices/feature-flags/manifest.json
planned_enforcement_ref: governance-microservice-doc-set
---

# feature-flags

Oyatie's canonical OpenFeature-compliant runtime feature-flag substrate. Implements the runtime feature-flag gate alongside ChangeSet code-deploy gates (ADR-0110) and progressive delivery (ADR-0160).

Hyperscaler precedents: LaunchDarkly relay-proxy model, Statsig server-side evaluation, Cloudflare Workers KV flag propagation, Unleash per-tenant strategy model, OpenFeature cross-SDK spec.

## Quick orientation

| Question | Answer |
|---|---|
| What does it do? | Runtime flag evaluation + experiment design + kill-switch with per-tenant Cedar targeting |
| Who calls it? | Every µservice via `feature-flags-sdk`; SDKs in Rust, TypeScript, Python |
| Latency target | ≤1ms p99 (cell-local evaluation) |
| Availability target | ≥99.99% |
| Service role | **Substrate** — consumed by all 46+ µservices |
| Cell topology | Control-plane substrate cell |
| Tenant model | ADR-0330 `tenant_class`: `demo_trial` capped usage, `paid` with composable `billing_components` |
| Authorization | Cedar v4.2 LTS; default-deny; per-flag step-up auth |
| Audit | All flag lifecycle events sealed per ADR-0028 |

## Directory layout

```
microservices/feature-flags/
├── ARCHITECTURE.md          — §principals §cedar-gates §tenant-scoping §transport §observability
├── CHANGELOG.md             — SemVer history
├── compliance.md            — §pack-overlay-roster §day-one-cert-readiness §detection-substrate-binding
├── competitor-parity-matrix.md
├── capacity-model.md
├── cost-budget.md
├── dpia.md
├── failure-modes.md
├── incident-response.md
├── manifest.json
├── multi-region.md
├── backfill-replay.md
├── sdk-plan.md
├── PRD.md
├── PHASE-01-LAUNCHDARKLY-CLASS-FLAG-SUBSTRATE.md
├── threat-model.md
├── IP-001-feature-flags-design-readiness.md
├── IP-002-flag-kernel.md … IP-020-*.md
├── AUDIT-FINDINGS-2026-05-20.json
├── scorecards/overrides.json
├── capabilities/
│   ├── flag-evaluate.yaml
│   ├── experiment-design.yaml
│   ├── killswitch-trigger.yaml
│   └── pack-overlay-subscribe.yaml
├── catalog/
│   └── feature-flags-{bc}-{layer}.yaml  (≥11 records)
├── contracts/
│   ├── openapi-v1.yaml           — OpenAPI 3.2.0; OpenFeature-compatible
│   ├── asyncapi-v1.yaml          — AsyncAPI 3.1.0; flag-state-changed events
│   ├── feature-flags-v1.proto    — proto3 gRPC surface
│   └── openfeature-sdk-contract.md
├── dashboards/
│   ├── flag-state-overview.json
│   ├── experiment-results.json
│   ├── killswitch-history.json
│   └── pack-override-coverage.md
├── iac/
│   ├── k8s-deployment.yaml
│   ├── helm-values.yaml
│   ├── network-policy.yaml
│   ├── secret-bindings.yaml
│   ├── openbao-policy.hcl
│   ├── ech-config.yaml
│   ├── pqc-cert.yaml
│   ├── edge-waf.yaml
│   └── terraform/main.tf
├── policy/
│   ├── flag-mutation-authorization.cedar
│   ├── experiment-design-authorization.cedar
│   ├── pack-flag-override.cedar
│   ├── safety-killswitch-authorization.cedar
│   ├── abuse-defence.cedar
│   ├── pack-overlay-authorization.cedar
│   ├── data-residency.md
│   ├── auditor-scope.cedar
│   ├── ci-scope.cedar
│   ├── emergency-services-bypass.cedar
│   └── tenant-targeting.cedar  (existing)
├── runbooks/
│   ├── killswitch-engaged.md
│   ├── flag-mutation-cascade.md
│   ├── experiment-rollback.md
│   ├── audit-replay.md
│   ├── pack-override-cascade.md
│   ├── stale-targeting-rule.md
│   ├── experiment-stat-sig-violation.md
│   ├── a11y-flag-violation.md
│   └── flag-evaluation-regression.md  (existing)
└── slos/
    ├── flag-eval-latency.openslo.yaml
    ├── flag-state-propagation.openslo.yaml
    ├── experiment-result-freshness.openslo.yaml
    ├── killswitch-fire-latency.openslo.yaml
    └── feature-flags.openslo.yaml  (existing)
```

## Bounded contexts

| BC | Purpose | Hot path latency |
|---|---|---|
| `flag` | CRUD lifecycle for flag definitions | N/A (control plane) |
| `targeting` | Cedar-based targeting rules + percentage rollout | ≤0.5ms |
| `experiment` | A/B + multivariate design + statistical analysis | ≤2ms (stat-sig on demand) |
| `metric-attribution` | Attribute conversion events to experiment variants | Async |
| `rollout` | Progressive rollout orchestration + auto-rollback | N/A (orchestration) |
| `killswitch` | Emergency flag disable + life-safety bypass | ≤100ms |

## How to call the SDK (Rust)

```rust
// Cargo.toml: feature-flags-sdk = { workspace = true }
use feature_flags_sdk::{FlagClient, EvaluationContext};

let client = FlagClient::new(config).await?;
let ctx = EvaluationContext::builder()
    .tenant_id("acme-corp")
    .principal_id("user-abc")
    .persona_tier(PersonaTier::B2C)
    .build();

// Boolean evaluation — p99 ≤1ms (cache hit)
let enabled = client.bool_value("dark-mode-v2", false, &ctx).await?;
```

## How to define a flag (REST)

```bash
curl -X POST https://feature-flags.internal/api/v1/flags \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Step-Up-Token: $STEP_UP_TOKEN" \
  -d '{
    "flag_key": "dark-mode-v2",
    "flag_type": "boolean",
    "default_variant": "off",
    "intent": "release_toggle",
    "sunset_at": "2026-08-01T00:00:00Z",
    "targeting_rules": [{
      "variant": "on",
      "percentage": 10
    }]
  }'
```

## How to engage a kill-switch (SRE)

See `runbooks/killswitch-engaged.md` for full procedure. CLI shortcut:

```bash
# Step-up auth required (Class C: TOTP + passkey)
oya flags kill-switch engage dark-mode-v2 \
  --reason "regression in dark-mode rendering on mobile" \
  --step-up-token $STEP_UP_TOKEN
# KillSwitchEngaged audit event emitted; all cells updated ≤1s
```

## CI lanes

| Lane | Purpose | Gate |
|---|---|---|
| `governance-adr-adherence-matrix` | 28-row ADR adherence check | Advisory → BLOCKER 2026-07-16 |
| `governance-pack-overlay-coverage` | Verifies all active packs have declared overrides | Advisory |
| `governance-microservice-doc-set` | Artifact count ≥70 | Advisory → BLOCKER 2026-07-16 |
| `governance-abuse-defence-ux-floor` | Default-path latency budget | Advisory |
| `governance-emergency-services-chaos-test` | Quarterly chaos test | Advisory |
| `governance-detection-fairness-audit` | Quarterly experiment fairness | Advisory |

## Key ADRs

- [ADR-0159](docs/decisions/ADR-0701-monorepo-capability-live-apex.md) — Binding ADR for feature-flag substrate.
- [ADR-0160](docs/decisions/ADR-0700-ci-admission-live-apex.md) — Progressive delivery integration.
- [ADR-0183](docs/decisions/ADR-0701-monorepo-capability-live-apex.md) — Cedar policy engine separation.
- [ADR-0243](docs/adr-archive/ADR-0243-cedar-as-universal-gate.md) — Cedar as universal gate.
- [ADR-0248](docs/decisions/ADR-0700-ci-admission-live-apex.md) — Cell architecture.

## On-call escalation

Runbook index: `runbooks/`. PagerDuty service: `feature-flags-sre`. Escalation: `axis-governance` → `axis-platform-oncall`.

## Doctrine references

- [ADR-0346](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): `./bin/oya verify --ci-required` is the canonical local pre-push verifier and MUST locally mirror the full CI matrix, blocking on exit-0 of each mandatory step before returning success. Enforced by `governance-verify-ci-mirror-coverage`, `governance-verify-ci-step-exit-semantics`, `governance-verify-skip-flag-allowlist`, `governance-submit-calls-verify`, and `governance-verify-exit-code-contract`.
- [ADR-0347](../../docs/decisions/ADR-0709-general-live-apex.md): Every `governance-*` CI lane prefix RENAMES to `governance-*` in one Wave 15-ZB bulk-rename pull request rather than 34 per-lane migration IPs. Enforced by `governance-no-foundry-fitness-residue`, `governance-lane-prefix-vocabulary`, and `governance-rename-inventory-presence`.
- [ADR-0348](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Cellular topology MUST support control-plane-driven AUTOSHARDING, AUTO-REBALANCE, and DYNAMIC SHARDING, with manifest-declared configuration, residency/compliance constraints, audit-chain emission, and reversibility. Enforced by `governance-sharding-automation-coverage`, `governance-autosharding-manual-mode-refusal`, `governance-auto-rebalance-residency-honored`, `governance-dynamic-sharding-threshold-coverage`, `governance-audit-chain-emit-on-automation-events`, and `governance-tenant-migration-reversibility`.
- [ADR-0349](../../docs/decisions/ADR-0700-ci-admission-live-apex.md): Jenkins (LTS) and ArgoCD are the canonical self-hostable CI/CD substrates; Jenkins augments GitHub Actions for self-hostable contexts, and ArgoCD is the canonical GitOps CD orchestrator that replaces manual `kubectl apply` and Helm CLI deploys. Enforced by `governance-jenkins-github-actions-parity`, `governance-argocd-application-cosign-verified`, `governance-argocd-tenant-namespace-isolation`, `governance-jenkins-jcasc-only`, and `governance-deploy-audit-chain-emit`.
