---
doc_class: PerformanceBenchmarkNumbers
microservice: sheets
audit_date: 2026-05-20
batch: Wave 3 Batch 3.2
status: complete
quality_model: single_industry_leader_target_set
retired_tier_rows: excluded
---

# sheets performance benchmark numbers - 2026-05-20

## Header

- Target microservice: `sheets`.
- Target path: `microservices/sheets/`.
- Counterpart set: Google Sheets, Microsoft Excel Online, Airtable.
- Methodology stance: public limits and quotas are cited directly; local latency targets are cited from local Oyatie specs; competitor latency numbers are treated as internal estimates unless sourced from official public limits.
- Claim boundary: this document does not claim measured Oyatie superiority.
- Claim boundary evidence: local competitor matrix forbids unsupported faster-than-Google claims at `microservices/sheets/competitor-parity-matrix.md:178-188`.
- Quality model: one industry-leader-grade target set.
- Deployment model: the target set is overlaid by deployment context where infrastructure constrains throughput, scale, or elasticity.
- Tenant model from prompt: `demo_trial`, `paid`, `revenue_share`.
- Tenant model caveat from memory: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-20` models `revenue_share` as a paid billing component rather than a separate class.
- Audit handling: this document gives an overlay for `revenue_share` because the user prompt requested it; if the memory correction prevails, reuse the `paid` overlay with `revenue_share` as billing component.
- Existing local benchmark document is not reused as final authority because it contains retired commercial schema rows and unsupported lead claims at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13-57`.

## Five-citation anchor block

- Canonical context source: `specs/master-plan-sequencing.json:704-746`.
- Canonical OpenTofu/source-of-deployment source: `specs/master-plan-sequencing.json:747-775`.
- Canonical OCI Always Free profile source: `specs/master-plan-sequencing.json:857-868`.
- Local Sheets performance target source: `microservices/sheets/PRD.md:82-100` and `microservices/sheets/PRD.md:472-482`.
- Local Sheets capacity source: `microservices/sheets/capacity-model.md:45-121` and `microservices/sheets/capacity-model.md:175-198`.
- Public Google file-limit source: https://support.google.com/drive/answer/37603
- Public Google API-limit source: https://developers.google.com/workspace/sheets/api/limits
- Public Microsoft Excel spec source: https://support.microsoft.com/en-au/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3
- Public Microsoft Excel for web source: https://learn.microsoft.com/en-us/office365/servicedescriptions/office-online-service-description/excel-online
- Public Airtable limits source: https://support.airtable.com/docs/workspace-settings-page-overview

## §1 Methodology

- Benchmark dimensions: file/workbook limits.
- Benchmark dimensions: grid size ceilings.
- Benchmark dimensions: API quotas.
- Benchmark dimensions: request processing timeouts.
- Benchmark dimensions: import/export target latency.
- Benchmark dimensions: sheet-open latency.
- Benchmark dimensions: cell-edit render latency.
- Benchmark dimensions: recalc latency.
- Benchmark dimensions: collaboration cursor sync.
- Benchmark dimensions: concurrent editor sessions.
- Benchmark dimensions: WebSocket fanout.
- Benchmark dimensions: connected-source refresh.
- Benchmark dimensions: chart render.
- Benchmark dimensions: AI formula draft latency.
- Benchmark dimensions: XLSX fidelity.
- Test workload 01: 10k-cell cold sheet open.
- Test workload 02: 10k-cell warm sheet open.
- Test workload 03: 100k-cell workbook open.
- Test workload 04: single cell edit in 100k-cell workbook.
- Test workload 05: 100k-cell dependent recalc cascade.
- Test workload 06: 1M-cell workbook recalc.
- Test workload 07: 100k-cell XLSX import.
- Test workload 08: 100k-cell XLSX export.
- Test workload 09: 10k-row connected-source refresh.
- Test workload 10: concurrent collaboration with 10 editors per workbook.
- Test workload 11: cursor sync across WebSocket gateway.
- Test workload 12: chart render over 100k-cell source range.
- Test workload 13: AI formula draft from short natural-language prompt.
- Test workload 14: smart-fill from 3 seed cells.
- Test workload 15: API read/write quota pressure.
- OS disclosure: current service lacks `microservices/sheets/supported-oses.json`, so OS-specific benchmark claims are not verified.
- OS target: future benchmark runs must cover the Tier-1 OS list from `specs/master-plan-sequencing.json:777-815`.
- Architecture disclosure: current service has no `src/` or `tests/` directory under `microservices/sheets/`, so this document sets targets rather than reporting implementation measurements.
- Deployment-context disclosure: current service lacks context OpenTofu modules, so context overlays are target constraints rather than deployed measurements.
- Tenant-class disclosure: current service lacks `tenant_class` semantics, so class overlays are target constraints rather than implemented limits.
- Public counterpart disclosure: Google, Microsoft, and Airtable publish limits and quotas more consistently than user-visible latency distributions.
- Public counterpart disclosure: where official latency SLOs are not published, this document uses local Oyatie target and local competitor-estimate references only as directional comparison.
- Measurement rule: no future claim may say Oyatie is faster than a counterpart until the benchmark harness, environment, data set, and raw result artifact are landed.
- Harness rule: benchmark code should be Rust-native or an allowlisted frontend test surface; the current PRD JavaScript load-test paths at `microservices/sheets/PRD.md:541-547` need replacement.
- Substrate rule: OpenTofu context modules must exist before deployment-context benchmark overlays can be considered proven.
- Tenant rule: demo/trial usage caps must be expressed as caps, not lower product-quality bars.
- Tenant rule: paid tenants scale with paid capacity, SLO contract, compliance packs, and BYOK eligibility.
- Tenant rule: revenue-share tenants scale at-cost or zero-margin substrate under the prompt model; if corrected under memory, this is a paid billing component.

## §2 Counterpart numbers

### §2.1 Google Sheets numbers

- Google number 01: spreadsheet file cap is 10 million cells for Sheets-created or converted spreadsheets; source: Google Drive file limits.
- Google number 02: column cap is 18,278 columns, through column ZZZ; source: Google Drive file limits.
- Google number 03: imported Excel and CSV spreadsheets have the same 10 million cell or 18,278 column cap; source: Google Drive file limits.
- Google number 04: cell content over 50,000 characters is removed during Excel-to-Sheets conversion; source: Google Drive file limits.
- Google number 05: Connected Sheets pivot table cap is 200k rows; source: Google Drive file limits.
- Google number 06: Connected Sheets extract cap is 500k rows; source: Google Drive file limits.
- Google number 07: Connected Sheets extract cap is also 5 million cells; source: Google Drive file limits.
- Google number 08: Sheets API read request limit is 300 per minute per project; source: Google Sheets API usage limits.
- Google number 09: Sheets API write request limit is 300 per minute per project; source: Google Sheets API usage limits.
- Google number 10: Sheets API read request limit is 60 per minute per user per project; source: Google Sheets API usage limits.
- Google number 11: Sheets API write request limit is 60 per minute per user per project; source: Google Sheets API usage limits.
- Google number 12: recommended maximum API payload is 2 MB; source: Google Sheets API usage limits.
- Google number 13: API processing timeout is 180 seconds; source: Google Sheets API usage limits.
- Google number 14: requests are applied atomically; source: Google Sheets API usage limits.
- Google number 15: no per-day API request cap applies if per-minute quotas are respected; source: Google Sheets API usage limits.

### §2.2 Microsoft Excel Online numbers

- Microsoft number 01: worksheet size cap is 1,048,576 rows by 16,384 columns; source: Microsoft Excel specifications.
- Microsoft number 02: derived maximum grid address space per worksheet is 17,179,869,184 cells; source: derived from Microsoft row and column limits.
- Microsoft number 03: column width cap is 255 characters; source: Microsoft Excel specifications.
- Microsoft number 04: row height cap is 409 points; source: Microsoft Excel specifications.
- Microsoft number 05: page-break cap is 1,026 horizontal and vertical; source: Microsoft Excel specifications.
- Microsoft number 06: cell content cap is 32,767 characters; source: Microsoft Excel specifications.
- Microsoft number 07: header/footer character cap is 255; source: Microsoft Excel specifications.
- Microsoft number 08: line-feed cap per cell is 253; source: Microsoft Excel specifications.
- Microsoft number 09: colors in a workbook are 16 million colors; source: Microsoft Excel specifications.
- Microsoft number 10: unique cell formats/cell styles cap is 65,490; source: Microsoft Excel specifications.
- Microsoft number 11: Excel for the web cannot view SharePoint Online workbooks exceeding 100 MB; source: Microsoft Excel for the web service description.
- Microsoft number 12: Microsoft 365 for web CSPP view/edit limit can be 25 MB or 100 MB depending on CSPP mode; source: Microsoft 365 web file-size FAQ.
- Microsoft number 13: Excel for the web supports most of more than 400 Excel worksheet functions; source: Microsoft Excel for the web service description.
- Microsoft number 14: Excel for the web supports real-time co-authoring; source: Microsoft Excel for the web service description.
- Microsoft number 15: Excel for the web cannot create VBA macros but can open and edit VBA-enabled spreadsheets without removing or corrupting VBA; source: Microsoft Excel for the web service description.

### §2.3 Airtable numbers

- Airtable number 01: Free plan record limit is 1,000 records per base; source: Airtable workspace settings page.
- Airtable number 02: Team plan record limit is 50,000 records per base; source: Airtable workspace settings page.
- Airtable number 03: Business plan record limit is 125,000 records per base; source: Airtable workspace settings page.
- Airtable number 04: Enterprise Scale plan record limit is 500,000+ records per base; source: Airtable workspace settings page.
- Airtable number 05: Free plan attachment storage is 1 GB per base; source: Airtable workspace settings page.
- Airtable number 06: Team plan attachment storage is 20 GB per base; source: Airtable workspace settings page.
- Airtable number 07: Business plan attachment storage is 100 GB per base; source: Airtable workspace settings page.
- Airtable number 08: Enterprise Scale attachment storage is 1,000 GB per base; source: Airtable workspace settings page.
- Airtable number 09: API rate limit is 5 requests/second per base across all plans; source: Airtable workspace settings page.
- Airtable number 10: API pagination cap is 100 records per page; source: Airtable workspace settings page.
- Airtable number 11: API batch size cap is 10 records per request; source: Airtable workspace settings page.
- Airtable number 12: Sync API rate limit is 20 requests per 5 minutes per base; source: Airtable workspace settings page.
- Airtable number 13: Sync API row limit is 10,000 rows per request; source: Airtable workspace settings page.
- Airtable number 14: plan table states 1,000 monthly API calls on Free, 100,000 on Team, and unlimited on Business/Enterprise; source: Airtable workspace settings page.
- Airtable number 15: Airtable plans page states 1,000 tables per base, 1,000 views per base, and 500 fields per table; source: Airtable plans overview.

## §3 Oyatie target numbers - single target set

### §3.1 Canonical target table

| Metric | Canonical target | Local source | Comparison intent |
|---|---:|---|---|
| Sheet-open cold, 10k cells, p50 | 200 ms | `microservices/sheets/PRD.md:82-85` | industry-leader target |
| Sheet-open cold, 10k cells, p95 | 400 ms | `microservices/sheets/PRD.md:82-85` | parity or better versus browser spreadsheet expectations |
| Sheet-open cold, 10k cells, p99 | 600 ms | `microservices/sheets/PRD.md:82-85` | tight tail target |
| Sheet-open cold, 10k cells, p999 | 1.5 s | `microservices/sheets/PRD.md:82-85` | long-tail cap |
| Sheet-open warm p50 | 80 ms | `microservices/sheets/PRD.md:85` | industry-leader target |
| Sheet-open warm p95 | 150 ms | `microservices/sheets/PRD.md:85` | fast resume |
| Sheet-open warm p99 | 250 ms | `microservices/sheets/PRD.md:85` | tail target |
| Cell-edit render p50 | 16 ms | `microservices/sheets/PRD.md:86` | 60fps frame budget |
| Cell-edit render p95 | 30 ms | `microservices/sheets/PRD.md:86` | smooth editing |
| Cell-edit render p99 | 50 ms | `microservices/sheets/PRD.md:86` | hard user-experience target |
| Recalc 100k-cell p50 | 400 ms | `microservices/sheets/PRD.md:87` | industry-leader target |
| Recalc 100k-cell p95 | 1 s | `microservices/sheets/PRD.md:87` | core financial-model target |
| Recalc 100k-cell p99 | 1.5 s | `microservices/sheets/PRD.md:87` | tail target |
| Recalc 1M-cell p50 | 4 s | `microservices/sheets/PRD.md:88` | high-scale workbook target |
| Recalc 1M-cell p95 | 10 s | `microservices/sheets/PRD.md:88` | high-scale workbook target |
| Recalc 1M-cell p99 | 15 s | `microservices/sheets/PRD.md:88` | high-scale tail target |
| Save round-trip p50 | 50 ms | `microservices/sheets/PRD.md:89` | authoring responsiveness |
| Save round-trip p95 | 80 ms | `microservices/sheets/PRD.md:89` | collaboration reliability |
| Save round-trip p99 | 100 ms | `microservices/sheets/PRD.md:89` | audit and persistence bound |
| Cursor sync p50 | 50 ms | `microservices/sheets/PRD.md:90` | collab presence |
| Cursor sync p95 | 100 ms | `microservices/sheets/PRD.md:90` | collab presence |
| Cursor sync p99 | 150 ms | `microservices/sheets/PRD.md:90` | collab presence |
| XLSX export 100k-cell p50 | 2 s | `microservices/sheets/PRD.md:91` | import/export experience |
| XLSX export 100k-cell p95 | 4 s | `microservices/sheets/PRD.md:91` | export target |
| XLSX export 100k-cell p99 | 5 s | `microservices/sheets/PRD.md:91` | export tail |
| XLSX import 100k-cell p50 | 2 s | `microservices/sheets/PRD.md:92` | import target |
| XLSX import 100k-cell p95 | 4 s | `microservices/sheets/PRD.md:92` | import target |
| XLSX import 100k-cell p99 | 5 s | `microservices/sheets/PRD.md:92` | import tail |
| Chart render p50 | 80 ms | `microservices/sheets/PRD.md:93` | chart UX |
| Chart render p95 | 150 ms | `microservices/sheets/PRD.md:93` | chart UX |
| Chart render p99 | 200 ms | `microservices/sheets/PRD.md:93` | chart UX |
| Formula call p50 | 50 microseconds | `microservices/sheets/PRD.md:94` | local formula micro-latency |
| Formula call p95 | 150 microseconds | `microservices/sheets/PRD.md:94` | local formula micro-latency |
| Formula call p99 | 500 microseconds | `microservices/sheets/PRD.md:94` | local formula micro-latency |
| AI formula draft p50 | 1.5 s | `microservices/sheets/PRD.md:95` | foundry-runtime dependent |
| AI formula draft p95 | 2.5 s | `microservices/sheets/PRD.md:95` | foundry-runtime dependent |
| AI formula draft p99 | 3 s | `microservices/sheets/PRD.md:95` | foundry-runtime dependent |
| Smart-fill inference p50 | 200 ms | `microservices/sheets/PRD.md:96` | foundry-runtime dependent |
| Smart-fill inference p95 | 500 ms | `microservices/sheets/PRD.md:96` | foundry-runtime dependent |
| Smart-fill inference p99 | 800 ms | `microservices/sheets/PRD.md:96` | foundry-runtime dependent |
| Connected refresh 10k rows p50 | 1 s | `microservices/sheets/PRD.md:97` | external source dependent |
| Connected refresh 10k rows p95 | 3 s | `microservices/sheets/PRD.md:97` | external source dependent |
| Connected refresh 10k rows p99 | 5 s | `microservices/sheets/PRD.md:97` | external source dependent |
| WebSocket gateway round-trip p50 | 10 ms | `microservices/sheets/PRD.md:98` | collab streaming |
| WebSocket gateway round-trip p95 | 30 ms | `microservices/sheets/PRD.md:98` | collab streaming |
| WebSocket gateway round-trip p99 | 50 ms | `microservices/sheets/PRD.md:98` | collab streaming |
| Active editor sessions per region | 100,000 | `microservices/sheets/PRD.md:99` | regional scale target |
| Concurrent editors per workbook | 10 | `microservices/sheets/PRD.md:100` | collaboration quality cap |
| Concurrent WebSocket connections per cell | 500,000 max | `microservices/sheets/PRD.md:500-509` | scale-out target |
| Recalc invocations/sec cluster-wide | 100,000 max | `microservices/sheets/PRD.md:500-509` | scale-out target |
| XLSX export jobs/sec | 100 max | `microservices/sheets/PRD.md:500-509` | scale-out target |
| AI formula requests/sec | 1,000 max | `microservices/sheets/PRD.md:500-509` | foundry-runtime dependent |

### §3.2 Deployment-context overlay

- `oyatie-public-cloud`: target full elasticity; use canonical targets unchanged when regional capacity is provisioned.
- `oyatie-public-cloud`: OpenTofu must prove regional autoscaling, CDN, WebSocket sharding, recalc worker pool, and storage substrate before the target is claimable.
- `guest-on-aws`: target canonical latency if customer account has equivalent compute, storage, CDN, and network primitives.
- `guest-on-aws`: cap active sessions and recalc throughput to customer-provisioned quota if AWS account limits are lower than public-cloud assumptions.
- `guest-on-oci`: target canonical latency when provisioned above demo/trial shape.
- `guest-on-oci`: OCI Always Free profile constrains throughput to the profile's 4 OCPU, 24 GB RAM, 200 GB block, 2 Autonomous DB instances, 10 Mbps load balancer, and 10 TB egress/month envelope from `specs/master-plan-sequencing.json:857-868`.
- `guest-on-oci`: under the Always Free profile, active editor sessions should be capped by measurement rather than assumed; initial target cap should start at 25-50 concurrent editor sessions until benchmark evidence exists.
- `guest-on-oci`: under the Always Free profile, 1M-cell recalc should be disabled by usage cap or queued best-effort because RAM and CPU contention can break the canonical p95.
- `on-prem`: target canonical latency only after facility bandwidth, storage IOPS, CPU generation, GPU/AI routing, and pack residency are certified.
- `on-prem`: disconnected or restricted network operation may preserve local authoring but degrade connected refresh and AI formula latency.
- `colo`: target canonical latency when facility network and substrate capacity match public-cloud reference.
- `colo`: latency overlay must include facility-specific east-west and north-south path measurements.
- `oyatie-as-cloud-provider`: target canonical latency with strongest elasticity because Oyatie owns substrate primitives.
- `oyatie-as-cloud-provider`: target scale ceilings should be the default claim baseline once OpenTofu modules and benchmark harnesses exist.
- Cross-context invariant: feature quality does not vary by context.
- Cross-context variable: throughput, maximum concurrent users, import/export queue time, and AI/connected-source quotas may vary by infrastructure.
- Cross-context blocker: no local context OpenTofu module exists today, so every overlay is a target, not a measured service claim.

### §3.3 Tenant-class overlay

- `demo_trial`: feature surface remains industry-leader grade.
- `demo_trial`: substrate should default to OCI Always Free profile where feasible.
- `demo_trial`: usage caps should limit workbook count, total cells, imports, exports, AI calls, connected refreshes, and concurrent editors.
- `demo_trial`: SLO should be best-effort.
- `demo_trial`: compliance packs should be unavailable unless explicitly sponsored.
- `demo_trial`: BYOK should be unavailable.
- `demo_trial`: suggested initial cap before measurement is 10 workbooks, 100k cells/workbook, 1M cells/tenant, 5 concurrent editors, 10 imports/day, 10 exports/day, 20 AI calls/day, 20 connected refreshes/day.
- `demo_trial`: suggested API cap should be lower than Google published quotas and Airtable base rate until local controls exist; target 60 read/write requests per minute per tenant as a safe first cap.
- `paid`: feature surface remains industry-leader grade.
- `paid`: paid tenants scale with contractual SLO, purchased capacity, usage-based meters, compliance pack eligibility, and BYOK eligibility.
- `paid`: canonical latency table applies when purchased capacity satisfies the deployment-context overlay.
- `paid`: active sessions, recalc throughput, import/export throughput, AI calls, and connected refreshes scale with payment and technical quota.
- `paid`: suggested initial API cap should meet or exceed Google project quota equivalents for normal workloads, then scale by contract.
- `revenue_share`: feature surface remains industry-leader grade.
- `revenue_share`: under the user-prompt model, substrate runs at-cost or zero-margin while Oyatie takes a share of gross revenue.
- `revenue_share`: performance should match paid when the revenue-share contract funds equivalent substrate.
- `revenue_share`: if revenue-share is demoted to a paid billing component per memory correction, use the paid target table and attach revenue-share billing meters.
- Cross-class invariant: no feature family is removed or degraded by class.
- Cross-class variable: quota, billing, compliance pack access, BYOK, support response, and SLO terms.
- Cross-class blocker: no service-local tenant-class contract exists today.

## §4 Comparison narrative

### §4.1 Sheet size and workbook ceiling

- Google public cap: 10 million cells or 18,278 columns.
- Microsoft public worksheet address space: 1,048,576 rows by 16,384 columns.
- Airtable public base records: 1,000 to 500,000+ records per base by plan.
- Oyatie target: 10 million cells at product scale, with PRD performance targets for 100k and 1M-cell workloads.
- Verdict: parity with Google on cell count target if implemented and measured.
- Verdict: parity with Microsoft row count at 1M-cell recalc only if workbook shapes map to Excel's worksheet semantics.
- Verdict: ahead of Airtable on free-form cell scale, but Airtable is a record/base model rather than a spreadsheet-only model.
- Evidence gap: no implementation source or benchmark harness exists in this service path.

### §4.2 API quotas and request handling

- Google public API quota: 300 read/write requests per minute per project and 60 per user per project.
- Google public payload recommendation: 2 MB.
- Google public timeout: 180 seconds.
- Airtable public API rate: 5 requests/second per base.
- Airtable public batch: 10 records/request and 100 records/page.
- Microsoft Excel Online API quotas were not used as a primary comparison because Graph/Excel API quotas vary by endpoint and tenant policy.
- Oyatie target: no local OpenAPI quota model was found.
- Verdict: catch-up needed; Sheets should publish route-class quotas and tenant-class overlays in contracts.
- Required target: read/write API quota should be at least Google-user-equivalent for paid tenants and capped lower for demo/trial tenants.
- Required target: batch requests should have documented maximum operations, payload size, and atomicity semantics.

### §4.3 Latency targets

- Oyatie sheet-open cold p95 target: 400 ms.
- Oyatie warm open p95 target: 150 ms.
- Oyatie cell-edit-render p99 target: 50 ms.
- Oyatie 100k-cell recalc p95 target: 1 s.
- Oyatie 1M-cell recalc p95 target: 10 s.
- Oyatie cursor sync p99 target: 150 ms.
- Local competitor estimates in `microservices/sheets/competitor-parity-matrix.md:127-139` place Oyatie targets at or above competitor expectations.
- Those local estimates are not a substitute for public benchmark citations or measured local harness output.
- Verdict: target is industry-leader aggressive.
- Verdict: claim status is unproven until benchmark harness and raw artifacts exist.

### §4.4 Import/export and XLSX fidelity

- Microsoft is the native XLSX reference.
- Google has practical import/export at large user scale but not native Excel home-field semantics.
- Airtable maps less cleanly to XLSX because its base/table/field model differs.
- Oyatie target: XLSX import/export 100k-cell p95 at 4 seconds, p99 at 5 seconds.
- Oyatie local matrix says strict OOXML round-trip is a gap at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Verdict: parity target on speed is aggressive.
- Verdict: catch-up needed on strict fidelity versus Microsoft.
- Required target: benchmark must separately report time-to-import, time-to-export, formula preservation, format preservation, chart preservation, pivot preservation, comments preservation, and named-range preservation.

### §4.5 Collaboration scale

- Google and Microsoft have mature real-time co-authoring.
- Airtable has collaborative database-grid editing and comments.
- Oyatie target: 10 concurrent editors per workbook for quality, 100,000 active editor sessions per region, and 500,000 concurrent WebSocket connections per cell at max scale.
- Capacity model says WebSocket connections per pod are 10,000 with 1.5 buffer at `microservices/sheets/capacity-model.md:175-180`.
- PRD says concurrent collab WebSocket connections can reach 500,000 at max per cell at `microservices/sheets/PRD.md:500-509`.
- Verdict: scale target is credible only after deployment-context modules and benchmark harnesses land.
- Verdict: no-silent-loss CRDT invariant is a differentiator if tested.

### §4.6 Connected data and AI workloads

- Google Connected Sheets public limits include 200k pivot rows and 500k rows or 5 million cells for extracts.
- Microsoft web supports many formulas and external-data viewing, but advanced creation workflows often depend on desktop or enterprise integrations.
- Airtable Sync API supports 10,000 rows per request and has a 20 requests per 5 minutes per base limit.
- Oyatie connected refresh target: 10k rows p95 at 3 seconds and p99 at 5 seconds.
- Oyatie AI formula draft target: p95 at 2.5 seconds and p99 at 3 seconds.
- Verdict: connected refresh target is aggressive versus public Airtable Sync API shape and within Google Connected Sheets row-scale envelope.
- Verdict: AI target depends on foundry-runtime and cannot be claimed by Sheets alone.
- Required target: split Sheets-owned latency from foundry-runtime and external-source latency in future measurements.

### §4.7 Cost, deployment, and tenant overlays

- Google, Microsoft, and Airtable publish product-plan limits but do not map to Oyatie's six deployment contexts.
- Oyatie's stricter obligation is to map each benchmark claim to deployment context, OS/arch, and tenant class.
- Current local cost budget still uses older scale labels and per-seat economics at `microservices/sheets/cost-budget.md:93-104`.
- Current local capacity model uses XS/S/M/L/XL labels at `microservices/sheets/capacity-model.md:123-132`.
- These are not the retired commercial tier names, but they still need mapping to deployment-context sizing and tenant-class usage caps.
- Verdict: cost/performance coherence is partial.
- Required target: express all future numbers as canonical target plus context overlay plus tenant-class overlay.

## Stop condition

- Counterpart numbers are sourced from official public pages where available.
- Local target numbers are sourced from the Sheets PRD and capacity model.
- No retired commercial tier headings or rows are used.
- No superiority claim is made without measured evidence.
- Deployment-context and tenant-class overlays are explicit.
- Remaining blocker is implementation and measurement, not audit authorship.
