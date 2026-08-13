---
purpose: Oyatie — Runbooks Index
doc_status: published
---

# Oyatie — Runbooks Index

> **Status:** Draft v0.1 — 2026-05-09.
> **Owner:** `ops-sre-reliability`.
> **Discoverability:** `oya ops runbook list`. Every runbook authored from [`templates/runbook-template.md`](templates/runbook-template.md).

## 1. Runbook organization

Per-axis under [`runbooks/<axis>/<runbook-id>.md`](runbooks/) (inside `docs/`). Cross-axis runbooks under [`runbooks/cross-axis/`](runbooks/cross-axis/). Forty-nine P0 stubs were authored on 2026-05-09 and are listed below; full-procedure authoring lands at the W-Foundation gate per the per-runbook `Status:` field.

## 2. Critical runbooks (P0 must-have for W-Foundation gate)

### Agentic pipeline

### Cross-axis
- `cross-axis/audit-chain-integrity-failure.md`
- `cross-axis/dsr-cascade-stuck.md`
- `cross-axis/foundation-bypass-expired.md`
- `cross-axis/cross-tenant-access-detected.md`
- `cross-axis/data-class-violation-detected.md`
- `cross-axis/cohesion-fitness-violation.md`
- `cross-axis/regional-pack-regulator-update.md`

### SaaS
- `saas/workflow-engine-deadlock.md`
- `saas/plugin-runtime-sandbox-escape.md`
- `saas/marketplace-listing-takedown.md`

### Workspace
- `workspace/mail-deliverability-collapse.md`
- `workspace/doc-crdt-divergence.md`
- `workspace/drive-permission-escalation.md`
- `workspace/meet-sfu-failover.md`
- `workspace/recording-archiver-stuck.md`

### Vertical (per-vertical clusters)
- `vertical-healthcare/phi-leak-suspected.md`
- `vertical-healthcare/clinical-safety-anomaly.md`
- `vertical-fintech/pci-incident-suspected.md`
- `vertical-fintech/aml-rule-fired.md`
- `vertical-fintech/cde-isolation-breach.md`
- `vertical-industrial/ot-safety-anomaly.md`
- `vertical-logistics/edi-counterparty-down.md`
- (per-vertical)

### Foundry
- `foundry/provider-quota-exhausted.md`
- `foundry/subscription-token-expired.md`
- `foundry/autonomy-ceiling-breach-attempt.md`
- `foundry/capability-eval-regression.md`
- `foundry/sandbox-escape-detected.md`
- `foundry/prompt-injection-fired.md`
- `foundry/cost-ceiling-exceeded.md`

### Cloud
- `cloud/iam-key-rotation.md`
- `cloud/kms-emergency-rotation.md`
- `cloud/root-of-trust-ceremony.md`
- `cloud/region-failover.md`
- `cloud/cell-isolation-breach.md`
- `cloud/billing-event-stream-stuck.md`
- `cloud/dcops-power-event.md` (post W-DC-Operations)
- `cloud/dcops-cooling-failure.md`

### Search
- `search/index-corruption.md`
- `search/crawler-blocked-by-host.md`
- `search/serp-quality-regression.md`
- `search/rtbf-cascade.md`

### Ads + Analytics
- `ads/auction-engine-overload.md`
- `ads/click-fraud-spike.md`
- `ads/data-use-boundary-violation.md`
- `analytics/dp-budget-exhausted.md`

### Ops
- [`runbooks/laptop-cas-gha-proof.md`](runbooks/laptop-cas-gha-proof.md) — protected GitHub-hosted
  reachability and integrity proof before remote-cache warm reads are licensed.
- `ops/sev-1-bridge-procedure.md`
- `ops/regulator-notification-procedure.md`
- `ops/trust-portal-publish-procedure.md`
- `ops/dr-drill-runbook.md`
- `ops/game-day-procedure.md`

## 3. Runbook freshness SLA

- Sev-1-supporting runbooks: tested in drill within 90 days
- Sev-2-supporting: 180 days
- Sev-3/4: 365 days
- Per-runbook `last verified` field; CI lane `runbook-discoverability` enforces freshness

## 4. Sources
[INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](SLO-CATALOG.md), `docs/runbooks/` legacy index.
