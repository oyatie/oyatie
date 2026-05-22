---
doc_class: Onboarding
microservice: itsm
persona: itsm-admin
related_adrs: [ADR-0316, ADR-0251]
date: 2026-05-20
doc_status: published
---

# itsm — ITSM Administrator First Week

Audience: an ITSM administrator joining your tenant's IT-Operations team. You have experience with ServiceNow / Jira Service Management or similar; you may be new to oyatie.

## Day 1 — orientation + access

Morning (3 h):
1. Receive `iam` invite. Cedar role `itsm::admin` binds: `itsm::{ticket,cmdb,change,problem,kb,workflow,automation}::{read,write,configure}`, `itsm::sla::author`, `itsm::report::*`.
2. Log in to ITSM admin portal: `https://itsm-admin.<tenant>.oyatie.io`.
3. Explore the dashboard: tickets by status, SLA compliance, recent changes, CMDB health.
4. List existing service catalog items + assignment groups + SLAs.

Afternoon (4 h):
5. Read substrate primer (~ 45 min): portal → Help → "ITSM 101".
6. Read the ITIL v4 + ISO 20000-1 mapping doc (~ 30 min): portal → Help → "Standards conformance".
7. Inventory your tenant's existing ITSM data (likely from a prior tool): ticket types, custom fields, workflows, SLAs.
8. Identify migration scope: which categories to migrate first, which to defer.

Deliverable: `inventory/itsm-scope.md`.

## Day 2 — service catalog + assignment groups

Morning (4 h):
1. Define assignment groups (teams that handle tickets): Network, Helpdesk-Tier-1, Helpdesk-Tier-2, Security, Facilities, HR-IT, etc.
2. For each, bind Cedar roles: `itsm::agent::network`, `itsm::agent::tier1`, etc. Tenant members with these roles can claim + work tickets assigned to the group.
3. Configure auto-routing rules: tickets containing "VPN" → Network; containing "password reset" → Helpdesk-Tier-1; etc.

Afternoon (3 h):
4. Author the service catalog. Categories: hardware request, software request, access request, security request, facilities request, HR-IT request.
5. For each catalog item: name, description, approval workflow (if any), SLA, default assignee group, custom fields.
6. Publish v1.0 of the catalog. Test by submitting a sample request as a "regular user" Cedar role.

Deliverable: assignment groups defined + service catalog v1.0 published.

## Day 3 — SLA authoring + ticket workflow

Morning (4 h):
1. Define SLAs per ticket priority:
   - P1 (Critical): response 15 min; resolution 4 h.
   - P2 (High): response 30 min; resolution 8 h.
   - P3 (Medium): response 4 h; resolution 24 h.
   - P4 (Low): response 24 h; resolution 5 days.
2. Configure business-hours per assignment group (e.g. Tier-1 24/7; Security 09:00-17:00 weekdays).
3. Define ticket priority matrix: Impact (1-Wide / 2-Departmental / 3-Individual) × Urgency (1-High / 2-Medium / 3-Low) → Priority P1-P4.

Afternoon (3 h):
4. Author standard ticket workflow:
   - `New` → triage (auto-assignment per routing rules).
   - `In Progress` → agent working.
   - `Pending` (waiting on customer / vendor / approval).
   - `Resolved` → solution applied; customer notified.
   - `Closed` → customer confirms or 72 h post-resolved-auto-close.
5. Test the workflow: create a synthetic ticket, advance through states, verify SLA timers + escalation alerts work.

Deliverable: SLAs published + ticket workflow tested.

## Day 4 — CMDB + change management

Morning (4 h):
1. Configure CMDB CI types your tenant uses. Standard ones: Computer, Server, NetworkDevice, Application, Database, Service, License, Contract. Add custom ones per your needs.
2. Configure CI relationships: "depends_on", "runs_on", "located_in", "managed_by".
3. Run the discovery agent against your infrastructure: portal → CMDB → "Run discovery" → choose scope (e.g. one subnet). The agent uses SNMP / WMI / SSH to enumerate devices + applications.
4. Review discovered CIs; reconcile duplicates; assign owners.

Afternoon (3 h):
5. Author change-management workflow:
   - Standard changes: pre-approved templates (e.g. "Apply OS patch", "Restart non-critical service"). Auto-approved if matching template.
   - Normal changes: require CAB review. Auto-impact analysis from CMDB ("This change affects 4 services dependent on the database; potential 200 users impacted").
   - Emergency changes: E-CAB review (small subset, fast).
6. Configure CAB membership + meeting cadence (typically weekly).
7. Test: submit a standard change ticket; verify auto-approval. Submit a normal change; verify CAB review queue.

Deliverable: CMDB populated + change-management workflow tested.

## Day 5 — knowledge management + go-live

Morning (4 h):
1. Author the top-10 KB articles for your highest-volume ticket categories (password reset, VPN issues, printer issues, etc).
2. Use the KB article template: Problem statement → Symptoms → Resolution steps → Related articles → Author + version + date.
3. Configure AI-deflection (retired-advanced+ tier): when a user submits a ticket, the AI searches the KB + returns the top-3 articles. If user marks "resolved by KB", deflection success counter increments.

Afternoon (4 h):
4. Run a tabletop with your IT team: walk through a P1 ticket from creation to closure. Validate all the moving parts work together.
5. Document tenant-specific runbooks: typical ticket categories, escalation paths, ownership, after-hours coverage.
6. Communicate launch to end-users: email + Slack + intranet announcement explaining how to submit tickets, the SLAs, and what to expect.

Deliverable: KB v1.0 + tabletop complete + end-user comms sent.

## What you should know by end of week 1

- Service catalog + assignment groups + auto-routing.
- SLA authoring + priority matrix + business-hours.
- Ticket workflow + state transitions.
- CMDB CI types + relationships + discovery agents.
- Change-management workflow (standard / normal / emergency).
- KB authoring + AI-deflection.
- Reporting + dashboards.

## What you should NOT do in week 1

- Don't disable SLA tracking. SLAs are the auditable substrate for service-level management.
- Don't bypass CAB review for normal changes without an exception path documented.
- Don't auto-close tickets without customer confirmation (use the 72-h auto-close grace period instead).
- Don't author CI relationships that don't reflect reality. The CMDB is only useful when accurate.
- Don't reduce KB article quality to chase deflection rate. Bad articles harm trust faster than they save tickets.
