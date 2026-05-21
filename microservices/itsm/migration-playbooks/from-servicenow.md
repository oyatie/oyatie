---
doc_class: MigrationPlaybook
microservice: itsm
source_vendor: ServiceNow
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# Migration Playbook — ServiceNow → oyatie itsm

Audience: an IT-Operations team currently on ServiceNow Enterprise ITSM Pro who wants to migrate to oyatie's substrate over 16-24 weeks. ServiceNow migrations are non-trivial; budget 4-6 months for a full enterprise migration.

Outcome: all tickets + workflows + CMDB + KB articles migrated, ServiceNow decommissioned, ITIL v4 + ISO 20000-1 audit continuity preserved.

## Phase 0 — discovery (weeks 1-2)

1. Inventory ServiceNow:
   - Tickets (Incidents, Requests, Changes, Problems) — table dumps via the ServiceNow REST API.
   - Workflows (Flow Designer + legacy Workflow Editor flows).
   - Custom JavaScript (Business Rules, Script Includes, UI Policies).
   - CMDB CI classes + relationships + data.
   - Knowledge base articles.
   - Service catalog items + variables.
   - SLA definitions + Service Offering definitions.
   - Integration adapters (LDAP, Active Directory, email, web services).
   - User permissions + roles + ACLs.
2. Inventory commercial exposure:
   - ServiceNow contract end date.
   - Subscription tier (Standard / Pro / Enterprise + per-module add-ons).
   - Per-user pricing + minimum-seat commit.
   - Discovery, CMDB, ITSM Pro, Customer Service, etc. modules in use.
3. Identify migration priorities + sequencing:
   - Pack-bound tenants first (KR-PIPA, CSAP, EU NIS2).
   - High-volume practices first (Incident, Service Request).
   - Long-tail (custom apps, integrations) last.

Deliverable: `migration-plan.md`.

## Phase 1 — stand up oyatie (weeks 3-4)

1. Deploy oyatie itsm IaC into the target cell.
2. Configure assignment groups + initial SLAs + service catalog skeleton.
3. Smoke-test: create sample tickets in each major category. Validate workflow transitions.

## Phase 2 — IdP sync + user migration (week 5)

1. Configure SCIM v2 sync between your IdP (Active Directory / Azure AD / Okta) and oyatie iam.
2. Map ServiceNow groups → Cedar roles. Common mappings:
   - `itil` → `itsm::agent::generic`
   - `change_manager` → `itsm::change::approver`
   - `cmdb_admin` → `itsm::cmdb::admin`
   - `service_desk` → `itsm::agent::helpdesk`
3. Run a parallel-sync for 2 weeks; verify user changes in ServiceNow are reflected in oyatie.

## Phase 3 — CMDB migration (weeks 6-9)

ServiceNow's CMDB has a rich CI class hierarchy. Map to oyatie:
- `cmdb_ci_server` → `Server` (with `os_type`, `os_version`, etc as fields).
- `cmdb_ci_computer` → `Computer`.
- `cmdb_ci_db_instance` → `Database`.
- `cmdb_ci_appl` → `Application`.
- `cmdb_ci_service` → `BusinessService`.
- `cmdb_ci_network_gear` → `NetworkDevice`.

For each CI class, run the converter:

```sh
cargo run -p oya-dev-cli -- itsm cmdb-import \
    --source servicenow \
    --table cmdb_ci_server \
    --output-mapping mappings/cmdb-ci-server.yaml
```

Run discovery against your infrastructure to validate the imported CMDB matches reality. Reconcile discrepancies.

Relationships (`cmdb_rel_ci`) port similarly with type translation:
- `Depends on::Used by` → `depends_on` / `used_by`
- `Runs on::Runs` → `runs_on` / `hosts`

## Phase 4 — workflow translation (weeks 10-14)

ServiceNow workflows use Flow Designer (newer) or legacy Workflow Editor. oyatie uses declarative workflow JSON.

For each ServiceNow workflow:
1. Document the stages + decision points + tasks.
2. Translate Business Rules (server-side JavaScript) — most port to oyatie's workflow steps as HTTP calls or Cedar policy evaluations.
3. Translate UI Policies (client-side JavaScript form behavior) — port to oyatie's form-level validation rules.
4. Test the workflow end-to-end with synthetic data.

Plan: 1-3 days per simple workflow; 1-2 weeks per complex workflow with extensive custom JavaScript.

## Phase 5 — KB + Service Catalog migration (week 15)

1. Export KB articles: ServiceNow API `kb_knowledge` table.
2. Import to oyatie:
   ```sh
   cargo run -p oya-dev-cli -- itsm kb-import --source servicenow --input kb-export.json
   ```
3. Re-link KB articles to their new ticket categories.
4. Service Catalog items: similar process for `sc_cat_item` + variables.

## Phase 6 — historical ticket archive (week 16)

For tickets older than the cutover date:
1. Export from ServiceNow via API.
2. Import as read-only archived tickets to oyatie.
3. Cross-emit Merkle anchors to audit-chain.
4. Preserve ServiceNow sys_id as `legacy_id` for back-reference.

## Phase 7 — cutover (weeks 17-19)

1. 2-week dual-system parallel operation (ServiceNow active, oyatie shadowing).
2. Day-of-cutover: disable ServiceNow ticket creation; redirect users to oyatie portal.
3. Monitor 1 week for user adoption + bug reports.
4. Open follow-up issues for any missing functionality.

## Phase 8 — ServiceNow wind-down (weeks 20-24)

1. Cancel ServiceNow contract per minimum-commit.
2. Pay residuals.
3. Update tenant ARCHITECTURE.md.

## Common pitfalls

| Pitfall | Mitigation |
|---|---|
| ServiceNow Business Rules with complex JavaScript | Budget 50-100 % more time than estimated; some need partial reimplementation in workflow steps |
| ServiceNow Update Sets versus oyatie's IaC | oyatie config is git-version-controlled; ServiceNow Update Sets must be flattened first |
| ServiceNow's "platform" features (UI Builder, App Engine Studio) | These have no direct oyatie equivalent; build custom UIs as separate Smart-on-oyatie apps |
| ServiceNow CMDB Identification + Reconciliation Engine | oyatie's de-duplication is similar but rule format differs; document each existing IRE rule + reimplement in oyatie |
| ServiceNow's Customer Service Management module | Out of scope for `itsm` µservice; CSM workflows go into the `crm` µservice |
| Custom UI macros / HTML widgets | Reimplement as Grafana panels or custom HTML in the oyatie self-service portal |
| Performance regression on high-volume tenants | Plan a 2-week shadow + performance test before cutover; tune ES + Postgres before going live |
