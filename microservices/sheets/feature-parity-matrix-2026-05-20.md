---
doc_class: FeatureParityMatrix
microservice: sheets
audit_date: 2026-05-20
batch: Wave 3 Batch 3.2
counterparts:
  - Google Sheets
  - Microsoft Excel Online
  - Airtable
status: complete
tier_delta_deliverable: retired
---

# sheets feature-parity matrix - 2026-05-20

## Header

- Target microservice: `sheets`.
- Target path: `microservices/sheets/`.
- Counterpart 1: Google Sheets.
- Counterpart 2: Microsoft Excel Online.
- Counterpart 3: Airtable.
- Purpose: compare the local Sheets product surface to the union coverage bar of the three counterparts.
- Scope: feature parity, feature-family coherence, and additive Oyatie surfaces.
- Excluded: retired commercial tier-delta analysis.
- Quality model: one industry-leader-grade target surface, not feature stratification by retired tiers.
- Deployment-context note: all features below must be evaluated across six deployment contexts once context OpenTofu artifacts exist.
- Tenant-class note: feature quality should remain uniform across tenant classes; usage, SLO, compliance, BYOK, and substrate caps are overlays, not lower feature bars.
- Local source: PRD purpose and requirements at `microservices/sheets/PRD.md:29-76`.
- Local source: competitor matrix at `microservices/sheets/competitor-parity-matrix.md:25-188`.
- Local source: architecture deployment and observability anchors at `microservices/sheets/ARCHITECTURE.md:568-704`.
- Local source: manifest components and dependencies at `microservices/sheets/manifest.json:1-120` and `microservices/sheets/manifest.json:316-423`.
- Chat source: `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311` confirms the target counterpart trio.
- Public Google limit source: Google Drive file limits page says Sheets supports up to 10 million cells or 18,278 columns and Connected Sheets pivot/extract limits.
- Public Google API source: Google Sheets API usage limits page states 300 read/write requests per minute per project, 60 per user per project, 2 MB recommended payload, and 180-second processing timeout.
- Public Microsoft source: Microsoft Excel specifications page states 1,048,576 rows by 16,384 columns and 32,767 characters per cell.
- Public Microsoft web source: Microsoft Excel for the web service description states workbooks above 100 MB cannot be viewed in Excel for the web from SharePoint Online.
- Public Airtable source: Airtable workspace settings page states records/base, storage/base, API rate, pagination, batch, and Sync API limits.

## Counterpart 1 capability surface - Google Sheets

- Google Sheets is the primary benchmark for browser-native spreadsheet collaboration.
- Google surface 01: free-form spreadsheet grid.
- Google surface 02: multi-sheet workbook.
- Google surface 03: formula entry with relative and absolute references.
- Google surface 04: broad function library; local matrix estimates roughly 470 functions at `microservices/sheets/competitor-parity-matrix.md:55`.
- Google surface 05: array formulas.
- Google surface 06: named ranges.
- Google surface 07: drag-fill semantics.
- Google surface 08: conditional formatting.
- Google surface 09: data validation.
- Google surface 10: charts.
- Google surface 11: pivot tables.
- Google surface 12: protected ranges and sheet-level permissions.
- Google surface 13: real-time multi-user editing.
- Google surface 14: cursor or presence indicators.
- Google surface 15: comments and threaded collaboration.
- Google surface 16: version history.
- Google surface 17: import/export for Excel, CSV, TSV, and common sheet formats.
- Google surface 18: best-effort XLSX round-trip, not native Excel-home fidelity.
- Google surface 19: Apps Script automation.
- Google surface 20: Connected Sheets for data-source-backed ranges.
- Google surface 21: Smart Fill and Gemini-assisted productivity surfaces.
- Google surface 22: template gallery and marketplace-adjacent ecosystem.
- Google surface 23: mobile editor clients.
- Google surface 24: web sharing link model.
- Google surface 25: Drive-native storage and folder integration.
- Google surface 26: Google Workspace identity and admin controls.
- Google surface 27: public file cap of 10 million cells or 18,278 columns.
- Google surface 28: public Connected Sheets pivot table cap of 200k rows.
- Google surface 29: public Connected Sheets extract cap of 500k rows or 5 million cells.
- Google surface 30: API rate limit of 300 read requests per minute per project.
- Google surface 31: API rate limit of 300 write requests per minute per project.
- Google surface 32: API per-user quota of 60 read/write requests per minute per project.
- Google surface 33: 2 MB recommended API payload.
- Google surface 34: 180-second API request processing timeout.
- Local Oyatie overlap: PRD covers grid, formulas, collaboration, comments, version history, sharing, import/export, AI, connected sheets, and embed bridge at `microservices/sheets/PRD.md:52-76`.
- Local Oyatie differentiator versus Google: never-silent-loss CRDT invariant is explicitly called out at `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Local Oyatie gap versus Google: Apps-Script-equivalent is deferred at `microservices/sheets/competitor-parity-matrix.md:121-123`.
- Local Oyatie gap versus Google: marketplace template ecosystem is deferred at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Local Oyatie gap versus Google: mobile app editor is deferred at `microservices/sheets/competitor-parity-matrix.md:156-164`.

## Counterpart 2 capability surface - Microsoft Excel Online

- Microsoft Excel Online is the primary benchmark for spreadsheet semantics and XLSX fidelity in a browser.
- Microsoft surface 01: free-form spreadsheet grid.
- Microsoft surface 02: workbook and worksheet model.
- Microsoft surface 03: authoritative Excel formula semantics.
- Microsoft surface 04: very broad function library; local matrix estimates roughly 500 functions at `microservices/sheets/competitor-parity-matrix.md:55`.
- Microsoft surface 05: native XLSX file family.
- Microsoft surface 06: high-fidelity desktop/web bridge.
- Microsoft surface 07: co-authoring in Excel for the web.
- Microsoft surface 08: real-time presence.
- Microsoft surface 09: comments.
- Microsoft surface 10: versioning through Microsoft 365 storage.
- Microsoft surface 11: conditional formatting.
- Microsoft surface 12: data validation.
- Microsoft surface 13: pivot tables in web.
- Microsoft surface 14: charts.
- Microsoft surface 15: named ranges created in desktop and usable in web.
- Microsoft surface 16: Power Query and external-data workflows, with some creation limits in web.
- Microsoft surface 17: Office Scripts and Power Automate adjacency.
- Microsoft surface 18: Copilot-assisted spreadsheet workflows.
- Microsoft surface 19: sensitivity labels and enterprise compliance integration.
- Microsoft surface 20: SharePoint and OneDrive storage integration.
- Microsoft surface 21: desktop fallback for unsupported advanced operations.
- Microsoft surface 22: public worksheet size of 1,048,576 rows by 16,384 columns.
- Microsoft surface 23: public cell text cap of 32,767 characters.
- Microsoft surface 24: public line-feed cap of 253 per cell.
- Microsoft surface 25: public unique cell-format cap of 65,490.
- Microsoft surface 26: Excel for the web SharePoint view limit of 100 MB in the service-description source.
- Microsoft surface 27: Microsoft 365 for web integration limit of 25 MB or 100 MB depending on CSPP mode.
- Microsoft surface 28: desktop client escape hatch for very large workbooks.
- Microsoft surface 29: macro/VBA preservation when opening web workbooks, but web cannot create VBA macros.
- Microsoft surface 30: enterprise admin posture through Microsoft 365.
- Local Oyatie overlap: PRD targets formula conformance, >=400 functions, XLSX import/export, collaboration, charts, pivots, validation, and per-range ACL.
- Local Oyatie differentiator versus Microsoft: per-range Cedar named-ACL is deeper than sheet-level sharing in local matrix at `microservices/sheets/competitor-parity-matrix.md:95-103`.
- Local Oyatie differentiator versus Microsoft: audit-chain Ed25519 seal per cell edit is listed at `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Local Oyatie gap versus Microsoft: strict OOXML round-trip is deferred at `microservices/sheets/competitor-parity-matrix.md:82-93` and `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Local Oyatie gap versus Microsoft: formula library target is lower than Excel's local estimate until the post-M03 catch-up phase.
- Local Oyatie gap versus Microsoft: desktop-class advanced analysis parity is not fully specified.

## Counterpart 3 capability surface - Airtable

- Airtable is the primary benchmark for typed-grid and database-workflow ergonomics.
- Airtable surface 01: base-level application container.
- Airtable surface 02: table-level record model.
- Airtable surface 03: typed fields rather than only free-form cells.
- Airtable surface 04: grid view.
- Airtable surface 05: gallery, calendar, timeline, Kanban, Gantt, and interface views depending on plan.
- Airtable surface 06: forms.
- Airtable surface 07: automations.
- Airtable surface 08: Interface Designer.
- Airtable surface 09: field permissions and sharing controls.
- Airtable surface 10: comments and collaboration.
- Airtable surface 11: revision and snapshot history.
- Airtable surface 12: attachment storage per base.
- Airtable surface 13: REST API.
- Airtable surface 14: metadata API.
- Airtable surface 15: sync API.
- Airtable surface 16: batch record create/update/delete with up to 10 records per request.
- Airtable surface 17: API pagination max 100 records per page.
- Airtable surface 18: API rate limit of 5 requests/second per base.
- Airtable surface 19: API batch handling up to 10 records per request and 50 records/second in the support page strategy.
- Airtable surface 20: Sync API limit of 20 requests per 5 minutes per base.
- Airtable surface 21: Sync API row limit of 10,000 rows per request.
- Airtable surface 22: Free plan record limit of 1,000 records/base.
- Airtable surface 23: Team plan record limit of 50,000 records/base.
- Airtable surface 24: Business plan record limit of 125,000 records/base.
- Airtable surface 25: Enterprise Scale record limit of 500,000+ records/base.
- Airtable surface 26: 1,000 tables per base.
- Airtable surface 27: 1,000 views per base.
- Airtable surface 28: 500 fields per table.
- Airtable surface 29: attachment storage from 1 GB to 1 TB per base depending on plan.
- Airtable surface 30: AI functionality and credits by plan.
- Local Oyatie overlap: PRD typed-column parity flows through ontology descriptors at `microservices/sheets/PRD.md:33` and FR-16/FR-19 at `microservices/sheets/PRD.md:69-72`.
- Local Oyatie overlap: local matrix says typed-column/database-grid parity is via ontology descriptors at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Local Oyatie differentiator versus Airtable: free-form spreadsheet and formula/recalc breadth is substantially broader.
- Local Oyatie differentiator versus Airtable: XLSX import/export is first-class; Airtable maps less cleanly to XLSX according to local benchmark notes at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:70-80`.
- Local Oyatie gap versus Airtable: interface designer, view ecosystem, and marketplace template workflows are not fully specified.
- Local Oyatie gap versus Airtable: no explicit base/table/field API shape is exposed in the service contract.

## Union-coverage matrix

| # | Capability | Google Sheets | Microsoft Excel Online | Airtable | Oyatie local status | Evidence |
|---|---|---|---|---|---|
| 1 | Free-form spreadsheet grid | yes | yes | partial | covered | `microservices/sheets/PRD.md:52-58` |
| 2 | Multi-sheet workbook model | yes | yes | no | covered | `microservices/sheets/PRD.md:29-35` |
| 3 | Typed columns | limited | limited | yes | planned via ontology | `microservices/sheets/PRD.md:33` |
| 4 | Formula entry | yes | yes | limited | covered | `microservices/sheets/PRD.md:54-57` |
| 5 | Formula library breadth | high | highest | low | target >=400 | `microservices/sheets/competitor-parity-matrix.md:52-60` |
| 6 | Excel-reference conformance | partial | native | no | target via corpus | `microservices/sheets/competitor-parity-matrix.md:55-56` |
| 7 | Drag-fill | yes | yes | yes | covered | `microservices/sheets/PRD.md:55` |
| 8 | Relative/absolute references | yes | yes | limited | covered | `microservices/sheets/PRD.md:55` |
| 9 | Named ranges | yes | yes | no | covered | `microservices/sheets/PRD.md:71` |
| 10 | Array formulas | yes | yes | no | planned | `microservices/sheets/competitor-parity-matrix.md:59` |
| 11 | Incremental recalc | proprietary | proprietary | no | target | `microservices/sheets/PRD.md:57` |
| 12 | Parallel recalc | opaque | opaque | no | target | `microservices/sheets/PRD.md:41` |
| 13 | Conditional formatting | yes | yes | partial | covered | `microservices/sheets/PRD.md:60` |
| 14 | Data validation | yes | yes | yes | covered | `microservices/sheets/PRD.md:63` |
| 15 | Pivot tables | yes | yes | partial | covered | `microservices/sheets/PRD.md:61` |
| 16 | Charts | yes | yes | partial | covered | `microservices/sheets/PRD.md:62` |
| 17 | Sparklines | yes | yes | limited | included in chart target | `microservices/sheets/competitor-parity-matrix.md:79` |
| 18 | Real-time co-editing | yes | yes | yes | covered | `microservices/sheets/PRD.md:59` |
| 19 | CRDT no-silent-loss invariant | no contracted claim | no contracted claim | no contracted claim | differentiator | `microservices/sheets/competitor-parity-matrix.md:166-176` |
| 20 | Cursor presence | yes | yes | yes | covered | `microservices/sheets/competitor-parity-matrix.md:68` |
| 21 | Comments | yes | yes | yes | covered | `microservices/sheets/PRD.md:65` |
| 22 | Threaded notes | yes | yes | yes | covered | `microservices/sheets/PRD.md:65` |
| 23 | Version history | yes | yes | yes | covered | `microservices/sheets/PRD.md:66` |
| 24 | Offline buffer | limited | desktop-dependent | limited | covered | `microservices/sheets/PRD.md:74` |
| 25 | Protected ranges | yes | partial | field permissions | per-range ACL target | `microservices/sheets/PRD.md:64` |
| 26 | Per-range Cedar policy | no | no | no | differentiator | `microservices/sheets/competitor-parity-matrix.md:166-176` |
| 27 | Data-class markers | no | partial sensitivity | no | differentiator | `microservices/sheets/PRD.md:70` |
| 28 | OIDC tenant scope | Workspace | M365 | Airtable account | covered | `microservices/sheets/PRD.md:102-115` |
| 29 | Per-seat license gate | account-level | account-level | workspace/base billing | covered | `microservices/sheets/PRD.md:73` |
| 30 | Billing tenant classes | external product model | external product model | plan model | missing | no tenant-class search hits |
| 31 | XLSX import | yes | native | no direct semantic match | covered | `microservices/sheets/PRD.md:58` |
| 32 | XLSX export | yes | native | CSV/Excel export only | covered best-effort | `microservices/sheets/competitor-parity-matrix.md:82-93` |
| 33 | Strict OOXML round-trip | no | yes | no | gap | `microservices/sheets/competitor-parity-matrix.md:156-164` |
| 34 | ODS import/export | partial | partial | no | covered | `microservices/sheets/competitor-parity-matrix.md:89` |
| 35 | CSV/TSV import/export | yes | yes | yes | covered | `microservices/sheets/competitor-parity-matrix.md:90` |
| 36 | JSON sheet model | partial | no | API JSON | covered | `microservices/sheets/competitor-parity-matrix.md:91` |
| 37 | Upload AV scan | opaque | opaque | not core | differentiator | `microservices/sheets/PRD.md:111` |
| 38 | gVisor import sandbox | opaque | opaque | not core | differentiator | `microservices/sheets/PRD.md:111` |
| 39 | Apps Script equivalent | yes | Office Scripts partial | scripting/automations | gap | `microservices/sheets/competitor-parity-matrix.md:121-123` |
| 40 | Workflow trigger bridge | Apps Script | Power Automate | automations | covered | `microservices/sheets/PRD.md:72` |
| 41 | Connected data | Connected Sheets | Power Query | sync/API | covered | `microservices/sheets/PRD.md:69` |
| 42 | External SQL source materialization | yes | yes | via integrations | covered | `microservices/sheets/PRD.md:46` |
| 43 | Prose-to-formula | Gemini | Copilot | Airtable AI | covered | `microservices/sheets/PRD.md:67` |
| 44 | Smart-fill from examples | Smart Fill | Flash Fill/Copilot | partial | covered | `microservices/sheets/PRD.md:68` |
| 45 | LLM prompt validation | opaque | opaque | opaque | differentiator | `microservices/sheets/competitor-parity-matrix.md:109-115` |
| 46 | EU AI Act bounded scope | opaque | partial | opaque | covered in intent | `microservices/sheets/competitor-parity-matrix.md:109-115` |
| 47 | Embed into docs | yes | yes | embed views | covered | `microservices/sheets/PRD.md:75` |
| 48 | Embed charts into slides | yes | yes | not core | covered | `microservices/sheets/PRD.md:75` |
| 49 | Template marketplace | yes | yes | yes | gap | `microservices/sheets/competitor-parity-matrix.md:156-164` |
| 50 | Mobile editor | yes | yes | yes | gap | `microservices/sheets/competitor-parity-matrix.md:156-164` |
| 51 | Public workbook sharing | yes | yes | yes | covered | `microservices/sheets/PRD.md:64` |
| 52 | Audit evidence export | Workspace logs | M365 logs | activity logs | covered | `microservices/sheets/PRD.md:76` |
| 53 | Cell-edit cryptographic seal | no public equivalent | no public equivalent | no public equivalent | differentiator | `microservices/sheets/competitor-parity-matrix.md:166-176` |
| 54 | Region/pack residency markers | Workspace/data regions | M365 data residency | Enterprise controls | covered in policy | `microservices/sheets/PRD.md:135-138` |
| 55 | Cross-pack residency enforcement | limited | limited | limited | covered in policy | `microservices/sheets/policy/data-residency.md:24-30` |
| 56 | Tenant-class usage caps | product plans | product plans | product plans | missing | §3.4.C in coherence audit |
| 57 | OCI Always Free profile | not applicable | not applicable | not applicable | missing | absent `microservices/sheets/iac/oci-guest/always-free/` |
| 58 | Six-context deployability | SaaS only | SaaS/private variants | SaaS only | missing evidence | absent context OpenTofu dirs |
| 59 | OS support manifest | vendor-controlled | vendor-controlled | vendor-controlled | missing | absent `supported-oses.json` |
| 60 | Rust-strict implementation | not applicable | not applicable | not applicable | unproven | no `src/`, no `tests/`, no forbidden files |
| 61 | WebSocket collab scale | yes | yes | yes | targeted | `microservices/sheets/PRD.md:500-509` |
| 62 | Import supply-chain control | opaque | opaque | limited | targeted | `microservices/sheets/PRD.md:111` |
| 63 | Interface designer | no | no | yes | gap | Airtable surface comparison |
| 64 | Multi-view database UX | limited | limited | yes | partial | ontology descriptor route |
| 65 | Record API ergonomics | Sheets API | Graph/Excel APIs | Airtable API | partial | contracts need table/base shape |
| 66 | API quotas declared | yes | Graph quotas external | yes | not declared | local OpenAPI lacks quota model |
| 67 | Public limits declared | yes | yes | yes | local targets only | performance deliverable covers |
| 68 | Cost/billing model clarity | Workspace | M365 | Airtable plans | partial | per-seat evidence but no tenant class |
| 69 | Compliance packs | Workspace | M365 | Enterprise | planned | `microservices/sheets/compliance.md` |
| 70 | BYOK eligibility | Workspace/M365 enterprise | M365 enterprise | enterprise | not expressed by tenant class | tenant-class gap |

## Family summary - spreadsheet core

- Spreadsheet core includes grid, workbook model, formulas, references, recalc, named ranges, formatting, charts, pivots, validation, and import/export.
- Google provides the strongest browser-native collaboration model.
- Microsoft provides the strongest formula and XLSX home-field model.
- Airtable is weaker on free-form spreadsheet semantics but stronger on typed-grid app workflows.
- Oyatie covers spreadsheet core in intent through PRD FR-01 through FR-10.
- Oyatie covers formula breadth with target >=400 functions at `microservices/sheets/PRD.md:56`.
- Oyatie covers recalc with incremental dependency graph at `microservices/sheets/PRD.md:57`.
- Oyatie covers performance targets at `microservices/sheets/PRD.md:82-100`.
- Oyatie still needs measured evidence before claiming superiority.
- Oyatie still needs strict OOXML catch-up before matching Microsoft's deepest workbook fidelity.
- Oyatie still needs the exact function-compatibility corpus and implementation source.
- Core family verdict: competitive target, implementation evidence not present in this service path.

## Family summary - collaboration and sharing

- Collaboration includes real-time edit merge, cursor presence, comments, notes, version history, permissions, and conflict recovery.
- Google and Microsoft provide mature real-time editing.
- Airtable provides collaboration around records, views, and comments.
- Oyatie's local differentiator is the never-silent-loss CRDT invariant.
- Local evidence: PRD requires CRDT merge without silent loss at `microservices/sheets/PRD.md:59`.
- Local evidence: PRD requires comments, version history, sharing, and per-range ACL at `microservices/sheets/PRD.md:64-66`.
- Local evidence: local matrix marks competitor last-writer-wins fallback versus Oyatie no-silent-loss at `microservices/sheets/competitor-parity-matrix.md:62-70`.
- Gap: no implementation tests are present.
- Gap: WebSocket gateway deployment is Helm/Kustomize-only and not mapped to six contexts.
- Collaboration family verdict: strongest differentiator in the spec, but still unproven.

## Family summary - typed-grid and database workflows

- Typed-grid includes field types, views, forms, automations, interfaces, API-first records, and business app ergonomics.
- Airtable leads this family.
- Google and Microsoft cover parts of it through data validation, tables, scripts, forms, and connected data.
- Oyatie routes typed-grid semantics through ontology descriptors.
- Local evidence: PRD says Sheets consumes ontology object-type descriptors for typed-column configuration at `microservices/sheets/PRD.md:33`.
- Local evidence: FR-16 and FR-19 cover connected sheets and workflow trigger bridge at `microservices/sheets/PRD.md:69-72`.
- Gap: no explicit Airtable-like base/table/field/view contract exists in OpenAPI.
- Gap: no interface-designer parity is specified.
- Gap: no form-builder ownership is inside `sheets`; forms appears as a dependency and a sibling flow.
- Typed-grid verdict: partial coverage; likely needs a deliberate typed-table submodel or clear reliance on ontology/forms/workflow services.

## Family summary - AI and automation

- AI includes formula drafting, smart-fill, anomaly detection, validation, governance, and provider-routing.
- Google uses Gemini and Smart Fill surfaces.
- Microsoft uses Copilot and broader Microsoft 365 AI surfaces.
- Airtable includes AI credits and AI functionality by plan.
- Oyatie uses foundry-runtime bridge and Cedar review.
- Local evidence: PRD FR-14 and FR-15 cover AI-formula and smart-fill at `microservices/sheets/PRD.md:67-68`.
- Local evidence: AI safety and validation are described in PRD security controls at `microservices/sheets/PRD.md:110`.
- Local evidence: AI parity and differentiators are listed at `microservices/sheets/competitor-parity-matrix.md:105-115`.
- Gap: tenant-class semantics do not state whether demo/trial gets hard AI usage caps and paid gets usage-based scaling.
- Gap: no event-meter contract exists for AI billing.
- Gap: no local implementation source or tests are present.
- AI family verdict: strong governance intent; billing and measured behavior missing.

## Family summary - import/export and fidelity

- Import/export includes XLSX, ODS, CSV, TSV, JSON-sheet, AV scan, sandboxing, and round-trip guarantees.
- Microsoft is the strictest XLSX reference.
- Google is important for practical XLSX interop but is not native Excel.
- Airtable maps to CSV/Excel export but is semantically a typed database-grid product.
- Oyatie targets best-effort XLSX fidelity.
- Local evidence: PRD FR-05 covers JSON-Sheet and XLSX import/export at `microservices/sheets/PRD.md:58`.
- Local evidence: PRD security covers gVisor plus ClamAV/OPSWAT scanning at `microservices/sheets/PRD.md:111`.
- Local evidence: local matrix marks strict OOXML as deferred at `microservices/sheets/competitor-parity-matrix.md:82-93`.
- Gap: strict OOXML not reached.
- Gap: local benchmark claims 96 percent fidelity but lacks the referenced harness in inventory.
- Gap: legal/privacy implications of export are covered in DPIA but sign-offs are pending.
- Fidelity verdict: defensible if framed as best-effort; not parity with native Excel until strict round-trip evidence exists.

## Family summary - platform, deployment, and governance

- Platform family includes deployment contexts, IaC, OS support, runtime language, OCI Always Free, tenant classes, compliance, observability, and runbooks.
- Google, Microsoft, and Airtable are largely SaaS services; Oyatie's bar is broader because it must support six deployment contexts.
- Local evidence: canonical contexts are in `specs/master-plan-sequencing.json:704-746`.
- Local evidence: current Sheets IaC has Helm/Kustomize only.
- Gap: no per-context OpenTofu modules.
- Gap: no OS support manifest.
- Gap: no OCI Always Free profile.
- Gap: no tenant-class semantics.
- Local strength: compliance, DPIA, threat model, incident response, failure modes, runbooks, and SLO files are broad.
- Governance family verdict: documentation depth is high but canonical machine-readable controls are incomplete.

## Headline gap analysis

- Gap 01: six-context deployability is not evidenced.
- Severity: P1 in coherence audit.
- Evidence: no context-specific OpenTofu directories under `microservices/sheets/iac/`.
- Counterpart implication: Oyatie's deployment promise is broader than the SaaS-only counterpart bar, so missing context evidence is a product-level gap.
- Gap 02: OpenTofu-only IaC is not satisfied.
- Severity: P1 in coherence audit.
- Evidence: `microservices/sheets/compliance.md:83` cites a Terraform path.
- Counterpart implication: IaC is not a direct user-facing counterpart feature, but it is canonical Oyatie product doctrine.
- Gap 03: OS support is absent.
- Severity: P1 in coherence audit.
- Evidence: `supported-oses.json` is absent.
- Counterpart implication: on-prem and colo buyers will require OS support clarity before accepting spreadsheet workloads.
- Gap 04: tenant-class adoption is absent.
- Severity: P2 in coherence audit.
- Evidence: no `tenant_class`, `demo_trial`, or `revenue_share` matches.
- Counterpart implication: Google/Microsoft/Airtable all express commercial usage boundaries; Oyatie currently has per-seat enforcement but not the replacement tenant model.
- Gap 05: retired tier language remains in local docs.
- Severity: P2 in coherence audit.
- Evidence: 42 token matches across 33 cited lines in §3.4.T of the coherence audit.
- Counterpart implication: it creates the false impression that product quality varies by tier.
- Gap 06: strict OOXML fidelity is not achieved.
- Severity: product parity gap.
- Evidence: local matrix defers strict OOXML at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Counterpart implication: Microsoft Excel Online remains ahead on native XLSX fidelity.
- Gap 07: Apps-Script-equivalent automation is deferred.
- Severity: product parity gap.
- Evidence: local matrix marks it absent at `microservices/sheets/competitor-parity-matrix.md:121-123`.
- Counterpart implication: Google remains ahead on spreadsheet-native scripting.
- Gap 08: mobile editor is deferred.
- Severity: product parity gap.
- Evidence: local matrix lists mobile app editor as a gap at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Counterpart implication: all three counterparts have mature mobile access patterns.
- Gap 09: template marketplace is deferred.
- Severity: ecosystem parity gap.
- Evidence: local matrix lists marketplace template ecosystem as a gap at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Counterpart implication: Google, Microsoft, and Airtable all benefit from templates and ecosystem distribution.
- Gap 10: benchmark superiority claims lack measured local evidence.
- Severity: P2 in coherence audit.
- Evidence: local benchmark doc asserts lead positions, while local claim-boundary rules forbid unsupported speed claims.

## Additive Oyatie surface

- Additive 01: never-silent-loss CRDT invariant.
- Evidence: `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Product value: spreadsheet collaboration becomes a contractual reliability property, not merely a UX best effort.
- Additive 02: per-range named-ACL in Cedar.
- Evidence: `microservices/sheets/PRD.md:64` and `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Product value: sensitive columns and ranges can be governed below workbook level.
- Additive 03: data-class markers on cells.
- Evidence: `microservices/sheets/PRD.md:70`.
- Product value: users see PII/PHI/SECRET risk before sharing or export.
- Additive 04: cryptographic audit-chain seal per cell edit.
- Evidence: `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Product value: high-assurance auditability for financial, healthcare, and regulated workflows.
- Additive 05: gVisor plus ClamAV/OPSWAT import sandboxing.
- Evidence: `microservices/sheets/PRD.md:111`.
- Product value: upload supply-chain defense is explicit and testable.
- Additive 06: hybrid Postgres plus Arrow/Parquet large-sheet substrate.
- Evidence: `microservices/sheets/PRD.md:491-495`.
- Product value: hot authoring and cold analytical blocks can be optimized separately.
- Additive 07: foundry-runtime connected-source bridge.
- Evidence: `microservices/sheets/PRD.md:69` and `microservices/sheets/PRD.md:346-359`.
- Product value: connected data can be governed through Oyatie's broader runtime and policy plane.
- Additive 08: workflow-engine trigger bridge.
- Evidence: `microservices/sheets/PRD.md:72`.
- Product value: cell edits can become governed workflow events.
- Additive 09: docs/slides live embed bridge.
- Evidence: `microservices/sheets/PRD.md:75`.
- Product value: Sheets becomes a structured data source for document and presentation surfaces.
- Additive 10: tenant-pack data residency through policy.
- Evidence: `microservices/sheets/PRD.md:135-138` and `microservices/sheets/policy/data-residency.md:24-30`.
- Product value: regional compliance is built into workbook state, edit logs, AI prompts, and connected results.

## Union coverage verdict

- Oyatie Sheets has a credible intended union surface for the spreadsheet core.
- Oyatie Sheets has a credible intended union surface for collaboration.
- Oyatie Sheets has partial intended union surface for Airtable-style typed-grid workflows.
- Oyatie Sheets has a strong additive governance story.
- Oyatie Sheets is behind Microsoft on strict XLSX fidelity.
- Oyatie Sheets is behind Google on spreadsheet-native scripting and ecosystem maturity.
- Oyatie Sheets is behind Airtable on interface designer and mature multi-view database app ergonomics.
- Oyatie Sheets is missing canonical deployment-context evidence.
- Oyatie Sheets is missing canonical OS support evidence.
- Oyatie Sheets is missing tenant-class adoption.
- Oyatie Sheets has retired commercial tier debt in service-local docs.
- Final feature-parity decision: do not downgrade the product concept; fix the canonical control surfaces and evidence gaps before claiming parity or superiority.
