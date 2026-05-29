---
doc_class: MigrationPlaybook
microservice: finops-portal
vendor: Apptio Cloudability (IBM Apptio Cost Management SaaS)
date: 2026-05-20
doc_status: published
---

# Migration playbook — Apptio Cloudability → oyatie finops-portal

Audience: a tenant or oyatie internal FinOps team moving from Apptio Cloudability (the SaaS) to oyatie's `finops-portal` µservice. Common drivers: (a) Apptio's per-scope-per-month pricing scaling unsustainably above 1 000 tenants, (b) the desire to integrate cost data with the rest of oyatie's audit-chain + sovereignty model, (c) multi-cloud + on-prem scope coverage gaps in Apptio's native connectors.

## Why this migration matters

Apptio Cloudability is mature in:

- Per-scope cost attribution + chargeback workflows.
- AWS / GCP / Azure connectors.
- FOCUS spec v1.0 export.

oyatie `finops-portal` adds:

- Native ClickHouse OLAP backend (3-10× faster dashboards).
- Native 15-min refresh at paid with per_usage billing_component (vs Apptio's daily ETL).
- Cryptographic audit-chain integration (every dashboard render → audit event).
- Sovereign-pack residency (paid with compliance_pack gating tenant_class).
- No per-tenant licensing fee (cost scales with hardware, not tenant count).

## Step 1 — Inventory the Apptio Cloudability setup (≤ 1 week)

```bash
# From Apptio admin:
# 1. Export the "Cloud Account Inventory" report
# 2. Export the "Cost Allocation Hierarchy" with all parent/child relationships
# 3. Export the active "Budgets" + "Anomaly Rules"
# 4. Export the active "Cost Categories" + their rules
# 5. Export the active "Dashboards" + "Reports" + "Saved Views"
```

Document:

- Cloud accounts connected (AWS account IDs, GCP project IDs, Azure subscription IDs).
- Cost-allocation hierarchy depth + node count (typical: 4-6 levels, 80-500 nodes).
- Active budgets (typical: 20-50).
- Active anomaly rules (typical: 5-20).
- Custom cost categories (typical: 10-40).
- Active dashboards (typical: 30-100).
- Active users + their Apptio role assignments.

Typical mid-enterprise Apptio install: $5-20 M annual cloud spend tracked, 100-500 cost-allocation nodes, 100-300 active users.

## Step 2 — Map the cost-allocation hierarchy (≤ 1-2 weeks)

oyatie models cost centers as a 2-level tree (root + sub-centers). For deeper Apptio hierarchies, flatten OR re-model as nested cost-center policies.

Use the schema converter:

```sh
oya finops-portal migrate convert-hierarchy \
    --source-format apptio-cloudability \
    --input apptio-hierarchy-export.csv \
    --output oyatie-cost-centers.yaml \
    --max-depth 2 \
    --flatten-strategy "merge-grandchildren-into-children"
```

The output is the YAML chargeback policy file. Review manually for any hierarchies that lost semantic meaning during flattening.

## Step 3 — Migrate cost-category rules (≤ 1 week)

Apptio cost categories use rule expressions like "Tag(Project)=ML AND Service=AWS-EC2". Convert to oyatie tag-allocation rules:

```sh
oya finops-portal migrate convert-cost-categories \
    --source-format apptio-cloudability \
    --input apptio-cost-categories-export.json \
    --output oyatie-allocation-rules.yaml
```

oyatie tag-allocation supports:

- Direct-tag attribution (resource tag = cost-center).
- Allocation-policy fallback (untagged resources → policy).
- Composite rules (multiple tag predicates + AND/OR).

Apptio's rule expressiveness is broader (regex, parent-class inheritance). Some rules require manual rewrite. Typically 80-90 % auto-convert; 10-20 % need human review.

## Step 4 — Migrate budgets + anomaly rules (≤ 3 days)

```sh
oya finops-portal migrate import-budgets \
    --source-format apptio-cloudability \
    --input apptio-budgets-export.csv \
    --tenant acme-corp

oya finops-portal migrate import-anomaly-rules \
    --source-format apptio-cloudability \
    --input apptio-anomaly-rules-export.csv \
    --tenant acme-corp
```

Apptio's anomaly rules are typically threshold-based (`if cost > $X over period`). oyatie's anomaly detection is STL + Holt-Winters + 3σ residual — it triggers without explicit thresholds.

Migration policy: convert Apptio threshold rules to oyatie *budget* rules; let oyatie's statistical anomaly detection run alongside. After 30 d of operating both, evaluate whether the legacy thresholds are still adding signal beyond the statistical detector. Typically they're not, and you retire the thresholds.

## Step 5 — Cost data backfill (≤ 1-3 weeks per 1 PB cost data)

```sh
oya finops-portal migrate import-cost-history \
    --source-format apptio-cloudability \
    --input apptio-cost-history.parquet \
    --tenant acme-corp \
    --since 2023-01-01 \
    --until 2026-05-20
```

The converter handles:

- Apptio's "EffectiveCost" → oyatie's `effective_cost` (with commitment-attribution preserved).
- Apptio's cost-category tags → oyatie's `tag_set` rows.
- Apptio's allocation-rule applied → oyatie's `cost_event_allocated` rows.
- Apptio's currency display rules → oyatie's tenant-base-currency setting.

Backfill rate: ~ 200 GiB/h on paid with per_seat billing_component, ~ 600 GiB/h on paid with per_usage billing_component (multi-shard ClickHouse). For 3 years × 5000 tenants × 10 services × hourly = ~ 1.3 PiB total cost-event data, plan 1-3 weeks for backfill.

## Step 6 — Dashboard migration (≤ 4-8 weeks)

Apptio dashboards do NOT auto-convert to oyatie dashboards. Manual rewrite is required because:

- Apptio's chart authoring uses its own DSL (TBM4Cloud).
- oyatie uses YAML panel-definitions + ClickHouse SQL.

Rewriting strategy:

1. Inventory all Apptio dashboards used in the last 90 d (older ones likely abandoned).
2. Group by "essential" (used by ≥ 5 users) vs "personal" (used by ≤ 5 users).
3. Rewrite essential dashboards first; let personal dashboards age out.

Typical timeline: 2-3 days per 10 essential dashboards; 4-8 weeks for the full migration.

## Step 7 — Shadow run + cutover (≤ 4-8 weeks)

```sh
oya finops-portal reconcile \
    --source-a apptio-cloudability \
    --source-b oyatie-finops \
    --tenant acme-corp \
    --window-day 2026-05-20 \
    --report ./reconciliation.json
```

The reconciliation compares per-cost-center daily totals + identifies drift sources. Acceptable cutover drift: < 0.5 % per day on aggregate cost (cost calculation has rounding + tag-rule differences; small drift is expected).

After 4-8 weeks of clean reconciliation, cut over:

```sh
oya governance set-config \
    --tenant acme-corp \
    --key default_finops_provider \
    --value oyatie

oya audit emit \
    --tenant acme-corp \
    --event-class governance.finops_substrate.cut_over \
    --payload '{"from":"apptio-cloudability","to":"oyatie","cutover_at":"2026-05-20T14:00:00Z"}'
```

## Step 8 — Apptio decommission (≤ 60 d post-cutover)

Keep Apptio active read-only for 60 d post-cutover so users can still query historical state. After 60 d:

- Export final Apptio state for archival.
- Cancel Apptio subscription per contract notice period.
- Emit `governance.finops_substrate.decommissioned`.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Cost-allocation hierarchy too deep to flatten | High | Pre-decide on 2-level flattening before backfill; document semantic loss |
| Dashboard migration takes longer than expected | Medium | Plan 8 wk minimum; let personal dashboards age out |
| Apptio rule expressiveness exceeds oyatie | Medium | Document the 10-20% rules that need rewrite; provide white-glove migration support |
| Cost-data backfill exhausts ClickHouse capacity | High | Pre-size the ClickHouse cluster for 1.5× expected historical data |
| Apptio API rate-limits slow export | Low | Schedule export over weekends |
| Users resist new dashboard UX | Medium | Run user training sessions; provide side-by-side comparison docs |
| Reconciliation drift > 0.5 % blocks cutover | High | Investigate root cause; often: tag rules + commitment-attribution differences |
| Anomaly detection signal differs between platforms | Low | Operate both for 30 d; evaluate whether to retire legacy thresholds |
| Cross-cloud connector parity gap | Medium | Pre-validate AWS / GCP / Azure / on-prem connectors on a sample dataset |
| Apptio's "TrueValue Cloud Index" external feed not replicable | Low | This is an Apptio-proprietary benchmark; oyatie doesn't replicate. Tenants who need it can keep an Apptio Lite subscription |
