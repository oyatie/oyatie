# Calendar Feature-Parity Matrix

Audit date: 2026-05-20.
Target microservice: `calendar`.
Counterpart 1: Google Calendar.
Counterpart 2: Microsoft Outlook Calendar.
Counterpart 3: Cal.com.
Purpose: compare the current Oyatie Calendar artifact surface against the union of the three counterpart product surfaces.
Doctrine: no customer capability ladder is introduced in this document.

Evidence anchors:
- Calendar product purpose: `microservices/calendar/PRD.md:20-26`.
- Calendar tenant outcomes: `microservices/calendar/PRD.md:30-35`.
- Calendar functional requirements: `microservices/calendar/PRD.md:41-55`.
- Calendar current competitor table: `microservices/calendar/PRD.md:227-247`.
- Calendar OpenAPI surface: `microservices/calendar/contracts/openapi/calendar.yaml:47-363`.
- Calendar AsyncAPI surface: `microservices/calendar/contracts/asyncapi/calendar-events.yaml:27-69`.
- Calendar proto surface: `microservices/calendar/contracts/proto/calendar.proto:75-199`.
- Calendar capacity envelope: `microservices/calendar/capacity-model.md:34-58`.
- Current Wave 3 counterpart assignment: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290-16311`.
- Google Calendar API quota source: `https://developers.google.com/workspace/calendar/api/guides/quota`.
- Google Calendar freebusy source: `https://developers.google.com/workspace/calendar/api/v3/reference/freebusy/query`.
- Google Calendar events source: `https://developers.google.com/workspace/calendar/api/v3/reference/events/list`.
- Microsoft Graph Outlook throttling source: `https://learn.microsoft.com/en-us/graph/throttling-limits`.
- Microsoft Graph list events source: `https://learn.microsoft.com/en-us/graph/api/calendar-list-events`.
- Microsoft Graph getSchedule source: `https://learn.microsoft.com/en-us/graph/api/calendar-getschedule`.
- Microsoft Graph findMeetingTimes source: `https://learn.microsoft.com/en-us/graph/api/user-findmeetingtimes`.
- Cal.com API v2 source: `https://cal.com/docs/api-reference/v2/introduction`.
- Cal.com booking source: `https://cal.com/docs/api-reference/v2/bookings/create-a-booking`.

## §1 Counterpart Capability Surface - Google Calendar

1. Google Calendar is the strongest baseline for consumer and Workspace calendar event CRUD.
2. Google Calendar exposes calendar resources, calendar list resources, events, ACLs, settings, colors, freebusy, push channels, and import/export-adjacent API flows through the Calendar API.
3. Google Calendar is a direct comparator for event CRUD, attendee management, recurrence, reminders, shared calendars, freebusy, ACLs, timezone behavior, and API-based synchronization.
4. Google Calendar API quota docs disclose per-minute project and per-user limits at `https://developers.google.com/workspace/calendar/api/guides/quota`.
5. Google Calendar freebusy docs disclose group and calendar expansion controls at `https://developers.google.com/workspace/calendar/api/v3/reference/freebusy/query`.
6. Google Calendar event list docs disclose pagination and maximum page size at `https://developers.google.com/workspace/calendar/api/v3/reference/events/list`.
7. The strongest Google surface for Oyatie parity is shared calendar governance plus freebusy privacy.
8. The second strongest Google surface for Oyatie parity is recurrence and event exception behavior.
9. The third strongest Google surface for Oyatie parity is high-volume API synchronization.
10. The fourth strongest Google surface for Oyatie parity is calendar ACLs and delegation.
11. The fifth strongest Google surface for Oyatie parity is push notification channels and incremental sync.

Google capability family:
12. Event create, read, update, delete.
13. Event list with pagination.
14. Event instances for recurring events.
15. Recurrence rules and recurring instances.
16. Event exceptions and detached instances.
17. Attendees, organizer, creator, guest permissions.
18. Reminders and notification defaults.
19. Attachments and conferencing metadata where Workspace allows it.
20. Calendar resource metadata.
21. Calendar list membership and hidden/selected states.
22. Calendar ACL entries.
23. Freebusy query across calendars and groups.
24. Timezone handling and RFC3339 request boundaries.
25. Import-style event creation.
26. Incremental synchronization through tokens.
27. Push notification channels.
28. Working location and presence-related event types in Workspace surfaces.
29. Out-of-office and focus-time style event types in Workspace surfaces.
30. Domain-level governance through Workspace administration.
31. Color and settings personalization.
32. Quota-backed public API operation.
33. OAuth-based delegated access.
34. Service-account domain-wide delegation for Workspace administrators.
35. Mobile, web, and native first-party clients.

Google parity implications for Oyatie:
36. Oyatie must match event CRUD and recurrence correctness.
37. Oyatie must exceed default freebusy privacy because Oyatie's PRD claims dual-context freebusy.
38. Oyatie must expose ACL semantics that can represent tenant boundaries, external invitees, and cross-tenant grants.
39. Oyatie must provide sync hooks or event streams that substitute for Google push channels.
40. Oyatie must prove timezone database refresh, pinning, and rollback.
41. Oyatie must handle large calendars and recurring-event expansion without recurrence storms.
42. Oyatie must avoid over-exposing private event metadata through freebusy projections.
43. Oyatie must support import from Google Calendar because a migration playbook already exists at `microservices/calendar/migration-playbooks/from-google-calendar.md`.
44. Oyatie must support outbound `.ics` because the PRD requires it at `microservices/calendar/PRD.md:41-55`.
45. Oyatie must provide admin-grade policy controls, not only end-user preferences.

## §2 Counterpart Capability Surface - Microsoft Outlook Calendar

1. Microsoft Outlook Calendar is the strongest baseline for enterprise calendar behavior in Microsoft 365 and Exchange Online.
2. Microsoft Graph exposes calendar events, calendars, calendar groups, calendar view, getSchedule, findMeetingTimes, places, rooms, attachments, categories, and subscriptions.
3. Microsoft Graph Outlook service limits apply per app and mailbox combination, including request and concurrency limits.
4. Outlook Calendar is a direct comparator for enterprise scheduling, resource mailboxes, rooms, group mailboxes, shared calendars, delegation, availability, and meeting-time suggestions.
5. Outlook is also a comparator for compliance posture because Exchange and Microsoft 365 environments frequently carry retention, legal hold, eDiscovery, and administrative constraints.
6. Microsoft Graph list-events docs are relevant for event retrieval parity.
7. Microsoft Graph getSchedule docs are relevant for availability parity.
8. Microsoft Graph findMeetingTimes docs are relevant for scheduling-suggestion parity.
9. Microsoft Graph service-specific throttling docs are relevant for throughput target comparison.
10. The strongest Outlook surface for Oyatie parity is enterprise scheduling plus resource-room behavior.
11. The second strongest Outlook surface for Oyatie parity is getSchedule/findMeetingTimes availability resolution.
12. The third strongest Outlook surface for Oyatie parity is compliance, retention, and legal hold adjacency.
13. The fourth strongest Outlook surface for Oyatie parity is subscription/delta synchronization.
14. The fifth strongest Outlook surface for Oyatie parity is delegated/shared mailbox governance.

Microsoft capability family:
15. Event create, read, update, delete.
16. Calendar view over time windows.
17. Calendar groups.
18. Multiple calendars per mailbox.
19. Shared calendars.
20. Delegated mailbox access.
21. Meeting invitations and RSVP flow.
22. Attachments and online meeting metadata.
23. Categories and Outlook-specific metadata.
24. getSchedule availability resolution.
25. findMeetingTimes scheduling suggestions.
26. Places and room resources.
27. Resource mailbox booking.
28. Group and team calendar contexts.
29. Subscriptions for change notifications.
30. Delta-query style synchronization in Graph calendar surfaces.
31. Compliance and legal hold integration through Microsoft 365 environment.
32. Exchange transport integration for invitations.
33. Mailbox-scoped throttling and concurrency limits.
34. Native Outlook web, desktop, and mobile clients.
35. Administrative control plane through Microsoft 365.

Microsoft parity implications for Oyatie:
36. Oyatie must treat room booking as a first-class calendar concern.
37. Oyatie must model shared resources separately from personal calendars.
38. Oyatie must preserve legal hold behavior because calendar metadata can be compliance-sensitive.
39. Oyatie must expose availability windows in a way that supports scheduling suggestions.
40. Oyatie must integrate mail invitations without creating mail loops.
41. Oyatie must preserve cross-service ownership with mail and identity.
42. Oyatie must have runbooks for mailbox-like sync and invitation dispatch failures.
43. Oyatie must prove concurrency behavior for high-volume freebusy and RSVP storms.
44. Oyatie must allow admin policies around delegation and external attendees.
45. Oyatie must have a migration story for Outlook even though the current migration playbook is Google-centered.

## §3 Counterpart Capability Surface - Cal.com

1. Cal.com is the strongest baseline for scheduling-product workflows rather than full enterprise calendar storage.
2. Cal.com API v2 docs disclose OAuth, API key, and platform authentication modes at `https://cal.com/docs/api-reference/v2/introduction`.
3. Cal.com API v2 docs disclose public and authenticated booking flows at `https://cal.com/docs/api-reference/v2/bookings/create-a-booking`.
4. Cal.com is a direct comparator for booking links, event types, availability schedules, team scheduling, routing forms, workflows, webhooks, and integration-led scheduling.
5. Cal.com is not a direct replacement for Google Calendar or Outlook Calendar as a complete enterprise calendar store.
6. Cal.com is essential because Oyatie Calendar includes external interview booking, availability grants, and scheduling workflows.
7. The strongest Cal.com surface for Oyatie parity is event-type-driven booking.
8. The second strongest Cal.com surface is team and organization scheduling.
9. The third strongest Cal.com surface is routing and assignment.
10. The fourth strongest Cal.com surface is webhook-driven automation.
11. The fifth strongest Cal.com surface is self-hostability and developer-facing API shape.

Cal.com capability family:
12. Event types.
13. Booking creation.
14. Booking cancellation and rescheduling.
15. Guest and attendee fields.
16. Custom booking fields.
17. Locations and conferencing options.
18. Team event types.
19. Organization event types.
20. Availability schedules.
21. Routing forms and routing response ids.
22. Team-member assignment.
23. Round-robin and pooled scheduling patterns.
24. Workflows and reminders.
25. Webhooks.
26. OAuth integrations.
27. API key access.
28. Managed-user and platform-style flows for existing platform customers.
29. Metadata on bookings.
30. Email verification codes for selected event types.
31. Conflict-check bypass for authenticated hosts.
32. Out-of-bounds booking bypass for authenticated hosts where supported.
33. Self-hostable product lineage.
34. Integration marketplace.
35. Public booking pages.

Cal.com parity implications for Oyatie:
36. Oyatie must distinguish calendar storage from booking product workflow.
37. Oyatie must have an explicit booking-link/event-type model if it wants Cal.com parity.
38. Oyatie's current OpenAPI has rooms, RSVP, availability, import/export, and CalDAV, but does not clearly expose event-type templates as Cal.com does.
39. Oyatie's current contracts expose cross-tenant grants, which can become a stronger enterprise version of public booking links.
40. Oyatie must add routing-form or assignment semantics if it wants Cal.com parity for team scheduling.
41. Oyatie must provide webhooks or event streams suitable for booking automations.
42. Oyatie AsyncAPI events provide a starting point for webhook parity at `microservices/calendar/contracts/asyncapi/calendar-events.yaml:27-69`.
43. Oyatie must ensure external attendee booking does not bypass tenant policy.
44. Oyatie must support migration from Cal.com only if the product target includes booking-product displacement.
45. Oyatie must not let Cal.com parity shrink the service into only booking links.

## §4 Oyatie Calendar Current Capability Surface

1. Oyatie Calendar owns native calendar storage and scheduling coordination.
2. Oyatie Calendar owns recurring-event expansion and recurrence storm prevention.
3. Oyatie Calendar owns freebusy projection across work and personal contexts.
4. Oyatie Calendar owns invitations and RSVP state.
5. Oyatie Calendar owns shared room booking and conflict detection.
6. Oyatie Calendar owns `.ics` import and export.
7. Oyatie Calendar owns CalDAV bridge behavior at the service boundary.
8. Oyatie Calendar owns timezone database refresh and rollback.
9. Oyatie Calendar owns legal hold and audit events for calendar metadata.
10. Oyatie Calendar owns cross-tenant grants for controlled freebusy disclosure.
11. Oyatie Calendar currently documents event CRUD in OpenAPI.
12. Oyatie Calendar currently documents legal hold operations in OpenAPI.
13. Oyatie Calendar currently documents recurrence expansion in OpenAPI.
14. Oyatie Calendar currently documents availability resolution in OpenAPI.
15. Oyatie Calendar currently documents cross-tenant grants in OpenAPI.
16. Oyatie Calendar currently documents room booking in OpenAPI.
17. Oyatie Calendar currently documents RSVP in OpenAPI.
18. Oyatie Calendar currently documents `.ics` import and export in OpenAPI.
19. Oyatie Calendar currently references CalDAV but lacks detailed CalDAV contract files.
20. Oyatie Calendar currently documents lifecycle events in AsyncAPI.
21. Oyatie Calendar currently documents invitation events in AsyncAPI.
22. Oyatie Calendar currently documents room-booking events in AsyncAPI.
23. Oyatie Calendar currently documents recurrence events in AsyncAPI.
24. Oyatie Calendar currently documents legal-hold events in AsyncAPI.
25. Oyatie Calendar currently documents freebusy privacy in proto.
26. Oyatie Calendar currently documents maximum attendees per freebusy query in proto.
27. Oyatie Calendar currently documents capacity envelopes in `capacity-model.md`.
28. Oyatie Calendar currently documents SLOs for agenda render, CalDAV availability, freebusy latency, ICS throughput, notification freshness, room conflict correctness, RSVP fanout, scheduling convergence, and timezone staleness.
29. Oyatie Calendar currently documents runbooks for recurrence storms, CalDAV sync loops, restore, room conflicts, RSVP storms, timezone refresh, and shared-calendar drift.
30. Oyatie Calendar currently lacks source code in the microservice path.
31. Oyatie Calendar currently lacks test code in the microservice path.
32. Oyatie Calendar currently lacks canonical per-context OpenTofu modules.
33. Oyatie Calendar currently lacks a supported-OS manifest.
34. Oyatie Calendar currently lacks tenant-class semantics.
35. Oyatie Calendar currently includes retired customer-class vocabulary in older docs.

## §5 Union-Coverage Matrix

Legend:
- `Covered-doc`: described in current calendar artifacts.
- `Covered-contract`: represented in OpenAPI, AsyncAPI, or proto.
- `Partial`: present but incomplete, stale, or lacking closure.
- `Gap`: absent from current calendar artifacts.
- `Out-of-scope`: not required unless product direction expands.

1. Event CRUD: Google strong, Microsoft strong, Cal.com adjacent, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:47-143`.
2. Event list pagination: Google strong, Microsoft strong, Cal.com booking-list adjacent, Oyatie `Partial` because event list behavior exists but pagination limits need executable proof.
3. Recurrence rules: Google strong, Microsoft strong, Cal.com recurring booking partial, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:174-177` and `contracts/asyncapi/calendar-events.yaml:27-69`.
4. Recurrence exceptions: Google strong, Microsoft strong, Cal.com partial, Oyatie `Covered-doc` through PRD recurrence requirements at `microservices/calendar/PRD.md:41-55`.
5. Recurrence storm prevention: Google opaque, Microsoft opaque, Cal.com opaque, Oyatie `Covered-doc` through runbook `microservices/calendar/runbooks/recurrence-storm.md`.
6. Attendees: Google strong, Microsoft strong, Cal.com strong for bookings, Oyatie `Covered-doc` through PRD invitations at `microservices/calendar/PRD.md:41-55`.
7. RSVP: Google strong, Microsoft strong, Cal.com booking state adjacent, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:286-295`.
8. External invitees: Google strong, Microsoft strong, Cal.com strong, Oyatie `Covered-doc` through PRD tenant outcomes at `microservices/calendar/PRD.md:30-35`.
9. Freebusy query: Google strong, Microsoft strong, Cal.com availability schedule strong, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:194-215`.
10. Freebusy privacy projection: Google partial through busy blocks, Microsoft partial through schedule details, Cal.com partial, Oyatie `Covered-contract` via `contracts/proto/calendar.proto:180-183`.
11. Cross-tenant freebusy grants: Google domain ACL adjacent, Microsoft sharing adjacent, Cal.com booking links adjacent, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:212-230`.
12. Calendar ACLs: Google strong, Microsoft strong, Cal.com organization roles adjacent, Oyatie `Partial` because Cedar policies exist but calendar-specific ACL contract needs closure.
13. Shared calendars: Google strong, Microsoft strong, Cal.com team schedules adjacent, Oyatie `Covered-doc` through shared-calendar runbook and PRD.
14. Room resources: Google resource calendars strong in Workspace, Microsoft resource mailboxes strong, Cal.com location/assignment partial, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:243-265`.
15. Room conflict detection: Google resource availability, Microsoft resource mailbox booking, Cal.com location availability, Oyatie `Covered-doc` through `slos/room-conflict-detection-correctness.openslo.yaml`.
16. Meeting-time suggestions: Google availability composition, Microsoft findMeetingTimes strong, Cal.com routing/availability strong, Oyatie `Partial` because availability exists but suggestion ranking is not explicit.
17. Booking links: Google appointment schedules, Microsoft Bookings adjacency, Cal.com strong, Oyatie `Gap` unless cross-tenant grants are intentionally promoted into booking links.
18. Event types/templates: Google appointment schedule adjacency, Microsoft Bookings service adjacency, Cal.com strong, Oyatie `Gap`.
19. Routing forms: Google gap, Microsoft Bookings adjacency, Cal.com strong, Oyatie `Gap`.
20. Round-robin scheduling: Google appointment scheduling partial, Microsoft Bookings adjacency, Cal.com strong, Oyatie `Gap`.
21. Workflow reminders: Google reminders, Microsoft reminders, Cal.com workflows strong, Oyatie `Covered-doc` through reminder requirement in `microservices/calendar/PRD.md:41-55`.
22. Webhooks/push: Google push channels strong, Microsoft subscriptions strong, Cal.com webhooks strong, Oyatie `Partial` through AsyncAPI events but no public webhook contract.
23. Incremental sync: Google sync tokens strong, Microsoft delta strong, Cal.com booking update webhooks, Oyatie `Partial` because event streams exist but sync token contracts are not explicit.
24. `.ics` import: Google import adjacent, Microsoft calendar import through client ecosystems, Cal.com connected calendar sync, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:313-333`.
25. `.ics` export: Google export/client ecosystem, Microsoft export/client ecosystem, Cal.com connected calendar output, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:313-333`.
26. CalDAV: Google limited, Microsoft primarily Exchange/Graph, Cal.com connected calendars, Oyatie `Partial` because OpenAPI points to a missing detailed contract directory.
27. JMAP calendar: Google gap, Microsoft gap, Cal.com gap, Oyatie `Out-of-scope` because ADR-CAL-0003 prioritizes CalDAV.
28. Timezone refresh: Google strong, Microsoft strong, Cal.com depends on platform, Oyatie `Covered-doc` through `runbooks/timezone-db-refresh.md` and `runbooks/tzdb-rollback.md`.
29. Timezone pinning: Google strong, Microsoft strong, Cal.com depends on platform, Oyatie `Covered-doc` through ADR-CAL-0004.
30. Legal hold: Google Workspace/Vault adjacency, Microsoft strong, Cal.com weak, Oyatie `Covered-contract` via `contracts/openapi/calendar.yaml:149-158`.
31. Audit chain: Google admin logs, Microsoft audit/compliance, Cal.com webhooks/logs partial, Oyatie `Covered-doc` through compliance and AsyncAPI legal hold events.
32. Data residency: Google regional controls depend on Workspace edition, Microsoft multi-geo controls, Cal.com deployment-dependent, Oyatie `Covered-doc` through `policy/data-residency.md` and compliance docs.
33. BYOK readiness: Google/Microsoft enterprise controls, Cal.com deployment-dependent, Oyatie `Partial` because tenant-class and key-management entitlement are not expressed.
34. Compliance packs: Google/Microsoft enterprise strong, Cal.com deployment-dependent, Oyatie `Covered-doc` through packs directory.
35. DPIA: Google/Microsoft vendor docs, Cal.com deployment-dependent, Oyatie `Covered-doc` through `dpia.md`.
36. Threat model: Google/Microsoft vendor-controlled, Cal.com open/self-host review possible, Oyatie `Covered-doc` through `threat-model.md`.
37. Capacity model: Google/Microsoft opaque public capacity, Cal.com rate limits public, Oyatie `Covered-doc` through `capacity-model.md`.
38. SLOs: Google/Microsoft service-level docs vary by contract, Cal.com public SLO limited, Oyatie `Covered-doc` through nine OpenSLO files.
39. Dashboards: Google/Microsoft admin surfaces, Cal.com product/admin surfaces, Oyatie `Covered-doc` through three dashboard JSON files.
40. Runbooks: Google/Microsoft internal, Cal.com self-host ops partial, Oyatie `Partial` because five failure-mode referenced runbooks are absent.
41. Migration from Google: Google source, Microsoft source, Cal.com possible source, Oyatie `Covered-doc` through `migration-playbooks/from-google-calendar.md`.
42. Migration from Outlook: Google not relevant, Microsoft source, Cal.com possible source, Oyatie `Gap`.
43. Migration from Cal.com: Google not relevant, Microsoft not relevant, Cal.com source, Oyatie `Gap`.
44. Developer API: Google strong, Microsoft strong, Cal.com strong, Oyatie `Covered-contract` through OpenAPI/AsyncAPI/proto.
45. Rust SDK example: Google client libraries broad, Microsoft SDKs broad, Cal.com REST examples, Oyatie `Covered-doc` through `reference-implementations/create-event-with-recurrence-rust-sdk.md`.
46. Native mobile clients: Google strong, Microsoft strong, Cal.com web-centered, Oyatie `Gap` in calendar path.
47. Web frontend: Google strong, Microsoft strong, Cal.com strong, Oyatie `Gap` in calendar path.
48. Leptos web SSR/selective hydration readiness: counterpart not relevant, Oyatie `Gap` in calendar path.
49. Swift client surface: Google/Microsoft native clients, Cal.com not central, Oyatie `Gap` in calendar path.
50. Kotlin client surface: Google/Microsoft native clients, Cal.com not central, Oyatie `Gap` in calendar path.
51. WinUI 3 client surface: Microsoft Outlook native baseline, Oyatie `Gap` in calendar path.
52. Supported OS matrix: counterpart client ecosystems broad, Oyatie `Gap` because no `supported-oses.json`.
53. OpenTofu six-context IaC: counterpart clouds opaque/self-host varies, Oyatie `Gap` because only Helm/Kustomize exists.
54. OCI Always Free profile: counterpart not comparable, Oyatie `Gap` because no `iac/oci-guest/always-free/`.
55. On-prem deployment: Google SaaS no, Microsoft Exchange/Graph hybrid through customer deployment, Cal.com self-host yes, Oyatie `Gap` until context module exists.
56. Colo deployment: Google SaaS no, Microsoft hybrid/customer infra possible, Cal.com self-host yes, Oyatie `Gap`.
57. Oyatie-as-cloud-provider deployment: counterpart not applicable, Oyatie `Gap` until `iac/oyatie-iaas/` exists.
58. Guest AWS deployment: Cal.com self-host possible, Google/Microsoft not direct, Oyatie `Gap`.
59. Guest OCI deployment: Cal.com self-host possible, Google/Microsoft not direct, Oyatie `Gap`.
60. Public-cloud managed deployment: Google/Microsoft SaaS strong, Cal.com SaaS strong, Oyatie `Partial` via Kubernetes docs but no OpenTofu context module.
61. Tenant-class controls: counterpart commercial packaging differs, Oyatie `Gap` because no tenant-class terms exist.
62. Usage caps for demo trials: Cal.com public limits and Google/Microsoft quotas exist, Oyatie `Gap` because no tenant-class usage caps.
63. Paid scaling: Google/Microsoft commercial scaling, Cal.com rate-limit increases through support, Oyatie `Partial` through capacity model but no tenant entitlement model.
64. Revenue-share economics: counterparts vary, Oyatie `Gap` because cost docs use retired labels and no billing-component vocabulary.
65. Recurrence benchmark: Google/Microsoft opaque, Cal.com opaque, Oyatie `Covered-doc` in PRD target lines `microservices/calendar/PRD.md:61-70`.
66. Event fetch benchmark: Google/Microsoft public quotas but not latencies, Cal.com API rate limits public, Oyatie `Covered-doc` in PRD target lines `microservices/calendar/PRD.md:61-70`.
67. Availability benchmark: Google freebusy limits public, Microsoft getSchedule surface public, Cal.com booking flow public, Oyatie `Covered-doc` in PRD target lines `microservices/calendar/PRD.md:61-70`.
68. ICS throughput benchmark: counterparts not public, Oyatie `Covered-doc` in PRD target lines `microservices/calendar/PRD.md:61-70`.
69. CalDAV benchmark: counterparts not direct, Oyatie `Covered-doc` in PRD target lines `microservices/calendar/PRD.md:61-70`.
70. Mail-loop prevention: Google/Microsoft transport integrated, Cal.com sends email/connected calendar updates, Oyatie `Covered-doc` through `runbooks/calendar-bridge-mail-loop-detection.md`.

## §6 Family Summary

Event storage family:
1. Google and Microsoft are mature event-store baselines.
2. Cal.com is booking-first and depends on connected calendars for much of the long-term calendar-store behavior.
3. Oyatie is product-positioned as a full event-store service.
4. Oyatie OpenAPI and proto evidence support the event-store claim.
5. Oyatie source and test absence prevents implementation-level parity validation.

Availability family:
6. Google freebusy exposes group and calendar expansion controls.
7. Microsoft getSchedule and findMeetingTimes expose availability and suggestion surfaces.
8. Cal.com availability schedules drive booking flows.
9. Oyatie availability API and freebusy proto are credible.
10. Oyatie cross-tenant privacy is a potential differentiator.
11. Oyatie must add booking-link and routing semantics if Cal.com parity is required.

Resource and room family:
12. Microsoft is the strongest comparator for room/resource scheduling.
13. Google Workspace resource calendars are a strong secondary baseline.
14. Cal.com supports locations and team assignment but is not a full facilities resource system.
15. Oyatie room-booking contract and SLO are strong.
16. Oyatie must settle ownership with a rooms microservice through a handoff file.

Protocol and migration family:
17. Google and Microsoft are API-first ecosystems.
18. Cal.com is API-first for booking and integration workflows.
19. Oyatie has OpenAPI, AsyncAPI, and proto artifacts.
20. Oyatie has Google migration docs.
21. Oyatie lacks Outlook and Cal.com migration playbooks.
22. Oyatie CalDAV detail is incomplete because the referenced contract directory is missing.

Compliance family:
23. Microsoft is the strongest compliance comparator.
24. Google Workspace is a strong compliance comparator.
25. Cal.com compliance depends on SaaS versus self-hosted deployment.
26. Oyatie compliance docs are unusually broad for this stage.
27. Oyatie must remove stale customer-class vocabulary from compliance-adjacent cost and entitlement docs.

Deployment family:
28. Google and Microsoft SaaS do not map one-to-one onto Oyatie deployment contexts.
29. Cal.com self-hostability is a useful comparator for guest, on-prem, and colo deployments.
30. Oyatie deployment ambition exceeds all three counterparts in context breadth.
31. Oyatie currently lacks the canonical OpenTofu evidence required to claim that breadth.
32. Deployment parity is the largest infrastructure gap.

## §7 Headline Gap Analysis

1. Largest product gap: event-type booking templates and routing forms against Cal.com.
2. Largest enterprise gap: Outlook-grade room/resource and shared-calendar administration closure.
3. Largest protocol gap: missing CalDAV detail contract.
4. Largest migration gap: no Outlook or Cal.com migration playbook.
5. Largest operational gap: failure-mode references to absent runbooks.
6. Largest infrastructure gap: no per-context OpenTofu modules.
7. Largest portability gap: no supported-OS manifest.
8. Largest commercial vocabulary gap: no tenant-class semantics and lingering retired customer-class language.
9. Largest executable gap: no source or tests under the microservice path.
10. Largest counterpart-lineage gap: older docs compare against Calendly and Apple Calendar while the current union uses Cal.com.

Gap severity by product family:
11. Event CRUD: low product gap, high implementation evidence gap.
12. Recurrence: low product gap, medium implementation evidence gap.
13. Availability: low product gap, medium implementation evidence gap.
14. Booking links: high product gap.
15. Routing forms: high product gap.
16. Team assignment: medium product gap.
17. Room booking: low product gap, medium handoff gap.
18. CalDAV: medium contract gap.
19. `.ics`: low contract gap, medium implementation evidence gap.
20. Legal hold: low product gap, medium cross-service evidence gap.
21. Compliance packs: low doc gap, medium entitlement gap.
22. Webhooks: medium product gap.
23. Sync tokens: medium product gap.
24. Native clients: high frontend artifact gap.
25. Web frontend: high frontend artifact gap.
26. Six-context deployment: high canonical gap.
27. OCI Always Free profile: high canonical gap.
28. Supported OS matrix: high canonical gap.
29. Rust source: medium canonical gap.
30. Tenant classes: medium canonical gap.

## §8 Additive Surface Recommendations

1. Add `contracts/caldav/` with explicit supported CalDAV methods, calendar collection behavior, sync-token semantics, recurrence limits, and error contracts.
2. Add Outlook migration playbook covering Exchange/Graph export, shared calendars, resource mailboxes, recurring exceptions, delegate access, categories, and legal hold constraints.
3. Add Cal.com migration playbook covering event types, booking links, team event types, availability schedules, routing forms, custom fields, webhooks, and connected calendar references.
4. Add booking event-type model if calendar is expected to compete directly with Cal.com.
5. Add routing-form and assignment model if team scheduling is in scope.
6. Add public webhook contract or explicitly map AsyncAPI events to customer-visible webhook subscriptions.
7. Add sync-token model for event list and CalDAV synchronization.
8. Add calendar ACL model that joins tenant-scope Cedar policy with end-user sharing semantics.
9. Add room-service handoff file covering resource ownership, constraints, conflict authority, and rollback.
10. Add mail-service handoff file covering invite dispatch, reply processing, mail-loop prevention, and bounce handling.
11. Add identity handoff file covering organizer identity, delegate authority, external guests, and service-account authority.
12. Add tenancy handoff file covering tenant-class caps, paid scaling, revenue-share treatment, and data residency.
13. Add cloud-iac handoff file covering OpenTofu module ownership and context admission.
14. Add supported-OS manifest covering backend workers, admin CLI, web frontend, and native client expectations.
15. Add OpenTofu context modules before claiming deployment parity.
16. Add OCI Always Free profile with explicit demo-trial caps.
17. Add source/test links or reclassify source/test claims as future implementation commitments.
18. Rewrite cost budget around tenant class and billing components.
19. Preserve existing runbooks but add the five missing runbooks referenced from failure modes.
20. Keep Google Calendar and Outlook Calendar as enterprise-calendar benchmarks.
21. Keep Cal.com as the scheduling-workflow benchmark.
22. Treat Calendly and Apple Calendar references as secondary background unless the counterpart set is expanded.
23. Keep cross-tenant freebusy as a differentiator because it is stronger than public busy-block sharing when correctly enforced.
24. Keep policy-backed legal hold as a differentiator because Cal.com parity alone would not require it.
25. Keep CalDAV and `.ics` as migration and interoperability differentiators.
26. Keep timezone pinning and rollback as reliability differentiators.
27. Keep incident runbooks as high-value operational content while repairing broken references.
28. Keep SLO documents, but add a runbook/SLO/dashboard map.
29. Keep proto freebusy privacy semantics, but align OpenAPI response schemas with the same privacy guarantee.
30. Add conformance tests for Google import, Outlook import, CalDAV clients, recurrence exceptions, freebusy privacy, and room conflict races.

## §9 Union-Coverage Verdict

1. Oyatie Calendar already has a coherent enterprise-calendar product thesis.
2. Oyatie Calendar already covers more compliance and operational surface than Cal.com-style scheduling alone.
3. Oyatie Calendar already has enough contract surface to justify continued investment.
4. Oyatie Calendar is not yet deployable across the six required contexts.
5. Oyatie Calendar is not yet portable across the required OS matrix.
6. Oyatie Calendar is not yet implementation-verified from the microservice path.
7. Oyatie Calendar is not yet tenant-class aligned.
8. Oyatie Calendar should not author or preserve customer capability ladder docs.
9. The correct next move is not more generic prose.
10. The correct next move is to close machine-readable context, OS, tenant, contract, source, and test evidence.
11. Google Calendar should drive event, recurrence, freebusy, ACL, and sync parity.
12. Microsoft Outlook Calendar should drive room/resource, enterprise scheduling, delegation, and compliance parity.
13. Cal.com should drive booking-link, event-type, routing, webhook, and workflow parity.
14. Calendar's additive opportunity is to combine enterprise calendar storage with policy-native booking and cross-tenant freebusy.
15. That additive opportunity remains credible only after the canonical gaps in the coherence audit are closed.
