---
doc_class: MicroserviceOwnershipCoherenceAudit
microservice: notes
audit_date: 2026-05-20
audit_owner: codex
scope: /Users/jasonlee/oyatie/microservices/notes
deliverable_set: Wave 3 Batch 3.2
counterparts: [Notion, Obsidian, Apple Notes]
tier_delta_deliverable: retired_per_2026_05_20_directive
status: landed
---
# notes µservice ownership-coherence audit
## Header
- Target µservice: `notes`.
- Target path: `microservices/notes/`.
- Deployable-context assumption: all six canonical contexts unless audit evidence narrows the claim.
- Counterpart bar: Notion, Obsidian, Apple Notes.
- Deliverable count: three documents; the former tenant-class-deltas document is retired.
- Audit method: read canonical docs, memory directives, chat-history queue context, all notes files, and public counterpart references.
- Inventory basis: `find microservices/notes -type f | sort` returned 160 files.
- Inventory line basis: `find microservices/notes -type f -print0 | xargs -0 wc -l | tail -n 1` returned 26395 total lines.
- Chat-history scan: `rg -n -i "notes" .../8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` returned 343 hits.
- Chat-history counterpart anchor: queue line 16311 assigns `notes` to Notion, Obsidian, and Apple Notes.
- Canonical deployment contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider` per `specs/master-plan-sequencing.json:704-745`.
- Canonical IaC substrate: OpenTofu only, with six context paths and OCI Always Free profile module per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2644`.
- Canonical OS support: 13 primary OS families plus ppc64le and s390x test lanes per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2646-3044`.
- Canonical language policy: Rust backend; Swift/Kotlin/WinUI3 native frontends; Leptos/WASM-SSR selective-island web per `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3045-3490`.
- Canonical tier amendment: demo_trial/paid/paid/paid compliance_pack feature tiers are retired; tenant-class semantics replace tier differentiation per `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:10-44` and the current task directive.
- Tenant-class model used by this audit: `demo_trial`, `paid`, `revenue_share`.
- Quality bar: uniform industry-leader-grade across all tenant classes, not tier-stratified.
- New-content constraint: this audit names demo_trial/paid/paid/paid compliance_pack only as retirement candidates, not as a planning framework.
## §1 Purpose
- §1.1 This audit decides whether the `notes` µservice artifacts cohere as an owned product surface, implementation plan, deployment unit, compliance surface, and canonical-direction participant.
- §1.2 The product purpose is strong: PRD lines 18-26 define personal-first notes, bidirectional knowledge graph, E2E-default capture, Markdown/frontmatter, backlinks, daily notes, templates, web clipper, share-links, drive embeds, tasks handoff, search, graph view, imports, exports, optional Loro collaboration, and AI assist for non-E2E content.
- §1.3 The product differentiation is explicit: PRD lines 28-44 position notes apart from docs by E2E-by-default personal capture, tenant-DEK professional context, wikilinks, graph view, import/export breadth, and web clipper capture.
- §1.4 The counterpart bar is appropriate: Notion covers block/database/workspace collaboration, Obsidian covers local-first Markdown and graph knowledge work, and Apple Notes covers native capture, locked notes, document scanning, sharing, and platform integration.
- §1.5 The audit tests whether the file corpus is internally consistent enough for a junior engineer to start implementing from it without guessing product boundaries.
- §1.6 The audit also tests whether the file corpus is externally aligned with the post-2026-05-20 canonical doctrine: six deployable contexts, OpenTofu only, OS support matrix, Rust-strict backend, OCI Always Free profile, tier retirement, and tenant-class adoption.
- §1.7 This is not an implementation audit of live Rust code because the inventory has no `src/`, no `tests/`, no `Cargo.toml`, and no `.rs` files under `microservices/notes`.
- §1.8 The absence of code is itself a coherence signal because PRD and IPs repeatedly reference crates, tests, CI lanes, and `cargo` commands.
- §1.9 The audit does not author the retired fourth tier-deltas deliverable.
- §1.10 The audit does not touch other µservices, shared docs, ADRs, specs, or registry files.
- §1.11 The audit treats existing tier language as evidence to retire, not as a valid product model.
- §1.12 The audit treats `Personal` and `Professional` context-kind language as a domain privacy model; it is not equivalent to `tenant_class`.
- §1.13 The audit treats `T0`, `T1`, and `T2` capability naming as tier-like language needing Wave 15J review where it functions as feature-level differentiation.
- §1.14 The audit treats "reference signals" as observability terminology, not a paid feature tier, unless the same file also uses paid as a tier.
- §1.15 The audit distinguishes product strength from delivery coherence: a strong PRD can still fail deployability and canonical-direction gates.
- §1.16 The audit distinguishes declarative Kubernetes manifests from canonical OpenTofu context modules.
- §1.17 The audit distinguishes old Terraform references from allowed OpenTofu paths; IP-001 still invokes `terraform`, which is forbidden by current doctrine.
- §1.18 The audit sets severity by blocker class: P0 data/safety impossibility, P1 deployability or canonical gate blocker, P2 documentation/coherence gap, P3 polish or local cleanup.
- §1.19 No P0 is assigned because the audit found no live implementation path that decrypts or leaks user data.
- §1.20 P1 findings are assigned where docs claim deployability or implementation readiness but required context, OpenTofu, OS, or code evidence is absent.
## §2 Inventory
- §2.1 Inventory command: `find microservices/notes -type f | sort`.
- §2.2 File count: 160.
- §2.3 Total lines read or scanned: 26395.
- §2.4 Required primary docs present: `PRD.md`, `ARCHITECTURE.md`, `README.md`, `manifest.json`.
- §2.5 Required ADR family present: `decisions/ADR-NOTES-0001..0006.md`, `decisions/ADR-NTS-001-block-based-data-model-with-bidirectional-links.md`, and `decisions/README.md`.
- §2.6 Required implementation plans present: `IP-001` through `IP-018`, plus journey-specific IP files.
- §2.7 Required contracts present: OpenAPI 3.2.0, AsyncAPI 3.1.0, and proto3.
- §2.8 Required SLO files present: ten OpenSLO documents.
- §2.9 Required tenant-class directory present: `tenant-class/tier-matrix.md`; this is now a retirement candidate.
- §2.10 Required cross-handoff file missing: `cross-microservice-handoffs.md` is not in the 160-file inventory.
- §2.11 Required `supported-oses.json` missing: `find microservices/notes -name 'supported-oses.json' -o -name '*oses*'` returned no files.
- §2.12 Required implementation source missing: `find microservices/notes -type d -name src` returned no directory.
- §2.13 Required implementation tests missing: `find microservices/notes -type d -name tests` returned no directory.
- §2.14 Rust-strict scan pass with caveat: forbidden backend code extensions `.py`, `.js`, `.ts`, `.tsx`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, `.fsx` returned no files, but there are also no Rust implementation files.
- I-001 | `microservices/notes/ARCHITECTURE.md` | architecture source; read to >200 lines and scanned for repeated expansion, context, IaC, tier, and deployment evidence.
- I-002 | `microservices/notes/AUDIT-FINDINGS-2026-05-18.json` | prior finding artifact; scanned as historical evidence, not current audit authority.
- I-003 | `microservices/notes/CHANGELOG.md` | top-level change log; inventoried.
- I-004 | `microservices/notes/IP-001-iac.md` | IaC implementation plan; read in full; contains current OpenTofu/Terraform contradiction.
- I-005 | `microservices/notes/IP-002-cargo-workspace-bootstrap.md` | workspace bootstrap plan; read in full; claims 111 crates with no `src/` present.
- I-006 | `microservices/notes/IP-003-note-store-kernel-domain.md` | implementation plan; scanned for domain shape and acceptance gates.
- I-007 | `microservices/notes/IP-004-tag-graph-kernel-domain.md` | implementation plan; scanned for Personal/Professional split.
- I-008 | `microservices/notes/IP-005-backlink-graph-kernel-domain.md` | implementation plan; scanned for wikilink and graph ownership.
- I-009 | `microservices/notes/IP-006-daily-note-template-gallery.md` | implementation plan; scanned for daily-note and template coverage.
- I-010 | `microservices/notes/IP-007-web-clipper-bridge.md` | implementation plan; scanned for browser-extension and non-Rust gate tension.
- I-011 | `microservices/notes/IP-008-share-link-and-embed.md` | implementation plan; scanned for share-link and drive embed evidence.
- I-012 | `microservices/notes/IP-009-checklist-and-version-history.md` | implementation plan; scanned for tasks handoff and version history.
- I-013 | `microservices/notes/IP-010-search-and-graph-view.md` | implementation plan; scanned for Meilisearch and graph view evidence.
- I-014 | `microservices/notes/IP-011-collab-edit-loro.md` | implementation plan; scanned for Loro collaboration scope.
- I-015 | `microservices/notes/IP-012-import-export-pipelines.md` | implementation plan; scanned for import/export coverage.
- I-016 | `microservices/notes/IP-013-ai-assist-and-e2e-refusal.md` | implementation plan; scanned for AI refusal.
- I-017 | `microservices/notes/IP-014-e2e-key-management.md` | implementation plan; scanned for MLS key management.
- I-018 | `microservices/notes/IP-015-hg-notes-conformance.md` | implementation plan; inventoried.
- I-019 | `microservices/notes/IP-016-collab-edit-mls-loro-hardening.md` | implementation plan; inventoried.
- I-020 | `microservices/notes/IP-017-hipaa-clinical-notes-overlay.md` | implementation plan; scanned for clinical overlay.
- I-021 | `microservices/notes/IP-018-abuse-defence-edge-wiring.md` | implementation plan; scanned for abuse defence.
- I-022 | `microservices/notes/IP-journey-j07-memory-preserving-notes-handoff.md` | journey IP; inventoried.
- I-023 | `microservices/notes/IP-journey-j100-pack-rollout-first-action.md` | journey IP; inventoried.
- I-024 | `microservices/notes/IP-journey-j11-offline-crdt-merge.md` | journey IP; inventoried.
- I-025 | `microservices/notes/IP-journey-j128-tax-year-index.md` | journey IP; inventoried.
- I-026 | `microservices/notes/IP-journey-j144-applications-database.md` | journey IP; inventoried.
- I-027 | `microservices/notes/IP-journey-j25-e2e-crdt-journal.md` | journey IP; inventoried.
- I-028 | `microservices/notes/IP-journey-j39-transcript-search-index.md` | journey IP; contains `authority_tier` and `cell_tier` language.
- I-029 | `microservices/notes/IP-journey-j43-shift-handoff-note.md` | journey IP; contains `authority_tier` and `cell_tier` language.
- I-030 | `microservices/notes/IP-journey-j44-consult-note.md` | journey IP; inventoried.
- I-031 | `microservices/notes/IP-journey-j45-record-correction-request.md` | journey IP; inventoried.
- I-032 | `microservices/notes/IP-journey-j57-week-one-notes.md` | journey IP; inventoried.
- I-033 | `microservices/notes/IP-journey-j58-action-items.md` | journey IP; inventoried.
- I-034 | `microservices/notes/IP-journey-j61-soap-note.md` | journey IP; inventoried.
- I-035 | `microservices/notes/IP-journey-j62-prescription-record.md` | journey IP; inventoried.
- I-036 | `microservices/notes/IP-journey-j63-study-data.md` | journey IP; inventoried.
- I-037 | `microservices/notes/IP-journey-j69-briefing-notes.md` | journey IP; inventoried.
- I-038 | `microservices/notes/IP-journey-j72-translated-minutes.md` | journey IP; inventoried.
- I-039 | `microservices/notes/IP-journey-j85-clinical-note-boundary.md` | journey IP; inventoried.
- I-040 | `microservices/notes/IP-journey-j91-us-msb-mtl-overlay.md` | journey IP; inventoried.
- I-041 | `microservices/notes/IP-journey-j92-br-lgpd-us-parent-dsar.md` | journey IP; inventoried.
- I-042 | `microservices/notes/IP-journey-j93-in-dpdpa-rbi-overlay.md` | journey IP; scanned; repeated merchant KYC tiering language.
- I-043 | `microservices/notes/IP-journey-j94-sox404-public-company-controls.md` | journey IP; inventoried.
- I-044 | `microservices/notes/IP-journey-j95-iso27001-soc2-annual-audit.md` | journey IP; inventoried.
- I-045 | `microservices/notes/IP-journey-j96-ksa-uae-mena-onboarding.md` | journey IP; inventoried.
- I-046 | `microservices/notes/IP-journey-j97-sg-pdpa-mas-tenant.md` | journey IP; inventoried.
- I-047 | `microservices/notes/IP-journey-j98-au-privacy-apra-cps234.md` | journey IP; inventoried.
- I-048 | `microservices/notes/IP-journey-j99-multi-pack-conflict-resolution.md` | journey IP; inventoried.
- I-049 | `microservices/notes/PHASE-01-NOTES-FOUNDATION.md` | phase plan; inventoried and scanned for IP registry.
- I-050 | `microservices/notes/PRD.md` | primary product source; read in full.
- I-051 | `microservices/notes/README.md` | µservice overview; read in full.
- I-052 | `microservices/notes/backfill-replay.md` | operational recovery doc; read/scanned for backfill and tier language.
- I-053 | `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md` | current benchmark doc; read in full; uses retired paid/paid model.
- I-054 | `microservices/notes/capabilities/T0-suggest.yaml` | capability doc; inventoried and scanned as tier-like capability naming.
- I-055 | `microservices/notes/capabilities/T1-assist.yaml` | capability doc; inventoried and scanned as tier-like capability naming.
- I-056 | `microservices/notes/capabilities/T2-auto.yaml` | capability doc; inventoried and scanned as tier-like capability naming.
- I-057 | `microservices/notes/tenant-class/tier-matrix.md` | tier matrix; read in full; retirement candidate.
- I-058 | `microservices/notes/capacity-model.md` | capacity model; read in full.
- I-059 | `microservices/notes/catalog/oya-notes-backlink-graph-kernel.yaml` | catalog record; inventoried.
- I-060 | `microservices/notes/catalog/oya-notes-checklist-kernel.yaml` | catalog record; inventoried.
- I-061 | `microservices/notes/catalog/oya-notes-collab-edit-adapter-loro.yaml` | catalog record; inventoried.
- I-062 | `microservices/notes/catalog/oya-notes-collab-edit-kernel.yaml` | catalog record; inventoried.
- I-063 | `microservices/notes/catalog/oya-notes-daily-note-kernel.yaml` | catalog record; inventoried.
- I-064 | `microservices/notes/catalog/oya-notes-e2e-key-management-adapter-mls.yaml` | catalog record; inventoried.
- I-065 | `microservices/notes/catalog/oya-notes-mls-key-escrow-adapter-openbao.yaml` | catalog record; inventoried.
- I-066 | `microservices/notes/catalog/oya-notes-note-store-adapter-postgres.yaml` | catalog record; inventoried.
- I-067 | `microservices/notes/catalog/oya-notes-note-store-adapter-valkey.yaml` | catalog record; inventoried.
- I-068 | `microservices/notes/catalog/oya-notes-note-store-adapter-s3.yaml` | catalog record; inventoried.
- I-069 | `microservices/notes/catalog/oya-notes-note-store-kernel.yaml` | catalog record; inventoried.
- I-070 | `microservices/notes/catalog/oya-notes-phi-classifier-kernel.yaml` | catalog record; inventoried.
- I-071 | `microservices/notes/catalog/oya-notes-search-index-adapter-meilisearch.yaml` | catalog record; inventoried.
- I-072 | `microservices/notes/catalog/oya-notes-search-index-kernel.yaml` | catalog record; inventoried.
- I-073 | `microservices/notes/catalog/oya-notes-share-link-adapter-postgres.yaml` | catalog record; inventoried.
- I-074 | `microservices/notes/catalog/oya-notes-share-link-kernel.yaml` | catalog record; inventoried.
- I-075 | `microservices/notes/catalog/oya-notes-tag-graph-kernel.yaml` | catalog record; inventoried.
- I-076 | `microservices/notes/catalog/oya-notes-template-gallery-kernel.yaml` | catalog record; inventoried.
- I-077 | `microservices/notes/catalog/oya-notes-web-clipper-bridge-kernel.yaml` | catalog record; inventoried.
- I-078 | `microservices/notes/competitor-parity-matrix.md` | existing competitor matrix; read in full.
- I-079 | `microservices/notes/compliance.md` | compliance doc; scanned for repeated tier metadata and dependencies.
- I-080 | `microservices/notes/contracts/asyncapi/notes-events.yaml` | AsyncAPI contract; read to 260 lines.
- I-081 | `microservices/notes/contracts/openapi/notes.yaml` | OpenAPI contract; read in full.
- I-082 | `microservices/notes/contracts/proto/notes.proto` | proto contract; read in full.
- I-083 | `microservices/notes/cost-budget.md` | cost model; scanned for billing, tier, and OCI evidence.
- I-084 | `microservices/notes/dashboards/abuse-defence-outcomes.json` | dashboard; inventoried.
- I-085 | `microservices/notes/dashboards/e2e-encryption-health.json` | dashboard; inventoried.
- I-086 | `microservices/notes/dashboards/privacy-and-e2e-health.json` | dashboard; inventoried.
- I-087 | `microservices/notes/dashboards/search-and-graph.json` | dashboard; inventoried.
- I-088 | `microservices/notes/dashboards/sync-and-realtime.json` | dashboard; inventoried.
- I-089 | `microservices/notes/decisions/ADR-NOTES-0001-e2e-encryption-default-personal-tier.md` | ADR; scanned for E2E posture and tier terminology.
- I-090 | `microservices/notes/decisions/ADR-NOTES-0002-bidirectional-link-and-graph-storage.md` | ADR; scanned for graph architecture.
- I-091 | `microservices/notes/decisions/ADR-NOTES-0003-crdt-library-for-optional-collab.md` | ADR; scanned for Loro decision.
- I-092 | `microservices/notes/decisions/ADR-NOTES-0004-search-architecture-respecting-e2e.md` | ADR; scanned for Meilisearch and Personal client-side search.
- I-093 | `microservices/notes/decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md` | ADR; scanned for AI refusal.
- I-094 | `microservices/notes/decisions/ADR-NOTES-0006-portable-export-and-import-format.md` | ADR; scanned for import/export.
- I-095 | `microservices/notes/decisions/ADR-NTS-001-block-based-data-model-with-bidirectional-links.md` | successor ADR; scanned for block-model tension.
- I-096 | `microservices/notes/decisions/README.md` | ADR index; scanned for current ADR surface.
- I-097 | `microservices/notes/dpia.md` | privacy impact doc; scanned for GDPR/HIPAA and tier language.
- I-098 | `microservices/notes/failure-modes.md` | failure-mode doc; scanned for SLO risks.
- I-099 | `microservices/notes/faqs/notes-engineer-faq.md` | FAQ; scanned for exact demo_trial/paid/paid references.
- I-100 | `microservices/notes/iac/ech-config.yaml` | Kubernetes/IaC file; inventoried.
- I-101 | `microservices/notes/iac/edge-waf.yaml` | Kubernetes/IaC file; scanned for per-tenant limit naming.
- I-102 | `microservices/notes/iac/helm/notes/Chart.yaml` | Helm chart; inventoried.
- I-103 | `microservices/notes/iac/helm/notes/templates/deployment.yaml` | Helm deployment; inventoried.
- I-104 | `microservices/notes/iac/helm/notes/templates/hpa.yaml` | Helm HPA; inventoried.
- I-105 | `microservices/notes/iac/helm/notes/templates/networkpolicy.yaml` | Helm network policy; inventoried.
- I-106 | `microservices/notes/iac/helm/notes/templates/pdb.yaml` | Helm PDB; inventoried.
- I-107 | `microservices/notes/iac/helm/notes/templates/prometheusrule.yaml` | Helm PrometheusRule; inventoried.
- I-108 | `microservices/notes/iac/helm/notes/templates/service.yaml` | Helm service; inventoried.
- I-109 | `microservices/notes/iac/helm/notes/templates/servicemonitor.yaml` | Helm ServiceMonitor; inventoried.
- I-110 | `microservices/notes/iac/helm/notes/values.yaml` | Helm values; inventoried.
- I-111 | `microservices/notes/iac/kustomize/base/kustomization.yaml` | Kustomize base; inventoried.
- I-112 | `microservices/notes/iac/kustomize/base/namespace.yaml` | Kustomize namespace; inventoried.
- I-113 | `microservices/notes/iac/kustomize/overlays/pack-eu/kustomization.yaml` | Kustomize pack overlay; inventoried.
- I-114 | `microservices/notes/iac/kustomize/overlays/pack-kr/kustomization.yaml` | Kustomize pack overlay; inventoried.
- I-115 | `microservices/notes/iac/openbao-policy.yaml` | secret policy config; inventoried.
- I-116 | `microservices/notes/iac/pqc-cert.yaml` | certificate config; inventoried.
- I-117 | `microservices/notes/iac/secret-bindings.yaml` | secret bindings; inventoried.
- I-118 | `microservices/notes/incident-response.md` | incident response; scanned for privacy and outage handling.
- I-119 | `microservices/notes/manifest.json` | manifest; read in full.
- I-120 | `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md` | migration playbook; scanned for counterpart gaps and exact paid references.
- I-121 | `microservices/notes/multi-region.md` | multi-region doc; scanned for OCI pack placement and RTO/RPO.
- I-122 | `microservices/notes/onboarding/notes-engineer-first-week.md` | onboarding; scanned for exact paid reference.
- I-123 | `microservices/notes/policy/abuse-defence.cedar` | Cedar policy; inventoried.
- I-124 | `microservices/notes/policy/auditor-scope.cedar` | Cedar policy; inventoried.
- I-125 | `microservices/notes/policy/ci-scope.cedar` | Cedar policy; inventoried.
- I-126 | `microservices/notes/policy/data-residency.md` | residency policy; scanned for Personal-tier constraints.
- I-127 | `microservices/notes/policy/dual-context-isolation.md` | dual-context policy; scanned for Personal/Professional separation.
- I-128 | `microservices/notes/policy/e2e-personal-tier-default.md` | E2E policy; read/scanned for tier posture.
- I-129 | `microservices/notes/policy/minor-protection.cedar` | Cedar policy; scanned for `tier` field.
- I-130 | `microservices/notes/policy/pci-payments-notes.cedar` | Cedar policy; inventoried.
- I-131 | `microservices/notes/policy/phi-hipaa-notes.cedar` | Cedar policy; scanned for clinical-tier language.
- I-132 | `microservices/notes/policy/public-read.cedar` | Cedar policy; scanned for Personal-tier sharing.
- I-133 | `microservices/notes/policy/share-link-scope.cedar` | Cedar policy; scanned for E2E personal-tier default.
- I-134 | `microservices/notes/policy/tenant-scope.cedar` | Cedar policy; scanned for Personal-tier and AI/collab forbids.
- I-135 | `microservices/notes/reference-implementations/block-edit-and-link-rust-sdk.md` | reference implementation; scanned for paid compliance_pack reference.
- I-136 | `microservices/notes/runbooks/ai-classifier-rollback-e2e-respect.md` | runbook; scanned for capability tier rollback.
- I-137 | `microservices/notes/runbooks/attachment-loss-recovery.md` | runbook; scanned for attachment recovery.
- I-138 | `microservices/notes/runbooks/clinical-note-leak-recovery.md` | runbook; inventoried.
- I-139 | `microservices/notes/runbooks/crdt-divergence-recovery.md` | runbook; inventoried.
- I-140 | `microservices/notes/runbooks/e2e-key-rotation-and-recovery.md` | runbook; scanned for key recovery.
- I-141 | `microservices/notes/runbooks/import-pipeline-failure.md` | runbook; scanned for import risks.
- I-142 | `microservices/notes/runbooks/notes-bot-score-recalibration.md` | runbook; inventoried.
- I-143 | `microservices/notes/runbooks/notes-share-link-revocation.md` | runbook; scanned for B2C_PERSONAL_E2E tenant_class reference.
- I-144 | `microservices/notes/runbooks/sync-conflict-resolution.md` | runbook; scanned for conflict recovery.
- I-145 | `microservices/notes/runbooks/tag-graph-corruption.md` | runbook; scanned for graph recovery.
- I-146 | `microservices/notes/runbooks/web-clipper-degraded.md` | runbook; inventoried.
- I-147 | `microservices/notes/scorecards/overrides.json` | scorecard override; inventoried.
- I-148 | `microservices/notes/sdk-plan.md` | SDK plan; scanned for TypeScript/Wasm and Rust SDK claims.
- I-149 | `microservices/notes/slos/collab-edit-merge-latency.openslo.yaml` | OpenSLO file; scanned.
- I-150 | `microservices/notes/slos/e2e-privacy-correctness.openslo.yaml` | OpenSLO file; scanned.
- I-151 | `microservices/notes/slos/full-text-search-latency.openslo.yaml` | OpenSLO file; scanned.
- I-152 | `microservices/notes/slos/graph-render-latency.openslo.yaml` | OpenSLO file; scanned.
- I-153 | `microservices/notes/slos/note-create-latency.openslo.yaml` | OpenSLO file; scanned.
- I-154 | `microservices/notes/slos/note-open-latency.openslo.yaml` | OpenSLO file; scanned.
- I-155 | `microservices/notes/slos/sync-data-residency-correctness.openslo.yaml` | OpenSLO file; scanned.
- I-156 | `microservices/notes/slos/sync-latency.openslo.yaml` | OpenSLO file; scanned.
- I-157 | `microservices/notes/slos/tag-search-latency.openslo.yaml` | OpenSLO file; scanned.
- I-158 | `microservices/notes/slos/web-clipper-capture-latency.openslo.yaml` | OpenSLO file; scanned.
- I-159 | `microservices/notes/threat-model.md` | threat model; scanned for data/security shape.
- I-160 | `microservices/notes/tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md` | tutorial; scanned for exact paid/paid references.
## §3 9-dimension audit
### §3.1 Dimension 1 — Product purpose and ownership
- D1.1 Finding: product purpose is clear and notes-specific.
- D1.2 Evidence: PRD lines 18-26 define note CRUD, Markdown/frontmatter, tags, backlinks, daily notes, templates, web clipper, share-links, embeds, checklist extraction, search, graph, import/export, optional Loro collaboration, and AI assist.
- D1.3 Evidence: PRD lines 28-44 distinguish notes from docs by private capture, E2E default, wikilinks, graph view, and import/export.
- D1.4 Evidence: README lines 16-18 summarize the product as Notion+Obsidian+Apple Notes style capture with Personal E2E and work search/intelligence.
- D1.5 Evidence: OpenAPI lines 316-502 expose CRUD, tags, backlinks, graph, daily notes, templates, search, share, assist, import, export, and web-clipper endpoints.
- D1.6 Evidence: AsyncAPI lines 23-48 enumerate Workflow events for note lifecycle, tags, backlinks, checklist, share link, daily note, web clipper, import/export, AI, disclosure, and MLS epoch.
- D1.7 Assessment: the µservice owns a coherent product slice, not merely a generic document service.
- D1.8 Risk: current `manifest.json` narrows bounded contexts to one `notes` bounded context at lines 6-29 while PRD and IP-002 claim 17 bounded contexts and 111 crates.
- D1.9 Risk severity: P2 because a junior implementer cannot know whether to follow the broad PRD/IP plan or the narrow manifest.
- D1.10 Recommendation: update manifest to enumerate the same BC set as IP-002 lines 31-49 or explicitly retire the 111-crate plan.
- D1.11 Product boundary: notes consumes drive for embeds, tasks for checklist items, intelligence for AI, audit-chain for seals, tenancy/identity for scope, and cloud-iac for substrate per manifest lines 383-394.
- D1.12 Missing ownership surface: no `cross-microservice-handoffs.md` exists despite PRD lines 220-247 and manifest lines 383-394 naming cross-service event and dependency flows.
- D1.13 Severity: P2 for handoff absence; it is not a product blocker, but it weakens ownership handoff.
- D1.14 Counterpart fit: Notion is the right collaboration/database benchmark, Obsidian is the right Markdown graph benchmark, and Apple Notes is the right native secure capture benchmark.
- D1.15 Chat corroboration: chat-history line 16311 assigns notes to Notion, Obsidian, and Apple Notes in the rolling audit queue.
- D1.16 Product purpose verdict: strong.
- D1.17 Ownership artifact verdict: mostly strong but manifest and handoff gaps must be cleaned.
- D1.18 Implementation readiness verdict: blocked by missing code and tests, not by product ambiguity.
- D1.19 Risk of false confidence: high because architecture and benchmarks contain "achieved" language without code or test evidence.
- D1.20 Stop condition for D1 closure: manifest, PRD, IP-002, catalog, and handoff docs agree on bounded contexts and dependencies.
### §3.2 Dimension 2 — Artifact completeness and inventory maturity
- D2.1 Finding: documentation breadth is high with 160 files and 26395 total lines.
- D2.2 Evidence: primary product docs are present: PRD, README, ARCHITECTURE, manifest.
- D2.3 Evidence: contracts are present and modern: OpenAPI 3.2.0 at `contracts/openapi/notes.yaml:1`, AsyncAPI 3.1.0 at `contracts/asyncapi/notes-events.yaml:1`, and proto3 at `contracts/proto/notes.proto:5`.
- D2.4 Evidence: ten OpenSLO files exist under `slos/`.
- D2.5 Evidence: multiple runbooks exist for AI rollback, attachments, clinical leakage, CRDT divergence, E2E keys, imports, abuse scoring, share revocation, sync conflicts, tag corruption, and web clipper degradation.
- D2.6 Evidence: dashboards exist for abuse, E2E encryption, privacy, search/graph, and sync/realtime.
- D2.7 Evidence: compliance, DPIA, threat model, failure modes, incident response, cost budget, capacity model, multi-region, backfill, and SDK plan are present.
- D2.8 Gap: no `src/` directory exists despite IP-002 lines 23-25 requiring workspace members and 111 crate skeletons.
- D2.9 Gap: no `tests/` directory exists despite PRD lines 344-360 and many IPs referencing `tests/e2e`, `tests/regression`, `cargo nextest`, and compile-fail checks.
- D2.10 Gap: no `cross-microservice-handoffs.md` exists despite dependencies on drive, tasks, tenancy, identity, observability, cell, audit-chain, intelligence, detection, and cloud-iac.
- D2.11 Gap: manifest lines 121-211 list only IP-001 through IP-015, while inventory contains IP-016 through IP-018 and many journey IPs.
- D2.12 Gap: the existing architecture doc repeats generic "Content-pass expansion" patterns; ARCHITECTURE lines 62-70 and repeated deployment-evidence lines 106, 217, 272, 327, 382, 437, 492, 547, 606, 657, 714, 768, 823, 880, 934, 1000, and 1055 indicate mechanical expansion rather than a stable implementation blueprint.
- D2.13 Canonical bar: brief-template §6 anti-patterns lines 1720-1855 reject scaffold, line-count-only, variable swaps, recycled boilerplate, scripted bodies, and clause-loop padding.
- D2.14 Severity: P2 for architecture substance cleanup because it does not block the product boundary but weakens implementation trust.
- D2.15 Positive: PRD line count and content depth are not shallow; PRD lines 58-83 list 24 functional requirements.
- D2.16 Positive: PRD lines 87-101 give concrete latency budgets.
- D2.17 Positive: PRD lines 103-113 define security invariants around E2E, tenant-DEK, AI refusal, share tokens, and context drift.
- D2.18 Positive: PRD lines 123-127 define availability, RTO, and RPO expectations.
- D2.19 Positive: PRD lines 268-301 compare many note products, including the three audit counterparts.
- D2.20 Completeness verdict: broad docs are present; executable substrate is not present.
### §3.3 Dimension 3 — Internal consistency and contradiction scan
- D3.1 Finding: the corpus has several material contradictions.
- D3.2 Contradiction: PRD line 175 claims 111 crates; IP-002 lines 31-51 also enumerate 111 crates; manifest lines 6-29 list only one bounded context with 16 crates.
- D3.3 Contradiction impact: implementers cannot know whether `notes` is a 17-BC product suite or a one-BC manifest projection.
- D3.4 Severity: P2; severe enough to block clean implementation planning but not an immediate safety issue.
- D3.5 Contradiction: capacity model line 33 says XL-max creates are 20k notes/sec, while PRD line 331 says shard once a cell hits 500k notes/sec aggregate.
- D3.6 Contradiction impact: performance benchmark and scaling targets can be off by 25x.
- D3.7 Severity: P1 for benchmark truth because capacity targets feed deployability and cost overlays.
- D3.8 Contradiction: IP-001 line 24 says one OpenTofu module for Grafana RBAC, but IP-001 line 43 invokes `terraform -chdir=microservices/notes/iac/tofu validate`.
- D3.9 Contradiction impact: current doctrine forbids Terraform binary and requires OpenTofu modules; no `iac/tofu` directory exists in the inventory.
- D3.10 Severity: P1 because OpenTofu-only is a canonical deployment gate.
- D3.11 Contradiction: IP-001 line 35 targets `microservices/notes/iac/terraform/grafana-rbac.tf`, while `find microservices/notes/iac -maxdepth 4 -type f` returned no Terraform or OpenTofu file.
- D3.12 Contradiction impact: the IaC plan declares a file target that is not present.
- D3.13 Severity: P1 because it undermines deployability evidence.
- D3.14 Contradiction: README lines 16-18 and PRD lines 103-113 use `Personal-tier` and `Professional-tier`, but the task directive retires feature tiers and requires tenant-class adoption.
- D3.15 Clarification: Personal/Professional may remain as privacy context names if rewritten away from tier semantics.
- D3.16 Severity: P2 documentation model migration.
- D3.17 Contradiction: benchmark file line 30 says paid and paid page-open targets are "achieved"; the inventory has no benchmark harness, no `benchmarks/notesbench/`, no Rust code, and no output file evidence.
- D3.18 Severity: P2 for unsupported benchmark claim; P1 if used as deployment readiness evidence.
- D3.19 Contradiction: IP-007 line 48 invokes `npm run test --prefix extensions/chrome`, but canonical Rust-strict doctrine forbids JS app logic and allows frontend/web surfaces only through scoped allowlists.
- D3.20 Severity: P2 pending classification; browser extension work needs an explicit allowed surface or Rust/WASM/Leptos-compatible policy exception.
- D3.21 Contradiction: SDK plan lines 26-28 names TypeScript SDK bindings, while D-18 allows Rust backend and specific frontend languages; TypeScript must be generator/binding-only, not app logic.
- D3.22 Severity: P2 classification gap, not a direct violation because no `.ts` file exists.
- D3.23 Contradiction: compliance lines 594, 656, 718, 780, 842, 904, 966, 1028, and 1090 repeat `tier product`; manifest line 359 says `tier_classification: hero-product`; canonical tier retirement makes this terminology stale.
- D3.24 Severity: P2 Wave 15J retirement candidate.
- D3.25 Internal consistency verdict: product and privacy architecture are coherent; deployment, manifest, capacity, and terminology need cleanup.
### §3.4 Dimension 4 — Canonical-direction alignment
- D4.1 Canonical source: ADR-0328 §D-15 through §D-20 establishes the audit dimensions and constraint surface.
- D4.2 Canonical source: `specs/master-plan-sequencing.json:704-745` defines the six deployment contexts.
- D4.3 Canonical source: `specs/master-plan-sequencing.json:747-775` defines OpenTofu-only IaC substrate and forbidden engines.
- D4.4 Canonical source: `specs/master-plan-sequencing.json:777-815` defines supported OS matrix.
- D4.5 Canonical source: `specs/master-plan-sequencing.json:817-856` defines language policy.
- D4.6 Canonical source: `specs/master-plan-sequencing.json:857-868` defines OCI Always Free profile constraints, with old tier language now superseded by the 2026-05-20 tier-retirement directive.
- D4.7 Canonical source: brief-template lines 666-807 require multi-context deployability evidence.
- D4.8 Canonical source: brief-template lines 809-965 require OpenTofu context modules and forbid Terraform/Pulumi/CloudFormation durable infra.
- D4.9 Canonical source: brief-template lines 967-1123 require OS support manifest.
- D4.10 Canonical source: brief-template lines 1125-1305 require Rust-strict backend and scoped frontend allowlist.
- D4.11 Alignment positive: contracts use OpenAPI 3.2.0 and AsyncAPI 3.1.0, matching brief-template expectations.
- D4.12 Alignment positive: many security policies are declarative Cedar, which D-18 allows as policy declarations.
- D4.13 Alignment positive: no forbidden backend code files were found under `microservices/notes`.
- D4.14 Alignment gap: no six-context OpenTofu directory exists.
- D4.15 Alignment gap: no supported-OS manifest exists.
- D4.16 Alignment gap: no tenant-class schema exists.
- D4.17 Alignment gap: old tier and tenant-class artifacts remain.
- D4.18 Alignment gap: IP-001 still names Terraform and Helm/Kustomize as primary IaC.
- D4.19 Alignment verdict: current notes docs are pre-amendment and need Wave 15J plus deployability remediation.
#### §3.4.T Tier retirement candidates
- T-000 Exact demo_trial/paid/paid/paid compliance_pack scan command: `rg -n -i "demo_trial|paid|paid|paid compliance_pack" microservices/notes`.
- T-001 Exact candidate: `microservices/notes/onboarding/notes-engineer-first-week.md:33` says expected page-open p95 is tied to "paid tenant_class"; severity P2.
- T-002 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:66` maps equation block import to "paid"; severity P2.
- T-003 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:68` maps database table-view import to "paid"; severity P2.
- T-004 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:72` maps timeline-view import to "paid"; severity P2.
- T-005 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:81` maps formula block import to "paid"; severity P2.
- T-006 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:82` maps template block import to "paid"; severity P2.
- T-007 Exact candidate: `microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:83` maps AI block import to "paid"; severity P2.
- T-008 Exact candidate: `microservices/notes/tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:15` requires "A paid tenant_class notes cell"; severity P2.
- T-009 Exact candidate: `microservices/notes/tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:181` says graph view is "paid tenant_class"; severity P2.
- T-010 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:13` labels hardware as "oyatie paid"; severity P2.
- T-011 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:21` benchmark row labels "oyatie notes paid"; severity P2.
- T-012 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:28` says "oyatie paid" is fastest among cloud-hosted; severity P2.
- T-013 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:30` states paid and paid page-open targets; severity P2.
- T-014 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:36` benchmark row labels "oyatie notes paid"; severity P2.
- T-015 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:43` says "oyatie paid leads cloud-hosted"; severity P2.
- T-016 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:61` benchmark row labels "oyatie notes paid"; severity P2.
- T-017 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:62` benchmark row labels "oyatie notes paid"; severity P2.
- T-018 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:69` says Qdrant ANN is used at paid; severity P2.
- T-019 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:88` TCO row labels "oyatie notes paid"; severity P2.
- T-020 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:89` TCO row labels "oyatie notes paid"; severity P2.
- T-021 Exact candidate: `microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:98` says "oyatie paid is competitive"; severity P2.
- T-022 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:13` defines demo_trial; severity P2.
- T-023 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:30` describes demo_trial tenants; severity P2.
- T-024 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:48` defines paid; severity P2.
- T-025 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:50` says paid adds to demo_trial; severity P2.
- T-026 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:82` defines paid; severity P2.
- T-027 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:84` says paid adds to paid; severity P2.
- T-028 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:112` compares paid and paid costs; severity P2.
- T-029 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:116` defines paid compliance_pack; severity P2.
- T-030 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:118` says paid compliance_pack adds to paid; severity P2.
- T-031 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:130` says latency same as paid; severity P2.
- T-032 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:132` says SLO posture same as paid; severity P2.
- T-033 Exact candidate: `microservices/notes/tenant-class/tier-matrix.md:144` defines demo_trial to paid to paid to paid compliance_pack promotion path; severity P2.
- T-034 Exact candidate: `microservices/notes/reference-implementations/block-edit-and-link-rust-sdk.md:250` references paid compliance_pack data-class violation; severity P2.
- T-035 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:30` says unidirectional system is demo_trial; severity P2.
- T-036 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:58` says T1 AI calls use paid/paid capacity envelope; severity P2.
- T-037 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:77` says semantic similarity is at paid; severity P2.
- T-038 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:80` gives paid and paid latency budgets; severity P2.
- T-039 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:118` asks about embedding pipeline at paid tenant_class; severity P2.
- T-040 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:134` says cross-workspace search at paid; severity P2.
- T-041 Exact candidate: `microservices/notes/faqs/notes-engineer-faq.md:143` bounds search to 50 workspaces at paid; severity P2.
- T-042 Not a retirement candidate: `manifest.json:307`, `manifest.json:357`, `failure-modes.md:126`, and capability eval paths use "reference" as observability/evaluation vocabulary, not feature-tier vocabulary.
- T-043 Broader tier-language candidate: `PRD.md:8` frontmatter says `tier: hero-product`; severity P2.
- T-044 Broader tier-language candidate: `README.md:47` says `Tier-0/1/2` cells; severity P2.
- T-045 Broader tier-language candidate: `manifest.json:330-334` declares `tenant_class`; severity P2.
- T-046 Broader tier-language candidate: `manifest.json:359` declares `tier_classification`; severity P2.
- T-047 Broader tier-language candidate: `manifest.json:396` declares `criticality_tier`; severity P2 unless terminology is retained solely for operational criticality and renamed to avoid feature-tier confusion.
- T-048 Broader tier-language candidate: `policy/e2e-personal-tier-default.md:29-32` expresses Personal/Professional as tiers; severity P2 rewrite to privacy contexts.
- T-049 Broader tier-language candidate: `backfill-replay.md:119-122` uses Business/Enterprise tier RTO rows; severity P2.
- T-050 Broader tier-language candidate: `cost-budget.md:72` says "Free-tier"; severity P2 rewrite to `demo_trial`.
- T-051 Broader tier-language candidate: `cost-budget.md:87` says `tier-allowance`; severity P2 rewrite to contract or tenant-class allowance.
- T-052 Retirement action: delete or replace `tenant-class/tier-matrix.md` in Wave 15J, not in this audit.
- T-053 Retirement action: rewrite benchmark and tutorial surfaces to use single target set plus deployment-context and tenant-class overlays.
- T-054 Retirement action: rewrite `Personal-tier` and `Professional-tier` to `context_kind=Personal` and `context_kind=Professional` where the model is privacy context, not commercial tier.
- T-055 Retirement action: rename `Free-tier` to `demo_trial tenant_class` and avoid equating OCI Always Free profile with any capability level.
#### §3.4.C Tenant-class adoption gaps
- C-001 Tenant-class scan command: `rg -n "tenant_class|demo_trial|revenue_share|per_seat|per-seat|usage-based|billing_components" microservices/notes`.
- C-002 Tenant-class scan result: zero hits.
- C-003 Gap: `notes` does not express `tenant_class` anywhere in PRD, README, manifest, contracts, policies, SLOs, capacity model, cost model, IaC, or runbooks.
- C-004 Gap: `cost-budget.md:72` has a "Free-tier" allowance that should become `demo_trial` with OCI Always Free profile and hard usage caps.
- C-005 Gap: `cost-budget.md:47` uses Team/Business monthly caps but not `paid` per-seat plus usage-based billing semantics.
- C-006 Gap: no document states how `revenue_share` tenants run at-cost or zero-margin substrate while maintaining the same quality bar.
- C-007 Gap: no OpenAPI header, JWT claim, manifest field, Cedar condition, or event envelope carries `tenant_class`.
- C-008 Gap: no capacity overlay separates demo-trial hard caps from paid elastic scaling and revenue-share at-cost scaling.
- C-009 Gap: no SLO overlay states demo-trial best-effort SLO versus paid contractual SLO while preserving the same feature-quality bar.
- C-010 Gap severity: P2 because this is a documentation/control-model adoption gap introduced by the 2026-05-20 directive.
- C-011 Recommended schema: add `tenant_class` to manifest, PRD nonfunctional section, capacity model, cost model, Cedar policy context, OpenAPI headers or JWT claims, and benchmark overlays.
- C-012 Recommended values: `demo_trial`, `paid`, `revenue_share`.
- C-013 Recommended non-goal: do not reintroduce demo_trial/paid/paid/paid compliance_pack under different labels.
- C-014 Recommended OCI language: "OCI Always Free profile" or "demo_trial tenant_class infrastructure".
- C-015 Verdict: tenant-class adoption is absent.
### §3.5 Dimension 5 — Product parity and counterpart coverage
- D5.1 Notion pressure: databases, pages-as-rows, block model, relations, rollups, templates, synced databases, publishing, rich embeds, and collaborative workspace administration.
- D5.2 Evidence: existing competitor matrix lines 21-30 covers capture and edit against Notion and others.
- D5.3 Evidence: existing competitor matrix lines 31-43 covers organization, backlinks, daily notes, templates, graph, and block-level references.
- D5.4 Evidence: existing competitor matrix lines 64-70 covers collaboration and sharing.
- D5.5 Evidence: PRD lines 58-83 includes CRUD, backlinks, templates, web clipper, share links, checklist extraction, search, graph, import/export, collab, and AI assist.
- D5.6 Gap: current PRD intentionally keeps notes note-level and treats block-level references as open/successor work, while Notion is block/database-native.
- D5.7 Evidence: ADR-NOTES-0002 lines 116-119 reject block-level references at minimum-shippable scope.
- D5.8 Evidence: ADR-NTS-001 lines 50-80 later proposes a block tree successor model.
- D5.9 Severity: P2; not a failure if notes remains note-level, but Notion parity needs explicit successor acceptance.
- D5.10 Obsidian pressure: local-first Markdown vaults, backlinks, graph view, plugins, canvas, publish, sync, and file portability.
- D5.11 Evidence: notes has Markdown/frontmatter in PRD lines 18-26 and export/import in ADR-NOTES-0006 lines 47-123.
- D5.12 Evidence: notes has graph architecture in ADR-NOTES-0002 lines 41-49.
- D5.13 Gap: Obsidian's plugin ecosystem is not matched; notes references SDKs and capabilities, but no plugin surface is owned by notes.
- D5.14 Severity: P3 for notes itself because plugin-app-store may own ecosystem integration.
- D5.15 Apple Notes pressure: native capture, locked notes, scanner/signing, attachments, collaboration, Smart Folders/tags, lock-screen instant note, Apple Intelligence writing tools.
- D5.16 Evidence: PRD lines 268-301 includes Apple Notes and names lock/privacy and import/export gaps.
- D5.17 Gap: native OS capture surfaces are declared but not backed by Swift/Kotlin/WinUI3/Leptos implementation files.
- D5.18 Severity: P2 product-to-implementation gap.
- D5.19 Gap: scanner/signing parity is not explicit in PRD functional requirements.
- D5.20 Severity: P3 unless Apple Notes parity is treated as launch-critical.
- D5.21 Positive: notes exceeds Apple Notes on open export and graph ambitions.
- D5.22 Positive: notes exceeds Obsidian on tenant-governed compliance and audit-chain ambitions.
- D5.23 Positive: notes potentially exceeds Notion on E2E refusal and tenant policy integration.
- D5.24 Counterpart verdict: union coverage is directionally strong but implementation and block/native capture gaps remain.
### §3.6 Dimension 6 — Multi-context deployability
- D6.1 Canonical contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
- D6.2 Required paths: `iac/oyatie-public-cloud/`, `iac/guest-on-aws/`, `iac/oci-guest/`, `iac/on-prem/`, `iac/colo/`, `iac/oyatie-iaas/`.
- D6.3 Actual `iac/` directories: `iac/helm`, `iac/helm/notes`, `iac/helm/notes/templates`, `iac/kustomize`, `iac/kustomize/base`, `iac/kustomize/overlays`, `iac/kustomize/overlays/pack-eu`, `iac/kustomize/overlays/pack-kr`.
- D6.4 Missing: `iac/oyatie-public-cloud/`.
- D6.5 Missing: `iac/guest-on-aws/`.
- D6.6 Missing: `iac/oci-guest/`.
- D6.7 Missing: `iac/oci-guest/always-free/`.
- D6.8 Missing: `iac/on-prem/`.
- D6.9 Missing: `iac/colo/`.
- D6.10 Missing: `iac/oyatie-iaas/`.
- D6.11 Evidence: `find microservices/notes/iac -maxdepth 3 -type d | sort` returned only Helm/Kustomize and two pack overlays.
- D6.12 Evidence: IP-001 lines 30-35 target Helm, Kustomize, and a Terraform path, not six context modules.
- D6.13 Evidence: multi-region lines 22-32 define pack placement on OCI regions, but not six deployable contexts.
- D6.14 Evidence: manifest lines 383-394 depends on `cloud-iac`, but does not expose deployment context support.
- D6.15 Severity: P1 because the task assumes all six contexts unless audit evidence narrows otherwise, and current evidence does not substantiate any six-context deployability.
- D6.16 Guest-on-AWS status: missing context module; no correctly documented exception.
- D6.17 Guest-on-OCI status: missing context module; OCI region prose exists but no context module.
- D6.18 On-prem status: missing context module.
- D6.19 Colo status: missing context module.
- D6.20 Oyatie-as-cloud-provider status: missing context module.
- D6.21 Public-cloud status: Kubernetes pack overlays exist but not canonical `oyatie-public-cloud` OpenTofu context module.
- D6.22 Recommended next step: add per-context OpenTofu modules or a formal N/A decision with authority and consequences; for notes, no context appears inherently N/A.
- D6.23 Multi-context verdict: not deployability-coherent today.
### §3.7 Dimension 7 — OpenTofu IaC compliance
- D7.1 Canonical rule: OpenTofu is the only IaC substrate for durable infrastructure.
- D7.2 Canonical required files: context modules need `main.tf`, `variables.tf`, `outputs.tf`, `versions.tf`, and `README.md` per ADR D-16 lines 2275-2299.
- D7.3 Actual files: no `.tf` or `.tofu` files under `microservices/notes`.
- D7.4 Actual files: Helm chart and Kustomize overlays exist.
- D7.5 Actual files: YAML configs exist for OpenBao, ECH, WAF, PQC certs, and secret bindings.
- D7.6 Forbidden language: IP-001 line 20 says Terraform-managed Grafana RBAC.
- D7.7 Forbidden language: IP-001 line 35 targets `iac/terraform/grafana-rbac.tf`.
- D7.8 Forbidden language: IP-001 line 43 invokes `terraform -chdir=microservices/notes/iac/tofu validate`.
- D7.9 Doctrine mismatch: ADR D-16 forbids Terraform binary and Terraform Cloud while allowing `.tf` syntax only as OpenTofu modules.
- D7.10 Severity: P1 because current IaC docs are materially out of canonical compliance.
- D7.11 Positive: Kubernetes charting can remain a deployment artifact, but it must be downstream of OpenTofu context modules or explicitly scoped as app manifest only.
- D7.12 Positive: no Pulumi, CloudFormation, ARM, or Bicep durable infra references were found in the focused scan.
- D7.13 Gap: no module-signing evidence.
- D7.14 Gap: no context-specific state backend evidence.
- D7.15 Gap: no tenant onboarding flow `tofu init -> tofu plan -> tofu apply`.
- D7.16 Gap: no OCI Always Free module.
- D7.17 Recommended next step: create context modules under canonical paths with OpenTofu and leave Helm/Kustomize as Kubernetes release inputs.
- D7.18 OpenTofu verdict: not compliant.
### §3.8 Dimension 8 — OS support and client/runtime support
- D8.1 Canonical OS manifest: `supported-oses.json` is required per ADR D-17 lines 2907-2928.
- D8.2 Actual result: no `supported-oses.json` or OS support file exists under notes.
- D8.3 Canonical primary OSes include Talos, RHEL, Oracle Linux, SLES, Ubuntu, Debian, Rocky, Alma, CentOS Stream, Amazon Linux, Flatcar, Photon, and macOS Apple Silicon M5+.
- D8.4 Canonical test-only architecture lanes include Linux ppc64le and Linux s390x.
- D8.5 Out-of-scope OSes include Intel macOS, M1-M4 macOS, FreeBSD, OpenBSD, Windows Server, and Solaris per ADR D-17.
- D8.6 Current notes docs do not publish an OS support matrix.
- D8.7 Current notes docs mention web, iOS, Android, macOS, Windows/Linux/Tauri in competitor matrix lines 96-104, but this is a product-client note, not canonical OS support.
- D8.8 Current notes docs mention native client technologies in PRD lines 28-44 and SDK plan, but not supported OS manifests.
- D8.9 Severity: P1 because notes claims broad product/deployment maturity but lacks required OS manifest.
- D8.10 Recommended next step: add `supported-oses.json` with server OSes, client OSes, architecture lanes, unsupported exclusions, and validation commands.
- D8.11 Apple Notes counterpart pressure: native capture is OS-bound; notes needs explicit client OS claims to compete credibly.
- D8.12 Obsidian counterpart pressure: desktop/local vault use is OS-bound; notes needs offline client support evidence.
- D8.13 Notion counterpart pressure: web-first collaboration is browser/OS-light; notes still needs server OS manifest because Oyatie deployment contexts require it.
- D8.14 OS verdict: missing required manifest.
### §3.9 Dimension 9 — Rust-strict backend and executable evidence
- D9.1 Rust-strict scan command: `find microservices/notes -type f` with forbidden code extensions.
- D9.2 Rust-strict scan result: no forbidden `.py`, `.js`, `.ts`, `.tsx`, `.rb`, `.go`, `.java`, `.scala`, `.groovy`, `.php`, `.fs`, or `.fsx` files.
- D9.3 Rust implementation scan result: no `.rs` files.
- D9.4 Cargo scan result: no `Cargo.toml` or `Cargo.lock` under notes.
- D9.5 Source scan result: no `src/` directory under notes.
- D9.6 Test scan result: no `tests/` directory under notes.
- D9.7 Positive: current path does not contain forbidden backend language implementation.
- D9.8 Gap: current path also does not contain Rust backend implementation.
- D9.9 Evidence: IP-002 lines 23-25 requires `Cargo.toml` members, 111 crate manifests, and 111 `src/lib.rs` skeletons; none exist.
- D9.10 Evidence: IP-003 lines 51-56 requires `cargo check`, `cargo test`, and governance gates; no crate exists to run those commands against.
- D9.11 Evidence: IP-013 lines 43-55 requires regression tests and governance gates; no tests path exists.
- D9.12 Severity: P1 if maturity claim includes implementation readiness; P2 if this wave only audits documentation.
- D9.13 Frontend caveat: IP-007 line 48 names `npm run test`, and ADR-NOTES-0002 line 84 names `sigma.js` and `graphology-layout-forceatlas2`; these must be reconciled with Leptos/WASM-SSR frontend policy or declared third-party client-only libraries.
- D9.14 SDK caveat: SDK plan line 26 names TypeScript bindings; this must be generator/binding-only or replaced by allowed frontend surfaces.
- D9.15 Rust verdict: no direct forbidden code, but no Rust implementation evidence.
## §4 Findings table
| ID | Sev | Finding | Evidence | Required correction |
|---|---:|---|---|---|
| F-01 | P1 | Six deployable contexts are not evidenced. | `specs/master-plan-sequencing.json:704-745`; actual iac dirs only Helm/Kustomize from `find microservices/notes/iac -maxdepth 3 -type d`. | Add canonical OpenTofu context modules or document authorized exceptions. |
| F-02 | P1 | OpenTofu-only doctrine is violated by Terraform language and missing modules. | `IP-001-iac.md:20`, `IP-001-iac.md:35`, `IP-001-iac.md:43`; ADR D-16 lines 2241-2644. | Replace Terraform binary/path wording with OpenTofu modules and `tofu` validation. |
| F-03 | P1 | OCI Always Free profile module is absent. | Required by ADR D-19 lines 3491-3827; missing `iac/oci-guest/always-free/`. | Add OCI Always Free profile module for `demo_trial` infrastructure or justify non-support. |
| F-04 | P1 | `supported-oses.json` is absent. | ADR D-17 lines 2907-2928; `find microservices/notes -name 'supported-oses.json'` returned no files. | Add OS/arch support manifest and validation matrix. |
| F-05 | P1 | Capacity model contradicts PRD by 25x on creates/sec. | `capacity-model.md:33`; `PRD.md:331`. | Reconcile scale trigger and maximum aggregate throughput. |
| F-06 | P1 | Implementation readiness is not evidenced. | `IP-002-cargo-workspace-bootstrap.md:23-25`; no `src/`, no `Cargo.toml`, no `.rs`. | Land or clearly mark implementation as pending; remove "achieved" wording. |
| F-07 | P2 | Manifest BC/crate shape contradicts PRD/IP-002. | `manifest.json:6-33`; `IP-002-cargo-workspace-bootstrap.md:31-51`; `PRD.md:175`. | Align manifest with 17-BC plan or retire the 111-crate shape. |
| F-08 | P2 | Manifest IP registry is stale. | `manifest.json:121-211`; inventory contains IP-016 through IP-018 and journey IPs. | Update manifest implementation-plan registry. |
| F-09 | P2 | Cross-microservice handoff doc is missing. | No `cross-microservice-handoffs.md`; dependencies at `manifest.json:383-394`; PRD events at `PRD.md:220-247`. | Add handoff document for drive/tasks/identity/tenancy/audit/intelligence/cloud-iac. |
| F-10 | P2 | Tier system remains embedded. | Exact candidates T-001 through T-041 and broader candidates T-043 through T-051. | Wave 15J rewrite; do not author new tier deltas. |
| F-11 | P2 | `tenant_class` is absent. | zero hits for `tenant_class`, `demo_trial`, `revenue_share`; cost gap at `cost-budget.md:72`. | Add `tenant_class` schema and overlays. |
| F-12 | P2 | Current benchmark doc uses retired paid/paid segmentation. | `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:13-30`, `61-69`, `88-98`. | Replace with single target set plus context and tenant-class overlays. |
| F-13 | P2 | Architecture contains repeated content-pass scaffolding. | `ARCHITECTURE.md:62-70`; repeated deployment lines 106, 217, 272, 327, 382, 437, 492, 547, 606, 657, 714, 768, 823, 880, 934, 1000, 1055. | Refactor architecture into concise, non-repeated implementation blueprint. |
| F-14 | P2 | Test surfaces are declared but absent. | PRD `344-360`; IP-013 `43-55`; no `tests/` directory. | Add tests or mark acceptance gates as planned-only. |
| F-15 | P2 | Browser extension and JS library references need language-policy classification. | `IP-007-web-clipper-bridge.md:48`; `ADR-NOTES-0002:84`; D-18 lines 3045-3490. | Move to allowed frontend surface or replace with Rust/WASM/Leptos-compliant plan. |
| F-16 | P2 | Personal/Professional is expressed as "tier" rather than privacy context. | `README.md:18`; `policy/e2e-personal-tier-default.md:29-32`; OpenAPI lines 20-22. | Rename to `context_kind` while preserving privacy invariants. |
| F-17 | P2 | Cost model uses legacy free/team/business tier economics. | `cost-budget.md:47`, `cost-budget.md:72`, `cost-budget.md:87`. | Rewrite to `demo_trial`, `paid`, and `revenue_share` economics. |
| F-18 | P2 | Backfill RTO rows use Business/Enterprise tiers. | `backfill-replay.md:119-122`. | Rewrite by workload size and tenant class. |
| F-19 | P2 | Benchmarks claim achieved results without harness evidence. | benchmark file lines 100-109 references `benchmarks/notesbench/`; no such path in inventory. | Add harness/results or downgrade wording to target. |
| F-20 | P2 | Notion block/database parity is successor-scoped but not clearly launch-scoped. | ADR-NOTES-0002 lines 116-119; ADR-NTS-001 lines 50-80. | Decide note-level launch vs block-level parity in PRD roadmap. |
| F-21 | P3 | Obsidian plugin ecosystem parity is delegated implicitly. | competitor matrix lines 106-113; no plugin integration doc in notes. | Add handoff to plugin-app-store if desired. |
| F-22 | P3 | Apple Notes scanner/signature parity is not explicit. | Apple support features; PRD functional requirements lines 58-83. | Add or explicitly defer scanner/signature capture. |
| F-23 | P2 | Compliance doc repeats `tier product` metadata many times. | `compliance.md:594`, `656`, `718`, `780`, `842`, `904`, `966`, `1028`, `1090`. | Rename to non-tier criticality/product taxonomy. |
| F-24 | P2 | Existing capability files are named T0/T1/T2. | `capabilities/T0-suggest.yaml`, `T1-assist.yaml`, `T2-auto.yaml`; manifest `45-64`, `330-334`. | Decide whether T labels survive as capability phases or are renamed in Wave 15J. |
| F-25 | P2 | Current deployment evidence is pack/region oriented, not context oriented. | `multi-region.md:22-32`; no six context dirs. | Add deployment-context matrix and OpenTofu modules. |
| F-26 | P2 | Audit-chain events are selective and may not match PRD audit expectations. | PRD `115-121`; manifest `310-317`; AsyncAPI `37-48`. | Reconcile Personal-minimal event model with compliance/audit claims. |
| F-27 | P2 | `criticality_tier` may be confused with retired feature tiers. | `manifest.json:396`. | Rename to `criticality_class` or document operational-only meaning. |
| F-28 | P2 | README deployability claim is too broad for actual evidence. | `README.md:45-47`; missing code/context modules. | Qualify GA and cell eligibility until context modules and tests land. |
| F-29 | P2 | IP-001 file targets are not present. | `IP-001-iac.md:30-35`; actual iac file list lacks `iac/terraform` and most pack overlays. | Update plan or land files. |
| F-30 | P2 | Chat history confirms notes was dispatched as part of rolling audit, not necessarily completed. | chat history line 16311 and line 16405. | Treat this audit as the authoritative completion artifact for notes. |
## §5 Open questions
- OQ-01 Should `Personal` and `Professional` remain as privacy `context_kind` values while all "tier" wording is retired? Proposed answer: yes, but rename all "Personal-tier" and "Professional-tier" labels.
- OQ-02 Should T0/T1/T2 capability names survive as non-commercial capability phases? Proposed answer: only if Wave 15J explicitly allows phase labels that do not imply feature-quality tiers.
- OQ-03 Should notes pursue Notion-like block/database parity at launch or as successor scope? Proposed answer: successor scope unless the product promise requires Notion database parity.
- OQ-04 Should scanner/signature capture be added for Apple Notes parity? Proposed answer: add to client roadmap if Apple Notes remains top-three benchmark.
- OQ-05 Should web clipper browser extensions use a non-Rust surface? Proposed answer: require explicit frontend/platform exception or Rust/WASM implementation plan.
- OQ-06 Should `demo_trial` tenants support all notes features with caps, or should AI be disabled for cost? Proposed answer: uniform quality bar says features are not tier-stratified, but usage caps can restrict volume.
- OQ-07 Should `revenue_share` tenants receive at-cost dedicated substrate? Proposed answer: define in cost model with no feature degradation.
- OQ-08 Should OCI Always Free profile support notes at all? Proposed answer: yes for `demo_trial`, but with hard storage/throughput/concurrent-operation caps.
- OQ-09 Should notes claim all six contexts now? Proposed answer: no; current evidence should state six-context support is planned, not landed.
- OQ-10 Should Kubernetes Helm/Kustomize remain in notes? Proposed answer: yes as release manifests, but not as the canonical infrastructure substrate.
- OQ-11 Should `cross-microservice-handoffs.md` be mandatory before implementation? Proposed answer: yes because notes depends heavily on drive, tasks, intelligence, identity, tenancy, audit-chain, cloud-iac, and observability.
- OQ-12 Should the capacity contradiction resolve toward 20k creates/sec or 500k creates/sec? Proposed answer: start from capacity model 20k unless a benchmark harness proves higher aggregate throughput.
- OQ-13 Should `supported-oses.json` include client OSes as well as server OSes? Proposed answer: include both, with separate server/client validation scopes.
- OQ-14 Should local-first Obsidian parity be implemented through a native client vault or through web cache? Proposed answer: native/offline client plan must be explicit.
- OQ-15 Should Apple Advanced Data Protection style recovery loss be mirrored? Proposed answer: yes; policy docs already accept permanent data destruction on lost recovery seed.
- OQ-16 Should AI assist ever touch E2E content? Proposed answer: no; ADR-NOTES-0005 and policy docs correctly refuse this.
- OQ-17 Should `benchmarks/notesbench/` be created before benchmark claims are published? Proposed answer: yes.
- OQ-18 Should line-count-heavy architecture expansion be retained? Proposed answer: no; replace with executable architecture.
- OQ-19 Should the existing `tenant-class/` directory be deleted by this audit? Proposed answer: no; this audit only flags it for Wave 15J retirement.
- OQ-20 Should current `README.md` say Product GA? Proposed answer: not until deployment context modules, OS manifest, Rust implementation skeleton, tests, and benchmark harness evidence land.
## §6 Evidence ledger
- E-001 Product purpose: `PRD.md:18-26`.
- E-002 Product differentiation: `PRD.md:28-44`.
- E-003 Functional requirements: `PRD.md:58-83`.
- E-004 Performance requirements: `PRD.md:87-101`.
- E-005 Security requirements: `PRD.md:103-113`.
- E-006 Audit and compliance requirements: `PRD.md:115-121`.
- E-007 Availability requirements: `PRD.md:123-127`.
- E-008 Bounded context and crate claim: `PRD.md:140-175`.
- E-009 Workflow events: `PRD.md:220-247`.
- E-010 Counterpart benchmark surface: `PRD.md:268-301`.
- E-011 Capacity contradiction first side: `capacity-model.md:33`.
- E-012 Capacity contradiction second side: `PRD.md:331`.
- E-013 Acceptance tests declared: `PRD.md:344-360`.
- E-014 README product summary: `README.md:16-18`.
- E-015 README GA/cell claim: `README.md:45-47`.
- E-016 Manifest narrow BC list: `manifest.json:6-29`.
- E-017 Manifest layers: `manifest.json:30-33`.
- E-018 Manifest capability tier fields: `manifest.json:45-64`.
- E-019 Manifest IP registry: `manifest.json:121-211`.
- E-020 Manifest dependencies: `manifest.json:383-394`.
- E-021 Manifest criticality tier: `manifest.json:396`.
- E-022 OpenAPI version: `contracts/openapi/notes.yaml:1`.
- E-023 OpenAPI context kind: `contracts/openapi/notes.yaml:65-72`.
- E-024 OpenAPI Personal/Professional description: `contracts/openapi/notes.yaml:20-22`.
- E-025 OpenAPI CRUD/search/share/import/export endpoints: `contracts/openapi/notes.yaml:316-502`.
- E-026 AsyncAPI version: `contracts/asyncapi/notes-events.yaml:1`.
- E-027 AsyncAPI event minimization: `contracts/asyncapi/notes-events.yaml:5-8`.
- E-028 AsyncAPI channel event list: `contracts/asyncapi/notes-events.yaml:23-48`.
- E-029 Proto syntax: `contracts/proto/notes.proto:5`.
- E-030 Proto Loro Professional-only boundary: `contracts/proto/notes.proto:11-18`.
- E-031 Proto MLS Personal path: `contracts/proto/notes.proto:57-87`.
- E-032 SLO note open target: `slos/note-open-latency.openslo.yaml:5-43`.
- E-033 SLO note create target: `slos/note-create-latency.openslo.yaml:5-42`.
- E-034 SLO sync target: `slos/sync-latency.openslo.yaml:5-42`.
- E-035 SLO graph target: `slos/graph-render-latency.openslo.yaml:5-42`.
- E-036 SLO full-text search target: `slos/full-text-search-latency.openslo.yaml:5-42`.
- E-037 SLO tag search target: `slos/tag-search-latency.openslo.yaml:5-41`.
- E-038 SLO web clipper target: `slos/web-clipper-capture-latency.openslo.yaml:5-41`.
- E-039 E2E privacy correctness target: `slos/e2e-privacy-correctness.openslo.yaml:48-53`.
- E-040 Sync data residency correctness: `slos/sync-data-residency-correctness.openslo.yaml:39-44`.
- E-041 IP-001 Terraform contradiction: `IP-001-iac.md:20`.
- E-042 IP-001 OpenTofu/Terraform boundary: `IP-001-iac.md:24`.
- E-043 IP-001 missing target path: `IP-001-iac.md:35`.
- E-044 IP-001 forbidden command: `IP-001-iac.md:43`.
- E-045 IP-002 crate claim: `IP-002-cargo-workspace-bootstrap.md:19`.
- E-046 IP-002 source targets: `IP-002-cargo-workspace-bootstrap.md:23-25`.
- E-047 IP-002 crate inventory: `IP-002-cargo-workspace-bootstrap.md:31-51`.
- E-048 Capability tier matrix title: `tenant-class/tier-matrix.md:9`.
- E-049 Capability tier matrix demo_trial: `tenant-class/tier-matrix.md:13`.
- E-050 Capability tier matrix paid: `tenant-class/tier-matrix.md:48`.
- E-051 Capability tier matrix paid: `tenant-class/tier-matrix.md:82`.
- E-052 Capability tier matrix paid compliance_pack: `tenant-class/tier-matrix.md:116`.
- E-053 Capability tier matrix promotion path: `tenant-class/tier-matrix.md:142-146`.
- E-054 Benchmark old paid hardware: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:13`.
- E-055 Benchmark old paid/paid target: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:30`.
- E-056 Benchmark harness claim: `benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:100-109`.
- E-057 Onboarding paid reference: `onboarding/notes-engineer-first-week.md:33`.
- E-058 Tutorial paid reference: `tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:15`.
- E-059 Tutorial paid reference: `tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:181`.
- E-060 FAQ demo_trial reference: `faqs/notes-engineer-faq.md:30`.
- E-061 FAQ paid/paid reference: `faqs/notes-engineer-faq.md:58`.
- E-062 Reference implementation paid compliance_pack reference: `reference-implementations/block-edit-and-link-rust-sdk.md:250`.
- E-063 Backfill Business/Enterprise tier rows: `backfill-replay.md:119-122`.
- E-064 Cost free-tier: `cost-budget.md:72`.
- E-065 Cost tier allowance: `cost-budget.md:87`.
- E-066 Multi-region OCI pack table: `multi-region.md:22-32`.
- E-067 Incident response Sev-1 privacy: `incident-response.md:22-25`.
- E-068 DPIA processing and Personal/Professional split: `dpia.md:36-43`.
- E-069 DPIA legal basis: `dpia.md:82-84`.
- E-070 DPIA E2E controls: `dpia.md:132-140`.
- E-071 Policy E2E tier table: `policy/e2e-personal-tier-default.md:29-32`.
- E-072 Policy Personal audit minimization: `policy/e2e-personal-tier-default.md:100-110`.
- E-073 Policy Professional posture: `policy/e2e-personal-tier-default.md:122-135`.
- E-074 Cedar public read Personal forbid: `policy/public-read.cedar:46-47`.
- E-075 Cedar tenant scope AI/collab Personal forbids: `policy/tenant-scope.cedar:230-247`.
- E-076 Minor protection tier field: `policy/minor-protection.cedar:12-13`.
- E-077 Architecture repeated expansion: `ARCHITECTURE.md:62-70`.
- E-078 Architecture repeated deployment evidence: `ARCHITECTURE.md:106`.
- E-079 Existing competitor matrix capture/edit: `competitor-parity-matrix.md:21-30`.
- E-080 Existing competitor matrix privacy: `competitor-parity-matrix.md:45-53`.
- E-081 Existing competitor matrix collaboration: `competitor-parity-matrix.md:64-70`.
- E-082 Existing competitor matrix AI: `competitor-parity-matrix.md:86-94`.
- E-083 Existing competitor matrix mobile/desktop: `competitor-parity-matrix.md:96-104`.
- E-084 ADR-NOTES-0002 graph privacy: `decisions/ADR-NOTES-0002-bidirectional-link-and-graph-storage.md:41-49`.
- E-085 ADR-NOTES-0002 block reference deferral: `decisions/ADR-NOTES-0002-bidirectional-link-and-graph-storage.md:116-119`.
- E-086 ADR-NOTES-0003 Loro decision: `decisions/ADR-NOTES-0003-crdt-library-for-optional-collab.md:45-55`.
- E-087 ADR-NOTES-0004 search split: `decisions/ADR-NOTES-0004-search-architecture-respecting-e2e.md:61-87`.
- E-088 ADR-NOTES-0005 AI refusal: `decisions/ADR-NOTES-0005-ai-assist-bounds-and-e2e-invariant.md:43-77`.
- E-089 ADR-NOTES-0006 import/export decision: `decisions/ADR-NOTES-0006-portable-export-and-import-format.md:47-123`.
- E-090 ADR-NTS-001 successor block model: `decisions/ADR-NTS-001-block-based-data-model-with-bidirectional-links.md:50-80`.
- E-091 Canonical six contexts: `specs/master-plan-sequencing.json:704-745`.
- E-092 Canonical OpenTofu: `specs/master-plan-sequencing.json:747-775`.
- E-093 Canonical OS matrix: `specs/master-plan-sequencing.json:777-815`.
- E-094 Canonical language policy: `specs/master-plan-sequencing.json:817-856`.
- E-095 Canonical OCI Always Free profile: `specs/master-plan-sequencing.json:857-868`.
- E-096 Canonical no-tier directive: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_2026_05_20.md:10-44`.
- E-097 Canonical ownership directive: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-59`.
- E-098 Canonical deliverable substance directive: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53`.
- E-099 Canonical no-scaffold directive: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-18`.
- E-100 Chat queue counterpart line: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.
## §7 Conclusion
- CON-01 The notes product definition is one of the stronger µservice product surfaces in this wave.
- CON-02 The privacy model is distinctive and internally meaningful: Personal content is client-side/E2E, Professional content can use tenant-DEK and server-side search/AI under controls.
- CON-03 The counterpart selection is correct and already reflected in the repo and chat history.
- CON-04 The documentation set is broad, but it overstates implementation readiness.
- CON-05 The largest blocker is not product clarity; it is canonical deployability.
- CON-06 Six-context deployment evidence is missing.
- CON-07 OpenTofu context modules are missing.
- CON-08 OCI Always Free profile evidence is missing.
- CON-09 OS support manifest evidence is missing.
- CON-10 Rust implementation and test evidence are missing.
- CON-11 The old tier system remains embedded across the notes corpus.
- CON-12 Tenant-class semantics are absent.
- CON-13 The retired fourth tier-deltas deliverable should not be authored.
- CON-14 The next durable cleanup should retire `tenant-class/tier-matrix.md`, rewrite benchmark/tutorial/cost references, and add tenant-class overlays.
- CON-15 The next deployability cleanup should add OpenTofu context modules and `supported-oses.json`.
- CON-16 The next implementation cleanup should land the IP-002 crate skeleton or explicitly mark the implementation plan as pending-only.
- CON-17 The next architecture cleanup should replace repeated content-pass expansion with a concise implementation blueprint.
- CON-18 Final audit verdict: product-coherent, documentation-broad, deployment-not-coherent, implementation-not-evidenced, tier-retirement-required.

<!-- ORCHESTRATOR REPORT
  µservice: notes
  deliverables_landed:
    - /Users/jasonlee/oyatie/microservices/notes/coherence-audit-2026-05-20.md (724 lines)
    - /Users/jasonlee/oyatie/microservices/notes/feature-parity-matrix-2026-05-20.md (415 lines)
    - /Users/jasonlee/oyatie/microservices/notes/performance-benchmark-numbers-2026-05-20.md (313 lines)
  inventory_files_seen: 160
  inventory_lines_read: 26395
  chat_history_matches_processed: 343
  findings_p0: 0
  findings_p1: 6
  findings_p2: 22
  findings_p3: 2
  tier_retirement_candidates_found: 41 exact color-metal feature-tier candidates:
    - microservices/notes/onboarding/notes-engineer-first-week.md:33
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:66
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:68
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:72
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:81
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:82
    - microservices/notes/migration-playbooks/from-notion-and-roam-and-obsidian.md:83
    - microservices/notes/tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:15
    - microservices/notes/tutorials/build-research-notebook-with-bidirectional-links-and-ai-summary.md:181
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:13
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:21
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:28
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:30
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:36
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:43
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:61
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:62
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:69
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:88
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:89
    - microservices/notes/benchmarks/notes-vs-notion-vs-roam-vs-obsidian-vs-evernote.md:98
    - microservices/notes/tenant-class/tier-matrix.md:13
    - microservices/notes/tenant-class/tier-matrix.md:30
    - microservices/notes/tenant-class/tier-matrix.md:48
    - microservices/notes/tenant-class/tier-matrix.md:50
    - microservices/notes/tenant-class/tier-matrix.md:82
    - microservices/notes/tenant-class/tier-matrix.md:84
    - microservices/notes/tenant-class/tier-matrix.md:112
    - microservices/notes/tenant-class/tier-matrix.md:116
    - microservices/notes/tenant-class/tier-matrix.md:118
    - microservices/notes/tenant-class/tier-matrix.md:130
    - microservices/notes/tenant-class/tier-matrix.md:132
    - microservices/notes/tenant-class/tier-matrix.md:144
    - microservices/notes/reference-implementations/block-edit-and-link-rust-sdk.md:250
    - microservices/notes/faqs/notes-engineer-faq.md:30
    - microservices/notes/faqs/notes-engineer-faq.md:58
    - microservices/notes/faqs/notes-engineer-faq.md:77
    - microservices/notes/faqs/notes-engineer-faq.md:80
    - microservices/notes/faqs/notes-engineer-faq.md:118
    - microservices/notes/faqs/notes-engineer-faq.md:134
    - microservices/notes/faqs/notes-engineer-faq.md:143
  tenant_class_adoption_gaps: yes; no tenant_class, demo_trial, paid/revenue_share replacement schema, or tenant-class quota/SLO/cost overlay appears in the notes path.
  top_3_counterparts_confirmed: Notion / Obsidian / Apple Notes
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1452
-->
