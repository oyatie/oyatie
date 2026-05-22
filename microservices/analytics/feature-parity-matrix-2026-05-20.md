# Analytics feature-parity matrix — 2026-05-20

Target µservice: `analytics`.
Counterpart-1: Google Analytics 4.
Counterpart-2: Mixpanel.
Counterpart-3: Amplitude.
Purpose: compare the current Oyatie analytics artifact set against the union of the three counterpart product surfaces.
Current local product scope: tenant-facing OLAP warehouse, dashboard query API, audit-log query API, billing rollups, regulator export, ClickHouse CDC/MV pipeline.
Important boundary: this matrix does not treat ClickHouse warehouse primitives alone as parity with product-analytics suites.

Source anchor block:
- Oyatie analytics PRD purpose: `microservices/analytics/PRD.md:8-17`.
- Oyatie analytics contracts: `microservices/analytics/contracts/openapi-v1.yaml:50-178`, `microservices/analytics/contracts/analytics.proto:15-22`, `microservices/analytics/contracts/graphql-v1.sdl:10-30`.
- Oyatie analytics benchmark and migration docs: `microservices/analytics/benchmarks/clickhouse-vs-snowflake-vs-bigquery-vs-druid.md:55-82`, `microservices/analytics/migration-playbooks/from-mixpanel-and-amplitude.md:11-49`, `:113-190`.
- GA4 feature/limit sources: https://support.google.com/analytics/answer/9327972, https://support.google.com/analytics/answer/9317498, https://support.google.com/analytics/answer/9670133, https://developers.google.com/analytics/devguides/reporting/data/v1/quotas, https://support.google.com/analytics/answer/9823238, https://developers.google.com/analytics/devguides/collection/protocol/ga4/sending-events.
- Mixpanel feature/limit sources: https://developer.mixpanel.com/reference/raw-event-export, https://developer.mixpanel.com/reference/import-events, https://developer.mixpanel.com/reference/list-recent-events, https://developer.mixpanel.com/reference/retention-query, https://mixpanel.com/blog/how-funnels-work/, https://mixpanel.com/content/guide-to-product-analytics/chapter_4/.
- Amplitude feature/limit sources: https://amplitude.com/docs/analytics, https://amplitude.com/docs/analytics/charts/funnel-analysis/funnel-analysis-how-amplitude-computes, https://amplitude.com/docs/analytics/charts/retention-analysis/retention-analysis-calculation, https://amplitude.com/docs/analytics/create-cohorts, https://amplitude.com/docs/analytics/dashboard-create, https://amplitude.com/docs/apis/analytics/dashboard-rest, https://amplitude.com/docs/apis/analytics/http-v2, https://amplitude.com/docs/faq/limits.

## §1 Current Oyatie analytics surface

1. Oyatie analytics owns OLAP warehouse queries over tenant business data, not SRE telemetry: `microservices/analytics/PRD.md:8-17`.
2. Oyatie analytics owns tenant dashboard query, billing rollup, audit-log search, and data export bounded contexts: `microservices/analytics/manifest.json:16-21`.
3. Oyatie analytics uses ClickHouse as primary OLAP substrate: `microservices/analytics/manifest.json:22-28`.
4. Oyatie analytics plans per-tenant database isolation and quota control: `microservices/analytics/PRD.md:60-65`.
5. Oyatie analytics exposes REST dashboard endpoints: `microservices/analytics/contracts/openapi-v1.yaml:50-59`.
6. Oyatie analytics exposes REST billing rollup endpoints: `microservices/analytics/contracts/openapi-v1.yaml:92-96`.
7. Oyatie analytics exposes REST audit-log search endpoints: `microservices/analytics/contracts/openapi-v1.yaml:124-132`.
8. Oyatie analytics exposes REST regulator export endpoints: `microservices/analytics/contracts/openapi-v1.yaml:170-178`.
9. Oyatie analytics exposes gRPC query/export/bootstrap operations: `microservices/analytics/contracts/analytics.proto:15-22`.
10. Oyatie analytics exposes GraphQL query fields for dashboard, billing, audit-log, and regulator export: `microservices/analytics/contracts/graphql-v1.sdl:10-30`.
11. Oyatie analytics has dashboard API latency SLOs: `microservices/analytics/slos/dashboard-api-latency.openslo.yaml:13`.
12. Oyatie analytics has audit-log hot/cold latency SLOs: `microservices/analytics/slos/audit-log-query-latency.openslo.yaml:14`, `microservices/analytics/slos/audit-log-query-cold-latency.openslo.yaml:14`.
13. Oyatie analytics has ingest lag SLOs: `microservices/analytics/slos/clickhouse-ingest-lag.openslo.yaml:13-15`.
14. Oyatie analytics has concrete per-cell capacity targets: `microservices/analytics/capacity-model.md:7-20`.
15. Oyatie analytics has a migration playbook for Mixpanel and Amplitude exports: `microservices/analytics/migration-playbooks/from-mixpanel-and-amplitude.md:11-49`.
16. Oyatie analytics has a funnel query reference using ClickHouse primitives: `microservices/analytics/benchmarks/clickhouse-vs-snowflake-vs-bigquery-vs-druid.md:55-67`.
17. Oyatie analytics does not currently provide a service-local product analytics workspace specification.
18. Oyatie analytics does not currently provide service-local event collection SDK ownership.
19. Oyatie analytics does not currently provide service-local audience activation ownership.
20. Oyatie analytics does not currently provide service-local experiment analysis ownership.

## §2 Counterpart 1 — Google Analytics 4 capability surface

1. GA4 provides event collection for web/app streams and Measurement Protocol server-side collection; source: https://developers.google.com/analytics/devguides/collection/protocol/ga4/sending-events.
2. GA4 Measurement Protocol supports batched events with documented request limits; source: https://developers.google.com/analytics/devguides/collection/protocol/ga4/sending-events.
3. GA4 event collection has per-property limits for events, event parameters, and user properties; source: https://support.google.com/analytics/answer/9267744.
4. GA4 provides standard reports for acquisition, engagement, monetization, retention, demographics, and technology surfaces; source: GA4 help center surface family.
5. GA4 provides Explorations as the analyst workspace for free-form, path, funnel, user explorer, segment builder, cohort, segment overlap, and related techniques; source: https://support.google.com/analytics/answer/9327972.
6. GA4 free-form exploration supports tables, donut charts, line charts, scatterplots, bar charts, geo maps, nested rows, segment filters, and audience creation from selected data; source: https://support.google.com/analytics/answer/9327972.
7. GA4 path exploration lets analysts inspect user journeys before or after a chosen event or page/screen; source: https://support.google.com/analytics/answer/9317498.
8. GA4 funnel exploration is part of the Explorations technique set; source: https://support.google.com/analytics/answer/9327972.
9. GA4 cohort exploration groups users by a shared characteristic and tracks return behavior over time; source: https://support.google.com/analytics/answer/9670133.
10. GA4 segment builder and segment overlap are first-class exploration surfaces; source: https://support.google.com/analytics/answer/9327972.
11. GA4 supports audiences and audience creation from explorations; source: https://support.google.com/analytics/answer/9327972.
12. GA4 supports BigQuery Export for raw event data; source: https://support.google.com/analytics/answer/9823238.
13. GA4 BigQuery daily export has a standard-property daily event ceiling and a separate streaming export behavior; source: https://support.google.com/analytics/answer/9823238.
14. GA4 Data API exposes Core, Realtime, and Funnel quota categories; source: https://developers.google.com/analytics/devguides/reporting/data/v1/quotas.
15. GA4 Data API quotas include property tokens, per-project per-property tokens, concurrent requests, server-error caps, and thresholded request limits; source: https://developers.google.com/analytics/devguides/reporting/data/v1/quotas.
16. GA4 supports demographic and interest thresholding controls in Data API quota behavior; source: https://developers.google.com/analytics/devguides/reporting/data/v1/quotas.
17. GA4 has reporting data thresholds that affect privacy-sensitive dimensions; source: GA4 Explorations/help docs.
18. GA4 integrates tightly with Google Ads, Firebase, BigQuery, and Google Cloud data workflows.
19. GA4 product strength: broad collection and marketing attribution ecosystem.
20. GA4 product strength: standard-property BigQuery export gives a common escape hatch for warehouse workflows.
21. GA4 product risk compared with Oyatie target: standard-property export and API quotas can cap high-volume tenants; source: https://support.google.com/analytics/answer/9823238 and https://developers.google.com/analytics/devguides/reporting/data/v1/quotas.
22. GA4 product gap that Oyatie can exploit: first-party tenant-cell residency and open infrastructure proof can be stronger than a third-party SaaS processor model, if Oyatie lands OpenTofu and context evidence.
23. GA4 parity implication: Oyatie analytics needs collection, exploration, audience, and export semantics, not only ClickHouse-backed aggregate APIs.
24. GA4 parity implication: Oyatie analytics should define whether GA4 migration/import is in scope; current migration doc covers Mixpanel and Amplitude but not GA4.

## §3 Counterpart 2 — Mixpanel capability surface

1. Mixpanel is an event-based product analytics platform centered on funnels, retention, cohorts, flows, reports, and dashboards.
2. Mixpanel funnel analysis measures conversion through a sequential user flow; source: https://mixpanel.com/blog/how-funnels-work/.
3. Mixpanel funnels are used for signup, purchase, level-completion, and other conversion outcomes; source: https://mixpanel.com/blog/how-funnels-work/.
4. Mixpanel retention analysis supports "did A and came back to do B" behavior and retained-user cohort creation; source: https://mixpanel.com/content/guide-to-product-analytics/chapter_4/.
5. Mixpanel retention API supports birth and compounded retention types; source: https://developer.mixpanel.com/reference/retention-query.
6. Mixpanel Query API exposes aggregate event counts over minutes, hours, days, weeks, and months; source: https://developer.mixpanel.com/reference/list-recent-events.
7. Mixpanel Query API has documented hourly and concurrent query limits; source: https://developer.mixpanel.com/reference/list-recent-events.
8. Mixpanel Raw Event Export returns raw event JSONL and includes rate/concurrency limits; source: https://developer.mixpanel.com/reference/raw-event-export.
9. Mixpanel Raw Event Export supports event filters, date ranges, gzip, and an optional event limit parameter; source: https://developer.mixpanel.com/reference/raw-event-export.
10. Mixpanel Import Events supports strict validation and returns validation errors for failed records; source: https://developer.mixpanel.com/reference/import-events.
11. Mixpanel Track Events supports API-based event tracking; source: https://developer.mixpanel.com/reference/track-event.
12. Mixpanel has distinct raw export and query APIs, which means migration and warehouse reconciliation need both event fidelity and report parity.
13. Mixpanel public docs and blogs emphasize self-serve analysis for product teams without requiring SQL.
14. Mixpanel product strength: fast funnel and retention workflows for PMs and growth teams.
15. Mixpanel product strength: event import/export APIs are explicit enough for migration playbooks.
16. Mixpanel product risk compared with Oyatie target: API rate limits and third-party processing can be hard for high-volume/regulatory tenants.
17. Mixpanel parity implication: Oyatie’s existing migration playbook is a strong start because it names event-shape conversion and funnel/cohort rewrite work: `microservices/analytics/migration-playbooks/from-mixpanel-and-amplitude.md:11-21`, `:113-168`.
18. Mixpanel parity implication: Oyatie must specify how non-SQL users build funnels, cohorts, and reports if it wants product parity rather than warehouse parity.
19. Mixpanel parity implication: Oyatie must define behavioral cohort persistence and activation semantics.
20. Mixpanel parity implication: Oyatie must define import validation and dedupe semantics beyond raw ClickHouse ingestion.
21. Mixpanel parity implication: Oyatie needs dashboard sharing, permissions, and report lifecycle if the UI surface belongs to analytics.
22. Mixpanel parity implication: Oyatie should offer raw export compatibility or a migration adapter for Mixpanel-style JSONL.
23. Mixpanel parity implication: Oyatie should explicitly measure backfill throughput and failure recovery, because current migration playbook gives numbers but needs source/reproducibility anchors.
24. Mixpanel parity implication: Oyatie should map Mixpanel flows/path analysis either into ClickHouse path queries or into a distinct journey-analysis component.

## §4 Counterpart 3 — Amplitude capability surface

1. Amplitude Analytics positions charts, cohorts, dashboards, journeys, and account-level reporting as product analytics primitives; source: https://amplitude.com/docs/analytics.
2. Amplitude includes event segmentation, funnel analysis, retention analysis, journeys, engagement matrix, and experiment results in chart families; source: https://amplitude.com/docs/analytics.
3. Amplitude cohorts are saved groups of users based on behavior or properties; source: https://amplitude.com/docs/analytics.
4. Amplitude dashboards assemble charts into shareable views for teams; source: https://amplitude.com/docs/analytics/dashboard-create.
5. Amplitude dashboards can include charts, cohort population, session replay, image/video content, official designation, comments, copy, download, export, refresh, and archive flows; source: https://amplitude.com/docs/analytics/dashboard-create.
6. Amplitude funnel analysis supports any-order, this-order, and exact-order conversion logic; source: https://amplitude.com/docs/analytics/charts/funnel-analysis/funnel-analysis-how-amplitude-computes.
7. Amplitude funnels apply segmentation to the first step in a defined way; source: https://amplitude.com/docs/analytics/charts/funnel-analysis/funnel-analysis-how-amplitude-computes.
8. Amplitude retention analysis documents return-on-or-after and return-on retention calculation semantics; source: https://amplitude.com/docs/analytics/charts/retention-analysis/retention-analysis-calculation.
9. Amplitude cohort creation supports Microscope, file import, and chart-based inline cohorts; source: https://amplitude.com/docs/analytics/create-cohorts.
10. Amplitude cohort import has a file-size limit and ID validation semantics; source: https://amplitude.com/docs/analytics/create-cohorts.
11. Amplitude Dashboard REST API exposes chart data in JSON with concurrency, hourly cost model, endpoint costs, and group-by limits; source: https://amplitude.com/docs/apis/analytics/dashboard-rest.
12. Amplitude HTTP V2 API defines upload limits, request-size limits, string lengths, ID minimum lengths, and dedupe guidance; source: https://amplitude.com/docs/apis/analytics/http-v2.
13. Amplitude limits page covers monthly event volume, instrumentation object limits, indexing limits, and overage behavior; source: https://amplitude.com/docs/faq/limits.
14. Amplitude product strength: deep self-serve product analytics UX with many chart types.
15. Amplitude product strength: coherent behavioral cohort and activation model.
16. Amplitude product strength: explicit Dashboard REST API cost model gives measurable parity targets.
17. Amplitude product risk compared with Oyatie target: public SaaS plan limits and agreement-based event volumes can obscure true scale ceilings for enterprise buyers.
18. Amplitude parity implication: Oyatie must define exact funnel semantics, including any-order, ordered, exact-order, exclusion, constant-property, and distribution views if targeting full parity.
19. Amplitude parity implication: Oyatie must define behavioral cohorts and reuse across reports, dashboards, and downstream activation.
20. Amplitude parity implication: Oyatie must define dashboard lifecycle: save, share, official status, comments, export, refresh, archive, and permissioning.
21. Amplitude parity implication: Oyatie must define account-level analytics if B2B tenants are first-class.
22. Amplitude parity implication: Oyatie must define experiment result ingestion or explicitly delegate it.
23. Amplitude parity implication: Oyatie must define session replay relationship or explicitly exclude it.
24. Amplitude parity implication: Oyatie must define governance limits for event types, properties, and user properties.

## §5 Union-coverage matrix

| # | Capability | GA4 | Mixpanel | Amplitude | Current Oyatie evidence | Gap |
|---:|---|---|---|---|---|---|
| 1 | Web event collection | Yes; Measurement Protocol and web tagging sources. | Yes; Track Events API and SDK ecosystem. | Yes; HTTP V2 and SDK ecosystem. | No service-local collection SDK ownership. | P1 product surface gap. |
| 2 | Server-side event collection | Yes; Measurement Protocol. | Yes; Import/Track APIs. | Yes; HTTP V2/Batch APIs. | Outbox to ClickHouse CDC planned: `PRD.md:67-69`. | Needs external-event ingest contract. |
| 3 | Event validation | Partial through GA4 rules and error behavior. | Import strict validation. | HTTP V2 validation responses and ID rules. | No explicit external event validation API. | P2. |
| 4 | Event dedupe | GA4 has event collection behavior but dedupe semantics are narrower. | Deduping via event fields and insert IDs is documented. | Recommends `insert_id` to avoid duplicates. | Outbox event IDs implied by capacity docs. | P2 explicit semantic gap. |
| 5 | Tracking plan governance | GA4 custom dimensions/events limits. | Lexicon/schema governance in platform docs. | Amplitude Data/tracking plan family. | No analytics tracking-plan artifact. | P1 for product parity. |
| 6 | Event type limits | Yes; collection limits doc. | API/project controls. | 2,000 event-types/project limit in docs. | No service-local event taxonomy limit. | P2. |
| 7 | Event property limits | Yes; collection limits doc. | API/schema controls. | 2,000 event properties/project and 1,000 user properties. | No property governance doc. | P2. |
| 8 | User identity model | GA4 client/user IDs. | Distinct ID model. | User ID/device ID/Amplitude ID. | Tenant IDs are strong; user analytics identity not specified. | P1. |
| 9 | Group/account analytics | GA4 has account/property concepts but product account analytics is limited. | Group analytics exists in Mixpanel ecosystem. | Account-level reporting is documented. | Tenant/account concepts exist; B2B account analytics not specified. | P2. |
| 10 | Sessions | Yes. | Product sessions/flows in product surface. | Sessions chart family. | No session model in analytics PRD. | P2. |
| 11 | Standard dashboards | Yes. | Yes. | Yes. | API returns dashboard data; dashboard UI not specified. | P1. |
| 12 | Dashboard sharing | Yes in GA ecosystem. | Yes in product. | Yes with comments/official dashboards. | No sharing lifecycle. | P2. |
| 13 | Dashboard API | GA4 Data API. | Query API. | Dashboard REST API. | REST/gRPC/GraphQL dashboard APIs exist. | Partial parity. |
| 14 | Dashboard p99 objective | Public SaaS internals not generally published. | Public SaaS internals not generally published. | Public SaaS internals not generally published. | p99 <500 ms documented: `PRD.md:34`. | Strong local target. |
| 15 | Free-form exploration | Yes. | Insights/report builder equivalent. | Event segmentation and charts. | No self-serve exploration UI spec. | P1. |
| 16 | SQL access | Via BigQuery export. | Warehouse export/import integrations. | Export/warehouse integrations. | ClickHouse is native substrate. | Oyatie strength, UX gap. |
| 17 | Funnel analysis | Yes. | Yes. | Yes. | ClickHouse `windowFunnel` benchmark exists. | Needs UX and exact semantics. |
| 18 | Ordered funnels | Yes. | Yes. | Yes with order modes. | Benchmark covers 5-stage funnel; contract semantics absent. | P2. |
| 19 | Any-order funnels | Not clearly primary. | Product supports flexible funnels. | Explicit any-order mode. | Not specified. | P2. |
| 20 | Exact-order funnels | Funnel controls vary. | Product supports conversion flow variants. | Explicit exact-order mode. | Not specified. | P2. |
| 21 | Funnel breakdowns | Yes. | Yes. | Yes. | Not specified in APIs. | P2. |
| 22 | Funnel time-to-convert | Yes in product surfaces. | Yes in funnel product docs. | Distribution data in funnel docs. | Not specified. | P2. |
| 23 | Funnel exclusion events | Supported in advanced products. | Supported in product workflows. | Cohort docs mention funnel exclusion effects. | Not specified. | P2. |
| 24 | Retention analysis | Cohort exploration. | Retention report/API. | Retention analysis chart. | No retention API explicitly. | P1. |
| 25 | Birth/cohort retention | Cohort exploration. | Birth retention API. | Cohort-entry retention. | No equivalent. | P1. |
| 26 | Compounded/rolling retention | Cohort exploration options. | Compounded retention API. | Return-on-or-after retention. | No equivalent. | P2. |
| 27 | Cohort creation from chart | Yes from explorations/audiences. | Yes from retention/report points. | Microscope/chart cohort creation. | No equivalent. | P1. |
| 28 | Static cohort import | Audience import ecosystem. | List/cohort tooling. | CSV/text cohort import. | No equivalent. | P2. |
| 29 | Behavioral cohorts | Audiences/segments. | Cohorts. | Behavioral cohorts. | No equivalent. | P1. |
| 30 | Cohort comparison | Segment overlap/audience analysis. | Cohort comparison/reporting. | Cohort comparison docs. | No equivalent. | P2. |
| 31 | Audience activation | Strong Google Ads/Firebase. | Integrations. | Activation and integrations. | No activation ownership. | P1. |
| 32 | Path analysis | GA4 Path exploration. | Flows. | Journeys. | Cross-cell query/funnel docs only. | P1. |
| 33 | Journey visualization | Path exploration. | Flows. | Journeys. | No UI spec. | P1. |
| 34 | User explorer | GA4 User explorer. | User profiles. | User activity/profile APIs. | No user explorer. | P2. |
| 35 | Real-time report | GA4 Realtime API quotas. | Real-time event ingestion and queries in product. | Near-real-time charts via platform. | MV lag p99 <5s; no realtime UX. | Partial. |
| 36 | Realtime API quota model | Yes. | Query limits. | Dashboard REST API limits. | SLOs and capacity model local. | Needs public API quotas. |
| 37 | Big raw export | BigQuery export. | Raw Event Export API. | Export APIs. | Regulator export and data export APIs. | Partial; product raw export contract thin. |
| 38 | Regulator export | Not core product. | Data export/compliance APIs. | DSAR/admin APIs. | Strong regulator export API. | Oyatie strength. |
| 39 | Audit-log analytics | Not primary. | Not primary. | Not primary. | First-class audit-log query API. | Oyatie additive strength. |
| 40 | Billing analytics | Ecommerce/revenue reports. | Revenue events. | Revenue/LTV charts. | Billing rollup API. | Partial. |
| 41 | Revenue-share analytics | Not a standard analytics vendor class. | Not standard. | Not standard. | No tenant_class model. | P2 business-model gap. |
| 42 | Privacy thresholding | GA4 thresholds. | Governance controls. | Governance controls. | Cedar policies and PII docs. | Needs threshold semantics. |
| 43 | PII column authorization | Limited in SaaS UI. | Governance/permissions. | Governance/permissions. | Cedar PII policy files. | Oyatie strength if implemented. |
| 44 | Residency packs | Google regions vary by service. | Managed SaaS constraints. | Managed SaaS constraints. | KR/EU pack overlays. | Needs six-context infra proof. |
| 45 | BYOK allowance | Enterprise-specific. | Enterprise-specific. | Enterprise-specific. | No tenant_class entitlement mapping. | P2. |
| 46 | Compliance packs | Enterprise-specific. | Enterprise-specific. | Enterprise-specific. | Packs in manifest. | Needs tenant_class controls. |
| 47 | First-party warehouse ownership | BigQuery export, not GA-owned warehouse. | Warehouse export/import. | Warehouse export/import. | ClickHouse first-party substrate. | Oyatie strength. |
| 48 | Query budget enforcement | Data API quotas. | Query API limits. | Dashboard REST cost model. | ClickHouse QUOTA planned. | Needs tenant_class rewrite. |
| 49 | API concurrency limits | Yes. | Yes. | Yes. | Not public-contract explicit. | P2. |
| 50 | API hourly limits | Yes. | Yes. | Yes. | Not public-contract explicit. | P2. |
| 51 | Usage-metered billing | Google paid products. | Event-volume pricing. | Event-volume agreements. | Cost budget old class model. | Needs paid/revenue_share model. |
| 52 | demo_trial tenant_class | GA standard property. | Vendor free/paid plan split. | Free/non-paying behavior docs. | Old trial wording, not demo_trial. | P2. |
| 53 | Hard usage caps | GA quotas. | API rate limits. | Limits and overage docs. | Quotas exist but old class model. | Needs tenant_class. |
| 54 | Best-effort SLO class | Not generally contract-stated in docs. | Plan-specific. | Non-paying account blocking. | No demo_trial SLO. | P2. |
| 55 | Contractual SLO class | Enterprise agreements. | Enterprise agreements. | Enterprise agreements. | SLO files exist. | Needs paid entitlement binding. |
| 56 | At-cost substrate class | Not product surface. | Not product surface. | Not product surface. | No revenue_share model. | P2. |
| 57 | Product templates | GA reports/templates. | Templates/examples. | Dashboard templates. | No product templates. | P3. |
| 58 | Analyst onboarding | GA docs. | Mixpanel docs/academy. | Amplitude role guides. | Data engineer onboarding exists. | Needs analyst onboarding. |
| 59 | Data engineer onboarding | BigQuery docs. | Import/export docs. | Data team guides. | Strong local guide. | Existing guide needs class cleanup. |
| 60 | Migration from Mixpanel | Not relevant. | Source platform. | Competitor. | Existing playbook. | Strong but needs class cleanup. |
| 61 | Migration from Amplitude | Not relevant. | Competitor. | Source platform. | Existing playbook. | Strong but needs class cleanup. |
| 62 | Migration from GA4 | Source platform. | Competitor can import. | Competitor can import. | No GA4 playbook. | P2. |
| 63 | Backfill controls | BigQuery/import controls. | Import API. | Backfill docs. | Backfill-replay doc exists. | Needs source-labeled throughput. |
| 64 | Historical replay | Export/import workflows. | Import API. | Backfill API. | Backfill-replay doc exists. | Partial. |
| 65 | Late event handling | GA4 collection/backdating rules. | Import old events. | Backfill guidance. | Not explicit in contracts. | P2. |
| 66 | Event timestamp rules | GA4 timestamp hierarchy. | Event `time`. | Milliseconds since epoch. | Not external-event specific. | P2. |
| 67 | Data freshness | Realtime/Data API/BigQuery export behavior. | Real-time product. | Near-real-time ingestion. | MV p99 targets. | Strong core, UX gap. |
| 68 | Cold-data rehydrate | BigQuery warehouse. | Raw export history. | Export. | Cold audit-log SLO. | Oyatie strength. |
| 69 | Multi-month export | BigQuery/export. | Raw export date ranges. | Export APIs. | Regulator export bulk path. | Partial. |
| 70 | Dashboard comments | Product UI. | Product UI. | Yes. | No UI spec. | P3. |
| 71 | Official dashboards | Product UI. | Product UI. | Yes. | No UI spec. | P3. |
| 72 | Dashboard archive | Product UI. | Product UI. | Yes. | No UI spec. | P3. |
| 73 | Chart catalog | Reports/explorations. | Reports. | 17 chart types. | No chart catalog. | P1. |
| 74 | Event segmentation chart | Reports. | Insights/event breakdown. | Yes. | Basic dashboard API only. | P2. |
| 75 | Engagement matrix | Not primary. | Similar reports. | Yes. | No equivalent. | P3. |
| 76 | Experiment results | Google Optimize retired; GA integrates with experiments ecosystem. | Experiment analysis integrations. | Experiment Results chart family. | No equivalent. | P2. |
| 77 | Session replay | Not core GA4. | Separate integrations. | Session Replay product. | No equivalent. | Explicit out-of-scope decision needed. |
| 78 | Anomaly detection | GA explorations and insights. | Signal analysis. | Anomalies and insights. | Anomaly MV class mentioned. | Partial. |
| 79 | AI-assisted analysis | GA insights. | Emerging product features. | Amplitude AI. | No analytics AI UI. | P3. |
| 80 | Natural language analysis | Google ecosystem. | Product-specific. | Amplitude AI docs. | No equivalent. | P3. |
| 81 | Admin permissions | GA roles. | Project/workspace roles. | Org/project roles. | Cedar policies. | Needs UI/admin docs. |
| 82 | Tenant isolation | Property/project separation. | Project separation. | Org/project separation. | Database-per-tenant design. | Oyatie strength. |
| 83 | Row-level fallback | Not customer-managed. | Not customer-managed. | Not customer-managed. | PRD row-level fallback. | Oyatie strength. |
| 84 | Cross-tenant query denial | Account/property isolation. | Project isolation. | Project isolation. | Cedar and tenant DBs. | Strong if implemented. |
| 85 | Cross-cell federation | BigQuery can query datasets. | Not primary. | Not primary. | Cross-cell federation planned. | Needs ops proof. |
| 86 | Multi-region analytics | Google global products. | Managed SaaS. | Managed SaaS. | `multi-region.md` and pack overlays. | Needs context proof. |
| 87 | On-prem deployment | Not GA4. | Not Mixpanel managed. | Not Amplitude managed. | Intended by master plan, not evidenced. | Oyatie differentiator if landed. |
| 88 | Colo deployment | Not GA4. | Not Mixpanel managed. | Not Amplitude managed. | Intended by master plan, not evidenced. | Oyatie differentiator if landed. |
| 89 | Guest-on-AWS deployment | Not customer-owned GA4. | Not managed self-host. | Not managed self-host. | Not evidenced. | P1. |
| 90 | Guest-on-OCI deployment | Not customer-owned GA4. | Not managed self-host. | Not managed self-host. | Not evidenced. | P1. |
| 91 | Public-cloud Oyatie deployment | Managed SaaS equivalent. | Managed SaaS. | Managed SaaS. | Not evidenced with context IaC. | P1. |
| 92 | Oyatie-as-cloud-provider deployment | Not counterpart. | Not counterpart. | Not counterpart. | Not evidenced. | Strategic differentiator gap. |
| 93 | OpenTofu context IaC | Not counterpart. | Not counterpart. | Not counterpart. | No service-local modules. | Canonical gap. |
| 94 | OS matrix | Client/browser docs. | SDK platform docs. | SDK platform docs. | No supported-oses manifest. | Canonical gap. |
| 95 | Rust backend implementation | Not counterpart. | Not counterpart. | Not counterpart. | No `src/` yet. | Implementation evidence gap. |
| 96 | Compliance evidence export | Enterprise reports. | Enterprise reports. | Enterprise reports. | Regulator export API. | Oyatie strength. |
| 97 | Audit-chain recursive query logging | Not primary. | Not primary. | Not primary. | PRD requires recursive audit. | Oyatie additive strength. |
| 98 | Query explain/debug | BigQuery SQL explain. | Report debugging. | Chart definitions. | Not specified. | P3. |
| 99 | Query cancellation | API/product control. | API/product control. | API/product control. | Not specified. | P3. |
| 100 | Query sampling/disclosure | GA has thresholds/sampling distinctions. | Product semantics. | Product semantics. | No sampling policy. | P2. |
| 101 | Exact vs approximate cardinality | BigQuery functions. | Product abstracts. | Product abstracts. | Benchmark covers HLL/exact. | Strong backend; UX gap. |
| 102 | Data lineage | BigQuery/project lineage. | Data pipelines. | Data governance. | Outbox/MV docs. | Needs UI/evidence. |
| 103 | Schema evolution | Event governance. | Lexicon/schema. | Tracking plans. | Catalog plus SQL templates. | Needs product workflow. |
| 104 | Alerting | GA custom insights. | Alerts/integrations. | Anomaly/insights. | SLO burn alerts planned. | Ops strong; product alerts missing. |
| 105 | Webhook/export activation | Google Ads/Firebase. | Integrations. | Destinations. | No activation webhooks. | P2. |
| 106 | Warehouse import | BigQuery is native export. | Warehouse imports supported. | Warehouse/source integrations. | Outbox ingest, no warehouse import UI. | P2. |
| 107 | Lookup-table enrichment | GA custom dimensions. | Lookup table import endpoint. | Governed properties. | Not explicit. | P3. |
| 108 | Data residency claim | Google managed regions. | Contractual. | Contractual. | Strong intent via packs. | Needs deployment evidence. |
| 109 | Data processor reduction | Third-party by default. | Third-party by default. | Third-party by default. | First-party cell intent. | Oyatie strength if delivered. |
| 110 | Cost transparency | Vendor plans. | Event-volume pricing. | Agreement/event-volume limits. | Cost budget exists. | Needs tenant-class rewrite. |
| 111 | Usage cap transparency | Quotas docs. | Rate limits docs. | Limits docs. | Old quota matrix. | Needs modern model. |
| 112 | Revenue analytics | Ecommerce reports. | Revenue tracking. | Revenue/LTV. | Billing rollup. | Partial. |
| 113 | LTV analysis | GA reports. | Reports. | Retention/LTV chart docs. | No LTV API. | P2. |
| 114 | Attribution | Strong GA surface. | Campaign analytics. | Marketing analytics. | No attribution model. | P1 if analytics owns marketing parity. |
| 115 | Mobile analytics | GA/Firebase. | SDKs. | SDKs. | No mobile event SDK spec. | P2. |
| 116 | Web analytics | GA core. | SDKs. | SDKs. | No web event SDK spec. | P2. |
| 117 | Offline import | GA event import/MP. | Import API. | Batch/backfill APIs. | Backfill docs. | Partial. |
| 118 | Governance warning at limits | GA quota return. | 429 responses. | 80/90/100/110% warnings. | No product-facing warnings. | P2. |
| 119 | Account blocking behavior | GA quota exhaustion. | Rate-limit 429. | Non-paying block/delete rules. | No demo_trial caps. | P2. |
| 120 | Bulk delete/DSR | Google privacy tools. | GDPR APIs. | DSAR APIs. | DSR/offboard docs. | Partial. |
| 121 | Data retention controls | GA retention settings. | Retention by plan. | Plan/event retention. | 7-year audit/billing and TTL. | Strong backend. |
| 122 | Cold storage policy | BigQuery storage. | Vendor-managed. | Vendor-managed. | Hot/cold TTL docs. | Oyatie strength. |
| 123 | Restore drill evidence | Vendor-managed. | Vendor-managed. | Vendor-managed. | Restore drill runbook/spec. | Oyatie strength if evidenced. |
| 124 | Materialized views | BigQuery/materialized tables. | Vendor internals. | Vendor internals. | MV canon exists. | Strong backend. |
| 125 | Capacity sharding | Vendor-managed. | Vendor-managed. | Vendor-managed. | Capacity model and cross-cell. | Strong backend. |
| 126 | Customer-visible capacity | Quotas and plans. | Rate limits. | Limits docs. | Old class matrix. | Needs tenant_class. |
| 127 | Public API schema | Data API. | Developer APIs. | Developer APIs. | REST/gRPC/GraphQL/AsyncAPI. | Strong, but old lifecycle terms. |
| 128 | Error taxonomy | API docs. | API docs. | API docs. | Cedar/quota errors documented. | Partial. |
| 129 | SDK plan | Google SDKs. | SDKs. | SDKs. | SDK plan exists. | Needs Rust-strict/developer-sdk alignment. |
| 130 | SDK runnable examples | Docs/examples. | Docs/examples. | Docs/examples. | Markdown reference only. | P3. |
| 131 | Product docs for PMs | GA Help. | Mixpanel guides. | Amplitude docs. | Data-engineer docs only. | P2. |
| 132 | Product docs for analysts | GA Help. | Mixpanel guides. | Amplitude role guides. | No analyst guide. | P2. |
| 133 | Product docs for engineers | Developer docs. | Developer docs. | Developer docs. | Strong engineering docs. | Partial. |
| 134 | Public benchmarks | Vendor limits public, latencies often private. | Vendor limits public, latencies often private. | Vendor limits public, latencies often private. | Local benchmark has unsourced managed-product numbers. | Needs source labels. |
| 135 | Privacy-preserving aggregate thresholds | GA thresholds. | Governance/permissions. | Governance/permissions. | Cedar but no k-anonymity threshold. | P2. |
| 136 | Per-query audit | Enterprise audit logs. | Audit logs. | Audit logs. | Recursive audit-chain required. | Oyatie strength. |
| 137 | ML/AI insights | GA insights. | Signals. | Amplitude AI. | No AI insight surface. | P3. |
| 138 | Activation destinations | Google ecosystem. | Integrations. | Integrations. | No destination catalog. | P2. |
| 139 | Marketplace seller analytics | Not primary. | Not primary. | Not primary. | Revenue_share class absent. | P2. |
| 140 | Embedded SaaS reseller analytics | Not primary. | Not primary. | Not primary. | Revenue_share class absent. | P2. |

## §6 Family summary

1. Collection family: Oyatie analytics has internal outbox/CDC strength but lacks explicit external web/mobile/server collection ownership.
2. Collection consequence: if another service owns collection, analytics needs a cross-service handoff artifact.
3. Event governance family: GA4, Mixpanel, and Amplitude all expose limits or governance workflows; Oyatie lacks a tracking-plan artifact.
4. Identity family: per-tenant isolation is strong, but end-user product analytics identity is not specified.
5. Dashboard family: Oyatie has APIs and SLOs, but not a self-serve dashboard product lifecycle.
6. Funnel family: Oyatie has an unusually strong ClickHouse compute base, but must specify counterpart semantics.
7. Retention family: Oyatie has no current retention product API, despite retention being central to all three counterparts.
8. Cohort family: Oyatie has no saved behavioral cohort model, which blocks activation and repeated analysis.
9. Journey/path family: Oyatie has no Path/Flows/Journeys equivalent.
10. Export family: Oyatie has strong regulator/export foundations and should add raw-product-analytics export shapes.
11. Compliance family: Oyatie can exceed SaaS counterparts with first-party residency, Cedar, and audit-chain evidence if OpenTofu context proof lands.
12. Deployment family: Oyatie’s differentiator is six-context deployability, but analytics does not yet evidence it.
13. Usage/billing family: counterpart vendors expose quota or event-volume constraints; Oyatie needs tenant_class and usage-meter semantics.
14. Documentation family: engineering docs are broad, but analyst/PM docs are thin.
15. Benchmark family: local ClickHouse numbers are useful, but managed counterpart latency claims need source labels or internal test harness evidence.

## §7 Headline gap analysis

1. P1 headline gap: counterpart surface mismatch; current docs prove OLAP warehouse intent, not full product-analytics suite parity.
2. P1 headline gap: no self-serve analyst workspace specification.
3. P1 headline gap: no collection SDK or event ingestion ownership boundary for web/mobile/server product events.
4. P1 headline gap: no six-context deployment evidence, which weakens Oyatie’s main differentiator against managed SaaS counterparts.
5. P2 headline gap: no behavioral cohort model.
6. P2 headline gap: no retention analysis model.
7. P2 headline gap: no journey/path analysis model.
8. P2 headline gap: no tracking-plan/event-governance product surface.
9. P2 headline gap: no activation destination model.
10. P2 headline gap: no tenant_class semantics for demo_trial, paid, or revenue_share.
11. P2 headline gap: old class vocabulary remains in capacity, contracts, docs, and migration playbooks.
12. P2 headline gap: benchmark docs use managed-product latency/pricing claims without adequate public-source labeling.
13. P2 headline gap: compliance/residency claims lack service-local OpenTofu context modules.
14. P3 headline gap: data-engineer onboarding exists, but PM/analyst onboarding is missing.
15. P3 headline gap: reference implementation is illustrative until SDK lands.

## §8 Additive surface Oyatie can lead on

1. First-party tenant-cell analytics can beat third-party processor posture for regulated tenants if OpenTofu and context modules land.
2. Cedar-authorized PII column access can be more explicit than typical SaaS dashboard permissions.
3. Recursive audit-chain emission for audit-log queries is a strong compliance differentiator.
4. Regulator export as a first-class API can exceed common product analytics vendor surfaces.
5. ClickHouse-backed window funnels can give strong latency and data-residency properties.
6. Database-per-tenant isolation is a strong blast-radius primitive when implemented.
7. Cold retention and restore drill evidence can create stronger compliance proof than vendor-managed opaque storage.
8. Six deployment contexts can differentiate Oyatie from GA4, Mixpanel, and Amplitude, but only after context evidence exists.
9. Revenue_share tenant_class can become a unique analytics billing model for marketplace sellers, B2C operators, embedded SaaS resellers, and affiliate partners.
10. OCI Always Free demo_trial profile can support constrained sample analytics without inventing a lower-quality feature class.
11. Uniform industry-leader quality across tenant classes can avoid the product degradation implied by retired tier models.
12. Open API contracts across REST/gRPC/GraphQL/AsyncAPI can exceed counterpart lock-in if the old lifecycle terms are removed.
13. Rust-strict backend implementation can improve supply-chain control if code lands under service-local crates or shared Rust crates.
14. Product analytics warehouse ownership can be paired with analyst UX through another µservice if ownership handoffs are explicit.
15. GA4 migration support would complete the requested counterpart import story and should sit next to the current Mixpanel/Amplitude playbook.

## §9 Parity remediation backlog

1. Backlog AN-FP-001: define product-analytics ownership boundary for analyst workspace; source gap §5 rows 11-15.
2. Backlog AN-FP-002: define external web event collection boundary; source gap §5 row 1.
3. Backlog AN-FP-003: define external mobile event collection boundary; source gap §5 row 115.
4. Backlog AN-FP-004: define server-side event ingestion contract separate from internal outbox CDC; source gap §5 row 2.
5. Backlog AN-FP-005: define import validation errors for malformed events; source gap §5 row 3.
6. Backlog AN-FP-006: define event dedupe key semantics and retry behavior; source gap §5 row 4.
7. Backlog AN-FP-007: define tracking-plan governance artifact; source gap §5 row 5.
8. Backlog AN-FP-008: define event-type and property-count governance limits; source gap §5 rows 6-7.
9. Backlog AN-FP-009: define end-user identity mapping from tenant principal to product analytics user; source gap §5 row 8.
10. Backlog AN-FP-010: define B2B account/group analytics semantics; source gap §5 row 9.
11. Backlog AN-FP-011: define session model for web, mobile, and server events; source gap §5 row 10.
12. Backlog AN-FP-012: define dashboard save/share/refresh/archive lifecycle; source gap §5 rows 11, 70-72.
13. Backlog AN-FP-013: define dashboard permission model using Cedar and tenant roles; source gap §5 row 81.
14. Backlog AN-FP-014: define dashboard API rate and concurrency limits; source gap §5 rows 49-50.
15. Backlog AN-FP-015: define free-form exploration product requirements; source gap §5 row 15.
16. Backlog AN-FP-016: define SQL access policy for advanced tenants and internal operators; source gap §5 row 16.
17. Backlog AN-FP-017: define funnel query DSL and exact semantics; source gap §5 rows 17-23.
18. Backlog AN-FP-018: define any-order funnel semantics; source gap §5 row 19.
19. Backlog AN-FP-019: define ordered and exact-order funnel semantics; source gap §5 rows 18 and 20.
20. Backlog AN-FP-020: define funnel breakdown and time-to-convert outputs; source gap §5 rows 21-22.
21. Backlog AN-FP-021: define exclusion-event and constant-property funnel handling; source gap §5 row 23.
22. Backlog AN-FP-022: define retention analysis contract; source gap §5 rows 24-26.
23. Backlog AN-FP-023: define birth-retention and returning-event semantics; source gap §5 row 25.
24. Backlog AN-FP-024: define rolling and return-on-or-after retention semantics; source gap §5 row 26.
25. Backlog AN-FP-025: define behavioral cohort persistence; source gap §5 rows 27-30.
26. Backlog AN-FP-026: define static cohort import and validation; source gap §5 row 28.
27. Backlog AN-FP-027: define cohort comparison and overlap reports; source gap §5 row 30.
28. Backlog AN-FP-028: define audience activation ownership or handoff; source gap §5 row 31.
29. Backlog AN-FP-029: define path/journey analysis primitives; source gap §5 rows 32-33.
30. Backlog AN-FP-030: define user explorer and profile-read privacy rules; source gap §5 row 34.
31. Backlog AN-FP-031: define realtime dashboard refresh semantics using MV lag budgets; source gap §5 row 35.
32. Backlog AN-FP-032: define raw event export format and cursoring; source gap §5 row 37.
33. Backlog AN-FP-033: define regulator export and raw export separation; source gap §5 row 38.
34. Backlog AN-FP-034: define revenue and LTV product analytics surfaces; source gap §5 rows 40 and 113.
35. Backlog AN-FP-035: define revenue_share analytics metrics and billing handoff; source gap §5 rows 41 and 139-140.
36. Backlog AN-FP-036: define privacy thresholds and aggregate suppression; source gap §5 rows 42 and 135.
37. Backlog AN-FP-037: define PII column access proof in product analytics reports; source gap §5 row 43.
38. Backlog AN-FP-038: define BYOK and compliance-pack entitlement handoffs; source gap §5 rows 45-46.
39. Backlog AN-FP-039: define query-budget API responses for tenant_class budgets; source gap §5 row 48.
40. Backlog AN-FP-040: define demo_trial analytics caps without reducing quality of implemented features; source gap §5 rows 52-54.
41. Backlog AN-FP-041: define paid tenant usage-meter and contractual SLO binding; source gap §5 row 55.
42. Backlog AN-FP-042: define at-cost substrate accounting for revenue_share customers; source gap §5 row 56.
43. Backlog AN-FP-043: define product templates for common activation, retention, revenue, and compliance dashboards; source gap §5 row 57.
44. Backlog AN-FP-044: write analyst onboarding that does not assume data-engineer privileges; source gap §5 row 58.
45. Backlog AN-FP-045: write PM onboarding for funnels, cohorts, retention, and dashboards; source gap §5 row 131.
46. Backlog AN-FP-046: add GA4 migration playbook beside Mixpanel/Amplitude migration; source gap §5 row 62.
47. Backlog AN-FP-047: source-label backfill throughput and failure recovery claims; source gap §5 row 63.
48. Backlog AN-FP-048: define late-event acceptance and reprocessing windows; source gap §5 rows 65-66.
49. Backlog AN-FP-049: define cold-data rehydrate product behavior and cost exposure; source gap §5 row 68.
50. Backlog AN-FP-050: define multi-month export guardrails for tenant admins and regulators; source gap §5 row 69.
51. Backlog AN-FP-051: define official dashboard governance and stale-dashboard archive policy; source gap §5 rows 71-72.
52. Backlog AN-FP-052: define chart catalog minimum set: segmentation, funnel, retention, journey, revenue, audit, billing, and capacity; source gap §5 rows 73-76.
53. Backlog AN-FP-053: decide whether session replay is explicitly out of scope; source gap §5 row 77.
54. Backlog AN-FP-054: define anomaly detection ownership between analytics, detection, and observability; source gap §5 row 78.
55. Backlog AN-FP-055: decide whether AI-assisted analysis belongs to analytics or intelligence; source gap §5 rows 79-80.
56. Backlog AN-FP-056: define cross-cell federation guardrails for global ops only; source gap §5 row 85.
57. Backlog AN-FP-057: distinguish residency packs from deployable contexts; source gap §5 rows 86-92.
58. Backlog AN-FP-058: add six-context deployability evidence; source gap §5 rows 87-93.
59. Backlog AN-FP-059: add OpenTofu declarations for analytics contexts; source gap §5 row 93.
60. Backlog AN-FP-060: add OS support manifest and package/test evidence links; source gap §5 row 94.
61. Backlog AN-FP-061: land Rust implementation evidence or mark service as specification-only until code exists; source gap §5 row 95.
62. Backlog AN-FP-062: define query explain/debug output for analyst and support users; source gap §5 row 98.
63. Backlog AN-FP-063: define query cancellation and long-running export cancellation; source gap §5 row 99.
64. Backlog AN-FP-064: define sampling, approximation, and threshold disclosure UX; source gap §5 rows 100-101.
65. Backlog AN-FP-065: define event lineage from outbox event to ClickHouse row to dashboard cell; source gap §5 row 102.
66. Backlog AN-FP-066: define schema evolution workflow for events and materialized views; source gap §5 row 103.
67. Backlog AN-FP-067: define product alerting separate from SLO burn alerts; source gap §5 row 104.
68. Backlog AN-FP-068: define activation webhook and destination catalog; source gap §5 rows 105 and 138.
69. Backlog AN-FP-069: define warehouse import from customer-owned data stores; source gap §5 row 106.
70. Backlog AN-FP-070: define lookup-table enrichment support and governance; source gap §5 row 107.
71. Backlog AN-FP-071: bind data-residency claims to OpenTofu evidence and Cedar policy; source gap §5 row 108.
72. Backlog AN-FP-072: define cost transparency report per tenant_class; source gap §5 rows 110-111.
73. Backlog AN-FP-073: define attribution model or route attribution to another service; source gap §5 row 114.
74. Backlog AN-FP-074: define web SDK, mobile SDK, and server SDK boundaries with developer-sdk if applicable; source gap §5 rows 115-117.
75. Backlog AN-FP-075: define product-facing warning thresholds for usage caps; source gap §5 row 118.
76. Backlog AN-FP-076: define account-blocking or throttle behavior for demo_trial overuse; source gap §5 row 119.
77. Backlog AN-FP-077: define DSAR export/delete interaction with ClickHouse tenant databases; source gap §5 row 120.
78. Backlog AN-FP-078: define tenant-facing retention controls and legal-hold exceptions; source gap §5 row 121.
79. Backlog AN-FP-079: define restore-drill evidence visible to paid compliance-pack customers; source gap §5 row 123.
80. Backlog AN-FP-080: expose materialized view lineage and lag in tenant support views; source gap §5 row 124.
81. Backlog AN-FP-081: define customer-visible capacity reports without retired customer classes; source gap §5 row 126.
82. Backlog AN-FP-082: remove old lifecycle vocabulary from AsyncAPI before public contract publication; source gap §5 row 127.
83. Backlog AN-FP-083: align SDK plan with Rust-strict and developer-sdk generation doctrine; source gap §5 row 129.
84. Backlog AN-FP-084: make funnel reference implementation executable after SDK lands; source gap §5 row 130.
85. Backlog AN-FP-085: define benchmark refresh cadence and accepted evidence types; source gap §5 row 134.
86. Backlog AN-FP-086: define destination activation consent and suppression policy; source gap §5 row 138.
87. Backlog AN-FP-087: define marketplace seller analytics for revenue_share tenants; source gap §5 row 139.
88. Backlog AN-FP-088: define embedded SaaS reseller analytics for revenue_share tenants; source gap §5 row 140.
89. Backlog AN-FP-089: decide whether audience activation belongs to analytics, marketing-automation, or both via handoff; source gap §5 row 31.
90. Backlog AN-FP-090: decide whether experiment results belong to analytics, feature-flags, or both via handoff; source gap §5 row 76.
91. Backlog AN-FP-091: decide whether account-level analytics belongs to analytics or CRM surfaces via handoff; source gap §5 row 21.
92. Backlog AN-FP-092: define minimum PM-facing product docs required before parity claim; source gap §5 rows 131-133.
93. Backlog AN-FP-093: define product analytics API versioning and deprecation rules; source gap §5 rows 127-128.
94. Backlog AN-FP-094: define tenant isolation proof artifacts for audits; source gap §5 rows 82-84.
95. Backlog AN-FP-095: define data processor reduction claim only after six-context evidence lands; source gap §5 row 109.
