# Notes µservice feature parity matrix — 2026-05-20

- µservice: `notes`.
- Scope path: `/Users/jasonlee/oyatie/microservices/notes/`.
- Top-3 counterpart bar: Notion, Obsidian, Apple Notes.
- Counterpart provenance: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.
- Product-purpose anchor: `microservices/notes/PRD.md:18-26`.
- Current feature anchor: `microservices/notes/PRD.md:58-83`.
- Current contract anchor: `microservices/notes/contracts/openapi/notes.yaml:316-502`.
- Privacy anchor: `microservices/notes/PRD.md:103-113`.
- Public Notion source: `https://www.notion.com/help` and `https://developers.notion.com/reference/request-limits`.
- Public Obsidian source: `https://obsidian.md/help/plugins/backlinks`, `https://obsidian.md/help/plugins/canvas`, `https://obsidian.md/help/sync/plans`, `https://obsidian.md/help/sync/security`.
- Public Apple source: `https://support.apple.com/en-euro/guide/notes/apda5307056b/mac` and `https://support.apple.com/en-ie/102651`.
- Method: compare the Oyatie notes product surface against the union of the three named counterparts, then classify gaps as covered, partially covered, documented-only, missing, or over-claimed.
- Important amendment: this matrix does not use feature tiers; all target quality is uniform across tenant classes and deployment contexts.
- Evidence standard: each gap references repo line evidence or public counterpart source evidence.

## §1 Counterpart 1 — Notion Capability Surface

- N-001 Notion is an all-in-one workspace with docs, databases, projects, wiki, AI, enterprise search, connections, templates, comments, reminders, and workspace security categories shown in its public help index.
- N-002 Notion exposes a broad docs surface: simple documents, rich text blocks, media embeds, comments, reminders, and collaborative editing.
- N-003 Oyatie coverage: notes PRD includes Markdown notes, block content, embeds through drive, comments through collaboration events, reminders through tasks handoff, and share-links at `PRD.md:18-26`.
- N-004 Gap classification: partially covered; the Oyatie docs describe core rich notes but do not prove implementation or UX parity.
- N-005 Notion databases are a defining counterpart surface: the public help index lists databases as a major topic, and the developer docs expose database/data-source APIs.
- N-006 Oyatie coverage: PRD imports Notion databases and tables, but the API model centers notes, blocks, tags, folders, backlinks, and graph operations at `contracts/openapi/notes.yaml:316-502`.
- N-007 Gap classification: partially covered; import compatibility is documented, but native relational database views are not a first-class surface.
- N-008 Notion project management is part of the counterpart surface: public help lists Projects and project management use cases.
- N-009 Oyatie coverage: PRD delegates checklist/reminder conversion to tasks at `PRD.md:18-26` and lists `tasks` dependency in `manifest.json:383-394`.
- N-010 Gap classification: partially covered through cross-service handoff, not through notes-native project databases.
- N-011 Notion wiki and knowledge-base workflows are public top-level product categories.
- N-012 Oyatie coverage: graph, backlinks, daily notes, tags, and templates are documented in `PRD.md:18-26`.
- N-013 Gap classification: covered for personal knowledge graph primitives; partially covered for enterprise wiki governance.
- N-014 Notion AI is a public major product category with AI tools, agents, meeting notes, enterprise search, and connectors listed in the help index.
- N-015 Oyatie coverage: PRD includes AI summarize, AI tag, AI link suggestions, and forbids AI on E2E Personal content at `PRD.md:103-113`.
- N-016 Gap classification: partially covered; local docs do not show agentic workflow, meeting notes, external connector search, or implementation evidence.
- N-017 Notion comments and reminders are public popular topics.
- N-018 Oyatie coverage: comments appear in workflow events at `PRD.md:220-247`; reminders hand off to tasks at `PRD.md:18-26`.
- N-019 Gap classification: partially covered; contract-level comment endpoints are not clearly enumerated in the OpenAPI lines sampled.
- N-020 Notion templates and marketplace content are public resource surfaces.
- N-021 Oyatie coverage: note templates appear in the PRD feature set at `PRD.md:18-26`.
- N-022 Gap classification: partially covered; a template marketplace, monetization, or revenue-share tenant-class path is absent.
- N-023 Notion web clipper is a public app topic.
- N-024 Oyatie coverage: web clipper is a functional requirement and SLO with p95 target at `PRD.md:58-83` and `PRD.md:87-101`.
- N-025 Gap classification: documented-only; there is no implementation or browser extension artifact in the notes path inventory.
- N-026 Notion desktop, web, and mobile app topics are public categories.
- N-027 Oyatie coverage: language policy allows Swift, Kotlin, WinUI3, and Leptos web surfaces through canonical direction, but notes has no frontend implementation evidence.
- N-028 Gap classification: missing evidence; the product target is declared but not proven.
- N-029 Notion enterprise security topics include organization controls, IP restrictions, legal holds, network controls, privacy, GDPR, and data residency in the help index.
- N-030 Oyatie coverage: notes has compliance, dpia, audit, and data residency docs; PRD includes audit events at `PRD.md:115-121`.
- N-031 Gap classification: partially covered; enterprise controls are doc-heavy and not tied to deployable enforcement.
- N-032 Notion developer API request limits publish concrete numbers: average 3 requests per second per connection, 500KB payload, 1000 block elements, and property-size limits.
- N-033 Oyatie coverage: notes OpenAPI exists, but no generated SDK, rate-limit contract, or request-shape limit table was found in the notes path.
- N-034 Gap classification: missing for developer-experience parity.
- N-035 Notion public help includes members vs guests and workspace administration.
- N-036 Oyatie coverage: notes depends on tenancy and identity at `manifest.json:383-394`, but notes artifacts do not define tenant_class adoption.
- N-037 Gap classification: missing for the new tenant-class model; see coherence audit §3.4.C.
- N-038 Notion connections and integrations are public product categories.
- N-039 Oyatie coverage: import/export surfaces are documented, and dependencies include drive, intelligence, audit-chain, tasks, and identity.
- N-040 Gap classification: partially covered; source-of-truth integrations beyond import/export are not specified.
- N-041 Notion file and media features are public popular topics.
- N-042 Oyatie coverage: PRD says embeds are via drive and web clipping exists at `PRD.md:18-26`.
- N-043 Gap classification: partially covered; local media UX and quota treatment are not shown.
- N-044 Notion status and reliability are public operational concerns.
- N-045 Oyatie coverage: notes has OpenSLO files and availability targets at `PRD.md:123-127`.
- N-046 Gap classification: partially covered; no service implementation or live telemetry evidence exists.

## §2 Counterpart 2 — Obsidian Capability Surface

- O-001 Obsidian is a local-first Markdown knowledge base centered on files, vaults, backlinks, graph view, plugins, canvas, sync, and publishing.
- O-002 Obsidian backlinks are official core behavior; the help page describes linked mentions and unlinked mentions.
- O-003 Oyatie coverage: backlinks are first-class PRD features and graph events at `PRD.md:18-26` and `PRD.md:220-247`.
- O-004 Gap classification: covered by product spec; implementation evidence is absent.
- O-005 Obsidian graph view is a signature counterpart surface.
- O-006 Oyatie coverage: graph render target for 5k notes appears at `PRD.md:87-101`.
- O-007 Gap classification: covered by target, but documented-only due to missing Rust code and harness.
- O-008 Obsidian Canvas provides an infinite 2D space and stores canvases as `.canvas` JSON files according to official help.
- O-009 Oyatie coverage: no canvas-equivalent surface appears in the notes PRD feature list or contracts sampled.
- O-010 Gap classification: missing; an Obsidian parity claim should include canvas import/render or an explicit non-goal.
- O-011 Obsidian local file ownership is a key product property.
- O-012 Oyatie coverage: PRD supports Markdown, frontmatter, imports, and exports at `PRD.md:18-26`.
- O-013 Gap classification: partially covered; export exists, but local-file vault semantics and offline-first merge rules are not proven.
- O-014 Obsidian Sync publishes storage numbers: 1 remote vault on Standard, up to 10 on Plus, 5MB file size Standard, 200MB file size Plus, 1GB Standard storage, and 10GB to 100GB Plus storage.
- O-015 Oyatie coverage: capacity model has tenant and cell sizing but no equivalent remote-vault quota map under tenant_class.
- O-016 Gap classification: missing; quota semantics need demo_trial, paid, and revenue_share overlays.
- O-017 Obsidian Sync defaults to end-to-end encryption and states Obsidian cannot access notes when E2E is used.
- O-018 Oyatie coverage: Personal notes are E2E-default and AI is forbidden on E2E Personal content at `PRD.md:103-113`.
- O-019 Gap classification: strong coverage at the doctrine level; cryptographic implementation evidence is absent.
- O-020 Obsidian Sync also documents metadata tradeoffs such as deterministic file-hash encryption and readable mapping metadata.
- O-021 Oyatie coverage: notes privacy docs state user-content protections, but metadata leakage analysis is not clearly surfaced in the PRD excerpt.
- O-022 Gap classification: partial; metadata threat modeling should be made explicit.
- O-023 Obsidian plugins are a major ecosystem surface.
- O-024 Oyatie coverage: no notes plugin API, extension marketplace, or local script/plugin compatibility path is visible in the notes inventory.
- O-025 Gap classification: missing; this is a significant counterpart gap if Obsidian parity is claimed.
- O-026 Obsidian Publish is a public publishing path.
- O-027 Oyatie coverage: share-links and exports are documented at `PRD.md:18-26`.
- O-028 Gap classification: partially covered; share-links are not equivalent to site publishing.
- O-029 Obsidian mobile and desktop operation is multi-device but fundamentally local-vault based.
- O-030 Oyatie coverage: canonical language policy supports native frontends, but notes path has no frontend artifacts.
- O-031 Gap classification: missing implementation evidence.
- O-032 Obsidian search operates over local Markdown vaults.
- O-033 Oyatie coverage: tag search and full-text search have OpenSLO targets at `slos/tag-search-latency.openslo.yaml:5-41` and `slos/full-text-search-latency.openslo.yaml:5-42`.
- O-034 Gap classification: documented-only.
- O-035 Obsidian links use Markdown-style internal links and backlinks.
- O-036 Oyatie coverage: PRD includes backlinks and graph, and tutorial/import docs mention bidirectional links.
- O-037 Gap classification: covered by docs; no parser or resolver implementation evidence.
- O-038 Obsidian supports attachments through normal file storage.
- O-039 Oyatie coverage: attachments are routed through drive embeds at `PRD.md:18-26`.
- O-040 Gap classification: partial; drive handoff may be right architecture but needs UX and sync semantics.
- O-041 Obsidian collaboration is narrower than Notion; official Sync supports shared vaults, while real-time multiplayer is not the main surface.
- O-042 Oyatie coverage: optional Loro collaboration for Professional notes is in `PRD.md:18-26` and `contracts/proto/notes.proto:11-18`.
- O-043 Gap classification: ahead on documented real-time collaboration, if implementation lands.
- O-044 Obsidian’s strongest differentiator is user-owned knowledge graph with private storage.
- O-045 Oyatie coverage: Personal E2E, Markdown export, backlinks, and graph together meet that product intent.
- O-046 Gap classification: product-coherent but not implementation-coherent.

## §3 Counterpart 3 — Apple Notes Capability Surface

- A-001 Apple Notes is a native OS note-taking product integrated with iCloud, device UX, folders, sharing, scanning, locking, tags, and platform search.
- A-002 Apple official support documents note and folder sharing, collaboration, permission controls, and restrictions on locked notes and smart folders.
- A-003 Oyatie coverage: share-links are documented and collaboration events exist at `PRD.md:18-26` and `PRD.md:220-247`.
- A-004 Gap classification: partially covered; shared-folder permission UX is not specified.
- A-005 Apple Notes supports iCloud-based collaboration and send-copy flows.
- A-006 Oyatie coverage: share-link tokens and export formats are documented at `PRD.md:103-113` and `PRD.md:18-26`.
- A-007 Gap classification: partial; static copy versus collaborative share semantics are not separated.
- A-008 Apple Notes has native device integration across Apple operating systems.
- A-009 Oyatie coverage: canonical frontend policy allows Swift for Apple surfaces, but notes has no Swift UI or native app artifacts.
- A-010 Gap classification: missing evidence.
- A-011 Apple iCloud standard data protection stores Notes encrypted in transit and on server with Apple-held keys.
- A-012 Apple Advanced Data Protection includes Notes in end-to-end encrypted categories and keeps keys on trusted devices.
- A-013 Oyatie coverage: Personal notes are E2E-default and Professional notes are tenant-DEK/server-search capable at `PRD.md:103-113`.
- A-014 Gap classification: competitive privacy model; implementation and recovery UX remain absent.
- A-015 Apple official iCloud sharing text says shared Notes can maintain E2E with Advanced Data Protection only when all participants support it, while some link/web sharing paths are not E2E.
- A-016 Oyatie coverage: PRD says share-link tokens are audited and Personal content stays E2E, but link-share cryptographic downgrade rules are not fully specified.
- A-017 Gap classification: partial; Apple exposes a subtle sharing/privacy nuance that Oyatie should encode.
- A-018 Apple Notes supports locked notes and disallows sharing locked notes.
- A-019 Oyatie coverage: Personal encrypted notes and policy docs imply protected content, but no locked-note UX appears in the PRD list.
- A-020 Gap classification: missing as a user-facing feature, though privacy primitives are stronger.
- A-021 Apple Notes supports folder sharing.
- A-022 Oyatie coverage: notebooks exist, but folder/notebook sharing permission semantics are not explicit.
- A-023 Gap classification: partial.
- A-024 Apple Notes supports smart folders and tags in the user experience.
- A-025 Oyatie coverage: tags and notebooks are documented at `PRD.md:18-26`.
- A-026 Gap classification: partial; smart-folder query semantics are not specified.
- A-027 Apple Notes supports document scanning and image/OCR workflows in its platform ecosystem.
- A-028 Oyatie coverage: web clipper and drive embeds exist, but scan-to-note and OCR are not clear first-class features.
- A-029 Gap classification: missing or cross-service-dependent.
- A-030 Apple Notes supports search across notes and scanned content through OS integration.
- A-031 Oyatie coverage: full-text search and tag search SLOs exist.
- A-032 Gap classification: partial; OCR search is not evidenced.
- A-033 Apple Notes prioritizes simple capture speed.
- A-034 Oyatie coverage: note-open and note-create targets are aggressive at `PRD.md:87-101`.
- A-035 Gap classification: covered as target; unproven by benchmark harness.
- A-036 Apple Notes has low-friction checklist and reminder integration.
- A-037 Oyatie coverage: checklists and reminders are mapped to tasks at `PRD.md:18-26`.
- A-038 Gap classification: partial; the task handoff contract needs UX confirmation.
- A-039 Apple Notes supports rich formatting and attachments without becoming a database workspace.
- A-040 Oyatie coverage: notes positions itself between Notion and Obsidian, with rich blocks and Markdown/frontmatter.
- A-041 Gap classification: covered by intent.
- A-042 Apple Notes’ main weakness versus Obsidian is graph/backlink depth.
- A-043 Oyatie coverage: graph/backlink targets exceed Apple Notes’ visible public surface.
- A-044 Gap classification: Oyatie can lead here if implementation lands.
- A-045 Apple Notes’ main weakness versus Notion is database/workspace breadth.
- A-046 Oyatie coverage: current notes artifacts do not close the database breadth gap.
- A-047 Gap classification: Notion-like databases remain the major family gap.

## §4 UNION-Coverage Matrix

| # | Union capability | Notion | Obsidian | Apple Notes | Oyatie current evidence | Classification |
|---|---|---|---|---|---|---|
| U-001 | Fast note create/open | Present | Present | Present | `PRD.md:87-101`; OpenSLOs | Documented-only |
| U-002 | Rich text blocks | Strong | Markdown-centered | Strong | `PRD.md:18-26`; OpenAPI blocks | Partial |
| U-003 | Markdown/frontmatter | Weak native | Strong | Weak native | `PRD.md:18-26` | Covered |
| U-004 | Backlinks | Weak to partial | Strong | Weak | `PRD.md:18-26` | Covered by spec |
| U-005 | Graph view | Weak | Strong | Weak | `PRD.md:87-101` | Documented-only |
| U-006 | Canvas/freeform visual map | Partial via whiteboards/templates | Strong Canvas | Partial via Freeform adjacency | No notes evidence | Missing |
| U-007 | Notebooks/folders | Present | File folders | Present | `PRD.md:18-26` | Covered by spec |
| U-008 | Smart folders/query collections | Database filters | Search/query plugins | Smart folders | Not explicit | Missing |
| U-009 | Tags | Present | Present | Present | `PRD.md:18-26` | Covered by spec |
| U-010 | Daily notes/journaling | Templates | Strong via core/community | Partial | `PRD.md:18-26` | Covered by spec |
| U-011 | Templates | Strong marketplace | Strong community | Limited | `PRD.md:18-26` | Partial |
| U-012 | Template marketplace | Strong | Community ecosystem | Weak | No notes evidence | Missing |
| U-013 | Databases/tables | Defining strength | Plugin/community | Weak | Import only | Partial |
| U-014 | Kanban/project management | Strong | Plugin/community | Weak | tasks dependency | Cross-service partial |
| U-015 | Checklists | Present | Markdown | Present | `PRD.md:18-26` | Covered by spec |
| U-016 | Reminders | Present | Plugin/community | Present | tasks handoff | Partial |
| U-017 | Comments | Strong | Limited | Collaboration comments less central | Workflow events | Partial |
| U-018 | Real-time collaboration | Strong | Limited shared vault | Strong enough for Apple users | Loro Professional path | Documented-only |
| U-019 | Shared folders/notebooks | Strong | Shared vaults | Strong | Notebooks exist; share semantics unclear | Partial |
| U-020 | Public share links | Strong | Publish | Send copy/link flows | Share-link tokens | Partial |
| U-021 | Site publishing | Strong-ish public pages | Obsidian Publish | Weak | No site-publish surface | Missing |
| U-022 | Web clipper | Strong | Community/extensions | Share sheet capture | PRD/SLO | Documented-only |
| U-023 | Mobile capture | Strong | Present | Strong | No frontend artifacts | Missing evidence |
| U-024 | Desktop native app | Strong | Strong | Strong | No frontend artifacts | Missing evidence |
| U-025 | Web app | Strong | Publish/web limited | iCloud web | Leptos allowed by canon, absent locally | Missing evidence |
| U-026 | Offline mode | Partial | Strong | Strong device cache | Not explicit | Missing |
| U-027 | Local-first vault | Weak | Strong | Device-local with iCloud | Markdown/export only | Partial |
| U-028 | Import from Notion | N/A | Common need | Weak | `PRD.md:18-26` | Covered by spec |
| U-029 | Import from Obsidian | Common need | N/A | Weak | `PRD.md:18-26` | Covered by spec |
| U-030 | Import from Apple Notes | Common need | Common need | N/A | `PRD.md:18-26` | Covered by spec |
| U-031 | Export Markdown | Partial | Native | Weak | `PRD.md:18-26` | Covered by spec |
| U-032 | Export PDF | Present | Present | Present | `PRD.md:18-26` | Covered by spec |
| U-033 | Export JSON | API/database path | Vault files/plugins | Weak | `PRD.md:18-26` | Covered by spec |
| U-034 | API surface | Strong developer API | Plugin/API local | Weak | OpenAPI/AsyncAPI/proto | Partial |
| U-035 | SDK/reference client | Strong ecosystem | Plugin APIs/community | Weak | Rust SDK doc only | Documented-only |
| U-036 | Rate limits documented | Public API limits | Sync quotas public | iCloud quotas external | Missing local rate policy | Missing |
| U-037 | File size quotas | API payload limits | 5MB/200MB Sync file limits | iCloud plan dependent | Missing tenant-class quotas | Missing |
| U-038 | Storage quotas | Workspace/plan dependent | 1GB to 100GB Sync | iCloud plan dependent | Capacity model only | Partial |
| U-039 | E2E private sync | No default full-workspace E2E | Sync E2E default | ADP optional for Notes | Personal E2E default | Strong spec |
| U-040 | Metadata leakage disclosure | Security docs | Explicit Sync metadata caveats | iCloud metadata caveats | Partial privacy docs | Partial |
| U-041 | Locked notes | Not central | Vault-level possible | Strong | No locked-note UX | Missing |
| U-042 | BYOK/compliance | Enterprise features | Limited | Apple-managed | Tenant docs elsewhere, not notes-specific | Partial |
| U-043 | Data residency | Enterprise | Sync regions | Apple region controls limited | notes docs mention residency | Partial |
| U-044 | Legal hold/admin | Enterprise | Weak | Weak | compliance docs, no enforcement proof | Partial |
| U-045 | Audit events | Enterprise | Weak | Weak | `PRD.md:115-121` | Partial |
| U-046 | AI summarization | Strong | Plugin/community | Emerging platform AI | `PRD.md:87-101` | Documented-only |
| U-047 | AI tagging/linking | Strong | Plugin/community | Weak | `PRD.md:87-101` | Documented-only |
| U-048 | AI privacy exclusion | Enterprise controls | Local-first can avoid cloud | Apple ADP/platform controls | `PRD.md:103-113` | Strong spec |
| U-049 | Enterprise search/connectors | Strong | Weak | OS Spotlight/iCloud search | No external connector surface | Missing |
| U-050 | OCR/scanned document search | Partial | Plugin/community | Strong | Not explicit | Missing |
| U-051 | Image/media embedding | Strong | Strong files | Strong | drive embeds | Partial |
| U-052 | Audio/voice capture | Not core | Plugin/community | Platform-adjacent | Not explicit | Missing |
| U-053 | Meeting notes | Notion AI feature | Community | Platform-adjacent | Not explicit | Missing |
| U-054 | Version history | Present | Sync version history | iCloud/versioning limited | Not explicit | Missing |
| U-055 | Conflict resolution | Collaboration engine | Sync merge model | iCloud model | Loro path only | Partial |
| U-056 | CRDT collaboration | Not visible as user term | Not primary | Not primary | Loro proto | Ahead by spec |
| U-057 | Workspace guests | Strong | Shared vault users | Shared-note recipients | tenancy dependency only | Partial |
| U-058 | Revenue-share marketplace path | Marketplace/templates | Community plugins | None | No tenant_class adoption | Missing |
| U-059 | Demo/trial caps | Public plans | Sync plans | iCloud plan caps | No tenant_class adoption | Missing |
| U-060 | Paid per-seat + usage path | Notion plans | Obsidian subscriptions | Apple service plans | No tenant_class adoption | Missing |
| U-061 | Six deployment contexts | SaaS-only | SaaS/local | Apple ecosystem | Missing notes IaC contexts | Missing |
| U-062 | OpenTofu deployability | N/A | N/A | N/A | Helm/Kustomize only | Missing |
| U-063 | OS support manifest | Product-specific | Cross-platform app | Apple-only native | Missing `supported-oses.json` | Missing |
| U-064 | Rust backend implementation | N/A | App-specific | Apple stack | No Rust files | Missing |
| U-065 | Test harness | Vendor internal | Not public | Vendor internal | No notes tests dir | Missing |

## §5 Family Summary

- FS-001 The notes PRD correctly defines a hybrid target: Notion-style rich workspace, Obsidian-style knowledge graph, and Apple Notes-style private native capture.
- FS-002 The Notion family pressure is breadth: databases, workspace admin, templates, AI, comments, permissions, web clipper, integrations, and enterprise search.
- FS-003 The Obsidian family pressure is ownership: Markdown, local-first files, backlinks, graph, canvas, plugins, and private sync.
- FS-004 The Apple Notes family pressure is capture ergonomics: device-native UX, scan/OCR, folders, tags, locked notes, iCloud collaboration, and simple sharing.
- FS-005 Oyatie is strongest on declared product intent and privacy doctrine.
- FS-006 Oyatie is weakest on deployable implementation evidence.
- FS-007 Oyatie can exceed Notion and Apple on E2E-default personal notes if cryptographic and UX implementation lands.
- FS-008 Oyatie can exceed Apple on backlinks and graph if graph rendering and link parsing land.
- FS-009 Oyatie can approach Obsidian on Markdown ownership if local-vault semantics, offline sync, and conflict handling land.
- FS-010 Oyatie does not yet approach Notion on native databases.
- FS-011 Oyatie does not yet approach Obsidian on plugin ecosystem or canvas.
- FS-012 Oyatie does not yet approach Apple Notes on native scan/OCR capture.
- FS-013 The current OpenAPI proves API intention, not product parity.
- FS-014 The current AsyncAPI proves event vocabulary, not production event delivery.
- FS-015 The current proto proves collaboration/privacy intent, not CRDT runtime readiness.
- FS-016 The absence of Rust code and tests means every implementation parity claim remains unverified.
- FS-017 The missing six-context IaC means deployable-context parity is not established.
- FS-018 The missing tenant_class semantics means pricing/usage parity cannot be assessed under the current doctrine.
- FS-019 Existing retired tier language is a cleanup blocker because it misframes capability quality.
- FS-020 Feature parity should be measured against one uniform quality bar, with only infrastructure and tenant usage overlays.

## §6 Headline Gap Analysis

- HG-001 P1 gap: no notes implementation surface exists, despite PRD and implementation plans claiming 111 crates; evidence is `IP-002-cargo-workspace-bootstrap.md:19-25` versus no `.rs` files under the path.
- HG-002 P1 gap: no tests directory exists under notes, despite PRD acceptance tests listed at `PRD.md:344-360`.
- HG-003 P1 gap: no six-context OpenTofu deployment modules exist, despite canonical context requirements in `specs/master-plan-sequencing.json:704-745`.
- HG-004 P1 gap: no `supported-oses.json` exists, despite canonical OS manifest requirement in `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2646-3044`.
- HG-005 P2 gap: Notion-style database surfaces are import-oriented, not native.
- HG-006 P2 gap: Obsidian Canvas parity is absent.
- HG-007 P2 gap: Obsidian plugin ecosystem parity is absent.
- HG-008 P2 gap: Apple Notes scan/OCR parity is absent.
- HG-009 P2 gap: Apple Notes locked-note UX parity is absent.
- HG-010 P2 gap: native frontend evidence is absent for Swift, Kotlin, WinUI3, and Leptos surfaces.
- HG-011 P2 gap: tenant_class overlays are absent across product, API, cost, capacity, and SLO docs.
- HG-012 P2 gap: rate-limit and quota policies are less concrete than Notion API and Obsidian Sync public numbers.
- HG-013 P2 gap: metadata leakage disclosure needs to match Obsidian Sync and Apple iCloud clarity.
- HG-014 P2 gap: version history and conflict-resolution UX are not first-class.
- HG-015 P2 gap: share-link privacy downgrade rules are not explicit enough.
- HG-016 P2 gap: site publishing is not equivalent to Obsidian Publish or Notion public pages.
- HG-017 P2 gap: template marketplace and revenue-share creator surface are not present.
- HG-018 P2 gap: current benchmark docs rely on retired segmentation and must be replaced by the performance report in this batch.
- HG-019 P3 gap: counterpart list omits Bear, Roam, Logseq, Craft, Reflect, and Mem from the official top-3, but those can stay secondary references.
- HG-020 P3 gap: some docs use broad repeated content-pass language that dilutes actionable feature ownership.

## §7 Additive Surface Recommendations

- AS-001 Add a first-class local-vault mode that stores Markdown/frontmatter files with deterministic sync manifests.
- AS-002 Add an explicit offline-first sync contract: local create, local edit, delayed merge, conflict note generation, and recovery journal.
- AS-003 Add Obsidian Canvas import and a native graph-canvas hybrid surface, or state canvas as a non-goal with rationale.
- AS-004 Add a plugin/extension boundary only if the platform strategy wants Obsidian ecosystem parity; otherwise document a closed-system stance.
- AS-005 Add database/table views for notes collections if Notion parity remains a headline claim.
- AS-006 Add smart folders as saved queries over tags, backlinks, dates, people, attachments, and encryption state.
- AS-007 Add locked-note UX semantics distinct from Personal E2E encryption.
- AS-008 Add scanner/OCR ingestion through the correct native frontend and drive handoff.
- AS-009 Add OCR search policy that distinguishes local OCR, server OCR, and E2E-prohibited server processing.
- AS-010 Add share privacy modes: private collaborator, audited organization link, public copy, and E2E-preserving shared note.
- AS-011 Add link-share downgrade warnings matching Apple’s explicit shared-content caveats.
- AS-012 Add version-history semantics with retention overlays by tenant_class, not feature-quality segmentation.
- AS-013 Add quota tables for remote storage, file size, sync bandwidth, API rate limits, and export jobs.
- AS-014 Add demo_trial caps using the OCI Always Free profile.
- AS-015 Add paid scaling language using per-seat plus usage billing.
- AS-016 Add revenue_share creator/marketplace economics for templates or public knowledge products.
- AS-017 Add Notion import fidelity matrix for databases, formulas, relations, rollups, comments, permissions, and AI blocks.
- AS-018 Add Obsidian import fidelity matrix for Markdown, wikilinks, backlinks, frontmatter, attachments, canvases, plugins, and vault settings.
- AS-019 Add Apple Notes import fidelity matrix for folders, locked notes, tags, scanned docs, shared notes, and attachment metadata.
- AS-020 Add export round-trip tests for Markdown, PDF, JSON, and counterpart-specific migration bundles.
- AS-021 Add Rust parser/resolver crates before claiming graph/backlink parity.
- AS-022 Add real benchmark harness output before repeating latency claims.
- AS-023 Add native frontends or explicit frontend backlog items by supported OS family.
- AS-024 Add OpenTofu modules for all six deployment contexts before claiming deployable-context readiness.
- AS-025 Add `supported-oses.json` with primary, secondary, and experimental OS support classes, not feature-quality classes.
- AS-026 Add API request limits comparable in specificity to Notion’s published 3 requests/second integration limit.
- AS-027 Add sync storage quotas comparable in specificity to Obsidian’s 1GB, 10GB, and 100GB public Sync limits.
- AS-028 Add metadata privacy disclosure comparable in specificity to Apple and Obsidian public docs.
- AS-029 Add enterprise admin controls only where notes owns them; otherwise cite identity, tenancy, and audit-chain handoffs.
- AS-030 Add a minimal notes product walkthrough that proves the core loop: create, edit, tag, link, search, graph, share, export.
- AS-031 Add a Professional collaboration walkthrough that proves Loro path boundaries and Personal-content refusal.
- AS-032 Add a Personal encrypted-note walkthrough that proves no AI or server-search processing touches E2E content.
- AS-033 Add web clipper install/use artifact if web clipper remains a flagship feature.
- AS-034 Add templates as reusable typed note bundles rather than only prose tutorial content.
- AS-035 Add meeting-note capture only if Notion AI meeting notes remain in the target parity set.
- AS-036 Add external connector search only if Notion enterprise search remains in target parity set.
- AS-037 Add mobile quick-capture latency budget and native UX acceptance criteria.
- AS-038 Add accessibility criteria for editor, graph, canvas, and sharing dialogs.
- AS-039 Add internationalization criteria for note metadata, search, OCR, and export.
- AS-040 Add a clear non-goal list to stop counterpart sprawl from becoming unbounded.

## §8 Coverage Verdicts

- CV-001 Against Notion, Oyatie notes is product-ambitious but materially incomplete.
- CV-002 Against Notion, the largest gaps are native databases, workspace automation, enterprise search, template marketplace, and integration breadth.
- CV-003 Against Notion, privacy differentiation is a possible advantage if implementation lands.
- CV-004 Against Obsidian, Oyatie notes has strong graph/backlink intent.
- CV-005 Against Obsidian, the largest gaps are local-vault semantics, canvas, plugin ecosystem, and sync implementation evidence.
- CV-006 Against Obsidian, Personal E2E doctrine is competitive but must be implemented and threat-modeled.
- CV-007 Against Apple Notes, Oyatie notes has stronger graph and knowledge-management ambition.
- CV-008 Against Apple Notes, the largest gaps are native capture UX, scan/OCR, locked-note UX, and frontend evidence.
- CV-009 Across all three, the top missing product family is frontend/user experience implementation.
- CV-010 Across all three, the top missing infrastructure family is six-context OpenTofu deployability.
- CV-011 Across all three, the top missing validation family is tests and benchmarks.
- CV-012 Across all three, the top governance gap is tenant_class adoption.
- CV-013 The current PRD should not be treated as proof of feature parity.
- CV-014 The current contracts should be treated as intent-bearing API drafts.
- CV-015 The current SLOs should be treated as target declarations until a harness exists.
- CV-016 The current benchmark doc should be retired or rewritten under the no-tier amendment.
- CV-017 The union bar is reachable only after implementation, deployment, frontend, and quota artifacts land.
- CV-018 Immediate next artifact for product parity should be a bounded MVP proof path, not another broad matrix.

## §9 Traceability Index

- TR-001 Product purpose: `PRD.md:18-26`.
- TR-002 Function list: `PRD.md:58-83`.
- TR-003 Performance goals: `PRD.md:87-101`.
- TR-004 Privacy model: `PRD.md:103-113`.
- TR-005 Audit model: `PRD.md:115-121`.
- TR-006 Availability: `PRD.md:123-127`.
- TR-007 Events: `PRD.md:220-247`.
- TR-008 Counterpart comparison: `PRD.md:268-301`.
- TR-009 Capacity model: `capacity-model.md:30-40`.
- TR-010 OpenAPI endpoints: `contracts/openapi/notes.yaml:316-502`.
- TR-011 AsyncAPI event split: `contracts/asyncapi/notes-events.yaml:5-8`.
- TR-012 Proto collaboration boundary: `contracts/proto/notes.proto:11-18`.
- TR-013 Manifest dependencies: `manifest.json:383-394`.
- TR-014 Implementation plan crate claim: `implementation-plans/IP-002-cargo-workspace-bootstrap.md:19-25`.
- TR-015 IaC plan mismatch: `implementation-plans/IP-001-iac.md:30-45`.
- TR-016 Notion public capability source: `https://www.notion.com/help`.
- TR-017 Notion public API limit source: `https://developers.notion.com/reference/request-limits`.
- TR-018 Obsidian backlinks source: `https://obsidian.md/help/plugins/backlinks`.
- TR-019 Obsidian Canvas source: `https://obsidian.md/help/plugins/canvas`.
- TR-020 Obsidian Sync plan source: `https://obsidian.md/help/sync/plans`.
- TR-021 Obsidian Sync security source: `https://obsidian.md/help/sync/security`.
- TR-022 Apple Notes sharing source: `https://support.apple.com/en-euro/guide/notes/apda5307056b/mac`.
- TR-023 Apple iCloud security source: `https://support.apple.com/en-ie/102651`.
- TR-024 Chat counterpart source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.

## §10 Closing Matrix Judgment

- CJ-001 Notes has the right counterpart set.
- CJ-002 Notes has a credible product thesis.
- CJ-003 Notes has enough prose to guide implementation.
- CJ-004 Notes does not yet have enough code to prove implementation.
- CJ-005 Notes does not yet have enough IaC to prove deployability.
- CJ-006 Notes does not yet have enough frontend surface to compete with native note products.
- CJ-007 Notes does not yet have enough quota and tenant_class policy to replace old segmentation.
- CJ-008 Notes should retain the Notion/Obsidian/Apple triad, but treat it as a union bar, not a loose inspiration list.
- CJ-009 The first parity-critical slice is core loop implementation: create, edit, tag, backlink, search, graph, share, export.
- CJ-010 The second parity-critical slice is privacy proof: Personal E2E refusal of AI/server search and Professional controlled collaboration.
- CJ-011 The third parity-critical slice is deployment proof: six-context OpenTofu, OS manifest, and tenant_class overlays.
- CJ-012 The fourth parity-critical slice is UX proof: native quick capture and web editor behavior.
- CJ-013 After those slices, counterpart parity can move from documentation claim to evidence-backed product claim.

## §11 Remediation Map By Counterpart Pressure

- RM-001 Notion pressure should drive native database/table decisions before import claims expand.
- RM-002 Notion pressure should drive API rate-limit and payload-limit documentation comparable to the public Notion API limit page.
- RM-003 Notion pressure should drive workspace permission modeling only where notes owns permission behavior.
- RM-004 Notion pressure should drive template governance if revenue_share creators are expected to sell or embed note systems.
- RM-005 Notion pressure should not drive unrestricted product expansion before the core note loop is implemented.
- RM-006 Obsidian pressure should drive Markdown round-trip tests first.
- RM-007 Obsidian pressure should drive wikilink, backlink, and frontmatter parser implementation before graph UX claims.
- RM-008 Obsidian pressure should drive local-vault and offline-first semantics before plugin ecosystem work.
- RM-009 Obsidian pressure should drive Canvas import only if graph-canvas workflows are accepted as in-scope.
- RM-010 Obsidian pressure should drive metadata leakage disclosure for sync and graph indexes.
- RM-011 Apple pressure should drive native quick-capture UX on Swift, Kotlin, and WinUI3 surfaces.
- RM-012 Apple pressure should drive locked-note and shared-folder rules because users understand those as note-specific controls.
- RM-013 Apple pressure should drive scanner/OCR decisions, especially whether OCR is local-only for Personal encrypted content.
- RM-014 Apple pressure should drive simple sharing distinctions: collaborate, send copy, public link, and private invite.
- RM-015 Apple pressure should not erase the stronger Obsidian-style graph ambition.
- RM-016 Cross-counterpart pressure should drive an explicit non-goal list.
- RM-017 Cross-counterpart pressure should drive a single evidence-backed MVP slice rather than more broad prose.
- RM-018 Cross-counterpart pressure should drive line-item tests tied to every flagship claim.
- RM-019 Cross-counterpart pressure should drive six-context deployability before public-cloud maturity claims.
- RM-020 Cross-counterpart pressure should drive tenant_class economics without changing product quality.
- RM-021 The matrix therefore recommends implementation proof before new capability catalog expansion.
- RM-022 The matrix also recommends retiring old segmentation language before comparing performance again.
- RM-023 The matrix treats external product breadth as a bar to verify, not as permission to copy every feature.
- RM-024 The matrix treats current notes docs as a strong product brief and a weak implementation record.
- RM-025 The matrix stop condition is clear: counterpart parity remains unproven until code, tests, frontend, IaC, and tenant_class overlays exist.
