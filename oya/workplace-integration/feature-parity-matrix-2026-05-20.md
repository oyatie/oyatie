# workplace-integration feature parity matrix
Audit date: 2026-05-20
Execution date: 2026-05-21
Target µservice: `microservices/workplace-integration/`
Counterpart 1: Slack App Directory
Counterpart 2: Microsoft Teams App Store
Counterpart 3: Zapier Integrations
Scope rule: union coverage, not average coverage
Retirement rule: no retired commercial-quality ladder scaffold in this report
Tenant-class rule: evaluate uniform product quality with deployment-context and tenant-class overlays outside feature tiers
Current local verdict: REVISE

## Citation anchor block
- Current µservice README scope: `microservices/workplace-integration/README.md:14-16`.
- Current µservice PRD route scope: `microservices/workplace-integration/PRD.md:54-63`.
- Current µservice event scope: `microservices/workplace-integration/contracts/asyncapi-v1.yaml:18-52`.
- Current µservice manifest dependencies: `microservices/workplace-integration/manifest.json:47-58`.
- Chat-history product brief: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:1355` and `:1444`.
- Chat-history counterpart queue: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.
- Slack app surface source: https://slack.com/help/articles/360001537467-Guide-to-apps-in-Slack lines 41-86.
- Slack app distribution source: https://docs.slack.dev/app-management/distribution/ lines 64-105.
- Slack manifest source: https://docs.slack.dev/app-manifests/ lines 61-79.
- Microsoft Teams app surface source: https://learn.microsoft.com/pl-pl/microsoftTeams/apps-in-teams lines 84-119.
- Microsoft Teams admin/governance source: https://learn.microsoft.com/en-us/microsoftteams/manage-apps lines 32-50 and 112-132.
- Microsoft Teams publishing source: https://developer.microsoft.com/en-us/microsoft-teams/app-publishing lines 84-132.
- Zapier integration model source: https://docs.zapier.com/integrations/quickstart/how-zapier-works lines 130-147.
- Zapier trigger/action/search recommendation source: https://docs.zapier.com/integrations/quickstart/recommended-triggers-and-actions lines 162-178 and 291-299.
- Zapier trigger model source: https://docs.zapier.com/integrations/build/trigger lines 143-154.

## §1 Current Oyatie surface summary
1. The local service currently presents as a workplace agreement, e-sign, roster, regulated workforce, clock, and DLP evidence substrate.
2. README scope is explicit at `README.md:16`.
3. PRD problem statement repeats the same eight-item workplace evidence surface at `PRD.md:17-22`.
4. Functional routes cover e-sign session initiation, signature proof, offer letters, engagement agreements, roster bindings, clock events, and DLP traces at `PRD.md:54-63`.
5. AsyncAPI events cover `WorkplaceESignSessionCreated`, `WorkplaceSignatureCaptured`, `WorkplaceOfferGenerated`, `WorkplaceAgreementBound`, `WorkplaceRosterBindingGranted`, `WorkplaceClockEventAttested`, and `WorkplaceDlpTraceSealed` at `contracts/asyncapi-v1.yaml:18-52`.
6. Manifest dependencies include identity, mail, drive, workflow-engine, community, compliance, audit-chain, marketplace, payments, and tenancy at `manifest.json:47-58`.
7. The current implementation source is a Rust scaffold declaring constants and version tests at `src/lib.rs:1-56`.
8. The current service therefore has domain events that can feed integration platforms.
9. The current service does not itself provide a general app directory.
10. The current service does not provide a Teams-style app package submission path.
11. The current service does not provide a Zapier-style trigger/action/search builder.
12. The current service does not provide third-party app review or certification workflow.
13. The current service does not provide app install request/approval UX.
14. The current service does not define OAuth installation flows for external workspaces.
15. The current service does not define private app sharing or org app catalog behavior.
16. The current service does not define public listing metadata or pricing disclosure for apps.
17. The current service can plausibly own a workplace-domain integration catalog if bounded by marketplace/workflow-engine/messenger/mail handoffs.
18. The current service cannot claim union parity with Slack/Teams/Zapier from current artifacts.
19. This report treats gaps as additive surfaces, not as feature tiers.
20. The recommended product path is a bounded domain-adapter model unless leadership explicitly broadens the µservice.

## §2 Counterpart 1 capability surface — Slack App Directory
1. Slack's user-facing app surface includes app types built by Slack, third parties, or a customer's own team.
2. Slack states that how an app was built determines install, management, and interaction behavior.
3. Slack app installation can be allowed by default or restricted by owners/admins.
4. Slack says installed apps can be connected by workspace members.
5. Slack app pages expose privacy policy and security/compliance information when developers submit it.
6. Slack Marketplace discovery includes browsing by specific app or work category.
7. Slack says users can browse more than 2,000 apps in Slack Marketplace.
8. Slack supports an app browser for already installed workspace apps.
9. Slack install flow uses an Add to Slack button from app pages.
10. Slack supports app requests when a user lacks install permission.
11. Slack apps may need channel-level addition for best functionality.
12. Slack supports app Home and Messages tabs.
13. Slack supports app shortcuts.
14. Slack supports slash commands.
15. Slack supports app-posted messages in channels.
16. Slack supports app DMs.
17. Slack supports app agents and assistants on paid plans.
18. Slack app pages can declare whether the underlying service is free, free-plus-paid, paid-trial, or paid.
19. Slack distribution docs distinguish undistributed single-workspace apps from distributed apps.
20. Slack unlisted distributed apps need OAuth 2.0 installation flows for additional workspaces.
21. Slack says commercial apps intended for distribution should be submitted and approved for listing in Slack Marketplace.
22. Slack Marketplace apps are reviewed against requirements and guidelines.
23. Slack distributed apps need an onboarding flow that scales beyond a single workspace.
24. Slack distributed apps require SSL for OAuth redirects, interactivity request URLs, options load URLs, and Events API request URLs.
25. Slack apps can use app manifests as reusable JSON/YAML configuration.
26. Slack manifests can be stored in version control for collaborators.
27. Slack manifests can create development clones of production apps.
28. Slack manifests for Deno Slack SDK declare custom types, steps, workflows, and automation configuration.
29. Slack's surface is therefore equal parts catalog, app lifecycle, OAuth installation, app UX, permissions, security disclosure, and workflow automation.
30. Current workplace-integration has none of Slack's public listing or install governance mechanics.
31. Current workplace-integration has a possible Slack domain event adapter for e-sign, offer, roster, clock, and DLP events.
32. Current workplace-integration lacks Slack app manifest generation for workplace domain events.
33. Current workplace-integration lacks Slack OAuth workspace installation for tenant-bound workplace events.
34. Current workplace-integration lacks Slack app approval/request state tied to tenant policy.
35. Current workplace-integration lacks Slack app Home surfaces for HR/compliance operators.
36. Current workplace-integration lacks Slack shortcuts/slash commands for workplace actions.
37. Current workplace-integration lacks Slack Marketplace listing metadata for this µservice's app surface.
38. Current workplace-integration lacks per-app privacy/security/compliance page metadata.
39. Current workplace-integration lacks app uninstall/token revocation event handling.
40. Current workplace-integration lacks workspace-to-tenant mapping for Slack install events.
41. Slack parity minimum: a workplace app manifest for each supported action.
42. Slack parity minimum: OAuth installation and revocation flow bound to tenant identity.
43. Slack parity minimum: app request/approval state machine for restricted workspaces.
44. Slack parity minimum: event subscriptions for signature, offer, roster, clock, and DLP trace callbacks.
45. Slack parity minimum: app Home panel for pending actions and evidence review.
46. Slack parity minimum: slash command or shortcut for common workplace workflows.
47. Slack parity minimum: security/compliance metadata page that maps to local compliance packs.
48. Slack parity minimum: pricing/tenant-class disclosure that uses tenant classes, not feature tiers.
49. Slack parity risk: Slack is an interaction channel, not necessarily the ownership home for workplace records.
50. Slack parity risk: workspace-level app installation must not bypass Oyatie tenant scoping.
51. Slack parity risk: Slack rate limits require backoff and durable replay, not synchronous one-off calls.
52. Slack parity opportunity: AsyncAPI events already form a candidate Slack notification and workflow event set.
53. Slack parity opportunity: Cedar policy already requires tenant and sub-scope facts in policies.
54. Slack parity opportunity: audit-chain evidence can support app security review and compliance disclosure.
55. Slack parity verdict: not present as product surface, feasible as integration adapter.

## §3 Counterpart 2 capability surface — Microsoft Teams App Store
1. Teams users can view apps from the Teams apps store in desktop or web client.
2. Teams users can search by app name.
3. Teams users can browse by category.
4. Teams users can browse apps built for their organization.
5. Teams users can browse apps built with Power Platform.
6. Teams apps can be pinned for access.
7. Teams admins can pin apps and control pinned app behavior through setup policies.
8. Teams users can add apps from the store.
9. Teams users can add apps directly from chat, channel tab, Teams meeting, or messaging area context.
10. Teams users can add apps only when admins allow the app and make it available by policy.
11. Teams users can request approval for blocked apps.
12. Teams app developers can enhance Teams apps to work in Outlook and Microsoft 365 App.
13. Teams app capabilities include Tabs.
14. Teams tabs are Teams-aware webpages in channel, chat, or personal contexts.
15. Teams app capabilities include webhooks and connectors.
16. Teams webhooks/connectors connect web services to channels and teams.
17. Teams app capabilities include messaging extensions.
18. Teams messaging extensions let users search or initiate actions without leaving the conversation.
19. Teams app capabilities include meeting extensions.
20. Teams meeting extensions enhance live meetings.
21. Teams app capabilities include bots.
22. Teams bots support personal chat, channel, or group chat interactions.
23. Teams app capabilities include cards and task modules.
24. Teams task modules support modal pop-up task experiences.
25. Teams app capabilities include activity feeds.
26. Teams apps can broadcast messages to members of a team or channel.
27. Teams admin center supports managing agents and apps from a central Manage apps page.
28. Teams admin center can review agents and apps before allowing them.
29. Teams admin center controls app availability per user or per app.
30. Teams admin center includes Microsoft 365 certified and publisher-attested counts.
31. Teams admin center supports custom app upload and approval.
32. Teams admin center can allow or block apps.
33. Teams admin center can export the app catalog as CSV.
34. Teams publishes app validation and enforcement mechanisms.
35. Teams app developers provide support, update, security, compliance, pricing, and setup information through app detail pages.
36. Teams publishing page states Teams apps reach over 145 million daily active users.
37. Teams publishing includes review store guidelines, create developer account, submit for publishing, resolve issues, and maintain app.
38. Teams publishing page says Teams has thousands of apps.
39. Current workplace-integration lacks Teams app package manifest generation.
40. Current workplace-integration lacks Teams tab surfaces for HR/compliance workflows.
41. Current workplace-integration lacks Teams bot command surface for clock/e-sign/offer actions.
42. Current workplace-integration lacks Teams messaging extension search/action contracts.
43. Current workplace-integration lacks Teams meeting extension model for workplace approvals or onboarding meetings.
44. Current workplace-integration lacks Teams card/task module flows for e-sign and clock disputes.
45. Current workplace-integration lacks Teams activity-feed notification mapping.
46. Current workplace-integration lacks Teams admin allow/block alignment.
47. Current workplace-integration lacks Teams custom app upload guidance.
48. Current workplace-integration lacks Microsoft 365 certification/publisher-attestation metadata.
49. Current workplace-integration lacks app details page support/pricing/compliance metadata.
50. Current workplace-integration lacks AppSource publishing and maintenance lifecycle.
51. Teams parity minimum: a Teams app package for workplace-integration.
52. Teams parity minimum: tab surface for pending workplace tasks and evidence.
53. Teams parity minimum: bot surface for common workplace commands.
54. Teams parity minimum: messaging extension for document/evidence search and action.
55. Teams parity minimum: activity feed notifications for signature, roster, clock, and DLP events.
56. Teams parity minimum: task module for e-sign and clock dispute resolution.
57. Teams parity minimum: admin governance mapping from Teams allow/block to Oyatie tenant policy.
58. Teams parity minimum: certification metadata produced from compliance and DPIA artifacts.
59. Teams parity risk: Teams shared channels limit bots/connectors/message extensions according to Microsoft limits docs, so shared-channel behavior needs explicit fallback.
60. Teams parity risk: Microsoft 365 admin policy must not become a second source of truth for Oyatie tenant authorization.
61. Teams parity opportunity: current Cedar policies already have per-action names and tenant checks.
62. Teams parity opportunity: current AsyncAPI events can drive activity feed and bot notifications.
63. Teams parity opportunity: current compliance docs can feed app certification evidence once substance gaps are closed.
64. Teams parity verdict: not present as product surface, feasible as channel package with governance bridge.

## §4 Counterpart 3 capability surface — Zapier Integrations
1. Zapier states Zaps automate repetitive tasks between two or more apps.
2. Each Zap starts with a trigger.
3. Each Zap then runs one or more actions.
4. To connect an app to Zapier's 7,000+ apps, the app needs a publicly accessible API.
5. Zapier integrations are built from Authentication, Triggers, and Actions.
6. Authentication lets Zapier access user data through app credentials.
7. Triggers start Zaps when an item is added or updated in the app.
8. Actions create new or update existing items in the app.
9. New Zapier integrations use Platform v3.
10. Zapier Platform UI is an online visual builder.
11. Zapier Platform CLI is a command-line builder.
12. Zapier recommends foundational triggers, actions, searches, and search-or-create surfaces by app category.
13. Zapier says integration terminology should match the app platform so users understand the connection.
14. Zapier's documents category recommends document triggers such as new document, completed/signed document, status updated, and document sent.
15. Zapier's documents category recommends create-document actions and find-document searches.
16. Zapier's signatures category recommends document/contract sent, completed, and signed triggers.
17. Zapier's signatures category recommends create document/contract and send signature request actions.
18. Zapier polling triggers check the app endpoint every one to fifteen minutes depending on plan.
19. Zapier polling trigger endpoints must list new or updated items in reverse chronological order.
20. Zapier automatically deduplicates polling trigger data.
21. Zapier REST Hook triggers require webhook subscriptions manipulated through a REST API.
22. Zapier REST Hook triggers run near real time when an app pushes data to Zapier.
23. Zapier supports immediate webhook handshake confirmations by echoing `X-Hook-Secret`.
24. Zapier does not support additional identity verification steps beyond that handshake in the cited trigger model.
25. Current workplace-integration has APIs and events that could become Zapier triggers/actions.
26. Current workplace-integration lacks an explicit Zapier auth model.
27. Current workplace-integration lacks polling trigger endpoints sorted for Zapier.
28. Current workplace-integration lacks REST Hook subscription CRUD endpoints.
29. Current workplace-integration lacks trigger naming aligned to Zapier categories.
30. Current workplace-integration lacks action definitions such as Create E-Sign Session or Send Signature Request.
31. Current workplace-integration lacks search definitions such as Find Workplace Agreement or Find Signed Document.
32. Current workplace-integration lacks search-or-create semantics for employee, agreement, and roster binding records.
33. Current workplace-integration lacks deduplication key guidance for Zapier polling.
34. Current workplace-integration lacks Zapier hydration/file payload limits mapping for signed documents.
35. Current workplace-integration lacks private app/public app distribution distinction for Zapier.
36. Current workplace-integration lacks test monitoring/logging guidance for Zapier integrations.
37. Zapier parity minimum: OAuth or API-key auth mapped to tenant principal claims.
38. Zapier parity minimum: triggers for signature captured, document sent, document completed, offer generated, roster binding granted, clock event attested, and DLP trace sealed.
39. Zapier parity minimum: actions for create e-sign session, send signature request, generate offer letter, bind roster, create clock event, and create DLP trace.
40. Zapier parity minimum: searches for workplace agreement, e-sign session, worker roster binding, clock event, and evidence package.
41. Zapier parity minimum: REST Hook subscription lifecycle for near-real-time workflows.
42. Zapier parity minimum: polling endpoints with reverse chronological stable ordering.
43. Zapier parity minimum: idempotency and deduplication keys documented per trigger.
44. Zapier parity minimum: error taxonomy that maps Cedar denial and state conflict to Zapier user-visible errors.
45. Zapier parity minimum: operation timing within Zapier's 30-second run constraints.
46. Zapier parity minimum: payload budget alignment for signed documents and evidence hashes.
47. Zapier parity risk: Current OpenAPI audit-event mismatch would make Zapier triggers semantically wrong.
48. Zapier parity risk: Current SLO metric shifts would hide trigger/action reliability failures.
49. Zapier parity opportunity: existing OpenAPI and AsyncAPI files are natural source material for integration generation.
50. Zapier parity opportunity: existing idempotency key in OpenAPI schema at `contracts/openapi-v1.yaml:192-211` supports safe retries.
51. Zapier parity opportunity: existing audit_chain_ref fields in event payloads support external evidence verification.
52. Zapier parity verdict: partially primed by contracts/events, not productized as an integration.

## §5 UNION-coverage matrix
| # | Union capability | Slack | Teams | Zapier | Current WPI status | Evidence |
|---:|---|---|---|---|---|---|
| 1 | Public app discovery catalog | yes | yes | yes | missing | Current catalog files are internal component catalog only, inventory files 35-47. |
| 2 | App listing metadata | yes | yes | yes | missing | No app listing schema in manifest `manifest.json:1-176`. |
| 3 | App install flow | yes | yes | yes | missing | No install route in `contracts/openapi-v1.yaml:21-189`. |
| 4 | Admin allow/block governance | partial | yes | partial | missing | Cedar gates actions, but no app governance object; policies only check tenant/sub-scope at `policies/*.cedar:8-14`. |
| 5 | User app request flow | yes | yes | no | missing | No request/approval route in OpenAPI. |
| 6 | OAuth installation flow | yes | yes | yes | missing | No OAuth install artifact in contracts or manifest. |
| 7 | API-key/private app auth | no | partial | yes | missing | No Zapier/private app auth mapping. |
| 8 | App manifest/package | yes | yes | partial | missing | Proto/OpenAPI/AsyncAPI exist, no Slack/Teams/Zapier package manifest. |
| 9 | Event subscription model | yes | yes | yes | partial | AsyncAPI has events at `contracts/asyncapi-v1.yaml:18-52`. |
| 10 | Webhook subscription CRUD | partial | partial | yes | missing | No REST Hook subscription endpoints. |
| 11 | Polling trigger endpoints | no | no | yes | missing | No reverse chronological polling routes. |
| 12 | Near-real-time trigger push | yes | yes | yes | partial | AsyncAPI events exist; push adapter absent. |
| 13 | Trigger deduplication key | no | no | yes | partial | `idempotency_key` appears in commands, not trigger docs at `contracts/openapi-v1.yaml:192-211`. |
| 14 | Trigger: signature captured | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:23-27`. |
| 15 | Trigger: offer generated | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:28-32`. |
| 16 | Trigger: agreement bound | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:33-37`. |
| 17 | Trigger: roster binding granted | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:38-42`. |
| 18 | Trigger: clock event attested | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:43-47`. |
| 19 | Trigger: DLP trace sealed | yes | yes | yes | partial | Event exists at `contracts/asyncapi-v1.yaml:48-52`. |
| 20 | Action: create e-sign session | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:22-45`. |
| 21 | Action: sign session | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:46-69`. |
| 22 | Action: generate offer letter | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:70-93`. |
| 23 | Action: bind engagement agreement | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:94-117`. |
| 24 | Action: bind roster | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:118-141`. |
| 25 | Action: create clock event | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:142-165`. |
| 26 | Action: create DLP trace | yes | yes | yes | partial | Route exists at `contracts/openapi-v1.yaml:166-189`. |
| 27 | Search: workplace agreement | yes | yes | yes | missing | No GET/search endpoint in OpenAPI. |
| 28 | Search: e-sign session | yes | yes | yes | missing | Proto evidence lookup is not a full search at `contracts/workplace-integration-v1.proto:10-13`. |
| 29 | Search: document/evidence | yes | yes | yes | partial | Proto evidence request exists at `contracts/workplace-integration-v1.proto:30-41`. |
| 30 | Search-or-create | no | partial | yes | missing | No search-or-create semantics in contracts. |
| 31 | App Home/personal dashboard | yes | partial | no | missing | No frontend/app surface files. |
| 32 | Teams tab/webview | no | yes | no | missing | No Teams package or frontend web surface. |
| 33 | Slack shortcut/slash command | yes | no | no | missing | No command mapping artifact. |
| 34 | Teams messaging extension | no | yes | no | missing | No message extension schema. |
| 35 | Bot conversational actions | yes | yes | no | missing | No bot or messenger adapter artifact. |
| 36 | Cards/task modules | partial | yes | no | missing | No card/task module payload schema. |
| 37 | Activity feed notification | partial | yes | no | missing | AsyncAPI events can feed it, adapter absent. |
| 38 | App compliance disclosure | yes | yes | partial | partial | `compliance.md` exists but needs substance verification. |
| 39 | Privacy policy/DPIA disclosure | yes | yes | partial | partial | `dpia.md:16-20` states purpose but lacks detailed map. |
| 40 | Pricing or plan disclosure | yes | yes | yes | wrong model | Existing docs use retired tiers; see §3.4.T in coherence audit. |
| 41 | Tenant-class disclosure | no | no | partial | missing | No tenant-class search hits. |
| 42 | Usage metering | no | partial | yes | missing | No usage meter definition for cloud-billing. |
| 43 | Rate-limit/backoff contract | yes | yes | yes | missing | No counterpart-specific rate-limit adapter doc. |
| 44 | Durable retry/replay | yes | yes | yes | partial | `backfill-replay.md` exists; event adapters absent. |
| 45 | Install revocation/uninstall handling | yes | yes | yes | missing | No uninstall/revoke event surface. |
| 46 | Third-party app certification | yes | yes | partial | missing | No certification workflow. |
| 47 | Public/private app distinction | yes | yes | yes | missing | No private/public app classification. |
| 48 | Organization app catalog | partial | yes | partial | missing | Current catalog is internal Backstage-like component catalog. |
| 49 | Developer submission workflow | yes | yes | yes | missing | No app submission workflow in contracts. |
| 50 | App support/contact metadata | yes | yes | yes | missing | No support metadata object. |
| 51 | App health monitoring | yes | yes | yes | partial | Dashboards exist, but not app-install health. |
| 52 | User-facing error taxonomy | yes | yes | yes | partial | OpenAPI has 403/409, but no Zapier/Slack/Teams mapping. |
| 53 | Audit evidence export | partial | partial | partial | partial | Proto evidence lookup exists. |
| 54 | Compliance-pack gating | partial | partial | partial | partial | PRD names packs, tenant-class gating absent. |
| 55 | BYOK posture | partial | partial | no | missing | No BYOK references in inspected service artifacts. |
| 56 | Marketplace seller/reseller economics | no | no | yes | missing | No revenue_share tenant-class model. |
| 57 | Demo/trial cap behavior | no | no | yes | missing | No demo_trial model or OCI Always Free module. |
| 58 | Paid scale behavior | partial | partial | yes | missing | No paid tenant-class scale or billing docs. |
| 59 | On-prem app governance | no | partial | partial | wrong model | FAQ says on-prem is restricted by retired tier at `faqs/hris-engineer-faq.md:160-163`. |
| 60 | All six deployment contexts | no | partial | partial | missing | No context declarations or modules. |
| 61 | OS portability disclosure | no | no | no | missing | No supported-OS manifest. |
| 62 | Rust-only backend claim | no | no | no | present | Rust source exists and forbidden source scan found no files. |
| 63 | Event-contract consistency | yes | yes | yes | broken | OpenAPI all use same audit event while AsyncAPI has distinct events. |
| 64 | SLO evidence consistency | yes | yes | yes | broken | SLO metrics are shifted across capabilities. |
| 65 | Channel-specific UX | yes | yes | yes | missing | No Slack/Teams/Zapier UX adapter docs. |
| 66 | Domain-specific workplace flows | no | partial | yes | present | PRD has domain routes and events. |
| 67 | HRIS migration playbook | no | no | partial | present but non-counterpart | Rippling/Gusto migration exists. |
| 68 | E-sign partner migration | partial | partial | yes | partial | FAQ mentions DocuSign/AdobeSign/HelloSign at `faqs/hris-engineer-faq.md:112-115`. |
| 69 | Clock-in dispute workflow | partial | partial | yes | partial | ADR names dispute/wage-risk mechanics at `decisions/ADR-WPI-001...md:55-85`. |
| 70 | DLP trace workflow | partial | partial | yes | partial | Route/event exist, adapter absent. |

## §6 Family summary
1. Slack family summary: Slack is primarily an app-discovery, app-install, workspace-permission, OAuth, manifest, and conversation-surface ecosystem.
2. Slack parity requires workplace-integration to expose installable workplace app packages, not just workplace APIs.
3. Slack parity also requires security/compliance/pricing metadata for app listing pages.
4. Teams family summary: Teams is an app store plus admin-governed Microsoft 365 app surface with tabs, bots, messaging extensions, cards, task modules, activity feeds, and certification evidence.
5. Teams parity requires package manifests, app capabilities, admin allow/block integration, and compliance metadata.
6. Zapier family summary: Zapier is an automation integration platform where apps expose authentication, triggers, actions, searches, search-or-create flows, and REST hooks.
7. Zapier parity is closest to the current workplace-integration contracts because the service already has events and mutating actions.
8. Zapier parity still requires trigger/action/search productization, not only OpenAPI existence.
9. Current workplace-integration family summary: strong domain substrate for workplace records and evidence, weak integration marketplace surface.
10. Current workplace-integration should not pretend HRIS vendor parity equals Slack/Teams/Zapier parity.
11. Current workplace-integration can become a provider of workplace triggers/actions to `workflow-engine`.
12. Current workplace-integration can become a channel adapter source for `messenger` and `mail`.
13. Current workplace-integration can become a listing payload source for `marketplace`.
14. Current workplace-integration should not own generic app marketplace mechanics if `marketplace` already owns them.
15. Current workplace-integration should not own generic no-code execution if `workflow-engine` already owns it.
16. Current workplace-integration should not own channel UI if `messenger`, `mail`, `calendar`, or `meet` own those surfaces.
17. The right ownership model is likely domain events plus bounded app/channel adapters.
18. The wrong model would be silently expanding into a second marketplace, second workflow engine, or second messenger.
19. Union coverage therefore demands explicit cross-microservice handoff before implementation.
20. This is a coherence gap, not a reason to discard the existing workplace evidence work.

## §7 Headline gap analysis
1. Gap A: No app directory object model.
2. Evidence: `manifest.json:1-176` defines microservice metadata but no app listing, listing state, owner support metadata, permissions disclosure, category, logo, or install URL.
3. Impact: Slack and Teams marketplace parity is impossible without a listing model.
4. Additive surface: `WorkplaceIntegrationAppListing` owned or delegated through `marketplace`.
5. Gap B: No install authorization workflow.
6. Evidence: OpenAPI routes at `contracts/openapi-v1.yaml:21-189` are workplace actions, not install requests, OAuth callbacks, or admin approvals.
7. Impact: Slack/Teams install governance cannot be mapped to tenant policy.
8. Additive surface: install request, approval, deny, revoke, and uninstall events.
9. Gap C: No Zapier trigger/action/search catalog.
10. Evidence: AsyncAPI events exist at `contracts/asyncapi-v1.yaml:18-52`, but no trigger/action/search document maps them into Zapier vocabulary.
11. Impact: Zapier users cannot discover or wire the workplace workflows.
12. Additive surface: integration catalog with trigger/action/search definitions and dedupe keys.
13. Gap D: Contract event mismatch blocks automation correctness.
14. Evidence: OpenAPI repeats one audit event across all operations at `contracts/openapi-v1.yaml:28`, `:52`, `:76`, `:100`, `:124`, `:148`, and `:172`.
15. Impact: external integrations would fire the wrong workflow on offer, roster, clock, and DLP events.
16. Additive surface: route-event conformance tests.
17. Gap E: No tenant-class overlay.
18. Evidence: no tenant-class search hits; retired tier docs still exist.
19. Impact: pricing/caps/support/compliance disclosure cannot be represented in app listings or Zapier limits.
20. Additive surface: tenant-class behavior spec for `demo_trial`, `paid`, and `revenue_share`.
21. Gap F: No rate-limit/backoff adapter.
22. Evidence: no Slack/Teams/Zapier rate-limit docs under the service; existing SLOs are internally shifted.
23. Impact: integrations will fail under Slack, Teams, or Zapier throttles.
24. Additive surface: durable adapter replay design with per-counterpart limits.
25. Gap G: No governance handoff.
26. Evidence: manifest dependencies include marketplace, workflow-engine, messenger-like mail/community surfaces, and payments at `manifest.json:47-58`, but no cross-handoff doc exists.
27. Impact: implementation teams may put marketplace/workflow/channel responsibilities in the wrong µservice.
28. Additive surface: cross-microservice handoff matrix.
29. Gap H: Existing counterpart docs use the wrong family.
30. Evidence: benchmark doc names Rippling/Gusto/Workday/Justworks/Deel at `benchmarks/...md:1-5`.
31. Impact: current parity claim answers a different market question.
32. Additive surface: keep HRIS comparison as appendix and add required Slack/Teams/Zapier union matrix.
33. Gap I: No public/private app distinction.
34. Evidence: no app distribution state in manifest or contracts.
35. Impact: Slack unlisted/listed and Teams custom/store distinctions cannot be represented.
36. Additive surface: `distribution_mode` enum with tenant policy and marketplace handoff.
37. Gap J: No certification evidence workflow.
38. Evidence: compliance/DPIA docs exist, but app certification metadata is not modeled.
39. Impact: Teams publisher attestation and Slack security review evidence cannot be generated consistently.
40. Additive surface: evidence exporter from compliance, DPIA, threat model, and audit-chain docs.

## §8 Additive surface proposal
1. Add `WorkplaceIntegrationTriggerCatalog` for event-to-integration mapping.
2. Include trigger `signature_captured` mapped to `WorkplaceSignatureCaptured`.
3. Include trigger `offer_generated` mapped to `WorkplaceOfferGenerated`.
4. Include trigger `agreement_bound` mapped to `WorkplaceAgreementBound`.
5. Include trigger `roster_binding_granted` mapped to `WorkplaceRosterBindingGranted`.
6. Include trigger `clock_event_attested` mapped to `WorkplaceClockEventAttested`.
7. Include trigger `dlp_trace_sealed` mapped to `WorkplaceDlpTraceSealed`.
8. Include action `create_esign_session` mapped to `/workplace/esign/sessions`.
9. Include action `record_signature_proof` mapped to `/workplace/esign/sessions/{session_id}/sign`.
10. Include action `generate_offer_letter` mapped to `/workplace/offer-letters`.
11. Include action `bind_engagement_agreement` mapped to `/workplace/engagement-agreements`.
12. Include action `bind_roster` mapped to `/workplace/roster-bindings`.
13. Include action `record_clock_event` mapped to `/workplace/clock-events`.
14. Include action `record_dlp_trace` mapped to `/workplace/dlp-traces`.
15. Include search `find_workplace_agreement` with tenant and sub-scope filters.
16. Include search `find_esign_session` with session id, employee id, document id, and state filters.
17. Include search `find_clock_event` with employee, worksite, shift, and date filters.
18. Include search `find_evidence_package` with audit-chain reference.
19. Include REST Hook subscription endpoints for near-real-time triggers.
20. Include polling endpoints for Zapier plan-dependent polling.
21. Include Slack app manifest generation with scopes and event subscriptions.
22. Include Slack app Home summary for pending signatures, clock disputes, and DLP reviews.
23. Include Slack slash commands or shortcuts for high-frequency workplace actions.
24. Include Teams app package generation with tabs, bot, messaging extension, and activity feed channels.
25. Include Teams task module for signature and dispute completion.
26. Include app listing metadata delegated to `marketplace`.
27. Include workflow execution ownership delegated to `workflow-engine`.
28. Include channel delivery ownership delegated to `messenger`, `mail`, `calendar`, or `meet` as appropriate.
29. Include tenant authorization delegated through identity/tenancy principal claims.
30. Include billing, usage, and tenant-class caps delegated to cloud-billing and payments.
31. Include compliance/certification evidence generated from compliance, DPIA, and threat-model docs.
32. Include app uninstall and token revocation events.
33. Include rate-limit adapter policy for Slack.
34. Include rate-limit adapter policy for Teams.
35. Include rate-limit adapter policy for Zapier.
36. Include retry/replay policy tied to audit-chain evidence.
37. Include generated-client provenance for non-Rust SDK metadata if needed.
38. Include contract conformance tests that align OpenAPI route, AsyncAPI event, Cedar action, SLO metric, and runbook.
39. Include tenant-class tests for cap-hit and paid/revenue-share scaling.
40. Include deployment-context tests for all six canonical contexts once IaC exists.

## §9 Current parity verdict by family
1. Slack App Directory: FAIL as marketplace/app surface.
2. Slack App Directory: PARTIAL as future channel adapter because domain events exist.
3. Microsoft Teams App Store: FAIL as Teams app package/admin-governed app surface.
4. Microsoft Teams App Store: PARTIAL as future activity/bot/task surface because domain events and Cedar gates exist.
5. Zapier Integrations: PARTIAL as future integration because triggers/actions can be derived from contracts.
6. Zapier Integrations: FAIL as current product surface because auth, trigger/action/search catalog, REST Hook, polling, and dedupe docs are absent.
7. Overall union coverage: REVISE.
8. Existing HRIS/e-sign parity material should not be deleted during this audit.
9. Existing HRIS/e-sign parity material should be labeled as secondary evidence or moved into a domain-competitor appendix during remediation.
10. The primary parity target for this batch remains Slack App Directory / Microsoft Teams App Store / Zapier Integrations.
11. No feature tier deltas are required or authored.
12. Uniform industry-leader quality is the bar across tenant classes.
