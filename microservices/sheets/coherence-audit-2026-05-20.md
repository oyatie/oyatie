---
doc_class: OwnershipCoherenceAudit
microservice: sheets
audit_date: 2026-05-20
auditor: Codex
batch: Wave 3 Batch 3.2
status: complete
deliverables_expected: 3
deliverables_authored:
  - microservices/sheets/coherence-audit-2026-05-20.md
  - microservices/sheets/feature-parity-matrix-2026-05-20.md
  - microservices/sheets/performance-benchmark-numbers-2026-05-20.md
tier_delta_deliverable: retired_per_2026_05_20_directive
---

# sheets ownership-coherence audit - 2026-05-20

## Header

- Target microservice: `sheets`.
- Target path: `microservices/sheets/`.
- Audit ownership model: one owner, one microservice, no cross-agent coordination.
- Read-only phase scope: complete recursive inventory plus canonical-direction sources, constraint memory files, and relevant chat history.
- Write phase scope: three deliverables only; the prior capability-tier delta deliverable is retired.
- Deployment-context expectation: all six contexts unless a service-specific artifact proves a justified exception.
- Six canonical contexts: `oyatie-public-cloud`, `guest-on-aws`, `guest-on-oci`, `on-prem`, `colo`, `oyatie-as-cloud-provider`.
- Canonical IaC substrate: OpenTofu only, with per-context modules under the context-specific directories.
- Canonical language policy: Rust backend; Swift, Kotlin, WinUI 3, and Leptos web frontends where applicable.
- Canonical web posture: Leptos/WASM SSR with selective island hydration for web product surfaces.
- Canonical OS posture: service-owned `supported-oses.json` aligned to the 13+2+6 support matrix.
- Canonical tenant model in the user prompt: `demo_trial`, `paid`, `revenue_share`.
- Constraint-memory caveat: `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-20` narrows canonical tenant class to `demo_trial` and `paid`, with `revenue_share` as a paid billing component.
- Audit treatment for that caveat: the service currently expresses neither model, so the adoption gap holds under both interpretations.
- Counterpart bar: Google Sheets, Microsoft Excel Online, Airtable.
- Chat-history confirmation: line 16311 in `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl` lists `sheets` with Google Sheets, Microsoft Excel Online, and Airtable.
- Existing service purpose: tenant-facing spreadsheet plus structured-data authoring surface, not merely an internal substrate.
- Existing local artifact strength: broad PRD, architecture, contracts, SLOs, runbooks, policy, DPIA, compliance, and capability docs are present.
- Existing local artifact weakness: canonical multi-context, OpenTofu, OS, and tenant-class artifacts are missing or stale.
- Substantive evidence rule: every finding below cites a local file, canonical source, memory file, chat-history line, or official public counterpart source.
- No code implementation was changed.
- No commits were made.

## Source anchors

- ADR-0328 D-15 deployment matrix: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2235`.
- ADR-0328 D-16 OpenTofu-only substrate: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:2241-2400`.
- ADR-0328 D-20 OS and language amendments: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:3950-4070`.
- ADR-0328 D-20 audit stop condition: `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:4202-4226`.
- Master plan deployment contexts: `specs/master-plan-sequencing.json:704-746`.
- Master plan OpenTofu substrate: `specs/master-plan-sequencing.json:747-775`.
- Master plan OS support matrix: `specs/master-plan-sequencing.json:777-815`.
- Master plan language policy: `specs/master-plan-sequencing.json:817-855`.
- Master plan OCI Always Free profile: `specs/master-plan-sequencing.json:857-868`.
- Brief-template line-floor and substance rules: `docs/standards/brief-template.md:169-187` and `docs/standards/brief-template.md:228-239`.
- Brief-template audit anchors: `docs/standards/brief-template.md:407-442`.
- Multi-context memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_multi_context_provider_agnostic_2026_05_20.md:10-38`.
- OpenTofu-only memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_zero_handroll_opentofu_only_2026_05_20.md:10-35`.
- OS support memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:10-76`.
- Rust-strict memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-18`.
- OCI Always Free memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_oci_always_free_maximization_2026_05_20.md:10-72`.
- No-tenant-class-adoption memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_no_automation_risk_classes_2026_05_20.md:10-45`.
- Tenant-class memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20.md:10-142`.
- Ownership-coherence memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_microservice_ownership_coherence_2026_05_20.md:10-84`.
- Deliverable verification memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53`.
- Substance memory: `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_docs_substance_not_scaffold_2026_05_20.md:10-20`.

## §1 Purpose

- This audit determines whether `sheets` is coherent as a microservice ownership unit under the current Oyatie direction.
- The target result is not a new implementation plan.
- The target result is a cited ownership audit with concrete gaps and retired-vocabulary cleanup candidates.
- The audit uses the service's actual artifacts, not desired future state.
- The product purpose is clear in the PRD: `sheets` is the spreadsheet and structured-data authoring hero product.
- The PRD says Sheets owns the cell-grid editor, workbook model, formula engine, recalc engine, charts, pivots, validation, collaborative editing, import/export, AI formula help, and connected sheets at `microservices/sheets/PRD.md:29-35`.
- The PRD also says Sheets is not merely substrate at `microservices/sheets/PRD.md:31`.
- The same paragraph then says the cell-grid, formula-engine, and recalc-engine are shared substrate consumed by other products at `microservices/sheets/PRD.md:35`.
- That dual identity is workable only if the service clearly separates product-shell ownership from reusable kernel ownership.
- The current PRD makes that distinction in prose but does not carry it consistently into manifest, IaC, OS, tenant-class, or benchmark artifacts.
- The audit therefore treats product purpose as present but ownership coherence as incomplete.
- The service has a strong local feature ambition.
- The service has broad local documentation.
- The service has contracts across REST, events, and gRPC.
- The service has SLO files for latency, correctness, merge safety, and export.
- The service has Cedar policy files and extensive threat/compliance/privacy documentation.
- The service lacks the canonical Wave-3 cross-cutting control surfaces.
- The missing controls matter because the service is user-facing, collaboration-heavy, compliance-sensitive, and performance-sensitive.
- Google Sheets, Microsoft Excel Online, and Airtable define the union-coverage bar for this audit.
- Google and Microsoft define the spreadsheet core, formula, import/export, and collaboration bar.
- Airtable defines the typed-column, database-grid, view, automation, and interface-design bar.
- Oyatie's differentiating bar should be uniform industry-leader grade across deployment contexts and tenant classes.
- The retired tier system cannot be used as the quality model for gaps, benchmarks, or rollout expectations.
- Existing tier-language inside `sheets` is therefore evidence of retirement debt, not a model to extend.
- The audit stop condition is satisfied when three deliverables are landed, line floors are verified, tier candidates are cataloged, and canonical constraints are evaluated.

## §2 Inventory

- Inventory method: `rg --files microservices/sheets | sort`.
- Inventory count: 126 files.
- Inventory line count observed by `wc -l`: 21,560 lines.
- Forbidden source-extension search: `rg --files microservices/sheets | rg '\.(py|js|ts|rb|go|java|scala|groovy|php|fs|fsx)$'`.
- Forbidden source-extension result: no files matched.
- `src/` directory result: no `src/` files were present under the service path.
- `tests/` directory result: no `tests/` files were present under the service path.
- Service-owned `supported-oses.json` result: absent.
- Context-specific OpenTofu directory result: absent for all six expected context directories.
- OCI Always Free profile directory result: absent at `microservices/sheets/iac/oci-guest/always-free/`.
- Files inventoried:
- `microservices/sheets/ARCHITECTURE.md`
- `microservices/sheets/AUDIT-FINDINGS-2026-05-18.json`
- `microservices/sheets/IP-001-iac-bootstrap.md`
- `microservices/sheets/IP-002-cargo-workspace-cell-grid-kernel-domain.md`
- `microservices/sheets/IP-003-formula-engine-kernel-domain-400-functions.md`
- `microservices/sheets/IP-004-recalc-engine-dep-graph-parallel.md`
- `microservices/sheets/IP-005-collab-crdt-loro-aligned-ws-0001.md`
- `microservices/sheets/IP-006-large-sheet-storage-postgres-arrow-parquet-hybrid.md`
- `microservices/sheets/IP-007-cell-grid-adapter-postgres-and-materialized-views.md`
- `microservices/sheets/IP-008-formatting-pivot-charts-data-validation.md`
- `microservices/sheets/IP-009-import-export-xlsx-calamine-rust-xlsxwriter-sandboxed.md`
- `microservices/sheets/IP-010-sharing-acl-named-range-cedar.md`
- `microservices/sheets/IP-011-ai-formula-smart-fill-foundry-runtime-bridge.md`
- `microservices/sheets/IP-012-connected-sheets-comments-version-history-trigger-embed-bridge.md`
- `microservices/sheets/IP-013-cell-grid-rest-leptos-wasm-app-license-gate.md`
- `microservices/sheets/IP-014-observability-slo-manifests-9-openslo.md`
- `microservices/sheets/IP-015-hg-sheets-registration-and-branch-protection.md`
- `microservices/sheets/IP-journey-j100-pack-rollout-first-action.md`
- `microservices/sheets/IP-journey-j91-us-msb-mtl-overlay.md`
- `microservices/sheets/IP-journey-j92-br-lgpd-us-parent-dsar.md`
- `microservices/sheets/IP-journey-j93-in-dpdpa-rbi-overlay.md`
- `microservices/sheets/IP-journey-j94-sox404-public-company-controls.md`
- `microservices/sheets/IP-journey-j95-iso27001-soc2-annual-audit.md`
- `microservices/sheets/IP-journey-j96-ksa-uae-mena-onboarding.md`
- `microservices/sheets/IP-journey-j97-sg-pdpa-mas-tenant.md`
- `microservices/sheets/IP-journey-j98-au-privacy-apra-cps234.md`
- `microservices/sheets/IP-journey-j99-multi-pack-conflict-resolution.md`
- `microservices/sheets/PHASE-01-SHEETS-FOUNDATION.md`
- `microservices/sheets/PRD.md`
- `microservices/sheets/backfill-replay.md`
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md`
- `microservices/sheets/capabilities/T0-suggest.yaml`
- `microservices/sheets/capabilities/T1-assist.yaml`
- `microservices/sheets/capabilities/T2-auto.yaml`
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md`
- `microservices/sheets/capacity-model.md`
- `microservices/sheets/catalog/oya-sheets-cell-grid-adapter-leptos-wasm.yaml`
- `microservices/sheets/catalog/oya-sheets-cell-grid-adapter-postgres.yaml`
- `microservices/sheets/catalog/oya-sheets-cell-grid-domain.yaml`
- `microservices/sheets/catalog/oya-sheets-cell-grid-kernel.yaml`
- `microservices/sheets/catalog/oya-sheets-cell-grid-rest.yaml`
- `microservices/sheets/catalog/oya-sheets-collab-crdt-adapter-loro.yaml`
- `microservices/sheets/catalog/oya-sheets-collab-crdt-adapter-valkey.yaml`
- `microservices/sheets/catalog/oya-sheets-collab-crdt-worker.yaml`
- `microservices/sheets/catalog/oya-sheets-formula-engine-domain.yaml`
- `microservices/sheets/catalog/oya-sheets-import-export-adapter-calamine.yaml`
- `microservices/sheets/catalog/oya-sheets-import-export-adapter-clamav.yaml`
- `microservices/sheets/catalog/oya-sheets-import-export-adapter-opswat.yaml`
- `microservices/sheets/catalog/oya-sheets-import-export-adapter-rust-xlsxwriter.yaml`
- `microservices/sheets/catalog/oya-sheets-import-export-worker.yaml`
- `microservices/sheets/catalog/oya-sheets-large-sheet-storage-adapter-arrow.yaml`
- `microservices/sheets/catalog/oya-sheets-large-sheet-storage-adapter-parquet.yaml`
- `microservices/sheets/catalog/oya-sheets-large-sheet-storage-adapter-s3.yaml`
- `microservices/sheets/catalog/oya-sheets-license-gate-cedar-adapter-postgres.yaml`
- `microservices/sheets/catalog/oya-sheets-recalc-engine-worker.yaml`
- `microservices/sheets/catalog/oya-sheets-sharing-acl-adapter-postgres.yaml`
- `microservices/sheets/competitor-parity-matrix.md`
- `microservices/sheets/compliance.md`
- `microservices/sheets/contracts/asyncapi/sheets-events.yaml`
- `microservices/sheets/contracts/openapi/sheets.yaml`
- `microservices/sheets/contracts/proto/sheets.proto`
- `microservices/sheets/cost-budget.md`
- `microservices/sheets/dashboards/collab-and-fanout.json`
- `microservices/sheets/dashboards/editor-experience.json`
- `microservices/sheets/dashboards/recalc-engine-health.json`
- `microservices/sheets/decisions/ADR-SHE-001-spreadsheet-formula-engine-with-incremental-recalc.md`
- `microservices/sheets/decisions/ADR-SHEETS-0001-crdt-library-selection.md`
- `microservices/sheets/decisions/ADR-SHEETS-0002-formula-engine-conformance-target.md`
- `microservices/sheets/decisions/ADR-SHEETS-0003-large-sheet-storage-substrate.md`
- `microservices/sheets/decisions/ADR-SHEETS-0004-recalc-engine-architecture.md`
- `microservices/sheets/decisions/ADR-SHEETS-0005-ai-formula-and-smart-fill-bounds.md`
- `microservices/sheets/decisions/ADR-SHEETS-0006-per-range-acl-granularity.md`
- `microservices/sheets/decisions/ADR-SHEETS-0007-export-fidelity-policy.md`
- `microservices/sheets/decisions/README.md`
- `microservices/sheets/dpia.md`
- `microservices/sheets/failure-modes.md`
- `microservices/sheets/faqs/sheets-engineer-faq.md`
- `microservices/sheets/iac/helm/visual-grid-rest/Chart.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/deployment.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/hpa.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/networkpolicy.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/pdb.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/prometheusrule.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/service.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/templates/servicemonitor.yaml`
- `microservices/sheets/iac/helm/visual-grid-rest/values.yaml`
- `microservices/sheets/iac/kustomize/base/cdn-edge-config.yaml`
- `microservices/sheets/iac/kustomize/base/gvisor-runtime-class.yaml`
- `microservices/sheets/iac/kustomize/base/kustomization.yaml`
- `microservices/sheets/iac/kustomize/base/namespace.yaml`
- `microservices/sheets/iac/kustomize/base/openbao-secret-references.yaml`
- `microservices/sheets/iac/kustomize/base/service-mesh-tenant-headers.yaml`
- `microservices/sheets/iac/kustomize/overlays/pack-eu/kustomization.yaml`
- `microservices/sheets/iac/kustomize/overlays/pack-kr/kustomization.yaml`
- `microservices/sheets/incident-response.md`
- `microservices/sheets/manifest.json`
- `microservices/sheets/migration-playbooks/from-google-sheets-and-airtable.md`
- `microservices/sheets/multi-region.md`
- `microservices/sheets/onboarding/sheets-engineer-first-week.md`
- `microservices/sheets/policy/auditor-scope.cedar`
- `microservices/sheets/policy/ci-scope.cedar`
- `microservices/sheets/policy/data-residency.md`
- `microservices/sheets/policy/editor-isolation.md`
- `microservices/sheets/policy/public-read.cedar`
- `microservices/sheets/policy/tenant-scope.cedar`
- `microservices/sheets/reference-implementations/cell-edit-and-formula-rust-sdk.md`
- `microservices/sheets/runbooks/chart-render-degraded.md`
- `microservices/sheets/runbooks/collab-conflict-resolution-crdt.md`
- `microservices/sheets/runbooks/export-pipeline-failure-xlsx.md`
- `microservices/sheets/runbooks/formula-engine-rollback.md`
- `microservices/sheets/runbooks/named-range-corruption.md`
- `microservices/sheets/runbooks/recalc-storm-throttle.md`
- `microservices/sheets/runbooks/share-acl-drift.md`
- `microservices/sheets/scorecards/overrides.json`
- `microservices/sheets/sdk-plan.md`
- `microservices/sheets/slos/cell-edit-render-latency.openslo.yaml`
- `microservices/sheets/slos/chart-render-latency.openslo.yaml`
- `microservices/sheets/slos/collab-cursor-sync-latency.openslo.yaml`
- `microservices/sheets/slos/crdt-merge-no-silent-loss.openslo.yaml`
- `microservices/sheets/slos/export-xlsx-latency.openslo.yaml`
- `microservices/sheets/slos/formula-engine-correctness.openslo.yaml`
- `microservices/sheets/slos/recalc-100k-cells-latency.openslo.yaml`
- `microservices/sheets/slos/recalc-1m-cells-latency.openslo.yaml`
- `microservices/sheets/slos/sheet-open-latency.openslo.yaml`
- `microservices/sheets/threat-model.md`
- `microservices/sheets/tutorials/build-100k-cell-financial-model.md`

## §3 9-dimension audit

### §3.1 Product purpose and ownership boundary

- Finding: product purpose is mostly coherent.
- Evidence: `microservices/sheets/PRD.md:29-35` defines a spreadsheet and structured-data authoring product surface.
- Evidence: `microservices/sheets/PRD.md:52-76` lists 23 functional requirements spanning grid entry, formulas, import/export, collaboration, pivots, charts, validation, sharing, AI, connected data, triggers, and embed bridges.
- Evidence: `microservices/sheets/PRD.md:31` says Sheets is not a substrate.
- Evidence: `microservices/sheets/PRD.md:35` says cell-grid, formula-engine, and recalc-engine are shared substrate consumed by other products.
- Assessment: the service can be both product and shared kernel owner, but the split must be formalized.
- Assessment: the current manifest does not encode that split; it exposes one bounded context named `sheets` at `microservices/sheets/manifest.json:6-33`.
- Assessment: the architecture file also labels the service as `tier product` at `microservices/sheets/ARCHITECTURE.md:22-24`, which is stale terminology after tier retirement.
- Risk: downstream services may import or depend on product-shell crates instead of SDK/kernel boundaries.
- Mitigation already stated: PRD requires SDK-only cross-product flow at `microservices/sheets/PRD.md:346-359`.
- Gap: no machine-readable boundary divides product shell, reusable grid kernel, formula engine, and recalc substrate.
- Gap: no `deployment_contexts` key states whether product shell and reusable kernels deploy together in all contexts.
- Gap: no `tenant_class` key states how demo/trial caps apply without feature-quality degradation.
- Counterpart implication: Google Sheets and Excel Online own both document UX and formula/recalc semantics; Airtable owns typed-grid semantics and API behavior.
- Oyatie implication: Sheets should own the end-user editor and the shared grid semantics, but it must publish the reuse boundary.

### §3.2 Artifact completeness and internal traceability

- Finding: artifact breadth is high.
- Evidence: the service path contains 126 files and 21,560 lines by `rg --files` plus `wc -l`.
- Evidence: top-level PRD has 597 lines; architecture has 877 lines; compliance has 1,213 lines; threat model has 778 lines.
- Evidence: contracts exist for OpenAPI, AsyncAPI, and proto at `microservices/sheets/contracts/openapi/sheets.yaml`, `microservices/sheets/contracts/asyncapi/sheets-events.yaml`, and `microservices/sheets/contracts/proto/sheets.proto`.
- Evidence: OpenAPI version is 3.2.0 at `microservices/sheets/contracts/openapi/sheets.yaml:1`.
- Evidence: AsyncAPI version is 3.1.0 at `microservices/sheets/contracts/asyncapi/sheets-events.yaml:1`.
- Evidence: proto uses proto3 at `microservices/sheets/contracts/proto/sheets.proto:10`.
- Evidence: nine OpenSLO files are present under `microservices/sheets/slos/`.
- Evidence: seven runbooks are present under `microservices/sheets/runbooks/`.
- Evidence: policy files include Cedar and data-residency/editor-isolation prose under `microservices/sheets/policy/`.
- Evidence: DPIA, compliance, incident response, failure modes, capacity model, cost budget, multi-region, migration, onboarding, FAQ, tutorial, and reference implementation docs exist.
- Positive assessment: a cold reader can learn the intended product shape from the PRD, competitor matrix, capacity model, failure modes, and threat model.
- Negative assessment: the service lacks the control-surface files required by newer canonical direction.
- Missing file: `microservices/sheets/supported-oses.json`.
- Missing directories: `microservices/sheets/iac/oyatie-public-cloud/`, `microservices/sheets/iac/guest-on-aws/`, `microservices/sheets/iac/oci-guest/`, `microservices/sheets/iac/on-prem/`, `microservices/sheets/iac/colo/`, `microservices/sheets/iac/oyatie-iaas/`.
- Missing profile: `microservices/sheets/iac/oci-guest/always-free/`.
- Missing semantics: no `tenant_class`, `demo_trial`, or `revenue_share` string was found in the service path.
- Traceability gap: PRD acceptance criteria cite load-test files under `tests/load/*.js` at `microservices/sheets/PRD.md:541-547`, but no `tests/` directory exists and JavaScript is outside the Rust-strict backend policy.
- Traceability gap: PRD says approximately 115 crates at `microservices/sheets/PRD.md:317`, while manifest lists 20 crate names at `microservices/sheets/manifest.json:10-30`.
- Traceability gap: architecture starts with an explicit content-pass instruction at `microservices/sheets/ARCHITECTURE.md:3`.
- Judgment: artifacts are plentiful, but governance-significant fields are incomplete.

### §3.3 Product surface versus counterpart union coverage

- Finding: intended feature surface is broad enough for the stated counterpart bar.
- Evidence: local competitor set includes Google Sheets, Microsoft Excel Web, Airtable, and broader long-tail competitors at `microservices/sheets/competitor-parity-matrix.md:25-47`.
- Evidence: PRD counterpart table lists Google Sheets, Microsoft Excel Web, and Airtable as primary parity references at `microservices/sheets/PRD.md:436-440`.
- Evidence: chat history confirms the orchestrated top-three list at `8f603fc7-eb0e-4752-ab03-f8ab63ce113d.jsonl:16311`.
- Google Sheets surface: free-form grid, formulas, collaboration, protected ranges, import/export, Connected Sheets, Apps Script, Gemini/Smart Fill.
- Microsoft Excel Online surface: free-form grid, the strongest formula ecosystem, native XLSX semantics, co-authoring, Power Query-related workflows, Copilot, desktop/web bridge.
- Airtable surface: typed columns, base/table records, views, interface designer, automations, API, permissions, and database-grid ergonomics.
- Oyatie stated matches: free-form grid, >=400 functions, formula conformance, CRDT collaboration, charts, pivots, validation, import/export, per-range ACL, AI formula, connected sheets, triggers, embed bridge.
- Evidence: feature matrix rows cover grid/formula at `microservices/sheets/competitor-parity-matrix.md:50-60`.
- Evidence: feature matrix rows cover collaboration at `microservices/sheets/competitor-parity-matrix.md:62-70`.
- Evidence: feature matrix rows cover formatting/visualization at `microservices/sheets/competitor-parity-matrix.md:72-80`.
- Evidence: feature matrix rows cover import/export at `microservices/sheets/competitor-parity-matrix.md:82-93`.
- Evidence: feature matrix rows cover sharing/audit at `microservices/sheets/competitor-parity-matrix.md:95-103`.
- Evidence: feature matrix rows cover AI and automation at `microservices/sheets/competitor-parity-matrix.md:105-123`.
- Gap: strict OOXML fidelity is explicitly not included in M03 and is deferred at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Gap: Apps Script equivalent is deferred at `microservices/sheets/competitor-parity-matrix.md:121-123`.
- Gap: mobile app editor is deferred at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Gap: marketplace template ecosystem is deferred at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Judgment: feature ambition is industry-serious, but the current deliverables must distinguish contracted targets from measured evidence.

### §3.4 Canonical-direction alignment

- Finding: canonical-direction alignment is the main failure zone.
- Multi-context requirement evidence: ADR-0328 D-15 requires explicit support or justified N/A across the six contexts at `docs/decisions/ADR-0328-substance-bar-as-canonical-sequence-and-batch-discipline.md:1730-2235`.
- Master-plan evidence: deployment contexts and their IaC paths are defined at `specs/master-plan-sequencing.json:704-746`.
- Local evidence: `microservices/sheets/manifest.json:1-423` has no `deployment_contexts` key.
- Local evidence: `microservices/sheets/ARCHITECTURE.md:568-579` describes Kubernetes pods and Helm/Kustomize manifests, not per-context OpenTofu.
- Local evidence: the only `iac` subtree is Helm and Kustomize under `microservices/sheets/iac/`.
- Severity: P1 because the service is presumed deployable in all six contexts unless proven otherwise.
- OpenTofu requirement evidence: master plan says OpenTofu is the IaC engine and forbids Terraform, Pulumi, CloudFormation, ARM, null resources, local-exec, SSH provisioners, hand-edited tfstate, and unsigned modules at `specs/master-plan-sequencing.json:747-775`.
- Local evidence: `microservices/sheets/compliance.md:83` cites `iac/terraform/sheets-rbac.tf`.
- Local evidence: no `.tf` OpenTofu module files exist under the expected context paths.
- Severity: P1 for missing per-context OpenTofu and stale Terraform path.
- OS support requirement evidence: service-owned support manifest is required by ADR-0328 D-20 and OS memory at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_os_support_matrix_2026_05_20.md:56-76`.
- Local evidence: `microservices/sheets/supported-oses.json` is absent.
- Severity: P1 because the web/worker mix and sandboxed import/export path need OS/arch proof.
- Rust-strict requirement evidence: Rust-strict memory requires Rust for runtime/tooling/validation/codegen/deploy automation except allowlisted frontend and config at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_rust_strict_only_no_python_2026_05_20.md:10-18`.
- Local positive evidence: no files with forbidden source extensions matched under the service path.
- Local negative evidence: PRD acceptance criteria cite JavaScript load harnesses at `microservices/sheets/PRD.md:541-547`.
- Severity: P1 because the tests are not present and the documented verification path violates current language policy.
- OCI Always Free requirement evidence: master plan defines the profile and module path at `specs/master-plan-sequencing.json:857-868`.
- Local evidence: `microservices/sheets/iac/oci-guest/always-free/` is absent.
- Severity: P1 because demo/trial economics and capacity overlays depend on this profile.
- Language policy nuance: Leptos/WASM web is allowed, and PRD uses Leptos/WASM at `microservices/sheets/PRD.md:31`.
- Language policy nuance: no Rust source files are present either, so this audit cannot validate actual implementation language.
- Canonical contradiction: capability docs use T0/T1/T2 AI risk classes at `microservices/sheets/manifest.json:51-69`; that is not the same as retired demo_trial/paid/paid/compliance_pack capability tiers.
- Canonical risk: the word `tier` appears in several non-demo_trial contexts, including product classification and scale labels, and should be normalized during Wave 15J.

#### §3.4.T Tier retirement candidates

- Default severity for the following references: P2 documentation gap, Wave 15J retirement candidate.
- Search method: `rg -n "\b(demo_trial|paid|paid|compliance_pack)\b" microservices/sheets`.
- Occurrence count by token search: 42 matches.
- Candidate line count: 33 lines.
- `microservices/sheets/tutorials/build-100k-cell-financial-model.md:15` uses `paid-tier+`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:13` defines `demo_trial`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:20` says `demo_trial`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:45` defines `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:47` says `demo_trial`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:77` defines `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:79` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:98` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:101` says `paid` twice.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:105` defines `compliance_pack`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:107` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:118` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:120` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:127` says `demo_trial` and `demo_trial/paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:130` says `paid`.
- `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:134` contains the old migration chain across all four retired names.
- `microservices/sheets/faqs/sheets-engineer-faq.md:89` says `paid` and `paid`.
- `microservices/sheets/faqs/sheets-engineer-faq.md:106` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:21` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:22` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:29` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:31` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:37` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:38` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:43` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:49` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:50` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:55` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:57` says `paid`.
- `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:86` says `paid`.
- `microservices/sheets/onboarding/sheets-engineer-first-week.md:36` says `paid tier`.
- `microservices/sheets/onboarding/sheets-engineer-first-week.md:153` says `paid`.
- Retirement assessment: `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md` should be retired or replaced with tenant-class and deployment-context semantics.
- Retirement assessment: the benchmark doc should be replaced by the new benchmark document authored in this batch.
- Retirement assessment: FAQ, onboarding, and tutorial references should be rewritten to explain uniform quality with usage/cost/deployment overlays.
- Retirement assessment: capability T0/T1/T2 AI risk terms should be audited separately because they are not identical to retired commercial tiers but still create naming confusion.

#### §3.4.C Tenant-class adoption gaps

- Search method: `rg -n "tenant_class|demo_trial|revenue_share|per_usage|per-seat|per seat" microservices/sheets`.
- Direct `tenant_class` evidence: absent.
- Direct `demo_trial` evidence: absent.
- Direct `revenue_share` evidence: absent.
- Direct `per_usage` evidence: absent.
- Per-seat evidence exists in PRD, policy, threat model, cost, contracts, and phase docs.
- Example per-seat evidence: `microservices/sheets/PRD.md:73` requires per-seat licensing through Cedar at workbook open.
- Example per-seat evidence: `microservices/sheets/contracts/openapi/sheets.yaml:338` says workbook session open enforces Cedar per-seat license gate.
- Example per-seat evidence: `microservices/sheets/threat-model.md:674` lists Cedar per-seat license-gate as a preventive billing control.
- Gap: the service has per-seat license enforcement but no tenant-class abstraction.
- Gap under user-prompt model: no explicit `demo_trial`, `paid`, or `revenue_share` semantics.
- Gap under memory-corrected model: no explicit `demo_trial` and `paid` classes, and no paid billing components for `revenue_share`, `per_seat`, and `per_usage`.
- Required adoption direction: move feature gating language out of commercial tiers and into uniform-quality policy plus usage, SLO, compliance, BYOK, and substrate overlays.
- Required demo/trial direction: express OCI Always Free profile caps as infrastructure and usage limits, not a reduced product-quality lane.
- Required paid direction: express per-seat and usage-based billing as meter semantics, not as quality tiers.
- Required revenue-share direction: if retained from the user prompt, encode it as tenant billing model or paid billing component, not as a separate feature-quality level.

### §3.5 Cross-cutting implementation readiness

- Finding: implementation readiness is mixed.
- Positive evidence: implementation plans IP-001 through IP-015 exist and cover IaC bootstrap, cargo workspace, formula engine, recalc, CRDT, storage, adapters, formatting, import/export, sharing, AI, connected sheets, REST/Leptos, SLOs, and registration.
- Positive evidence: ten 400-line journey IPs exist for pack overlays and first action.
- Positive evidence: PRD acceptance criteria list concrete cargo tests for formula, ACL, CRDT, license gate, import scans, AI, and Cedar validation at `microservices/sheets/PRD.md:531-559`.
- Negative evidence: no `src/` directory exists under `microservices/sheets/`.
- Negative evidence: no `tests/` directory exists under `microservices/sheets/`.
- Negative evidence: acceptance criteria reference `tests/load/*.js` at `microservices/sheets/PRD.md:541-547`.
- Negative evidence: local catalog lists component metadata but not source implementation.
- Risk: the service is spec-complete in intent but not buildable from this path alone.
- Risk: an intern could understand the desired architecture but could not run the named tests.
- Risk: deployment proof is blocked by missing OpenTofu modules.
- Risk: OS proof is blocked by missing `supported-oses.json`.
- Risk: benchmark proof is blocked by missing measured harness artifacts.
- Judgment: implementation readiness should be classified as design-complete but execution-unproven.

### §3.6 Performance, SLOs, and benchmark coherence

- Finding: SLO coverage is substantial, but benchmark coherence is not current.
- Evidence: PRD performance targets are listed at `microservices/sheets/PRD.md:82-100`.
- Evidence: duplicated PRD target table is at `microservices/sheets/PRD.md:472-482`.
- Evidence: horizontal scalability envelope is at `microservices/sheets/PRD.md:500-509`.
- Evidence: capacity model throughput formulae are at `microservices/sheets/capacity-model.md:45-58`.
- Evidence: storage formulae are at `microservices/sheets/capacity-model.md:60-91`.
- Evidence: component replica formulae are at `microservices/sheets/capacity-model.md:104-121`.
- Evidence: WebSocket sizing is 10,000 connections per pod at `microservices/sheets/capacity-model.md:175-180`.
- Evidence: recalc worker sizing is 50 recalcs/sec per worker at `microservices/sheets/capacity-model.md:184-189`.
- Evidence: XLSX export worker sizing is 60 jobs/minute at `microservices/sheets/capacity-model.md:193-198`.
- Gap: current benchmark doc uses retired commercial tier headings and rows at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13-57`.
- Gap: current benchmark doc asserts superiority claims at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:29-57`.
- Contradiction: competitor parity matrix forbids faster-than-Google claims without measured benchmark evidence at `microservices/sheets/competitor-parity-matrix.md:178-188`.
- Gap: no service-local benchmark harness directory exists at `microservices/sheets/benchmarks/sheetsbench/`, despite the benchmark reproducibility block at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:95-104`.
- Gap: performance targets are not overlaid by deployment context or tenant class.
- New deliverable direction: performance benchmark numbers must use a single industry-leader target set plus deployment-context and tenant-class overlays.
- Judgment: SLO intent is strong; benchmark evidence and canonical expression need remediation.

### §3.7 Reliability, failure modes, incident response, and operations

- Finding: reliability documentation is mature relative to many services.
- Evidence: failure modes enumerate collaboration desync, recalc storms, formula rollback, export pipeline failure, share ACL drift, cross-tenant leak, XLSX regression, AI timeout, and external-source failure at `microservices/sheets/failure-modes.md:30-262`.
- Evidence: failure-mode RTO/RPO summary exists at `microservices/sheets/failure-modes.md:264-285`.
- Evidence: detection meta-SLO is stated at `microservices/sheets/failure-modes.md:287-296`.
- Evidence: incident severity definitions exist at `microservices/sheets/incident-response.md:28-37`.
- Evidence: incident roles and escalation path are stated at `microservices/sheets/incident-response.md:39-71`.
- Evidence: incident lifecycle is stated at `microservices/sheets/incident-response.md:75-88`.
- Evidence: regulatory notification details are present at `microservices/sheets/incident-response.md:147-206`.
- Gap: incident response still uses broader vocabulary, including `Production-tier` and an on-call table header at `microservices/sheets/incident-response.md:34` and `microservices/sheets/incident-response.md:231-241`.
- Gap: failure modes are not tied to deployment-context-specific OpenTofu modules or OS support lanes.
- Gap: runbooks are present but not proven executable by CI artifacts in this audit.
- Judgment: operational readiness docs are useful; canonical context overlays remain missing.

### §3.8 Security, privacy, compliance, and data governance

- Finding: security and privacy surface is strong but has unresolved evidence debt.
- Evidence: PRD security controls include OIDC, per-seat Cedar, CSP, XSS posture, CDN cache partitioning, WebSocket auth, AI formula validation, XLSX sandboxing, tenant isolation, SRI, and per-range ACL at `microservices/sheets/PRD.md:102-115`.
- Evidence: PRD audit and compliance controls are at `microservices/sheets/PRD.md:116-124`.
- Evidence: data residency is pack-region-pinned at `microservices/sheets/PRD.md:135-138`.
- Evidence: OpenAPI requires tenant header semantics at `microservices/sheets/contracts/openapi/sheets.yaml:35-41`.
- Evidence: proto has pack, jurisdiction, license decision, and data-class enums at `microservices/sheets/contracts/proto/sheets.proto:81-129`.
- Evidence: compliance mapping includes SOC 2, ISO 27001, OWASP ASVS, EU AI Act, and per-pack overlays.
- Gap: compliance line `microservices/sheets/compliance.md:83` points to `iac/terraform/sheets-rbac.tf` while claiming OpenTofu-managed RBAC.
- Gap: DPIA sign-offs remain pending at `microservices/sheets/dpia.md:188-198`.
- Gap: DPIA references legal artifacts under `microservices/sheets/legal/` at `microservices/sheets/dpia.md:203-206`, but no `legal/` directory appears in the inventory.
- Gap: no tenant-class privacy/billing semantics exist, so demo/trial versus paid compliance behavior is not encoded.
- Gap: no OCI Always Free profile exists, so demo/trial privacy and capacity limits cannot be verified.
- Judgment: security design is detailed, but evidence links and canonical billing/tenant dimensions need cleanup.

### §3.9 Build, verification, and audit-deliverable readiness

- Finding: the service is not locally verifiable as an implementation from this path.
- Evidence: no service-local Rust source files are present in `microservices/sheets/src/`.
- Evidence: no service-local tests are present in `microservices/sheets/tests/`.
- Evidence: no forbidden source-extension files are present, which is good for policy compliance.
- Evidence: PRD verification commands are named, but many targets reference packages and tests not present in the service path at `microservices/sheets/PRD.md:531-559`.
- Evidence: IP-015 asks for HG registration and branch protection, but this audit did not perform VCS state transitions because the user explicitly requested no commits and report-only output.
- Evidence: brief-template says line count is only a floor and not a substance proxy at `docs/standards/brief-template.md:169-187`.
- Evidence: deliverable verification memory says not to trust line count or self report alone at `/Users/jasonlee/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_verify_deliverables_not_just_line_count_2026_05_20.md:10-53`.
- This audit therefore validates authored file existence, line count, absence of retired fourth deliverable, tier candidate count, and final report fields.
- This audit does not claim implementation tests pass.
- This audit does not claim deployment plans can run.
- This audit does not claim performance numbers are measured in this repository.
- This audit does claim that the three requested reports have been authored to the required line floors after validation.

## §4 Findings table

| ID | Severity | Finding | Evidence | Remediation |
|---|---|---|---|---|
| SHEETS-P1-001 | P1 | Missing six-context deployment declaration and context-specific OpenTofu modules. | `specs/master-plan-sequencing.json:704-746`; `microservices/sheets/ARCHITECTURE.md:568-575`; inventory under `microservices/sheets/iac/` | Add `deployment_contexts` to manifest and per-context OpenTofu modules or explicit N/A rationales. |
| SHEETS-P1-002 | P1 | OpenTofu doctrine is violated by a stale Terraform evidence path and no context modules. | `specs/master-plan-sequencing.json:747-775`; `microservices/sheets/compliance.md:83` | Replace stale Terraform path and create OpenTofu evidence under canonical context dirs. |
| SHEETS-P1-003 | P1 | OS support matrix is missing. | `specs/master-plan-sequencing.json:777-815`; absent `microservices/sheets/supported-oses.json` | Add service-owned OS support manifest covering Tier-1 OSes and explicit out-of-scope notes. |
| SHEETS-P1-004 | P1 | Rust-strict verification path is contradicted by documented JavaScript load tests and no tests directory. | `feedback_rust_strict_only_no_python_2026_05_20.md:10-18`; `microservices/sheets/PRD.md:541-547`; no `tests/` inventory | Replace load harness references with Rust-native or allowlisted web test posture and land actual tests. |
| SHEETS-P1-005 | P1 | OCI Always Free profile is absent. | `specs/master-plan-sequencing.json:857-868`; absent `microservices/sheets/iac/oci-guest/always-free/` | Add profile module and capacity/billing overlay for demo/trial tenants. |
| SHEETS-P1-006 | P1 | Manifest under-declares the PRD crate and layer surface. | `microservices/sheets/PRD.md:317-359`; `microservices/sheets/manifest.json:6-39` | Align manifest bounded contexts, crates, layers, and SDK boundaries to the PRD. |
| SHEETS-P2-001 | P2 | Retired commercial vocabulary remains in 42 token occurrences across 33 lines. | §3.4.T candidate list | Retire `tenant-class-adoption/` and rewrite benchmark, onboarding, FAQ, and tutorial references. |
| SHEETS-P2-002 | P2 | Tenant-class semantics are absent. | no `tenant_class`, `demo_trial`, or `revenue_share` matches; per-seat-only evidence at `microservices/sheets/PRD.md:73` | Add tenant-class and billing-component semantics without feature-quality tiers. |
| SHEETS-P2-003 | P2 | Benchmark document carries retired schema rows and unsupported superiority language. | `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13-57`; `microservices/sheets/competitor-parity-matrix.md:178-188` | Replace with single target set and mark public limits versus internal targets. |
| SHEETS-P2-004 | P2 | Product versus substrate boundary is not machine-readable. | `microservices/sheets/PRD.md:31-35`; `microservices/sheets/manifest.json:6-33` | Split product shell, reusable grid kernel, formula, and recalc ownership in manifest. |
| SHEETS-P2-005 | P2 | Architecture file still carries generated-stub instruction. | `microservices/sheets/ARCHITECTURE.md:3` | Run a content pass that removes generator residue after validating each anchor. |
| SHEETS-P2-006 | P2 | Capacity and cost docs use broader scale-vocabulary not mapped to tenant classes. | `microservices/sheets/capacity-model.md:123-132`; `microservices/sheets/cost-budget.md:97-104` | Re-express scale as deployment-context size overlays and tenant usage caps. |
| SHEETS-P2-007 | P2 | DPIA sign-offs and legal evidence paths are unresolved. | `microservices/sheets/dpia.md:188-206`; missing `microservices/sheets/legal/` in inventory | Land sign-off evidence or mark pre-onboarding gate as blocked with owner and path. |
| SHEETS-P2-008 | P2 | Contract surfaces lack explicit tenant-class and billing-meter event semantics. | `microservices/sheets/contracts/openapi/sheets.yaml:35-41`; `microservices/sheets/contracts/asyncapi/sheets-events.yaml:19-120`; no tenant-class search hits | Add billing/meter events and class-aware entitlements without making tenant_class caller-controlled. |
| SHEETS-P2-009 | P2 | Existing benchmark reproducibility path is not present. | `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:95-104`; no `benchmarks/sheetsbench/` inventory | Add Rust-native benchmark harness or remove reproducibility claim. |
| SHEETS-P3-001 | P3 | Local competitor matrix is broader than the Batch 3.2 top-three scope. | `microservices/sheets/competitor-parity-matrix.md:25-47`; chat `8f603fc7...jsonl:16311` | Keep broad matrix as additive, but make top-three explicit in batch deliverables. |
| SHEETS-P3-002 | P3 | Some non-retired tier words remain as classification labels and may confuse Wave 15J cleanup. | `microservices/sheets/manifest.json:336-340`; `microservices/sheets/manifest.json:369-420`; `microservices/sheets/ARCHITECTURE.md:22-24` | Audit T0/T1/T2 AI risk and service criticality terms separately from retired commercial tiers. |

## §5 Open questions

1. Should `revenue_share` remain a first-class tenant class per this prompt, or follow the memory correction that makes it a billing component under `paid`?
2. Should the reusable cell-grid, formula-engine, and recalc-engine be separate bounded contexts inside `sheets`, or split into separate services after Wave 3?
3. Which context should own the canonical workbook graph when `cell` already owns per-workbook cell boundary storage?
4. Should `sheets` deploy the full editor shell in all six contexts, or can some contexts deploy only API/worker surfaces with explicit N/A for browser UX?
5. What is the required minimum demo/trial cap for cells, workbooks, collab editors, import size, AI calls, and connected refreshes under the OCI Always Free profile?
6. Which team owns the replacement for `tenant-class-adoption/tenant-class-adoption-record.md` during Wave 15J cleanup?
7. Should T0/T1/T2 AI capability terminology be renamed to avoid collision with retired commercial vocabulary?
8. What is the approved Rust-native substitute for the documented `tests/load/*.js` benchmark files?
9. Which benchmark harness path should be canonical if `benchmarks/sheetsbench/` does not exist today?
10. What is the authoritative public/private source for measured Google Sheets, Excel Online, and Airtable latency comparisons, given local claim-boundary rules?
11. Should strict OOXML round-trip be a post-M03 enhancement or a pre-GA blocker for parity with Microsoft Excel Online?
12. How should Airtable-style database views be represented in contracts: as sheets workbook metadata, ontology descriptors, or a separate typed-table submodel?
13. How should per-range ACL evidence be surfaced to auditors without leaking sensitive cell-range names?
14. Which deployment-context overlay owns disconnected/offline buffer semantics for on-prem and colo tenants?
15. How should BYOK availability be constrained for demo/trial versus paid tenants after tenant-class adoption?
16. What SLO downgrade language is allowed for demo/trial best-effort operation while keeping uniform feature quality?
17. Should DPIA legal artifacts be created under `microservices/sheets/legal/`, or should the DPIA cite shared legal templates instead?
18. What is the minimum OS/arch smoke test matrix for the Leptos/WASM editor, import sandbox, recalc worker, and WebSocket gateway?
19. Should the service define a separate OCI Always Free capacity model or inherit a shared profile from cloud-iac?
20. What is the promotion gate for removing all demo_trial/paid/paid/compliance_pack references from service-local docs?

### §5.1 Evidence-backed remediation sequence

- Step 01: preserve the current PRD product definition because it clearly states the hero-product surface at `microservices/sheets/PRD.md:29-35`.
- Step 02: formalize the product-shell versus reusable-kernel split because the PRD says both `not substrate` and `shared substrate` at `microservices/sheets/PRD.md:31-35`.
- Step 03: update `microservices/sheets/manifest.json` before adding more prose because the manifest currently lists one bounded context and 20 crates at `microservices/sheets/manifest.json:6-39`.
- Step 04: align the manifest with the PRD's approximately 115-crate claim at `microservices/sheets/PRD.md:317`.
- Step 05: add `deployment_contexts` to the manifest using the six context ids from `specs/master-plan-sequencing.json:704-746`.
- Step 06: create or explicitly justify each context path required by OpenTofu doctrine.
- Step 07: replace the Helm/Kustomize-only deployment evidence with context-specific OpenTofu modules or move Helm/Kustomize under OpenTofu-managed release units.
- Step 08: fix the stale `iac/terraform/sheets-rbac.tf` evidence path at `microservices/sheets/compliance.md:83`.
- Step 09: land `microservices/sheets/iac/oci-guest/always-free/` because the OCI Always Free profile is required by `specs/master-plan-sequencing.json:857-868`.
- Step 10: add `microservices/sheets/supported-oses.json` before claiming on-prem, colo, or guest-account portability.
- Step 11: derive the OS matrix from `specs/master-plan-sequencing.json:777-815` rather than inventing service-local support classes.
- Step 12: replace JavaScript load-test references at `microservices/sheets/PRD.md:541-547` with Rust-native or approved frontend verification paths.
- Step 13: create the benchmark harness path before retaining the existing reproducibility claim at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:95-104`.
- Step 14: retire `microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md` or replace it with tenant-class and deployment-context semantics.
- Step 15: rewrite onboarding line `microservices/sheets/onboarding/sheets-engineer-first-week.md:36` so sheet-open targets are not tied to retired commercial tiers.
- Step 16: rewrite onboarding line `microservices/sheets/onboarding/sheets-engineer-first-week.md:153` so XLSX fidelity is not tied to retired commercial tiers.
- Step 17: rewrite FAQ line `microservices/sheets/faqs/sheets-engineer-faq.md:89` so workbook sheet-count guidance is based on measured limits or tenant caps.
- Step 18: rewrite FAQ line `microservices/sheets/faqs/sheets-engineer-faq.md:106` so workflow bridge stability is not described through retired tiers.
- Step 19: replace benchmark doc rows at `microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13-57` with a single target table plus overlays.
- Step 20: preserve local claim-boundary rules at `microservices/sheets/competitor-parity-matrix.md:178-188`.
- Step 21: add tenant-class semantics without letting callers submit arbitrary tenant class headers.
- Step 22: map existing per-seat gate evidence at `microservices/sheets/PRD.md:73` into the paid tenant billing model.
- Step 23: if `revenue_share` remains a class per prompt, model it as billing and substrate economics rather than feature quality.
- Step 24: if `revenue_share` follows memory correction, model it as a paid billing component and cite the memory correction.
- Step 25: add billing and usage events to AsyncAPI because `microservices/sheets/contracts/asyncapi/sheets-events.yaml:19-120` does not currently express tenant-class meters.
- Step 26: add contract quota language to OpenAPI because Google and Airtable publish API quotas while Sheets local contracts do not.
- Step 27: keep the no-silent-loss CRDT invariant as a differentiator because it is already explicit in `microservices/sheets/competitor-parity-matrix.md:166-176`.
- Step 28: keep per-range Cedar ACL as a differentiator because it is already explicit in `microservices/sheets/PRD.md:64`.
- Step 29: keep gVisor plus ClamAV/OPSWAT import defense because it is already explicit in `microservices/sheets/PRD.md:111`.
- Step 30: move strict OOXML fidelity to an explicit gap with owner and exit criteria because local matrix already defers it at `microservices/sheets/competitor-parity-matrix.md:156-164`.
- Step 31: move Apps-Script-equivalent automation to an explicit gap with owner and exit criteria because local matrix marks it absent at `microservices/sheets/competitor-parity-matrix.md:121-123`.
- Step 32: add Airtable-style view and interface gaps to the product backlog because typed-column parity through ontology is not equivalent to Interface Designer.
- Step 33: resolve DPIA sign-offs at `microservices/sheets/dpia.md:188-198` before first regulated tenant onboarding.
- Step 34: create or correct the legal artifact paths named at `microservices/sheets/dpia.md:203-206`.
- Step 35: update capacity-model scale labels at `microservices/sheets/capacity-model.md:123-132` so they cannot be mistaken for retired commercial tiers.
- Step 36: update cost-budget scale wording at `microservices/sheets/cost-budget.md:97-104` to tenant-class and context overlays.
- Step 37: update observability metric dimension `cell_tier` at `microservices/sheets/ARCHITECTURE.md:636` because the word collides with retired commercial-tier cleanup.
- Step 38: update abuse-defense reference to tenant_class at `microservices/sheets/ARCHITECTURE.md:697` with quota or tenant-class semantics.
- Step 39: keep T0/T1/T2 AI risk records only if their meaning is documented as AI capability risk, not commercial feature quality.
- Step 40: rerun the exact vocabulary search after cleanup and require zero demo_trial/paid/paid/compliance_pack matches under `microservices/sheets/`.
- Step 41: rerun tenant-class search after adoption and require explicit `demo_trial` plus `paid` semantics, and `revenue_share` either as class or paid billing component according to final doctrine.
- Step 42: rerun forbidden source-extension search after benchmark harness work and require no backend Python, JavaScript, Ruby, Go, Java, Scala, Groovy, PHP, or F# files.
- Step 43: rerun `wc -l` only as a floor check, not as proof of substance.
- Step 44: use this audit's findings table as the local remediation backlog seed.
- Step 45: do not author a fourth tier-delta document for this batch.

<!-- ORCHESTRATOR REPORT
  µservice: sheets
  deliverables_landed:
    - microservices/sheets/coherence-audit-2026-05-20.md (621 lines)
    - microservices/sheets/feature-parity-matrix-2026-05-20.md (410 lines)
    - microservices/sheets/performance-benchmark-numbers-2026-05-20.md (333 lines)
  inventory_files_seen: 126
  inventory_lines_read: 21560
  chat_history_matches_processed: 5
  findings_p0: 0
  findings_p1: 6
  findings_p2: 9
  findings_p3: 2
  tier_retirement_candidates_found: 42 token matches across 33 cited lines:
    microservices/sheets/tutorials/build-100k-cell-financial-model.md:15
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:13
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:20
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:45
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:47
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:77
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:79
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:98
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:101
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:105
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:107
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:118
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:120
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:127
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:130
    microservices/sheets/tenant-class-adoption/tenant-class-adoption-record.md:134
    microservices/sheets/faqs/sheets-engineer-faq.md:89
    microservices/sheets/faqs/sheets-engineer-faq.md:106
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:13
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:21
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:22
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:29
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:31
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:37
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:38
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:43
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:49
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:50
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:55
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:57
    microservices/sheets/benchmarks/sheets-vs-google-sheets-vs-excel-web-vs-airtable.md:86
    microservices/sheets/onboarding/sheets-engineer-first-week.md:36
    microservices/sheets/onboarding/sheets-engineer-first-week.md:153
  tenant_class_adoption_gaps: yes - no tenant_class/demo_trial/revenue_share/per_usage semantics; per-seat licensing exists but is not the replacement tenant-class model.
  top_3_counterparts_confirmed: Google Sheets / Microsoft Excel Online / Airtable
  five_constraint_dimensions_evaluated: yes
  halt_cleanly_invoked: no
  total_lines_authored: 1364
-->
