# Wave 3 Batch 3.2 Microservice Ownership-Coherence Audit - docs

Audit date: 2026-05-20.
Audit owner: single-agent docs owner for this batch.
Target microservice: `docs`.
Microservice path: `/Users/jasonlee/oyatie/microservices/docs/`.
Top-3 counterpart bar: Google Docs, Microsoft Word Online, Notion Docs.
Deliverable set: coherence audit, feature parity matrix, performance benchmark numbers.
Retired deliverable: capability-tier deltas document is intentionally not authored.
Verdict: REVISE before claiming deployment-context-complete or tenant-class-current.
Primary risk: the service has rich product artifacts but does not yet encode the six canonical deployment contexts, OpenTofu-only infrastructure substrate, per-microservice OS support manifest, or the retired-vocabulary replacement model.

## Five-citation Anchor Block

- Canonical sequence source: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md` lines 936-1018 define the D-4 audit shape, read-before-write rule, inventory requirement, canonical-direction check, and verdict vocabulary.
- Canonical multi-context and infrastructure source: `specs/master-plan-sequencing.json` lines 704-775 define six deployment contexts plus OpenTofu as the only IaC engine and forbid Terraform/Pulumi/CloudFormation handoff patterns.
- Canonical OS and language source: `specs/master-plan-sequencing.json` lines 777-855 require per-microservice OS manifests and Rust backend / Swift-Kotlin-WinUI3-Leptos frontend language boundaries.
- Canonical OCI profile source: `specs/master-plan-sequencing.json` lines 857-867 define the OCI Always Free profile and require `iac/oci-guest/always-free/` per microservice.
- Tier-retirement source: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md` lines 10-24 record the 2026-05-20 directive that demo_trial/paid/paid/compliance_pack tiers are being retired, and the current batch prompt supersedes older two-class notes with three tenant classes: `demo_trial`, `paid`, and `revenue_share`.

## §1 Purpose

- This audit evaluates whether the `docs` microservice owns a coherent product, contract, operational, deployment, and compliance story.
- The audit is docs-only: no other microservice files were edited or normalized.
- The audit checks existing evidence, not aspirational product claims without artifact support.
- The audit treats Google Docs, Microsoft Word Online, and Notion Docs as the union-coverage counterpart bar.
- The audit treats Coda, Quip, ONLYOFFICE, Collabora, Confluence, and others as useful context but not the required top-3 bar for this batch.
- The audit uses the current Wave 3 Batch 3.2 instruction that the fourth tier-deltas deliverable is retired.
- Existing demo_trial/paid/paid/compliance_pack language is cataloged as Wave 15J retirement evidence, not copied forward as a new design.
- The audit checks whether `docs` expresses tenant-class semantics for `demo_trial`, `paid`, and `revenue_share`.
- The audit checks whether `docs` can honestly claim all six deployment contexts.
- The audit checks whether infrastructure evidence is OpenTofu-first rather than Helm/Kustomize-only.
- The audit checks whether OCI Always Free is modeled as a profile for demo/trial infrastructure, not as a feature tier.
- The audit checks whether OS support exists as a microservice-scoped manifest.
- The audit checks whether the Rust-strict backend boundary is respected by file extensions and contract metadata.
- The audit checks product purpose against the counterpart surface, not only against internal scaffold count.
- The audit checks contract coverage across OpenAPI, AsyncAPI, and proto.
- The audit checks SLO and runbook coverage for the editor, CRDT, sharing, export, and import paths.
- The audit checks whether benchmark numbers are current enough to support sales or architecture claims.
- The audit checks whether missing README, handoff, src, and tests surfaces weaken ownership coherence.
- The audit checks whether chat history indicates known prior generation context for the docs artifacts.
- The audit stops at audit authoring; it does not implement fixes or rewrite existing artifacts.

### §1.1 Product Purpose Found

- Evidence: `microservices/docs/PRD.md` lines 20-24 define `docs` as Oyatie's native collaborative document substrate parallel to Google Docs, Microsoft Word Web, Notion pages, and Coda.
- Evidence: `microservices/docs/PRD.md` lines 42-59 enumerate authoring, block, CRDT collaboration, comments, suggestions, version history, per-block ACL, sharing, import/export, embed, search, attachment, AI assist, legal hold, webhook, math/citation, and accessibility requirements.
- Evidence: `microservices/docs/PRD.md` lines 67-77 define latency targets for document open, save, cursor sync, search, export, import, comments, and attachment upload.
- Evidence: `microservices/docs/PRD.md` lines 110-117 define eight bounded contexts: `document-store`, `collab-crdt`, `block-types`, `comments-and-suggestions`, `version-history`, `sharing-and-permissions`, `export-import`, and `embed-resolver`.
- Evidence: `microservices/docs/PRD.md` lines 213-237 define produced and consumed workflow events, making docs a workflow-integrated product rather than a standalone text editor only.
- Evidence: `microservices/docs/PRD.md` lines 262-268 name Google Docs, Microsoft Word Web, and Notion as explicit competitive references.
- Evidence: `microservices/docs/competitor-parity-matrix.md` lines 24-42 includes an expanded competitor set but the top-3 batch bar remains Google Docs, Microsoft Word Online, and Notion Docs.
- Product classification: first-party collaborative authoring system with block-oriented content, review, import/export, compliance evidence, and embeddable cross-service composition.
- Counterpart fit: Google Docs covers mainstream real-time writing and sharing; Microsoft Word Online covers OOXML and enterprise review fidelity; Notion Docs covers block model, embeds, pages, and workspace-native docs.
- Counterpart gap implication: the service must simultaneously meet editor collaboration, document-format fidelity, and block-system ergonomics; a partial wiki-only or API-only interpretation would under-scope the product.

### §1.2 Stop Condition Applied

- The audit is complete when the inventory is enumerated, canonical constraints are evaluated, top-3 counterpart parity is analyzed, tier references are cataloged, tenant-class gaps are stated, and the three required deliverables are landed.
- The audit is not complete if it silently writes a fourth tier-deltas artifact.
- The audit is not complete if it uses a line count as a substitute for artifact-specific findings.
- The audit is not complete if tier retirement candidates are only summarized without file:line citations.
- The audit is not complete if the OCI Always Free profile is mislabeled as a tier.
- The audit is not complete if tenant classes are treated as feature quality tiers.

## §2 Complete Inventory

- Inventory count: 128 files under `microservices/docs/` before the three audit deliverables were added.
- Inventory line count read/sampled: 20,096 total lines across the target path before adding this audit set.
- Inventory command evidence: `find microservices/docs -type f | sort | wc -l` returned `128`.
- Line-count command evidence: `find microservices/docs -type f -print0 | xargs -0 wc -l | tail -1` returned `20096 total`.
- Missing expected root README: `find microservices/docs -maxdepth 2 -type f -name README.md` found only `microservices/docs/decisions/README.md`, not a root `README.md`.
- Missing expected OS manifest: `find microservices/docs -maxdepth 2 -type f -name supported-oses.json` returned no file.
- Missing expected cross-microservice handoff file: `find microservices/docs -maxdepth 2 -type f -name cross-microservice-handoffs.md` returned no file.
- Missing expected source/test sample: `find microservices/docs -maxdepth 2 -type d` showed no top-level `src/` or `tests/` directory.

### §2.1 File Inventory

- INV-001 `microservices/docs/ARCHITECTURE.md` - architecture anchor file; strong cross-service prose but still contains tier vocabulary and Helm/Kustomize-only IaC evidence.
- INV-002 `microservices/docs/AUDIT-FINDINGS-2026-05-18.json` - prior audit evidence file; useful historical context but not a Wave 3 Batch 3.2 substitute.
- INV-003 `microservices/docs/IP-001-iac-bootstrap.md` - implementation plan for infrastructure bootstrap; must be reconciled with OpenTofu-only context modules.
- INV-004 `microservices/docs/IP-002-document-store-kernel.md` - document-store kernel implementation plan.
- INV-005 `microservices/docs/IP-003-document-store-domain-and-usecase.md` - document-store domain/usecase plan.
- INV-006 `microservices/docs/IP-004-document-store-adapter-postgres-and-s3.md` - storage adapter plan with Postgres/S3 concerns.
- INV-007 `microservices/docs/IP-005-block-types-kernel-domain.md` - block model kernel/domain plan.
- INV-008 `microservices/docs/IP-006-collab-crdt-kernel-domain.md` - CRDT kernel/domain plan.
- INV-009 `microservices/docs/IP-007-collab-crdt-adapter-valkey-worker.md` - CRDT Valkey/worker plan.
- INV-010 `microservices/docs/IP-008-comments-and-suggestions.md` - review/comment workflow plan.
- INV-011 `microservices/docs/IP-009-version-history.md` - version history plan.
- INV-012 `microservices/docs/IP-010-sharing-and-permissions.md` - sharing and permission plan.
- INV-013 `microservices/docs/IP-011-export-import.md` - export/import plan.
- INV-014 `microservices/docs/IP-012-embed-resolver.md` - embed resolver plan.
- INV-015 `microservices/docs/IP-013-rest-websocket-protocol.md` - REST/WebSocket protocol plan.
- INV-016 `microservices/docs/IP-014-ai-assist-wire.md` - AI assist wire plan.
- INV-017 `microservices/docs/IP-015-hg-docs-registration-and-branch-protection.md` - hyperscaler gate registration plan.
- INV-018 `microservices/docs/IP-DOCS-001-mdbook-techdocs-pipeline.md` - docs pipeline plan; currently a documentation-tool plan, not the collaborative editor product itself.
- INV-019 `microservices/docs/IP-DOCS-002-sveltekit-marketing-site.md` - marketing site plan; SvelteKit conflicts with the current Leptos web allowlist unless retired or re-scoped.
- INV-020 `microservices/docs/IP-DOCS-003-redoc-asyncapi-renderer.md` - API documentation renderer plan; needs Rust/Leptos-compatible delivery posture.
- INV-021 `microservices/docs/IP-DOCS-004-mermaid-c4-build.md` - diagram build plan; not deployment-context evidence.
- INV-022 `microservices/docs/IP-DOCS-005-backstage-techdocs-renderer.md` - Backstage TechDocs renderer plan; should not become a Node/Svelte runtime dependency for the microservice.
- INV-023 `microservices/docs/IP-journey-j100-pack-rollout-first-action.md` - journey overlay plan.
- INV-024 `microservices/docs/IP-journey-j91-us-msb-mtl-overlay.md` - jurisdiction journey overlay.
- INV-025 `microservices/docs/IP-journey-j92-br-lgpd-us-parent-dsar.md` - jurisdiction journey overlay.
- INV-026 `microservices/docs/IP-journey-j93-in-dpdpa-rbi-overlay.md` - jurisdiction journey overlay.
- INV-027 `microservices/docs/IP-journey-j94-sox404-public-company-controls.md` - compliance journey overlay.
- INV-028 `microservices/docs/IP-journey-j95-iso27001-soc2-annual-audit.md` - audit journey overlay.
- INV-029 `microservices/docs/IP-journey-j96-ksa-uae-mena-onboarding.md` - regional onboarding overlay.
- INV-030 `microservices/docs/IP-journey-j97-sg-pdpa-mas-tenant.md` - Singapore/MAS journey overlay.
- INV-031 `microservices/docs/IP-journey-j98-au-privacy-apra-cps234.md` - Australian privacy/APRA journey overlay.
- INV-032 `microservices/docs/IP-journey-j99-multi-pack-conflict-resolution.md` - multi-pack conflict overlay.
- INV-033 `microservices/docs/PHASE-01-DOCS-FOUNDATION.md` - phase foundation plan.
- INV-034 `microservices/docs/PRD.md` - primary product definition and acceptance criteria.
- INV-035 `microservices/docs/backfill-replay.md` - replay/backfill operational document.
- INV-036 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md` - prior benchmark surface with retired paid/compliance_pack schema rows.
- INV-037 `microservices/docs/capabilities/T0-suggest.yaml` - capability manifest using `tier: T0`, a capability-level vocabulary distinct from demo_trial/paid/paid but requiring terminology review.
- INV-038 `microservices/docs/capabilities/T1-assist.yaml` - capability manifest using `tier: T1`.
- INV-039 `microservices/docs/capabilities/T2-auto.yaml` - capability manifest using `tier: T2`.
- INV-040 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md` - explicit demo_trial/paid/paid/compliance_pack tenant_class adoption record; Wave 15J retirement candidate.
- INV-041 `microservices/docs/capacity-model.md` - capacity model; must align with single industry-leader targets plus deployment overlays.
- INV-042 `microservices/docs/catalog/oya-docs-block-types-kernel.yaml` - catalog entry.
- INV-043 `microservices/docs/catalog/oya-docs-collab-crdt-adapter-valkey.yaml` - catalog entry.
- INV-044 `microservices/docs/catalog/oya-docs-collab-crdt-adapter.yaml` - catalog entry.
- INV-045 `microservices/docs/catalog/oya-docs-collab-crdt-kernel.yaml` - catalog entry.
- INV-046 `microservices/docs/catalog/oya-docs-comments-and-suggestions-adapter-postgres.yaml` - catalog entry.
- INV-047 `microservices/docs/catalog/oya-docs-comments-and-suggestions-kernel.yaml` - catalog entry.
- INV-048 `microservices/docs/catalog/oya-docs-document-store-adapter-postgres.yaml` - catalog entry.
- INV-049 `microservices/docs/catalog/oya-docs-document-store-adapter-s3.yaml` - catalog entry.
- INV-050 `microservices/docs/catalog/oya-docs-document-store-app.yaml` - catalog entry.
- INV-051 `microservices/docs/catalog/oya-docs-document-store-kernel.yaml` - catalog entry.
- INV-052 `microservices/docs/catalog/oya-docs-embed-resolver-kernel.yaml` - catalog entry.
- INV-053 `microservices/docs/catalog/oya-docs-export-import-adapter-chromium.yaml` - catalog entry.
- INV-054 `microservices/docs/catalog/oya-docs-export-import-adapter-clamav.yaml` - catalog entry.
- INV-055 `microservices/docs/catalog/oya-docs-export-import-adapter-opswat.yaml` - catalog entry.
- INV-056 `microservices/docs/catalog/oya-docs-export-import-adapter-pandoc.yaml` - catalog entry.
- INV-057 `microservices/docs/catalog/oya-docs-export-import-adapter-weasyprint.yaml` - catalog entry.
- INV-058 `microservices/docs/catalog/oya-docs-export-import-kernel.yaml` - catalog entry.
- INV-059 `microservices/docs/catalog/oya-docs-sharing-and-permissions-kernel.yaml` - catalog entry.
- INV-060 `microservices/docs/catalog/oya-docs-version-history-kernel.yaml` - catalog entry.
- INV-061 `microservices/docs/competitor-parity-matrix.md` - existing parity matrix; includes top-3 and broader set.
- INV-062 `microservices/docs/compliance.md` - compliance artifact; includes dependency inventory language.
- INV-063 `microservices/docs/contracts/asyncapi/docs-events.yaml` - AsyncAPI event contract.
- INV-064 `microservices/docs/contracts/openapi/docs.yaml` - OpenAPI contract.
- INV-065 `microservices/docs/contracts/proto/docs.proto` - proto contract; includes `go_package` generation option requiring Rust-strict classification.
- INV-066 `microservices/docs/cost-budget.md` - cost model; must be rechecked for tenant-class rather than tenant-vocabulary.
- INV-067 `microservices/docs/dashboards/collab-health.json` - dashboard artifact.
- INV-068 `microservices/docs/dashboards/editor-experience.json` - dashboard artifact.
- INV-069 `microservices/docs/dashboards/export-import-pipeline.json` - dashboard artifact.
- INV-070 `microservices/docs/decisions/ADR-DOC-001-collaborative-editing-yjs-crdt-vs-google-docs-OT.md` - local ADR comparing CRDT and Google Docs-style OT.
- INV-071 `microservices/docs/decisions/ADR-DOCS-0001-crdt-library-selection.md` - CRDT library ADR.
- INV-072 `microservices/docs/decisions/ADR-DOCS-0002-block-type-system.md` - block type system ADR.
- INV-073 `microservices/docs/decisions/ADR-DOCS-0003-export-pipeline-architecture.md` - export pipeline ADR.
- INV-074 `microservices/docs/decisions/ADR-DOCS-0004-acl-granularity-per-block.md` - per-block ACL ADR.
- INV-075 `microservices/docs/decisions/ADR-DOCS-0005-ai-writing-assist-bounds.md` - AI assist boundary ADR.
- INV-076 `microservices/docs/decisions/ADR-DOCS-0006-import-fidelity-policy.md` - import fidelity ADR with "fidelity tier" language.
- INV-077 `microservices/docs/decisions/README.md` - decisions index.
- INV-078 `microservices/docs/deprecation-notice.md` - deprecation notice.
- INV-079 `microservices/docs/dpia.md` - DPIA artifact.
- INV-080 `microservices/docs/failure-modes.md` - failure mode artifact.
- INV-081 `microservices/docs/faqs/docs-engineer-faq.md` - FAQ with many demo_trial/paid/paid/compliance_pack references.
- INV-082 `microservices/docs/iac/helm/Chart.yaml` - Helm chart, not OpenTofu context module.
- INV-083 `microservices/docs/iac/helm/templates/deployment.yaml` - Helm template.
- INV-084 `microservices/docs/iac/helm/templates/hpa.yaml` - Helm template.
- INV-085 `microservices/docs/iac/helm/templates/networkpolicy.yaml` - Helm template.
- INV-086 `microservices/docs/iac/helm/templates/pdb.yaml` - Helm template.
- INV-087 `microservices/docs/iac/helm/templates/prometheusrule.yaml` - Helm template.
- INV-088 `microservices/docs/iac/helm/templates/service.yaml` - Helm template.
- INV-089 `microservices/docs/iac/helm/templates/servicemonitor.yaml` - Helm template.
- INV-090 `microservices/docs/iac/helm/values.yaml` - Helm values.
- INV-091 `microservices/docs/iac/kustomize/base/kustomization.yaml` - Kustomize base.
- INV-092 `microservices/docs/iac/kustomize/base/namespace.yaml` - Kustomize base.
- INV-093 `microservices/docs/iac/kustomize/base/serviceaccount.yaml` - Kustomize base.
- INV-094 `microservices/docs/iac/kustomize/overlays/pack-eu/kustomization.yaml` - Kustomize overlay.
- INV-095 `microservices/docs/iac/kustomize/overlays/pack-kr/kustomization.yaml` - Kustomize overlay.
- INV-096 `microservices/docs/incident-response.md` - incident response artifact.
- INV-097 `microservices/docs/manifest.json` - machine-readable manifest with capability and tier fields.
- INV-098 `microservices/docs/migration-from-connect.md` - Connect migration artifact.
- INV-099 `microservices/docs/migration-playbooks/from-google-docs-and-notion.md` - migration playbook with tier provisioning commands.
- INV-100 `microservices/docs/multi-region.md` - multi-region artifact.
- INV-101 `microservices/docs/onboarding/docs-engineer-first-week.md` - onboarding artifact with tier commands.
- INV-102 `microservices/docs/policy/auditor-scope.cedar` - Cedar policy.
- INV-103 `microservices/docs/policy/ci-scope.cedar` - Cedar policy.
- INV-104 `microservices/docs/policy/data-residency.md` - data residency policy.
- INV-105 `microservices/docs/policy/editor-isolation.md` - editor isolation policy.
- INV-106 `microservices/docs/policy/public-read.cedar` - Cedar policy.
- INV-107 `microservices/docs/policy/tenant-scope.cedar` - Cedar policy.
- INV-108 `microservices/docs/reference-implementations/create-collab-and-export-rust-sdk.md` - Rust SDK reference implementation.
- INV-109 `microservices/docs/runbooks/attachment-restore.md` - runbook.
- INV-110 `microservices/docs/runbooks/collab-conflict-resolution.md` - runbook.
- INV-111 `microservices/docs/runbooks/doc-version-restore-corruption.md` - runbook.
- INV-112 `microservices/docs/runbooks/editor-session-storm-throttle.md` - runbook.
- INV-113 `microservices/docs/runbooks/embed-source-stale-detection.md` - runbook.
- INV-114 `microservices/docs/runbooks/export-pipeline-failure-pandoc-rollback.md` - runbook.
- INV-115 `microservices/docs/runbooks/share-acl-drift.md` - runbook.
- INV-116 `microservices/docs/scorecards/overrides.json` - scorecard overrides.
- INV-117 `microservices/docs/sdk-plan.md` - SDK plan.
- INV-118 `microservices/docs/slos/collab-cursor-sync-latency.openslo.yaml` - SLO.
- INV-119 `microservices/docs/slos/crdt-merge-no-silent-loss.openslo.yaml` - SLO.
- INV-120 `microservices/docs/slos/doc-list-latency.openslo.yaml` - SLO.
- INV-121 `microservices/docs/slos/doc-open-latency.openslo.yaml` - SLO.
- INV-122 `microservices/docs/slos/export-pdf-latency.openslo.yaml` - SLO.
- INV-123 `microservices/docs/slos/pandoc-export-pipeline-availability.openslo.yaml` - SLO.
- INV-124 `microservices/docs/slos/save-latency.openslo.yaml` - SLO.
- INV-125 `microservices/docs/slos/search-within-doc-latency.openslo.yaml` - SLO.
- INV-126 `microservices/docs/slos/share-acl-enforcement-correctness.openslo.yaml` - SLO.
- INV-127 `microservices/docs/threat-model.md` - threat model artifact.
- INV-128 `microservices/docs/tutorials/create-collab-edit-branch-merge-sign.md` - tutorial with tier command.

### §2.2 Read Coverage Notes

- The PRD was read for product purpose, functional requirements, performance targets, security controls, competitive benchmark, capacity envelope, and acceptance criteria.
- The architecture file was read for principal, Cedar, deployment-shape, observability, abuse-defense, and structural-note claims.
- All local ADR files under `microservices/docs/decisions/` were included in the inventory and sampled for decision alignment.
- Implementation plan files were included in the inventory and sampled for bounded-context coverage and documentation-tooling conflicts.
- Contracts under `contracts/openapi`, `contracts/asyncapi`, and `contracts/proto` were sampled for API surface coverage.
- SLOs under `slos/` were scanned for labels and target presence.
- `tenant-class-adoption/tenant-class-adoption-record.md` was read as a retirement-candidate artifact, not as live doctrine.
- `capacity-model.md`, `failure-modes.md`, `incident-response.md`, `cost-budget.md`, `dpia.md`, and `compliance.md` were included as required ownership surfaces.
- `benchmarks/`, `faqs/`, `onboarding/`, `migration-playbooks/`, `reference-implementations/`, `tutorials/`, and `runbooks/` were included as product-adjacent evidence.
- `iac/` was read for substrate coverage; only Helm and Kustomize surfaces were found.
- No `src/` or `tests/` files exist under the microservice path to sample.
- No root `README.md` exists to read.
- Chat history was searched for docs-related context in `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.
- Chat history line 11572 records a prior "µservice doc-suite gapfill wave 5" that authored docs `tenant-class-adoption`, onboarding, FAQ, tutorial, benchmark, migration, and reference implementation surfaces.
- Chat history line 16311 embeds the rolling audit queue row mapping `docs` to Google Docs, Microsoft Word Online, and Notion Docs.
- Chat history line 16439 lists `docs` in an in-flight Phase 3 first cohort of microservice audits.
- Chat history context confirms several docs artifacts were generated before the no-tenant-class-drifts correction, explaining why tier retirement candidates are concentrated in onboarding, FAQ, benchmark, migration, and tenant-class-adoption-record surfaces.

## §3 Nine-Dimension Audit

### §3.1 Dimension 1 - Product Purpose Coherence

- Status: PARTIAL PASS.
- Evidence: `microservices/docs/PRD.md` lines 20-24 clearly defines product purpose and counterpart analogs.
- Evidence: `microservices/docs/PRD.md` lines 42-59 gives a broad but coherent functional requirement set.
- Evidence: `microservices/docs/PRD.md` lines 110-117 maps functionality to bounded contexts.
- Evidence: `microservices/docs/competitor-parity-matrix.md` lines 48-120 maps core model, collaboration, protocols, accessibility, privacy, and AI capabilities.
- Strength: the service is not an abstract documentation site; it is a collaborative editor substrate.
- Strength: the bounded contexts match the actual product surface: document storage, CRDT, block types, comments, versions, sharing, export/import, and embeds.
- Strength: legal hold, per-block ACL, AI assist, audit chain, WCAG, and export formats are represented in the PRD.
- Weakness: `ARCHITECTURE.md` line 3 says the file was created by the Wave-3-C anchor-sweep and still instructs expansion of stub sections during content-pass review.
- Weakness: architecture structural notes such as `ARCHITECTURE.md` lines 68-69 and 568-631 repeatedly claim evidence surfaces exist without distinguishing OpenTofu context gaps from Helm/Kustomize runtime manifests.
- Weakness: some existing product docs mix collaborative-doc product content with docs-site tooling plans such as mdBook, SvelteKit, Backstage, Redoc, and Mermaid build surfaces.
- Weakness: `IP-DOCS-002-sveltekit-marketing-site.md` and `IP-DOCS-005-backstage-techdocs-renderer.md` are documentation-platform plans, not the collaborative editor product; they need explicit ownership separation if retained.
- Risk: a reader could confuse the `docs` microservice with a documentation publishing pipeline unless the root README and product overview clarify the primary product.
- Required correction: add or update a root README that distinguishes the `docs` collaborative editor from doc-site build tooling.
- Required correction: mark docs-site tooling as secondary internal documentation surfaces or split them if they are not owned by the tenant-facing collaborative docs service.
- Required correction: update architecture structural notes to state exact IaC and deployment-context limitations instead of broad "present" language.
- Counterpart fit: Google Docs covers collaborative editor expectations; Word Online covers OOXML and review fidelity; Notion covers block-based authoring and embeds.
- Product bar conclusion: the product story is coherent, but root-level onboarding and ownership boundaries are not yet crisp enough for a cold reader.

### §3.2 Dimension 2 - Artifact Completeness

- Status: PARTIAL PASS.
- Present: PRD, architecture, local ADRs, implementation plans, contracts, SLOs, dashboards, runbooks, compliance, DPIA, threat model, capacity model, failure modes, incident response, policies, migration playbook, onboarding, FAQ, tutorial, benchmark, and Rust SDK reference implementation.
- Present: OpenAPI contract in `microservices/docs/contracts/openapi/docs.yaml`.
- Present: AsyncAPI contract in `microservices/docs/contracts/asyncapi/docs-events.yaml`.
- Present: proto contract in `microservices/docs/contracts/proto/docs.proto`.
- Present: SLO files under `microservices/docs/slos/`.
- Present: runbooks under `microservices/docs/runbooks/`.
- Present: dashboard JSON under `microservices/docs/dashboards/`.
- Present: 19 catalog entries under `microservices/docs/catalog/`.
- Present: policy fragments and policy docs under `microservices/docs/policy/`.
- Missing: root `README.md` was not found.
- Missing: `supported-oses.json` was not found.
- Missing: `cross-microservice-handoffs.md` was not found.
- Missing: top-level `src/` was not found.
- Missing: top-level `tests/` was not found.
- Missing: canonical OpenTofu context directories were not found.
- Missing: `iac/oci-guest/always-free/` was not found.
- Missing: tenant-class behavior or tenant-class contract artifact was not found.
- Finding impact: artifact completeness is broad at the docs level but weak on canonical execution surfaces.
- Evidence: `microservices/docs/ARCHITECTURE.md` lines 22-29 lists evidence surfaces, but the inventory shows no root README, supported OS manifest, handoff doc, source, tests, or canonical OpenTofu contexts.
- Evidence: `microservices/docs/manifest.json` lines 80-89 lists contracts, and that contract inventory is supported by actual files.
- Evidence: `microservices/docs/manifest.json` lines 91-109 lists capability records, but the tier field needs terminology review.
- Required correction: create machine-readable missing surfaces in a later implementation wave, not in this audit.

### §3.3 Dimension 3 - Contract and Boundary Coherence

- Status: PARTIAL PASS.
- Evidence: `microservices/docs/contracts/openapi/docs.yaml` exists and begins with OpenAPI 3.2.0.
- Evidence: `microservices/docs/contracts/asyncapi/docs-events.yaml` exists and begins with AsyncAPI 3.1.0.
- Evidence: `microservices/docs/contracts/proto/docs.proto` lines 1-8 define proto syntax, package, imports, and a generated Go package option.
- Evidence: `microservices/docs/contracts/proto/docs.proto` lines 225-233 defines the `DocumentStore` service.
- Evidence: `microservices/docs/manifest.json` lines 80-89 points to OpenAPI, AsyncAPI, and proto contracts.
- Strength: three contract surfaces exist, which is strong for a collaborative document substrate.
- Strength: the proto service surface names document creation, retrieval, metadata update, archive, list, legal hold open, and legal hold release.
- Strength: PRD lines 213-237 maps produced and consumed workflow events.
- Weakness: tenant-class behavior is absent from contracts and surrounding docs; the current doctrine says the gateway/IAM should enforce tenant class transparently, but per-microservice docs must declare meters and cap behavior.
- Weakness: the proto `option go_package` at `contracts/proto/docs.proto` line 8 is not application code, but it is non-Rust generated-client metadata and should be classified as generated SDK compatibility, not backend implementation permission.
- Weakness: cross-microservice handoffs are described in PRD prose but lack the required dedicated `cross-microservice-handoffs.md`.
- Weakness: deployment contexts and OS support are not encoded in contract metadata.
- Boundary risk: contracts can look complete while still not proving deployment portability or tenant-class behavior.
- Required correction: add a tenant-class and meter declaration artifact that states docs storage, editor session, export, import, attachment, and AI-assist meters emitted to billing.
- Required correction: add a contract metadata note classifying proto language options as generated-client compatibility and confirming Rust backend ownership.
- Required correction: add a dedicated handoff document mapping dependencies on workflow-studio, audit-chain, tenancy, drive, mail, messenger, sheets, slides, identity, observability, network, intelligence, ontology, detection, cell, cloud-iac, and calendar from `manifest.json` lines 425-443.

### §3.4 Dimension 4 - Canonical-Direction Alignment

- Status: FAIL for current canonical direction.
- Required canonical direction: six deployment contexts, OpenTofu IaC, per-microservice OS support manifest, Rust-strict backend, OCI Always Free profile, no demo_trial/paid/paid/compliance_pack tiers, and tenant-class replacement model.
- Evidence for six contexts: `specs/master-plan-sequencing.json` lines 704-743 names `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, and `oyatie-as-cloud-provider`.
- Evidence for OpenTofu: `specs/master-plan-sequencing.json` lines 747-775 defines OpenTofu as the only engine and forbids Terraform, Pulumi, CloudFormation, ARM, `null_resource`, `local-exec`, SSH provisioners, hand-edited state, and unsigned modules.
- Evidence for OS support: `specs/master-plan-sequencing.json` lines 777-815 requires per-microservice OS manifests.
- Evidence for language policy: `specs/master-plan-sequencing.json` lines 817-855 defines Rust backend and the frontend allowlist.
- Evidence for OCI profile: `specs/master-plan-sequencing.json` lines 857-867 defines the Always Free profile and `iac/oci-guest/always-free/`.
- Evidence for current local gap: `find microservices/docs/iac -maxdepth 3 -type f | sort` shows Helm and Kustomize files only.
- Evidence for local statement: `microservices/docs/ARCHITECTURE.md` lines 571-578 says runtime is Kubernetes pods and IaC manifests in scope are Helm templates.
- Finding: Helm/Kustomize runtime manifests can remain useful, but they are not the canonical OpenTofu per-context IaC substrate.
- Finding: no `microservices/docs/iac/oyatie-public-cloud/` exists.
- Finding: no `microservices/docs/iac/guest-on-aws/` exists.
- Finding: no `microservices/docs/iac/oci-guest/` exists.
- Finding: no `microservices/docs/iac/oci-guest/always-free/` exists.
- Finding: no `microservices/docs/iac/on-prem/` exists.
- Finding: no `microservices/docs/iac/colo/` exists.
- Finding: no `microservices/docs/iac/oyatie-iaas/` exists.
- Finding: no `microservices/docs/supported-oses.json` exists.
- Finding: no `tenant_class`, `demo_trial`, `paid`, or `revenue_share` string appears under the microservice path.
- Finding: no forbidden source extension files were found under the docs microservice path in the scan for `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx`, `.cs`, `.kt`, or `.swift`.
- Finding: the proto `go_package` option is metadata and not a backend source file; it still needs clear classification because the user explicitly disallowed non-Rust backend implementations.
- Canonical-direction conclusion: the product surface is rich, but the service is not yet canonical-direction-complete.

#### §3.4.T Tier Retirement Candidates

- Rule: every demo_trial/paid/paid/compliance_pack reference below is a Wave 15J retirement candidate, default severity P2 unless attached to a P1 canonical claim.
- TR-001 `microservices/docs/onboarding/docs-engineer-first-week.md:23` uses `TENANT_CLASS=demo_trial` in a dev tenant command.
- TR-002 `microservices/docs/onboarding/docs-engineer-first-week.md:59` uses `availability = "paid"`.
- TR-003 `microservices/docs/migration-playbooks/from-google-docs-and-notion.md:45` uses `--tenant-class paid`.
- TR-004 `microservices/docs/migration-playbooks/from-google-docs-and-notion.md:165` claims `10k concurrent editors per doc at compliance_pack`.
- TR-005 `microservices/docs/tutorials/create-collab-edit-branch-merge-sign.md:10` uses `TENANT_CLASS=paid`.
- TR-006 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:11` uses `docs (paid)` for latency.
- TR-007 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:22` uses `docs (paid)` for cold load.
- TR-008 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:33` uses `docs (paid)` for editor cap.
- TR-009 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:34` uses `docs (compliance_pack)` for editor cap.
- TR-010 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:45` uses `docs (paid)` for block richness.
- TR-011 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:56` uses `docs (paid)` for branching and review workflow.
- TR-012 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:67` uses `docs (compliance_pack)` for compliance and e-sign.
- TR-013 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:78` uses `docs (paid)` for TCO.
- TR-014 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:85` uses `docs (paid)` in price narrative.
- TR-015 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:91` uses `compliance_pack` in editor ceiling narrative.
- TR-016 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:94` uses `compliance_pack pack`.
- TR-017 `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:95` uses `compliance_pack`.
- TR-018 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:11` defines `Tier demo_trial`.
- TR-019 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:29` defines `Tier paid`.
- TR-020 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:49` defines `Tier paid`.
- TR-021 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:69` defines `Tier compliance_pack`.
- TR-022 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:77` references `all paid`.
- TR-023 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:85` references `All paid`.
- TR-024 `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:95` references `demo_trial/paid`, `paid`, and `compliance_pack` in AI co-author policy.
- TR-025 `microservices/docs/faqs/docs-engineer-faq.md:65` references `demo_trial/paid`, `paid`, and `compliance_pack`.
- TR-026 `microservices/docs/faqs/docs-engineer-faq.md:78` references `compliance_pack`.
- TR-027 `microservices/docs/faqs/docs-engineer-faq.md:109` references `demo_trial`.
- TR-028 `microservices/docs/faqs/docs-engineer-faq.md:110` references `paid`.
- TR-029 `microservices/docs/faqs/docs-engineer-faq.md:111` references `paid`.
- TR-030 `microservices/docs/faqs/docs-engineer-faq.md:112` references `compliance_pack`.
- TR-031 `microservices/docs/faqs/docs-engineer-faq.md:120` references `demo_trial`, `paid`, `paid`, and `compliance_pack` latency budgets.
- TR-032 `microservices/docs/faqs/docs-engineer-faq.md:128` references `paid` and `demo_trial + paid`.
- TR-033 `microservices/docs/faqs/docs-engineer-faq.md:142` references `paid` and `compliance_pack` regional budgets.
- TR-034 `microservices/docs/faqs/docs-engineer-faq.md:148` references `demo_trial`, `paid`, `paid`, and `compliance_pack` doc-open budgets.
- Tier-adjacent review candidate: `microservices/docs/PRD.md:8` uses `tier: tenant-facing`; this is not demo_trial/paid/paid/compliance_pack but should be normalized to `audience_class` or `service_class`.
- Tier-adjacent review candidate: `microservices/docs/PRD.md:285` uses "best-effort fidelity" for OOXML import fidelity; this should become `fidelity_profile`.
- Tier-adjacent review candidate: `microservices/docs/ARCHITECTURE.md:23` uses `tier product`; this should become service classification.
- Tier-adjacent review candidate: `microservices/docs/ARCHITECTURE.md:576` uses `Tier 0/1 paths`; this should become runtime isolation class or capability autonomy class.
- Tier-adjacent review candidate: `microservices/docs/ARCHITECTURE.md:639` uses `cell_tier`; this should become `cell_class` or `runtime_profile`.
- Tier-adjacent review candidate: `microservices/docs/ARCHITECTURE.md:700` uses `tenant_class`; this should become `tenant_class`.
- Tier-adjacent review candidate: `microservices/docs/manifest.json:370` uses `automation_risk_classes`.
- Tier-adjacent review candidate: `microservices/docs/manifest.json:400` uses `tier_classification`.
- Tier-adjacent review candidate: `microservices/docs/manifest.json:446` uses `criticality_tier`.
- Tier-adjacent review candidate: SLO files under `microservices/docs/slos/*.openslo.yaml:11` use `tier: hero-product` or `tier: external-facing`.

#### §3.4.C Tenant-Class Adoption Gaps

- Search result: `rg -n "tenant_class|demo_trial|revenue_share|\bpaid\b" microservices/docs` returned no matches.
- Gap: the service does not express `tenant_class` anywhere under its microservice path.
- Gap: the service does not distinguish `demo_trial` usage caps from `paid` contractual scaling.
- Gap: the service does not identify `revenue_share` billing treatment for marketplace sellers, B2C operators, embedded SaaS resellers, or affiliate partners.
- Gap: the service does not state which docs-specific meters feed `cloud-billing`.
- Required docs meters: stored documents, active editor sessions, CRDT operations, export jobs, import jobs, attachment bytes, AI-assist invocations, and share-link events.
- Required demo_trial behavior: OCI Always Free profile where applicable, hard usage caps, best-effort SLO, no compliance packs, and no BYOK.
- Required paid behavior: per-seat plus usage billing, any deployment context, contractual SLO, compliance packs allowed, and BYOK allowed.
- Required revenue_share behavior: at-cost or zero-margin substrate, gross-revenue share billing, and no lower quality bar.
- Important boundary: tenant classes are not feature-quality tiers; all classes inherit the same industry-leader product quality target.
- Current result: tenant-class adoption gap is YES.

### §3.5 Dimension 5 - Industry-Counterpart Parity

- Status: PARTIAL PASS.
- Required counterpart set: Google Docs, Microsoft Word Online, Notion Docs.
- Evidence: `microservices/docs/PRD.md` lines 262-268 names those three among a larger competitor set.
- Evidence: `microservices/docs/competitor-parity-matrix.md` lines 28-30 names Google Docs, Microsoft Word Web, and Notion.
- Evidence: `microservices/docs/benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md` line 1 names Google Docs, Microsoft Word Online, and Notion in the benchmark title.
- Strength: parity coverage already includes the top-3 counterparts.
- Strength: the product explicitly targets Google Docs style collaboration, Word Web/OOXML fidelity, and Notion-style blocks.
- Strength: `PRD.md` lines 281-288 identifies parity gaps for CRDT zero-silent-loss, per-block ACL, embeds, dual-context isolation, OOXML fidelity, PDF/A export, AI assist, and WCAG.
- Weakness: the existing benchmark file uses retired paid/compliance_pack rows, so it cannot remain the current performance narrative.
- Weakness: `competitor-parity-matrix.md` line 128 uses "best-effort fidelity" even outside demo_trial/paid/paid/compliance_pack.
- Weakness: the existing matrix uses checkmarks but not enough public citation discipline for each competitor capability.
- Weakness: Notion's database/page surface and Microsoft Word Online's track-change/OOXML expectations need stronger union coverage in a current matrix.
- Weakness: Google Docs official collaborator ceiling and API quotas should replace unsupported claims where public sources exist.
- Required correction: use the sibling feature-parity deliverable from this batch as the current top-3 union matrix.
- Required correction: use the sibling performance benchmark deliverable from this batch as the non-tiered benchmark target set.
- Parity conclusion: product ambition is on target, but existing parity artifacts need tier-retirement and source-strengthening.

### §3.6 Dimension 6 - Operational Readiness

- Status: PARTIAL PASS.
- Evidence: SLO files exist for collab cursor sync, CRDT merge, doc list, doc open, export PDF, Pandoc export availability, save latency, search, and share ACL enforcement.
- Evidence: runbooks exist for attachment restore, conflict resolution, version restore, editor session storm throttling, stale embed detection, export pipeline failure rollback, and share ACL drift.
- Evidence: `microservices/docs/PRD.md` lines 96-98 defines availability, RTO, RPO, and stale-embed degradation.
- Evidence: `microservices/docs/PRD.md` lines 313-319 defines per-cell capacity envelope.
- Evidence: `microservices/docs/PRD.md` lines 321-328 defines scale-out policy.
- Strength: operational concern coverage is broader than many microservices because editor UX, CRDT, export, and share correctness all have named SLO/runbook surfaces.
- Weakness: SLO labels still use `tier` vocabulary at line 11 in each OpenSLO file.
- Weakness: the SLO set is not expressed per deployment context.
- Weakness: the SLO set is not expressed per tenant class.
- Weakness: no OCI Always Free profile cap overlays exist for demo_trial infrastructure.
- Weakness: no OS support test matrix exists.
- Weakness: no top-level tests directory exists for validating runbook claims.
- Weakness: no source files exist locally to tie SLOs to instrumentation code.
- Operational conclusion: good paper coverage, incomplete canonical deployment and verification binding.

### §3.7 Dimension 7 - Security, Privacy, and Compliance Coherence

- Status: PARTIAL PASS.
- Evidence: `microservices/docs/PRD.md` lines 81-85 defines encryption, sandboxed export workers, attachment scanning, mTLS embeds, and tenant-DEK-wrapped AI prompts.
- Evidence: `microservices/docs/PRD.md` lines 89-92 defines audit-chain records, legal hold, jurisdiction retention, and WCAG evidence.
- Evidence: `microservices/docs/policy/*.cedar` and policy markdown files exist.
- Evidence: `microservices/docs/dpia.md`, `microservices/docs/threat-model.md`, `microservices/docs/compliance.md`, and `microservices/docs/failure-modes.md` exist.
- Evidence: `microservices/docs/ARCHITECTURE.md` lines 40-46 names cross-service dependencies including tenancy, identity, policy-engine, observability, audit-chain, cloud-secrets, cell, and cloud-iac.
- Strength: per-block ACL, legal hold, export scanning, audit signatures, and data residency are first-class.
- Strength: compliance posture includes policy fragments and regulatory journey overlays.
- Weakness: compliance packs are not re-expressed against the current `demo_trial`, `paid`, and `revenue_share` tenant classes.
- Weakness: the current tenant-class prompt says `demo_trial` cannot use compliance packs or BYOK, while `paid` can and `revenue_share` runs at cost/zero-margin; that is absent locally.
- Weakness: tier-language in FAQ and capability tenant_class adoption record creates a compliance-claim hazard.
- Weakness: no per-context OpenTofu module proves that compliance controls are enforceable in all six deployment contexts.
- Security conclusion: the service has strong security intent but lacks current tenant-class and context-enforcement codification.

### §3.8 Dimension 8 - Implementation Feasibility and Build Surface

- Status: PARTIAL PASS.
- Evidence: `microservices/docs/manifest.json` lines 6-74 lists bounded contexts and crate names.
- Evidence: `microservices/docs/catalog/*.yaml` covers kernel, adapter, app, and export/import components.
- Evidence: `microservices/docs/reference-implementations/create-collab-and-export-rust-sdk.md` is a Rust SDK reference implementation.
- Evidence: forbidden extension scan under `microservices/docs` found no `.py`, `.js`, `.ts`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx`, `.cs`, `.kt`, or `.swift` files.
- Strength: the file-extension scan does not reveal prohibited implementation source files inside this microservice path.
- Strength: backend crate naming in manifest and catalog is Rust-oriented.
- Weakness: no actual Rust `src/` implementation files exist under the microservice path.
- Weakness: no `tests/` files exist under the microservice path.
- Weakness: docs-site implementation plans mention SvelteKit and Backstage, which require explicit retirement or rehosting outside the Rust/Leptos allowed web frontend posture.
- Weakness: proto includes a `go_package` option at `contracts/proto/docs.proto` line 8; if generated clients are allowed, this needs explicit generated-SDK classification.
- Weakness: the prior doc-suite chat line 11572 mentions developer-sdk artifacts with Rust, TypeScript, and Python SDKs; current docs microservice reference implementation is Rust, but the surrounding project history increases the need for language-boundary clarity.
- Feasibility conclusion: implementation architecture is plausible, but the local path is documentation-first and not implementation-verifiable yet.

### §3.9 Dimension 9 - Ownership Coherence and Handoff Readiness

- Status: PARTIAL PASS.
- Evidence: `microservices/docs/manifest.json` line 5 owns the service to `axis-docs`.
- Evidence: `microservices/docs/PRD.md` line 14 owns the PRD to `axis-docs`.
- Evidence: `microservices/docs/competitor-parity-matrix.md` lines 7-14 names date, owners, deciders, related artifacts, and review cadence.
- Evidence: `microservices/docs/manifest.json` lines 425-443 lists many dependency microservices.
- Strength: service ownership is consistently `axis-docs` in core metadata.
- Strength: dependencies are explicit in manifest and architecture.
- Weakness: no `cross-microservice-handoffs.md` exists to bind dependency responsibilities.
- Weakness: no root README exists for cold-start ownership context.
- Weakness: no deployment-context manifest exists to prevent overclaiming.
- Weakness: no tenant-class behavior artifact exists to prevent billing/support/compliance drift.
- Weakness: existing generated artifacts and chat history show broad prior automation; artifact generation provenance should be distinguished from canonical ownership evidence.
- Handoff conclusion: ownership is named, but not coherent enough for another team to implement, deploy, and operate across all six contexts without additional artifacts.

## §4 Findings Table

| ID | Severity | Finding | Evidence | Required resolution |
|---|---:|---|---|---|
| DOCS-001 | P1 | Missing canonical OpenTofu six-context IaC modules. | `specs/master-plan-sequencing.json` lines 704-775 require six contexts and OpenTofu; local `iac/` only has Helm/Kustomize files. | Add or plan `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, and `iac/oyatie-iaas/` OpenTofu modules. |
| DOCS-002 | P1 | OCI Always Free profile module is absent. | `specs/master-plan-sequencing.json` lines 857-867 require `iac/oci-guest/always-free/`; local path has no such directory. | Add docs-specific OCI Always Free profile plan/module for demo_trial infrastructure. |
| DOCS-003 | P2 | No per-microservice OS support manifest exists. | `specs/master-plan-sequencing.json` lines 777-815 require per-microservice manifests; `supported-oses.json` not found. | Add `supported-oses.json` with Linux, BSD, illumos, Darwin, Windows, and mobile/front-end scope decisions. |
| DOCS-004 | P2 | No tenant-class semantics are expressed. | `rg` for `tenant_class`, `demo_trial`, `paid`, and `revenue_share` returned no matches. | Add tenant-class behavior and meter declaration for docs. |
| DOCS-005 | P2 | Explicit demo_trial/paid/paid/compliance_pack references remain. | TR-001 through TR-034 in §3.4.T. | Retire or rewrite during Wave 15J; do not propagate into new docs. |
| DOCS-006 | P2 | `tenant-class-adoption/tenant-class-adoption-record.md` remains as live-looking documentation. | `microservices/docs/tenant-class-adoption/tenant-class-adoption-record.md:11`, `:29`, `:49`, `:69`. | Delete or supersede in Wave 15J per no-tenant-class-drift directive. |
| DOCS-007 | P2 | Existing benchmark file is tier-segmented. | `microservices/docs/benchmarks/...md:11`, `:22`, `:33`, `:34`, `:67`, `:91`. | Replace with single target set plus deployment and tenant-class overlays. |
| DOCS-008 | P2 | Onboarding and tutorial commands provision by tier. | `onboarding/docs-engineer-first-week.md:23`; `tutorials/create-collab-edit-branch-merge-sign.md:10`. | Replace with tenant-class and usage-profile language. |
| DOCS-009 | P2 | Migration playbook provisions `--tenant-class paid`. | `migration-playbooks/from-google-docs-and-notion.md:45`. | Replace with tenant class plus billing/substrate choices. |
| DOCS-010 | P2 | FAQ uses tier-specific feature and performance gates. | `faqs/docs-engineer-faq.md:65`, `:109-112`, `:120`, `:148`. | Rewrite FAQ around uniform quality and tenant-class caps. |
| DOCS-011 | P2 | SLO labels use `tier`. | `slos/*.openslo.yaml:11` via `rg -n "tier:"`. | Rename labels to `service_class`, `slo_class`, or `audience_class`. |
| DOCS-012 | P2 | Manifest uses tier fields. | `manifest.json:370`, `:400`, `:446`. | Rename to capability/autonomy/service/criticality classes as appropriate. |
| DOCS-013 | P2 | Root README missing. | `find` found no `microservices/docs/README.md`. | Add root README distinguishing collaborative docs from documentation tooling. |
| DOCS-014 | P2 | Cross-microservice handoff file missing. | `find` found no `cross-microservice-handoffs.md`; manifest dependencies at lines 425-443 are broad. | Add handoff artifact with dependency owner, event, contract, and failure responsibilities. |
| DOCS-015 | P2 | No local source or tests exist for implementation verification. | `find microservices/docs -maxdepth 2 -type d` found no `src/` or `tests/`. | Add implementation/test surface or explicitly state docs-only status until code lands. |
| DOCS-016 | P2 | Proto generated-client metadata needs Rust-strict classification. | `contracts/proto/docs.proto:8` has `option go_package`. | Mark as generator compatibility metadata, not backend implementation permission. |
| DOCS-017 | P2 | Architecture overstates IaC evidence as present. | `ARCHITECTURE.md:568-631` says IaC evidence surfaces are present, but only Helm/Kustomize exist. | Amend architecture to distinguish runtime manifests from canonical OpenTofu context modules. |
| DOCS-018 | P2 | Prior chat confirms docs artifacts came from a pre-retirement doc-suite gapfill. | Chat history line 11572 lists docs tenant-class-adoption-record, onboarding, FAQ, tutorial, benchmark, migration, and reference implementation. | Treat those artifacts as review inputs, not final current doctrine. |
| DOCS-019 | P3 | Existing parity matrix includes extra competitors without current top-3 methodology separation. | `competitor-parity-matrix.md` lines 24-42 includes many competitors beyond top-3. | Keep extended set as secondary, but separate top-3 union coverage. |
| DOCS-020 | P3 | PRD/competitor matrix include `M03-onward1` typo-like milestone text. | `competitor-parity-matrix.md` lines 61, 89, 114-116, 138-140. | Normalize milestone spelling in a cleanup pass. |
| DOCS-021 | P3 | Architecture still includes anchor-sweep provenance warning. | `ARCHITECTURE.md:3`. | Complete content-pass review and remove obsolete scaffold warning when true. |

### §4.1 Severity Counts

- P0 findings: 0.
- P1 findings: 2.
- P2 findings: 16.
- P3 findings: 3.
- Total findings: 21.
- The two P1 findings are deployment-context blockers, not product-purpose blockers.
- The P2 group is dominated by canonical-direction drift and retired-vocabulary cleanup.
- The P3 group is cleanup quality and naming coherence.

### §4.2 Constraint Evaluation Summary

- Multi-context: FAIL, because no six per-context OpenTofu modules exist.
- OpenTofu IaC: FAIL, because current `iac/` evidence is Helm/Kustomize only.
- OS support: FAIL, because no `supported-oses.json` exists.
- Rust-strict: PASS for local file-extension scan; REVIEW for proto generated-client option and docs-site tooling plans.
- OCI Always Free: FAIL, because `iac/oci-guest/always-free/` is absent.
- Tier retirement: FAIL until TR-001 through TR-034 and adjacent tier fields are scrubbed or superseded.
- Tenant class: FAIL until `demo_trial`, `paid`, and `revenue_share` semantics are expressed.
- Industry parity: PARTIAL PASS, because top-3 counterpart surfaces are named but existing benchmark/matrix artifacts need current methodology and no-tenant-class-drift rewrite.
- Operational readiness: PARTIAL PASS, because SLO/runbook coverage exists but lacks context and tenant-class overlays.

## §5 Open Questions

- OQ-001 Which team owns the future `tenant-class-behavior` artifact for docs: `axis-docs`, `cloud-billing`, or a joint ownership model?
- OQ-002 Should the docs-site tooling plans under `IP-DOCS-*` remain inside the tenant-facing `docs` microservice or move to an internal documentation platform scope?
- OQ-003 Should generated proto metadata retain `go_package` for compatibility, or should the project use a generator-neutral option set plus per-SDK generation outside this microservice?
- OQ-004 What is the exact demo_trial docs cap set: maximum documents, maximum storage, maximum concurrent editors, maximum export/import jobs, and maximum AI assist invocations?
- OQ-005 How should revenue_share tenants be metered for collaborative docs when docs activity may not directly produce gross revenue?
- OQ-006 Does docs need all six deployable contexts at first ship, or should missing contexts be explicitly scoped as blocked until cloud-iac context modules exist?
- OQ-007 Are Helm and Kustomize runtime manifests intended to be rendered by OpenTofu modules, or are they legacy artifacts to retire?
- OQ-008 Should SLO labels use `service_class`, `audience_class`, `slo_class`, or another canonical replacement for the overloaded `tier` key?
- OQ-009 Does the product require offline editing parity with Google Docs and Word, and if so where should that be encoded?
- OQ-010 Does Notion database integration belong in docs proper or in a separate database/table microservice embedding flow?
- OQ-011 Should Microsoft Word Online parity require change-tracking semantics beyond suggestions, especially reviewer identity preservation in OOXML?
- OQ-012 What is the canonical line between per-block ACL and cross-document embedding policy when embedded content crosses service boundaries?
- OQ-013 Should `docs` expose public publishing URLs like Notion, or should public-read remain a policy-controlled share mode only?
- OQ-014 Which OSes require native editor packaging and which are web-only support targets under the OS support matrix?
- OQ-015 Should PDF/A, eIDAS, KR PKI, and FDA 21 CFR Part 11 be compliance-pack capabilities only for paid/revenue_share tenant classes?
- OQ-016 How should demo_trial no-compliance-pack behavior be reflected in existing compliance and legal-hold docs without lowering base product quality?
- OQ-017 Does `docs` need a dedicated import-fidelity scoring contract for Google Docs, Word, and Notion migrations?
- OQ-018 Should editor-session storm throttling be tenant-class-aware, deployment-context-aware, or both?
- OQ-019 Should AI assist capabilities T0/T1/T2 be renamed away from `tier` fields while preserving autonomy ceilings?
- OQ-020 Is `M03-onward1` an intentional milestone label or a typo that should become `M03-onward`?

## Appendix A - Evidence Commands

- Inventory command: `find microservices/docs -type f | sort`.
- Inventory count command: `find microservices/docs -type f | sort | wc -l`.
- Inventory line count command: `find microservices/docs -type f -print0 | xargs -0 wc -l | tail -1`.
- Vocabulary scan command: `rg -n "demo_trial|paid|paid|compliance_pack" microservices/docs`.
- Tenant-class scan command: `rg -n "tenant_class|demo_trial|revenue_share|\bpaid\b" microservices/docs`.
- Forbidden-language scan command: `find microservices/docs -type f \( -name '*.py' -o -name '*.js' -o -name '*.ts' -o -name '*.rb' -o -name '*.go' -o -name '*.java' -o -name '*.scala' -o -name '*.groovy' -o -name '*.php' -o -name '*.fs' -o -name '*.fsx' -o -name '*.cs' -o -name '*.kt' -o -name '*.swift' \) -print | sort`.
- IaC file command: `find microservices/docs/iac -maxdepth 3 -type f | sort`.
- Root special-file command: `find microservices/docs -maxdepth 2 -type f \( -name 'README.md' -o -name 'supported-oses.json' -o -name 'cross-microservice-handoffs.md' \) -print | sort`.
- Chat search target: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl`.

## Appendix B - Substantive Read Ledger

- READ-001 PRD purpose: `PRD.md:20-24` confirms docs is a Google Docs / Microsoft Word Web / Notion / Coda parallel.
- READ-002 PRD requirements: `PRD.md:42-59` confirms 18 functional requirements.
- READ-003 PRD performance: `PRD.md:67-77` confirms latency and export/import targets.
- READ-004 PRD security: `PRD.md:81-85` confirms encryption, sandboxing, scanning, mTLS, and AI prompt handling.
- READ-005 PRD audit/compliance: `PRD.md:89-92` confirms audit-chain and legal-hold behavior.
- READ-006 PRD availability: `PRD.md:96-98` confirms availability, RTO, RPO, and stale embed fallback.
- READ-007 PRD contexts: `PRD.md:110-117` confirms eight bounded contexts.
- READ-008 PRD competitive set: `PRD.md:262-268` confirms top counterparts and extra references.
- READ-009 PRD gaps: `PRD.md:281-288` confirms known parity gaps and one tier-adjacent fidelity phrase.
- READ-010 PRD capacity: `PRD.md:313-319` confirms per-cell envelope.
- READ-011 Architecture provenance: `ARCHITECTURE.md:3` confirms anchor-sweep origin.
- READ-012 Architecture evidence claim: `ARCHITECTURE.md:68-69` claims evidence surfaces exist.
- READ-013 Architecture deployment: `ARCHITECTURE.md:571-578` confirms Kubernetes/Helm framing.
- READ-014 Architecture observability: `ARCHITECTURE.md:633-644` confirms metrics and one `cell_tier` label.
- READ-015 Architecture abuse defense: `ARCHITECTURE.md:695-707` confirms bot-score and tenant-class policy language.
- READ-016 Manifest contexts: `manifest.json:6-74` confirms bounded context inventory.
- READ-017 Manifest contracts: `manifest.json:80-89` confirms OpenAPI, AsyncAPI, and proto references.
- READ-018 Manifest capabilities: `manifest.json:91-109` confirms T0/T1/T2 capability records with `tier` fields.
- READ-019 Manifest tier fields: `manifest.json:370`, `:400`, and `:446` confirm tier-adjacent machine fields.
- READ-020 Manifest dependencies: `manifest.json:425-443` confirms broad dependency graph.
- READ-021 Proto package: `contracts/proto/docs.proto:1-8` confirms proto syntax and `go_package`.
- READ-022 Proto service: `contracts/proto/docs.proto:225-233` confirms document service methods.
- READ-023 Existing benchmark title: `benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:1` confirms counterpart mix.
- READ-024 Existing benchmark latency: `benchmarks/...md:11-14` confirms old tiered docs row and counterpart latency estimates.
- READ-025 Existing benchmark editor cap: `benchmarks/...md:33-37` confirms old paid/compliance_pack rows and counterpart cap estimates.
- READ-026 Existing benchmark compliance: `benchmarks/...md:67-70` confirms old compliance_pack compliance row.
- READ-027 Existing benchmark narrative: `benchmarks/...md:90-95` confirms old compliance_pack differentiator claims.
- READ-028 Existing parity matrix top-3: `competitor-parity-matrix.md:28-30` confirms Google, Microsoft, and Notion.
- READ-029 Existing parity matrix extra set: `competitor-parity-matrix.md:31-42` confirms broader context set.
- READ-030 Existing parity matrix core model: `competitor-parity-matrix.md:48-61` confirms feature comparison.
- READ-031 Existing parity matrix collaboration: `competitor-parity-matrix.md:65-74` confirms collaboration comparison.
- READ-032 Existing parity matrix protocols: `competitor-parity-matrix.md:78-95` confirms import/export and protocol comparison.
- READ-033 Existing parity matrix AI: `competitor-parity-matrix.md:108-120` confirms T0/T1/T2 vocabulary.
- READ-034 SLO scan: `rg -n "tier:" microservices/docs/slos` found tier labels in all SLO files.
- READ-035 Capability scan: `rg -n "tier:" microservices/docs/capabilities` found `tier: T0`, `tier: T1`, and `tier: T2`.
- READ-036 Vocabulary scan: `rg -n "demo_trial|paid|paid|compliance_pack" microservices/docs` found 34 explicit retirement candidates.
- READ-037 Tenant-class scan: no matches for `tenant_class`, `demo_trial`, `revenue_share`, or standalone `paid`.
- READ-038 Forbidden extension scan: no forbidden source files found under the microservice path.
- READ-039 IaC scan: only Helm and Kustomize files found under `microservices/docs/iac`.
- READ-040 Root file scan: no root README, no supported-oses manifest, and no cross-microservice handoff file found.

## Appendix C - Audit Verdict

- Product purpose: coherent, but root ownership docs incomplete.
- Artifact breadth: strong, but canonical execution surfaces missing.
- Contract posture: strong API/event/proto presence, weak tenant-class declaration.
- Canonical direction: failing until OpenTofu, contexts, OS manifest, tenant-class, and tier retirement are fixed.
- Counterpart parity: directionally strong, but existing files must be rewritten to top-3/current/no-tenant-class-drift posture.
- Operations: strong SLO/runbook count, weak deployment/tenant overlays.
- Security/compliance: strong design intent, weak tenant-class codification.
- Implementation feasibility: plausible Rust-first plan, no local source/test proof.
- Handoff readiness: named owner, broad dependencies, missing dedicated handoff.
- Final audit disposition: REVISE.

<!-- ORCHESTRATOR REPORT
  µservice: docs
  deliverables_landed: microservices/docs/coherence-audit-2026-05-20.md (625 lines); microservices/docs/feature-parity-matrix-2026-05-20.md (412 lines); microservices/docs/performance-benchmark-numbers-2026-05-20.md (322 lines)
  inventory_files_seen: 128
  inventory_lines_read: 20096
  chat_history_matches_processed: 4 high-signal line ranges
  findings_p0: 0
  findings_p1: 2
  findings_p2: 16
  findings_p3: 3
  tier_retirement_candidates_found: 34 explicit demo_trial/paid/paid/compliance_pack candidates: onboarding/docs-engineer-first-week.md:23; onboarding/docs-engineer-first-week.md:59; migration-playbooks/from-google-docs-and-notion.md:45; migration-playbooks/from-google-docs-and-notion.md:165; tutorials/create-collab-edit-branch-merge-sign.md:10; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:11; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:22; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:33; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:34; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:45; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:56; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:67; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:78; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:85; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:91; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:94; benchmarks/docs-vs-google-docs-vs-word-online-vs-notion-vs-coda-vs-quip.md:95; tenant-class-adoption/tenant-class-adoption-record.md:11; tenant-class-adoption/tenant-class-adoption-record.md:29; tenant-class-adoption/tenant-class-adoption-record.md:49; tenant-class-adoption/tenant-class-adoption-record.md:69; tenant-class-adoption/tenant-class-adoption-record.md:77; tenant-class-adoption/tenant-class-adoption-record.md:85; tenant-class-adoption/tenant-class-adoption-record.md:95; faqs/docs-engineer-faq.md:65; faqs/docs-engineer-faq.md:78; faqs/docs-engineer-faq.md:109; faqs/docs-engineer-faq.md:110; faqs/docs-engineer-faq.md:111; faqs/docs-engineer-faq.md:112; faqs/docs-engineer-faq.md:120; faqs/docs-engineer-faq.md:128; faqs/docs-engineer-faq.md:142; faqs/docs-engineer-faq.md:148
  tenant_class_adoption_gaps: yes - no tenant_class, demo_trial, paid, or revenue_share semantics found under microservices/docs
  top_3_counterparts_confirmed: Google Docs / Microsoft Word Online / Notion Docs
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1359
-->
