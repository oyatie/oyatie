# consent-graph feature parity matrix — 2026-05-20

Audit owner: Wave 3 Batch 3.2 consent-graph audit.
Target microservice: `microservices/consent-graph/`.
Required counterpart set: OneTrust / TrustArc / Cookiebot.
Scope: union-coverage comparison, not a remediation plan.
Tenant-class retirement migration posture: this document does not define feature tiers.
Tenant-class posture: feature quality is uniform; economic overlays belong to tenant_class controls.
Primary local product anchor: `microservices/consent-graph/PRD.md:17-59`.
Primary local contract anchor: `microservices/consent-graph/contracts/openapi/consent-graph.yaml:33-285`.
Primary migration anchor: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:13-27`.
Primary chat anchor: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16290`.

## Source anchors

1. OneTrust product catalog describes Universal Consent & Preference Management, CMP, DSR automation, and platform integrations: https://www.onetrust.com/product/.
2. OneTrust consent-and-preferences page describes centralized preference centers, consent sync across domains/apps/systems, A/B testing, and first-party data collection: https://www.onetrust.com/solutions/consent-and-preferences/.
3. OneTrust developer SLO page publishes UCPM API availability and response objectives for selected APIs: https://developer.onetrust.com/onetrust/reference/consent-preference-management-api-service-level-objectives.
4. OneTrust developer rate-limit page publishes account-level and UCPM endpoint-specific rate limits: https://developer.onetrust.com/onetrust/reference/rate-limits-overview.
5. TrustArc Consent & Preference Manager page describes centralized consent repository, real-time syncing, preference centers, audit trails, geolocation, multilingual support, and marketing/CRM integrations: https://trustarc.com/products/consent-consumer-rights/consent-preference-manager/.
6. TrustArc integrations page describes a library of 300+ prebuilt connectors across privacy workflows: https://trustarc.com/products/integrations/.
7. TrustArc mobile app consent page describes SDK-based mobile consent, app scanning, location/language experiences, and customizable preference center support: https://trustarc.com/products/consent-consumer-rights/mobile-app-consent/.
8. Cookiebot Google Consent Mode page describes Google Consent Mode integration, cookie/tracker scan, consent banner, and plan limits such as subpage counts and no traffic limitations: https://www.cookiebot.com/us/cookiebot-cmp-google-consent-mode/.
9. Cookiebot support page describes CMP data processing, consent IDs, consent log database, and first-party consent cookie behavior: https://support.cookiebot.com/hc/en-us/articles/14455846346652-Data-processed-when-using-Cookiebot-CMP.
10. Cookiebot support page describes consent-data extraction API: https://support.cookiebot.com/hc/en-us/articles/4405045044882-Extracting-consent-data-via-API.
11. Cookiebot support page describes logging and demonstration of user consents: https://support.cookiebot.com/hc/en-us/articles/360003782654-Logging-and-demonstration-of-user-consents.
12. Cookiebot support page describes website scanner page discovery and up-to-10,000-page URL list visibility for subscribed accounts: https://support.cookiebot.com/hc/en-us/articles/360003773214-How-does-the-Cookiebot-scanner-define-pages.
13. Cookiebot support page describes IAB TCF support and CMP API exposure: https://support.cookiebot.com/hc/en-us/articles/360007652694-Cookiebot-CMP-and-the-Transparency-and-Consent-Framework-TCF.
14. Cookiebot support page describes Google Consent Mode check scanning time and report retention: https://support.cookiebot.com/hc/en-us/articles/15485609486492-Google-Consent-Mode-Checker.

## §1 Counterpart 1 — OneTrust capability surface

1. OneTrust is a broad responsible-data-use platform, not only a consent microservice.
2. OneTrust Consent & Preferences includes Universal Consent & Preference Management and a Consent Management Platform.
3. OneTrust public catalog says UCPM gives users a single portal for consent, preferences, and first-party data.
4. OneTrust public catalog says CMP captures and manages cookie consent on websites, mobile apps, OTT apps, and connected TVs.
5. OneTrust public catalog also includes DSR automation from intake through secure response.
6. OneTrust public catalog says more than 14,000 customers rely on the platform.
7. OneTrust consent page emphasizes capturing and honoring consent across the customer lifecycle.
8. OneTrust consent page emphasizes website visitors, app users, marketing subscribers, and customers.
9. OneTrust consent page emphasizes centralized preference centers.
10. OneTrust consent page emphasizes syncing consent across domains, apps, and systems.
11. OneTrust consent page emphasizes pre-built API integrations with common business apps.
12. OneTrust consent page emphasizes A/B testing of privacy experiences.
13. OneTrust consent page emphasizes first-party data capture in addition to consent capture.
14. OneTrust pricing page says UCPM pricing is based on total data-subject profiles captured.
15. OneTrust developer SLO page says UCPM APIs are available 99 percent of the time as an objective.
16. OneTrust developer SLO page says selected requests have P99 latency within a 500 ms satisfactory threshold.
17. OneTrust developer SLO page says less than 0.5 percent of requests should return 5XX.
18. OneTrust developer SLO page says preference centers remain accessible during maintenance.
19. OneTrust developer SLO page says selected data subject, preference, and receipt APIs have P99 under 500 ms.
20. OneTrust developer rate-limit page says account default is 200,000 calls/hour and 20,000 calls/minute.
21. OneTrust developer rate-limit page says sandbox default is 50,000 calls/hour and 5,000 calls/minute.
22. OneTrust developer rate-limit page says UCPM consent receipts have 2,000 calls/minute for one endpoint.
23. OneTrust developer rate-limit page says preferences have 3,000 calls/minute for one endpoint.
24. OneTrust developer rate-limit page says consent receipt bulk ingestion has 3,000 calls/minute.
25. OneTrust developer rate-limit page says some consent profile APIs have 300 calls/minute or 1,000 calls/minute quotas.
26. OneTrust has strong consumer-facing privacy UX coverage.
27. OneTrust has strong enterprise API and reporting coverage.
28. OneTrust has strong cookie and mobile CMP coverage.
29. OneTrust has public API SLO and rate-limit detail that can be benchmarked.
30. OneTrust does not publicly present the same cross-tenant zero-copy projection and bilateral audit-chain design that consent-graph owns.
31. OneTrust is a high bar for consent capture, preference management, privacy UX, and operational reporting.
32. OneTrust is a weaker direct comparator for real-time B2B data-sharing enforcement.
33. Consent-graph must match OneTrust's consent-record, preference, audit, reporting, and integration affordances where those are in scope.
34. Consent-graph should not copy OneTrust's workflow-platform sprawl into the enforcement service.
35. Consent-graph needs explicit handoffs to workflow-engine, identity, api-gateway, and application for OneTrust-style UX surfaces.

## §2 Counterpart 2 — TrustArc capability surface

1. TrustArc is a privacy management platform with Consent & Preference Manager, Cookie Consent Manager, Individual Rights Manager, and Trust Center surfaces.
2. TrustArc CPM page says the product captures consent and preferences across channels.
3. TrustArc CPM page positions the product as a central hub for brands, tools, and digital channels.
4. TrustArc CPM page says customers can view consent history and modify preferences at any time.
5. TrustArc CPM page says public-facing web elements meet WCAG 2.2 Level AA and ADA standards.
6. TrustArc CPM page emphasizes documented user consent for collection, use, and disclosure.
7. TrustArc CPM page emphasizes secure consent storage and real-time syncing.
8. TrustArc CPM page emphasizes third-party application integration.
9. TrustArc CPM page emphasizes preference data sync to downstream systems.
10. TrustArc CPM page describes cross-platform compatibility across web and mobile.
11. TrustArc CPM page describes dynamic intake forms and consent prompts.
12. TrustArc CPM page lists GDPR, CCPA, LGPD, PIPEDA, UK DPA, ePrivacy, GPC, and more.
13. TrustArc CPM page describes role-based access and data anonymization/pseudonymization.
14. TrustArc CPM page describes drag-and-drop setup, templates, and custom CSS for forms.
15. TrustArc CPM page describes audit trails and reports for compliance.
16. TrustArc CPM page describes magic links for personalized consent and preference forms.
17. TrustArc CPM page describes multilingual and geolocation-based banner experiences.
18. TrustArc CPM FAQ says CPM is purpose-built for zero-party and first-party preference data.
19. TrustArc CPM FAQ says second-party data typically flows through the same CPM records once ingested.
20. TrustArc CPM FAQ says preferences sync through marketing, CRM, and communication systems.
21. TrustArc CPM FAQ lists Salesforce, Marketo, HubSpot, Adobe Experience Platform, Mailchimp, Iterable, Twilio, BigQuery, and custom applications.
22. TrustArc integrations page describes 300+ prebuilt connectors.
23. TrustArc integrations page says connector templates automate consent syncing, DSR fulfillment, vendor onboarding, and other privacy workflows.
24. TrustArc mobile app consent page describes SDK-based app consent.
25. TrustArc mobile app consent page describes in-depth app scanning for third parties collecting mobile-app data.
26. TrustArc mobile app consent page says SDK support includes Android, iOS, React Native, and Flutter.
27. TrustArc mobile app consent page describes configurable experience by user location and language.
28. TrustArc Individual Rights Manager page describes DSR automation and integrations.
29. TrustArc is a high bar for privacy program breadth, connector breadth, web/mobile consent UX, and compliance reporting.
30. TrustArc does not publicly present the same low-latency bilateral data-sharing enforcement substrate that consent-graph owns.
31. TrustArc is a closer counterpart than Snowflake for consumer consent and preference workflows.
32. Consent-graph must map TrustArc's consent repository and preference sync into its own data-sharing enforcement scope.
33. Consent-graph should assign TrustArc-style DSR fulfillment to workflow-engine while owning downstream revocation receipts.
34. Consent-graph should assign TrustArc-style trust center to another µservice unless product scope changes.
35. Consent-graph needs explicit handoffs for TrustArc-style mobile SDK and preference-center UI surfaces.

## §3 Counterpart 3 — Cookiebot capability surface

1. Cookiebot is a focused consent management platform for websites and tracking technologies.
2. Cookiebot Google Consent Mode page says the product integrates with Google Consent Mode.
3. Cookiebot Google Consent Mode page says it performs initial scans for cookies and trackers.
4. Cookiebot Google Consent Mode page says setup can be automated with minimal manual effort.
5. Cookiebot pricing/feature page says free and paid plan eligibility can be based on subpage count.
6. Cookiebot pricing/feature page says Cookiebot does not charge based on page views or usage.
7. Cookiebot pricing/feature page says it supports GDPR and ePrivacy.
8. Cookiebot pricing/feature page says it supports applicable US privacy laws including CCPA/CPRA.
9. Cookiebot pricing/feature page says Google Consent Mode is supported.
10. Cookiebot pricing/feature page says premium plans can customize banner design.
11. Cookiebot pricing/feature page says multiple domain handling is supported.
12. Cookiebot pricing/feature page says advanced automated reporting is supported.
13. Cookiebot pricing/feature page says 47+ languages are supported.
14. Cookiebot pricing/feature page says banner distribution by region and country is supported.
15. Cookiebot support page says the CMP script loads a consent banner on website load.
16. Cookiebot support page says the CMP blocks non-essential services before consent.
17. Cookiebot support page says it does not collect, retain, or share PII before consent beyond data needed to serve the banner.
18. Cookiebot support page says it generates a Consent ID and saves it with Consent State in a consent log database.
19. Cookiebot support page says the Consent ID and state are stored client-side in a first-party consent cookie.
20. Cookiebot consent-log page says logged consent data includes anonymized IP, date/time, user agent, URL, encrypted key value, and consent state.
21. Cookiebot consent-log page says the first-party consent cookie can persist for up to 12 months.
22. Cookiebot extraction API page describes an API for consent statistics by domain and date range.
23. Cookiebot scanner page says the scanner scans website-accessible HTML content and linked content.
24. Cookiebot scanner page says subscribed accounts can access URL lists with up to 10,000 found subpages.
25. Cookiebot scan-report page says newly added domains are automatically scanned monthly.
26. Cookiebot TCF page says Cookiebot supports IAB TCF and exposes the standard CMP API.
27. Cookiebot TCF page says the IAB consent string is stored in the existing consent cookie and included in consent-log downloads.
28. Cookiebot Google Consent Mode checker page says checks take approximately 10-15 seconds and reports remain available for 30 days.
29. Cookiebot is a high bar for web CMP specificity, scan evidence, tag gating, consent logs, and adtech protocol compatibility.
30. Cookiebot is not a broad B2B data-sharing enforcement platform.
31. Cookiebot is a necessary union-coverage comparator because consent-graph currently acknowledges no tenant-facing cookie banner.
32. Consent-graph should not silently claim Cookiebot parity through enforcement APIs alone.
33. Consent-graph must either own a Cookiebot-style web CMP surface or name the handoff owner.
34. Consent-graph must define how cookie consent records flow into DataSharingAgreement or subject-consent state.
35. Consent-graph must define whether Google Consent Mode and IAB TCF strings are source inputs, outputs, or out-of-scope artifacts.

## §4 UNION-coverage matrix

| # | Capability | OneTrust | TrustArc | Cookiebot | consent-graph current evidence | Audit status |
|---:|---|---|---|---|---|---|
| 1 | Central consent record repository | Strong | Strong | Narrow web consent log | PRD DataSharingAgreement and consent registry goals `PRD.md:48-59` | Partial |
| 2 | Marketing preference center | Strong | Strong | Banner preferences | No UI owner in service path | Gap |
| 3 | Cookie consent banner | Strong CMP product | Strong Cookie Consent Manager | Core product | Migration says not shipped `migration-playbooks/from-onetrust-and-trustarc.md:102-107` | Gap |
| 4 | Cookie/tracker scanner | Strong CMP adjacency | Strong cookie manager/app scanner | Core scanner | No scanner contract | Gap |
| 5 | Mobile consent SDK | Strong mobile/CTV SDK coverage | Strong mobile SDK coverage | Web-script centric | PRD SDK goal but no mobile SDK docs | Gap |
| 6 | OTT/CTV consent | Public OneTrust product surface | Not primary source in inspected TrustArc docs | Not primary source in inspected Cookiebot docs | No OTT/CTV scope | Gap |
| 7 | Consent receipt creation | Public OneTrust API endpoint | Audit trail repository | Consent ID and log | Revocation receipts and audit-chain events, not web consent receipts | Partial |
| 8 | Consent receipt lookup | Public OneTrust receipt APIs | Audit trail export | Consent-log lookup | OpenAPI revocation receipts only `contracts/openapi/consent-graph.yaml:250-285` | Partial |
| 9 | Consent history view | Strong | Strong | Consent log | Audit-chain and agreement lifecycle, no user UI | Partial |
| 10 | User self-service revocation | Strong preference center | Strong preference center | Banner preference change | PRD self-service revocation `PRD.md:57-58`; ADR-SVC-CG-005 | Covered for DSA |
| 11 | Cross-domain consent sync | Strong | Strong multi-system sync | Cross-domain consent sharing support | No web-domain sync contract | Gap |
| 12 | Cross-tenant DSA lifecycle | Not core public surface | Not core public surface | Not core public surface | OpenAPI agreement lifecycle `contracts/openapi/consent-graph.yaml:33-151` | Covered |
| 13 | Data-sharing agreement draft | Not core public surface | Partial B2B agreement migration | Not core public surface | `POST /v1/agreements` and schemas | Covered |
| 14 | Data-sharing agreement offer/accept | Not core public surface | Partial through workflows | Not core public surface | OpenAPI offer and accept endpoints | Covered |
| 15 | Data-sharing agreement amend | Not core public surface | Partial | Not core public surface | OpenAPI amend endpoint | Covered |
| 16 | Data-sharing agreement revoke | Preference withdrawal | Preference withdrawal | Consent withdrawal | OpenAPI revoke endpoint and revocation service | Covered |
| 17 | Real-time enforcement check | API SLO for selected APIs | Real-time sync claim | Banner/tag gating | OpenAPI enforcement endpoint `contracts/openapi/consent-graph.yaml:152-179` | Covered |
| 18 | Cedar policy gate | Not public | Not public | Not public | Cedar policies and ADR-0243 alignment | Covered |
| 19 | Field-level scope narrowing | Preference purpose scoping | Preference/purpose scoping | Cookie categories | EntityScope and field sets `contracts/openapi/consent-graph.yaml:327-377` | Covered |
| 20 | Purpose-of-use enforcement | Purpose preferences | Purpose preferences | Consent categories | SharingTerms and policy context | Covered |
| 21 | Data minimization for projections | Not same architecture | Not same architecture | Tracker blocking | Projection gateway and scope narrowing IPs | Covered |
| 22 | Aggregate sharing mode | Not primary | Not primary | Not primary | ADR-SVC-CG-003 and OpenAPI mode enum | Covered |
| 23 | Attested query mode | Not primary | Not primary | Not primary | ADR-SVC-CG-003 and OpenAPI mode enum | Covered |
| 24 | Differential privacy budget | Not public UCPM core | Not public CPM core | Not public CMP core | PRD marketplace aggregate use case `PRD.md:135-144` | Covered |
| 25 | k-anonymity suppression | Not public UCPM core | Not public CPM core | Not public CMP core | Cedar aggregate policy `policy/aggregate-k-anonymity.cedar:11-26` | Covered |
| 26 | Break-glass healthcare mode | Privacy workflow adjacency | Healthcare privacy workflow adjacency | Not primary | OpenAPI break-glass and Cedar overlay | Covered |
| 27 | Bilateral audit chain | Audit reports | Audit trails | Consent log | PRD and ADR-SVC-CG-001 | Covered |
| 28 | Audit-chain cross-pointer | Not public | Not public | Not public | ADR-SVC-CG-001 and IP-013 | Covered |
| 29 | Revocation fanout | Integration workflows | Real-time sync | Consent cookie/tag state | Revocation service and AsyncAPI priority topic | Covered |
| 30 | Revocation p99 target | Public OneTrust SLO not revocation-specific | Not public numeric | Not public numeric | SLO p99 1s `slos/revocation-propagation-latency.openslo.yaml:22-34` | Covered |
| 31 | Projection freshness target | Not public | Not public | Not applicable | SLO p95 500ms `slos/cross-tenant-projection-freshness.openslo.yaml:31-33` | Covered |
| 32 | Consent grant p95 target | API SLO selected endpoints | Not public numeric | Not public numeric | SLO p95 2s `slos/consent-grant-latency.openslo.yaml:31-33` | Covered |
| 33 | Cedar p99 target | Not public | Not public | Not public | SLO p99 10ms `slos/cedar-evaluation-latency.openslo.yaml:31-33` | Covered |
| 34 | Partner directory handshake | Not core public UCPM | Integrations/connectors | Not primary | OpenAPI partner directory and SLO p95 30s | Covered |
| 35 | Partner offboarding | Third-party lifecycle elsewhere | Integrations/privacy workflows | Not primary | Runbook partner-offboarding | Covered |
| 36 | Consent forgery detection | Audit report possible | Audit trail possible | Consent log tamper check | Runbook consent-forgery-detected | Covered |
| 37 | Data-residency enforcement | Platform privacy controls | Geolocation and laws | Region/country banner | Data-residency docs and ADR-SVC-CG-004 | Covered |
| 38 | Grantor-region topic ownership | Not public | Not public | Not public | ADR-SVC-CG-004 | Covered |
| 39 | Cross-border transfer control | Privacy program | Privacy program | Geo banner | OpenAPI sovereignty schema `contracts/openapi/consent-graph.yaml:383-395` | Covered |
| 40 | Regulatory pack overlays | Privacy rules | Privacy rules | Region rules | Compliance and Kustomize overlays | Partial |
| 41 | GDPR support | Strong | Strong | Strong | Compliance map and DSAR runbook | Covered |
| 42 | CCPA/CPRA support | Strong | Strong | Strong | Compliance map | Covered |
| 43 | HIPAA support | Module workflow adjacency | Privacy platform adjacency | Not primary | Break-glass healthcare and compliance map | Covered |
| 44 | KR PIPA support | Likely privacy program | Global privacy laws | Region-specific banner | Compliance map KR section | Covered |
| 45 | LGPD support | Strong | Strong | Not primary but applicable | Compliance map BR section | Covered |
| 46 | GPC support | Likely CMP | Explicit TrustArc CPM page | Web CMP likely | No GPC field in consent-graph contracts | Gap |
| 47 | Google Consent Mode | CMP support likely | Cookie manager possible | Core public support | No GCM contract | Gap |
| 48 | IAB TCF | CMP support likely | Cookie manager possible | Core public support | No TCF string schema | Gap |
| 49 | Consent Mode checker | Not primary inspected | Not primary inspected | Public check tool | No equivalent checker | Gap |
| 50 | Consent log extraction API | API/reporting | Reports/export | Public extraction API | Revocation receipt APIs only | Partial |
| 51 | Consent proof for regulator | Strong | Strong | Strong consent log | Audit-chain evidence | Covered |
| 52 | Preference analytics dashboard | Strong | Strong | Consent statistics paid plans | Dashboards are service-SLO, not marketing preference analytics | Partial |
| 53 | Consent opt-in A/B testing | Strong OneTrust | Possible form customization | Banner customization | No experimentation surface | Gap |
| 54 | Custom CSS/form builder | Strong | Strong | Banner customization | No UI builder | Gap |
| 55 | Magic links | Not primary inspected | Public TrustArc CPM feature | Not primary | No magic-link preference form | Gap |
| 56 | WCAG/ADA web-element claim | Not inspected | Public TrustArc CPM claim | Banner UX likely accessible | No UX components | Gap |
| 57 | 47+ language support | Not inspected | 60+ languages | 47+ languages | Compliance packs mention regions, not UI language | Gap |
| 58 | Geolocation-based banner | CMP support likely | Public TrustArc feature | Public Cookiebot feature | Sovereignty and region rules, no banner geolocation | Partial |
| 59 | Multi-domain management | Strong | Strong | Public Cookiebot feature | Partner/tenant not domain group | Gap |
| 60 | Multiple brands | Strong | Strong | Domain groups | No brand preference center | Gap |
| 61 | CRM/MAP sync | Strong integrations | Strong integrations | Not primary | Partner-directory and projection, not marketing sync | Partial |
| 62 | Salesforce connector | Public TrustArc FAQ | Public TrustArc FAQ | Not primary | No named connector | Gap |
| 63 | Marketo connector | Public TrustArc FAQ | Public TrustArc FAQ | Not primary | No named connector | Gap |
| 64 | HubSpot connector | Public TrustArc FAQ | Public TrustArc FAQ | Not primary | No named connector | Gap |
| 65 | Adobe Experience Platform connector | Public TrustArc FAQ | Public TrustArc FAQ | Not primary | No named connector | Gap |
| 66 | BigQuery sync | Public TrustArc FAQ | Public TrustArc FAQ | Not primary | No named connector | Gap |
| 67 | Webhook integration | OneTrust rate-limit docs | TrustArc integrations | Cookiebot API | AsyncAPI events, no public webhook intake | Partial |
| 68 | Event streaming | Not public UCPM core | Not public CPM core | Not primary | AsyncAPI/Pulsar design | Covered |
| 69 | Bulk consent ingestion | Public OneTrust endpoint | Integration templates | API stats extraction | Migration inventory commands, no public bulk OpenAPI | Partial |
| 70 | DSR intake | Strong DSR Automation | Strong Individual Rights Manager | Not primary | Routed to workflow-engine in migration | External |
| 71 | DSR ID verification | Strong OneTrust | Strong TrustArc | Not primary | Routed to workflow-engine | External |
| 72 | DSR fulfillment | Strong OneTrust | Strong TrustArc | Not primary | DSAR cascade runbook for cross-tenant effects | Partial |
| 73 | DSR downstream deletion cascade | Workflow automation | Workflow automation | Not primary | DSAR cascade runbook | Covered |
| 74 | Vendor onboarding | OneTrust third-party | TrustArc integrations | Not primary | Partner-directory handshake | Partial |
| 75 | Vendor offboarding | OneTrust third-party | TrustArc integrations | Not primary | Partner offboarding runbook | Covered |
| 76 | Privacy trust center | OneTrust platform adjacency | TrustArc Trust Center | Not primary | No trust center surface | External or gap |
| 77 | Privacy notice management | OneTrust platform adjacency | TrustArc platform adjacency | Banner disclosures | No notice management surface | External |
| 78 | Data inventory/data mapping | OneTrust privacy ops | TrustArc data mapping | Not primary | Ontology dependency, not owned | External |
| 79 | Data-subject profile | Strong UCPM | Strong CPM | Anonymous consent log | Subject-level consent registry goal, no detailed schema | Partial |
| 80 | Anonymous visitor consent | Strong CMP | Strong cookie manager | Core product | No anonymous web visitor schema | Gap |
| 81 | Known customer consent | Strong | Strong | Consent log keyed to anonymous state | Agreement principals and tenants | Covered for DSA |
| 82 | Household/shared account consent | Not inspected | Not inspected | Not primary | Journey IP exists `IP-journey-j04-shared-account-consent-rewrite.md` | Partial |
| 83 | B2C self-service rights ledger | Platform surface | Platform surface | Not primary | Multiple journey IPs | Partial |
| 84 | Emergency 911 opt-in fields | Not primary | Not primary | Not primary | Journey IP exists | Covered by domain extension |
| 85 | Consent purpose model | Strong | Strong | Cookie categories/purposes | SharingTerms purpose and policy context | Covered |
| 86 | Consent withdrawal model | Strong | Strong | Banner state changes | RevocationService | Covered |
| 87 | Consent expiry model | Strong | Strong | Cookie expiry | Agreement state and expiration | Covered |
| 88 | Agreement state machine | Not core public UCPM | Not core public CPM | Not primary | PRD and OpenAPI | Covered |
| 89 | Partner chain integrity | Not public | Not public | Not public | Bilateral-chain SLO and ADR | Covered |
| 90 | Sovereignty violation SLO | Privacy controls | Privacy controls | Geo controls | OpenSLO target 1.0 | Covered |
| 91 | Audit completeness SLO | Reporting | Reporting | Consent log | OpenSLO target 1.0 | Covered |
| 92 | Agreement divergence SLO | Not public | Not public | Not public | OpenSLO target 1.0 | Covered |
| 93 | Capacity model | Public API limits only | Public connector count | Public scan/page facts | Detailed local capacity model | Covered |
| 94 | Cost model | Pricing public, not infra | Pricing/TEI public | Public plan pricing | Local cost-budget | Covered internally |
| 95 | Tenant economic model | Pricing profiles | SaaS pricing | Plan pricing | No tenant_class semantics | Gap |
| 96 | Demo trial infrastructure | SaaS trials | SaaS demo | Free/small-site plan | No OCI Always Free profile | Gap |
| 97 | Revenue-share substrate controls | Not public | Not public | Not public | No revenue_share semantics | Gap |
| 98 | Paid contractual SLO | Enterprise contracts | Enterprise contracts | Paid plans | No tenant_class expression | Gap |
| 99 | BYOK controls | Platform-dependent | Platform-dependent | Not primary | cloud-secrets dependency, no tenant_class BYOK policy | Partial |
| 100 | Compliance packs by tenant | Platform-dependent | Platform-dependent | Region/legal plans | Pack overlays, no tenant_class gating | Partial |
| 101 | All-six deployment contexts | SaaS hosted | SaaS hosted | SaaS hosted | Not declared | Gap |
| 102 | OpenTofu per-context IaC | Not counterpart feature | Not counterpart feature | Not counterpart feature | Absent | Gap |
| 103 | OCI Always Free profile | Not counterpart feature | Not counterpart feature | Free plan analogy | Absent | Gap |
| 104 | Supported OS manifest | Not public | Not public | Not public | Absent | Gap |
| 105 | Rust backend implementation | Not public | Not public | Not public | No source path | Gap |
| 106 | Generated SDK provenance | Public SDK docs | Public SDK/connector docs | Public API | Rust reference only, no generator boundary | Partial |
| 107 | Security threat model | Platform security | Platform security | CMP privacy model | Threat model present | Covered |
| 108 | Incident response | Enterprise support | Enterprise support | Support docs | Incident response present | Covered |
| 109 | Runbook coverage | Enterprise operations | Enterprise operations | Support docs | Runbooks present | Covered |
| 110 | Human onboarding | Customer success | Customer success | Setup guides | Onboarding doc present | Covered |
| 111 | Migration from counterpart | Professional services | Professional services | Not covered | OneTrust/TrustArc migration present; Cookiebot absent | Partial |
| 112 | Cookiebot-specific migration | Not applicable | Not applicable | Self-source | No Cookiebot migration playbook | Gap |
| 113 | Consent API public SLO parity | Published by OneTrust | Not public numeric | Not public numeric | Local OpenSLO strong, no live API evidence | Partial |
| 114 | API rate-limit policy | Published by OneTrust | Not inspected | Not inspected | No tenant-class rate-limit contract | Gap |
| 115 | Rate-limit enforcement | API gateway/platform | Integrations | Banner/server APIs | Cedar context has max_qps; no tenant_class overlays | Partial |
| 116 | Legal audit export | Strong | Strong | Strong | Audit-chain evidence and compliance map | Covered |
| 117 | Evidence retention | Strong | Strong | Consent log/cookie limits | Retention by pack in compliance map | Covered |
| 118 | Partner audit proof exchange | Not public | Not public | Not public | Partner handshake and audit-chain roots | Covered |
| 119 | Consent analytics for marketers | Strong | Strong | Consent statistics | No marketing analytics | Gap |
| 120 | Data-sharing moat vs CMPs | Not public as DSA graph | Not public as DSA graph | Not applicable | Core service differentiator | Ahead |

## §5 Family summary

1. Consent capture family: consent-graph covers cross-tenant DSA consent, while OneTrust, TrustArc, and Cookiebot lead web/mobile/user-facing consent capture.
2. Preference management family: consent-graph has no dedicated preference center, while OneTrust and TrustArc are strong and Cookiebot has banner preference choices.
3. Cookie/tracker family: consent-graph has no scanner or banner, while Cookiebot is strong and OneTrust/TrustArc have CMP surfaces.
4. Data-sharing enforcement family: consent-graph is ahead because it owns agreement lifecycle, Cedar enforcement, projection, aggregate, attested query, and revocation fanout.
5. Audit evidence family: consent-graph is ahead for bilateral audit-chain and service-level SLO evidence; counterparts are strong in regulatory reporting and consent logs.
6. Revocation family: consent-graph is ahead for p99 enforcement freshness and downstream receipt semantics.
7. UX family: consent-graph depends on other services for UI and is behind the top-three in self-service UX breadth.
8. Integration family: OneTrust and TrustArc are ahead on named enterprise app integrations; consent-graph has event streams and migration commands but lacks connector catalog parity.
9. Compliance pack family: consent-graph has broad pack docs, but top-three vendors have mature regulatory UX and reporting surfaces.
10. Deployment family: consent-graph is behind canonical Oyatie requirements because all-six context, OpenTofu, OCI Always Free, and OS manifest evidence are absent.
11. Tenant-class family: consent-graph is behind the current doctrine because it has no tenant_class semantics.
12. Runtime policy family: consent-graph is ahead on Cedar-specific enforcement and fail-closed behavior.
13. Mobile family: OneTrust and TrustArc are ahead on mobile-app consent SDK surfaces.
14. Adtech protocol family: Cookiebot is ahead on IAB TCF and Google Consent Mode.
15. Migration family: consent-graph is partial; it covers OneTrust and TrustArc but not Cookiebot.

## §6 Headline gap analysis

1. Gap: no tenant-facing cookie-banner surface.
2. Evidence: migration playbook says a tenant may keep OneTrust/TrustArc for cookie-banner UI because Oyatie did not ship one as of 2026-05: `microservices/consent-graph/migration-playbooks/from-onetrust-and-trustarc.md:102-107`.
3. Counterpart pressure: Cookiebot's core product is consent banner plus cookie/tracker blocking and logs.
4. Impact: consent-graph cannot claim union coverage against Cookiebot without a formal handoff.
5. Gap: no cookie/tracker scanning surface.
6. Evidence: no scanner contract, schema, runbook, or worker in inventory.
7. Counterpart pressure: Cookiebot scanner and TrustArc mobile/app scanning are explicit public capabilities.
8. Impact: no source-of-truth for cookies, trackers, or adtech vendor discovery.
9. Gap: no IAB TCF or Google Consent Mode contract.
10. Evidence: no `TCF`, `Consent Mode`, `gcm`, or consent-string schema appears in inspected local contracts.
11. Counterpart pressure: Cookiebot advertises and documents both surfaces.
12. Impact: web CMP parity is absent.
13. Gap: no marketing preference center.
14. Evidence: service owns APIs and runbooks, not UI components; PRD places workflow/UI adjacency outside the microservice.
15. Counterpart pressure: OneTrust and TrustArc lead with centralized preference centers.
16. Impact: OneTrust/TrustArc parity depends on application/workflow services.
17. Gap: no enterprise connector breadth.
18. Evidence: catalog entries are internal Oyatie components, not Salesforce/Marketo/HubSpot/Adobe/Mailchimp/Twilio connector artifacts.
19. Counterpart pressure: TrustArc publishes 300+ prebuilt connectors and lists common marketing systems.
20. Impact: migration from privacy platforms requires connector strategy outside consent-graph.
21. Gap: no tenant_class model.
22. Evidence: no tenant_class search hits and capability-ladder fields remain in contracts and policy.
23. Counterpart pressure: counterparts sell by SaaS plans or volume; Oyatie replacement model requires demo_trial, paid, and revenue_share.
24. Impact: commercial/operational controls cannot be audited.
25. Gap: no OpenTofu context modules.
26. Evidence: service IaC contains Helm/Kustomize only.
27. Counterpart pressure: not a commercial counterpart feature, but a canonical Oyatie requirement.
28. Impact: deployability claims are not proven.
29. Gap: no OS support manifest.
30. Evidence: no supported-OS artifact in inventory.
31. Counterpart pressure: not a public vendor feature, but an Oyatie product maturity requirement.
32. Impact: deployability and support posture remain ambiguous.
33. Gap: no service-local implementation code or tests.
34. Evidence: no `src/` and no `tests/` under the service path.
35. Counterpart pressure: top-three vendors are shipping products; docs alone are not parity.
36. Impact: intern-buildability is design-only.
37. Gap: existing benchmark and competitor matrix do not use the required top-three union.
38. Evidence: current matrix centers Snowflake/Databricks/open-banking/HIE, and benchmark omits Cookiebot from measured rows.
39. Counterpart pressure: current dispatch requires OneTrust / TrustArc / Cookiebot.
40. Impact: older parity artifacts cannot be used as-is for Wave 3 Batch 3.2 closure.

## §7 Additive surface required for union coverage

1. Add a consent-source ingestion contract for OneTrust, TrustArc, and Cookiebot exports.
2. Add a Cookiebot-specific migration playbook or explicitly assign it to a web-CMP owner service.
3. Add a consent-record schema that separates data-subject consent, cookie consent, marketing preferences, and DataSharingAgreement consent.
4. Add a mapping from web consent categories to Cedar purpose-of-use where the mapping is legally valid.
5. Add a mapping from IAB TCF consent strings to internal consent evidence if web CMP ownership stays in consent-graph.
6. Add a mapping from Google Consent Mode states to internal consent evidence if web CMP ownership stays in consent-graph.
7. Add a preference-center handoff contract to application/workflow-engine if consent-graph does not own UI.
8. Add DSAR workflow handoff from consent-graph to workflow-engine and back to revocation receipts.
9. Add connector handoff records for Salesforce, Marketo, HubSpot, Adobe Experience Platform, Mailchimp, Iterable, Twilio, BigQuery, and custom applications.
10. Add tenant_class fields to policy context, contracts, manifest, rate-limit logic, and cost controls.
11. Add demo_trial usage caps tied to OCI Always Free profile.
12. Add paid tenant controls tied to contractual SLO, compliance packs, and BYOK.
13. Add revenue_share tenant controls tied to at-cost substrate and revenue share settlement.
14. Add all-six deployment_context declarations to manifest.
15. Add OpenTofu per-context IaC modules.
16. Add OCI Always Free profile module for demo_trial infrastructure.
17. Add supported-oses manifest.
18. Add source and test evidence or downgrade GA claims.
19. Add live benchmark harness evidence for consent-grant, projection freshness, revocation propagation, Cedar evaluation, and partner handshake.
20. Add public-vendor benchmark evidence with published, estimated, and local-measured numbers separated.
21. Add no-retired-tier migration notes to FAQ, tutorials, benchmarks, policy, contracts, manifest, and capability docs.
22. Add web-cookie scope decision: own in consent-graph, hand off to a CMP service, or intentionally out-of-scope with migration boundary.
23. Add accessibility ownership decision for preference center and consent banner UX.
24. Add mobile consent SDK ownership decision.
25. Add OTT/CTV consent ownership decision.
26. Add adtech protocol ownership decision.
27. Add consent-log export API if consent-graph owns regulator evidence for web consent.
28. Add user-visible consent history API if consent-graph owns preference-center backend.
29. Add audit-chain evidence mapping for every imported consent-source class.
30. Add partner-directory mapping for B2B agreement migration.
31. Add business-process mapping for keeping counterpart systems active during cutover.
32. Add explicit statement that OneTrust/TrustArc workflow features are external unless implemented by workflow-engine.
33. Add explicit statement that Cookiebot web-CMP features are external unless implemented by a web-CMP owner.
34. Add one union-coverage tracker that can be updated without reintroducing feature tiers.
35. Add acceptance tests for imported consent records, revocation cascade, preference withdrawal, and cookie-consent evidence if those surfaces are in scope.

## §8 Bottom line

1. consent-graph is ahead of the top-three counterparts for real-time B2B data-sharing enforcement.
2. consent-graph is ahead for bilateral audit-chain linkage and sub-second revocation goals.
3. consent-graph is at parity or partial parity for consent record history, regulatory evidence, DSAR cascade, and compliance-pack mapping.
4. consent-graph is behind OneTrust and TrustArc for preference centers, marketing-system integrations, mobile consent SDKs, and broad privacy workflow UX.
5. consent-graph is behind Cookiebot for cookie/tracker scanning, banner delivery, IAB TCF, Google Consent Mode, and consent-log extraction.
6. consent-graph is behind canonical Oyatie direction for OpenTofu, six deployment contexts, OS support, tenant_class, and OCI Always Free profile evidence.
7. The correct product stance is not "consent-graph replaces OneTrust, TrustArc, and Cookiebot wholesale."
8. The correct product stance is "consent-graph is the enforcement substrate and must either own or formally hand off the user-facing CMP/preference surfaces represented by the top-three union."
9. The next parity artifact should track union coverage by capability family and owner service, not by feature tier.
10. The 2026-05-20 reports should be treated as the current evidence baseline for this microservice.

## §9 Ownership-coherence implications

1. consent-graph should own DataSharingAgreement state, not every consent UX.
2. consent-graph should own enforcement decisions, not marketing-channel preference orchestration.
3. consent-graph should own revocation fanout and receipts, not every DSAR workflow screen.
4. consent-graph should own bilateral audit linkage, not the global audit-chain substrate.
5. consent-graph should own projection-scope narrowing, not source-system data modeling.
6. consent-graph should consume Ontology entity shapes rather than redefine them.
7. consent-graph should consume identity principals rather than implement account recovery.
8. consent-graph should consume tenancy pack activation rather than decide tenant lifecycle.
9. consent-graph should emit observability evidence rather than own platform observability.
10. consent-graph should use policy-engine corpus provenance rather than hand-own every policy release workflow.
11. consent-graph should expose migration adapters for OneTrust and TrustArc consent records.
12. consent-graph should not silently absorb OneTrust/TrustArc DSR workflow builders.
13. consent-graph should expose revocation callbacks used by workflow-engine during DSR fulfillment.
14. consent-graph should not silently absorb Cookiebot scanner semantics.
15. consent-graph should either add or hand off Cookiebot-style consent banner runtime.
16. consent-graph should either add or hand off Google Consent Mode mapping.
17. consent-graph should either add or hand off IAB TCF string mapping.
18. consent-graph should either add or hand off anonymous visitor consent-state storage.
19. consent-graph should define how anonymous web consent becomes tenant-scoped evidence.
20. consent-graph should define when subject consent becomes DSA revocation evidence.
21. consent-graph should define connector ownership for marketing-system sync.
22. consent-graph should keep B2B agreement enforcement separate from campaign preference sync.
23. consent-graph should keep uniform quality across all tenant classes.
24. consent-graph should express tenant_class as economic and infrastructure policy.
25. consent-graph should not use tenant_class to disable safety, audit, revocation, or sovereignty controls.
26. consent-graph should treat demo_trial caps as capacity caps, not product-depth caps.
27. consent-graph should treat paid controls as contractual capacity and compliance controls.
28. consent-graph should treat revenue_share controls as settlement and margin controls.
29. consent-graph should include deployment_context in benchmark and parity evidence.
30. consent-graph should include tenant_class in benchmark and parity evidence.
31. consent-graph should include counterpart-source confidence for every vendor comparison.
32. consent-graph should not carry over the old Snowflake/Databricks-centered matrix as the final top-three parity proof.
33. consent-graph can still keep Snowflake/Databricks as secondary adjacency comparisons.
34. consent-graph must make OneTrust, TrustArc, and Cookiebot the primary union-coverage bar for this batch.
35. consent-graph has enough product definition to proceed to targeted remediation planning.
36. consent-graph does not have enough deployment evidence to claim all-six-context production readiness.
37. consent-graph has enough SLO documentation to define live benchmark harnesses.
38. consent-graph does not have enough implementation evidence to certify those targets.
39. consent-graph has enough migration evidence for OneTrust and TrustArc starting points.
40. consent-graph lacks a Cookiebot migration path.
41. consent-graph has enough audit-chain design evidence to differentiate from CMP vendors.
42. consent-graph lacks enough web consent UX evidence to replace CMP vendors.
43. consent-graph parity should be described as enforcement-substrate parity plus explicit UX handoffs.
44. consent-graph should not be described as a wholesale OneTrust, TrustArc, or Cookiebot replacement until those handoffs are named.
45. consent-graph's next high-value artifact is an owner map for CMP/preference/DSAR/connector surfaces.
