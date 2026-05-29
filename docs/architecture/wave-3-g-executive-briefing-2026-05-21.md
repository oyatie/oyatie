---
id: ARCH-WAVE-3-G-EXECUTIVE-BRIEFING-2026-05-21
title: Wave-3-G Executive Briefing
doc_class: ExecutiveBriefing
shape: Narrative
status: Proposed
date: 2026-05-21
authority_tier: 2
audience: board-director / venture-capital / sales-leader / marketing-lead / GTM team
line_floor: 1500
line_ceiling: 2500
purpose: >
  A non-technical narrative summary of what oyatie is after Wave-3-G's documentation
  expansion: the unified-ecosystem thesis, the fragmentation tax it dissolves, the
  scope and persona reach, the compliance posture, the doctrine cluster, the roadmap,
  the competitive landscape, and what it all means for board, investors, and GTM.
related_adrs:
  - ADR-0244-tenant-as-universal-scoping-primitive.md
  - ADR-0245-substrate-vs-product-layering.md
  - ADR-0249-multi-category-marketplace-doctrine.md
  - ADR-0297-abuse-defence-baseline-anti-bot-spoof-scrape.md
  - ADR-0298-emergency-services-bypass-life-safety.md
  - ADR-0299-account-recovery-resilience.md
  - ADR-0311-dual-tenant-identity-personal-vs-work-boundary.md
  - ADR-0313-conglomerate-tenant-hierarchy-sovereign-children.md
  - ADR-0314-marketplace-as-universal-deal-settlement.md
  - ADR-0315-erp-coverage-doctrine-sap-parity.md
  - ADR-0316-capability-tier-over-product-fragmentation.md
  - ADR-0317-role-based-projection-unified-ux-shell.md
  - ADR-0318-collar-color-workspace-universality.md
  - ADR-0319-front-middle-back-office-information-barrier.md
  - ADR-0320-apprentice-intern-resident-fellow-transient-identity.md
  - ADR-0321-b2b-saas-industry-leader-coverage.md
companion_docs:
  - docs/architecture/unified-ecosystem-thesis-2026-05-21.md
  - docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md
  - docs/architecture/training-cost-doctrine-2026-05-21.md
  - docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/personas/MASTER-ROSTER-2026-05-21.md
external_market_refs:
  - Gartner SaaS sprawl + management research (sizing references in §2)
  - Forrester tech-sprawl + SaaS-integration research (sizing references in §2)
  - SAP, Salesforce, Workday, ServiceNow, Atlassian, Microsoft, Stripe, Adobe, HubSpot, Zendesk, Snowflake, Databricks public product pages (competitive references in §11)
---

# Wave-3-G Executive Briefing — what oyatie is after the doc expansion

> **Audience.** Board directors, venture-capital partners, sales-leaders, marketing-leads, and GTM teams.
>
> **Tone.** Plain language, executive-readable, narrative-first. We cite the underlying ADRs and architecture docs but you do not need to read them to follow this briefing.
>
> **Reading time.** Under 90 minutes end-to-end. Each section is self-contained, so feel free to jump to §3, §9, §10, or §11 first if you are pressed.
>
> **Confidence level.** All quantitative claims below are either drawn from public Gartner / Forrester research (cited at the bottom of the briefing), from oyatie's own architecture corpus (cited inline), or labeled as internal sizing assumptions where customer-facing validation is still pending. The architecture team has flagged each sizing assumption explicitly so finance, legal, and procurement can validate before any number is used in customer-facing marketing.

---

## §1. The thesis in three sentences

1. **What is oyatie?** Oyatie is a single, unified software ecosystem that replaces the 110-plus separate SaaS applications a modern enterprise — and the increasingly software-saturated modern human — runs on today. One login, one policy engine, one workflow engine, one data model, one audit trail, one marketplace, one user-interface vocabulary; every "product" (CRM, HR, ERP, IT-service-management, marketplace, mail, calendar, notes, sheets, meet, community, workflow studio, audit) is a role-based projection of that single substrate.
2. **Why does it exist?** Because SaaS fragmentation has become an industrial-scale tax. Enterprises now spend a third of their IT budget on integration alone, train their workforce repeatedly on overlapping tools, and re-prove compliance posture vendor-by-vendor. Oyatie ends that cycle: train the user once, certify the platform once, audit the platform once, and carry that investment across personal, professional, regulated, and side-business contexts for a lifetime.
3. **What does it displace?** Functionally, the next decade of enterprise SaaS — SAP, Salesforce, Workday, ServiceNow, Atlassian, Microsoft, Adobe, HubSpot, Zendesk, Snowflake, Stripe-— and the next decade of personal-productivity-and-communication tools — Gmail, iMessage, WhatsApp, Slack, Notion, Calendly, Google Drive, Google Meet, Reddit, Instagram, TikTok, and the per-vertical apps stitched on top of them. Not by feature-matching them one-by-one, but by collapsing the fragmentation that makes them all expensive to own together.

That is the whole thesis. The rest of this briefing is the evidence, the scope, the doctrine, the personas, the journeys, the compliance posture, the roadmap, and the competitive landscape that supports it.

### §1.1 What changed in Wave-3-G

For a board director or investor who saw an earlier oyatie briefing (pre-2026-05-21), the most important update is this: Wave-3-G is the documentation expansion that closes the gap between "an ambitious unified-ecosystem architecture" and "a board-ready, sales-ready, investor-ready strategic story." Specifically:

- Wave-3-G produced the unified-ecosystem thesis manifesto (7,369 lines), the day-in-the-life narrative (7,611 lines), the training-cost doctrine (2,325 lines), the enterprise-software coverage matrix (9,248 lines), and the persona master roster (1,019 lines) — totaling roughly 27,900 lines of new strategic-narrative content.
- Wave-3-G crystallized the 30-ADR doctrine cluster (ADR-0297..0321) that establishes what the platform does at the operational edges (life-safety, abuse, recovery, jurisdiction, AI-agent delegation, M&A, vertical-pack coverage).
- Wave-3-G expanded the microservice corpus by 13 (the new ADR-0321 B2B leader anchors) and the vertical-pack corpus by 9 (insurance, automotive, oil-and-gas, pharma, legal-services, hospitality, agri-food, media, nonprofit).
- Wave-3-G is what enables this executive briefing to exist. The architecture is now coherent enough to summarize without losing fidelity.

### §1.2 Who this briefing is for

This briefing is structured for five audiences. Each audience can read selectively:

- **Board directors** — read §1, §2, §10, §11, §12. Optional: §17 (FAQ).
- **VC partners** — read §1, §2, §3, §9, §10, §17. Optional: §11 (competitive landscape).
- **Sales leaders** — read §3, §4, §5, §6, §9.6 (migration playbook), §11. Optional: §18 (vertical-pack deep-dive).
- **Marketing leads** — read §1, §2, §3, §5, §6, §10.6 (market timing), §12. Optional: §17 (objection handling).
- **GTM team** — read all sections; the briefing is structured to be the GTM enablement bible for the next 12 months.

---

## §2. The fragmentation tax — the executive problem statement

### §2.1 The shape of the problem

Every executive who reads this briefing already knows the symptoms. We restate them here as a single coherent stack of costs, because the unified-ecosystem case rests on the proposition that fragmentation is one problem with many faces — not many independent problems.

- **The SaaS portfolio is large and growing.** Public sprawl research (Gartner SaaS sprawl + management research, Forrester tech-sprawl research) consistently puts the average mid-market-to-enterprise SaaS portfolio at roughly 110 separate applications, with the largest enterprises running 200-400. Each application is a separate procurement, a separate vendor-management relationship, a separate IT-security review, a separate compliance posture, and a separate training cohort.
- **Integration is the single largest hidden line item.** Independent analyst research consistently puts integration spend at roughly 30 percent of enterprise IT budget — the iPaaS contracts, the bespoke ETL jobs, the Zapier seats, the consulting hours that bridge tools the vendors themselves never meaningfully bridged. This is the cost of one tool not talking to the next.
- **Training is paid per-tool, per-employee, per-year.** Internal sizing assumptions (pending customer-validated source confirmation) put per-employee per-tool training pressure at roughly $1,500 per year. Across a 110-tool portfolio, that is approximately $165,000 of training pressure per employee per year — most of it absorbed as "lost time" rather than line-item spend, but real.
- **Onboarding velocity is throttled.** Major tool deployments — a CRM swap, an HR-system swap, an ERP go-live — routinely take 6-9 months of organization-wide retraining and process realignment before the workforce reaches steady-state productivity again.
- **Compliance posture multiplies.** Every tool is its own SOC 2 review, its own GDPR DPA, its own HIPAA BAA, its own data-residency contract, its own SSO integration, its own DSAR endpoint, and its own breach-notification clock. The cost is not the tool's compliance certificate. The cost is the cumulative legal, security, and audit time spent reconciling those certificates against the enterprise's own policy.
- **Exit costs are punishing.** Every tool's data model is a private snowflake. Migrating off any one of them — even a tool the enterprise no longer wants — means rebuilding the integrations, retraining the workforce, re-certifying the compliance posture, and re-onboarding every downstream tool that depended on the one being removed. The exit cost compounds with portfolio size.

### §2.2 The dollar shape of the problem

Translating the fragmentation tax into rough dollar terms (these are sizing assumptions for executive framing; customer-facing collateral will quote analyst-validated figures only):

- **Per-employee per-year SaaS spend:** $5,000-$25,000 (white-collar / knowledge worker baseline). Heavily-licensed roles (sales engineers, financial analysts, developers) routinely exceed $50,000 per year in directly-licensed SaaS bills alone.
- **Per-employee per-year integration spend:** $1,500-$7,500. iPaaS seats, integration consulting, custom-code maintenance, integration-broken-things firefighting.
- **Per-employee per-year training pressure:** $1,000-$5,000. Heavily weighted toward tool churn (when a tool is swapped) and role transitions (when a person changes jobs).
- **Aggregate per-employee per-year fragmentation cost:** $50,000-$500,000 depending on role complexity. This includes lost productivity from context-switching across 100+ tools, the time spent reconciling data that lives in multiple systems, and the slow-down imposed by per-tool authentication and per-tool approval workflows.
- **Enterprise-aggregate fragmentation cost:** at a 5,000-person enterprise, the annual fragmentation cost is in the high-nine-figure to low-ten-figure range. At a 50,000-person multinational, the cost is in the low-eleven-figure range.

These are not numbers the CFO sees on the SaaS invoice. They are the numbers that show up as IT-budget overruns, as integration-debt write-downs, as audit-finding remediation, as the post-acquisition-integration line item in M&A diligence, and as the silent productivity drag that makes large enterprises feel slower than small ones.

### §2.3 Why fragmentation got this bad

The cynical answer — "every SaaS vendor wants you locked in" — is true but incomplete. The structural answer is:

- **Each vendor optimizes for its own seat-count growth, not for the buyer's portfolio cost.** Every SaaS vendor's product strategy concludes with "and now do more inside our tool" — never "and now do less in everyone's tool." The economic incentives at vendor-level make fragmentation the rational equilibrium.
- **Integration platforms exist because vendors will never integrate.** iPaaS (MuleSoft, Boomi, Workato, Zapier) is a billion-dollar market that exists entirely because vendors refuse to converge on shared identity, shared workflow, shared object models, and shared audit trails.
- **Procurement, security review, and compliance review are per-vendor, not per-portfolio.** Enterprise buying processes evaluate one vendor at a time, with no mechanism for measuring portfolio coherence.
- **Each new tool's value is visible; the cumulative drag is invisible.** A specific new tool always solves a specific local pain. The cost it adds to the portfolio is diffuse and shows up months or years later.

The fragmentation tax compounded over 25 years of SaaS adoption. It is now large enough — and visible enough — that the market is ready for an alternative.

### §2.4 What the fragmentation tax really costs

Beyond dollars, the fragmentation tax extracts four costs that are harder to put a number on but easier to recognize:

- **Cognitive overhead.** Knowledge workers context-switch between 9-12 tools per hour. Studies of context-switching put the productivity cost at 20-40 percent. This is the cost of "having to remember which tool does what."
- **Trust erosion.** Every tool is a separate attack surface, a separate breach-notification commitment, a separate insider-risk vector. The enterprise's overall security posture is the worst posture among the tools it runs.
- **Decision latency.** Cross-functional decisions (a quote-to-cash cycle, a hire-to-onboard cycle, an incident-to-remediation cycle) move at the speed of the slowest hand-off between the slowest tools. Adding tools never speeds the chain; it slows it.
- **Employee experience drag.** New hires spend their first weeks creating tool accounts. Departing employees leave behind orphaned accounts. Role transitions trigger permission re-engineering across dozens of tools. The HR director's job is increasingly the IT director's job.

This is the tax oyatie exists to dissolve.

### §2.5 Real-world fragmentation in one mid-market enterprise

To make the fragmentation tax concrete, picture a representative 5,000-employee mid-market enterprise — a SaaS-native company with global operations. Their tool portfolio breaks down like this:

- **Identity + access management:** Okta + Azure AD + Auth0 + a half-dozen tool-specific identity stores. 8 tools.
- **Communication:** Slack + Zoom + Microsoft Teams + Gmail (personal) + Outlook (corporate) + Loom + Otter.ai. 7 tools.
- **Collaboration:** Notion + Confluence + Google Drive + OneDrive + Dropbox + Box + Figma + Miro + Lucidchart. 9 tools.
- **CRM + marketing + customer-success:** Salesforce + HubSpot + Marketo + Pardot + Mailchimp + Outreach + Salesloft + Drift + Intercom + Zendesk + Front + Gainsight. 12 tools.
- **HR + talent:** Workday + Greenhouse + Lever + BambooHR + 15Five + Lattice + Culture Amp + Carta + Deel + Rippling. 10 tools.
- **Finance + ERP:** NetSuite + QuickBooks (subsidiaries) + Stripe + Bill.com + Expensify + Brex + Pigment + Anaplan + Avalara + Tipalti. 10 tools.
- **Engineering + DevOps:** GitHub + GitLab (legacy) + Jira + Linear + CircleCI + GitHub Actions + Datadog + Sentry + PagerDuty + Statuspage + LaunchDarkly + Snyk + Tailscale. 13 tools.
- **Data + analytics:** Snowflake + Databricks + dbt + Looker + Tableau + Mode + Hex + Segment + Mixpanel + Amplitude + Heap. 11 tools.
- **ITSM + IT-ops:** ServiceNow + Jamf + Kandji + Okta Workflows + custom internal portals. 6 tools.
- **Security + compliance:** Vanta + Drata + Secureframe + Tugboat Logic + 1Password + Snyk + Lacework + a half-dozen point security tools. 10 tools.
- **Procurement + legal:** Coupa + Ironclad + DocuSign + Concord + Spendesk + Ramp. 6 tools.
- **Marketing-ops + analytics:** HubSpot + Marketo + Mixpanel + Iterable + Customer.io + Braze + a half-dozen attribution + ad-tech tools. 10 tools.

That is **112 tools**. Each tool is a separate vendor relationship, a separate SOC 2 review, a separate SSO connector, a separate DPA, a separate user-provisioning flow, a separate billing relationship, a separate procurement cycle. The CIO of this enterprise has a vendor-management team of 8 people whose entire job is keeping this portfolio alive.

What does oyatie replace? Roughly 80-95 of these tools fold into the unified substrate as capability tiers or hero-product surfaces. The remaining 5-15 (specialist tools — Figma's creative depth, GitHub's developer-network, specific industry tools, audit certifications that hold incumbent advantage) compose with oyatie through the plugin-app-store substrate.

### §2.6 The hidden third tax — the AI tax

A new fragmentation-tax line item is rising fast: the AI tax. Each SaaS vendor is racing to ship its own AI assistant — Salesforce Einstein, ServiceNow Now Assist, Microsoft Copilot, HubSpot Breeze, Workday AI, Adobe Sensei, GitHub Copilot. The economic logic is sound at the vendor level (AI-driven seat upsell). The economic logic is broken at the portfolio level:

- Each tool's AI is bounded by the data that tool sees. The CRM AI does not see calendar data; the calendar AI does not see CRM data; the project-management AI does not see either.
- Each AI subscription is its own line item. A heavily-AI-licensed enterprise can pay $1,000-$3,000 per user per year in AI add-ons alone, spread across the portfolio.
- Cross-tool AI features (the ones that would actually move productivity) require iPaaS-mediated data integration, which the AI vendors do not provide.

The result: the enterprise pays for 12 separate AIs that together do less than one well-built unified AI would do.

Oyatie's intelligence substrate (the AI Substrate + Consumer Brand Surface layer per ADR-0255) sees the whole platform's data. A single AI assistant has access to messenger context, mail context, calendar context, CRM context, document context, workflow context, marketplace context, and audit context simultaneously. The productivity unlock is structural, not incremental.

---

## §3. The unified-ecosystem answer

### §3.1 One platform, one of everything that matters

The unified-ecosystem thesis is captured in ten "one" clauses (companion doc: `docs/architecture/unified-ecosystem-thesis-2026-05-21.md`):

- **ONE-IDENTITY.** One passkey-bound human identity with N tenant memberships, not one account per tool. Yejin-the-nurse, Yejin-the-parent, and Yejin-the-side-business-owner are the same identity in different contexts — not three separate accounts on three separate platforms.
- **ONE-POLICY-ENGINE.** One Cedar-based authorization engine for every "may this happen" decision. Every UI click, API call, workflow transition, marketplace settlement, and audit-evidence access goes through the same policy primitive. No more permission-set debugging across 100+ tools.
- **ONE-WORKFLOW-ENGINE.** One state-machine-and-DAG substrate for every durable process. Approving a purchase order uses the same workflow primitives as approving a paid time off request, the same primitives as approving a code change, the same primitives as approving a clinical-trial protocol. The user learns "approve / reject / route / defer / escalate / attach evidence" once.
- **ONE-ONTOLOGY.** One object graph with role, capability, and jurisdiction projections. A Customer is a Customer is a Customer — whether the role projection labels them "lead" (sales), "patient" (healthcare), "borrower" (finance), or "tenant" (real estate). The underlying identity, history, and audit trail are unified.
- **ONE-AUDIT-CHAIN.** One evidence chain for identity, policy, workflow, settlement, and operations. Auditors and regulators query one timeline, not 100+ per-tool exports.
- **ONE-MARKETPLACE.** One deal-settlement substrate (ADR-0314) for every commercial exchange — consumer purchases, B2B procurement, plugin entitlements, workforce contracts, M&A transitions, joint ventures, data licenses, and receivables assignments. Stripe Connect, SAP Ariba, Coupa, and Salesforce Commerce Cloud's settlement primitives collapse into one DealSet object.
- **ONE-UX-SHELL.** One stable interaction vocabulary across roles, devices, collar colors, and locales. The forklift driver's handheld scanner and the CFO's desktop multi-monitor setup share the same gestures, the same approval grammar, and the same context indicators — adjusted for ergonomics, not reinvented.
- **ONE-TRAINING-MODEL.** One vocabulary that the user learns once and carries across departments and career stages. The nurse who becomes a unit-manager learns no new approval grammar. The intern who becomes a senior engineer learns no new escalation grammar. The retiree who returns as a board director learns no new evidence-review grammar.
- **ONE-COMPLIANCE-POSTURE.** One pack-and-evidence model applied before data or workflow exposure. HIPAA, GDPR, SOC 2, FedRAMP, KR-PIPA, EU-AI-Act — each is a tenant-scoped compliance pack overlay, not a separate compliance ecosystem.
- **ONE-PLUGIN-EXTENSIBILITY.** One governed extension model with isolation, admission, settlement, and auditability. Third-party developers ship to one marketplace and one runtime — not to Salesforce AppExchange, ServiceNow Store, Atlassian Marketplace, Slack App Directory, Stripe App Marketplace, and another dozen places.

### §3.2 Why "one" beats "best-of-breed"

The "best-of-breed" defense of fragmentation says: every tool is the best at its narrow job, and the fragmentation tax is the price of that excellence. This was true in 2005. In 2026 it is not, for three reasons:

1. **The substrate is the moat.** Identity, policy, workflow, data model, audit trail, marketplace settlement, and UX vocabulary are not where any one tool's value lives anymore. They are commodity infrastructure that every tool reimplements. Best-of-breed today means "we built our own commodity substrate badly."
2. **The "best" tools are increasingly themselves bundled ecosystems.** Salesforce is not one tool — it is Sales Cloud + Service Cloud + Marketing Cloud + Commerce Cloud + Tableau + Slack + MuleSoft + Heroku. Microsoft 365 is not one tool — it is Outlook + Teams + SharePoint + OneDrive + Power Platform + Copilot. The "best-of-breed" claim has already been quietly conceded by the largest vendors; they have been re-bundling for a decade.
3. **AI changes the calculus.** The value of an AI assistant inside a single tool is bounded by the data that tool sees. The value of an AI assistant inside a unified ecosystem is bounded by the data the whole platform sees — which is roughly an order of magnitude larger. The unified platform makes the AI fundamentally more useful, which makes the unified platform fundamentally more valuable.

### §3.3 The per-department experience without the per-tool tax

A common executive concern: "Will my sales team accept a CRM that is part of a unified ecosystem? Will my engineers accept a code-review surface that is part of a unified ecosystem? Won't they miss the specialist tool?"

The answer is the **capability-tier projection** doctrine (ADR-0316). A capability tier is a tenant-activated bundle of Cedar permits + ontology projections + workflow templates + UX shell vocabulary + compliance overlays + observability metadata that surfaces as a familiar product label.

- The sales team sees a sales-CRM surface — opportunity pipelines, lead routing, forecast roll-ups, quote-to-cash automation. The label is "CRM"; the substrate is shared.
- The HR team sees an HR surface — performance reviews, comp planning, learning paths, candidate funnels. The label is "HRIS"; the substrate is shared.
- The finance team sees an ERP-finance surface — general ledger, accounts payable, accounts receivable, treasury operations, FP&A planning. The label is "ERP"; the substrate is shared.
- The IT-ops team sees an ITSM surface — incident management, change management, asset tracking, configuration management. The label is "ITSM"; the substrate is shared.

Each department gets the labels and the affordances they expect. None of them carries the per-tool integration tax, the per-tool training tax, the per-tool compliance tax, or the per-tool exit tax.

### §3.4 The operating model — what is shared, what is composed

The unified-ecosystem model rests on a clean separation between **substrate** (the shared layer) and **product surface** (the composed layer). To picture this concretely:

- The **substrate** layer is roughly 21 microservices providing identity, tenancy, cell-deployment, governance, consent-graph, audit-chain, compliance pack overlay, IaC, K8s orchestration, secrets management, observability, API gateway, application shell, foundry pipeline, ontology object graph, workflow-engine state machine, intelligence (AI), network mesh, detection signals, feature-flag system, and translation runtime. Every product runs on top of these. None of these are themselves products. Nothing breaks if the substrate stays stable while products evolve.
- The **hero-product surface** layer is roughly 18 microservices providing the day-one consumer-and-productivity surfaces (messenger, mail, community, marketplace, workflow-studio, drive, calendar, meet, recordings, notes, shorts, social, plugin-app-store, sheets, slides, sites, forms, tasks). Each surface is a separately deployable, separately versioned, separately ownable microservice — but every one of them composes against the same substrate.
- The **vertical-and-enterprise surface** layer is roughly 30 microservices for ERP (finance, treasury, warehouse, global-trade, real-estate, supply-chain-planning, production-planning, quality-management, plant-maintenance) + B2B leader coverage (marketing-automation, contact-center, performance-management, learning-management, ITSM, incident-management, financial-planning, data-warehouse, contract-lifecycle-management, data-pipeline, healthcare-integration, connect) + cross-cutting services (CRM, analytics, comms-email, developer-SDK, ops-dashboard, workplace-integration).

The substrate-vs-product layering (ADR-0245) is the load-bearing architectural commitment that makes everything else work. Without it, oyatie is just another SaaS vendor adding services. With it, every new service is incrementally cheap because the hard problems (identity, policy, workflow, audit, settlement, UX, training, compliance) are already solved once.

### §3.5 Training cost amortized across a career

Companion doc: `docs/architecture/training-cost-doctrine-2026-05-21.md`.

The training-cost case is the simplest and most under-appreciated of oyatie's strategic advantages. In a fragmented portfolio, every job change, every tool swap, and every team rotation triggers retraining. In oyatie, the user learns one vocabulary — approve, assign, comment, sign, attach evidence, route, defer, escalate, switch role, verify context, review history, export with policy, recover from denial — and carries it across:

- Personal life (messaging family, ordering groceries, paying bills, managing kids' school).
- Education (high school assignments, university coursework, professional certifications, apprenticeships).
- Frontline work (warehouse shifts, retail clerking, delivery routes, field service calls, restaurant production).
- Office work (sales pipeline, HR cycle, finance close, engineering review, customer success).
- Regulated work (hospital shift, audit fieldwork, regulatory inspection, clinical trial).
- Side businesses (independent consulting, e-commerce, gig work, farmer's market, freelance creative).
- Family ops (eldercare, childcare coordination, estate management, multi-generational decisions).
- Retirement (managing health appointments, financial accounts, volunteer board duties, grandparent communication).

The vocabulary that the 17-year-old retail clerk learns at their first part-time job is the vocabulary the 50-year-old CFO uses to approve a $200M acquisition is the vocabulary the 75-year-old retiree uses to manage their healthcare appointments. The training investment compounds across the user's life, instead of being thrown away with every tool swap and every job change.

For the enterprise buyer, this dissolves the training-tax line of the fragmentation cost. New hires arrive pre-fluent. Internal mobility costs less. Role transitions are faster. M&A integration is faster. Workforce productivity per dollar of training spend goes up.

For the employee, this means their software skills become a durable career asset instead of an expiring per-tool credential.

---

## §4. What oyatie covers — the scope

### §4.1 The hero consumer-and-productivity surfaces (day-one shipping intent)

Oyatie ships, day one of GA, a hero set of consumer-and-productivity surfaces designed to be every modern human's default ecosystem. From companion docs and the microservices corpus:

1. **Messenger** — end-to-end-encrypted messaging (RFC 9420 MLS protocol; ADR-0246 substrate). Replaces iMessage, WhatsApp, Signal, Telegram, and the workplace overlap with Slack DMs.
2. **Mail** — secure email with full MIME and SMTP/IMAP/JMAP federation. Replaces Gmail, Outlook, Yahoo Mail, ProtonMail.
3. **Community** — forums, threads, voting, moderation, locale-aware mod policy. Replaces Reddit, Discord communities, Facebook Groups, Discourse.
4. **Marketplace** — universal deal-settlement substrate (ADR-0314) for consumer purchases, B2B procurement, plugin entitlements, workforce contracts, data licenses, M&A transitions. Composes everything Stripe Connect, Shopify, SAP Ariba, Coupa, and Salesforce Commerce Cloud do.
5. **Workflow Studio** — n8n-class visual workflow editor for personal automations, team automations, enterprise process automation, and agentic flows. Replaces Zapier, Make, n8n, IFTTT, Power Automate, and Workday's process designers.
6. **Drive** — object storage with policy-gated sharing, versioning, and content-aware classification. Replaces Google Drive, OneDrive, Dropbox, Box.
7. **Calendar** — scheduling with cross-tenant availability, booking pages, agenda projections, and role-aware time blocks. Replaces Google Calendar, Outlook Calendar, Calendly, Cal.com.
8. **Meet** — voice + video + screen-share with E2EE call mode, recording, transcription, and breakout rooms. Replaces Zoom, Google Meet, Teams, Webex.
9. **Recordings** — meeting recordings, asynchronous video, podcast hosting, screen recordings, lecture archives. Replaces Loom, Vimeo enterprise, podcast hosts, lecture-capture systems.
10. **Notes** — text editor with linked references, multiplayer editing, AI-assist, and tenant-scoped sharing. Replaces Notion, Obsidian, OneNote, Evernote, Bear, Roam.
11. **Shorts** + **Social** — vertical-video and social-graph surfaces, with creator-economy settlement and minor-protection guardrails (ADR-0292). Replaces TikTok, Instagram Reels, YouTube Shorts.
12. **Plugin App Store** — the third-party developer ecosystem; one signed runtime, one marketplace-mediated settlement, one Cedar-gated isolation model. Replaces App Stores generally — and absorbs Salesforce AppExchange, ServiceNow Store, Atlassian Marketplace, and Slack App Directory's strategic role.

Day-one productivity adjuncts:

- **Sheets** (collaborative spreadsheets), **Slides** (presentations), **Sites** (intranet-and-public site builder), **Forms** (data collection + validation), **Tasks** (lightweight task tracking), **Translate** (translation + localization runtime).

Total day-one hero surface count: **18 consumer-and-productivity surfaces**.

### §4.2 Full SAP S/4HANA module parity

The SAP S/4HANA enterprise-resource-planning suite is the global standard for finance + supply-chain + manufacturing operations. ADR-0315 commits oyatie to full SAP S/4HANA module parity. From the keystone bundle and ADR-0315:

- **FI (Financial Accounting):** General ledger, accounts payable, accounts receivable, fixed assets, bank accounting.
- **CO (Controlling):** Cost-center accounting, profit-center accounting, internal orders, profitability analysis.
- **MM (Materials Management):** Purchasing, inventory, vendor management, invoice verification.
- **SD (Sales & Distribution):** Sales orders, deliveries, billing, pricing, credit management.
- **PP (Production Planning):** Demand planning, MRP, manufacturing execution, capacity planning.
- **QM (Quality Management):** Quality inspection, certificates, complaints, audit support.
- **PM (Plant Maintenance):** Equipment, maintenance orders, preventive maintenance, work clearance.
- **EWM (Extended Warehouse Management):** Warehouse layout, picking strategies, slotting, labor management.
- **TM (Transportation Management):** Freight planning, carrier selection, shipping execution, settlement.
- **GTS (Global Trade Services):** Export control, customs, sanctioned-party screening, trade preference.
- **HCM / SuccessFactors:** Core HR, talent acquisition, performance management, learning, comp.
- **CRM:** Sales cloud, service cloud, marketing cloud equivalence.
- **SRM (Supplier Relationship Management):** Sourcing, contract negotiation, supplier portal, Ariba-class procurement network.
- **TRM (Treasury & Risk Management):** Cash management, in-house bank, debt/investment, hedge accounting.
- **PS (Project System):** Project structures, milestones, billing, cost allocation.
- **RE (Real Estate):** Lease accounting, property management, facility allocations.
- **EHS (Environment, Health, Safety):** Incident management, safety inspections, dangerous-goods, sustainability reporting.

Plus the SAP industry verticals — utilities (IS-U), oil-and-gas (IS-OIL), automotive (IS-Auto), insurance (FS-Insurance), banking (FS-Banking), public sector (IS-PS), retail (IS-Retail), healthcare (IS-H).

**Total SAP modules covered: 28.** ADR-0315 establishes 9 new microservice anchors specifically for SAP-parity domains (treasury, warehouse, global-trade, real-estate, supply-chain-planning, production-planning, quality-management, plant-maintenance, plus the existing finance/CRM/HR composition pattern).

### §4.3 Full B2B SaaS industry-leader coverage beyond SAP

ADR-0321 extends the coverage doctrine to the rest of the B2B SaaS landscape. Vendor-by-vendor:

- **Salesforce** (Sales Cloud, Service Cloud, Marketing Cloud, Commerce Cloud, Experience Cloud, Tableau, MuleSoft, Slack) — covered via existing CRM + marketing-automation + contact-center + analytics + plugin-app-store composition.
- **ServiceNow** (Now Platform, ITSM, ITOM, ITAM, HRSD, CSM, SecOps, GRC) — covered via new ITSM + incident-management microservices composed with workflow-engine, identity, ontology.
- **Workday** (HCM, Financial Management, Adaptive Planning, Talent, Learning) — covered via existing identity + new performance-management + new learning-management + new financial-planning microservices, with HCM composed from identity + workflow.
- **Atlassian** (Jira, Confluence, Bitbucket, Trello, Jira Service Management) — covered via tasks + notes + developer-sdk + ITSM composition.
- **Microsoft 365** (Outlook, Teams, SharePoint, OneDrive, Office, Power Platform, Copilot) — covered by mail + messenger + meet + drive + sheets/slides/notes + workflow-studio composition.
- **Adobe** (Creative Cloud, Document Cloud, Experience Cloud, Marketo, Workfront) — covered via new design-collaboration + drive + workflow-studio + marketing-automation composition.
- **HubSpot** (Marketing Hub, Sales Hub, Service Hub, CMS Hub, Operations Hub) — covered via crm + marketing-automation + contact-center + sites composition.
- **Zendesk** (Support, Sell, Sunshine, Talk) — covered via new contact-center + crm composition.
- **Snowflake** (Data Cloud) — covered via new data-warehouse + analytics composition.
- **Databricks** (Data Intelligence Platform) — covered via new data-pipeline + data-warehouse + intelligence composition.
- **Stripe Connect** (platform payments, multi-party settlement) — covered via existing payments + marketplace DealSet substrate.
- **DocuSign / Ironclad** (CLM) — covered via new contract-lifecycle-management microservice.
- **Miro / FigJam** (whiteboarding) — covered via new whiteboard microservice.
- **Figma** (design collaboration) — covered via new design-collaboration microservice.

ADR-0321 enumerates **165 vendor benchmark dossiers** and authorizes **13 new microservice anchors** to cover the gaps. These are the operationally-distinct concerns that could not be absorbed into existing services as capability tiers.

### §4.4 Vertical compliance + locale packs

Oyatie's vertical-pack model (ADR-0251 + ADR-0316 capability-tier overlay) treats compliance + locale + industry as overlays over the unified substrate. Currently in scope:

**Wave-3-G new vertical packs (9):**

1. **Insurance** — policy lifecycle, underwriting, claims, reinsurance, regulatory reporting.
2. **Automotive** — dealer-network management, vehicle inventory, service-and-parts, recall management, OEM-supplier coordination.
3. **Oil & gas** — upstream production, joint-venture accounting, well-and-asset tracking, HSE compliance.
4. **Pharma** — clinical-trial management, GxP-compliant document control, regulatory submission, pharmacovigilance.
5. **Legal services** — matter management, conflicts-checking, billable-time tracking, e-discovery, retention.
6. **Hospitality** — property management, reservations, point-of-sale, housekeeping, distribution channels.
7. **Agri-food** — farm-to-fork traceability, cooperative settlement, organic certification, weather-and-yield analytics.
8. **Media** — rights management, royalty settlement, content licensing, advertising operations.
9. **Nonprofit** — donor management, grants, fund accounting, volunteer coordination, impact reporting.

**Existing vertical packs (15):** Healthcare (HIPAA + provider workflow), banking (Basel, KYC, AML), retail (POS, inventory, omnichannel), public sector (FedRAMP, gov procurement), education (FERPA, LMS), telecom (subscriber, billing), utilities (meter-to-cash, outage), manufacturing (MES, OEE), construction (project, safety), real estate (property, lease), professional services (utilization, time-and-billing), creative agency (project, brief, asset), trucking-and-logistics (TMS, dispatch), gig-economy (worker classification, settlement), fintech (KYC/AML, settlement, BaaS).

**Total vertical packs: 24** (15 existing + 9 new in Wave-3-G).

### §4.5 The microservice corpus

The architecture corpus currently lists **69 microservices** (post-ADR-0321; growing toward 71 with optional logistics-integration + personal-health-tracker follow-ups). Breakdown:

- **Substrate (commodity infrastructure):** identity, tenancy, cell, governance, consent-graph, audit-chain, compliance, cloud-iac, cloud-k8s, cloud-secrets, observability, api-gateway, application, foundry, ontology, workflow-engine, intelligence, network, detection, feature-flags, translate. (21 microservices.)
- **Hero consumer + productivity surfaces:** messenger, mail, community, marketplace, workflow-studio, drive, calendar, meet, recordings, notes, shorts, social, plugin-app-store, sheets, slides, sites, forms, tasks. (18 microservices.)
- **Office collaboration:** docs, whiteboard, design-collaboration. (3 microservices.)
- **ERP coverage (SAP parity):** finops-portal, payments, treasury, warehouse, global-trade, real-estate, supply-chain-planning, production-planning, quality-management, plant-maintenance. (10 microservices.)
- **B2B SaaS leader coverage (ADR-0321):** marketing-automation, contact-center, performance-management, learning-management, itsm, incident-management, financial-planning, data-warehouse, contract-lifecycle-management, data-pipeline, healthcare-integration, connect. (12 microservices.)
- **Cross-cutting + ops:** crm, analytics, comms-email, developer-sdk, ops-dashboard-control-center, workplace-integration. (6 microservices.)

Each microservice is independently deployable, flat-laid-out (one source-root, no nested suites per ADR-0131), single-concern (no "bundle" services per ADR-0132), and uniformly governed.

### §4.6 What does NOT belong in the scope

To be precise about the scope, here is what oyatie does not aim to do:

- **Not a hyperscaler.** Oyatie runs on top of AWS, Azure, GCP, KR-NCP, JP-NTT-Cloud, and other regional cloud substrates. It is not a competitor to AWS-EC2 or Azure-VM. ADR-0254 (Kubernetes + Cloud Hypervisor + Kata) is the deployment shape; the hyperscaler is the underlying infrastructure provider.
- **Not a specialist creative tool.** Adobe Photoshop, Adobe Illustrator, Adobe Premiere, Logic Pro, Final Cut, and category-leader creative tools remain best-of-breed. Oyatie composes with them via the plugin-app-store substrate; oyatie does not replace them.
- **Not a specialist developer tool.** The developer-sdk microservice provides oyatie-platform-extension primitives, but oyatie does not replace VS Code, JetBrains IDEs, language compilers, or the GitHub developer-network. Developer tools sit on top of oyatie's identity + governance + marketplace primitives, but they remain separate ecosystems.
- **Not a specialist scientific tool.** MATLAB, R, SAS, SPSS, Jupyter-as-an-IDE, and similar specialist analytic environments remain best-of-breed. The data-warehouse + data-pipeline + analytics microservices in oyatie cover BI + general analytics; they do not aim to displace deep-scientific environments.
- **Not a specialist game engine.** Unreal, Unity, Godot, and game-specific runtimes remain best-of-breed. Oyatie's plugin-app-store may host games as plugins, but oyatie is not a game-engine vendor.
- **Not a hardware vendor.** Oyatie is a software platform. Apple, Samsung, Microsoft, Lenovo, and others remain the hardware providers. The handheld-rugged + vehicle-mount + assistive device profiles describe how oyatie targets hardware — not hardware oyatie ships.

The scope discipline is itself a strategic choice. A unified-ecosystem play that tries to be everything loses the substrate coherence that makes the unified-ecosystem thesis valuable in the first place.

---

## §5. The personas it serves

### §5.1 Continuity-of-identity doctrine

The personas case is the strongest reason a board director or VC partner should believe the unified-ecosystem thesis is durable.

The doctrine, from `docs/personas/MASTER-ROSTER-2026-05-21.md`:

- **One human, many contexts.** A single biological human is one identity. The same human can be a consumer, an employee, a healthcare patient, a side-business owner, a parent, an apprentice, an auditor, a nurse, a manager, a warehouse worker, a surgeon, and a family member — at different points of their day, their week, their career, or their life.
- **One passkey, multiple tenant memberships.** Per ADR-0311 (dual-tenant personal-vs-work boundary) and ADR-0313 (conglomerate tenant hierarchy), the same passkey-bound identity holds memberships in many tenants: personal tenant + employer tenant + healthcare provider tenant + side-business tenant + government-services tenant + extended-family conglomerate tenant.
- **Persona = identity × tenant × role × workspace × locale × device × skill-tier.** A persona is a coordinate in a multi-dimensional context space. The same human surfaces as different personas in different coordinates. Yejin-the-nurse and Yejin-the-parent are the same human in different coordinates — not two separate users.
- **Cross-context bridges are first-class.** Every persona declares which other personas are the same human. Apple's "personal Apple ID + business Apple ID" pattern, Microsoft's "personal Microsoft account + work-or-school account" pattern, Google's "personal Google account + Google Workspace account" pattern — oyatie unifies them under one passkey-bound identity with Cedar-enforced per-tenant scoping, not just UX hints.

### §5.2 The anchor personas (the executive shorthand)

Six anchor personas carry the day-in-the-life narratives (companion doc: `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`):

1. **Yejin Park** — A Korean ICU nurse (pink-collar, clinical workspace, mid-level skill, KR locale, mobile-primary device). She is also a parent (B2C family-parent) and a side-business owner (small home-baking business). Her morning is shift-work in the hospital; her afternoon is school pickup; her evening is invoicing customers. All under one passkey, three tenants, three role projections, one trained vocabulary.
2. **Marcus Chen** — A 5,000-person-multinational CEO (white-collar + gold-collar, executive workspace, executive skill, KR+US locale, desktop-primary device). He is also a spouse and a father. His day is board prep, M&A diligence, P&L review, school-board volunteer work, and family time — across three tenants.
3. **Aiyana Singh** — A senior ML engineer (white-collar, back-office, senior skill, IN locale, desktop-primary). She is also a tech-blogger (side-business tenant) and a parent. Her day is engineering work, weekly blog publication, school logistics.
4. **Tomás García** — A Brazilian restaurant owner (white-collar + green-collar, executive + production workspace, senior skill, BR locale, mobile-primary). He is the tenant-admin of his restaurant tenant, an employee of his own tenant (he cooks), a consumer (he buys his groceries), and a father.
5. **Hiroshi Tanaka** — A retired Japanese widower (white-collar retired, field workspace, senior-retired skill, JP locale, mobile + assistive devices). His day is grandparent communication, village photography, doctor's appointments.
6. **Anya Mironova** — An EU investigative journalist + activist + parent. Field-based, multi-device, multi-locale. ADR-0300 (whistleblower + press freedom) protects her source-protection workflow under court-warrant pressure.

### §5.3 The enterprise tenant cast

Anchor tenants that recur across journeys:

- **KrampusCorp** — A 50,000-employee multinational that uses oyatie as its corporate substrate. Marcus is the CEO. Priya is the HR director. Sam is the internal-audit director. Aiyana is a senior IC. Used to demonstrate large-enterprise ERP-parity, governance, and compliance posture.
- **AcmeRawMaterials** — A mining + commodities tenant in the KrampusCorp supply chain. Used to demonstrate inter-tenant supply-chain coordination.
- **GlobalLogistics** — A logistics carrier in the KrampusCorp shipping flow. Used to demonstrate cross-tenant operational handoffs.
- **Yejin's Hospital** — A regional hospital tenant. Used to demonstrate HIPAA-equivalent compliance posture under KR-Privacy + KR-Medical-Records-Law overlays.
- **Tomás's Restaurant** — A small-business tenant. Used to demonstrate that the same substrate that runs KrampusCorp also runs a 12-employee restaurant.
- **Anya's Newsroom** — A pan-EU investigative journalism non-profit. Used to demonstrate ADR-0300 source-protection.
- **Diana's GAO Office** — A US federal-government auditor tenant. Used to demonstrate INTERNAL_AUDITOR_3PAO and GOV_INSPECTOR audience types.

### §5.4 The persona graph at full breadth

The full persona roster enumerates **127 personas** spanning the orthogonal axes of:

- **Collar-color (6):** white-collar, blue-collar, pink-collar, gold-collar, gray-collar, green-collar. (ADR-0318 codifies this as the universal workforce-segmentation primitive.)
- **Workspace (7):** front-office, middle-office, back-office, field, clinical, executive, production. (ADR-0319 codifies the information-barrier doctrine across these workspaces.)
- **Skill-tier (6):** in-training (apprentice/intern/resident/fellow per ADR-0320), junior, mid-level, senior/staff, principal/distinguished, executive.
- **Locale (6+ anchored, 20+ supported):** KR, US, EU, JP, IN, BR — anchored. Plus CN, UK, FR, DE, ES, NL, SE, SG, NG, GH, IL, AE, MX, AR, CL, ZA, EG, TR, PL, RO, IE — supported via pack overlays.
- **Device profile (6):** mobile-primary, desktop-primary, handheld-rugged, kiosk/shared, assistive, vehicle-mount.
- **Audience-type (32+ values per ADR-0244):** B2C_CONSUMER, B2C_FAMILY_PARENT, B2C_JOB_SEEKER_ACTIVE, B2C_MINOR_UNDER_13 (COPPA-blocked), B2C_MINOR_14_17 (KOSA-tiered), B2B_EMPLOYEE, B2B_TENANT_ADMIN, B2B_HR_ADMIN, B2B_INTERNAL_AUDIT, B2B_CSUITE, B2B_BOARD_DIRECTOR, B2B_CONTRACTOR, B2B_APPRENTICE_INTERN, B2B_MEDICAL_RESIDENT, B2B_FIELD_WORKER, B2B_KIOSK_USER, B2B_BANK_INTERNAL, B2B_HEALTHCARE_PROVIDER, B2B_HEALTHCARE_PATIENT, B2B_REGULATOR_EXTERNAL, B2B_EXTERNAL_AUDITOR, B2B_EXTERNAL_COUNSEL, B2B_INVESTOR_LP, B2B_CHANNEL_PARTNER, INTERNAL_AUDITOR_3PAO, GOV_INSPECTOR, EDU_TEACHER, EDU_STUDENT, EDU_PARENT, RELIGIOUS_LEADER, LAW_ENFORCEMENT, EMERGENCY_RESPONDER.

The 127 enumerated personas anchor specific journeys. The roster scales gracefully toward 1,000+ personas without combinatorial explosion because the axes compose orthogonally and persona records are templates, not allocations.

### §5.5 Why personas are the GTM unlock, not just an engineering artifact

The persona roster looks, on the surface, like an engineering-rigor artifact. It is also the GTM strategy.

- **Per-persona use-case stories.** The 127-persona roster gives the sales motion 127 native use-case stories — each grounded in a real role × tenant × workspace × locale tuple. Sales does not have to invent "what would a forklift driver care about." The forklift-driver persona (Carlos Martinez) already has a journey.
- **Per-vertical anchor accounts.** The enterprise tenants (KrampusCorp, GlobalLogistics, AcmeRawMaterials, Hospital, Restaurant, Newsroom) give the sales motion lighthouse-account templates. Each vertical-pack rollout can name its anchor tenant.
- **Per-locale rollout playbook.** The 6 anchored locales (KR, US, EU, JP, IN, BR) and 20+ supported locales give the GTM team an explicit geographic-rollout sequencing template.
- **Per-collar-color training pattern.** The 6 collar-colors give learning-and-development teams an explicit per-cohort training plan, not a generic one-size-fits-all rollout.

In short: the persona graph is not just engineering rigor. It is the demand-generation engine, the sales-enablement engine, and the customer-success enablement engine for the first 18 months of GTM.

### §5.6 A day in Yejin's life — what continuity-of-identity feels like

To make the personas case visceral, picture one day from Yejin Park's life (companion doc: `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`).

- **6:15 AM.** Yejin's passkey unlocks the oyatie shell on her phone. The shell shows her in her personal tenant. She checks family messages — her daughter has a field trip permission slip due, signed via oyatie's notes + forms composition. She reviews her side-business bakery's overnight orders in marketplace.
- **6:45 AM.** Yejin's morning commute. The shell automatically projects her into her hospital tenant as she taps into the hospital Wi-Fi geofence. Her role projection changes from "parent + side-business owner" to "ICU nurse." The Cedar permit set, the navigation density, the ontology projections (now seeing PHI), and the workflow templates (now seeing clinical-shift-handoff, MAR administration, break-glass access) all switch — but the same gestures (approve, route, attach evidence) apply.
- **7:00 AM.** Shift handoff. Yejin reviews her assigned patients in oyatie's clinical surface (a capability tier over ontology + workflow-engine, gated by HIPAA-equivalent KR-Medical-Records pack). She acknowledges receipt of the handoff, uses Cedar-mediated break-glass to read one out-of-team patient's chart (audit-logged with reason), and updates a care plan.
- **10:30 AM.** Yejin's mother (in her family tenant) sends a message about her grandmother's medication. Yejin reads it in her personal tenant — the work-vs-personal tenant boundary (ADR-0311) cleanly isolates it. The message uses oyatie's MLS (RFC 9420) E2EE. Yejin schedules a video call with her mother for that evening.
- **12:00 PM.** Lunch break. Yejin reviews bakery orders. She approves a custom-cake order, generates an invoice via marketplace's DealSet, schedules production for the next morning. Cedar permits route her as "tenant_admin of small-business-tenant" — different from "employee of hospital-tenant."
- **3:00 PM.** Yejin's daughter's school sends a message — a teacher needs Yejin to review and sign a permission form. Yejin signs the form via her personal tenant. Audit-chain records the signing.
- **6:30 PM.** Shift ends. Yejin clocks out. Her shell automatically projects back to personal tenant as she leaves the geofence. The "work" identity stays in the work-tenant; she does not have to consciously sign out of anything.
- **8:00 PM.** Family video call with her mother and grandmother. Same meet surface. Same gestures. Different tenant context.
- **9:00 PM.** Yejin reviews her bakery's payroll for her one part-time helper, paid via marketplace + payments. She also reviews her grandmother's medical-power-of-attorney delegation (ADR-0305 delegated-agent authority chain) — Yejin has been designated her grandmother's medical proxy.
- **10:15 PM.** Yejin reviews a continuing-education learning module for her nursing license. The learning module sits in oyatie's learning-management microservice — a capability tier she sees via her hospital tenant. She tracks 30 minutes of CEU credit.

Across one day, Yejin moves through:
- 3 active tenants (personal-family, hospital-work, side-business).
- 4 audience-types (B2B_HEALTHCARE_PROVIDER, B2C_FAMILY_PARENT, B2C_CONSUMER, B2B_TENANT_ADMIN).
- 2 device profiles (mobile-primary, briefly desktop at the hospital workstation).
- 3 compliance posture overlays (KR-Medical-Records, KR-PIPA, COPPA-adjacent for her daughter's school).
- 12+ "approve / sign / route / attach" gestures — all using the same UX shell vocabulary.

There is no per-tenant retraining. No re-onboarding. No "I have to remember which app this is in." That is the unified-ecosystem thesis lived out one day at a time.

### §5.7 A day in Marcus Chen's life — what scale feels like

To balance Yejin's micro-perspective, picture Marcus Chen's day as a 5,000-employee multinational CEO:

- **5:30 AM.** Marcus's first signal is the ops-dashboard-control-center microservice projecting his org's overnight metrics. P&L flash, key-account-health flash, SEV incident flash, regulatory-filing-status flash. Cedar permits him as CSUITE; the ontology shows aggregate roll-ups across KrampusCorp + sovereign-child subsidiaries (ADR-0313).
- **6:30 AM.** Marcus reviews his calendar. His EA (Olivia Reyes) has prepared a board pre-read deck — sites + notes + drive composition. Marcus reviews and signs off on the pre-read in 15 minutes.
- **8:00 AM.** Board meeting. Six independent directors join via meet, with recording + transcription enabled (per board-governance pack). Diana (the external auditor) presents the SOC 2 + ISO 27001 + KR-PIPA quarterly evidence — pulled from one audit-chain stream, not three separate compliance tools. The board votes on a $500M acquisition; the vote is recorded as a marketplace DealSet (ADR-0314) for the M&A transition.
- **10:30 AM.** Marcus's CFO walks him through the M&A integration plan. The plan uses ADR-0313 conglomerate-tenant hierarchy: the acquired tenant becomes a sovereign-child of KrampusCorp's parent tenant, with policy-mediated cross-tenant grants. No data merger, no integration project — just a Cedar policy update plus a marketplace DealSet for the transition.
- **12:30 PM.** Marcus reviews three quarterly performance check-ins — including Aiyana Singh (his senior ML engineer in IN). Same performance-management microservice (a capability tier surfaced by Workday-equivalent surface for Marcus, surfaced as a self-development tracker for Aiyana). Cedar gates Marcus's read of Aiyana's full record; Aiyana sees her own record + her manager's notes; their HR director (Priya Krishnan) sees the team aggregate.
- **3:00 PM.** Marcus's CHRO (Linda Foster) walks him through a workforce-realignment plan — closing one EU office, opening a new APAC office. The workflow-studio surface composes the cross-jurisdictional labor-law overlays (ADR-0304 cross-jurisdiction conflict resolution) — EU Worker Council notification clock, Korean Labor Standards Act consultation clock, Japan APPI data-residency for affected workers' records.
- **5:30 PM.** Marcus joins a town-hall over meet — recorded, transcribed, translated into 6 languages live via the translate microservice. 4,200 of the 5,000 employees join (the others see it as a recording within 24 hours). The unified shell means the EU-locale, JP-locale, and KR-locale employees see the town-hall in their own language without any per-locale conferencing tool.
- **8:00 PM.** Marcus's family time. His tenant projects back to personal. His daughter shares her school recital recording (a recordings + drive composition). His son shares a code project (developer-sdk + notes). The same shell, the same gestures.
- **10:00 PM.** Marcus reviews tomorrow's investor-day deck. The deck pulls live data from data-warehouse + analytics + finops-portal — no copy-and-paste, no slide-version-drift.

Marcus's day touches 9 microservices, 2 tenants (KrampusCorp work, personal-family), 4 audience-types (CSUITE, B2B_BOARD_DIRECTOR-as-attendee, B2C_FAMILY_PARENT, B2C_CONSUMER), 3 locales (KR-resident, US-traveling, board with EU + JP + UK + IE + DE directors). One trained vocabulary. One audit chain. One identity.

### §5.8 The persona-tenant matrix — a snapshot

Beyond individual personas, the persona-tenant matrix shows how oyatie's continuity-of-identity model collapses identity sprawl across an enterprise's life. A snapshot:

- **5,000-employee enterprise (e.g., KrampusCorp).** ~5,000 active human identities, each with passkey + 1-3 tenant memberships (the enterprise + their personal + possibly side-business). In a fragmented portfolio, those 5,000 humans have ~600,000 tool-specific accounts (112 tools × ~5,000 average penetration). In oyatie, they have 5,000 identities and ~12,000 tenant memberships. The identity-management surface area drops by ~50x.
- **Mid-size SaaS startup (~500 employees).** Same dynamic at smaller scale. Identity surface area drops from ~50,000 accounts to ~500 identities + ~1,200 tenant memberships.
- **Global multinational (~50,000 employees).** Identity surface area drops from ~5M accounts to ~50,000 identities + ~150,000 tenant memberships. The CIO's identity-management cost goes from "industrial-scale" to "manageable."

This is the operational form of the unified-ecosystem thesis. The fragmentation tax is paid per-account-per-tool-per-year. The unified ecosystem pays it per-identity-per-tenant.

---

## §6. The journeys it tells

### §6.1 The journey catalog at a glance

The user-journey catalog is the connective tissue between the personas, the ADRs, the microservices, and the GTM motion. It enumerates **150 canonical user journeys (j01-j150)** covering every load-bearing flow in the platform.

- **j01-j20 — Life-safety journeys.** Emergency-services flow (ADR-0298), account-recovery flow (ADR-0299), survivor-safety / domestic-abuse mode (ADR-0301), cognitive-impairment decision resilience (ADR-0303), deceased-user inheritance (ADR-0302), disaster-mode cell resilience (ADR-0306), warrant-piercing flow (ADR-0312), whistleblower / source-protection (ADR-0300). These are the journeys that establish oyatie as a platform that treats users as humans, not just as transactions.
- **j21-j50 — Hero-product day-to-day journeys.** Messenger sends + replies + reactions; Mail inbox + threading + filters; Community post + comment + moderation; Marketplace browse + buy + settle; Workflow Studio canvas + run + audit; Drive upload + share + version; Calendar schedule + join + reschedule; Meet join + share + record; Notes write + link + share; Sheets edit + formula + share; Slides edit + present + share; Recordings record + transcript + share; Shorts post + react + share; Social follow + post + react.
- **j51-j75 — Cross-product compound journeys.** Quote-to-cash (CRM + workflow + marketplace + payments + audit-chain). Hire-to-onboard (HR + workflow + identity + community + tasks). Incident-to-postmortem (ITSM + meet + notes + audit). Procure-to-pay (marketplace + workflow + payments + supply-chain). Plan-to-execute (workflow-studio + tasks + meet + notes). The journeys that prove the unified-ecosystem thesis is operationally real, not just an architecture diagram.
- **j76-j100 — Locale-pack overlay journeys.** Korean KR-CSAP audit prep. EU GDPR DSAR response. Japan APPI consent flow. Indian DPDP 2023 consent flow. Brazilian LGPD breach notification. US state-by-state privacy variation. The journeys that prove oyatie's locale-pack model dissolves the per-jurisdiction integration tax.
- **j101-j125 — Tenant-to-tenant ecosystem journeys.** Inter-tenant supply-chain coordination (KrampusCorp ↔ AcmeRawMaterials ↔ GlobalLogistics). Hiring-from-Community (Anya recruiting a freelance investigator, Chris job-hunting post-layoff). Gig-economy multi-platform-worker (Sarah-the-delivery-driver across Amazon DSP tenant + side-hustle tenant). M&A merger (KrampusCorp acquires AcmeRawMaterials; tenant-conglomerate hierarchy per ADR-0313).
- **j126-j147 — Diana / Priya / Sam / Chris ecosystem journeys.** The auditor (Diana), the HR director (Priya), the internal-audit director (Sam), and the laid-off-engineer-turned-job-seeker (Chris) walk through 22 inter-tenant journeys that demonstrate the ecosystem behaves coherently across every type of cross-tenant interaction. The j126-j150 catalog file is the most-cited single journey artifact in the corpus.
- **j148-j150 — Creative ecosystem journeys.** A creative agency (with designers, account managers, and freelance contractors) collaborates with a multinational client (KrampusCorp) on a campaign, settling through marketplace, sharing assets through drive, planning through workflow-studio, reviewing through meet + recordings.

Total: **150 journeys, 1,092+ artifacts in the catalog, 1,340+ per-microservice implementation-plan slices derived from journeys.**

### §6.2 Why journeys matter at the executive level

Three reasons:

1. **Journeys are the operational definition of "the platform works."** A journey passes if and only if the user can complete the flow end-to-end without a context switch, a re-authentication, a separate-tool integration, a separate-tool training, or a separate-tool compliance review. A journey catalog of 150 is an operational test set for the unified-ecosystem thesis.
2. **Journeys are the sales motion.** Every journey is a customer story. The sales-leader who walks a prospect through j43 (quote-to-cash) does not have to explain the architecture — they show the prospect what their CFO + their VP of Sales + their AR clerk + their audit team see in the same flow.
3. **Journeys are the implementation backlog.** Each journey decomposes into per-microservice implementation-plan slices. 1,340+ IP slices is the engineering-team's marching orders. The corpus is sized; the team is sequencing.

### §6.3 The compounding-narrative effect

The journey catalog is the artifact that makes oyatie defensible against an incumbent vendor's "we'll add that feature too" response.

A single feature is easy to copy. A coherent 150-journey narrative is not. The compounding-narrative effect — every journey reinforces every other journey because the substrate is shared — is what every previous "unified suite" attempt missed.

### §6.4 Three journeys executives should know by heart

The journey catalog is large. Three specific journeys are worth memorizing for any board director, investor, or sales-leader who needs to tell the oyatie story in 90 seconds.

**Journey j43 — Quote-to-Cash (the unified-enterprise demo).**

A B2B sale moves from a prospect inquiry to a signed contract to a paid invoice to a recognized revenue entry to an audit-ready evidence record. In a fragmented portfolio, this journey crosses 12+ tools (CRM for opportunity, configure-price-quote for quote, contract-lifecycle-management for the contract, e-signature for the signing, marketing-automation for the post-sale handoff, customer-success for onboarding, finance system for billing, payments processor for collection, GL for revenue, audit-tool for evidence, BI for reporting, data-warehouse for analysis). Each tool has its own data model, its own user, its own approval gate, its own integration tax. The cycle time, end-to-end, in a mature enterprise is 45-90 days.

In oyatie, the same journey is one DealSet (ADR-0314) in marketplace, surfaced through CRM (capability tier), workflow-engine (state machine), contract-lifecycle-management (one microservice), payments (one microservice), and audit-chain (one substrate). The user-visible flow is approve > sign > settle > recognize > evidence. The cycle time, end-to-end, target is 5-15 days for a well-instrumented seller. The audit trail is one timeline, not 12 reconciled exports.

**Journey j62 — Hire-to-Onboard (the unified-HR demo).**

A new employee moves from a job posting to a signed offer to a first day to a full-productive day. In a fragmented portfolio, this crosses ATS + interview-scheduling + assessment-platform + offer-letter-tool + HRIS + payroll + benefits + IT-provisioning + Identity + Slack-channel-onboarder + LMS + manager-1:1-platform + culture-survey-platform. The fragmentation means the new employee waits 5-10 business days for IT-provisioning, then another 5-10 days for full role-onboarding.

In oyatie, the same journey is one workflow run in workflow-engine, mediated by identity (passkey-provisioned), HR-capability-tier (in HCM-equivalent surface), Cedar permits auto-granted on day-zero, learning-management modules auto-enrolled, community-channel auto-joined, manager-1:1 auto-scheduled in calendar. Time to first-productive-day target is 4-24 hours, not 10-20 business days. The employee starts pre-fluent because they already know the platform.

**Journey j101 — Inter-tenant supply-chain coordination (the unified-ecosystem demo).**

KrampusCorp orders raw materials from AcmeRawMaterials. AcmeRawMaterials ships via GlobalLogistics. In a fragmented portfolio, this is three separate ERPs, two separate EDI integrations, three separate audit trails, and a settlement-reconciliation process that takes 30-60 days. Disputes propagate slowly across tenant boundaries because each tenant only sees its own slice.

In oyatie, the journey is one DealSet between three tenants, with each tenant's Cedar permits granting the other tenants exactly the cross-tenant visibility they need (and nothing more — sovereign-child semantics per ADR-0313). The supply-chain-planning microservice in KrampusCorp's tenant has policy-mediated read into AcmeRawMaterials's inventory + GlobalLogistics's shipment status. Settlement is one DealSet transition, not a three-way reconciliation. Dispute resolution is one workflow run, not a 60-day phone-call chain.

These three journeys, told back-to-back, cover the day-to-day fragmentation tax of three different enterprise functions. They are the GTM team's three demo scripts.

---

## §7. The compliance + sovereignty posture

### §7.1 Compliance packs

Oyatie ships **25+ compliance pack overlays** (ADR-0251):

- **United States:** SOC 2, HIPAA, FedRAMP (Low/Moderate/High), IL5, IL6, CCPA, PCI DSS, GLBA, FERPA, NIST 800-53, NIST 800-171, CMMC, state-by-state privacy (CA, VA, CO, CT, UT, IA, IN, MT, OR, TX, FL, TN, DE, NJ, MN, RI, KY, NE, MD, NH).
- **European Union:** GDPR, DORA, DSA, DMA, NIS2, EU AI Act, ePrivacy, eIDAS, MiCA, GDPR-sectoral overlays.
- **Korea:** KR-PIPA, KR-CSAP, KR-FSS, KR-Medical-Records-Law, KR-Labor-Act.
- **China:** PIPL, DSL, CSL, CAC.
- **Japan:** APPI, JP-Labor-Standards-Act, JP-FISC.
- **India:** DPDP-2023, RBI-pack, SEBI-pack.
- **Brazil:** LGPD, BR-Labor-Code, BR-Anvisa.
- **Industry overlays:** GxP (pharma), Basel (banking), Solvency II (insurance), API 510/570 (oil-and-gas), GFSI (food safety), SOX (public-company finance), MiFID II (EU securities).

### §7.2 Cell-tier certification model

Oyatie's deployment substrate uses a cell-based topology (ADR-0248, Amazon-shape cellular architecture). Each cell carries a certification tier:

- **Tier 0:** Best-effort cells (community deployments, non-regulated workloads).
- **Tier 1:** SOC 2 + GDPR + privacy-pack-default cells.
- **Tier 2:** HIPAA + PCI + financial-services-grade cells.
- **Tier 3:** FedRAMP High + IL5/IL6 + KR-CSAP + EU-sovereign + CN-sovereign cells.

A tenant lands in the certification tier their pack overlay requires. A regulated tenant cannot accidentally schedule work onto a non-certified cell. The cell-tier registry is Cedar-enforced.

### §7.3 Build-ahead-of-certification doctrine (ADR-0250)

Oyatie's posture is to build certified shape on day one, never retrofit compliance. This mirrors how Apple Pay rolled out — built to the highest compliance bar on day one, then certified jurisdiction-by-jurisdiction.

For the board, this means oyatie's HIPAA + GDPR + FedRAMP + EU-AI-Act + DORA-readiness is not a roadmap item. It is the architecture.

### §7.4 Dual-tenant identity boundary (ADR-0311)

The personal-vs-work tenant boundary is enforced at the substrate level. A user's personal tenant data and their employer tenant data are Cedar-isolated, audit-isolated, and (where pack-required) encryption-key-isolated.

- An employer cannot read an employee's personal-tenant data.
- An employee cannot leak employer-tenant data into their personal-tenant.
- A divorce, a job change, or a death (ADR-0302 deceased-user inheritance) does not collapse the boundary.

This is the engineering-rigor backbone of the personas case in §5.

### §7.5 Per-jurisdiction overlays

Oyatie's locale-pack model treats jurisdiction as a runtime overlay, not a per-jurisdiction product fork. KR-Labor-Act overtime rules apply to KR-tenanted workforce flows; JP-APPI consent flows apply to JP-tenanted personal data; EU-DSA transparency-reporting applies to EU-tenanted marketplace listings. The same substrate handles all of them; the pack manifests express the differences.

### §7.6 Sovereignty stance

For governments and regulated industries:

- **Data residency:** per-cell, per-tenant, with Cedar enforcement.
- **Sovereign clouds:** EU-sovereign cells (compliant with GDPR + DORA + EU-AI-Act + NIS2); CN-sovereign cells (PIPL + DSL + CSL); KR-sovereign cells (CSAP + PIPA); FedRAMP High + IL5/IL6 cells (US federal).
- **Court-warrant scoped piercing (ADR-0312):** law-enforcement access to encrypted data is gated by a Cedar-enforced warrant-scope envelope. Piercing is auditable, scope-limited, and time-bounded. No master key, no admin backdoor.
- **Warrant-piercing audit:** every piercing event is itself an audit-chain event, visible to the tenant's privacy officer.

This posture is what lets oyatie ship into pharmaceutical, financial-services, healthcare, defense-contractor, and government tenants without per-vertical product forks.

### §7.7 What the build-ahead-of-certification doctrine actually means

ADR-0250 (build ahead of certification) is one of the most strategically important commitments in the corpus. It says: oyatie engineers every microservice, every data flow, every audit event, and every Cedar permit to the highest applicable certification bar from day one. Certification itself happens jurisdiction-by-jurisdiction, but the engineering shape never has to be retrofitted.

Concretely, this means:

- **FedRAMP High shape from day one.** Every substrate microservice ships with FIPS-validated cryptography, supply-chain-attestation (sigstore + cosign + Rekor anchored to FIPS 140-3 L3 HSM root signing per ADR-0254), audit-immutability, separation-of-duties, and incident-response runbooks.
- **HIPAA shape from day one.** Every microservice that may touch PHI ships with Cedar-enforced minimum-necessary access, audit-chain immutability, breach-notification workflows, and BAA-ready DPA language.
- **EU AI Act high-risk shape from day one.** Every microservice that includes an ML model ships with model cards, dataset cards, fairness audit records (ADR-0309), drift-detection signals, and rollback paths (ADR-0308).
- **KR-CSAP shape from day one.** Every microservice destined for KR-tenant deployment ships with KR-data-residency cells, KR-Korean-language audit-evidence formatting, and KR-FSS-equivalent reporting cadence.

The contrast with the typical SaaS-incumbent posture is sharp. Most SaaS vendors achieve compliance certifications post-launch via retrofit. The retrofit cost is high (often 18-36 months of remediation per certification), the retrofit risk is high (compliance gaps surface in audit), and the retrofit timing is unpredictable (each certification body sets its own cadence).

Oyatie's "certified shape day one" approach means the platform can serve a regulated tenant before the formal certification lands — the engineering bar is already met; the certification is paperwork on top.

### §7.8 The conglomerate-tenant pattern

ADR-0313 (conglomerate-tenant hierarchy) deserves an executive-level unpacking because it is the architectural primitive that lets oyatie scale into Fortune 500 + multinational tenants without becoming a per-subsidiary product.

A Fortune-500 multinational typically has:

- A parent corporate entity (e.g., KrampusCorp).
- 10-100 sovereign-child subsidiaries (e.g., KrampusJapan, KrampusBrazil, KrampusFinance, KrampusAutoParts) — each with its own legal entity, its own regulatory regime, its own audit obligations, and often its own data-residency requirements.
- A complex matrix of cross-entity authority (parent can read certain subsidiary data; subsidiary can settle deals on parent's behalf; certain subsidiaries are firewalled from each other due to information-barrier rules per ADR-0319).

In a fragmented portfolio, each subsidiary runs its own SaaS portfolio. The parent's view of subsidiary operations is mediated by 100+ separate consolidation tools — Hyperion, OneStream, Anaplan, BlackLine, and a long tail of finance consolidation tools. Each subsidiary's per-jurisdiction compliance posture is independent. Cross-entity workflows (e.g., parent-imposed treasury-policy enforcement across all subsidiaries) require dedicated integration projects.

In oyatie, the parent + each subsidiary is a sovereign tenant in a conglomerate hierarchy. Cedar policies at the parent level can grant scoped read or write access to subsidiary data — but the subsidiary's data sovereignty is preserved. Cross-entity workflows are first-class. Consolidation is one ontology projection across the conglomerate, not 100 separate integrations.

This is the architectural primitive that lets oyatie serve the Fortune 500 without forking the platform per subsidiary.

---

## §8. The 30-ADR doctrine cluster (post-keystone-bundle 2026-05-20)

The keystone bundle of 2026-05-20 and the subsequent Wave-3-G doctrine expansion lay down a 30-ADR cluster that codifies how the platform behaves under load, edge cases, hostile inputs, and life-safety conditions.

### §8.1 The abuse-defence + safety layer

- **ADR-0297 — Abuse-Defence Baseline.** Anti-bot, anti-spoof, anti-scrape. Internet-facing surfaces get a uniform abuse-defense profile.
- **ADR-0298 — Emergency-Services Bypass.** Life-safety hard rule: nothing the platform does can stand in the way of a 911 / 119 / 110 / 112 call or its equivalent. Policy is denylist-bypassed for life-safety actions; audit is preserved.
- **ADR-0299 — Account-Recovery Resilience.** Hijack-recovery + lost-credential recovery + family-bereavement recovery. Recovery is rooted in passkey + offline-shamir + delegated-trusted-contact patterns.

### §8.2 The critical-path doctrine cluster (300s)

- **ADR-0300 — Whistleblower + Press Freedom.** Source-protection workflow under court pressure.
- **ADR-0301 — Survivor Safety.** Domestic-abuse mode + stalker-defense + concealment-friendly UX flow.
- **ADR-0302 — Deceased-User Inheritance.** Estate transition, beneficiary access, regulator-friendly evidence trail.
- **ADR-0303 — Cognitive-Impairment Decision Resilience.** Slow-down + co-sign + delegated-decision flows for cognitive-decline scenarios.
- **ADR-0304 — Cross-Jurisdiction Conflict Resolution.** When KR-Labor-Act + US-California-Privacy + EU-DSA disagree, the conflict-resolution policy is explicit, auditable, and tenant-configurable.
- **ADR-0305 — Delegated-Agent Authority Chain.** When a user delegates authority to an AI agent (or a human assistant, or a power-of-attorney holder), the authority chain is Cedar-traceable.
- **ADR-0306 — Disaster-Mode Cell Resilience.** Cell-evacuation, regional-failover, and graceful-degradation when a region is offline.

### §8.3 The detection substrate (DRMP) — 307-310

The Detection / Response / Mitigation / Prevention layer:

- **ADR-0307 — Detection Substrate (streaming + batch).** The signal-emission, signal-aggregation, and signal-evaluation primitives every microservice publishes into.
- **ADR-0308 — ML Model Lifecycle.** EU AI Act high-risk-model lifecycle. NIST AI RMF. ISO/IEC 42001 alignment. Model cards, dataset cards, evaluation reports, drift detection, rollback paths.
- **ADR-0309 — Detection Fairness Audit.** Civil-rights compliance baseline for any model that affects employment, credit, housing, healthcare access, or government services.
- **ADR-0310 — Investigation Case-Management.** Internal-security, fraud, abuse, and integrity investigations get a first-class substrate — not an ad-hoc spreadsheet.

### §8.4 The dual-tenant + warrant layer

- **ADR-0311 — Dual-Tenant Identity.** Personal-vs-work boundary (already covered in §7).
- **ADR-0312 — Court-Warrant Scoped Piercing.** Warrant-scoped, time-bounded, Cedar-enforced law-enforcement access.

### §8.5 The conglomerate + marketplace + ERP layer

- **ADR-0313 — Conglomerate Tenant Hierarchy.** Sovereign-child + policy-engine-mediated controlling-entity grant. A KrampusCorp can hold subsidiary tenants (KrampusJapan, KrampusBrazil, KrampusFinance) with policy-mediated cross-tenant access. Each subsidiary is sovereign; the parent governs by policy, not by backdoor.
- **ADR-0314 — Marketplace as Universal Deal-Settlement.** Every commercial exchange is a DealSet. Covered in §3.1.
- **ADR-0315 — ERP Coverage Doctrine (SAP parity).** Full S/4HANA module parity. Covered in §4.2.

### §8.6 The unified-ecosystem doctrine layer (316-320)

- **ADR-0316 — Capability-Tier Over Product Fragmentation.** A product label is a capability-tier projection over the substrate, not a microservice boundary. Covered in §3.3.
- **ADR-0317 — Role-Based Projection + Unified UX Shell.** The same human sees different role projections of the same substrate based on which tenant + role is active. Covered in §3.1 (ONE-UX-SHELL).
- **ADR-0318 — Collar-Color and Workspace Universality.** The 6 collar-colors and 7 workspaces are universal taxonomy primitives — covered in §5.4.
- **ADR-0319 — Front / Middle / Back Office Information-Barrier.** Chinese-wall semantics across front-office (customer-facing), middle-office (risk + compliance), back-office (internal-support) workspaces.
- **ADR-0320 — Apprentice / Intern / Resident / Fellow Transient Identity.** A first-class in-training tier with supervisor co-sign on high-stakes operations.

### §8.7 The B2B leader coverage layer

- **ADR-0321 — B2B SaaS Industry-Leader Coverage.** 165 vendor dossiers, 13 new microservice anchors. Covered in §4.3.

### §8.8 What the cluster says together

The 30-ADR cluster is the answer to the executive question: "How does oyatie behave at the edges?"

- Under hostile inputs (bots, scrapers, spoofers, abuse): ADR-0297.
- Under life-safety pressure: ADR-0298.
- Under credential loss: ADR-0299.
- Under press-freedom + civil-rights pressure: ADR-0300, ADR-0301, ADR-0309.
- Under cognitive-decline or estate-transition pressure: ADR-0302, ADR-0303.
- Under jurisdictional conflict: ADR-0304.
- Under AI-agent delegation: ADR-0305.
- Under regional disaster: ADR-0306.
- Under signal + ML + investigation load: ADR-0307, ADR-0308, ADR-0309, ADR-0310.
- Under personal-vs-work tenant separation: ADR-0311.
- Under court-warrant compulsion: ADR-0312.
- Under conglomerate ownership: ADR-0313.
- Under universal commerce: ADR-0314.
- Under ERP-parity workloads: ADR-0315.
- Under product-label expectation: ADR-0316.
- Under role-projection expectation: ADR-0317.
- Under universal workforce coverage: ADR-0318.
- Under chinese-wall expectation: ADR-0319.
- Under transient-identity expectation: ADR-0320.
- Under B2B leader competitive landscape: ADR-0321.

That is the executive-visible operational envelope of the platform.

### §8.9 The keystone bundle of 2026-05-20 — what came before

The 30-ADR doctrine cluster in §8.1-§8.8 sits on top of an earlier 17-ADR keystone bundle (ADR-0242..0258 plus remediation ADRs) landed in 2026-05-20. The keystone bundle established:

- **ADR-0242 — Oyatie-is-a-tenant.** Even oyatie itself is a reserved-namespace tenant. No carve-outs. The platform follows its own rules.
- **ADR-0243 — Cedar as universal gate.** Every gate is a Cedar evaluation. No policy in code. No backdoor authorization paths.
- **ADR-0244 — Tenant as universal scoping primitive.** Every row, every audit event, every cost dimension carries tenant context. The foundation of the data model.
- **ADR-0245 — Substrate vs product layering.** Substrate microservices serve all products. No duplication. The foundation of the architecture.
- **ADR-0246 — MLS-canonical messenger.** End-to-end encryption uses RFC 9420 (Messaging Layer Security). The cryptographic substrate of communication.
- **ADR-0247 — Self-modification doctrine.** Oyatie's Foundry pipeline runs as a first-class principal under Cedar. The autonomous-execution substrate is itself governed.
- **ADR-0248 — Amazon-shape cellular architecture.** AWS-cell-topology Tiers 0-4, shuffle-sharding, Cloud Hypervisor.
- **ADR-0249 — Multi-category marketplace.** Plugins, apps, workflows, agents, models, datasets — one marketplace, many categories.
- **ADR-0250 — Build-ahead-of-certification.** Certified shape day one; never retrofit compliance.
- **ADR-0251 — Compliance pack primitive.** HIPAA, GDPR, SOC 2, FedRAMP, KR-CSAP, PCI, EU-AI-Act as packs per tenant + cell.
- **ADR-0252 — HLC default + TrueTime tier.** Hybrid logical clocks default for causality; TrueTime opt-in for fin-grade.
- **ADR-0253 — HTTP/3 + QUIC default.** Default protocol across the substrate.
- **ADR-0254 — Kubernetes everywhere + Cloud Hypervisor.** K8s except edge; Cloud Hypervisor + Kata pods.
- **ADR-0255 — Intelligence two-layer substrate.** AI Substrate + Consumer Brand Surface. Absorbs Foundry.

The keystone bundle is the architecture-decision substrate. The 30-ADR doctrine cluster of Wave-3-G is the operational-edge envelope on top of it. Together they constitute the architectural commitment surface that lets the rest of the platform exist coherently.

### §8.10 What the doctrine cluster says about oyatie's posture

If a board director read only the ADR titles (§8.1-§8.7) and nothing else, they would still get the operative posture in three observations:

1. **Oyatie takes the edges seriously.** Most SaaS platforms treat life-safety, survivor-safety, deceased-user inheritance, cognitive-decline resilience, and warrant-piercing as out-of-scope or after-the-fact compliance items. Oyatie treats them as load-bearing ADRs co-equal with the core data model.
2. **Oyatie treats the user as a human across a lifetime.** The dual-tenant boundary, the conglomerate hierarchy, the apprentice-intern-resident transient identity, the cross-jurisdiction conflict resolution, the deceased-user inheritance — these ADRs say the platform is not optimized for a 6-month employment relationship. It is optimized for a 50-year human life.
3. **Oyatie treats AI as a regulated technology.** The ML lifecycle ADR, the fairness audit ADR, the delegated-agent authority-chain ADR — these say the platform is built for the post-EU-AI-Act regulatory regime, not the pre-regulation one.

These three observations are the executive-level signal of where the platform is headed. The detailed ADR text is the engineering substrate that backs the signal.

---

## §9. The roadmap from here

### §9.1 Where Wave-3-G left us (today, 2026-05-20)

- The keystone-bundle ADRs (0242-0292) are merged in `Proposed` status; per-ADR promotion to `Accepted` is gated on the 15-item fix-set in `keystone-bundle-2026-05-20-synthesis.md` §5.
- The unified-ecosystem thesis, training-cost doctrine, enterprise-software-coverage-matrix, and persona master roster are authored and cross-linked.
- 30 of the 30 ADRs in the §8 cluster exist; per-ADR content density still varies. Five ADRs (ADR-0244, ADR-0314, ADR-0316, ADR-0317, ADR-0321) are the load-bearing five and are at the highest density.
- 69 microservices in the corpus, 921 implementation-plan slices, 1,092+ journey artifacts.

### §9.2 Wave 3-H — Content-pass on anchor stubs (in-flight)

The corpus has 1,150-ish anchor-injected-but-stub stubs (12-15 per microservice across ARCHITECTURE.md + compliance.md per the corpus-rigor audit). Wave 3-H expands each into substantive prose per documentation-rigor.md §3.2.1 row obligations. This is the parallel-content-pass that turns the architecture corpus from "shape complete" to "content complete."

### §9.3 Wave 3-I — Capability-tier registry + CI lane authoring

The capability-tier registry (per ADR-0316) is the runtime primitive that turns a tenant's purchase of "CRM" or "ITSM" into a Cedar permit bundle + ontology projection set + workflow template library + UX shell vocabulary. Wave 3-I authors:

- The registry schema.
- The first wave of tier manifests (CRM, ITSM, HRIS, ERP-finance, ERP-supply-chain).
- The CI lanes that enforce no-product-fragmentation, no-grouping-microservices, and capability-tier coverage.

### §9.4 Wave 3-J — Code authoring (the microservices need actual Rust)

Wave 3-J is the heaviest single piece of work in the roadmap. The 69 microservices need their actual Rust crates, their actual OpenAPI 3.2.0 contracts, their actual AsyncAPI 3.1.0 event definitions, their actual proto3 gRPC definitions, their property tests, their replay tests, and their integration tests.

This is paced in launch sequencing per Wave 3-K below.

### §9.5 Wave 3-K — Launch sequencing

Which microservices ship in which quarter is governed by the dependency DAG. Roughly:

- **Q1 (substrate first):** identity, tenancy, cell, governance, consent-graph, audit-chain, compliance, cloud-iac, cloud-k8s, cloud-secrets, observability, api-gateway, application, foundry. 14 microservices, no user-facing surface yet.
- **Q2 (substrate + first user-facing wave):** ontology, workflow-engine, intelligence, network, detection, feature-flags, translate + the first 5 hero surfaces (messenger, mail, community, marketplace, drive). 12 microservices.
- **Q3 (productivity wave):** calendar, meet, recordings, notes, sheets, slides, sites, forms, tasks, plugin-app-store + workflow-studio. 11 microservices.
- **Q4 (consumer-social + first enterprise wave):** shorts, social, docs, whiteboard, design-collaboration + crm, analytics, comms-email, developer-sdk, finops-portal, payments, treasury. 12 microservices.
- **Y2 H1 (ERP wave):** warehouse, global-trade, real-estate, supply-chain-planning, production-planning, quality-management, plant-maintenance + workplace-integration, ops-dashboard-control-center. 9 microservices.
- **Y2 H2 (B2B leader wave):** marketing-automation, contact-center, performance-management, learning-management, itsm, incident-management, financial-planning, data-warehouse, contract-lifecycle-management, data-pipeline, healthcare-integration, connect. 12 microservices.

Total: 69 microservices shipped across 24 months.

### §9.5.1 The launch-sequencing rationale

Why this specific sequencing? Two principles govern:

- **Substrate before surface.** A consumer-product surface cannot ship until identity, tenancy, Cedar, ontology, workflow-engine, and audit-chain are all production-stable. Any consumer that lands on an unstable substrate has a bad first experience that compounds into churn. Q1 is therefore substrate-first, with no consumer-visible surfaces.
- **Hero surfaces before vertical depth.** The consumer-and-productivity hero surfaces (messenger, mail, marketplace, drive) are the demand-generation engine. They need to be running at scale (10M+ users) before the enterprise sales motion has the social proof it needs. Q2-Q3 is therefore hero-first, with B2B + ERP layering on top in Q4 and Y2.

The launch sequencing is also designed so that no quarter's launch creates a dependency on the next quarter's launch. Each quarter ships independently usable functionality. If Q3 slips by a quarter, Q2's launches keep producing user growth.

### §9.5.2 The capacity-and-team shape implied

The 24-month launch sequencing implies a specific engineering team shape:

- **Substrate team:** ~80-120 engineers across the 21 substrate microservices. Each substrate microservice has 3-6 engineers, plus the architecture council, plus the SRE-reliability function.
- **Hero-surface team:** ~150-200 engineers across the 18 hero microservices. Each hero surface has 6-12 engineers depending on consumer-grade quality requirements.
- **Enterprise + vertical team:** ~200-300 engineers across the 30 enterprise + vertical microservices. Many of these are co-built with anchor customers (the lighthouse-account strategy).
- **Foundry + tooling team:** ~30-50 engineers maintaining the autonomous-execution pipeline (the agent + CI + reviewer-agent + merge-queue substrate that scales code authoring).
- **Quality + security + compliance team:** ~50-80 engineers across security review, fairness audit (ADR-0309), compliance pack authoring, certification operations.

Total engineering team shape: **~500-700 engineers at full velocity**. This is large but not unprecedented — Atlassian, GitLab, Notion, and Linear have run engineering teams in this range. The differentiating factor is that oyatie's autonomous-execution pipeline (Foundry) lets a smaller human team author more code per engineer-week than a traditional SaaS company.

### §9.6 Wave 3-L — Tenant migration journeys

The single highest-leverage GTM artifact is the migration-from-incumbent journey set:

- **FROM SAP S/4HANA TO oyatie ERP** — module-by-module migration journey, data extraction, validation, parallel-run, cutover.
- **FROM Salesforce TO oyatie CRM** — opportunity / account / lead / case migration; Apex equivalence in workflow-studio; AppExchange migration to oyatie plugin-app-store.
- **FROM Workday TO oyatie HR** — worker / position / job / comp / talent / learning migration.
- **FROM ServiceNow TO oyatie ITSM** — incident / change / problem / asset / configuration migration.
- **FROM Atlassian (Jira + Confluence) TO oyatie** — issue / wiki / project / portfolio migration.
- **FROM Microsoft 365 TO oyatie** — mail / calendar / drive / teams / sharepoint / power-platform migration.
- **FROM Stripe TO oyatie marketplace settlement** — platform-payment migration with parallel-run.
- **FROM Snowflake TO oyatie data-warehouse** — table / view / pipeline / RBAC migration.

Each migration journey is itself a multi-month customer engagement and a multi-million-dollar contract. The migration playbooks are the GTM unlock for the first 18 months of enterprise sales.

### §9.7 The migration playbook shape

Every migration journey follows the same five-phase shape, drawn from precedent in SAP-to-Workday migrations, Lotus-Notes-to-Microsoft-365 migrations, and on-prem-to-cloud migrations:

1. **Phase 0 — Diagnostic.** A 4-6 week engagement where oyatie's customer-success team maps the incumbent tool's configuration, data model, integrations, and user-base into oyatie's capability-tier projections. Deliverable: a per-tenant migration plan with risks, scope, and timeline.
2. **Phase 1 — Substrate setup.** Tenant onboarding to oyatie, including identity provisioning, Cedar policy seeding, compliance pack activation, audit-chain stream provisioning, and ontology object-type alignment with the incumbent tool's data model. Deliverable: a substrate-ready tenant.
3. **Phase 2 — Parallel run.** The incumbent tool keeps running while oyatie is configured in shadow mode. Data flows are replicated; workflows are mirrored; users get optional early access. Deliverable: parallel-run validation report demonstrating data-and-workflow parity.
4. **Phase 3 — Cutover.** A planned cutover window. Incumbent tool's data is migrated to oyatie ontology; users are switched over; integrations are repointed. Deliverable: cutover runbook executed; live oyatie operation.
5. **Phase 4 — Hardening.** Post-cutover stabilization. Audit-chain reconciliation. User-training delivered. Incumbent tool decommissioned. Deliverable: certified post-migration audit report.

The migration playbook scales from a 6-week SMB migration to a 12-24 month enterprise migration. The shape is the same; the duration scales with portfolio size.

### §9.8 The Wave 3-M and beyond — what comes after the launch sequence

Beyond the 24-month launch sequence, the next-wave waves are:

- **Wave 3-M (Y3) — Industry-vertical expansion.** Beyond the 9 new vertical packs in Wave 3-G, the next 12-24 verticals (telecommunications, mining, aerospace-defense, government-services, education-higher-ed, education-K12, religious-organizations, NGO-international-aid, professional-sports, gaming, entertainment, fashion-retail, etc.).
- **Wave 3-N — Locale + sovereignty expansion.** From 6 anchored locales (KR, US, EU, JP, IN, BR) and 20+ supported locales to full global coverage. Adding MENA-pack (Saudi, UAE, Egypt), Africa-pack (Nigeria, South Africa, Kenya, Egypt), Latam-pack (Mexico, Argentina, Chile, Colombia, Peru), SE-Asia-pack (Thailand, Vietnam, Indonesia, Philippines, Malaysia, Singapore expansion), Oceania-pack (Australia, New Zealand).
- **Wave 3-O — Edge + offline + air-gap.** ADR-0254 already commits Cloud Hypervisor + Kata-containers + edge-deployment shape. Wave 3-O is the actual edge buildout — for tenants who need air-gapped (defense, intelligence), offline-tolerant (field service, maritime, mining), or low-bandwidth (rural healthcare, agricultural cooperatives) deployments.
- **Wave 3-P — Agentic-product layer.** Oyatie's intelligence substrate becomes a first-class agentic-product layer. Users (consumer + enterprise) get a personal AI assistant, a team AI assistant, and an org-wide AI assistant — each Cedar-permitted differently, each accessing the unified data substrate. This is where the AI moat against fragmented incumbents widens fastest.

---

## §10. What this means for investors, board, and GTM

### §10.1 TAM expansion

Before the Wave-3-G doc expansion, oyatie's narrative was "consumer messenger + community + marketplace + workflow studio + a long-term ERP-parity ambition." The TAM that supported was the consumer-messaging + consumer-productivity adjacent TAM — roughly the WhatsApp + Notion + Stripe + TikTok overlap. Large but bounded.

After Wave-3-G, the narrative is "the unified ecosystem that displaces the enterprise SaaS portfolio." The TAM is the global enterprise-SaaS market — Gartner sizes that at approximately $200B (2024) and growing 12-18 percent annually. Plus the consumer-productivity adjacent TAM. Plus the migration-services TAM. Plus the developer-ecosystem TAM (the plugin-app-store).

This is not a doubling of TAM. It is an order-of-magnitude expansion.

### §10.2 The moat

The moat is the unified-ecosystem flywheel:

- **Day-zero adoption advantage.** A new user (consumer or enterprise) does not have to migrate to oyatie one tool at a time. They get the whole substrate from day one.
- **Cross-career training amortization.** The 17-year-old retail clerk who learns oyatie at their first part-time job is a pre-trained user of every employer's CRM, ERP, HR, ITSM, and audit surface for the next 50 years. The training-cost curve over a career compounds in oyatie's favor and against the fragmentation incumbents.
- **Compounding network effects.** Every new tenant on the marketplace makes the marketplace more useful to every other tenant. Every new plugin in the app-store makes the app-store more useful to every other tenant. Every new compliance pack makes oyatie usable for more verticals.
- **Substrate cost advantage.** Building the 100th capability tier on top of the existing substrate is incrementally cheap. Building the 100th SaaS app is incrementally expensive. The unit economics get better over time.

### §10.3 The risk

Three honest risks:

- **Execution risk.** 69 microservices, 24 months, 24 vertical packs, 25+ compliance packs. The largest unified-ecosystem attempt ever undertaken. Execution discipline + architecture coherence + AI-assisted code authoring (Wave 3-J) is the bet.
- **GTM risk.** Enterprise software sales is a relationship business. The incumbent vendors (SAP, Salesforce, Microsoft, Workday, ServiceNow) have 10-30 years of CIO relationships. Oyatie needs a lighthouse-account strategy + a migration-services strategy + a partner-channel strategy that can move customers off those relationships.
- **Per-vertical market-entry risk.** Insurance, oil-and-gas, pharma, legal-services, hospitality, agri-food — each vertical has incumbents, regulatory inertia, and per-vertical buying cycles. The 9 new vertical packs (§4.4) are bets, not guarantees.

### §10.4 The next 12 months — the strategic focus

To convert architecture-completeness into market-traction, the next 12 months should concentrate on:

1. **Five hero products at consumer-grade quality.** Messenger, mail, community, marketplace, drive. The consumer-side adoption flywheel starts here. Get to 10M+ users on each before pivoting to enterprise.
2. **Three vertical packs as enterprise lighthouses.** Healthcare, financial services, public sector. Each carries its own anchor tenant and its own migration playbook. These three are where the build-ahead-of-certification doctrine pays off first.
3. **The migration-from-incumbent journey set.** Eight journeys (per §9.6) authored as customer-facing migration playbooks. These are the GTM unlock.

### §10.5 The economic case in one paragraph

A 5,000-person enterprise running a 110-tool SaaS portfolio carries roughly $250M-$1B in cumulative fragmentation cost per year (SaaS bills + integration spend + training pressure + lost productivity + compliance + exit-cost amortization). Migrating to oyatie carries a transition cost (estimated 12-24 months and 5-15% of one year's fragmentation cost) plus an ongoing oyatie license. The steady-state savings — net of license — are roughly 40-70% of the fragmentation cost. The ROI horizon is 18-36 months. The mature run-rate is a 2-3x lower total-cost-of-ownership per employee for the same operational coverage.

The board's question is whether oyatie can execute. The market-fit question — is there demand? — is, after Wave-3-G, settled.

### §10.6 Why now — the market-timing window

Three structural shifts in the enterprise-SaaS market converge in 2025-2027 and create the timing window oyatie is built to take advantage of:

1. **The SaaS-portfolio backlash is finally CFO-visible.** For 15 years, the SaaS-portfolio cost was hidden inside IT-budget line items. CFOs treated it as fixed. In 2024-2025, Gartner's "SaaS sprawl" research started making the cost visible at the board level. Enterprise CFOs are now explicitly chartered to rationalize the portfolio. The buyer is ready.
2. **AI changes the calculus.** Every enterprise is being asked by its board "what is your AI strategy?" The fragmented-portfolio answer ("we have 12 AIs, one per tool") is increasingly recognized as a non-answer. The unified-substrate answer ("our AI sees the whole platform's data") is what boards now expect to hear. The narrative shift makes the unified-ecosystem thesis 10x easier to sell than it would have been in 2020.
3. **Identity is being recentralized around passkeys.** Apple, Google, Microsoft, and the FIDO Alliance have driven passkeys from niche to default. The 2025-2026 enterprise is rebuilding its identity model around passkeys anyway. Oyatie's passkey-bound continuity-of-identity primitive (ADR-0299, ADR-0311) lands in a market that is actively rebuilding to receive it.

A unified-ecosystem play in 2015 would have been too early. A unified-ecosystem play in 2030 would be too late (the AI-substrate moat would already be locked in by an incumbent). 2026-2028 is the window.

### §10.7 The investor case in three numbers

For VC partners and board directors evaluating oyatie purely on the numbers:

- **TAM**: $200B+ global enterprise SaaS (Gartner-anchored) + $100B+ consumer-productivity adjacent (anchored to combined market caps of WhatsApp, Notion, Stripe-Connect, TikTok categories) + $50B+ migration services (anchored to SAP/Workday/Salesforce implementation-services market). Aggregate TAM: $350B+. Conservatively, oyatie targets a low-single-digit-percent of this market by Y5 — which is a $10-15B revenue bar.
- **Unit economics at scale**: A mature 5,000-person enterprise tenant pays roughly $25M-$75M per year in oyatie license (assumed at 40-60% of the fragmentation cost it replaces). A typical Y5 customer base of 200-500 enterprise tenants (mid-market and up) at this rate produces $5B-$15B in annual recurring revenue. Consumer + SMB revenue is incremental.
- **Capital efficiency**: Oyatie's Foundry pipeline (the autonomous-execution substrate for code authoring) targets a 3-5x improvement in code-per-engineer-week over traditional SaaS development. If achieved, the same $300M of engineering-team capital produces 3-5x the platform coverage of a comparable SaaS vendor. This is the structural-cost advantage that lets a unified ecosystem outpace per-vertical incumbents.

### §10.8 The defensibility analysis

Investors will rightly ask: "What stops Microsoft or Salesforce or SAP from building this?"

Three structural reasons each of those vendors will struggle:

- **Substrate-rebuild cost is prohibitive.** Each incumbent has 10-25 years of legacy substrate (Salesforce Lightning Platform, Microsoft Graph, SAP Business Technology Platform). Rebuilding the substrate to be truly unified — passkey-bound identity, Cedar-class policy, ontology-class data model, audit-chain-class evidence — means breaking every customer's existing integration. No public incumbent's CEO will absorb a year of broken-customer pain to do this. The incumbents are locked into their per-product fragmentation by their own customer base.
- **AppExchange-class ecosystems are anti-unified.** Salesforce's AppExchange, ServiceNow's Store, Microsoft's marketplace each exist because the vendor's platform is fragmented enough that third-party vendors fill the gaps. Unifying the platform deflates the AppExchange economy. Salesforce will not deliberately deflate AppExchange. Oyatie's plugin-app-store is built for unified-platform economics from day one.
- **Incumbent CIO incentives.** A CIO of a Salesforce or Microsoft account does not want their vendor to converge on a unified-ecosystem. Converging means the CIO's per-vendor team becomes redundant; their per-tool integration debt becomes worthless; their per-vendor procurement skill becomes deprecated. The political economy of the incumbent's largest customers is anti-unification.

The defensibility analysis says: oyatie's window is open because the incumbents structurally cannot follow. The risk is execution + GTM, not competitive response.

### §10.9 The risk-mitigation playbook

Three risks dominate the investor view:

- **Execution.** Mitigation: the Foundry autonomous-execution pipeline (proven through Wave-3-G's autonomous documentation generation at ~10x the velocity of human authoring). The same primitives that let Wave-3-G produce 9,248-line coverage matrices in hours rather than months are designed to scale code authoring at the same velocity. The execution bet is on the AI-substrate primitives, not on raw human capacity.
- **GTM.** Mitigation: (a) the 5 hero consumer-and-productivity products produce a consumer demand pull that incumbent enterprise vendors cannot match; (b) the 8 migration-from-incumbent playbooks (§9.6) give the sales motion a contractual entry point with measurable savings; (c) the 24 vertical packs let GTM enter verticals with anchor tenants rather than horizontal sales motion.
- **Per-vertical market-entry.** Mitigation: each new vertical pack ships with an anchor tenant identified before the pack lands. No vertical pack ships without a Y1 customer commitment in that vertical. The vertical-pack count is bounded by demonstrated demand, not aspirational coverage.

---

## §11. The competitive landscape

For each major vendor or vendor-category, the head-to-head positioning is as follows.

### §11.1 SAP

- **What they do.** Global ERP standard. S/4HANA. 28 modules. Deep finance + supply-chain + manufacturing + industry verticals.
- **Where they win today.** Incumbent enterprise relationships. 50-year regulatory + audit-firm familiarity. Strong industry-vertical depth.
- **Where they lose.** Implementation cost (often $100M+ for large enterprises). Time-to-value (3-7 years). Per-module integration tax. Per-locale fork (S/4HANA-Public-Cloud vs S/4HANA-Private-Cloud vs SAP-ECC long-tail).
- **Oyatie's positioning.** Full S/4HANA module parity (ADR-0315), unified-substrate-integrated (no per-module tax), capability-tier projected (no platform lock-in), with build-ahead-of-certification regulatory readiness.

### §11.2 Salesforce

- **What they do.** Sales + service + marketing CRM + Commerce Cloud + Tableau + Slack + MuleSoft + Heroku + AppExchange.
- **Where they win today.** Sales-CRM mind-share. AppExchange ecosystem (8,000+ partners). Strong enterprise-sales motion.
- **Where they lose.** Per-cloud licensing complexity. MuleSoft integration tax. Lightning-vs-Classic UI fragmentation. Per-org customization debt.
- **Oyatie's positioning.** Sales-CRM is a capability tier over the substrate (ADR-0316). Service-Cloud equivalent is the contact-center microservice. Marketing-Cloud equivalent is marketing-automation. Slack is messenger + community. Plugin-App-Store replaces AppExchange's role with one unified marketplace.

### §11.3 Workday

- **What they do.** HCM + Financial Management + Adaptive Planning + Talent + Learning. Strong public-company HR + finance.
- **Where they win today.** Public-company-grade HR + finance compliance posture. Workday Studio + extensibility.
- **Where they lose.** Implementation cost. Per-product licensing. Limited extensibility outside HR-finance lane.
- **Oyatie's positioning.** HCM is a capability tier; financial-planning is its own microservice (ADR-0321); learning-management is its own microservice; performance-management is its own microservice. The unified substrate puts oyatie-HR adjacent to oyatie-CRM, oyatie-ITSM, oyatie-ERP without any integration tax.

### §11.4 ServiceNow

- **What they do.** Now Platform + ITSM + ITOM + HRSD + CSM + SecOps + GRC. The workflow-engine-as-product play.
- **Where they win today.** IT-service-management mind-share. Strong enterprise-IT relationships.
- **Where they lose.** Now Platform pricing escalates fast. Workflow-builder lock-in. Limited beyond IT and adjacent workflows.
- **Oyatie's positioning.** Workflow-engine is the universal substrate (not a product). ITSM is a capability tier on top of the workflow-engine. GRC is a capability tier on top of audit-chain + compliance + governance. HRSD is HR + workflow + community composition. CSM is contact-center + crm + workflow composition.

### §11.5 Atlassian

- **What they do.** Jira + Confluence + Bitbucket + Trello + Jira Service Management.
- **Where they win today.** Developer-tool mind-share. Cloud-migration momentum.
- **Where they lose.** Per-product licensing. Limited cross-product narrative. Confluence-Jira integration is still per-instance.
- **Oyatie's positioning.** Jira is tasks + workflow-studio + community composition. Confluence is notes + drive + sites composition. Bitbucket is the developer-sdk substrate. Service-Management is the new ITSM microservice. All on one unified substrate.

### §11.6 Microsoft 365

- **What they do.** Outlook + Teams + SharePoint + OneDrive + Office + Power Platform + Copilot.
- **Where they win today.** Office-suite incumbency. Active-Directory + Azure-AD identity lock-in. Massive enterprise reach.
- **Where they lose.** Best-of-breed-quality variance (Teams ≠ Slack; Outlook ≠ Gmail; SharePoint ≠ Notion). Identity tied to AD.
- **Oyatie's positioning.** Mail + messenger + meet + drive + sheets / slides / notes + workflow-studio composition gives oyatie functional parity with Microsoft 365. The differentiator is identity: passkey-bound continuity-of-identity (ADR-0299, ADR-0311) is fundamentally different from Azure-AD-bound work-identity. For users carrying their identity across personal + work + side-business, oyatie is structurally better.

### §11.7 Adobe

- **What they do.** Creative Cloud + Document Cloud + Experience Cloud + Marketo + Workfront.
- **Where they win today.** Creative-tool mind-share. PDF mind-share.
- **Where they lose.** Per-cloud licensing. Limited cross-product narrative. Experience-Cloud is a CRM + marketing-automation aspirant against Salesforce.
- **Oyatie's positioning.** Design-collaboration + drive + workflow-studio + marketing-automation + connect composition. Adobe's strength remains the creative tools themselves (Photoshop / Illustrator / Premiere); oyatie does not compete on creative tooling, but does displace Marketo + Workfront + Experience-Cloud.

### §11.8 HubSpot

- **What they do.** Marketing + Sales + Service + CMS + Operations Hub. SMB-friendly CRM.
- **Where they win today.** SMB CRM mind-share. Inbound-marketing methodology.
- **Where they lose.** Plateaus into mid-market. Limited enterprise depth.
- **Oyatie's positioning.** CRM + marketing-automation + contact-center + sites composition matches HubSpot's surface. The unified-ecosystem advantage means the SMB tenant can grow into oyatie's enterprise capabilities without a tool swap — which HubSpot's customers eventually face when they outgrow it.

### §11.9 Zendesk

- **What they do.** Support + Sell + Sunshine + Talk. Customer-support standard.
- **Where they win today.** Support-ticket mind-share. Fast time-to-value.
- **Where they lose.** Per-product licensing. Limited beyond support.
- **Oyatie's positioning.** Contact-center microservice + CRM composition matches Zendesk's surface. Unified-ecosystem advantage: the support agent sees the customer's marketplace history, their drive-shared documents, their meet-recorded calls, their workflow-managed tickets, all in one role projection.

### §11.10 Snowflake / Databricks

- **What they do.** Cloud data warehouse + data intelligence platform.
- **Where they win today.** Separation of storage and compute. Strong analytics ecosystem.
- **Where they lose.** Per-query cost surprises. Data-movement cost. Vendor-specific SQL dialects.
- **Oyatie's positioning.** Data-warehouse + data-pipeline + analytics microservices match Snowflake/Databricks's surface. The unified-ecosystem advantage: the data warehouse is populated by ontology, gated by Cedar, and queryable through the same workflow primitives that run the operational systems. No ETL tax.

### §11.11 Stripe Connect

- **What they do.** Platform-payments + multi-party settlement. The financial substrate for marketplace platforms.
- **Where they win today.** Developer-experience excellence. Global payment-rail coverage.
- **Where they lose.** Stripe-Connect-only narrative. Per-platform integration. Tax + compliance scope.
- **Oyatie's positioning.** Marketplace DealSet (ADR-0314) is the universal deal-settlement substrate — broader than Stripe Connect, encompassing services + subscriptions + capability grants + workforce contracts + M&A deals + data licenses. Payments microservice handles the money-movement; marketplace handles the deal envelope.

### §11.12 The aggregate competitive narrative

In every vendor head-to-head above, oyatie loses on feature-by-feature mind-share today (we are pre-launch; they are mature). Oyatie wins on:

- Unified substrate (no per-tool integration tax).
- Continuity-of-identity (no per-tool account creation).
- Training-cost amortization (no per-tool retraining).
- Compliance posture (no per-tool certification reconciliation).
- Exit-cost (no per-tool data extraction).
- AI usefulness (the unified data substrate is fundamentally more useful for AI than any single-tool data slice).

The aggregate competitive narrative is: each individual vendor is the best at their own narrow surface today; oyatie is the best at the surface-of-surfaces tomorrow. The bet is that "the surface-of-surfaces" is what the next decade's enterprises will actually want.

### §11.13 The consumer-side competitive landscape

The enterprise-vendor head-to-head is the visible competitive landscape. The consumer-side landscape is just as important for the long-term unified-ecosystem moat.

- **WhatsApp, iMessage, Signal, Telegram.** Messenger is one of oyatie's two highest-leverage consumer products (the other is marketplace). The competitive case rests on unified-ecosystem advantages — the same identity carries to mail, community, marketplace, calendar, meet, drive — which no consumer-messenger incumbent offers.
- **Google Workspace + Gmail + Drive + Calendar.** Google's hold on the consumer-productivity market is deep but increasingly resented (privacy concerns, ad-monetization tension, AI-data-use tension). Oyatie's unified-ecosystem privacy posture (per-tenant data, Cedar-gated access, no advertising substrate) is a clean differentiator.
- **Notion.** Notion is the rising consumer-and-team workspace. Oyatie's notes microservice composes with workflow-studio + drive + community + meet — a substantially deeper integration than Notion's third-party-tool ecosystem.
- **Reddit, Discord.** Community surfaces with strong network effects. Oyatie's community microservice composes with messenger + marketplace + recordings + plugin-app-store — letting communities run their own commerce + their own asynchronous video + their own bots/tools without the per-platform integration tax.
- **TikTok, Instagram Reels, YouTube Shorts.** Shorts + social microservices. The differentiator is creator-economy settlement (marketplace DealSet) and minor-protection guardrails (ADR-0292) — both stronger than any incumbent's current posture.
- **Stripe (consumer-side), Square, Shopify, PayPal.** Marketplace + payments. Marketplace's DealSet substrate is broader than Stripe's payment-rail focus and broader than Shopify's commerce focus. The same DealSet handles a $5 consumer purchase and a $500M M&A transition.

### §11.14 What the competitive landscape misses

Most competitive analyses focus on per-product feature parity. The unified-ecosystem case rests on a different competitive axis: portfolio coherence.

A CIO evaluating SaaS today asks: "Which CRM is best, which HR is best, which ERP is best?" After oyatie launches at scale, the same CIO will increasingly ask: "Which unified ecosystem is best?" The set of vendors who can answer that question — versus the set of vendors who can answer the per-product question — is dramatically smaller. That is the structural competitive shift the unified-ecosystem thesis bets on.

---

## §11.5. The data + AI substrate — why oyatie's AI is structurally more useful

### §11.5.1 The fragmented AI problem

Every modern SaaS vendor is shipping AI assistants. The result, in a fragmented portfolio:

- The CRM AI sees CRM data only.
- The HR AI sees HR data only.
- The collaboration AI sees collaboration data only.
- The code-review AI sees code only.
- The customer-support AI sees tickets only.

Cross-system AI ("what is happening to this customer across our entire portfolio") requires either iPaaS-mediated integration (slow, expensive, lossy) or a vendor-specific "data graph" (Microsoft Graph, Salesforce Genie, ServiceNow Now Assist) that re-centralizes data inside one vendor's walled garden.

The economic outcome: each AI subscription is worth a fraction of what it could be, because each AI is bounded by its tool's data slice.

### §11.5.2 The unified AI substrate

Oyatie's intelligence substrate (ADR-0255 Intelligence Two-Layer Substrate) sees the full ontology graph. A single AI assistant has access to:

- Communication context (messenger, mail, meet recordings, community discussions).
- Calendar + scheduling context.
- Document + drive context (with policy-mediated access).
- CRM context (leads, opportunities, accounts).
- HR context (with strict Cedar-mediated personal-privacy gates).
- ERP context (orders, invoices, inventory, financial planning).
- Marketplace context (deals, settlements, entitlements).
- Workflow context (active processes, pending approvals).
- Audit-chain context (evidence, history).

Because the substrate is one platform, the AI does not need a separate integration to each context. The AI just queries the ontology.

### §11.5.3 What that unlocks

- **Per-user AI assistant.** A consumer user's AI assistant sees their messenger, mail, calendar, marketplace, and family-tenant context all at once. Asking "remind me to call Mom on Saturday" and "did Dad's prescription get refilled" and "what was the last invoice from the contractor" are all the same query class.
- **Per-team AI assistant.** A team's AI assistant sees the team's calendar, the team's project plans, the team's CRM accounts, the team's documents, the team's chat threads. Asking "what's blocking us this sprint" is a real query, not a feature request.
- **Per-org AI assistant.** An organization's AI assistant sees aggregate ontology projections (subject to Cedar). The CFO's AI can answer "what is our cash position by region accounting for FX exposure" by joining finops + treasury + global-trade + finance contexts.
- **Cross-tenant AI assistant (with policy mediation).** A marketplace AI assistant can recommend a deal across tenants while respecting each tenant's data-sharing policy — something no fragmented portfolio can do without bilateral integration contracts.

The AI moat scales with the ecosystem moat. Every new microservice oyatie ships makes every existing AI assistant more useful. The fragmented incumbents' AIs do not get this compounding.

### §11.5.4 The AI-governance posture

ADR-0308 (ML Model Lifecycle — EU AI Act + NIST AI RMF + ISO/IEC 42001) and ADR-0309 (Detection Fairness Audit) give oyatie a regulatory-grade AI-governance posture from day one. Every model has a model card, a dataset card, a fairness-audit record, a drift-detection signal, and a rollback path. The build-ahead-of-certification doctrine (ADR-0250) means EU AI Act high-risk compliance is the architecture, not a future remediation.

This matters for B2B buyers. Enterprise procurement now requires AI-governance attestation. Oyatie ships with it. Incumbents are catching up.

---

## §11.6. The corporate-governance + risk posture

### §11.6.1 Why this matters for boards

Board directors evaluating a unified-ecosystem investment will ask: "How does the platform handle risk?" Six axes matter:

- **Security risk.** Substrate-wide identity + Cedar + audit-chain + cell-isolation + warrant-piercing-with-audit + supply-chain-integrity (sigstore + cosign + FIPS-HSM root signing per ADR-0250).
- **Privacy risk.** Per-tenant Cedar enforcement + per-pack overlays (GDPR / HIPAA / KR-PIPA / etc.) + dual-tenant boundary + per-jurisdiction-residency.
- **Operational risk.** Cellular architecture (ADR-0248) + shuffle-sharding + per-region failover + disaster-mode (ADR-0306) + 99.99%+ availability target.
- **AI risk.** ADR-0308 ML Model Lifecycle + ADR-0309 Fairness Audit + EU AI Act + NIST AI RMF + ISO/IEC 42001 compliance.
- **Reputational risk.** Abuse-defense (ADR-0297) + survivor-safety (ADR-0301) + minor-protection (ADR-0292) + emergency-services bypass (ADR-0298) + cognitive-impairment resilience (ADR-0303).
- **Litigation + regulatory risk.** Whistleblower + press-freedom protection (ADR-0300) + warrant-scoped piercing (ADR-0312) + cross-jurisdiction conflict resolution (ADR-0304) + audit-chain immutability for evidence preservation.

### §11.6.2 The substrate-as-product-of-the-board

A board director's signature on an oyatie-vs-fragmented-portfolio decision is structurally easier to defend. The risk posture is unified. There is one CISO conversation, not 110. There is one DPA, not 110. There is one breach-notification clock, not 110. The director's fiduciary duty is easier to discharge with a unified platform.

This is part of why the unified-ecosystem thesis is not just an architecture choice — it is a corporate-governance choice. Wave-3-G's ADR cluster (ADR-0297..0321) is structured so that the platform can defend itself in a board-level risk review.

---

## §12. Closing — the vision in five sentences

1. Oyatie ends the SaaS fragmentation tax — one identity, one policy, one workflow, one ontology, one audit, one marketplace, one UX, one training model, one compliance posture, one plugin extensibility — across every role a human carries through a career and across every vertical an enterprise operates in.
2. The platform covers 18 hero consumer-and-productivity surfaces day-one, 28 SAP ERP modules, 165 B2B SaaS vendor surfaces (via capability tiers + 13 new microservices), 24 vertical packs, and 25+ compliance pack overlays — 69 microservices total, one unified substrate.
3. The persona graph (127 personas) and journey catalog (150 journeys, 1,092+ artifacts) prove the unified-ecosystem thesis is operationally real, not just an architecture diagram, and give the GTM team a pre-loaded sales-enablement, customer-success-enablement, and demand-generation engine.
4. The 30-ADR doctrine cluster (ADR-0297..0321) — abuse-defence, life-safety, account-recovery, whistleblower-protection, survivor-safety, deceased-user inheritance, cognitive-impairment resilience, jurisdictional conflict, AI-agent delegation, disaster mode, detection substrate, ML lifecycle, fairness audit, investigation, dual-tenant boundary, court-warrant piercing, conglomerate hierarchy, marketplace settlement, ERP parity, capability-tier projection, role-based projection, collar-color universality, information-barrier, transient identity, B2B leader coverage — establishes what the platform does at the edges, where every previous "unified suite" failed.
5. The board case is execution-and-GTM, not market-fit; the architecture-completeness is settled by Wave-3-G, and the next 12 months are about getting 5 hero products to 10M+ users, getting 3 vertical packs to lighthouse-account status, and authoring 8 migration-from-incumbent journeys — after which the unified-ecosystem flywheel runs against every fragmentation incumbent on every dimension that the buying market is starting to weight as more valuable than feature-by-feature parity.

---

## §13. Cross-references

### §13.1 Primary architecture documents (Wave-3-G)

- `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` — the manifesto. 7,369 lines. Read for the doctrine clauses.
- `docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md` — the 6-persona narrative. 7,611 lines. Read for the operating texture.
- `docs/architecture/training-cost-doctrine-2026-05-21.md` — the 30-year career-arc claim. 2,325 lines. Read for the training-cost economic case.
- `docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md` — full vendor-by-vendor + module-by-module coverage. 9,248 lines. Read for the scope detail.
- `docs/architecture/keystone-bundle-2026-05-20-synthesis.md` — the 21-facet multispectrum-review synthesis + the 15-item promotion gate set. Read for the architecture-decision provenance.
- `docs/personas/MASTER-ROSTER-2026-05-21.md` — the 127-persona master roster. 1,019 lines. Read for the persona axes + the anchor-roster.
- `docs/user-journeys/CATALOG-j126-j150-ecosystem.md` — the ecosystem-journey catalog (j126-j150). Read for the cross-tenant + cross-product narrative.

### §13.2 The 5 most load-bearing ADRs

- **ADR-0244** — Tenant as the universal scoping primitive. (Every row, every audit, every cost carries tenant context.) The keystone of the entire data model.
- **ADR-0314** — Marketplace as universal deal-settlement substrate. (Every commercial exchange is a DealSet.) The keystone of the commercial model.
- **ADR-0316** — Capability-tier over product fragmentation. (Product labels are tenant activations, not microservice boundaries.) The keystone of the substrate-vs-product layering.
- **ADR-0317** — Role-based projection + unified UX shell. (The same human sees different role projections of the same substrate.) The keystone of the user-experience model.
- **ADR-0321** — B2B SaaS industry-leader coverage. (165 vendor benchmark dossiers + 13 new microservice anchors.) The keystone of the B2B competitive coverage.

### §13.3 The 30-ADR doctrine cluster

- ADR-0297 — Abuse-Defence Baseline.
- ADR-0298 — Emergency-Services Bypass.
- ADR-0299 — Account-Recovery Resilience.
- ADR-0300 — Whistleblower + Press Freedom.
- ADR-0301 — Survivor-Safety Mode.
- ADR-0302 — Deceased-User Inheritance.
- ADR-0303 — Cognitive-Impairment Decision Resilience.
- ADR-0304 — Cross-Jurisdiction Conflict Resolution.
- ADR-0305 — Delegated-Agent Authority Chain.
- ADR-0306 — Disaster-Mode Cell Resilience.
- ADR-0307 — Detection Substrate (streaming + batch).
- ADR-0308 — ML Model Lifecycle (EU AI Act).
- ADR-0309 — Detection Fairness Audit.
- ADR-0310 — Investigation Case-Management.
- ADR-0311 — Dual-Tenant Identity (personal-vs-work).
- ADR-0312 — Court-Warrant Scoped Piercing.
- ADR-0313 — Conglomerate Tenant Hierarchy.
- ADR-0314 — Marketplace as Universal Deal-Settlement.
- ADR-0315 — ERP Coverage Doctrine (SAP parity).
- ADR-0316 — Capability-Tier Over Product Fragmentation.
- ADR-0317 — Role-Based Projection + Unified UX Shell.
- ADR-0318 — Collar-Color and Workspace Universality.
- ADR-0319 — Front / Middle / Back Office Information-Barrier.
- ADR-0320 — Apprentice / Intern / Resident / Fellow Transient Identity.
- ADR-0321 — B2B SaaS Industry-Leader Coverage.
- (Plus the keystone bundle's earlier ADR-0242..0258 cluster covering tenant primitive, Cedar engine, substrate-vs-product layering, MLS messenger, self-modification doctrine, Amazon-shape cellular architecture, compliance-pack primitive, build-ahead-of-certification, provider-credential BYOK doctrine (ADR-0255 §D-4), encryption-key BYOK doctrine (ADR-0251 §D-10), HTTP/3+QUIC default, multi-category marketplace, HLC + TrueTime tier, Kubernetes-everywhere with Cloud Hypervisor, and intelligence two-layer substrate.)

### §13.4 External market references

- Gartner SaaS sprawl + management research (sizing references in §2.1, §2.2): public Gartner library, document IDs cited in `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` Section 14.
- Forrester tech-sprawl + SaaS-integration research (sizing references in §2.1, §2.2): public Forrester reports cited in `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` Section 14.
- Vendor public product pages (§11): SAP S/4HANA, Salesforce Platform, Workday HCM + Financial Management, ServiceNow Now Platform, Atlassian Cloud, Microsoft 365, Adobe Creative + Experience Cloud, HubSpot Hubs, Zendesk Suite, Snowflake Data Cloud, Databricks Data Intelligence Platform, Stripe Connect.

### §13.5 Sizing-assumption disclaimer

Internal sizing assumptions used in this briefing:

- $50K-$500K per-employee per-year fragmentation cost.
- $1,500 per-employee per-tool per-year training pressure.
- 110-plus SaaS apps per enterprise (Gartner anchored).
- 30 percent IT budget on integration (Forrester anchored).
- 6-9 months training horizon per major tool change (analyst-anchored).
- $200B+ global enterprise-SaaS TAM (Gartner anchored).
- 40-70 percent steady-state savings vs fragmentation portfolio.
- 18-36 months ROI horizon for migrating enterprise.

Per `docs/architecture/unified-ecosystem-thesis-2026-05-21.md` Section 14 — every customer-facing publication of these numbers must pass legal + procurement validation of exact analyst-report wording rights and date-specific accuracy.

---

## §14. One-page executive summary (for board pre-read)

> **What.** Oyatie is one unified software ecosystem — one identity, one policy engine, one workflow engine, one ontology, one audit chain, one marketplace, one UX shell, one training model, one compliance posture, one plugin extensibility — that absorbs the role of the 110-plus separate SaaS applications the average enterprise runs today.
>
> **Why.** SaaS fragmentation costs the average enterprise $50K-$500K per employee per year in license + integration + training + compliance + lost productivity. The fragmentation cost has reached the point where the market is ready for an alternative. Oyatie is the alternative.
>
> **Scope.** 18 hero consumer-and-productivity surfaces day-one. 28 SAP S/4HANA modules covered. 165 B2B SaaS vendor surfaces covered through capability tiers + 13 new microservices. 24 vertical packs (15 existing + 9 new). 25+ compliance pack overlays. 69 microservices, one unified substrate.
>
> **Personas.** 127 personas across 6 collar-colors, 7 workspaces, 6 skill-tiers, 20+ locales, 6 device profiles, 32+ audience-types. 6 anchor personas (Yejin / Marcus / Aiyana / Tomás / Hiroshi / Anya) carry the day-in-the-life narrative.
>
> **Journeys.** 150 canonical user journeys (j01-j150), 1,092+ artifacts. j01-j20 life-safety, j21-j50 hero-product, j51-j75 cross-product, j76-j100 locale-pack, j101-j125 inter-tenant, j126-j147 Diana/Priya/Sam/Chris ecosystem, j148-j150 creative ecosystem.
>
> **Compliance.** Build-ahead-of-certification (ADR-0250). 25+ compliance pack overlays. Cell-tier certification (Tier 0/1/2/3). Dual-tenant identity boundary (ADR-0311). Per-jurisdiction overlays. Court-warrant scoped piercing (ADR-0312). EU-sovereign + CN-sovereign + KR-sovereign + FedRAMP-High cells.
>
> **Doctrine.** 30-ADR cluster (ADR-0297..0321) covers abuse, life-safety, recovery, press-freedom, survivor-safety, inheritance, cognitive-decline, jurisdiction, AI-agent delegation, disaster, detection, ML lifecycle, fairness, investigation, dual-tenant, warrant-piercing, conglomerate, marketplace, ERP, capability-tier, role-projection, collar-color, information-barrier, transient-identity, B2B leader coverage.
>
> **Roadmap.** Wave 3-H content pass (in-flight). Wave 3-I capability-tier registry. Wave 3-J code authoring. Wave 3-K launch sequencing (substrate Q1, hero Q2-Q3, consumer-social + first enterprise Q4, ERP Y2H1, B2B leader Y2H2). Wave 3-L migration journeys.
>
> **Next 12 months.** Five hero products at 10M+ users each. Three vertical packs at lighthouse-account status (healthcare + financial-services + public-sector). Eight migration-from-incumbent journey playbooks.
>
> **TAM.** Global enterprise-SaaS market ~$200B (Gartner-anchored) + consumer-productivity adjacent + migration-services + developer-ecosystem. Order-of-magnitude TAM expansion over the pre-Wave-3-G narrative.
>
> **Moat.** Unified-ecosystem flywheel. Day-zero adoption advantage. Cross-career training amortization. Compounding network effects. Substrate cost advantage. AI usefulness compounding with substrate completeness.
>
> **Risk.** Execution (69 microservices, 24 months). GTM (incumbent CIO relationships). Per-vertical market-entry (9 new verticals).
>
> **Economic case.** 5,000-person enterprise carries $250M-$1B annual fragmentation cost; migration cost is 5-15% of one year's fragmentation; steady-state savings are 40-70% net of license; ROI horizon is 18-36 months; mature run-rate is 2-3x lower TCO per employee.
>
> **Bottom line.** Architecture-completeness is settled by Wave-3-G. Market-fit question is settled by the persona + journey + competitive coverage. The remaining question is execution, and that is what the next 12 months are about.

---

## §15. Glossary (for non-technical readers)

- **ADR.** Architecture Decision Record. The canonical document that records why a specific architectural choice was made and what it commits the platform to. Oyatie has 320+ ADRs; this briefing references the most load-bearing 30.
- **Capability tier.** A tenant-activated bundle of permissions + data projections + workflow templates + UX vocabulary + compliance overlays + observability metadata that surfaces in the product as a familiar label (CRM, ITSM, HRIS, etc.).
- **Cedar.** The policy-engine language oyatie uses for every authorization decision. AWS-developed, open-source. The "one policy engine" of the unified-ecosystem thesis.
- **Cell.** A unit of deployment in the Amazon-shape cellular architecture (ADR-0248). Each cell carries a certification tier (Tier 0/1/2/3).
- **DealSet.** The canonical envelope (ADR-0314) for every commercial exchange — offer, acceptance, obligation, entitlement, settlement, dispute, revocation, amendment, renewal.
- **DRMP.** Detection / Response / Mitigation / Prevention — the abuse-and-safety layer (ADR-0307..0310).
- **E2EE.** End-to-end encrypted. Oyatie uses MLS (RFC 9420) as the canonical E2EE protocol for messenger.
- **Foundry.** Oyatie's autonomous-execution pipeline that orchestrates the agent + CI + reviewer-agent + merge-queue substrate for code-authoring at scale.
- **MLS.** Messaging Layer Security (RFC 9420). The IETF-standard E2EE protocol for group messaging.
- **Microservice.** A flat, single-concern, independently-deployable service. Oyatie has 69. No "suite" services per ADR-0132.
- **Ontology.** Oyatie's universal object graph (the Palantir-equivalent). One canonical schema for Customer, Product, Order, Account, Patient, Employee, Asset, etc., with per-role + per-jurisdiction projections.
- **Pack.** A jurisdiction-specific or industry-specific overlay (HIPAA, GDPR, KR-PIPA, etc.) that adjusts compliance + UX + workflow behavior for a tenant.
- **Passkey.** WebAuthn-standard cryptographic credential bound to a device. Oyatie's primary identity primitive (no passwords).
- **Persona.** A coordinate in (identity × tenant × role × workspace × locale × device × skill-tier) space. The same human projects as different personas in different coordinates.
- **Substrate.** The unified platform layer (identity + policy + workflow + ontology + audit-chain + marketplace + UX shell) that every product surface composes against.
- **Tenant.** The universal scoping primitive (ADR-0244). Every data row, every audit event, every cost dimension carries a tenant context. A human can hold N tenant memberships.
- **Workspace.** The category of where a persona's primary activity happens (front-office, middle-office, back-office, field, clinical, executive, production) — per ADR-0319.

---

## §16. Final note on confidence + scope

This briefing is a synthesis, not original architecture. Every claim is sourced from the underlying corpus. The unified-ecosystem thesis is the manifesto (`docs/architecture/unified-ecosystem-thesis-2026-05-21.md`); the day-in-the-life narrative is the operational texture (`docs/architecture/day-in-the-life-coherent-ecosystem-2026-05-21.md`); the training-cost economics are the moat case (`docs/architecture/training-cost-doctrine-2026-05-21.md`); the coverage matrix is the scope evidence (`docs/architecture/enterprise-software-coverage-matrix-2026-05-21.md`); the keystone-bundle synthesis is the architecture-decision provenance (`docs/architecture/keystone-bundle-2026-05-20-synthesis.md`); the persona roster is the audience evidence (`docs/personas/MASTER-ROSTER-2026-05-21.md`); the ADR cluster is the operational-edge envelope (the 30 ADRs in §8).

For board members or investors who want to drill deeper than this briefing:

- For the strategic thesis: read the unified-ecosystem thesis manifesto.
- For the operational texture: read the day-in-the-life narrative.
- For the economic case: read the training-cost doctrine.
- For the scope: skim the enterprise-software coverage matrix.
- For the architecture-decision provenance: read the keystone-bundle synthesis.
- For the GTM-ready audience evidence: read the persona master roster.
- For specific operational-edge questions: read the relevant ADR from the 30-ADR cluster.

For sales-leaders, marketing-leads, and GTM teams who want to convert this briefing into customer-facing material:

- Each anchor persona (Yejin / Marcus / Aiyana / Tomás / Hiroshi / Anya) is a ready-made day-in-the-life sales story.
- Each anchor tenant (KrampusCorp / AcmeRawMaterials / GlobalLogistics / Hospital / Restaurant / Newsroom) is a ready-made enterprise-account story.
- Each journey (j01-j150) is a ready-made demo script.
- Each vendor head-to-head (§11) is a ready-made competitive-positioning one-pager.
- Each migration journey (§9.6) is a ready-made customer-engagement playbook.

The architecture corpus is now coherent enough that GTM, sales-engineering, and marketing can build directly against it. That is the meaning of "Wave-3-G is complete."

---

## §17. Frequently asked board + investor questions

### §17.1 "Is the SaaS market really ready for a unified ecosystem?"

The market signals say yes. Three converging indicators:

- **CFO-level SaaS rationalization mandates** are now standard at most enterprises. Procurement teams have explicit charters to consolidate vendor portfolios. The buyer is no longer "the department head buying a tool" — it is increasingly "the CFO + CIO together rationalizing the portfolio."
- **The integration market is consolidating.** MuleSoft (Salesforce), Boomi (Vista), and the iPaaS category broadly are being absorbed into larger platforms. The market is voting that integration-as-a-separate-purchase is fragile.
- **AI is forcing the unified-data conversation.** The board-level "what is your AI strategy" question is now structurally answered by "we have unified data" or "we have fragmented data." Unified-ecosystem buyers win that conversation.

### §17.2 "Why hasn't a unified-ecosystem play succeeded before?"

It has been attempted multiple times — and the failure modes are instructive:

- **Microsoft (1990s-2000s)** tried to unify office productivity + enterprise. The attempt mostly succeeded for office productivity but stalled at the enterprise tier (Dynamics, SharePoint, Power Platform never reached SAP/Salesforce parity). Reason: legacy Windows + Office substrate could not stretch to enterprise-grade data + workflow + audit.
- **SAP (2000s-2010s)** tried to unify ERP + CRM + HCM + cloud + analytics. The attempt mostly worked for ERP but stalled at CRM (lost to Salesforce) and HCM (lost to Workday). Reason: SAP's substrate was on-premise-shaped and could not pivot to cloud-native shape fast enough.
- **Salesforce (2010s-2020s)** tried to unify sales + service + marketing + commerce + analytics + collaboration (Slack). The attempt mostly worked for sales + service but stalled at ERP (Salesforce has no ERP), at collaboration (Slack is independent of Sales Cloud), and at AI (Einstein remained per-cloud).
- **Microsoft (2010s-2020s) v2** tried again with Office 365 + Dynamics + Power Platform + Azure + Microsoft Graph. The attempt is partially succeeding for productivity + collaboration but stalled at ERP (Dynamics remains second-tier), at CRM (lost to Salesforce), and at enterprise-grade audit posture.

The pattern: every prior unified-ecosystem attempt was built on a substrate that did not start as unified. Oyatie is built unified from day one. That is the structural advantage.

### §17.3 "How does oyatie make money?"

Three revenue streams, in order of magnitude:

- **Enterprise license.** Per-tenant subscription, sized by user-count + capability-tier set + compliance-pack overlay + cell-tier requirement. This is the dominant revenue stream at maturity.
- **Marketplace settlement.** Take-rate on marketplace DealSet transactions — consumer purchases, B2B procurement, plugin entitlements, workforce contracts. Modest take-rate (1-3%) but high volume at scale.
- **Migration services.** Customer-success engagements for migration-from-incumbent. Multi-month, multi-million-dollar contracts at enterprise scale. Bridge revenue during the first 18-36 months, declining as the migration playbooks become self-service.

Long-term, oyatie's revenue model resembles a hybrid of Salesforce (enterprise license), Stripe (marketplace settlement), and SAP-Implementation-Services (migration). The blend gives the company multiple revenue diversifications across customer-size segments and product-maturity stages.

### §17.4 "What is the consumer-side monetization model?"

Three principles govern the consumer-side approach:

- **No advertising substrate.** Oyatie's user-data is not monetized via advertising. This is a substrate-level commitment, not a per-product policy. The Cedar substrate does not have an "advertising" permit class for cross-tenant data sale.
- **Freemium with capability-tier upsell.** Consumer hero products (messenger, mail, community, marketplace consumer-side, drive, calendar, meet, notes) are free at a basic capability tier. Power-user features (custom domains, higher storage, advanced workflows, larger meetings, AI-assist depth) are paid capability tiers.
- **Marketplace settlement take-rate.** Consumer purchases through marketplace generate take-rate revenue, the same way Stripe-or Shopify do.

The consumer-side revenue is not the largest revenue line. It is the GTM unlock that makes the enterprise sale easier. A workforce that already uses oyatie at home is a workforce pre-disposed to advocate for oyatie at work.

### §17.5 "What is oyatie's privacy posture vs Google + Meta?"

Five structural differences:

- **No advertising substrate.** Already covered. Oyatie does not monetize user data through targeted advertising.
- **Cedar-enforced data-access boundaries.** Every data access (including by oyatie itself) is Cedar-gated and audit-logged. Oyatie's own employees cannot access tenant data without policy-authorized + audit-logged reason.
- **Per-tenant data residency.** Where a tenant's compliance pack requires it, data physically stays in a specific jurisdiction's cells.
- **Court-warrant scoped piercing.** Law-enforcement access is Cedar-mediated, scope-bounded, time-limited, and audit-logged to the tenant's privacy officer. No master key. No silent compliance.
- **encryption-key BYOK + provider-credential BYOK.** Tenants can bring their own KMS root key (ADR-0251 §D-10) for at-rest encryption and bring their own LLM-provider credentials (ADR-0255 §D-4) for AI workloads. The substrate operates without seeing the keys in regulated mode.

### §17.6 "How does oyatie handle data portability + exit?"

Three commitments:

- **Open object model.** Oyatie's ontology object types are documented as open schemas. A tenant can export their full ontology graph in canonical JSON-LD + Parquet + SQL formats.
- **Workflow + Cedar exports.** Workflow templates and Cedar policy fragments are exportable as portable artifacts. A tenant migrating off oyatie can take their operational logic with them.
- **Audit-chain exports.** The full audit-chain stream is exportable as portable evidence (with cryptographic verification of integrity).

The substrate's exit story is structurally better than the fragmented portfolio's. A fragmented portfolio's exit involves 110 separate vendor exits, each on a separate timeline, each with separate data-extraction tooling. Oyatie's exit involves one substrate export.

### §17.7 "How does the migration playbook handle the political-economy resistance inside the customer org?"

The honest answer: the migration is most likely to succeed when sponsored by the CFO + CIO together, with explicit board-level mandate. Departmental sponsorship alone is usually insufficient — the per-tool incumbents have decade-long department-level relationships that resist consolidation.

The migration playbook design:

- **Phase 0 diagnostic identifies the political stakeholders.** Each per-tool incumbent has an internal champion (the user who selected the tool, the admin who maintains it, the procurement contact who renews it). The diagnostic phase names these stakeholders explicitly.
- **The CFO-aligned ROI model becomes the political tool.** The fragmentation-cost calculation (§2.2) per-employee gives the CFO a board-ready number. Internal opposition rarely survives a clear board-mandated savings number.
- **Phased migration preserves face for incumbent-vendor champions.** A department head who selected Salesforce 8 years ago does not want their decision "reversed." The migration playbook is framed as "consolidation, not reversal" — the department head's CRM continues to operate (as a capability tier in oyatie), but the surrounding tooling consolidates.

### §17.8 "What is the AI substrate's competitive position vs OpenAI / Anthropic / Google?"

Oyatie does not aim to build a foundation model. Oyatie is a unified application substrate. The AI substrate composes with foundation models from OpenAI, Anthropic, Google, Mistral, Cohere, plus open-source models (Llama family, Qwen family, etc.).

The provider-credential BYOK doctrine (ADR-0255 §D-4) lets each tenant choose:

- **Platform-default mode.** Oyatie provides default provider credentials (Anthropic / OpenAI / Google / Bedrock). B2C personal-use defaults to this.
- **provider-credential BYOK mode.** Tenant brings their own provider subscription or API key.
- **provider-credential BYOK-required mode.** Pack-required for regulated tenants (HIPAA, PCI, FedRAMP, IL5-6, KR-FSS, EU-AI-Act high-risk). Substrate owns zero provider credentials in this mode.

The competitive moat is not "we own the foundation model." The competitive moat is "we have the unified data substrate that makes the foundation model 10x more useful."

### §17.9 "What is the timeline before oyatie is at enterprise-ready feature parity with Salesforce + Workday + SAP?"

The launch sequencing (§9.5) is honest:

- **Y1 (Q1-Q4):** Substrate + hero consumer surfaces + first enterprise wave (CRM-equivalent, finops, payments, treasury). Enterprise sales begin in Q3-Q4 with anchor accounts.
- **Y2 H1:** ERP wave (SAP-parity coverage). Enterprise sales reach mid-market scale.
- **Y2 H2:** B2B leader wave (Workday + Salesforce + ServiceNow + Atlassian + Microsoft 365 parity-grade coverage). Enterprise sales reach upper-mid-market scale.
- **Y3-Y4:** Industry-vertical expansion (Wave 3-M) + locale expansion (Wave 3-N). Enterprise sales reach Fortune-500 scale.

The honest answer to "when does oyatie hit Fortune-500 parity" is Y3-Y4. The honest answer to "when does oyatie start displacing incumbents at mid-market scale" is Y2 H1.

### §17.10 "What goes wrong if the execution slips?"

The most likely slip modes:

- **Substrate stabilization takes longer than Q1.** Identity, Cedar, ontology, workflow-engine, and audit-chain at production-stable scale is the hardest part of Y1. A 1-quarter slip here pushes everything else back by a quarter.
- **Hero-product consumer-adoption flywheel takes longer than expected.** Messenger + mail + community + marketplace reaching 10M+ users each is the demand-generation engine. If consumer adoption is slower than projected, enterprise sales lose social proof.
- **Anchor enterprise account migration takes longer than expected.** The lighthouse-account migrations in Y1-Y2 are GTM-critical. A delayed lighthouse account delays the publishability of the first migration case studies.
- **AI substrate falls behind incumbent feature pace.** If oyatie's AI substrate is not visibly better than Salesforce Einstein + Microsoft Copilot + ServiceNow Now Assist within 12-18 months, the "structural AI advantage" narrative weakens.

Mitigation across all of these: the Foundry autonomous-execution pipeline lets the platform absorb velocity slips by re-prioritizing without re-staffing. The architecture-completeness work of Wave-3-G means execution decisions are reversible without re-architecting.

---

## §18. The vertical-pack deep-dive — what each new pack unlocks

The 9 new vertical packs in Wave-3-G (§4.4) deserve a brief unpacking because each unlocks a distinct market segment.

### §18.1 Insurance

- **Market size:** Global insurance industry premiums ~$7 trillion annually. Insurance-specific software market ~$50B annually.
- **Incumbents displaced:** Guidewire (P&C policy + claims), Duck Creek (P&C), Sapiens (life + annuity), Majesco, FINEOS.
- **Key capabilities:** Policy lifecycle, underwriting workflows, claims workflows, reinsurance settlement (DealSet), regulatory reporting (Solvency II + state-by-state + IRDAI + FSS), actuarial-substrate composition.
- **Anchor segment:** Mid-market P&C insurers, regional life insurers, insurtech-startups.

### §18.2 Automotive

- **Market size:** Global automotive industry ~$3 trillion. Automotive-specific software ~$30B.
- **Incumbents displaced:** CDK Global (dealer management), Reynolds & Reynolds (dealer), Cox Automotive (industry-wide), SAP IS-Auto (OEM).
- **Key capabilities:** Dealer-network management, vehicle inventory, service-and-parts, recall management workflows, OEM-supplier coordination (cross-tenant supply-chain), warranty management.
- **Anchor segment:** Regional dealer networks, EV startups, automotive parts suppliers.

### §18.3 Oil & gas

- **Market size:** Global oil-and-gas industry ~$3 trillion. Oil-and-gas-specific software ~$10B.
- **Incumbents displaced:** Petrel (upstream geology), Aveva (production + asset), SAP IS-Oil, Quorum (joint-venture accounting), Enverus (analytics).
- **Key capabilities:** Upstream production tracking, joint-venture accounting (multi-tenant DealSet), well-and-asset tracking, HSE compliance (EHS), regulatory reporting.
- **Anchor segment:** Independent upstream operators, joint-venture partners, oilfield-services companies.

### §18.4 Pharma

- **Market size:** Global pharmaceutical industry ~$1.5 trillion. Pharma-specific software ~$25B.
- **Incumbents displaced:** Veeva (CRM + content + quality), MasterControl (QMS), Oracle Argus (pharmacovigilance), Medidata (clinical trials).
- **Key capabilities:** Clinical-trial management, GxP-compliant document control, regulatory submission (FDA / EMA / PMDA), pharmacovigilance, manufacturing-quality (QMS).
- **Anchor segment:** Mid-cap pharma, contract-research-organizations (CROs), biotech startups.

### §18.5 Legal services

- **Market size:** Global legal-services industry ~$1 trillion. Legal-specific software ~$30B.
- **Incumbents displaced:** Thomson Reuters Elite (matter + billing), Aderant, Clio (mid-market), iManage (document management), Relativity (e-discovery).
- **Key capabilities:** Matter management, conflicts-checking (Cedar-enforced + ADR-0319 information-barrier), billable-time tracking, e-discovery, retention policies.
- **Anchor segment:** Mid-market law firms, in-house counsel teams, legal-tech startups.

### §18.6 Hospitality

- **Market size:** Global hospitality industry ~$5 trillion. Hospitality-specific software ~$15B.
- **Incumbents displaced:** Oracle Opera (PMS), Mews (PMS), Cloudbeds (PMS), Toast (restaurant POS), Sabre (distribution).
- **Key capabilities:** Property management, reservations, point-of-sale, housekeeping workflows, distribution channels (cross-tenant marketplace), guest-CRM.
- **Anchor segment:** Boutique hotel chains, restaurant groups, vacation-rental operators.

### §18.7 Agri-food

- **Market size:** Global agri-food industry ~$8 trillion. Agri-food-specific software ~$10B.
- **Incumbents displaced:** John Deere Operations Center (precision farming), Climate FieldView, Granular, AgriWebb (livestock).
- **Key capabilities:** Farm-to-fork traceability, cooperative settlement (multi-tenant DealSet), organic certification (compliance pack), weather-and-yield analytics, field-service-coordination.
- **Anchor segment:** Mid-size farm cooperatives, food-processing companies, traceability-mandated retailers.

### §18.8 Media

- **Market size:** Global media + entertainment industry ~$2.5 trillion. Media-specific software ~$20B.
- **Incumbents displaced:** Avid MediaCentral (broadcast), Rightsline (rights), Vistex (royalty), Adobe Workfront (creative-ops).
- **Key capabilities:** Rights management, royalty settlement, content licensing (multi-tenant DealSet), advertising operations, creative-workflow.
- **Anchor segment:** Mid-cap broadcasters, streaming services, content-creator platforms.

### §18.9 Nonprofit

- **Market size:** Global nonprofit sector ~$2 trillion. Nonprofit-specific software ~$5B.
- **Incumbents displaced:** Blackbaud (Raiser's Edge + Financial Edge), Salesforce NPSP (nonprofit), Donor Tools, NetSuite for Nonprofits.
- **Key capabilities:** Donor management, grants workflow, fund accounting, volunteer coordination, impact reporting.
- **Anchor segment:** Mid-size nonprofits, international NGOs, religious organizations.

### §18.10 The vertical-pack economics

Each vertical pack adds incremental ARR potential roughly as follows (mid-market enterprise focus, conservative assumptions):

- **Anchor tenant count Y2:** 5-15 per vertical.
- **Anchor tenant ACV Y2:** $1M-$10M per tenant.
- **Y2 vertical-pack ARR per pack:** $10M-$100M.
- **Y3-Y4 scaling**: each successful Y2 vertical pack scales to $100M-$500M ARR by Y4.

Across 9 new packs, the Y4 ARR potential is $1B-$5B incremental — not counting the existing 15 packs. This is the vertical-segmentation case for the unified-ecosystem TAM expansion.

---

## §19. Glossary tail + acknowledgments

This briefing is the consolidated narrative work of the architecture council, the foundry orchestration agents, and the multispectrum-review v2.4.0 process applied to the Wave-3-G doc expansion. The underlying corpus is the property of the oyatie program; this briefing is an executive-readable synthesis.

For corrections, scope additions, or new questions from board / investor / GTM stakeholders, file directly into the Foundry pipeline as a documentation-issue against this briefing's path:

`docs/architecture/wave-3-g-executive-briefing-2026-05-21.md`

Future iterations of this briefing (Wave 3-H executive briefing, Wave 3-J executive briefing, etc.) will track the platform's progression. The current version reflects the architecture-completeness state as of 2026-05-20.

---

## §20. Closing executive narrative — the unified-ecosystem bet in one page

If the entire briefing collapses to a single page that an executive can hand a board director on the way into a meeting:

**The premise.** The enterprise-SaaS industry has spent 25 years fragmenting. The average enterprise now runs 110+ separate SaaS applications, pays 30% of IT budget on integration, and absorbs $50K-$500K of fragmentation cost per employee per year. The buyer side of the market — CFOs, CIOs, boards — has finally noticed. The cost is no longer hidden.

**The product.** Oyatie is a single unified software ecosystem that absorbs the role of the 110+ tools. One identity, one policy engine, one workflow engine, one ontology, one audit chain, one marketplace, one UX shell, one training model, one compliance posture, one plugin extensibility. Each "product" (CRM, HR, ERP, ITSM, mail, calendar, messenger, marketplace, drive, etc.) is a role-based capability-tier projection over the shared substrate — not a separate microservice silo.

**The scope.** 18 hero consumer-and-productivity surfaces day-one. 28 SAP S/4HANA modules covered. 165 B2B SaaS vendor surfaces covered through capability tiers + 13 new microservices. 24 vertical packs (15 existing + 9 new in Wave-3-G). 25+ compliance pack overlays. 69 microservices total, all on one unified substrate.

**The audience.** 127 personas across 6 collar-colors, 7 workspaces, 6 skill-tiers, 20+ locales, 6 device profiles, 32+ audience-types. 6 anchor personas (Yejin, Marcus, Aiyana, Tomás, Hiroshi, Anya) tell the day-in-the-life narrative. 7 anchor enterprise tenants (KrampusCorp, GlobalLogistics, AcmeRawMaterials, Yejin's Hospital, Tomás's Restaurant, Anya's Newsroom, Diana's GAO Office) carry the enterprise narrative. 150 canonical user journeys (j01-j150) prove the substrate works end-to-end.

**The doctrine.** 30 ADRs (ADR-0297..0321) establish what the platform does at the edges — life-safety, abuse, recovery, whistleblower-protection, survivor-safety, deceased-user inheritance, cognitive-decline resilience, jurisdiction conflict, AI-agent delegation, disaster mode, detection substrate, ML lifecycle, fairness, investigation, dual-tenant boundary, court-warrant piercing, conglomerate hierarchy, marketplace settlement, ERP parity, capability-tier projection, role-projection, collar-color universality, information-barrier, transient-identity, B2B leader coverage. Plus the keystone-bundle 17-ADR substrate (ADR-0242..0258) underneath.

**The roadmap.** Wave 3-H content pass (in-flight). Wave 3-I capability-tier registry. Wave 3-J code authoring. Wave 3-K launch sequencing (substrate Q1, hero surfaces Q2-Q3, consumer-social + first enterprise wave Q4, ERP Y2 H1, B2B leader Y2 H2). Wave 3-L migration-from-incumbent journeys. Wave 3-M industry-vertical expansion (Y3+). Wave 3-N global locale expansion (Y3+). Wave 3-O edge + offline + air-gap (Y3+). Wave 3-P agentic-product layer (Y3+).

**The first 12 months.** Five hero products to 10M+ users each. Three vertical packs to lighthouse-account status (healthcare + financial-services + public-sector). Eight migration-from-incumbent playbooks (SAP, Salesforce, Workday, ServiceNow, Atlassian, Microsoft 365, Stripe Connect, Snowflake).

**The TAM.** $200B+ enterprise SaaS (Gartner-anchored) + $100B+ consumer-productivity adjacent + $50B+ migration services. Aggregate: $350B+. Y5 revenue target at low-single-digit-percent capture: $10-15B.

**The moat.** Unified-ecosystem flywheel + day-zero adoption + cross-career training-cost amortization + compounding network effects + substrate cost advantage + AI usefulness compounding with substrate completeness.

**The risk.** Execution (69 microservices, 24 months) + GTM (incumbent CIO relationships) + per-vertical market-entry (9 new verticals). Mitigated by the Foundry autonomous-execution pipeline, the consumer-side demand-pull strategy, and the lighthouse-account / anchor-tenant vertical-entry posture.

**The bottom line.** Architecture-completeness is settled. Market-fit thesis is settled. The remaining question is execution + GTM. That is what the next 12 months are about. The board is being asked to bet on execution — not on the strategy. The strategy is the part Wave-3-G locked in.

---

## §21. Reader instructions — how to use this briefing

This briefing is designed to be used three different ways:

- **As a one-meeting board pre-read.** Read §14 (one-page executive summary) and §20 (closing executive narrative). 5 minutes. That is all the context a board director needs for the unified-ecosystem strategic discussion.
- **As a 90-minute deep-dive.** Read the document in order. The narrative flow takes a reader from problem (§2) through solution (§3) through scope (§4) through audience (§5) through operational evidence (§6) through compliance (§7) through doctrine (§8) through roadmap (§9) through investor case (§10) through competitive landscape (§11 + §11.5 + §11.6) through closing (§12) — with appendix sections (§13-§21) available for deeper reference.
- **As a GTM enablement reference.** Sales-engineering teams should treat §3.3 (capability tiers), §5 (personas), §6 (journeys), §11 (vendor head-to-heads), §17 (FAQ), and §18 (vertical-pack deep-dive) as the structured customer-conversation handbook. Each subsection maps to a customer conversation type.

The briefing is intentionally non-technical in tone. Engineering teams who need the same scope at engineering-grade depth should read the underlying companion documents enumerated in §13.1 instead. This briefing exists to be the bridge between those engineering-grade docs and the boardroom.

A note on cadence: this briefing will be refreshed at the close of each Wave (3-H, 3-I, 3-J, 3-K, 3-L). Each refresh keeps the narrative shape stable while updating the scope, the personas, the journeys, the roadmap, and the competitive landscape with the new wave's progress. Board directors and investors should expect to re-read the briefing at each Wave milestone — the strategic thesis stays constant; the operational evidence accumulates.

---

*End of Wave-3-G Executive Briefing. 2026-05-21.*

---

*End of Wave-3-G Executive Briefing. 2026-05-21.*
