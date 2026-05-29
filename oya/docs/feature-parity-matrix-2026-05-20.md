# Wave 3 Batch 3.2 Feature Parity Matrix - docs

Audit date: 2026-05-20.
Target microservice: `docs`.
Required top-3 counterparts: Google Docs, Microsoft Word Online, Notion Docs.
Scope: union-coverage bar for collaborative document authoring, not a pricing-tier or feature-tier comparison.
Tier posture: no demo_trial/paid/paid/compliance_pack model is introduced here.
Tenant-class posture: feature quality is uniform across `demo_trial`, `paid`, and `revenue_share`; any differences belong to usage caps, compliance eligibility, support/SLO contracts, and substrate economics.
Local product source: `microservices/docs/PRD.md` lines 20-24 define docs as a native collaborative document substrate parallel to Google Docs, Microsoft Word Web, Notion pages, and Coda.
Local requirement source: `microservices/docs/PRD.md` lines 42-59 define authoring, collaboration, review, sharing, export/import, embed, search, attachment, AI, legal hold, webhook, math, citation, and accessibility requirements.
Local counterpart source: `microservices/docs/PRD.md` lines 262-268 name Google Docs, Microsoft Word Web, and Notion.
Local existing matrix source: `microservices/docs/competitor-parity-matrix.md` lines 28-30 name the same top-3 counterparts.
External source: Google Docs API limits, https://developers.google.com/docs/api/limits.
External source: Google Docs sharing help, https://support.google.com/docs/answer/2494822.
External source: Google Docs version history help, https://support.google.com/docs/answer/190843.
External source: Microsoft Word real-time coauthoring help, https://support.microsoft.com/en-gb/office/collaborate-on-word-documents-with-real-time-co-authoring-7dd3040c-3f30-4fdd-bab0-8586492a1f1d.
External source: Microsoft Graph throttling limits, https://learn.microsoft.com/en-us/graph/throttling-limits.
External source: Notion API request limits, https://developers.notion.com/reference/request-limits.
External source: Notion append block children API, https://developers.notion.com/reference/patch-block-children.
External source: Notion comment object API, https://developers.notion.com/reference/comment-object.
Verdict: feature direction is strong, but the local matrix must be current-source-backed and disentangled from retired tiers.

## §1 Counterpart Capability Surface - Google Docs

- Google capability G-001: real-time document editing and visible collaborator presence.
- Google capability G-002: share roles for viewer, commenter, and editor.
- Google capability G-003: up to 100 people can simultaneously view, edit, or comment on a shared file according to Google Docs sharing help.
- Google capability G-004: version history can show and restore earlier versions according to Google Docs version history help.
- Google capability G-005: suggesting mode supports reviewer-style edit proposals.
- Google capability G-006: comments and comment resolution support review workflows.
- Google capability G-007: document outline, headings, lists, tables, images, and drawing integration support mainstream rich documents.
- Google capability G-008: smart canvas elements provide chips and embedded collaboration surfaces.
- Google capability G-009: Drive/Workspace integration provides storage, sharing, ownership transfer, and admin policy surface.
- Google capability G-010: Docs API supports document read/write automation with quotas.
- Google capability G-011: Docs API public quota includes 3,000 read requests per minute per project and 300 per minute per user per project.
- Google capability G-012: Docs API public quota includes 600 write requests per minute per project and 60 per minute per user per project.
- Google capability G-013: import/export via Drive supports common office formats, though fidelity varies by document feature.
- Google capability G-014: offline editing exists through Google Workspace client support.
- Google capability G-015: Workspace admin, Vault, DLP, retention, and audit logs provide enterprise controls.
- Google capability G-016: accessibility tooling and screen reader support are mature relative to most web editors.
- Google capability G-017: Gemini/Workspace AI features add summarization, writing assistance, and smart composition.
- Google capability G-018: weak spot for Oyatie differentiation is per-block ACL; Google sharing is primarily document-scoped and range/comment-scoped.
- Google capability G-019: weak spot for Oyatie differentiation is tamper-evident lifecycle audit on every document event.
- Google capability G-020: weak spot for Oyatie differentiation is first-class cross-microservice embedding with Cedar policy propagation.
- Google capability G-021: Oyatie must not underbid Google on editor responsiveness or collaboration reliability.
- Google capability G-022: Oyatie must account for Google’s ecosystem lock-in through import, export, and coexistence paths.
- Google capability G-023: Oyatie’s PRD lines 281-284 already positions CRDT, per-block ACL, embed resolver, and dual-context isolation as differentiators.
- Google capability G-024: Oyatie needs public-read/publish policy clarity because Notion and Google both support share-link publication modes.
- Google capability G-025: Oyatie needs migration tooling that preserves comments, suggestions, permissions, and version history where possible.

## §2 Counterpart Capability Surface - Microsoft Word Online

- Microsoft capability M-001: real-time coauthoring allows collaborators to edit the same Word document and see changes.
- Microsoft capability M-002: Word Online inherits the Microsoft Office/OOXML document ecosystem.
- Microsoft capability M-003: comments, threaded review, and track-changes heritage are central expectations.
- Microsoft capability M-004: Word format fidelity is the highest bar for DOCX import/export.
- Microsoft capability M-005: tables, citations, footnotes, headers, page layout, images, comments, and style metadata are core.
- Microsoft capability M-006: OneDrive and SharePoint provide storage, sharing, links, enterprise retention, and eDiscovery substrate.
- Microsoft capability M-007: Microsoft Graph provides API automation, with OneDrive service throttle documentation including large app-level request ceilings.
- Microsoft capability M-008: Word Online has strong enterprise identity and admin integration through Microsoft 365.
- Microsoft capability M-009: Microsoft Purview, eDiscovery, retention, and compliance tooling shape enterprise expectations.
- Microsoft capability M-010: Copilot in Word sets the expected bar for embedded AI assistance.
- Microsoft capability M-011: desktop Word compatibility creates an expectation that web edits round-trip to desktop Word.
- Microsoft capability M-012: offline desktop editing is a Microsoft advantage even when the comparison target is Word Online.
- Microsoft capability M-013: accessibility, screen reader support, and enterprise localization are mature.
- Microsoft capability M-014: Microsoft’s weak spot for Oyatie differentiation is Notion-style block model with per-block ACL.
- Microsoft capability M-015: Microsoft’s weak spot for Oyatie differentiation is audit-chain evidence at every document lifecycle event.
- Microsoft capability M-016: Microsoft’s weak spot for Oyatie differentiation is cross-service embedding under Oyatie policy semantics.
- Microsoft capability M-017: Oyatie must treat OOXML fidelity as a first-class acceptance criterion, not an export afterthought.
- Microsoft capability M-018: Oyatie’s PRD lines 337-338 already defines a >=95 percent OOXML round-trip target.
- Microsoft capability M-019: Oyatie must support track-change semantics sufficiently to preserve author/reviewer state.
- Microsoft capability M-020: Oyatie must clarify whether PDF/A and legal evidence flows map to document export or an external signature workflow.
- Microsoft capability M-021: Oyatie must ensure table layout, pagination, comments, and citations survive import/export workflows.
- Microsoft capability M-022: Oyatie must expose compliance pack behavior without using retired capability tiers.
- Microsoft capability M-023: Oyatie must make `paid` and `revenue_share` enterprise controls eligible without lowering demo_trial product quality.
- Microsoft capability M-024: Oyatie must keep the Rust backend boundary even if external users expect Graph-compatible generated SDKs.
- Microsoft capability M-025: Oyatie should separate Graph-compatible API shims from the native docs API.

## §3 Counterpart Capability Surface - Notion Docs

- Notion capability N-001: block-based pages are the core editing unit.
- Notion capability N-002: pages can contain rich blocks, callouts, headings, lists, tables, media, code, embeds, and synced content.
- Notion capability N-003: comments and discussions exist on pages and blocks.
- Notion capability N-004: page history/versioning is an expected workspace feature.
- Notion capability N-005: pages can be shared publicly or inside workspaces with permission controls.
- Notion capability N-006: databases and documents are tightly integrated.
- Notion capability N-007: templates and reusable page patterns are core user experience primitives.
- Notion capability N-008: API rate limits are public: average three requests per second per integration, with bursts allowed.
- Notion capability N-009: API payloads are constrained by size, including 1,000 block elements and 500 KB payload limits.
- Notion capability N-010: append block children API caps request granularity, commonly no more than 100 children per request.
- Notion capability N-011: rich text and URL limits shape integration design.
- Notion capability N-012: Notion AI sets expectations for summarize, improve writing, translate, and transform workflows.
- Notion capability N-013: Notion’s weak spot for Oyatie differentiation is enterprise-grade DOCX round-trip fidelity.
- Notion capability N-014: Notion’s weak spot for Oyatie differentiation is Microsoft-style track changes.
- Notion capability N-015: Notion’s weak spot for Oyatie differentiation is tamper-evident audit chain and compliance-pack depth.
- Notion capability N-016: Oyatie must meet Notion-class block ergonomics before claiming block model parity.
- Notion capability N-017: Oyatie’s PRD line 22 includes block types that broadly match Notion’s authoring surface.
- Notion capability N-018: Oyatie’s PRD line 48 requires per-block visibility, a differentiator if backed by policy and tests.
- Notion capability N-019: Oyatie’s embed resolver must make workflow-studio, sheets, and slides embeddings feel native.
- Notion capability N-020: Oyatie needs public-read and publish behavior clarity to match Notion’s common publication workflows.
- Notion capability N-021: Oyatie needs page/database boundary clarity; docs should not silently absorb a database product.
- Notion capability N-022: Oyatie needs migration playbooks that preserve block hierarchy, comments, embeds, and database links.
- Notion capability N-023: Oyatie needs performance targets for large block trees, not only text documents.
- Notion capability N-024: Oyatie must not gate block richness by feature tier; all tenant classes inherit the same feature surface.
- Notion capability N-025: Oyatie should express demo_trial caps as usage and storage ceilings, not block-type limitations.

## §4 Union-Coverage Matrix

| ID | Capability | Google Docs | Microsoft Word Online | Notion Docs | Oyatie evidence | Audit result |
|---|---|---|---|---|---|---|
| U-001 | Create document with title and initial content | covered | covered | covered | `PRD.md:42` FR-01 | covered |
| U-002 | Rich paragraph editing | covered | covered | covered | `PRD.md:22`, `PRD.md:43` | covered |
| U-003 | Headings and outline semantics | covered | covered | covered | `PRD.md:43`, `competitor-parity-matrix.md:53` | covered |
| U-004 | Ordered and unordered lists | covered | covered | covered | `PRD.md:43` | covered |
| U-005 | Checklists/tasks inside documents | partial | partial | covered | `contracts/proto/docs.proto:32-40` begins block enum; local exact checklist evidence needs deeper contract citation | partial |
| U-006 | Tables | covered | covered | covered | `PRD.md:43` | covered |
| U-007 | Images and alt text | covered | covered | covered | `PRD.md:54`, `competitor-parity-matrix.md:56` | covered |
| U-008 | Embeds | covered | partial | covered | `PRD.md:52`, `PRD.md:117` | covered |
| U-009 | Code blocks | partial | partial | covered | `PRD.md:43` | covered |
| U-010 | Math equations | partial | covered | partial | `PRD.md:58` | covered |
| U-011 | Citations/BibTeX | partial | covered | missing | `PRD.md:58`, `competitor-parity-matrix.md:61` | partial |
| U-012 | Callouts/blockquotes | covered | partial | covered | `PRD.md:43` | covered |
| U-013 | Mermaid diagrams | missing | missing | missing | existing benchmark claims support at `benchmarks/...md:45`, but PRD line 43 does not name Mermaid | partial |
| U-014 | Drawing/canvas embed | covered | covered | missing | benchmark claims drawing at `benchmarks/...md:45`; PRD line 52 covers workflow-studio embed | partial |
| U-015 | Reusable templates | covered | covered | covered | no explicit root requirement found | missing |
| U-016 | Smart chips / mentions | covered | covered | covered | `competitor-parity-matrix.md:72` says mentions via messenger | partial |
| U-017 | Real-time coediting | covered | covered | covered | `PRD.md:44`, `PRD.md:70` | covered |
| U-018 | Cursor presence | covered | covered | covered | `PRD.md:70`, SLO `collab-cursor-sync-latency` | covered |
| U-019 | CRDT/OT conflict-free merge | covered | covered | covered | `PRD.md:31`, `PRD.md:44` | covered |
| U-020 | Zero silent edit loss | partial | partial | partial | `PRD.md:31`, `PRD.md:340` | covered if tests land |
| U-021 | Conflict resolution UI | partial | partial | partial | `PRD.md:31`, runbook `collab-conflict-resolution.md` | partial |
| U-022 | Comments | covered | covered | covered | `PRD.md:45` | covered |
| U-023 | Threaded comments | covered | covered | covered | `PRD.md:45`, `contracts/proto/docs.proto` comment messages sampled | covered |
| U-024 | Suggested edits | covered | covered | partial | `PRD.md:46` | covered |
| U-025 | Track-change author identity | covered | covered | partial | `PRD.md:46`, `PRD.md:89` | partial |
| U-026 | Accept/reject suggestions | covered | covered | partial | `PRD.md:46`, `PRD.md:89` | covered |
| U-027 | Version history | covered | covered | covered | `PRD.md:47`, `PRD.md:114` | covered |
| U-028 | Restore previous version | covered | covered | covered | `PRD.md:47`, runbook `doc-version-restore-corruption.md` | covered |
| U-029 | Named versions | covered | covered | partial | no explicit evidence found | missing |
| U-030 | Audit trail of lifecycle events | partial | covered | partial | `PRD.md:89` | covered |
| U-031 | Tamper-evident audit chain | missing | partial | missing | `PRD.md:89`, existing parity matrix `:74` | covered if audit-chain binding lands |
| U-032 | Whole-document share roles | covered | covered | covered | `PRD.md:49` | covered |
| U-033 | Per-block ACL | missing | missing | partial | `PRD.md:48`, ADR-DOCS-0004 | covered |
| U-034 | Public read links | covered | covered | covered | `policy/public-read.cedar` exists | partial |
| U-035 | Link expiration | covered | covered | partial | no explicit evidence found | missing |
| U-036 | Domain/workspace restricted sharing | covered | covered | covered | Cedar policy and tenancy dependency implied | partial |
| U-037 | External guest sharing | covered | covered | covered | `PRD.md:49`; guest edge cases not explicit | partial |
| U-038 | Legal hold | covered | covered | partial | `PRD.md:56`, `PRD.md:89-90` | covered |
| U-039 | Retention policy | covered | covered | partial | `PRD.md:91`, policy files | covered |
| U-040 | Data residency | covered | covered | partial | `PRD.md:100-102`, `policy/data-residency.md` | covered |
| U-041 | Compliance pack activation | covered | covered | partial | `manifest.json:375-397`, compliance docs | partial, tenant-class missing |
| U-042 | BYOK eligibility | covered | covered | partial | architecture mentions provider-BYOK fields; tenant-class missing | partial |
| U-043 | Demo/trial usage caps | partial | partial | partial | no `demo_trial` found | missing |
| U-044 | Paid contractual SLO behavior | covered | covered | covered | no `paid` tenant-class found | missing |
| U-045 | Revenue-share tenant economics | missing | missing | missing | no `revenue_share` found | missing |
| U-046 | DOCX import | covered | covered | partial | `PRD.md:51`, `PRD.md:338` | covered |
| U-047 | DOCX export | covered | covered | partial | `PRD.md:50`, `PRD.md:300` | covered |
| U-048 | OOXML round-trip target | partial | covered | partial | `PRD.md:338` | covered |
| U-049 | PDF export | covered | covered | partial | `PRD.md:50`, `PRD.md:73` | covered |
| U-050 | PDF/A archival export | missing | covered | missing | `PRD.md:50`, `PRD.md:286` | covered |
| U-051 | Markdown import | partial | partial | covered | `PRD.md:51` | covered |
| U-052 | Markdown export | partial | partial | covered | `PRD.md:50` | covered |
| U-053 | HTML import | covered | covered | partial | `PRD.md:51`, `PRD.md:83` | covered |
| U-054 | HTML export | covered | covered | partial | `PRD.md:50` | covered |
| U-055 | EPUB export | missing | missing | missing | `PRD.md:50`, `PRD.md:74` | covered |
| U-056 | LaTeX export | partial | missing | missing | `PRD.md:50`, `PRD.md:75` | covered |
| U-057 | ODT export | partial | covered | missing | FAQ mentions ODT but in retired tier context | partial |
| U-058 | Structured content export | missing | partial | missing | FAQ mentions DITA/eCTD under retired tiers | partial |
| U-059 | Import sanitation | covered | covered | partial | `PRD.md:83` | covered |
| U-060 | Attachment scanning | covered | covered | partial | `PRD.md:83` | covered |
| U-061 | Macro disablement | covered | covered | missing | `PRD.md:83` | covered |
| U-062 | Export sandboxing | partial | partial | missing | `PRD.md:82` | covered |
| U-063 | Search within document | covered | covered | covered | `PRD.md:53`, `PRD.md:71` | covered |
| U-064 | Search structured filters | partial | covered | covered | `PRD.md:53` | covered |
| U-065 | Doc list pagination | covered | covered | covered | `PRD.md:72`, proto `ListDocuments` | covered |
| U-066 | Attachments | covered | covered | covered | `PRD.md:54`, `PRD.md:77` | covered |
| U-067 | Image/PDF/video attachments | covered | covered | partial | `PRD.md:54` | covered |
| U-068 | Cross-doc embedding | covered | partial | covered | `PRD.md:35`, `PRD.md:52` | covered |
| U-069 | Workflow-studio canvas embed | missing | missing | missing | `PRD.md:52` | covered, Oyatie-specific |
| U-070 | Sheets cell embed | partial | partial | covered | `PRD.md:35`, `PRD.md:52` | covered |
| U-071 | Slides deck embed | covered | covered | partial | `PRD.md:52` | covered |
| U-072 | Embed stale fallback | partial | partial | partial | `PRD.md:98` | covered |
| U-073 | Embed mTLS source fetch | missing | missing | missing | `PRD.md:84` | covered, Oyatie-specific |
| U-074 | AI grammar suggestion | covered | covered | covered | `PRD.md:55` | covered |
| U-075 | AI summarization | covered | covered | covered | `PRD.md:55` | covered |
| U-076 | AI translation | covered | covered | covered | `PRD.md:55` | covered |
| U-077 | AI policy deny by context | partial | partial | partial | `PRD.md:85`, ADR-DOCS-0005 | partial |
| U-078 | No cross-tenant training | partial | partial | partial | `PRD.md:85` | covered |
| U-079 | WCAG 2.2 AA | covered | covered | partial | `PRD.md:59`, `PRD.md:92` | covered |
| U-080 | Screen reader export evidence | partial | partial | partial | `PRD.md:92` | partial |
| U-081 | Keyboard navigation | covered | covered | partial | not explicit in PRD | missing |
| U-082 | Localization | covered | covered | covered | no docs-specific evidence found | missing |
| U-083 | Mobile editor | covered | covered | covered | no docs-specific Swift/Kotlin plan found | missing |
| U-084 | Desktop/web editor | covered | covered | covered | no Leptos/WASM editor plan found under docs | missing |
| U-085 | Offline editing | covered | covered | partial | no local evidence found | missing |
| U-086 | API read automation | covered | covered | covered | OpenAPI/proto contracts exist | covered |
| U-087 | API write automation | covered | covered | partial | OpenAPI/proto contracts exist | covered |
| U-088 | Webhooks/lifecycle events | partial | partial | covered | `PRD.md:57`, AsyncAPI contract | covered |
| U-089 | Event replay/backfill | partial | partial | missing | `backfill-replay.md` exists | partial |
| U-090 | SDK reference implementation | covered | covered | covered | `reference-implementations/create-collab-and-export-rust-sdk.md` | covered |
| U-091 | Generated SDK language policy | covered | covered | covered | proto `go_package` needs classification | partial |
| U-092 | Admin audit log | covered | covered | partial | `PRD.md:89`, audit-chain dependency | covered |
| U-093 | DLP integration | covered | covered | partial | not explicit in docs PRD | partial |
| U-094 | eDiscovery | covered | covered | partial | legal hold exists; eDiscovery flow not explicit | partial |
| U-095 | Encryption at rest | covered | covered | covered | `PRD.md:81` | covered |
| U-096 | Tenant DEK | partial | partial | missing | `PRD.md:81`, `PRD.md:85` | covered |
| U-097 | E2E personal docs mode | covered | partial | partial | `PRD.md:81` | partial |
| U-098 | Dual personal/professional contexts | partial | partial | partial | `PRD.md:24`, `PRD.md:36` | covered |
| U-099 | Cross-context share grants | partial | partial | partial | `PRD.md:24` | partial |
| U-100 | Per-jurisdiction retention | covered | covered | partial | `PRD.md:91` | covered |
| U-101 | Multi-region deployment | covered | covered | partial | `multi-region.md` exists; OpenTofu contexts missing | partial |
| U-102 | On-prem deployment | partial | partial | partial | canonical requires it; local OpenTofu missing | missing |
| U-103 | Colo deployment | partial | partial | partial | canonical requires it; local OpenTofu missing | missing |
| U-104 | Guest-on-AWS deployment | partial | partial | partial | canonical requires it; local OpenTofu missing | missing |
| U-105 | Guest-on-OCI deployment | partial | partial | partial | canonical requires it; local OpenTofu missing | missing |
| U-106 | Oyatie-as-cloud-provider deployment | partial | partial | partial | canonical requires it; local OpenTofu missing | missing |
| U-107 | OCI Always Free profile | missing | missing | missing | canonical requires `iac/oci-guest/always-free/`; local missing | missing |
| U-108 | OpenTofu IaC | not applicable | not applicable | not applicable | local only Helm/Kustomize | missing |
| U-109 | OS support manifest | not applicable | not applicable | not applicable | no `supported-oses.json` | missing |
| U-110 | Root service README | covered | covered | covered | no root `README.md` | missing |
| U-111 | Cross-microservice handoff doc | not comparable | not comparable | not comparable | no `cross-microservice-handoffs.md` | missing |
| U-112 | Source implementation files | covered | covered | covered | no top-level `src/` | missing |
| U-113 | Test files | covered | covered | covered | no top-level `tests/` | missing |
| U-114 | SLO runbooks | covered | covered | partial | local runbooks and SLOs exist | covered |
| U-115 | Non-tiered benchmark targets | covered | covered | covered | old benchmark is tiered; this batch supplies replacement | partial |
| U-116 | Demo/trial no compliance pack | partial | partial | partial | no tenant-class evidence | missing |
| U-117 | Paid compliance pack eligibility | covered | covered | partial | no tenant-class evidence | missing |
| U-118 | Revenue-share at-cost substrate | missing | missing | missing | no tenant-class evidence | missing |
| U-119 | Uniform feature quality across classes | not comparable | not comparable | not comparable | current prompt requires it; local docs absent | missing |
| U-120 | Wave 15J tier retirement readiness | not comparable | not comparable | not comparable | tier refs cataloged in coherence audit | missing until cleanup |

## §5 Capability Family Summary

- Family F-01 Core editing: mostly covered; templates, keyboard navigation, localization, mobile/native editor, and offline mode need explicit artifacts.
- Family F-02 Collaboration: strongly covered in PRD and SLOs; conflict UI, named versions, and author identity preservation need sharper contracts.
- Family F-03 Sharing and permissions: strong on per-block ACL ambition; weak on link expiration, domain restriction details, and public publishing semantics.
- Family F-04 Import/export: strong ambition; Microsoft Word Online sets the highest DOCX fidelity bar.
- Family F-05 Blocks and embeds: strong Notion-style product direction; database boundary and workflow/sheets/slides embed contracts need clarity.
- Family F-06 AI assist: present but must remain policy-bounded and tenant-DEK protected; no tier gating should remain.
- Family F-07 Compliance: strong security posture; tenant-class eligibility and deployment-context enforceability are missing.
- Family F-08 APIs and events: three contract families exist; generated SDK language boundaries need classification.
- Family F-09 Operations: SLOs and runbooks exist; deployment overlays and OS matrix are missing.
- Family F-10 Canonical platform: six-context OpenTofu, OCI Always Free, tenant classes, and OS support are the largest parity blockers.

## §6 Headline Gap Analysis

- Gap H-001: Google-style 100-person active collaboration is a minimum external expectation; Oyatie targets much higher in old benchmarks but those claims must be revalidated without tiers.
- Gap H-002: Microsoft-style OOXML round-trip fidelity is a hard requirement; PRD has a >=95 percent acceptance criterion, but the import/export test corpus is not present under the microservice path.
- Gap H-003: Notion-style block ergonomics requires more than a block enum; templates, public pages, databases, synced blocks, and embed UX need explicit product decisions.
- Gap H-004: The local docs path has contract files but no implementation or tests, so parity remains design-level until code lands.
- Gap H-005: The service cannot claim all deployment contexts until OpenTofu modules exist.
- Gap H-006: The service cannot claim demo_trial readiness until OCI Always Free profile caps and tenant-class usage caps exist.
- Gap H-007: The service cannot claim paid/revenue_share readiness until billing meters and contractual SLO surfaces exist.
- Gap H-008: The service cannot claim no-tenant-class-drift compliance until all demo_trial/paid/paid/compliance_pack references are retired.
- Gap H-009: The service cannot claim Rust-strict implementation completeness while docs-site plans still mention SvelteKit/Backstage without an allowed runtime boundary.
- Gap H-010: The service needs a root README and handoff document so product scope is not confused with documentation-site tooling.

## §7 Additive Surface Recommendations

- Add AS-001: `README.md` describing docs as the collaborative editor product, not a docs-site build pipeline.
- Add AS-002: `tenant-class-behavior.md` or machine-readable equivalent listing docs meters and class overlays for `demo_trial`, `paid`, and `revenue_share`.
- Add AS-003: `supported-oses.json` with explicit backend, web, mobile, and native-editor support decisions.
- Add AS-004: OpenTofu modules for the six canonical contexts.
- Add AS-005: `iac/oci-guest/always-free/` for demo_trial OCI Always Free profile constraints.
- Add AS-006: `cross-microservice-handoffs.md` mapping events, APIs, owners, retry semantics, and failure handling.
- Add AS-007: an import/export fidelity scorecard for DOCX, Markdown, HTML, PDF/A, EPUB, LaTeX, Notion blocks, and Google Docs exports.
- Add AS-008: a collaboration scale test spec for 10, 50, 100, 500, 1,000, and 10,000 editor simulations without tier names.
- Add AS-009: a block-model conformance matrix for Notion-style blocks, Google smart-canvas analogs, and Word layout constructs.
- Add AS-010: a generated-SDK policy note that classifies proto generator metadata without weakening Rust backend policy.
- Add AS-011: a public-read/publish decision record for Notion-style pages and Google-style share links.
- Add AS-012: a link expiration and domain restriction policy for external sharing.
- Add AS-013: a keyboard navigation and accessibility acceptance spec tied to WCAG 2.2 AA.
- Add AS-014: a native-editor or web-only decision for Swift/Kotlin/WinUI3/Leptos surfaces.
- Add AS-015: an offline editing decision record because Google and Microsoft both create user expectations here.
- Add AS-016: a named-version and restore contract to match Google/Microsoft version history behavior.
- Add AS-017: a track-change identity model for Microsoft Word Online parity.
- Add AS-018: a template/library model for Google/Notion parity.
- Add AS-019: a database boundary ADR deciding whether docs consumes sheets/database blocks or owns database-like content.
- Add AS-020: a no-tenant-class-drift cleanup plan targeting the exact TR references from the coherence audit.

## §8 Family-by-Family Verdicts

- Verdict V-001 Core editor: REVISE, because editing requirements are broad but root implementation evidence is absent.
- Verdict V-002 Collaboration: PASS WITH FINDINGS, because CRDT, comments, suggestions, cursor, and version history are named.
- Verdict V-003 Permissions: PASS WITH FINDINGS, because per-block ACL is a strong differentiator but link/public/domain behavior needs detail.
- Verdict V-004 Import/export: PASS WITH FINDINGS, because targets exist but fidelity corpus and tests are not local.
- Verdict V-005 Blocks/embeds: PASS WITH FINDINGS, because Notion-like blocks and cross-service embeds are present but database/page boundary is unclear.
- Verdict V-006 AI assist: REVISE, because T0/T1/T2 vocabulary should be separated from retired vocabulary and tenant-class policy.
- Verdict V-007 Compliance: REVISE, because tenant-class eligibility and deployment enforceability are missing.
- Verdict V-008 APIs/events: PASS WITH FINDINGS, because three contract families exist.
- Verdict V-009 Operations: REVISE, because SLOs/runbooks exist but OpenTofu contexts and OS manifest are absent.
- Verdict V-010 Counterpart union: REVISE, because Google/Microsoft/Notion coverage is credible but not fully evidenced with current non-tier artifacts.

## §9 No-Tier Confirmation

- This matrix does not introduce demo_trial, paid, paid, or compliance_pack headings.
- Existing tier strings are referenced only as findings in the coherence audit.
- The benchmark comparison uses counterpart capability surfaces, not feature tiers.
- Tenant class is treated as billing/substrate/support/compliance eligibility, not product quality.
- Demo_trial should receive the same feature quality with hard usage caps and best-effort SLO.
- Paid should receive contractual SLO, compliance pack eligibility, and BYOK eligibility.
- Revenue_share should receive at-cost or zero-margin substrate economics without quality degradation.
- OCI Always Free is treated as a profile for demo_trial infrastructure, not as a tier.

## §10 Current Source Status

- Google source status: official public docs provide API quota and sharing/version-history behavior, but not full internal latency.
- Microsoft source status: official public docs provide coauthoring behavior and Graph throttling classes, but not Word Online p99 edit latency.
- Notion source status: official public API docs provide request and payload limits, but not editor p99 latency.
- Internal benchmark status: `benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md` provides prior internal browser-harness estimates, but it is tiered and must not be the final current benchmark.
- This matrix therefore separates public-source facts from Oyatie target claims.

## §11 Closing Assessment

- The docs microservice has a strong product thesis against the top-3 counterparts.
- The feature gap is not mostly about feature imagination; it is about making canonical constraints executable and non-tiered.
- The most important additive work is not adding another parity table; it is codifying tenant classes, OpenTofu contexts, OS support, and implementation/test evidence.
- The top-3 counterpart bar is confirmed: Google Docs / Microsoft Word Online / Notion Docs.
- The microservice should be considered parity-ambitious but not parity-proven.

## §12 Evidence-Backed Gap-Closure Backlog

- GC-001 Root README: write the first-viewport product identity as collaborative documents, with docs-site tooling explicitly secondary.
- GC-002 Root README: cite `PRD.md:20-24` as purpose and `PRD.md:110-117` as bounded-context source.
- GC-003 Root README: name Google Docs, Microsoft Word Online, and Notion Docs as the top-3 counterpart bar.
- GC-004 Root README: state that no feature tier model exists and that tenant classes only change caps, contractual SLO, compliance eligibility, support, and economics.
- GC-005 Tenant behavior artifact: define document count meter.
- GC-006 Tenant behavior artifact: define storage byte meter.
- GC-007 Tenant behavior artifact: define active editor session meter.
- GC-008 Tenant behavior artifact: define CRDT operation meter.
- GC-009 Tenant behavior artifact: define export job meter.
- GC-010 Tenant behavior artifact: define import job meter.
- GC-011 Tenant behavior artifact: define attachment byte meter.
- GC-012 Tenant behavior artifact: define AI assist invocation meter.
- GC-013 Tenant behavior artifact: define share-link and public-read event meter.
- GC-014 Tenant behavior artifact: map `demo_trial` caps without hiding features.
- GC-015 Tenant behavior artifact: map `paid` per-seat plus usage billing.
- GC-016 Tenant behavior artifact: map `revenue_share` gross-revenue settlement signals.
- GC-017 OpenTofu: add `iac/oyatie-public-cloud/` for Oyatie-operated public cloud.
- GC-018 OpenTofu: add `iac/guest-on-aws/` for customer AWS accounts.
- GC-019 OpenTofu: add `iac/oci-guest/` for customer OCI tenancy.
- GC-020 OpenTofu: add `iac/oci-guest/always-free/` for demo_trial OCI profile.
- GC-021 OpenTofu: add `iac/on-prem/` for customer facility deployment.
- GC-022 OpenTofu: add `iac/colo/` for colocated deployment.
- GC-023 OpenTofu: add `iac/oyatie-iaas/` for Oyatie-as-cloud-provider.
- GC-024 OpenTofu: specify whether Helm/Kustomize files are rendered outputs, module inputs, or legacy runtime manifests.
- GC-025 OS support: add Linux server rows for the Rust backend.
- GC-026 OS support: add BSD/illumos rows only if the Rust backend and storage dependencies can be validated.
- GC-027 OS support: add Darwin rows for development or native client scope.
- GC-028 OS support: add Windows server/client rows consistent with WinUI3 frontend allowance.
- GC-029 OS support: add iOS Swift client row only if native editor scope is accepted.
- GC-030 OS support: add Android Kotlin client row only if native editor scope is accepted.
- GC-031 OS support: add web/Leptos row for browser support.
- GC-032 Google parity: add automated sharing role tests for viewer/commenter/editor.
- GC-033 Google parity: add 100 active collaborator test as a minimum external compatibility threshold.
- GC-034 Google parity: add version restore flow matching Google version-history expectations.
- GC-035 Google parity: add suggestion-mode acceptance/rejection tests.
- GC-036 Google parity: add import/export coexistence tests for Google-origin documents.
- GC-037 Microsoft parity: add DOCX round-trip corpus with comments.
- GC-038 Microsoft parity: add DOCX round-trip corpus with tracked changes.
- GC-039 Microsoft parity: add DOCX round-trip corpus with tables and nested styles.
- GC-040 Microsoft parity: add DOCX round-trip corpus with citations and footnotes.
- GC-041 Microsoft parity: add DOCX round-trip corpus with headers, footers, and page layout.
- GC-042 Microsoft parity: add reviewer identity preservation checks.
- GC-043 Notion parity: add block tree import/export corpus.
- GC-044 Notion parity: add nested block stress test.
- GC-045 Notion parity: add callout/code/media/embed fixtures.
- GC-046 Notion parity: add synced-block or transclusion decision record.
- GC-047 Notion parity: add public page publishing decision record.
- GC-048 Notion parity: add database boundary decision record.
- GC-049 API parity: publish API request-size limits.
- GC-050 API parity: publish block-append batch limits.
- GC-051 API parity: publish rate-limit semantics by tenant class.
- GC-052 API parity: keep tenant_class out of request payloads if gateway/IAM owns the claim.
- GC-053 API parity: classify generated SDK options in proto as codegen metadata.
- GC-054 API parity: add event replay/backfill contract to AsyncAPI.
- GC-055 Collaboration: add named-version support if Google version parity is required.
- GC-056 Collaboration: add range-stable comment anchor tests after concurrent edits.
- GC-057 Collaboration: add suggestion anchor tests after concurrent edits.
- GC-058 Collaboration: add conflict surfacing UX contract for non-mergeable edits.
- GC-059 Collaboration: add presence fanout tests at 25, 100, 500, 1,000, and 10,000 editors.
- GC-060 Sharing: add link expiration policy.
- GC-061 Sharing: add domain restriction policy.
- GC-062 Sharing: add external guest policy.
- GC-063 Sharing: add public-read audit and revocation policy.
- GC-064 Sharing: add per-block ACL property tests.
- GC-065 Compliance: express compliance-pack eligibility by tenant class.
- GC-066 Compliance: express BYOK eligibility by tenant class.
- GC-067 Compliance: preserve uniform feature quality while denying compliance packs to demo_trial.
- GC-068 Compliance: document legal-hold behavior for paid and revenue_share tenants.
- GC-069 Compliance: document what happens when demo_trial attempts legal-hold or regulated exports.
- GC-070 Accessibility: add keyboard navigation acceptance checks.
- GC-071 Accessibility: add screen-reader acceptance checks.
- GC-072 Accessibility: add export accessibility evidence format.
- GC-073 Accessibility: add mobile accessibility if native clients are in scope.
- GC-074 AI assist: define no-cross-tenant-training evidence.
- GC-075 AI assist: define tenant-DEK prompt wrapping evidence.
- GC-076 AI assist: define policy-deny examples for sensitive HR or regulated contexts.
- GC-077 AI assist: rename capability `tier` fields if those fields are not the canonical autonomy vocabulary.
- GC-078 Operations: rewrite SLO labels from `tier` to a non-tier classification key.
- GC-079 Operations: add deployment-context overlays to SLO expectations.
- GC-080 Operations: add tenant-class cap overlays to load tests.
- GC-081 Operations: add on-prem/colo facility prerequisite checklist.
- GC-082 Operations: add OCI Always Free resource ceiling test.
- GC-083 Benchmark: treat prior internal estimates as inputs, not externally publishable claims.
- GC-084 Benchmark: rerun top-3 browser harness with exact date, browser, region, OS, and account plan.
- GC-085 Benchmark: store raw traces and screenshots in a reproducible evidence location.
- GC-086 Documentation cleanup: retire `tenant-class-adoption/tenant-class-adoption-record.md` in Wave 15J.
- GC-087 Documentation cleanup: rewrite onboarding commands away from tier variables.
- GC-088 Documentation cleanup: rewrite migration commands away from tier variables.
- GC-089 Documentation cleanup: rewrite FAQ performance answers as single targets plus deployment/tenant overlays.
- GC-090 Documentation cleanup: update existing benchmark file or supersede it with this no-tenant-class-drift benchmark schema.
