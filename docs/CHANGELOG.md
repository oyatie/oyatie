
## 2026-05-12 — Lifted 5 reference docs (deep-dive ×2, hyperscaler, LTS-versions, cutover-amendments) to canonical docs/{specs,research,plans}/ tree

## 2026-05-12 — Lifted 11 branch-pipeline docs including ADR-0055 to docs/advanced-cicd/branch-pipeline/

## 2026-05-12 — Lifted 10 release-versioning docs to docs/advanced-cicd/release-versioning/

## 2026-05-12 — Lifted 9 progressive-delivery specs (shard A) to docs/advanced-cicd/progressive-delivery/

- 9 progressive-delivery specs lifted from `.omc/advanced-cicd/progressive-delivery/` to `docs/advanced-cicd/progressive-delivery/`. Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; `adrs_cited: [ADR-0053, ADR-0052, ADR-0054]` added to all frontmatter. Body content preserved verbatim.
- Files landed: `INDEX.md`, `blue-green-spec.md`, `canary-rail-spec.md`, `dark-launch-spec.md`, `enforcement-lanes.md`, `feature-flag-architecture.md`, `playbook-ads.md`, `playbook-cloud.md`, `playbook-cross-axis-contract.md`.

## 2026-05-12 — Stage 1 Wave 2: 17 standards landed at docs/standards/

- 17 cross-cutting authoring standards lifted from `.omc/standards/` to `docs/standards/` (INDEX + 16 standard files). Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added to all files. ADR-0053 (sanctioned primitives), ADR-0052 (pre-grit artifact inventory), and ADR-0054 (scaffold-claim pattern) cited in every file's frontmatter `related_adrs:` field. Body content preserved verbatim.
- Files landed: `INDEX.md`, `doc-style.md`, `code-style-rust.md`, `error-handling.md`, `testing.md`, `security-review.md`, `on-call.md`, `claude-code-harness.md`, `multi-agent-tool-map.md`, `observability.md`, `release-management.md`, `git-workflow.md`, `dependency-policy.md`, `image-discipline.md`, `data-class.md`, `autonomy-ceiling.md`, `agent-instructions-discipline.md`.
- Resolves all `<!-- forward-reference: wave-2 -->` sentinels in `docs/AGENTS.md`, `docs/README.md`, and `docs/CONSTITUTION.md` pointing at `standards/*` rows.

## 2026-05-12 — Stage 1 Wave 2: 64 fitness-lane specs lifted to docs/fitness-lanes/

- 64 fitness-lane catalogue specs lifted from `.omc/fitness-lanes/` to `docs/fitness-lanes/` (64 lane files + INDEX). Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added. ADR-0053 cited in lanes enforcing sanctioned-primitive rules (adapter-kernel, banned-primitives, bypass, cloud-mutation, cutover-bootstrap-window, direct-tool-invocation-audit, provider-agnostic); ADR-0052 cited in portfolio-citation (inventory); ADR-0054 cited in agent-completion-checklist, claim-ceiling, scaffold-claim-pattern. Kernel implementations deferred to Stage 3.

## 2026-05-12 — Stage 1 Wave 2: templates + checklists lifted to docs/templates/ + docs/checklists/ (25 files)

- **doc.templates-index** (Tier 2): 13 template files lifted from `.omc/templates/` to `docs/templates/` (INDEX + 12 templates); 12 checklist files lifted from `.omc/templates/checklists/` to `docs/checklists/`. Status set to `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; ADR-0052 + ADR-0053 + ADR-0054 cited in every file's frontmatter and body prose where sanctioned primitives, inventory ledger, and scaffold-claim are referenced.
- 4 templates renamed to `-v2` due to conflicts with existing `docs/templates/` files: `pull-request-template-v2.md`, `adr-template-v2.md`, `runbook-template-v2.md`, `capability-record-template-v2.yaml`. Each carries `header_note: "Supersedes prior docs/templates/<name>.md once reviewed."` and `supersedes:` frontmatter field.
- 0 checklist conflicts (all 12 checklists are new additions; existing `docs/checklists/cross-axis-contract-change.md` preserved; new `cross-axis-contract-change-checklist.md` carries `extends:` pointer to the prior file).
- Existing `docs/templates/` files preserved as-is: `migration-runbook-template.md`, `dpia-template.md`, `team-charter-template.md`, `threat-model-template.md`, `incident-postmortem-template.md`, and others out of scope of this delivery.
  - Authors: jason931225
  - ADRs cited: ADR-0052, ADR-0053, ADR-0054
  - Related lanes: oya-foundry-fitness-plan-hierarchy, oya-foundry-fitness-pr-shape, oya-foundry-fitness-capability-publish, oya-foundry-fitness-inventory-tracker, guard-pr-merge-review.mjs
  - Commit: Stage-1-Wave-2-templates-checklists

## 2026-05-12 — Stage 1 Wave 2: automation pipeline + visualization + discipline specs landed (19 files)

- **doc.automation-index** (Tier 2): 19 automation specs lifted from `.omc/automation/` to `docs/automation/`; covers 8 auto-doc-generation pipelines (rustdoc, openapi, adr-index, runbook-freshness, fitness-lane-reports, schema-doc, changelog, glossary), 7 architecture-visualization specs (architecture-map-kernel, product-map, service-map, tech-stack-map, roadmap-visualization, dependency-graph, audit-chain-map), and 3 discipline specs (doc-freshness, orphan-detection, cross-reference-index). Status set to Accepted; `lift_target:` removed; `date: 2026-05-12` added; ADR-0052 + ADR-0053 + ADR-0054 cited in every file. Kernel crates (architecture-map, doc-freshness, orphan-detection) land in Stage 3.
  - Authors: jason931225
  - ADRs cited: ADR-0052, ADR-0053, ADR-0054
  - Related lanes: oya-foundry-fitness-doc-freshness, oya-foundry-fitness-orphan-detection, oya-foundry-fitness-cross-reference-index
  - Commit: Stage-1-Wave-2

## 2026-05-12 — Stage 1 Wave 3: ai-slop-defense lifted to docs/quality/ai-slop-defense/ (7 files)

- Lifted all 7 files from `.omc/advanced-cicd/ai-slop-defense/` to `docs/quality/ai-slop-defense/`: INDEX, ai-slop-failure-mode-catalogue, production-quality-bar, gap-analysis-ai-vs-production, defense-in-depth-architecture, additional-tooling-recommendations, impossible-to-fail-environment-spec.
- Per-file transforms: `status: pending approval` → `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; ADR-0053 + ADR-0055 cited in each file's frontmatter (`adr_citations:`) and body prose.

## 2026-05-12 — Stage 1 Wave 2: agent-kickoff layer lifted to docs/agents/ (11 files)

- Lifted all 11 files from `.omc/agent-kickoff/` to `docs/agents/`: INDEX, AGENT-ENTRY-POINT, AGENT-DECISION-TREE, AGENT-TOOL-PROTOCOL, AGENT-COMPLETION-PROTOCOL, AGENT-FAILURE-RECOVERY, AGENT-ICM-TOPIC-CONVENTIONS, CROSS-REFERENCE-INDEX, AGENT-CHEAT-SHEET, HUMAN-OPERATOR-GUIDE, ESCALATION-MATRIX.
- Per-file transforms: `status: pending approval` → `Accepted`; `lift_target:` field removed; `date: 2026-05-12` added; internal references updated from `.omc/standards/` → `docs/standards/`, `.omc/templates/` → `docs/templates/`, `.omc/fitness-lanes/` → `docs/fitness-lanes/`.
- Foundation ADRs ADR-0053 (sanctioned primitives) and ADR-0054 (scaffold-claim) cited in each file's frontmatter and body.

## 2026-05-12 — ADR-0052 Inventory ledger for grit/icm cutover landed

- ADR-0052 inventory ledger for grit/icm cutover landed; classifies 211 artifacts across oyatie/ and bominal/ by closed-set action; satisfies spec A2; ADR-INDEX updated.

## 2026-05-12 — MASTERPLAN lifted to canonical docs/MASTERPLAN.md (Stage 1 Wave 1)

- Promoted `.omc/plans/MASTERPLAN.md` to `docs/MASTERPLAN.md` as the Accepted canonical Master Plan anchor (authority tier 0).
- Status changed from `pending approval` to `Accepted`; `lift_target` field removed; `date: 2026-05-12` and `owners: ["council-architecture"]` added.
- §Authority-anchor section added: all milestone/phase/IP files under `docs/plans/milestones/M*/` derive authority from this document and ultimately from `docs/CONSTITUTION.md`.
- Foundation ADRs ADR-0052, ADR-0053, ADR-0054 cited in §Principles as the underpinning ADR triad.
- All internal milestone/phase/IP links updated to `docs/plans/milestones/...` canonical paths.
- `docs/README.md` updated: MASTERPLAN.md added to Tier-1 documents section and root document index table (`doc.masterplan`, tier 0).

## 2026-05-12 — ADR-0053: grit + icm + oya-tooling-agent-read as sole sanctioned primitives

- Authored `docs/decisions/ADR-0053-grit-icm-as-sanctioned-primitives.md` (Accepted).
- Fixes the agent-callable coordination/state-transition primitive set at `{grit, icm, oya-tooling-agent-read}`; direct `git`/`gh` permitted only with documented rationale per Directive 12.
- Enforced by `oya-foundry-fitness-banned-primitives` lane (defined P4, activated P5 merge boundary).
- Consensus reached iter-2 via Planner+Architect+Critic; operational driver: `.omc/plans/ralplan-oyatie-sst-consolidation.md`.
- Sibling ADRs landing in parallel: ADR-0052 (pre-grit artifact inventory), ADR-0054 (grit scaffold-claim pattern).

## 2026-05-12 — ADR-0054: grit scaffold-claim pattern (icm-coordination-lock fallback)

- Authored `docs/decisions/ADR-0054-grit-scaffold-claim-pattern.md` (Accepted).
- Formalises the icm-coordination-lock fallback (`scaffold-locks-oyatie` topic) as the canonical scaffold-claim path for new-crate creation at grit v0.3.0, following Lane 3 deep-dive trace confirmation that `Cargo.toml::workspace_members` is not indexed by grit (zero matches, 2026-05-12).
- Documents the verbatim 7-step sequence, two rejected alternatives (workspace_members grit claim; per-file-path lock), worked example with icm store rows, and two upstream follow-up issues.
- Updated `ADR-INDEX.md`: total ADRs 52, next number 0055, ADR-0052/0053 placeholders noted, ADR-0054 row appended.

## 2026-05-12 — Foundry RAG retrieve API contract

- Added the stable `foundry.rag.retrieve` REST boundary via `oya-foundry-rag-api`, enforcing tenant/index namespace binding, Foundry authorization evidence, idempotent retrieval semantics, privacy-program data-class allowlists, and purpose-bound consent receipts before citation return.
- Registered `contracts/openapi/foundry/rag-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Foundry capability publish API contract

- Added the stable `foundry.capability.publish` REST boundary via `oya-foundry-registry-api`, enforcing path/body capability binding, Cedar authorization evidence, idempotent publish semantics, typed capability schema projection, provider/cost validation, and signed passing eval gates.
- Registered `contracts/openapi/foundry/registry-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Foundry autonomy ceiling policy publish API contract

- Added the stable `foundry.policy.autonomy-ceiling.publish` REST boundary over `oya-foundry-policy-kernel`, including idempotent publish semantics, Cedar policy refs, autonomy decision evidence, and OpenAPI runtime/schema parity.
- Registered `contracts/openapi/foundry/policy-v1.yaml` in the OpenAPI registries, catalog, SPEC, Foundry PRD, and machine-readable contract mirror.

## 2026-05-12 — Platform DSR cascade execute API contract

- Added `dsr.cascade.execute` OpenAPI/runtime/schema/catalog parity via `oya-platform-dsr-app`.
- Bound idempotent DSR cascade execution to tenant privacy-officer authorization, path/body DSR identity, cross-axis store scope, terminal acknowledgements, proof-of-erasure coverage, SLA status projection, and stable error envelopes.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Forms submission ingest API contract

- Added `workspace.forms.submission.ingest` OpenAPI/runtime/schema/catalog parity via `oya-workspace-forms-api`.
- Bound idempotent Forms submission ingest to tenant form schemas, submitter-principal validation, required-answer enforcement, Object Graph route projection, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Chat message send API contract

- Added `workspace.chat.message.send` OpenAPI/runtime/schema/catalog parity via `oya-workspace-chat-api`.
- Bound idempotent Chat message sends to tenant channel membership, sender-principal validation, parent-thread existence, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Meet session start API contract

- Added `workspace.meet.session.start` OpenAPI/runtime/schema/catalog parity via `oya-workspace-meet-api`.
- Bound idempotent Meet session starts to tenant cell/SFU placement, host participant validation, privacy-program data-class labels, and stable error envelopes.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Workspace Drive object API contract

- Added `workspace.drive.put` and `workspace.drive.get` OpenAPI/runtime/schema/catalog parity via `oya-workspace-drive-api`.
- Bound idempotent Drive object metadata writes and ACL-checked reads to the Workspace Drive kernel, preserving KMS-shred object bindings and tenant-scoped data-class labels.
- Mirrored the contract in SPEC, Workspace PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Foundry eval run API contract

- Added `foundry.eval.run` OpenAPI/runtime/schema/catalog parity via `oya-foundry-eval-app`.
- Bound authenticated, idempotent eval-run recording to signed eval sets, mandatory adversarial + linguistic cohorts, pass-threshold enforcement, and stable error envelopes.
- Mirrored the contract in SPEC, Foundry PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform regulatory pack bind API contract

- Added `regulatory-pack.bind` OpenAPI/runtime/schema/catalog parity via `oya-platform-regulatory-pack-api`.
- Bound authenticated, idempotent tenant pack binding to regional-pack validation, immutable tenant residency binding, multi-pack record projection, and authorization evidence.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform Object Graph entity upsert API contract

- Added `object-graph.entity.upsert` OpenAPI/runtime/schema/catalog parity via `oya-platform-object-graph-api`.
- Bound authenticated, idempotent entity upsert to tenant row-isolation, property tier labels, privacy-program data-class labels, and mutation-event evidence.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform Cedar policy publish API contract

- Added `cedar.policy.publish` OpenAPI/runtime/schema/catalog parity via `oya-platform-policy-cedar-api`.
- Bound authenticated, idempotent policy publication to path/body policy version, principal authorization evidence, semver supersession, and tenant/global Cedar scope.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform identity user upsert API contract

- Added `identity.user.upsert` OpenAPI/runtime/schema/catalog parity via `oya-platform-identity-api`.
- Bound authenticated, idempotent user upsert to path/body tenant and user identity, principal authorization evidence, per-tenant primary-identifier uniqueness, and regional IdP binding.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform tenant create API contract

- Added `tenant.create` OpenAPI/runtime/schema/catalog parity via `oya-platform-tenant-api`.
- Bound authenticated, idempotent tenant creation to path/body tenant identity, operator authorization evidence, global tenant-id uniqueness, and the tenant/residency kernels.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform identity token issue API contract

- Added `identity.token.issue` OpenAPI/runtime/schema/catalog parity via `oya-platform-identity-app`.
- Bound authenticated, purpose-parsed, idempotent STS token issue to the platform identity kernel while forbidding long-lived API keys.
- Mirrored the contract in SPEC, SaaS Platform PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Platform audit event emit contract

- Added `audit.event.emit` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-audit-chain-app`.
- Bound CloudEvents envelope validation, producer authorization, privacy-program data-class parsing, hash-chain append, and eventing outbox publication in typed tests.

## 2026-05-12 — Platform metering event ingest contract

- Added `metering.event.ingest` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-metering-app`.
- Bound CloudEvents envelope validation, producer authorization, plane/axis/unit/data-class parsing, metering kernel recording, and eventing outbox publication in typed tests.
- Mirrored the event contract in SPEC, SaaS Platform PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Platform eventing outbox publish contract

- Added `eventing.outbox.publish` AsyncAPI/Protobuf/runtime/catalog parity via `oya-platform-eventing-app`.
- Bound CloudEvents envelope validation, producer authorization, privacy-program data classes, regulatory packs, and idempotent outbox publication in typed tests.
- Mirrored the event contract in SPEC, SaaS Platform PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Cloud billing event ingest contract

- Added `cloud.billing.event.ingest` AsyncAPI/Protobuf/runtime/catalog parity via `oya-cloud-billing-app`.
- Bound CloudEvents envelope, producer authorization, idempotency fingerprinting, billing kernel ingest, platform metering, and eventing outbox publication in typed tests.
- Mirrored the event contract in SPEC, Cloud PRD, machine-readable contracts, and API semver metadata.

## 2026-05-12 — Cloud cell binding API contract

- Added `cloud.cell.bind` OpenAPI/runtime/schema/catalog parity via `oya-cloud-cell-app`.
- Bound authenticated, idempotent tenant cell assignment to tenant/principal/authorization evidence before `CloudRegionCatalog::bind_route_for_tenant`.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Cloud FinOps report API contract

- Added `cloud.finops.report` OpenAPI/runtime/schema/catalog parity via `oya-cloud-finops-api`.
- Bound authenticated, idempotent report generation to tenant/principal/authorization evidence before `CloudFinopsLedger::generate_report`.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

## 2026-05-12 — Cloud observability audit read API contract

- Added `cloud.observability.audit.read` OpenAPI/runtime/schema/catalog parity via `oya-cloud-observability-api`.
- Bound the authenticated audit-read request to tenant/principal/authorization evidence before kernel projection and exposed cursor/chain metadata in the success envelope.
- Mirrored the contract in SPEC, Cloud PRD, machine-readable contracts, and OpenAPI registries.

# Oyatie — Canonical Docs Changelog

> Per-commit log for `docs/`. Auto-emitted (per [DOC-CATALOG.md](DOC-CATALOG.md) `doc.changelog`).

---

## 2026-05-12 — Cloud billing invoice API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.billing.invoice.generate` to `contracts/openapi/cloud/cloud-billing-invoice-v1.yaml` and `oya-cloud-billing-tax-app`.

## 2026-05-12 — Cloud network load balancer API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.lb.create` to `contracts/openapi/cloud/cloud-network-lb-v1.yaml` and `oya-cloud-network-lb-api`.

## 2026-05-12 — Cloud network DNS API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.dns.zone.create` to `contracts/openapi/cloud/cloud-network-dns-v1.yaml` and `oya-cloud-network-dns-api`.

## 2026-05-12 — Cloud network VPC API contract

### Updated
- **SPEC.md**, **products/cloud/PRD.md**, and **machine-readable/contracts.json** — bound `cloud.network.vpc.create` to `contracts/openapi/cloud/cloud-network-vpc-v1.yaml` and `oya-cloud-network-vpc-api`.

## 2026-05-12 — Foundry capability record schema gate

### Updated
- **templates/capability-record-template.yaml** — split capability descriptions into agent/human MCP fields and aligned the template with the fail-closed capability-record schema gate.

## 2026-05-11 — OpenAPI 3.2 operation parity hardening

### Updated
- **standards/api-design.md** — documented OpenAPI 3.2 `QUERY` and `additionalOperations` governance, including fixed-method collision rules and runtime parity requirements.
- **MISTAKES-LEDGER.md** — added `MFL-0013` for the OpenAPI 3.2 operation traversal/runtime-parity prevention.

## 2026-05-11 — Flat-crates documentation consistency

### Updated
- **ADR-0015**, **ADR-INDEX.md**, and **machine-readable/decisions.json** — promoted architectural flattening to Accepted and aligned CI lane names with the live flat-crates guard.
- **DESIGN.md**, **ROADMAP.md**, **STANDARDS-AND-TEMPLATES.md**, **TOOLCHAIN.md**, **AGENTS.md**, and **teams/axis-foundry/CHARTER.md** — separated live 64-crate flat baseline from historical 89/91 split planning and retired legacy-root wording.
- **products/foundry/PRD.md**, **ADR-0020**, and **ADR-0022** — replaced current `services/agent/daemon` / `tools/repoctl` references with flat `crates/oya-*` and `crates/oya-tooling-cli-dev-runtime` bindings.
- **PRIVACY-PROGRAM.md**, **ADR-0008**, **ADR-0019**, **ADR-0025**, **GLOSSARY.md**, **checklists/pre-push.md**, and **templates/capability-record-template.yaml** — aligned lane names and catalog paths with the live flat-crates governance model.
- **CONSTITUTION.md**, **README.md**, **DOC-CATALOG.md**, **DOCUMENTATION.md**, ADR references, product PRDs, templates, and machine-readable batches — normalized canonical doc-tree references from retired consolidated-tree paths to the live `docs/` tree.

## 2026-05-11 — Foundry capability invoke ingress hardening

### Updated
- **SPEC.md** — `foundry.capability.invoke` status vocabulary now includes explicit `422` idempotency-conflict errors alongside `202`/`400`/`403`.

## 2026-05-11 — Flat-crates governance hardening

### Updated
- **MISTAKES-LEDGER.md** — added `MFL-0012` for legacy implementation-tree regression prevention.
- **standards/ci-lanes.md** — clarified flat-crates and catalog-record lane behavior.
- **runbooks/flat-crates-move-pr.md**, **runbooks/per-context-flatten-phase.md**, **runbooks/workspace-members-merge-queue.md** — replaced stubs with active ADR-0015 procedures.

## 2026-05-09 — initial consolidation

This is the founding consolidation, authored in one session as the project repositions from "Oyatie" → "Oyatie" and from a 5-axis vertical-cloud thesis to a 7-axis ecosystem-as-a-service behemoth.

### Created
- **README.md** — directory orientation
- **PRD.md** — product north star, 7 axes, optimal-path waves, anti-scope, success metrics, decision log
- **DESIGN.md** — cohesion thesis, planes, Foundry-as-accelerator (incl. multi-provider auth + in-house AI substrate + DC-ops sub-axis + Robotics/Vision/Speech sub-substrates + cloud trajectory + automation-first pipeline), per-axis bounded contexts, tenancy model, audit chain, Data Use Boundary, flattening, horizontal-scale primitives, cross-axis contract surface §10, contradiction audit §11, regional-pack architecture §12
- **PRIVACY-PROGRAM.md** — Data Use Boundary ADR draft (12 data classes + orthogonal subject_class + purpose-permission matrix + four-pillar matrix + KR-specific obligations + agent-runtime privacy + DSR cascade)
- **DOC-CATALOG.md** — protocol + catalog + 19 update-trigger events + per-doc owner+cadence+dependent-docs+validation-check
- **GLOSSARY.md** — industry-aligned vocabulary + Oyatie-specific terms with industry analogs + KR↔EN parity + 13-section structure
- **ADR-INDEX.md** — 127-ADR index + status counts + per-axis distribution + supersession chains + drift findings
- **TOOLCHAIN.md** — best-for-task language stack matrix + agent-specific toolchain + parallelization-first tools + MCP gateway (Section 4.A) + license manifest + investment sequence
- **DOCUMENTATION.md** — Diátaxis-aligned doc system + storage map + generation pipelines + DaaP norms
- **STANDARDS-AND-TEMPLATES.md** — catalog of templates / checklists / hooks / skills / tools / standards / requirements
- **COMPLIANCE-MATRIX.md** — regulator × control × evidence × cadence × owner across KR / JP / US / EU / IN / BR / KSA / UAE / AU / SG + cross-regional standards
- **RISK-REGISTER.md** — 27 scored risks (severity × likelihood × velocity) + 6 anti-risks + per-axis slice
- **CONTRADICTION-LEDGER.md** — LEDG-001..029 from Codex verdict + recon files + team-charter review
- **SECURITY-PROGRAM.md** — threat model + 12 controls + per-axis controls + continuous control monitoring
- **SLO-CATALOG.md** — per-surface SLOs + error-budget policy + burn-rate gates
- **RELEASE-MANAGEMENT.md** — trunk-based + release branch + CI lane catalog + progressive delivery + hotfix path + per-axis release exceptions
- **QA-TEST-STRATEGY.md** — test pyramid + required tests per change class + fixture discipline + flaky-test policy + coverage targets
- **INCIDENT-MANAGEMENT.md** — Sev taxonomy + roles + lifecycle + comms templates + drills + prevention loop
- **RACI-OWNERSHIP.md** — cross-axis ownership matrix + per-surface CODEOWNERS map + decision rights
- **ROADMAP.md** — wave list (Foundation → Foundry-Preview → Cloud-Preview / SaaS-Preview / Workspace-Preview / Search-Preview parallel → Vertical-Pilot → Vertical-Fan-Out → Cloud-Stable → Search-Stable → Ads-Preview → Ads-Stable → AI-Model-Substrate → DC-Operations → Region-Fan-Out)
- **ADR-CONSOLIDATION-PLAN.md** — strategy for consolidating 127 legacy ADRs into ~30-40 new ADRs
- **products/_TEMPLATE.md** + **products/README.md** — per-product PRD template
- **products/{saas-platform,foundry,cloud,search,ads-analytics,workspace}/PRD.md** — 6 axis PRDs (Foundry deepest at 852 lines)
- **products/vertical-{corporate,healthcare,industrial,logistics,fintech,legal}/PRD.md** — 6 deep vertical PRDs
- **products/vertical-{retail,education,public-sector,hospitality,construction,real-estate,agriculture,food}/PRD.md** — 8 skeleton vertical PRDs
- **teams/README.md + 37 team charters**
- **standards/fintech-compliance.md** — PCI-DSS v4.0 scope + per-jurisdiction overlays for Toss/KakaoBank/PayPal/Robinhood-class
- **templates/{adr-template.md, capability-record-template.yaml, runbook-template.md}**
- **checklists/{pre-push.md, wave-gate.md, foundry-capability-publishing.md}**
- **machine-readable/catalog.json** — initial machine-readable doc catalog

### Drafted ADRs
- `decisions/ADR-0013-product-license-policy.md` — defines product license policy; AGPL/GPL forbidden in product code (Apache-2/MIT/BSD/MPL-2 allowed)

### Direction changes integrated
1. Brand standardized as Oyatie (`oya-*` Cargo prefix)
2. 7 axes (SaaS / Workspace NEW / Vertical / Foundry / Cloud / Search / Ads)
3. Foundry consolidates Foundry engineering platform (originally separate axis)
4. Multi-provider Foundry: Anthropic Claude / OpenAI / Google Gemini × subscription + API
5. Canonical + regional-pack architecture (parallel global launch)
6. Multi-year structural cost-of-deferral horizon
7. In-house build preference + license-conscious posture
8. Architectural flattening (`crates/oya-<context>-<role>`)
9. M0/M1/M2/M3/MVP vocab retired → wave-named phases
10. Repoctl persona-split (`oya dev/admin/build/agent/ops/pack/catalog/gate`)
11. MCP gateway for agent-discoverable CLI
12. Workspace / Productivity Suite as Axis 2 (NEW)
13. In-house AI model training + inference (W-AI-Model-Substrate, long-horizon)
14. DCIM software for own DC ops (W-DataCenter-Operations)
15. Robotics / Vision / Speech intelligence sub-substrates
16. Compute trajectory: OCI + AWS now → Oyatie colo at scale → own greenfield mega-DC (DC-from-scratch back in scope)
17. Automation-first principle (Google + Amazon doctrine; highest yield in git/CI/CD/PR pipeline)
18. ADR consolidation directive (existing 127-ADR corpus → ~30-40 new ADRs)

### Consensus pass
- Codex critic verdict at `docs/raw/codex-verdict.md` — REQUEST CHANGES with 8 BLOCKERs + ~20 HIGH items; 6 of 8 BLOCKERs addressed in this consolidation; 2 partial (re-sequencing in PRD §3.1 needs further pass; Build-vs-Buy ADR drafted in TOOLCHAIN, formal ADR pending)

### v2 backlog
- `docs/raw/plan-v2-draft.md` — 1,847 leaves across P0-P20 in 110 batch tags; full schema per-leaf; covers all 7 axes + cross-cutting + contradiction-resolution + brand-rename + long-tail
