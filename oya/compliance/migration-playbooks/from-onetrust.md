---
doc_class: MigrationPlaybook
microservice: compliance
vendor: OneTrust Privacy & Data Governance
date: 2026-05-20
doc_status: published
---

# Migration playbook — OneTrust → oyatie compliance

Audience: a privacy/compliance team running OneTrust Privacy & Data Governance for multi-jurisdiction privacy management. Drivers: deterministic 6-step pack precedence + immutable versioned packs + cryptographic audit-chain + multi-pack conflict resolution with transparency reports + sovereign-pack residency + ADR-0304 cross-jurisdiction authority + ~50% TCO reduction at enterprise scale.

## Why this migration matters

OneTrust is excellent at:

- Broad regulatory coverage (60+ frameworks).
- Mature DSAR management.
- Privacy impact assessment templates.
- Cookie consent + cross-site privacy.
- Auditor portal + collaboration features.

oyatie compliance adds:

- **Deterministic 6-step precedence** with Cedar-coded enforcement (OneTrust uses configuration-driven resolution).
- **Immutable versioned packs** with hotfix superseding (OneTrust's rules are mutable).
- **Cryptographic audit-chain** (OneTrust's audit trail is server-mutable).
- **Multi-pack conflict transparency reports** with legal basis + appeal pathway.
- **Cross-microservice DSAR fan-in** (OneTrust integrates per-source; oyatie joins natively).
- **Sovereign-pack residency** (paid compliance_pack).
- **ADR-0304 cross-jurisdiction conflict authority** (paid tenant_class with required billing_components or compliance_pack).
- **~50% TCO reduction** at 10k employees (oyatie paid dedicated-cloud self-hosted ~ $892k vs OneTrust ~ $1.4M).
- **EU AI Act Annex III refusal pipeline** (paid tenant_class with required billing_components or compliance_pack).

The trade-off: OneTrust's broad regulatory coverage (60+ frameworks) is more extensive than oyatie's 30 packs at paid compliance_pack. For tenants with very-niche jurisdictional requirements, OneTrust may have packs oyatie doesn't yet support. Plan for parallel use for un-covered jurisdictions during transition.

## Step 1 — Inventory the OneTrust estate (≤ 2 weeks)

```bash
# OneTrust Admin → Settings → Data Export
# Or use OneTrust API
curl -X GET "https://acme-corp.my.onetrust.com/api/datasubject/v3/dsar/requests" \
    -H "Authorization: Bearer $ONETRUST_API_TOKEN" \
    > ./onetrust-export/dsar-requests.json

curl -X GET "https://acme-corp.my.onetrust.com/api/privacymanagement/v1/policies" \
    -H "Authorization: Bearer $ONETRUST_API_TOKEN" \
    > ./onetrust-export/policies.json

curl -X GET "https://acme-corp.my.onetrust.com/api/datamap/v3/processing-activities" \
    -H "Authorization: Bearer $ONETRUST_API_TOKEN" \
    > ./onetrust-export/processing-activities.json
```

Document:

- Active OneTrust frameworks subscribed (GDPR, CCPA, HIPAA, etc.).
- DSAR queue (typical mid-size: 50-500 DSARs/year).
- Active DPIAs.
- Privacy policies (customer-facing).
- Data Subject Access Records (DSAR-RoPA).
- Cookie consent configurations.
- Vendor risk management records.
- Active breach notifications.
- Auditor + DPO user accounts.
- Custom risk-assessment templates.

Typical mid-size: 1k-10k employees, 50-500 DSARs/year, 20-100 DPIAs, 60+ frameworks subscribed.

## Step 2 — Map OneTrust concepts to oyatie compliance (≤ 1 week)

| OneTrust concept | oyatie compliance equivalent |
|---|---|
| Framework (GDPR, CCPA, etc.) | Pack overlay (per ADR-COMP-001) |
| Privacy Policy | Tenant-level public-facing policy + Cedar policy fragment |
| Data Subject Access Request (DSAR) | DsarRequest (per IP-003) |
| Data Privacy Impact Assessment (DPIA) | DpiaRequest (per IP-DPIA-001) |
| Records of Processing Activity (RoPA) | RecordsOfProcessing (per Art 30) |
| Cookie Consent Configuration | Per-tenant consent management config |
| Vendor Risk Management | Vendor + tenancy µservice relationship records |
| Auditor User Account | identity µservice principal with `governance::evidence::query` + `compliance::regulator_request::*` permissions |
| DPO User Account | identity µservice principal with DPO role + Cedar permits |
| Custom Risk Assessment Template | DPIA template registry |
| Processing Activity | Data flow tagged with data_class + pack_set |
| Breach Notification | Per-pack breach notification clock (per Art 33 / 60-d / etc.) |
| Cross-border Transfer | Cross-jurisdictional transfer evidence (PIPL Art 38 / GDPR Art 49) |

## Step 3 — Data migration (≤ 4-12 weeks)

```sh
oya compliance migrate import-onetrust \
    --tenant acme-corp \
    --onetrust-export-dir ./onetrust-export/ \
    --map-frameworks-to-packs '{
        "GDPR": "gdpr",
        "CCPA": "ccpa",
        "HIPAA": "hipaa",
        "PCI DSS": "pci-dss",
        "LGPD": "lgpd",
        "PIPEDA": "pipeda"
    }' \
    --import-dsar-history true \
    --import-dpia-history true \
    --import-vendors-as-tenancy-relationships true \
    --throttle-rate 100-records-per-min
```

The migration:

1. Creates oyatie pack subscriptions from OneTrust frameworks.
2. Migrates historical DSARs (preserved as imported_from=onetrust evidence).
3. Migrates historical DPIAs.
4. Maps vendors to tenancy µservice parent-child relationships.
5. Maps RoPA to processing-activity records.
6. Imports cookie consent configurations.
7. Maps auditor + DPO accounts to identity µservice with Cedar grants.

Verify:

```sh
oya compliance tenant pack-list --tenant acme-corp
# Output:
#   active_packs: 12 (gdpr, ccpa, hipaa, pci-dss, lgpd, pipeda, ...)
#   imported_dsars: 487
#   imported_dpias: 142
#   imported_from: onetrust
```

## Step 4 — Custom risk assessment template migration (≤ 4-8 weeks)

OneTrust supports highly-customized risk assessment templates. Map to oyatie DPIA template registry:

```sh
oya compliance dpia-template import-onetrust \
    --tenant acme-corp \
    --template-export ./onetrust-export/templates/ \
    --map-categories auto
```

Templates that require deep customization may need manual port. Use `oya compliance dpia-template convert` for stubs.

## Step 5 — Pack publishing for any custom packs (≤ 2-4 weeks)

OneTrust may have org-specific custom frameworks. Convert to oyatie pack overlays:

```sh
oya compliance pack publish \
    --pack-id acme-custom-internal-privacy \
    --version 2026.05.20 \
    --jurisdiction acme-corp-internal \
    --rules-file ./acme-custom-rules.yaml \
    --cedar-policies-file ./acme-custom-policies.cedar \
    --legal-basis "Internal Acme Privacy Policy 2026 + signed Code of Conduct" \
    --requesting-principal u-compliance-owner@acme-corp.com
```

## Step 6 — Shadow run + cutover (≤ 8-16 weeks)

Phase 1 (weeks 1-4): OneTrust remains primary. oyatie compliance ingests in parallel.
Phase 2 (weeks 5-8): DSAR processing dual-track (both systems receive; oyatie executes).
Phase 3 (weeks 9-12): DPIAs migrate; auditor access shifts.
Phase 4 (weeks 13-16): Vendor risk management + RoPA.

```sh
oya audit emit \
    --tenant acme-corp \
    --event-class governance.compliance_substrate.cut_over \
    --payload '{"from":"onetrust","to":"oyatie","cutover_at":"2026-09-15T14:00:00Z"}'
```

## Step 7 — OneTrust decommission (≤ 180-365 d post-cutover)

After ≥ 180 d:

- Export final OneTrust state for archival.
- Cancel OneTrust subscription.
- Retain OneTrust archive read-only for legal-hold + audit duration (typically 6-7 y depending on pack).

## Risk register

| Risk | Severity | Mitigation |
|---|---|---|
| OneTrust 60+ framework gap (oyatie has 30 at paid compliance_pack) | High | Pre-audit; for un-covered jurisdictions, dual-run with OneTrust for the gap |
| DSAR workflow customization | High | Auto-converter handles ~ 70% of OneTrust DSAR workflow patterns; manual port for complex flows |
| OneTrust Privacy Policy templates | Medium | Auto-import; oyatie has template registry; some manual review |
| Cookie consent integration | Medium | oyatie has equivalent (consent management primitive); re-configure per tenant property |
| Vendor risk assessments | Medium | Map to tenancy µservice relationships; preserve assessment evidence |
| Historical DSAR + DPIA audit-trail | High | Import with provenance metadata; audit-chain emit imported_from for forensic clarity |
| OneTrust auditor portal UX | Medium | oyatie portal at paid tenant_class; provide side-by-side comparison + training |
| DPO workflow + dashboards | Medium | paid on-prem-connected cell_topology provides per-tenant DPO workspace (per IP-008-pii-scrubber + IP-009-retention-tier-policy) |
| Customer-facing privacy URLs | Low | DNS pointer to oyatie tenant-public privacy page |
| Cross-jurisdictional transfer assessments | High | oyatie's transfer evidence pipeline (paid tenant_class with required billing_components or compliance_pack); preserve OneTrust SCC docs |
| Data Inventory + RoPA | Medium | Import RoPA into oyatie processing-activity registry |
| Active audits in progress | High | Do not migrate during audit window; bridge mode through audit completion |
| Multi-language privacy policies | Low | oyatie supports per-tenant locale + multi-language templates |
| Vendor + processor onboarding workflows | Medium | tenancy µservice relationship + permit flows replace |
| Breach notification timer continuity | High | Preserve active breach clocks during migration; pack-specific timer continues |
| Custom data subject rights (e.g., LGPD vs GDPR rights nuance) | Medium | Pack-specific DSAR rules; oyatie supports multiple rights primitives |
| OneTrust SaaS billing transition | Low | Run both for 60-90 d; cutover invoicing at end of period |
| Sub-processor consent records | Medium | Import via tenancy µservice + identity µservice consent flows |
| Compliance certification continuity during transition | High | Pre-coordinate with audit firm; dual-attestation during transition |
| OneTrust integrations (e.g., Salesforce, Workday) | Medium | Re-integrate via oyatie plugin SDK + SCIM + webhook |
| Risk register migration | Medium | governance µservice risk-scorecard view (per ADR-GOV-001) |
| Trust center page (customer-facing certifications) | Low | oyatie tenant-public trust center (paid tenant_class with required billing_components or compliance_pack) |
| Custom dashboards + reports | Medium | oyatie compliance dashboard with regulator-portal customization |
