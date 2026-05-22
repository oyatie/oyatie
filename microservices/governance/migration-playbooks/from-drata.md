---
doc_class: MigrationPlaybook
microservice: governance
vendor: Drata
date: 2026-05-20
doc_status: published
---

# Migration playbook — Drata → oyatie governance

Audience: a security/compliance team running Drata for SOC 2/ISO 27001/HIPAA/PCI continuous compliance evidence collection. Drivers: cryptographic audit-chain source-of-truth + per-pack retention with higher-restriction-wins + multi-pack conflict resolution + sovereign-pack residency + CI-enforced governance lanes + ~3× TCO reduction at 10k employees vs Drata.

## Why this migration matters

Drata is excellent at:

- Streamlined SOC 2/ISO 27001/HIPAA evidence collection.
- Strong integrations (~ 100+ control monitors).
- Good auditor portal.
- Quick onboarding (typically green in 60-90 d).

oyatie governance adds:

- **Cryptographic audit-chain source-of-truth** (Drata uses mutable database for evidence).
- **Per-pack retention with higher-restriction-wins** (Drata has retention but conflict resolution is manual).
- **Multi-pack conflict transparency reports** (Drata doesn't formally model multi-pack conflicts).
- **Aggregation replay from audit-chain** (Drata data must be re-collected from sources if corrupted).
- **Cross-microservice evidence fan-in** (Drata integrates per-source; oyatie joins natively).
- **Sovereign-pack residency** (KR/EU/HIPAA/FedRAMP/CN).
- **CI-enforced governance lanes** (per ADR-0329 + ADR-0330 + ADR-0331 + per-microservice flat layout).
- **Per-µservice lane runtime acceleration** (paid tenant_class scaled deployment).
- **~3× TCO reduction** at 10k employees ($912k self-hosted vs ~$316k Drata; but oyatie covers broader scope).

Trade-off: Drata's 100+ pre-built integrations is a significant head-start. oyatie's plugin SDK can host most equivalent monitors, but pre-built breadth is smaller at launch.

## Step 1 — Inventory the Drata estate (≤ 1 week)

```bash
# Drata Admin → Settings → Data Export
# Or via Drata API
curl -X GET "https://api.drata.com/v1/employees?limit=200" \
    -H "Authorization: Bearer $DRATA_API_TOKEN" \
    > ./drata-export/employees.json

curl -X GET "https://api.drata.com/v1/controls?limit=500" \
    -H "Authorization: Bearer $DRATA_API_TOKEN" \
    > ./drata-export/controls.json

curl -X GET "https://api.drata.com/v1/evidence?limit=10000" \
    -H "Authorization: Bearer $DRATA_API_TOKEN" \
    > ./drata-export/evidence.json
```

Document:

- Drata frameworks subscribed (SOC 2 Type 2, ISO 27001, HIPAA, PCI DSS, GDPR, etc.).
- Drata controls + their test status (passing/failing).
- Drata integrations (AWS, GCP, Azure, GitHub, etc.) + monitor status.
- Drata users (auditors + admins + employees).
- Custom controls (org-specific).
- Active audits in progress.

Typical: 200-500 controls; 10-50 integrations; 50-500 evidence pieces per quarter.

## Step 2 — Map Drata concepts to oyatie governance (≤ 1 week)

| Drata concept | oyatie governance equivalent |
|---|---|
| Framework (SOC 2, ISO 27001, etc.) | Pack overlay (`compliance` µservice; per ADR-COMP-001) |
| Control | Lane (governance lane runtime; per IP-004) |
| Control test status (passing/failing) | Lane state (green/red); audit-chain `governance.lane.*.v1` |
| Evidence (auto-collected) | Audit-chain event from emitting µservice |
| Integration (e.g., AWS account) | µservice or external IdP federation |
| Auditor portal | Auditor query API + Cedar-scoped grants |
| Employee | identity µservice principal |
| Policy document | governance retention policy + pack metadata |
| Risk register | `governance` µservice risk-scorecard view (per ADR-GOV-001) |

## Step 3 — Data migration (≤ 2-6 weeks)

```sh
oya governance migrate import-drata \
    --tenant acme-corp \
    --drata-export-dir ./drata-export/ \
    --map-frameworks-to-packs '{
        "SOC 2 Type 2": "soc2",
        "ISO 27001": "iso27001",
        "HIPAA": "hipaa",
        "PCI DSS": "pci-dss"
    }' \
    --map-controls-to-lanes auto \
    --import-historical-evidence true \
    --throttle-rate 1000-evidence-per-min
```

The migration:

1. Creates oyatie pack subscriptions from Drata frameworks.
2. Maps each Drata control to an oyatie lane (best-effort; manual review for org-specific controls).
3. Imports historical evidence into audit-chain (with provenance metadata `imported_from: drata`).
4. Creates governance retention policies based on framework retention requirements.
5. Imports auditor accounts into identity µservice with Cedar `governance::evidence::query` permission.

Verify:

```sh
oya governance tenant stats --tenant acme-corp
# Output:
#   active_packs: [soc2, iso27001, hipaa, pci-dss]
#   total_lanes: 247
#   green_lanes: 232 (94%)
#   red_lanes: 15
#   total_evidence_imported: 4 821 (90-day window)
#   imported_from: drata
```

## Step 4 — CI lane integration (≤ 4-8 weeks)

oyatie governance lanes integrate directly with CI:

```yaml
# .github/workflows/oya-governance-lanes.yml
name: oya governance lanes
on: [push, pull_request]
jobs:
  governance-lanes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run governance lanes
        run: |
          oya governance lane run-all --tenant acme-corp --pack-set soc2,iso27001,hipaa,pci-dss
          # Fails CI if any lane is red
```

This shifts compliance evidence to be CI-enforced (per ADR-0329 + ADR-0330 + ADR-0331). Drata's monitors continue to work during migration via API bridge.

## Step 5 — Shadow run + cutover (≤ 8-12 weeks)

Phase 1 (weeks 1-4): Drata remains primary. oyatie governance ingests evidence in parallel.
Phase 2 (weeks 5-8): Auditors get oyatie portal access; can cross-reference with Drata.
Phase 3 (weeks 9-12): Per-framework cutover — Drata becomes read-only for the framework.

After cutover:

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.governance_substrate.cut_over \
    --payload '{"from":"drata","to":"oyatie","cutover_at":"2026-09-15T14:00:00Z"}'
```

## Step 6 — Drata decommission (≤ 90-180 d post-cutover)

After ≥ 90 d:

- Export final Drata state for archival.
- Cancel Drata subscription.
- Retain Drata archive read-only for legal-hold duration.

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| Drata 100+ integration breadth gap | High | Pre-audit; port top-20 integrations to oyatie governance evidence emitters; long-tail to plugin SDK |
| Historical evidence integrity (Drata's database is mutable) | Medium | Import Drata's evidence with provenance metadata; audit-chain emit for new-going-forward only |
| Auditor familiarity with Drata UX | Medium | Auditor training; provide side-by-side comparison docs; preserve URL structure where possible |
| Custom controls (org-specific) | Medium | Map to oyatie lane DSL; auto-converter best-effort |
| Continuous monitoring of cloud accounts (AWS/GCP/Azure) | Medium | Port Drata's cloud-monitor logic to oyatie µservice integrations |
| Active audit in progress during migration | High | Coordinate with audit firm; provide bridge to both systems; do not migrate during audit window |
| Risk register migration | Medium | Map to oyatie governance risk-scorecard view (per ADR-GOV-001) |
| Drata Trust Center (public-facing) | Low | oyatie has equivalent regulator/customer dashboard (paid tenant_class) |
| SOC 2 Type 2 evidence continuity (12-month observation period) | High | Preserve historical evidence by importing into audit-chain; bridge mode covers gap |
| Drata Workflow automations | Medium | Port to oyatie workflow-engine µservice |
| Per-employee compliance attestations (e.g., training completion) | Medium | identity µservice attestation primitive |
| Policy template library | Low | oyatie provides equivalent template library |
| Vendor risk management (Drata module) | Medium | oyatie's compliance µservice + tenancy µservice handle vendor relationships |
| Drata Vendor Compliance Watch | Limited | Replace with oyatie's per-pack monitoring + external vendor lookup integrations |
| HRIS integration (Workday, BambooHR via Drata) | Low | identity µservice HRIS adapter (per IP-009) |
| Pen-test result tracking | Low | governance lane: `pen-test-fresh-within-12-months` |
| Drata SaaS billing during transition | Low | Run both for 90 d; cutover at end of period |
| Cross-framework conflict (e.g., GDPR + HIPAA on same retention class) | High | This is where oyatie excels; transparency reports + higher-restriction-wins |
| Auditor access provisioning | Medium | Cedar-scoped grants + identity µservice external IdP federation for auditor SSO |
| Drata's "Compliance Dashboard" replacement | Medium | oyatie has equivalent + pack-overlay coverage view |
