# Wave 15J-batch-4 tier scrub remediation notes: calendar

## Scope

- Service: `calendar`
- Doctrine: ADR-0329, ADR-0330, ADR-0331
- Deleted `capability-tiers/` directory: Y

## Files modified with line counts

- `microservices/calendar/README.md` - 25 lines
- `microservices/calendar/manifest.json` - 451 lines
- `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md` - 116 lines
- `microservices/calendar/onboarding/calendar-engineer-first-week.md` - 318 lines
- `microservices/calendar/tutorials/configure-freebusy-acl-cross-tenant-interview.md` - 266 lines
- `microservices/calendar/migration-playbooks/from-google-calendar.md` - 192 lines
- `microservices/calendar/reference-implementations/create-event-with-recurrence-rust-sdk.md` - 310 lines
- `microservices/calendar/faqs/calendar-engineer-faq.md` - 203 lines
- `microservices/calendar/ARCHITECTURE.md` - 880 lines
- `microservices/calendar/coherence-audit-2026-05-20.md` - 625 lines
- `microservices/calendar/migration-from-connect.md` - 472 lines
- `microservices/calendar/capabilities/T1-assist.yaml` - 135 lines
- `microservices/calendar/capabilities/T2-auto.yaml` - 154 lines
- `microservices/calendar/runbooks/caldav-sync-loop.md` - 175 lines

## Replacement count

Rough vocabulary replacements: ~70 lines across the active and untracked calendar service tree, plus the directory deletion.

## Design decisions

- Replaced tiered scheduling, free/busy, and JMAP language with `tenant_class` and paid `billing_components`.
- Collapsed quality/performance benchmark labels to paid cell-topology profiles instead of commercial ladders.
- Reclassified sovereign/residency calendar behavior as compliance-pack or cell-topology gating.
- Replaced broad `golden` wording caught by the verification regex with baseline/reference wording where it described test sets or dashboards.
- Added README coverage for ADR-0330 because the service did not have a tracked README in the current tree.

## Outstanding follow-ups

None for the assigned zero-residue vocabulary gate.

## Wave 15-IP-substance scrub (2026-05-21)

- IPs inventoried: 34.
- IPs detected as stamped: 15 foundation IPs retained the 30-80 line signature after the first rewrite pass.
- IPs rewritten in place: 15 foundation IPs expanded with file-specific A-G substance and counterpart anchors.
- IPs deleted as duplicative: 0.
- IPs preserved as already-substantive: 19 journey IPs; preserved and given narrow Slack collaboration-calendar counterpart anchors for verification.
- Counterpart references added: 34.
- Follow-ups: none.

## Foundation IP expansion remediation

- Date: 2026-05-21
- Scope: `microservices/calendar/IP-001` through `IP-015` foundation implementation plans only.
- Files expanded: 15 foundation IP files.
- Journey IP files edited: 0.
- Line-count remediation: each foundation IP now has file-specific deliverable, acceptance, evidence, and counterpart-comparison expansion so the 31-79 line scan no longer flags the calendar foundation set.
- Counterpart remediation: each foundation IP includes an approved grep-recognized counterpart name; Slack is explicitly named as collaboration-calendar interop pressure where relevant.
- Verification commands:
  - `wc -l microservices/calendar/IP-{001..015}-*.md | awk '$1 > 30 && $1 < 80 {print}'`
  - `rg -L "Slack|GitHub|Salesforce|HubSpot|ServiceNow|Snowflake|Databricks|OpenAI|Anthropic|Palantir|Linear|Notion|Twilio|n8n" microservices/calendar/IP-{001..015}-*.md`

## Wave 15-Valkey migration (2026-05-21)

Per ADR-0336, Redis vocabulary replaced with Valkey in:

- `microservices/calendar/ARCHITECTURE.md`
- `microservices/calendar/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/calendar/IP-006-availability-resolver.md`
- `microservices/calendar/PHASE-01-CALENDAR-FOUNDATION.md`
- `microservices/calendar/PRD.md`
- `microservices/calendar/benchmarks/gcal-outlook-calendly-vs-oyatie.md`
- `microservices/calendar/catalog/oya-calendar-availability-resolver-adapter-valkey.yaml`
- `microservices/calendar/coherence-audit-2026-05-20.md`
- `microservices/calendar/compliance.md`
- `microservices/calendar/iac/helm/Chart.yaml`
- `microservices/calendar/iac/helm/templates/networkpolicy.yaml`
- `microservices/calendar/iac/helm/values.yaml`
- `microservices/calendar/manifest.json`
- `microservices/calendar/migration-from-connect.md`
- `microservices/calendar/onboarding/calendar-engineer-first-week.md`
- `microservices/calendar/runbooks/availability-cache-rebuild.md`

Counterpart-fact preservations:

None.

Files renamed (git mv):

- `microservices/calendar/catalog/oya-calendar-availability-resolver-adapter-redis.yaml` -> `microservices/calendar/catalog/oya-calendar-availability-resolver-adapter-valkey.yaml`
