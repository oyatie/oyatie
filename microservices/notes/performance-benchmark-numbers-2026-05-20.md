# Notes µservice performance benchmark numbers — 2026-05-20

- µservice: `notes`.
- Scope path: `/Users/jasonlee/oyatie/microservices/notes/`.
- Counterpart bar: Notion / Obsidian / Apple Notes.
- Citation anchor 1: canonical notes latency targets are in `microservices/notes/PRD.md:87-101`.
- Citation anchor 2: notes OpenSLO latency targets are in `microservices/notes/slos/note-open-latency.openslo.yaml:5-43`, `note-create-latency.openslo.yaml:5-42`, `sync-latency.openslo.yaml:5-42`, `tag-search-latency.openslo.yaml:5-41`, `full-text-search-latency.openslo.yaml:5-42`, `graph-render-latency.openslo.yaml:5-42`, and `web-clipper-capture-latency.openslo.yaml:5-41`.
- Citation anchor 3: notes per-cell capacity targets are in `microservices/notes/capacity-model.md:30-40`.
- Citation anchor 4: the existing local benchmark document is in `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:17-110`; its old segmentation language is not reused here.
- Citation anchor 5: deployment-context requirements are in `specs/master-plan-sequencing.json:704-745`; OpenTofu-only requirements are in `specs/master-plan-sequencing.json:747-775`.
- Public counterpart source 1: Notion API limits at `https://developers.notion.com/reference/request-limits`.
- Public counterpart source 2: Obsidian Sync plans at `https://obsidian.md/help/sync/plans` and security at `https://obsidian.md/help/sync/security`.
- Public counterpart source 3: Apple iCloud data security at `https://support.apple.com/en-ie/102651` and Notes sharing at `https://support.apple.com/en-euro/guide/notes/apda5307056b/mac`.
- Methodology disclosure: public vendors do not publish complete p50/p95/p99 note-open, search, graph, or edit benchmarks for all workloads; this report separates published numeric limits, existing unverified repo benchmark numbers, and explicitly labeled engineering estimates.
- No rows or headings in this report use retired color-metal feature-tier scaffolding.
- Tenant-class model used here: `demo_trial`, `paid`, `revenue_share`.
- Deployment contexts used here: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.

## §1 Methodology

- M-001 Benchmark dimensions: latency, throughput, concurrency, scale ceiling, quota limit, storage envelope, privacy correctness, and deployability constraint.
- M-002 Latency dimensions: note open, note create, sync-after-edit, collaborative merge ack, tag search, full-text search, graph render, web clipper capture, AI summarize, AI tag suggest, and AI link suggest.
- M-003 Throughput dimensions: note creates/sec, edits/sec, opens/sec, active collaborative sessions, web clipper installations, and attachment handoffs.
- M-004 Scale dimensions: active accounts per cell, tags per tenant, backlinks per note, search index size, and version-history storage.
- M-005 Correctness dimensions: E2E AI-block correctness and sync data-residency correctness.
- M-006 Test workload W1: warm note open through REST, cache, Cedar, and response serialization.
- M-007 Test workload W2: note create through insert, policy check, and audit-chain seal where applicable.
- M-008 Test workload W3: sync-after-edit through client delta, WebSocket acknowledgement, and durable persist.
- M-009 Test workload W4: collaborative edit merge acknowledgement for Loro op-sets.
- M-010 Test workload W5: tag search through adjacency index.
- M-011 Test workload W6: full-text search in server-side Professional context.
- M-012 Test workload W7: graph render for 5k-note vault.
- M-013 Test workload W8: web clipper REST acknowledgement for 50KB to 500KB raw HTML capture.
- M-014 Test workload W9: AI summarize for non-E2E content.
- M-015 Test workload W10: AI tag and link suggestion for non-E2E content.
- M-016 OS disclosure: notes has no `supported-oses.json`, so OS-specific targets remain canonical expectations rather than locally proven results.
- M-017 Architecture disclosure: no Rust implementation files exist under the notes path, so target numbers are not measured implementation results.
- M-018 Test disclosure: no notes `tests/` directory exists, so benchmark validation is not present in this µservice path.
- M-019 Existing benchmark disclosure: the local benchmark file claims measured numbers and a harness path, but the inventory did not find that harness under notes.
- M-020 Counterpart disclosure: public counterpart numbers below use public docs first; where a vendor does not publish a latency percentile, the row is labeled as an estimate or local repo benchmark estimate.
- M-021 Deployment-context disclosure: `oyatie-public-cloud` can use platform elasticity; guest contexts inherit host quotas; on-prem and colo inherit facility capacity; `guest-on-oci` may use an OCI Always Free profile for demo_trial.
- M-022 Tenant-class disclosure: product quality targets do not change by tenant_class.
- M-023 Tenant-class constraint: `demo_trial` constrains usage volume and infrastructure budget, not feature quality.
- M-024 Tenant-class constraint: `paid` can scale with per-seat and usage billing.
- M-025 Tenant-class constraint: `revenue_share` can run at cost or zero-margin substrate when revenue participation replaces normal margin.
- M-026 Measurement stop condition: a number is claimable only when a benchmark harness, workload, OS/arch, deployment context, tenant_class, run output, and timestamp exist.
- M-027 Current report status: target-setting and audit comparison, not production benchmark certification.

## §2 Counterpart Numbers

### §2.1 Notion

- N-001 Published API request-rate limit: average 3 requests per second per integration; source: Notion API request limits.
- N-002 Published rate-limit response: HTTP 429 when request rate exceeds the average limit; source: Notion API request limits.
- N-003 Published retry guidance: `Retry-After` header is an integer number of seconds; source: Notion API request limits.
- N-004 Published payload limit: request payloads are limited to 500KB; source: Notion API request limits.
- N-005 Published block element limit: request payloads are limited to 1000 block elements; source: Notion API request limits.
- N-006 Published rich-text content limit: 2000 characters; source: Notion API request limits.
- N-007 Published URL limit: 2000 characters; source: Notion API request limits.
- N-008 Published equation-expression limit: 1000 characters; source: Notion API request limits.
- N-009 Published array limit: 100 block elements or rich text objects; source: Notion API request limits.
- N-010 Published multi-select option limit: 100 options; source: Notion API request limits.
- N-011 Published relation property limit: 100 related pages; source: Notion API request limits.
- N-012 Published people property limit: 100 users; source: Notion API request limits.
- N-013 Local repo benchmark estimate for page-open cold 10k blocks: p50 240ms, p99 580ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:17-24`; validation status: unverified.
- N-014 Local repo benchmark estimate for block edit/render: p50 28ms, p99 65ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:32-40`; validation status: unverified.
- N-015 Local repo benchmark estimate for collaborative cursor sync: p50 140ms, p99 320ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:45-55`; validation status: unverified.
- N-016 Local repo benchmark estimate for search query: p50 180ms, p99 480ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:57-69`; validation status: unverified.
- N-017 Local repo benchmark estimate for bidirectional-link parsing accuracy: 98%; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:71-82`; validation status: unverified.
- N-018 Public pricing benchmark from local doc: Plus $120/user/year, Business $180/user/year, Enterprise estimated $360/user/year; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:84-96`; validation status: price may drift and requires refresh before commercial use.
- N-019 Notion headline: strongest published quantitative source is API limits, not latency percentiles.
- N-020 Notion comparison use: Oyatie should beat Notion on open/create/search latency while matching or exceeding payload/limit transparency.

### §2.2 Obsidian

- O-001 Published Sync plan number: Standard has 1 remote vault; source: Obsidian Sync plans.
- O-002 Published Sync plan number: Plus has up to 10 remote vaults; source: Obsidian Sync plans.
- O-003 Published Sync storage number: Standard has 1GB storage; source: Obsidian Sync plans.
- O-004 Published Sync storage number: Plus has 10GB to 100GB storage; source: Obsidian Sync plans.
- O-005 Published Sync file limit: Standard max file size 5MB; source: Obsidian Sync plans.
- O-006 Published Sync file limit: Plus max file size 200MB; source: Obsidian Sync plans.
- O-007 Published Sync version history: Standard stores 1 month; source: Obsidian Sync plans.
- O-008 Published Sync version history: Plus stores 12 months; source: Obsidian Sync plans.
- O-009 Published Sync device limit: unlimited devices; source: Obsidian Sync plans.
- O-010 Published Sync sharing number: Standard shared vaults are 0; source: Obsidian Sync plans.
- O-011 Published Sync sharing number: Plus shared vaults are available; source: Obsidian Sync plans.
- O-012 Published Sync encryption: AES-256 in GCM mode; source: Obsidian Sync security.
- O-013 Published key-derivation parameter: scrypt-derived key from user password; source: Obsidian Sync security.
- O-014 Published server-region count: four selectable regions, Asia, Europe, North America, and Oceania; source: Obsidian Sync security.
- O-015 Local repo benchmark estimate for local page-open cold 10k blocks: p50 35ms, p99 80ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:17-24`; validation status: unverified.
- O-016 Local repo benchmark estimate for block edit/render: p50 8ms, p99 20ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:32-40`; validation status: unverified.
- O-017 Local repo benchmark estimate for local search: p50 35ms, p99 90ms; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:57-69`; validation status: unverified.
- O-018 Local repo benchmark estimate for bidirectional-link parsing accuracy: 99%; source: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:71-82`; validation status: unverified.
- O-019 Obsidian headline: local-first operation creates a hard latency bar for note open, edit, search, and graph-like traversal.
- O-020 Obsidian comparison use: Oyatie should target local-like warm-open latency while accepting network ceilings in hosted contexts.

### §2.3 Apple Notes

- A-001 Published data-security number: standard iCloud protection includes end-to-end encryption for 15 data categories; source: Apple iCloud data security overview.
- A-002 Published data-security number: Advanced Data Protection expands end-to-end encryption to 25 data categories; source: Apple iCloud data security overview.
- A-003 Published platform requirement: Advanced Data Protection requires iOS 16.2 or later for iPhone; source: Apple iCloud data security overview.
- A-004 Published platform requirement: Advanced Data Protection requires iPadOS 16.2 or later for iPad; source: Apple iCloud data security overview.
- A-005 Published platform requirement: Advanced Data Protection requires macOS 13.1 or later for Mac; source: Apple iCloud data security overview.
- A-006 Published Notes encryption mode: Notes is standard-encrypted by default in iCloud and end-to-end encrypted under Advanced Data Protection; source: Apple iCloud data security overview.
- A-007 Published sharing modes: Apple Notes supports collaboration and send-copy flows; source: Apple Notes sharing guide.
- A-008 Published permission modes: shared notes support view-only and edit permission classes; source: Apple Notes sharing guide.
- A-009 Published access modes: sharing can be restricted to invited people or made available through a link when configured; source: Apple Notes sharing guide.
- A-010 Published locked-note constraint: locked notes cannot be shared; source: Apple Notes sharing guide.
- A-011 Published smart-folder constraint: smart folders cannot be shared directly; source: Apple Notes sharing guide.
- A-012 Published ADP sharing caveat: shared Notes can remain end-to-end encrypted only when participants support the required protections; source: Apple iCloud data security overview.
- A-013 Published web-sharing caveat: iCloud web and some sharing paths may require data availability to Apple servers; source: Apple iCloud data security overview.
- A-014 Estimated warm local note-open latency: under 100ms p95 on supported Apple devices; source: engineering estimate from Apple native-local product shape, vendor does not publish p95.
- A-015 Estimated note-create latency: under 150ms p95 for local capture before iCloud propagation; source: engineering estimate from Apple native-local product shape, vendor does not publish p95.
- A-016 Estimated iCloud sync propagation target for user-perceived responsiveness: under 2s p95 on healthy network; source: engineering estimate, vendor does not publish p95.
- A-017 Estimated search latency for local indexed notes: under 200ms p95 on moderate vaults; source: engineering estimate, vendor does not publish p95.
- A-018 Estimated scanner/OCR ingestion latency: seconds-scale and device-dependent; source: engineering estimate, vendor does not publish p95.
- A-019 Apple headline: Apple publishes privacy and sharing constraints more clearly than latency percentiles.
- A-020 Apple comparison use: Oyatie should match Apple capture simplicity and beat Apple on explicit graph/backlink performance.

## §3 Oyatie Target Numbers — Single Industry-Leader Set

### §3.1 Canonical Targets

| Metric | Canonical target | Source | Counterpart stance |
|---|---:|---|---|
| Note open warm p50 | ≤20ms | `PRD.md:91` | Ahead of Notion, near local-app expectation |
| Note open warm p95 | ≤50ms | `PRD.md:91`; `slos/note-open-latency.openslo.yaml:5-43` | Must beat Notion and approach Obsidian/Apple local feel |
| Note open warm p99 | ≤100ms | `PRD.md:91` | Industry-leading hosted target |
| Note open warm p999 | ≤250ms | `PRD.md:91` | Hosted tail-control target |
| Note create p50 | ≤15ms | `PRD.md:92` | Ahead of hosted counterparts |
| Note create p95 | ≤30ms | `PRD.md:92`; `slos/note-create-latency.openslo.yaml:5-42` | Industry-leading target |
| Note create p99 | ≤60ms | `PRD.md:92` | Hosted tail-control target |
| Note create p999 | ≤150ms | `PRD.md:92` | Capture responsiveness target |
| Sync-after-edit p50 | ≤100ms | `PRD.md:93` | Competitive with real-time editors |
| Sync-after-edit p95 | ≤250ms | `PRD.md:93` | Faster than local Notion estimate |
| Sync-after-edit p99 | ≤500ms | `PRD.md:93`; `slos/sync-latency.openslo.yaml:5-42` | SLO-aligned tail target |
| Sync-after-edit p999 | ≤1s | `PRD.md:93` | Acceptable human collaboration tail |
| Collaborative merge ack p99 | ≤200ms | `slos/collab-edit-merge-latency.openslo.yaml:5-34` | Aggressive CRDT ack target |
| Tag search p50 | ≤40ms | `PRD.md:94` | Near local-app feel |
| Tag search p95 | ≤100ms | `PRD.md:94`; `slos/tag-search-latency.openslo.yaml:5-41` | Beats Notion estimate |
| Tag search p99 | ≤250ms | `PRD.md:94` | Hosted tail target |
| Full-text search p50 | ≤80ms | `PRD.md:95` | Hosted parity-plus target |
| Full-text search p95 | ≤200ms | `PRD.md:95`; `slos/full-text-search-latency.openslo.yaml:5-42` | Beats Notion estimate, behind local-only ideal |
| Full-text search p99 | ≤500ms | `PRD.md:95` | Tail target at larger indexes |
| Graph render 5k p50 | ≤400ms | `PRD.md:96` | Beats most hosted graph UX |
| Graph render 5k p95 | ≤1s | `PRD.md:96`; `slos/graph-render-latency.openslo.yaml:5-42` | Acceptable interactive graph target |
| Graph render 5k p99 | ≤2s | `PRD.md:96` | Tail target for dense vaults |
| Daily-note auto-create p95 | ≤80ms | `PRD.md:97` | Native-capture target |
| Web clipper capture p95 | ≤500ms | `PRD.md:98`; `slos/web-clipper-capture-latency.openslo.yaml:5-41` | Competitive capture target |
| AI summarize p95 | ≤5s | `PRD.md:99` | Human-usable AI target |
| AI tag suggest p95 | ≤1s | `PRD.md:100` | Inline-assist target |
| AI link suggest p95 | ≤1.5s | `PRD.md:101` | Inline graph-assist target |
| E2E privacy correctness | 100% | `slos/e2e-privacy-correctness.openslo.yaml:5-54` | Must be invariant, not optimization |
| Data-residency correctness | 100% | `slos/sync-data-residency-correctness.openslo.yaml:5-45` | Must be invariant |
| Note open/create availability | 99.95% monthly | `PRD.md:123-127` | Hosted reliability target |
| Sync/graph availability | 99.9% monthly | `PRD.md:123-127` | Secondary interaction reliability |
| RTO note-store | ≤15min | `PRD.md:123-127` | Operational recovery target |
| RPO note-store | ≤5min | `PRD.md:123-127` | Replication target |

### §3.2 Capacity Targets

| Dimension | XS baseline | M target | XL max | Source | Interpretation |
|---|---:|---:|---:|---|---|
| Active notes accounts | 50k | 200k | 500k | `capacity-model.md:30-40` | Per-cell account ceiling |
| Notes/sec creates | 1k | 5k | 20k | `capacity-model.md:30-40` | Per-cell create throughput |
| Notes/sec edits | 2k | 10k | 50k | `capacity-model.md:30-40` | Per-cell edit throughput |
| Notes/sec opens warm | 5k | 25k | 100k | `capacity-model.md:30-40` | Per-cell hot-open throughput |
| Tags per tenant | 100k | 1M | 10M | `capacity-model.md:30-40` | Taxonomy scale |
| Backlinks per note | 200 | 1k | 50k | `capacity-model.md:30-40` | Fan-in cap |
| Web clipper installations | 50k | 200k | 500k | `capacity-model.md:30-40` | Extension install scale |
| Active Loro collab sessions | 100 | 500 | 5k | `capacity-model.md:30-40` | Real-time collaboration scale |
| Search index size | 50GB | 500GB | 2TB | `capacity-model.md:30-40` | Server-side search scale |
| Attachments/day via drive | 100k | 500k | 2M | `capacity-model.md:30-40` | Cross-service throughput |
- CT-001 Capacity note: PRD scalability text also mentions a 500k notes/sec aggregate shard trigger; this conflicts with the capacity-model 20k notes/sec XL per-cell ceiling and should be reconciled before external performance claims.
- CT-002 Capacity note: the report uses `capacity-model.md:30-40` as the more bounded target because it is tabular and per-cell.
- CT-003 Capacity note: scale-out beyond XL max requires additional cell math, not larger single-cell claims.

### §3.3 Deployment-Context Overlay

- DC-001 `oyatie-public-cloud`: target latencies remain canonical; throughput can scale horizontally with managed platform elasticity if OpenTofu modules and benchmark harness exist.
- DC-002 `oyatie-public-cloud`: expected capacity path is XS to XL per cell, then cell multiplication.
- DC-003 `guest-on-aws`: target latencies remain canonical; throughput is capped by tenant AWS account quotas, chosen instance families, managed database limits, and network egress rules.
- DC-004 `guest-on-aws`: no notes-specific OpenTofu module exists yet, so current status is target-only.
- DC-005 `guest-on-oci`: target latencies remain canonical; throughput is capped by OCI tenant quotas and selected shape.
- DC-006 `guest-on-oci`: OCI Always Free profile is allowed for demo_trial infrastructure, but it constrains sustained throughput and storage.
- DC-007 OCI Always Free profile engineering cap: treat note open as approximately 200 rps, note create as approximately 50 rps, full-text search as approximately 50 rps, and web clipper capture as approximately 20 rps until measured on 4 OCPU/24GB Ampere profile.
- DC-008 OCI Always Free profile engineering cap: these caps are estimates, not measured results, and should be validated before onboarding demo_trial tenants.
- DC-009 `on-prem`: target latencies remain canonical only if facility storage, network, and Kubernetes substrate meet baseline performance.
- DC-010 `on-prem`: throughput must be facility-specific because hardware diversity is large.
- DC-011 `colo`: target latencies remain canonical if colo networking, storage, and packet-loss budgets meet platform assumptions.
- DC-012 `colo`: multi-region sync depends on customer circuit quality and peering.
- DC-013 `oyatie-as-cloud-provider`: target latencies remain canonical; Oyatie owns infrastructure assumptions and can enforce platform SLOs directly.
- DC-014 `oyatie-as-cloud-provider`: throughput should be strongest here if provider cells are designed around the notes XS/M/XL capacity model.
- DC-015 Cross-context rule: context overlays can cap usage volume or require cell multiplication, but they do not lower the product quality target.

### §3.4 Tenant-Class Overlay

- TC-001 `demo_trial`: feature quality target remains identical to paid production quality.
- TC-002 `demo_trial`: usage volume is capped by time, quota, and infrastructure budget.
- TC-003 `demo_trial`: recommended starting cap is 1 active vault, 1GB note/ciphertext storage, 5GB attachment handoff allowance, 10 requests/sec burst, and 50 requests/min sustained API usage until measured.
- TC-004 `demo_trial`: on OCI Always Free profile, enforce hard backpressure before saturating CPU, memory, storage, or egress.
- TC-005 `demo_trial`: best-effort SLO language is acceptable only for contractual remedy, not for code quality.
- TC-006 `paid`: feature quality target remains identical.
- TC-007 `paid`: per-seat and usage billing allow scale-out by account, cell, storage, search index, and AI-assist consumption.
- TC-008 `paid`: contractual SLOs may use 99.95% note open/create and 99.9% sync/graph as the starting commitment if implementation proves them.
- TC-009 `paid`: BYOK and compliance packs are allowed when notes handoffs to tenancy, identity, audit-chain, and OpenBao are implemented.
- TC-010 `revenue_share`: feature quality target remains identical.
- TC-011 `revenue_share`: at-cost or zero-margin substrate means profitability comes from gross-revenue share, not reduced technical service.
- TC-012 `revenue_share`: public template, embedded SaaS, B2C operator, affiliate, and seller workflows need quota accounting before launch.
- TC-013 `revenue_share`: performance targets should be identical to paid for equivalent workload volume.
- TC-014 Cross-class invariant: Personal E2E AI refusal remains 100% for every tenant_class.
- TC-015 Cross-class invariant: data-residency correctness remains 100% for every tenant_class.
- TC-016 Cross-class invariant: retired feature segmentation must not reappear as tenant_class-specific quality downgrade.

## §4 Comparison Narrative

- CN-001 Note open: Oyatie target p95 50ms is ahead of the local repo Notion estimate p99 580ms and close enough to challenge Obsidian local p99 80ms if warm-cache implementation exists.
- CN-002 Note open: Apple Notes local capture is estimated under 100ms p95; Oyatie’s 50ms hosted p95 is aggressive and requires cache, frontend, and network discipline.
- CN-003 Note create: Oyatie p95 30ms is industry-leading as a hosted target and likely ahead of Notion and Apple iCloud server round trips.
- CN-004 Sync-after-edit: Oyatie p99 500ms is human-acceptable and stronger than the local repo Notion estimate of 320ms p99 only if measured under comparable collaboration load.
- CN-005 Collaborative merge: Oyatie p99 200ms merge ack is ahead of most public note products as a declared target, but no implementation proves it.
- CN-006 Tag search: Oyatie p95 100ms should beat Notion’s local repo search estimate and approach local Obsidian search.
- CN-007 Full-text search: Oyatie p95 200ms should beat the local repo Notion search estimate but may trail Obsidian local search for small vaults.
- CN-008 Graph render: Oyatie p95 1s for 5k-note vault is strong if client WebGL render and server snapshot assembly are both measured.
- CN-009 Web clipper: Oyatie p95 500ms is competitive for acknowledgement, but full content extraction and sanitization need separate measurement.
- CN-010 AI summarize: Oyatie p95 5s is usable but must exclude E2E content structurally.
- CN-011 AI tag suggest: Oyatie p95 1s is appropriate for inline assistance.
- CN-012 AI link suggest: Oyatie p95 1.5s is appropriate if embedding lookup and graph scoring stay bounded.
- CN-013 E2E privacy correctness: Oyatie target 100% is a hard invariant and compares favorably with Apple ADP and Obsidian Sync privacy posture at the spec level.
- CN-014 Data-residency correctness: Oyatie target 100% is a hard invariant and must be proven per deployment context.
- CN-015 Capacity: Oyatie XL 100k warm opens/sec per cell is far beyond public Notion API request limits, but it is a platform internal target, not a per-integration API allowance.
- CN-016 Capacity: Oyatie 20k creates/sec per cell is ambitious and should not be compared directly to Notion’s 3 requests/sec integration API limit without separating public API and internal service throughput.
- CN-017 Obsidian local latency: Obsidian remains the hardest bar for open/edit/search because local clients avoid network round trips.
- CN-018 Apple native UX: Apple remains the hardest bar for capture ergonomics because the app is integrated into the OS and device camera flow.
- CN-019 Notion breadth: Notion remains the hardest bar for database/workspace breadth, not raw latency.
- CN-020 OCI Always Free profile: demo_trial infrastructure can prove small-tenant viability but cannot be used to claim full XL capacity.
- CN-021 `oyatie-public-cloud`: should be the reference context for industry-leader hosted performance.
- CN-022 `guest-on-aws`: should be benchmarked separately because customer quota and instance choices can dominate.
- CN-023 `guest-on-oci`: should have two benchmark tracks, Always Free profile demo_trial and paid OCI substrate.
- CN-024 `on-prem`: should require a facility-readiness benchmark before contractual SLO.
- CN-025 `colo`: should require network-loss and storage-latency measurements before contractual SLO.
- CN-026 `oyatie-as-cloud-provider`: should be the cleanest environment for repeatable performance certification.
- CN-027 Current benchmark readiness: no local harness evidence means no performance claim is complete.
- CN-028 Current target quality: the target set is coherent, aggressive, and counterpart-aware.
- CN-029 Current audit verdict: performance report can set the target bar but cannot certify measured results.
- CN-030 Required next evidence: add Rust benchmark harness, produce run artifacts, include OS/arch/context/tenant_class, and link results to SLOs.

## §5 Measurement Backlog

- MB-001 Add benchmark workload for W1 warm note open with cache-hit and cache-miss labels.
- MB-002 Add benchmark workload for W2 note create with audit-chain enabled and disabled by content context.
- MB-003 Add benchmark workload for W3 sync-after-edit across one, two, and five devices.
- MB-004 Add benchmark workload for W4 Loro collaborative merge with 2, 10, and 50 collaborators.
- MB-005 Add benchmark workload for W5 tag search at 100k, 1M, and 10M tags.
- MB-006 Add benchmark workload for W6 full-text search at 50GB, 500GB, and 2TB index sizes.
- MB-007 Add benchmark workload for W7 graph render at 1k, 5k, 25k, and 100k note vaults.
- MB-008 Add benchmark workload for W8 web clipper capture at 50KB, 500KB, and 5MB raw source sizes.
- MB-009 Add benchmark workload for W9 AI summarize with explicit non-E2E guard.
- MB-010 Add benchmark workload for W10 AI tag/link suggestions with embedding cache hot and cold states.
- MB-011 Add correctness test for E2E AI refusal.
- MB-012 Add correctness test for sync data-residency.
- MB-013 Add deployment benchmark for `oyatie-public-cloud`.
- MB-014 Add deployment benchmark for `guest-on-aws`.
- MB-015 Add deployment benchmark for `guest-on-oci` paid substrate.
- MB-016 Add deployment benchmark for `guest-on-oci` OCI Always Free profile demo_trial infrastructure.
- MB-017 Add deployment benchmark for `on-prem`.
- MB-018 Add deployment benchmark for `colo`.
- MB-019 Add deployment benchmark for `oyatie-as-cloud-provider`.
- MB-020 Add capability matrix for `demo_trial`, `paid`, and `revenue_share`.
- MB-021 Add OS/arch matrix once `supported-oses.json` exists.
- MB-022 Add public API rate-limit policy and compare it to Notion’s public API limits.
- MB-023 Add storage quota policy and compare it to Obsidian Sync public storage limits.
- MB-024 Add privacy-sharing downgrade tests and compare them to Apple iCloud sharing caveats.
- MB-025 Add a benchmark results JSON schema so future reports cite run artifacts rather than prose.

## §6 Final Performance Position

- FP-001 Oyatie’s declared latency targets are stronger than Notion’s local repo benchmark estimates.
- FP-002 Oyatie’s declared latency targets are close to Obsidian local-app expectations for warm open and search, but hosted networking makes this difficult.
- FP-003 Oyatie’s declared privacy correctness targets are competitive with Apple ADP and Obsidian Sync at the doctrine level.
- FP-004 Oyatie’s declared native capture experience is not proven against Apple Notes.
- FP-005 Oyatie’s declared capacity model is high-scale but unvalidated.
- FP-006 Oyatie’s existing benchmark document is not sufficient evidence because the harness is absent and the document uses retired segmentation.
- FP-007 The valid target is a single industry-leader set plus deployment-context and tenant_class overlays.
- FP-008 The stop condition for performance maturity is benchmark evidence, not another benchmark prose file.

## §7 Certification Readiness Gates

- RG-001 Certification gate one: `supported-oses.json` exists and maps every benchmark run to an OS support class.
- RG-002 Certification gate two: Rust benchmark harness exists under the notes implementation surface.
- RG-003 Certification gate three: each workload emits machine-readable JSON with run ID, timestamp, git SHA, OS, arch, deployment context, and tenant_class.
- RG-004 Certification gate four: W1 through W10 workloads run successfully in `oyatie-public-cloud`.
- RG-005 Certification gate five: W1 through W10 workloads run successfully in `guest-on-aws`.
- RG-006 Certification gate six: W1 through W10 workloads run successfully in `guest-on-oci`.
- RG-007 Certification gate seven: W1 through W10 workloads run successfully on OCI Always Free profile for demo_trial infrastructure.
- RG-008 Certification gate eight: W1 through W10 workloads run successfully in `on-prem`.
- RG-009 Certification gate nine: W1 through W10 workloads run successfully in `colo`.
- RG-010 Certification gate ten: W1 through W10 workloads run successfully in `oyatie-as-cloud-provider`.
- RG-011 Certification gate eleven: SLO Prometheus queries match benchmark metric names.
- RG-012 Certification gate twelve: E2E AI refusal test proves zero plaintext server access.
- RG-013 Certification gate thirteen: data-residency test proves zero wrong-pack writes.
- RG-014 Certification gate fourteen: Notion public API limit comparison is kept separate from internal service throughput.
- RG-015 Certification gate fifteen: Obsidian local-app latency comparison is kept separate from hosted-network latency.
- RG-016 Certification gate sixteen: Apple privacy and sharing comparison is kept separate from unpublicized Apple latency percentiles.
- RG-017 Certification gate seventeen: demo_trial caps are enforced by quota and backpressure, not by reduced target quality.
- RG-018 Certification gate eighteen: paid scale claims include per-seat and usage billing assumptions.
- RG-019 Certification gate nineteen: revenue_share scale claims include gross-revenue share assumptions and at-cost substrate policy.
- RG-020 Certification gate twenty: every published number cites either a measured run artifact, a public source, or a labeled estimate.
- RG-021 Until all gates pass, notes may publish target numbers but should not publish measured-performance claims.
- RG-022 The first certification batch should prioritize W1, W2, W3, W5, and W6 because those map directly to note capture and retrieval.
- RG-023 The second certification batch should prioritize W4 and W7 because collaboration and graph are flagship differentiators.
- RG-024 The third certification batch should prioritize W8, W9, and W10 because clipper and AI paths have the largest policy surface.
- RG-025 Certification completion requires deleting or rewriting stale benchmark prose that claims results without local run artifacts.
