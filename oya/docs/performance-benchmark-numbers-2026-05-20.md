# Wave 3 Batch 3.2 Performance Benchmark Numbers - docs

Audit date: 2026-05-20.
Target microservice: `docs`.
Benchmark scope: collaborative document authoring, review, sharing, import/export, block rendering, API automation, and compliance-sensitive operations.
Counterpart set: Google Docs, Microsoft Word Online, Notion Docs.
Tier posture: this document uses one industry-leader target set, plus deployment-context and tenant-class overlays.
No demo_trial/paid/paid/compliance_pack benchmark rows are defined.
Tenant-class posture: `demo_trial`, `paid`, and `revenue_share` receive the same quality bar; caps and contractual rights differ.
Local source 1: `microservices/docs/PRD.md` lines 67-77 provide existing latency targets.
Local source 2: `microservices/docs/PRD.md` lines 294-301 provide existing benchmark commands.
Local source 3: `microservices/docs/PRD.md` lines 313-319 provide existing per-cell capacity envelope.
Local source 4: `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md` lines 11-14, 22-25, and 33-37 provide prior internal browser-harness counterpart estimates.
Local source 5: `microservices/docs/coherence-audit-2026-05-20.md` §3.4.T catalogs the retired schema rows that must not be copied forward.
External source 1: Google Docs API usage limits, https://developers.google.com/docs/api/limits.
External source 2: Google Docs sharing help, https://support.google.com/docs/answer/2494822.
External source 3: Google Docs version history help, https://support.google.com/docs/answer/190843.
External source 4: Microsoft Word real-time coauthoring help, https://support.microsoft.com/en-gb/office/collaborate-on-word-documents-with-real-time-co-authoring-7dd3040c-3f30-4fdd-bab0-8586492a1f1d.
External source 5: Microsoft Graph throttling limits, https://learn.microsoft.com/en-us/graph/throttling-limits.
External source 6: Notion API request limits, https://developers.notion.com/reference/request-limits.
External source 7: Notion append block children API, https://developers.notion.com/reference/patch-block-children.
Methodology disclosure: public vendor latency is incomplete, so public hard numbers are labeled as public-source facts and latency numbers are labeled as internal-estimated where they come from the existing browser-harness benchmark.
Verdict: Oyatie targets can be industry-leader-grade, but current local evidence must be recast away from retired tiers and tied to deployment and tenant-class constraints.

## §1 Methodology

- Benchmark dimension M-001: editor keystroke-to-ack latency.
- Benchmark dimension M-002: cursor/presence propagation latency.
- Benchmark dimension M-003: cold document open latency.
- Benchmark dimension M-004: warm document open latency.
- Benchmark dimension M-005: save/commit latency.
- Benchmark dimension M-006: search-within-document latency.
- Benchmark dimension M-007: comment post latency.
- Benchmark dimension M-008: suggested-edit accept/reject latency.
- Benchmark dimension M-009: share ACL enforcement correctness.
- Benchmark dimension M-010: version restore latency.
- Benchmark dimension M-011: PDF export latency for a 50-page document.
- Benchmark dimension M-012: DOCX export latency for a 50-page document.
- Benchmark dimension M-013: DOCX import latency for a 50-page document.
- Benchmark dimension M-014: attachment upload latency for a 10 MB attachment.
- Benchmark dimension M-015: concurrent editor ceiling per document.
- Benchmark dimension M-016: concurrent editor sessions per cell.
- Benchmark dimension M-017: edits per second per cell.
- Benchmark dimension M-018: export jobs per cell.
- Benchmark dimension M-019: import jobs per cell.
- Benchmark dimension M-020: embed refresh throughput.
- Benchmark dimension M-021: API read quota compatibility.
- Benchmark dimension M-022: API write quota compatibility.
- Benchmark dimension M-023: block append batch size.
- Benchmark dimension M-024: payload size handling for API writes.
- Benchmark dimension M-025: compliance-sensitive audit event latency.
- Test workload W-001: 10-page doc, 2 editors, 1 commenter, plain text and lists.
- Test workload W-002: 100-page doc, 25 editors, images, tables, comments, and suggestions.
- Test workload W-003: 1,000-block Notion-style page with embeds and cross-document references.
- Test workload W-004: 50-page DOCX import/export corpus with comments, tables, images, headings, and citations.
- Test workload W-005: legal-hold document with version history, comments, suggestions, and audit-chain seal.
- Test workload W-006: editor-session storm with 100, 500, 1,000, and 10,000 simulated collaborators.
- Test workload W-007: demo_trial cap test using OCI Always Free profile constraints.
- Test workload W-008: paid tenant scale test using elastic public-cloud or paid customer substrate.
- Test workload W-009: revenue_share tenant at-cost substrate test using the same quality targets and settlement meters.
- OS disclosure: current docs path lacks `supported-oses.json`; the target matrix must later cover the canonical OS list from `specs/master-plan-sequencing.json` lines 777-815.
- Architecture disclosure: benchmark targets assume Rust backend services, Leptos/WASM-SSR web where applicable, and sanctioned Swift/Kotlin/WinUI3 native frontends only where explicit clients exist.
- Deployment-context disclosure: all targets use one canonical number set, then apply overlays for `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
- Tenant-class disclosure: all targets use one quality bar, then apply caps or contractual/economic overlays for `demo_trial`, `paid`, and `revenue_share`.
- Public-source disclosure: Google and Notion publish API quotas; Microsoft publishes broad Graph throttle classes; none of the three publishes comprehensive editor p99 latency, so local latency comparisons use internal-estimated browser harness rows from the existing benchmark file.
- Retired-tier disclosure: prior rows such as `docs (paid)` and `docs (compliance_pack)` in `benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md` are not reused as active schema.

## §2 Counterpart Numbers

### §2.1 Google Docs Numbers

- GNUM-001 Public-source: Google Docs API read requests per minute per project: 3,000.
- GNUM-002 Public-source: Google Docs API read requests per minute per user per project: 300.
- GNUM-003 Public-source: Google Docs API write requests per minute per project: 600.
- GNUM-004 Public-source: Google Docs API write requests per minute per user per project: 60.
- GNUM-005 Public-source: Google Docs sharing help states up to 100 people can view, edit, or comment on a shared file at the same time.
- GNUM-006 Public-source: Google version history supports viewing and restoring earlier versions; no latency number is published.
- GNUM-007 Internal-estimated from `benchmarks/...md:12`: keystroke ack p50 22 ms.
- GNUM-008 Internal-estimated from `benchmarks/...md:12`: keystroke ack p95 65 ms.
- GNUM-009 Internal-estimated from `benchmarks/...md:12`: keystroke ack p99 145 ms.
- GNUM-010 Internal-estimated from `benchmarks/...md:23`: 100-page cold-load p50 1.2 s.
- GNUM-011 Internal-estimated from `benchmarks/...md:23`: 100-page cold-load p95 2.4 s.
- GNUM-012 Internal-estimated from `benchmarks/...md:35`: observed degradation above 50 active editors.
- GNUM-013 Public-source: Google Docs collaborator hard reference is 100 simultaneous users from sharing help, so any Oyatie target above 100 must be tested, not asserted.
- GNUM-014 Public-source gap: Google does not publish comprehensive editor p95/p99, conflict-merge latency, or export pipeline latency for Docs.
- GNUM-015 Product implication: Oyatie must beat 65 ms p95 keystroke ack and 2.4 s p95 100-page load to claim ahead of the existing internal comparison.

### §2.2 Microsoft Word Online Numbers

- MNUM-001 Public-source: Microsoft Word real-time coauthoring support lets collaborators edit the same document and see changes.
- MNUM-002 Public-source: Microsoft Graph throttling limits document OneDrive service limits; one app-level limit is 130,000 requests per 10 seconds across all tenants for the service class shown in the Microsoft table.
- MNUM-003 Public-source: Microsoft does not publish a Word Online editor p95 keystroke latency number.
- MNUM-004 Public-source: Microsoft does not publish a universal Word Online concurrent editor cap in the coauthoring help page used here.
- MNUM-005 Internal-estimated from `benchmarks/...md:13`: keystroke ack p50 35 ms.
- MNUM-006 Internal-estimated from `benchmarks/...md:13`: keystroke ack p95 110 ms.
- MNUM-007 Internal-estimated from `benchmarks/...md:13`: keystroke ack p99 240 ms.
- MNUM-008 Internal-estimated from `benchmarks/...md:24`: 100-page cold-load p50 1.8 s.
- MNUM-009 Internal-estimated from `benchmarks/...md:24`: 100-page cold-load p95 3.6 s.
- MNUM-010 Internal-estimated from `benchmarks/...md:36`: observed degradation above 30 active editors.
- MNUM-011 Product implication: Microsoft Word Online remains the DOCX fidelity benchmark even when its web collaboration latency is not the lowest observed value.
- MNUM-012 Product implication: Oyatie must target >=95 percent OOXML round-trip from `PRD.md:338` and treat this as a benchmark number, not a qualitative claim.
- MNUM-013 Product implication: Oyatie must preserve comments, track changes, images, tables, headings, citations, and style metadata in the 50-page import/export workload.
- MNUM-014 Product implication: Oyatie must not use API throughput numbers alone as proof of editor parity.
- MNUM-015 Public-source gap: Microsoft’s published Graph throttles do not substitute for Word Online UX latency.

### §2.3 Notion Docs Numbers

- NNUM-001 Public-source: Notion API average request limit is 3 requests per second per integration, with bursts allowed.
- NNUM-002 Public-source: Notion API payloads are limited to 1,000 block elements.
- NNUM-003 Public-source: Notion API payloads are limited to 500 KB overall request body size.
- NNUM-004 Public-source: Notion rich text object text content limit is 2,000 characters.
- NNUM-005 Public-source: Notion URL property/string limit is 2,000 characters.
- NNUM-006 Public-source: Notion append block children API limits request granularity, with 100 children per append request documented for the endpoint.
- NNUM-007 Internal-estimated from `benchmarks/...md:14`: keystroke ack p50 48 ms.
- NNUM-008 Internal-estimated from `benchmarks/...md:14`: keystroke ack p95 145 ms.
- NNUM-009 Internal-estimated from `benchmarks/...md:14`: keystroke ack p99 320 ms.
- NNUM-010 Internal-estimated from `benchmarks/...md:25`: 100-page cold-load p50 2.4 s.
- NNUM-011 Internal-estimated from `benchmarks/...md:25`: 100-page cold-load p95 4.8 s.
- NNUM-012 Internal-estimated from `benchmarks/...md:37`: observed degradation above 20 active editors.
- NNUM-013 Product implication: Oyatie must handle large block trees better than Notion’s public API write granularity if it claims high-scale block editing.
- NNUM-014 Product implication: Oyatie should publish block append and payload limits in its own API contract.
- NNUM-015 Public-source gap: Notion does not publish comprehensive editor p95/p99 latency, so internal browser-harness estimates require reproducibility artifacts before external claims.

## §3 Oyatie Target Numbers - Single Industry-Leader Set

| ID | Metric | Canonical target | Deployment-context overlay | Tenant-class overlay |
|---|---|---:|---|---|
| OYA-001 | Keystroke ack p50 | <= 8 ms | Public cloud and Oyatie IaaS should meet; on-prem/colo depend on LAN and cell sizing; guest clouds require regional cell placement. | Same target for all classes; demo_trial may cap concurrent editors before latency degrades. |
| OYA-002 | Keystroke ack p95 | <= 18 ms | Matches prior internal docs target and beats internal-estimated Google 65 ms, Word 110 ms, Notion 145 ms. | Same quality; demo_trial cap limits sustained high-load sessions. |
| OYA-003 | Keystroke ack p99 | <= 42 ms | Requires in-region editor gateways and CRDT fanout; cross-region writes should not be allowed to hide residency violations. | Same target until cap breached; cap breach should fail gracefully. |
| OYA-004 | Cursor sync p50 | <= 20 ms | Public cloud elastic cells should meet; on-prem/colo require local Valkey/CRDT fanout sizing. | Same target; demo_trial may cap editor count. |
| OYA-005 | Cursor sync p99 | <= 150 ms | Directly aligns with `PRD.md:70` and `PRD.md:297`. | Same target; cap behavior is admission control, not slower UX. |
| OYA-006 | Cold document open p50, 100-page doc | <= 400 ms | Public cloud target; OCI Always Free profile may cap document size or concurrent opens. | Same feature quality; demo_trial may cap 100-page test frequency/storage. |
| OYA-007 | Cold document open p95, 100-page doc | <= 700 ms | Must be measured per context; on-prem/colo storage latency must be certified. | Same target if workload admitted. |
| OYA-008 | Cold document open p99, normal workload | <= 300 ms | `PRD.md:67` target for canonical cold open; reconcile with 100-page benchmark shape. | Same target for admitted docs. |
| OYA-009 | Warm document open p99 | <= 100 ms | `PRD.md:68` target; depends on cache residency and tenant-cell routing. | Same target. |
| OYA-010 | Save p99 | <= 100 ms | `PRD.md:69` and `PRD.md:296`; requires local CRDT op commit and audit emit. | Same target; demo_trial cap on edits/s if needed. |
| OYA-011 | Save p999 | <= 300 ms | `PRD.md:69`; must include audit and conflict surfacing. | Same target for admitted workload. |
| OYA-012 | Search-within-doc p99 | <= 100 ms | `PRD.md:71`; search index locality required. | Same target. |
| OYA-013 | Doc-list 1,000 docs p99 | <= 200 ms | `PRD.md:72`; requires cursor pagination. | Demo_trial document count cap may reduce observed list size. |
| OYA-014 | PDF export 50-page p99 | <= 3 s | `PRD.md:73`; public cloud worker pool elastic; OCI Always Free profile caps concurrent exports. | Same target when export admitted; demo_trial cap on export count. |
| OYA-015 | PDF export 50-page p999 | <= 7 s | `PRD.md:73`; gVisor sandbox cold starts must stay bounded. | Same target for admitted jobs. |
| OYA-016 | DOCX export p99 | <= 2 s | `PRD.md:74`; Pandoc worker must be sized per context. | Same target for admitted jobs. |
| OYA-017 | DOCX import 50-page p99 | <= 3 s | `PRD.md:75`; import sanitation and macro stripping included. | Same target for admitted jobs. |
| OYA-018 | Comment post p99 | <= 100 ms | `PRD.md:76`; indexed insert and audit event included. | Same target. |
| OYA-019 | Suggested edit accept/reject p99 | <= 150 ms | Derived from save/comment path; must be explicit in future SLO. | Same target. |
| OYA-020 | Attachment upload 10 MB p99 | <= 2 s | `PRD.md:77`; storage and scanning profile must be local to context. | Demo_trial cap on attachment bytes; paid/revenue_share scale with billing/substrate. |
| OYA-021 | Concurrent active editors per doc, clean | >= 500 | Must be tested in public cloud and Oyatie IaaS; on-prem/colo require certified sizing; OCI Always Free profile lower admission cap allowed. | Demo_trial cap can be lower; paid/revenue_share can buy/subsidize higher capacity. |
| OYA-022 | Concurrent active editors per doc, stress ceiling | >= 10,000 | Prior old benchmark claimed this under retired tier; current target must be revalidated without tier labels. | Demo_trial not expected to admit this load; paid/revenue_share admit by purchased/at-cost capacity. |
| OYA-023 | Concurrent editor sessions per cell baseline | >= 50,000 | `PRD.md:314`; requires cell-level HPA and gateway sizing. | Demo_trial aggregate cap; paid/revenue_share capacity based on contract/substrate. |
| OYA-024 | Concurrent editor sessions per cell max | >= 500,000 | `PRD.md:314`; requires multi-node cell and admission control. | Not promised to demo_trial; paid/revenue_share by capacity contract. |
| OYA-025 | Edits per second per cell baseline | >= 5,000 | `PRD.md:315`; CRDT op stream and audit path included. | Demo_trial cap; paid/revenue_share scale with contract. |
| OYA-026 | Edits per second per cell max | >= 50,000 | `PRD.md:315`; requires benchmark evidence before sales claim. | Paid/revenue_share capacity target; demo_trial capped. |
| OYA-027 | Export jobs concurrent baseline | >= 50 | `PRD.md:316`; worker pool baseline. | Demo_trial cap lower; paid/revenue_share contract/substrate. |
| OYA-028 | Export jobs concurrent max | >= 500 | `PRD.md:316`; public cloud and Oyatie IaaS first. | Paid/revenue_share only at scale. |
| OYA-029 | Import jobs concurrent baseline | >= 20 | `PRD.md:317`; includes sanitation and fidelity scoring. | Demo_trial cap lower. |
| OYA-030 | Import jobs concurrent max | >= 200 | `PRD.md:317`; requires worker autoscale evidence. | Paid/revenue_share only at scale. |
| OYA-031 | Attachment uploads/s baseline | >= 100 | `PRD.md:318`; object storage p99 trigger included. | Demo_trial byte cap. |
| OYA-032 | Attachment uploads/s max | >= 1,000 | `PRD.md:318`; context storage determines admission. | Paid/revenue_share capacity. |
| OYA-033 | Embed refresh/s baseline | >= 500 | `PRD.md:319`; embed resolver worker queue included. | Same target until tenant cap. |
| OYA-034 | Embed refresh/s max | >= 5,000 | `PRD.md:319`; public cloud and Oyatie IaaS first. | Paid/revenue_share by substrate. |
| OYA-035 | Docs API read capacity per project-equivalent | >= 3,000/min | Matches Google public project quota at minimum. | Same API behavior; demo_trial cap may be lower by tenant allowance. |
| OYA-036 | Docs API write capacity per project-equivalent | >= 600/min | Matches Google public project write quota at minimum. | Same quality; demo_trial cap can rate-limit. |
| OYA-037 | API block append max children per request | >= 100 | At least matches Notion append child granularity. | Same limit unless cap policy rejects workload. |
| OYA-038 | API write payload size | >= 500 KB | At least matches Notion public payload size. | Same limit. |
| OYA-039 | Block elements per API payload | >= 1,000 | At least matches Notion public payload element limit. | Same limit. |
| OYA-040 | OOXML round-trip fidelity | >= 95 percent | `PRD.md:338`; all deployment contexts must use same fidelity corpus. | Same fidelity quality across classes. |
| OYA-041 | Share ACL correctness | 100 percent | SLO `share-acl-enforcement-correctness`; no p95 substitute allowed. | Same correctness across classes. |
| OYA-042 | CRDT no silent loss | 100 percent | SLO `crdt-merge-no-silent-loss`; no p95 substitute allowed. | Same correctness across classes. |
| OYA-043 | Audit event emission for lifecycle mutation | 100 percent | `PRD.md:89`; every mutation must emit. | Same correctness across classes. |
| OYA-044 | Legal-hold preservation completeness | 100 percent | `PRD.md:90`; content, edit history, comment thread, audit chain preserved. | Not available to demo_trial if compliance packs are forbidden; paid/revenue_share as contract allows. |
| OYA-045 | Availability document-read path | 99.95 percent monthly | `PRD.md:96`; context overlay determines contractual eligibility. | Demo_trial best-effort; paid/revenue_share contractual where purchased. |
| OYA-046 | Availability write path | 99.9 percent monthly | `PRD.md:96`; measured per context. | Demo_trial best-effort; paid/revenue_share contractual where purchased. |
| OYA-047 | Availability export pipeline | 99.9 percent monthly | `PRD.md:96`; worker isolation and retries required. | Demo_trial best-effort; paid/revenue_share contractual where purchased. |
| OYA-048 | RTO | <= 15 min | `PRD.md:97`; applies where deployment context has required replication. | Demo_trial best-effort; paid/revenue_share contractual where purchased. |
| OYA-049 | RPO | <= 60 s | `PRD.md:97`; applies where deployment context has required replication. | Demo_trial best-effort; paid/revenue_share contractual where purchased. |
| OYA-050 | OCI Always Free profile resource ceiling | 4 OCPU, 24 GB RAM, 200 GB block, 10 GB object, 10 GB archive, 2 ADB of 20 GB each, 10 TB egress, 10 Mbps LB | From canonical OCI profile; constrains admitted demo_trial workload. | Applies primarily to demo_trial; paid/revenue_share may run paid OCI or other contexts. |

## §4 Comparison Narrative

- CN-001 Keystroke latency: Oyatie target p95 18 ms is ahead of internal-estimated Google 65 ms, Word 110 ms, and Notion 145 ms, but this must be proven by fresh browser harness evidence.
- CN-002 Keystroke p99: Oyatie target 42 ms is ahead of internal-estimated Google 145 ms, Word 240 ms, and Notion 320 ms.
- CN-003 Cold load: Oyatie target 100-page p95 700 ms is ahead of internal-estimated Google 2.4 s, Word 3.6 s, and Notion 4.8 s.
- CN-004 Warm open: Oyatie p99 100 ms has no direct public counterpart number, so it should be treated as an internal SLO and verified under all contexts.
- CN-005 Save p99: Oyatie 100 ms target is an internal product SLO; counterpart public p99 is not disclosed.
- CN-006 Cursor sync p99: Oyatie 150 ms target should be measurable with 2, 25, 100, and 500 editors.
- CN-007 Concurrent editor count: Google’s public simultaneous user reference is 100; Oyatie clean target 500 and stress target 10,000 are ahead, but they require fresh no-tenant-class-drift evidence.
- CN-008 Demo_trial editor count: demo_trial may cap active editors below the paid/revenue_share target; that is a usage cap, not a feature downgrade.
- CN-009 Paid editor count: paid tenants can scale editor counts through paid substrate and contractual capacity.
- CN-010 Revenue-share editor count: revenue_share tenants can scale at cost or zero-margin substrate when gross-revenue share justifies it.
- CN-011 API reads: Oyatie should at minimum match Google’s 3,000 read requests/min/project for project-equivalent docs automation.
- CN-012 API writes: Oyatie should at minimum match Google’s 600 write requests/min/project.
- CN-013 Notion block append: Oyatie should support at least 100 child blocks per append request and document higher limits if it can safely do so.
- CN-014 Notion payload: Oyatie should meet or exceed 1,000 block elements and 500 KB payload sizes for API writes.
- CN-015 Microsoft OOXML: the 95 percent round-trip target is a catch-up/parity metric against Microsoft, not a differentiator until proven.
- CN-016 PDF/A: Oyatie target is ahead of Google and Notion, and parity/near-parity with Microsoft/LibreOffice-style expectations.
- CN-017 Audit chain: Oyatie target 100 percent lifecycle mutation audit is ahead of the visible counterpart product surfaces.
- CN-018 Share ACL correctness: Oyatie must treat correctness as a hard invariant, not a latency percentile.
- CN-019 CRDT no silent loss: Oyatie must treat no silent edit loss as a hard invariant, not an availability target.
- CN-020 Legal hold: paid and revenue_share tenants may activate legal-hold/compliance features; demo_trial should not if the current tenant-class doctrine forbids compliance packs.
- CN-021 Availability: paid/revenue_share may have contractual SLO; demo_trial best-effort status does not lower feature correctness.
- CN-022 OCI Always Free: the resource profile is sufficient for constrained demo/trial workloads, not for proving the 500k editor-session max.
- CN-023 Public cloud: oyatie-public-cloud should be the first context to prove elasticity targets.
- CN-024 Guest-on-AWS: AWS guest context should prove portability with customer-owned substrate and the same quality targets.
- CN-025 Guest-on-OCI: OCI guest context must include both Always Free demo profile and paid OCI capacity path.
- CN-026 On-prem: on-prem target requires facility-specific storage, network, and replication qualification.
- CN-027 Colo: colo target requires network and hardware qualification, especially for editor fanout and storage latency.
- CN-028 Oyatie-as-cloud-provider: Oyatie IaaS should meet or exceed public-cloud elasticity because Oyatie controls the substrate.
- CN-029 Unsupported current claim: any benchmark that says `paid` or `compliance_pack` is no longer current schema and must be rewritten or retired.
- CN-030 Current benchmark acceptance: use this target set until fresh benchmark evidence replaces prior tiered rows.

## §5 Deployment-Context Overlay Detail

- DCO-001 `oyatie-public-cloud`: target all canonical numbers, elastic scale, contractual SLO for paid/revenue_share, best-effort for demo_trial.
- DCO-002 `guest-on-aws`: target all canonical numbers when customer AWS substrate meets sizing; OpenTofu module must encode prerequisites.
- DCO-003 `guest-on-oci`: target all canonical numbers on paid OCI sizing; OCI Always Free profile admits only constrained demo_trial workloads.
- DCO-004 `on-prem`: target correctness invariants everywhere; latency and throughput require certified facility sizing.
- DCO-005 `colo`: target correctness invariants everywhere; latency and throughput require dedicated network/storage qualification.
- DCO-006 `oyatie-as-cloud-provider`: target all canonical numbers and use Oyatie-owned cloud substrate to prove maximal elasticity.
- DCO-007 All contexts: same code path, same correctness invariants, same feature quality.
- DCO-008 All contexts: OpenTofu context modules are required before any context-specific claim is complete.
- DCO-009 All contexts: Helm/Kustomize may be rendered/deployed by the OpenTofu module, but they are not the canonical substrate by themselves.
- DCO-010 All contexts: OS support manifest must state which runtime and client surfaces are supported.

## §6 Tenant-Class Overlay Detail

- TCO-001 `demo_trial`: free, usage-capped, best-effort SLO, no compliance packs, no BYOK, usually OCI Always Free profile when on OCI.
- TCO-002 `demo_trial`: same editor, sharing, import/export, and block feature quality as paid, within caps.
- TCO-003 `demo_trial`: cap examples to define later include stored docs, storage bytes, editors per doc, exports per day, imports per day, AI invocations, and API writes.
- TCO-004 `paid`: per-seat plus usage billing, any deployment context, contractual SLO, compliance packs allowed, BYOK allowed.
- TCO-005 `paid`: scale targets are purchased or contractually reserved, not gated by feature tier.
- TCO-006 `revenue_share`: Oyatie takes a percentage of customer gross revenue, at-cost or zero-margin substrate, and no quality downgrade.
- TCO-007 `revenue_share`: docs meters must feed gross-revenue context only when docs is part of a revenue-generating embedded SaaS, marketplace, affiliate, or B2C operator flow.
- TCO-008 all classes: share ACL correctness, CRDT no-silent-loss, audit emission, and encryption are invariants.
- TCO-009 all classes: no class-specific UI degradation, hidden block-type limitations, or lower import fidelity.
- TCO-010 all classes: admission control and cap breach must be explicit, audited, and user-visible.

## §7 Benchmark Evidence Required Next

- ER-001 Fresh browser benchmark run for Google Docs, Microsoft Word Online, and Notion Docs with date, region, browser, OS, and account plan recorded.
- ER-002 Fresh Oyatie local benchmark run once implementation exists, using the same workloads.
- ER-003 Per-context `tofu plan` evidence for each deployment context before claiming deployment-specific numbers.
- ER-004 OCI Always Free demo_trial load test using 4 OCPU and 24 GB RAM ceiling.
- ER-005 DOCX corpus fidelity report with at least 100 documents and Microsoft Word round-trip comparison.
- ER-006 Notion block import/export corpus with nested blocks, embeds, comments, and database references.
- ER-007 Google Docs migration corpus with comments, suggestions, version history, permissions, and embedded Drive objects.
- ER-008 Share ACL and per-block ACL property tests.
- ER-009 CRDT no-silent-loss randomized concurrent-edit tests.
- ER-010 Legal hold and audit-chain completeness tests.

## §8 Closing Benchmark Decision

- Decision BD-001: current active benchmark schema is single target set plus overlays.
- Decision BD-002: old `docs (paid)` and `docs (compliance_pack)` rows are retired evidence, not current schema.
- Decision BD-003: public-source numbers are used where vendors publish them.
- Decision BD-004: internal-estimated numbers are labeled and cannot be sales claims without reproducibility evidence.
- Decision BD-005: the first benchmark implementation wave should prioritize editor latency, cold load, editor concurrency, DOCX fidelity, Notion block API parity, and OCI Always Free demo caps.
- Decision BD-006: correctness invariants outrank latency percentiles for CRDT no silent loss, share ACL enforcement, audit event emission, and legal hold preservation.

## §9 Measurement Acceptance Checklist

- MAC-001 Every benchmark run must record git SHA, image digest, region, cell, deployment context, tenant class, OS, CPU architecture, memory, browser, and account plan.
- MAC-002 Every browser benchmark must record raw navigation timing, WebSocket or HTTP stream timing, paint timing, and user-observable latency.
- MAC-003 Every editor benchmark must separate local echo latency from server acknowledgement latency.
- MAC-004 Every CRDT benchmark must record operation count, document block count, payload bytes, collaborator count, and merge-conflict count.
- MAC-005 Every save benchmark must include audit event emission time, not only database commit time.
- MAC-006 Every share benchmark must include authorization policy evaluation and negative-access checks.
- MAC-007 Every import benchmark must include sanitation, macro handling, attachment extraction, and fidelity scoring.
- MAC-008 Every export benchmark must include sandbox start time, renderer time, validation time, and storage write time.
- MAC-009 Every attachment benchmark must include scan time and object-storage write time.
- MAC-010 Every AI-assist benchmark must include policy evaluation and prompt wrapping time.
- MAC-011 Demo_trial benchmark runs must record the exact cap being tested and whether the admission controller rejected or admitted the workload.
- MAC-012 Paid benchmark runs must record purchased capacity assumptions and contractual SLO class.
- MAC-013 Revenue_share benchmark runs must record at-cost substrate sizing and settlement-meter activation.
- MAC-014 OCI Always Free benchmark runs must record 4 OCPU, 24 GB RAM, 200 GB block storage, object/archive use, ADB use, egress, and load balancer ceiling.
- MAC-015 On-prem benchmark runs must record facility network latency, storage class, replication mode, and operator-managed dependency versions.
- MAC-016 Colo benchmark runs must record hardware SKU, network path, storage latency, and power/network redundancy assumptions.
- MAC-017 Guest-on-AWS benchmark runs must record region, instance type, managed-service choices, and customer-account constraints.
- MAC-018 Guest-on-OCI benchmark runs must record whether the profile is Always Free demo_trial or paid OCI capacity.
- MAC-019 Oyatie-public-cloud benchmark runs must record elasticity limits and whether noisy-neighbor controls were active.
- MAC-020 Oyatie-as-cloud-provider benchmark runs must record substrate generation and any hardware offload assumptions.

## §10 Metric-to-Artifact Traceability

- TRACE-001 Keystroke latency traces to `PRD.md:69-70` and SLO `collab-cursor-sync-latency.openslo.yaml`.
- TRACE-002 Document open traces to `PRD.md:67-68` and SLO `doc-open-latency.openslo.yaml`.
- TRACE-003 Save traces to `PRD.md:69` and SLO `save-latency.openslo.yaml`.
- TRACE-004 Search traces to `PRD.md:71` and SLO `search-within-doc-latency.openslo.yaml`.
- TRACE-005 Doc list traces to `PRD.md:72` and SLO `doc-list-latency.openslo.yaml`.
- TRACE-006 PDF export traces to `PRD.md:73` and SLO `export-pdf-latency.openslo.yaml`.
- TRACE-007 Export pipeline availability traces to SLO `pandoc-export-pipeline-availability.openslo.yaml`.
- TRACE-008 Share ACL correctness traces to `PRD.md:48-49` and SLO `share-acl-enforcement-correctness.openslo.yaml`.
- TRACE-009 CRDT no silent loss traces to `PRD.md:31` and SLO `crdt-merge-no-silent-loss.openslo.yaml`.
- TRACE-010 Concurrent editor sessions trace to `PRD.md:313-315`.
- TRACE-011 Export/import concurrency traces to `PRD.md:316-317`.
- TRACE-012 Attachment upload traces to `PRD.md:77` and `PRD.md:318`.
- TRACE-013 Embed refresh throughput traces to `PRD.md:98` and `PRD.md:319`.
- TRACE-014 OOXML fidelity traces to `PRD.md:338`.
- TRACE-015 Legal hold preservation traces to `PRD.md:56` and `PRD.md:89-90`.
- TRACE-016 Availability/RTO/RPO traces to `PRD.md:96-97`.
- TRACE-017 Google API quota comparison traces to https://developers.google.com/docs/api/limits.
- TRACE-018 Google active collaboration comparison traces to https://support.google.com/docs/answer/2494822.
- TRACE-019 Microsoft coauthoring comparison traces to https://support.microsoft.com/en-gb/office/collaborate-on-word-documents-with-real-time-co-authoring-7dd3040c-3f30-4fdd-bab0-8586492a1f1d.
- TRACE-020 Microsoft API throughput comparison traces to https://learn.microsoft.com/en-us/graph/throttling-limits.
- TRACE-021 Notion request-rate comparison traces to https://developers.notion.com/reference/request-limits.
- TRACE-022 Notion append-block comparison traces to https://developers.notion.com/reference/patch-block-children.
- TRACE-023 Retired local benchmark rows trace to `benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:11-14`, `:22-25`, and `:33-37`.
- TRACE-024 Tier-retirement treatment traces to `coherence-audit-2026-05-20.md` §3.4.T.
- TRACE-025 Tenant-class absence traces to `coherence-audit-2026-05-20.md` §3.4.C.

## §11 Benchmark Claim Boundaries

- BCB-001 "Ahead" means Oyatie target is numerically stronger than the best available counterpart number and has fresh evidence.
- BCB-002 "Parity" means Oyatie target matches the best available counterpart number or public limit and has fresh evidence.
- BCB-003 "Catch-up" means the counterpart remains stronger or the Oyatie metric exists only as an unproven target.
- BCB-004 "Current target" means a design target in this document, not a measured result.
- BCB-005 "Internal-estimated" means prior browser-harness estimate from the existing benchmark file, not a public vendor promise.
- BCB-006 "Public-source" means a hard number published by the vendor source linked above.
- BCB-007 No benchmark claim should combine demo_trial admission caps with paid/revenue_share scale targets without naming the tenant-class overlay.
- BCB-008 No benchmark claim should combine OCI Always Free profile limits with paid OCI or public-cloud elasticity.
- BCB-009 No benchmark claim should cite retired tier labels as active product levels.
- BCB-010 No benchmark claim should convert correctness invariants into percentile goals.
