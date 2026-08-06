---
id: ADR-0131
status: Superseded
planning_impact: true
deciders: council-architecture, council-engineering, axis-foundry, ops-sre-reliability, ops-security
date: 2026-05-17
owner: council-architecture
supersedes:
  - "ADR-0015 (partial — supersedes the docs-vs-crates top-level split for per-service ownership; ADR-0015's BC and layer rules remain in force)"
amends:
  - "ADR-0119 (partial — product-owned specs colocate with their owner; /specs/ remains the flat root for cross-cutting specifications)"
superseded_by: [ADR-701]
amended_by: [ADR-0245, ADR-0333, ADR-0341, ADR-0512]
related: [ADR-0015, ADR-0056, ADR-0105, ADR-0110, ADR-0115, ADR-0116, ADR-0119, ADR-0122, ADR-0139, ADR-0512]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/masterplan.json, /specs/master-plan-sequencing.json]
bominal_source: no Bominal equivalent
purpose: Mandate one universal artifact layout — per-microservice flat folder containing PRD, phase specs, IPs, service-scoped ADRs, contracts, catalog, specs, runbooks, threat model, IaC, OpenSLO, crates, tests — for every µservice and every product oyatie ships, matching the convention used by AWS, Google, Microsoft, Oracle, and Stripe.
---

# ADR-0131: Per-microservice flat layout (universal)

## Status

Accepted — 2026-05-17.

**Amended — 2026-06-02 (pure split):** ADR-0512/platform-readiness updates the top-level service root from
`microservices/<ms>/` to `{oya,cloud}/<service>/`. The ADR-0131 colocation principle survives: service-owned docs,
contracts, specs, runbooks, SLOs, IaC, catalog records, and code travel with the service. The old `microservices/`
root is legacy only and must be removed after migration evidence proves every service has landed under `oya/` or
`cloud/` (or shared code under `libs/`).

**Amended — 2026-06-29 (review evidence retirement):** ADR-0515 and `docs/AGENTS.md` retire the standalone multispectrum evidence-file convention. The colocated `evidence/` slot remains available for typed quality-gate artifacts and audit bundles only; existing multispectrum paths are historical provenance and are not a current coverage or review-evidence requirement.

**Amended — 2026-07-13 (lifecycle metadata repair):** this ADR partially amends, rather than
supersedes, ADR-0119. ADR-0119's flat `specs/` rule remains binding for cross-cutting specifications;
this ADR only colocates product-owned specifications with their owning boundary. ADR-0562 as
amended by ADR-0615 controls the current destination topology.

## Context

oyatie's artifacts for a single µservice are currently scattered across seven type-based locations:

- `crates/oya-<ms>-<bc>-<layer>/` — code
- `docs/prds/<ms>.md` — PRD
- `docs/products/<product>/{PRD.md, PHASE-NN-*.md}` — product-shaped PRD + phase docs (parallel convention)
- `.omc/plans/milestones/M0X-<slug>/phases/P0Y-<slug>/IP-NNN-*.md` — phase + IP files nested four levels deep
- `registry/catalog/<crate>.yaml` — per-crate catalog records
- `/specs/<topic>.json` and `/specs/microservices/<product>.json` — machine-readable specs (mixed: cross-cutting and per-product side-by-side)
- `contracts/openapi/**/*.yaml`, `contracts/*.proto`, `contracts/asyncapi/*.yaml` — API contracts grouped by surface, not by owner

A reader (human or agent) tracing one µservice's narrative must visit at least six directories. The same µservice's PRD, contract, catalog row, and IP plan never share a parent path. This violates bounded-context cohesion (per `feedback_clean_architecture_requirements.md`) and the user-mandated bar "any of our practices should be hyperscaler grade, industry leading."

Public engineering practice at the canonical hyperscalers and one industry-leading SaaS contradicts oyatie's current scatter:

| Company | Convention | Source |
|---|---|---|
| **AWS** | Per-service folder containing Smithy IDL, source, docs, runbooks, integration tests, build config. PR/FAQ ("working backwards" 6-pager) and service-scoped ADRs live in the service folder. | AWS internal monorepo patterns; Smithy at `smithy-lang/smithy`; Lambda Powertools repo layout. |
| **Google** | `google3/<area>/<service>/{server,client,docs,BUILD}` with design docs in `docs/design/<title>.md` colocated with code. SLO definitions and runbooks travel with the service binary. | "Software Engineering at Google" (Winters, Manshreck, Wright, 2020); Google SRE Workbook ch. 5 + ch. 6; Borg / Monarch internals papers. |
| **Microsoft** | `services/<service>/{docs,src,tests,infra,pipelines,.ado}` with ADRs at `docs/decisions/` per service. | Microsoft Engineering Playbook (`microsoft/code-with-engineering-playbook`); Azure SDK repos. |
| **Oracle (OCI)** | `<service>/{api-specs,src,docs/{prfaq,design,runbook},terraform,tests}` with per-service PRFAQ. | OCI service-team repo template; public SDK repos. |
| **Stripe** | Monorepo per-service: `<service>/{lib,test,api,docs}`. Service-level docs (API, runbooks, SLOs) colocate with code; org-wide RFCs centralized in `docs/rfcs/`. | Stripe engineering blog 2018–2024; Sorbet repo (`sorbet/sorbet`); public Stripe SDKs. |

The common pattern: **per-service folder containing all artifacts that belong to that service; type-based central folders (`docs/prds/`, `registry/catalog/`, etc.) are the exception, reserved only for repo-wide cross-cutting items.**

oyatie's bar (per `feedback_quality_performance_scalability_bar.md`, `feedback_autonomous_decision_principles.md`, and explicit 2026-05-17 user directive) is hyperscaler-grade in every practice. The current scatter falls below that bar.

This ADR has no Bominal equivalent; Bominal's layout decisions inherit forward to oyatie only where applicable, and this is an oyatie originating structural decision.

## Decision

oyatie adopts one universal artifact layout: **flat colocated service folders under `{oya,cloud}/<service>/`**, mandatory for every service/product in the repo. `oya/` holds product/domain services; `cloud/` holds platform/tenant-substrate services; shared cross-cutting code remains under `libs/`. Sales segmentation remains a PRD-frontmatter field, not a directory split. Historical references to `microservices/<ms>/` in this ADR are legacy-path examples and must be read through this amendment.

### Canonical folder shape

For a service named `<service>` (kebab-case, per BNF v4.1 ADR-0056), choose `{tree}` as `oya` for product/domain services or `cloud` for platform services. The shape is **language-agnostic at the source root**: `src/` is mandatory where language source lives, and Rust bounded-context crates follow ADR-0512 under `crates/<crate>/` during the platform-readiness migration. Tooling treats the colocated service root as the canonical path-based ownership unit.

```text
{oya,cloud}/<service>/
  PRD.md                                              # the µservice's product requirements doc
  PRFAQ.md                                            # OPTIONAL — Amazon-style press-release / FAQ when authored
  PHASE-NN-<slug>.md                                  # phase specs; one per implementation phase
  IP-NNN-<slug>.md                                    # implementation plans; one ChangeSet per IP
  README.md                                           # entry point; quick-start, owner, status
  CODEOWNERS                                          # OPTIONAL — service-team RACI override
  decisions/
    ADR-####-<slug>.md                                # service-scoped ADRs only; cross-cutting ADRs stay at docs/decisions/
  contracts/
    openapi/<surface>.yaml                            # REST API contracts
    asyncapi/<surface>.yaml                           # async/event contracts
    proto/<surface>.proto                             # gRPC contracts
  specs/
    <topic>.json                                      # service-scoped machine-readable specs
  catalog/
    <crate-or-pkg>.yaml                               # per-crate / per-package catalog records; one file per code unit
  runbooks/
    <scenario>.md                                     # operational runbooks
  threat-model.md                                     # STRIDE threat model per docs/templates/threat-model-template.md
  slos/
    <sli>.openslo.yaml                                # OpenSLO manifests (per ADR-0139 SLO gate)
  dpia.md                                             # OPTIONAL — Data Protection Impact Assessment when regulated capability
  iac/
    helm/<chart>/                                     # Helm charts (Layer-A self-hosted infra)
    terraform/<module>/                               # Terraform modules
    kustomize/<overlay>/                              # Kustomize overlays
  src/                                                # MANDATORY; canonical code root for path-based ownership
    crates/                                           # Rust convention: one crate per layer per BC per ADR-0056 BNF v4.1
      oya-<ms>-<bc>-<layer>/
        src/                                          # Rust crate's own src/
        tests/
        Cargo.toml
    packages/                                         # TypeScript convention (when applicable): one package per concern
      <pkg>/
        src/
        package.json
    # Python convention (when applicable): src/<pkg>/ + pyproject.toml at microservice root
    # JVM convention (when applicable): src/main/{kotlin,java}/ + build.gradle.kts at microservice root
  tests/
    integration/                                      # cross-crate / cross-package integration tests
    e2e/                                              # end-to-end scenarios
    load/                                             # k6/vegeta load tests per PRD §"Performance Targets"
  evidence/
    quality-gates/                                  # typed cloud-ci/oya-ci gate artifacts; no new multispectrum evidence files
    audit/                                          # service-local audit bundles when the service owns them
```

The colocated service root is what lets the layout CI lane perform path-based change-ownership: a commit modifying `{oya,cloud}/<a>/**` is owned by `<a>` and may not be bundled with unrelated structural/service-root edits without an explicit packet. Co-located docs (PRD, PHASE, IP, runbooks) sit beside source so doc-only PRs can be classified without triggering unnecessary build pipelines.

### What stays central (cross-cutting only)

Five categories of artifact remain at repo-root locations, exclusively for items that are cross-cutting (govern more than one µservice or the repo as a whole). Authoring a per-service artifact at any of these locations is a CI violation under the new enforcement lane.

| Central location | Scope | What lives here |
|---|---|---|
| `docs/decisions/ADR-####-*.md` | Cross-cutting ADRs | Decisions that govern multiple µservices or the repo (ADR-0056 BNF v4.1; ADR-0105 layer enum; ADR-0110 ChangeSet state machine; ADR-0139 SLO gate; this ADR). Service-scoped ADRs move to `{oya,cloud}/<service>/decisions/`. |
| `docs/standards/<topic>.md` | Cross-cutting standards | Code style, commit message, API design, schema migration, error handling, observability/SLO authoring rules, etc. |
| `docs/templates/<artifact>.md` | Authoring templates | PRD template, ADR template, IP template, phase-spec template, runbook template, etc. |
| `/specs/<topic>.json` | Cross-cutting machine-readable specs | masterplan.json, master-plan-sequencing.json, hyperscaler-gates.json, per-microservice-flat-layout.json, agentic-slo-gated-promotion.json. Per-product specs (workflow.json, ontology.json, workflow-studio.json) move to `{oya,cloud}/<service>/specs/`. |
| `Cargo.toml` (workspace) + `.github/`, `.gitignore`, `CLAUDE.md`, `docs/AGENTS.md`, `docs/MASTERPLAN.md`, `docs/README.md` | Repo-wide infrastructure | Workspace manifest, CI workflows, contributor docs, root memory. Per-µservice CI workflow steps reference the µservice's IaC and tests via canonical relative paths. |

### What moves (every µservice and every product)

Every artifact currently scattered across the type-based folders **moves** into its owning µservice's folder. Concretely:

| Current location | Moves to |
|---|---|
| `crates/oya-<ms>-<bc>-<layer>/` | `{oya,cloud}/<service>/crates/oya-<ms>-<bc>-<layer>/` |
| `docs/prds/<ms>.md` | `{oya,cloud}/<service>/PRD.md` |
| `docs/products/<product>/PRD.md` | `{oya,cloud}/<service>/PRD.md` |
| `docs/products/<product>/PHASE-NN-*.md` | `{oya,cloud}/<service>/PHASE-NN-*.md` |
| `.omc/plans/milestones/M0X-*/phases/P0Y-*/IP-NNN-*.md` | `{oya,cloud}/<service>/IP-NNN-*.md` (denested; milestone+phase carried in IP frontmatter, not directory path) |
| `.omc/plans/milestones/M0X-*/phases/P0Y-*/README.md` | `{oya,cloud}/<service>/PHASE-NN-*.md` |
| `registry/catalog/<crate>.yaml` | `{oya,cloud}/<service>/catalog/<crate>.yaml` |
| `/specs/microservices/<product>.json` and per-product JSON specs | `{oya,cloud}/<service>/specs/<topic>.json` |
| `contracts/openapi/<surface>/*.yaml` (per-service) | `{oya,cloud}/<service>/contracts/openapi/<surface>.yaml` |
| `contracts/asyncapi/<surface>/*.yaml` (per-service) | `{oya,cloud}/<service>/contracts/asyncapi/<surface>.yaml` |
| `contracts/<surface>.proto` (per-service) | `{oya,cloud}/<service>/contracts/proto/<surface>.proto` |
| `docs/runbooks/<service>-*.md` | `{oya,cloud}/<service>/runbooks/<scenario>.md` |
| Threat models, DPIAs, OpenSLO manifests, IaC charts (where currently exist outside service roots) | `{oya,cloud}/<service>/threat-model.md`, `{oya,cloud}/<service>/dpia.md`, `{oya,cloud}/<service>/slos/`, `{oya,cloud}/<service>/iac/` |
| Service-scoped ADRs currently at `docs/decisions/` (rare) | `{oya,cloud}/<service>/decisions/ADR-####-*.md`, with a redirect stub at the old path (RETIRED.md row) |
| Historical multispectrum evidence formerly under `/evidence/multispectrum/<change_id>-*.json` | Historical provenance only; no migration creates `{oya,cloud}/<service>/evidence/multispectrum/**` as current coverage. New review/coverage evidence uses typed quality-gate artifacts, PR Code Review, and cloud-ci/oya-ci gate packets. |

The aggregation indices that previously lived as primary sources (`registry/catalog/`, `docs/prds/INDEX.md`, `/specs/microservices/`) become **generated views** sourced from the per-µservice folders. The generation lane is `oya-governance-aggregation-index-generation` (added by this ADR).

### Naming and roots — active 2026-06-02 amendment

The historical `microservices/` top-level name is no longer the destination. Active roots are:

```
SERVICE ROOTS: {oya,cloud}/<service>/
SHARED ROOT: libs/<lib>/
PACK ROOTS: packs/, regional-packs/ only for ADR-0010/ADR-0064-authorized pack artifacts
LEGACY: microservices/ is provenance/removal-candidate after verified migration
```

Rationale: `oya/` carries product/domain services, `cloud/` carries platform/tenant-substrate services, and `libs/` carries shared code. This preserves ADR-0131's colocation principle without creating a third service tree.

## Rejected alternatives

- **Hybrid (PRD per-service, phase+IPs in milestone tree)** — rejected. Splits one µservice's narrative across two locations; agents and humans still hunt across directories. Half-measure that contradicts the user mandate "do everything properly now."
- **Type-based scatter (current state)** — rejected. Documented above as the inverse of every industry-leading convention.
- **Product-vs-substrate split (`docs/products/<product>/` for products, `docs/prds/<ms>.md` for substrate)** — rejected. The user explicitly stated 2026-05-17 that observability isn't a hero product yet still requires hyperscaler-grade artifact handling; the product/substrate distinction does not earn a directory split. Sales segmentation moves entirely into PRD frontmatter (`sales_segment: <segment>`).
- **Per-microservice repos (one repo per µservice)** — rejected. Loses the monorepo benefits oyatie has already chosen: atomic cross-µservice refactor, shared workspace Cargo, single CI surface, single doc-cross-ref resolver. Per-service folder inside a monorepo is what Stripe and Google (google3) do; that is the precedent.
- **Migrate-on-touch (existing µservices migrate only when next touched)** — rejected. Contradicts the user mandate "nothing scheduled-for-distinct-tracked-work until later." Migration is in M01 scope, scheduled as a per-µservice IP series.

## Consequences

### Concrete file and crate changes

This ADR mandates a repo-wide restructure. The mechanical work is decomposed into one **migration IP per µservice/product** in the M01 scope. Each migration IP is one ChangeSet per ADR-0110 (claimable, verifiable, bundleable, promotable).

Migration IPs added to M01. **All migration IPs are independent and parallel-safe**: each touches only one µservice's owning paths. Serial order applies *within* a single µservice's migration (move + crate-rename + workspace-member update + cross-ref regen + cargo build), never across. The DAG below lists dependency ordering driven by which µservices' types other µservices import (substrate → product); independent µservices may execute concurrently.

### Migration cost quantification

Per the historical `/specs/microservice-migration-tooling.json` `cost_estimate` block (provenance only), now re-targeted to cloud-ci/Rust migration packets instead of new legacy-migration CLI authority:

| Migration class | Per-IP files moved | Per-IP refs updated | Per-IP wall time |
|---|---|---|---|
| Small µservice (1–3 crates, 1 PRD) | ~5 | ~15 | ≤ 30 s |
| Medium µservice (5–10 crates, multiple PRDs + catalog rows) | ~25 | ~100 | ≤ 2 min |
| Large µservice (foundry-runtime / foundry-supervisor / etc.; 5–8 crates) | ~50 | ~200 | ≤ 5 min |
| Governance bundle (~50 `oya-check-*` crates with name rename) | ~250 | ~1000 | ≤ 30 min (atomic ChangeSet) |

Sum across all 30 migration IPs at the M01 launch tier: ≈ 1500 files moved + 3500 cross-refs updated + ≈ 3h cumulative wall time. Migrations run in parallel per the DAG below; end-to-end M01 migration window ≈ 1 working day with parallel execution.

Migration DAG (concurrency tiers; all IPs within one tier run in parallel). Per ADR-0132 (product-platform-dissolution), prior grouping bundles `foundry`, `workflow`, `cloud`, Tenant/PBAC packaging (RBAC + ABAC), workspace productivity composition, and Tenant PBAC are dissolved into 36 flat µservices below — matching AWS / Google / Microsoft / Stripe ship-as-separate-service precedent.

```text
Tier 0 (substrate; no inter-µservice deps):
  IP-M01-MIGR-001  tenancy
  IP-M01-MIGR-002  ontology
  IP-M01-MIGR-012  audit-chain
  IP-M01-MIGR-013  cell
  IP-M01-MIGR-014  governance              (bundles all ~50 oya-check-* crates)

Tier 1a (substrate-base; depends on tier 0):
  IP-M01-MIGR-003a workflow-engine        (was: workflow; split per ADR-0132)
  IP-M01-MIGR-NEW-1 observability         (NEW; authored natively)
  IP-M01-MIGR-FND-1 foundry-providers     (vendor-adapter SDKs; no internal Foundry deps)
  IP-M01-MIGR-FND-4 foundry-evidence      (audit emission; no internal Foundry deps)
  IP-M01-MIGR-CLD-1 cloud-iac             (Helm/Terraform/Kustomize modules)

Tier 1b (substrate; depends on tier 1a):
  IP-M01-MIGR-FND-2 foundry-runtime       (depends on foundry-providers)
  IP-M01-MIGR-FND-5 foundry-guardrails    (depends on foundry-runtime)
  IP-M01-MIGR-CLD-2 cloud-k8s             (depends on cloud-iac)

Tier 1c (substrate; depends on tier 1b):
  IP-M01-MIGR-FND-3 foundry-supervisor    (depends on foundry-runtime)
  IP-M01-MIGR-FND-6 foundry-eval          (depends on foundry-runtime + foundry-evidence)
  IP-M01-MIGR-CLD-3 cloud-secrets         (OpenBao operator; depends on cloud-k8s)

Tier 2 (product host; depends on substrate):
  IP-M01-MIGR-008  application            (Application Shell)
  IP-M01-MIGR-003b workflow-studio        (visual-editor product; depends on workflow-engine)

Tier 3 (end-user / tenant-facing flat µservices; product-tier, unbundled per ADR-0132):
  IP-M01-MIGR-CONN-1  mail               ← mail service (deduplicated from legacy communications + workspace mail)
  IP-M01-MIGR-CONN-2  messenger          ← messenger service (deduplicated from legacy chat surfaces)
  IP-M01-MIGR-CONN-3  calendar           ← calendar service (deduplicated from legacy workspace surfaces)
  IP-M01-MIGR-CONN-4  community          ← community service (org-wide threads, Q&A, KB)
  IP-M01-MIGR-WS-1    docs               ← from workspace document surface
  IP-M01-MIGR-WS-2    sheets
  IP-M01-MIGR-WS-3    slides
  IP-M01-MIGR-WS-4    drive
  IP-M01-MIGR-WS-5    meet
  IP-M01-MIGR-WS-6    forms
  IP-M01-MIGR-WS-7    sites
  IP-M01-MIGR-WS-8    tasks
  IP-M01-MIGR-WS-9    notes
  IP-M01-MIGR-WS-10   translate
  IP-M01-MIGR-WS-11   recordings
  IP-M01-MIGR-ENT-1   hr                 ← unbundled from /specs/microservices/tenant-rbac.json
  IP-M01-MIGR-ENT-2   payroll
  IP-M01-MIGR-ENT-3   accounting
```

Total: 36 flat µservices, 30 migration IPs (six µservices ship natively, no migration). Compares against industry catalogs: AWS ~200+ services; Google Cloud ~100+; Azure ~200+; Stripe ~30; Linear ~8. 36 is mid-range for a multi-domain enterprise platform; each µservice is a meaningful unit of ownership, SLO, release pointer, and deploy cadence.

| Migration IP | µservice | Scope of move |
|---|---|---|
| IP-M01-MIGR-001 | `tenancy` | `docs/prds/tenancy.md` → `{oya,cloud}/tenancy/PRD.md`; `crates/oya-tenancy-*` → `{oya,cloud}/tenancy/crates/oya-tenancy-*`; contracts, catalog rows, runbooks, threat model. |
| IP-M01-MIGR-002 | `ontology` | `docs/prds/ontology.md`, `/specs/ontology.json`, related crates, contracts, catalog. |
| IP-M01-MIGR-012 | `audit-chain` | `crates/oya-audit-chain-*` family + any PRD/catalog/runbooks into `{oya,cloud}/audit-chain/`. |
| IP-M01-MIGR-013 | `cell` | `crates/oya-cell-*` family. |
| IP-M01-MIGR-014 | `governance` (NEW name; replaces `foundry-fitness` working name) | **Bundles all ~50 `oya-check-*` crates into one µservice.** Moves `crates/oya-check-<topic>/` → `{oya,cloud}/governance/crates/oya-check-<topic>/`. Crate renames to BNF v4.1 conformant `oya-governance-<topic>-<layer>` are staged inside this IP (atomic move+rename). The `governance` µservice owns the CI fitness lanes (architecture cohesion, supply-chain, license, ADR citation, doc coverage, naming conformance, SLO coverage, statelessness, shardability, etc.). Author single PRD describing the governance substrate; per-check evidence stays as lane output. |
| IP-M01-MIGR-003 | `workflow` | `docs/prds/workflow.md`, `/specs/workflow.json`, `/specs/microservices/workflow-studio.json`, related crates, contracts, catalog. |
| IP-M01-MIGR-NEW-1 | `observability` | NEW µservice; ships natively under this convention; no migration scope, only authoring. Cross-referenced from ADR-0139. |
| IP-M01-MIGR-008 | `application` | `docs/prds/application.md`, related crates, catalog. |
| IP-M01-MIGR-009 | `foundry` | Consolidate legacy foundry docs into intelligence/governance target service roots per ADR-0335/ADR-0363; related crates, catalog, contracts. |
| IP-M01-MIGR-010 | `cloud` | `docs/products/cloud/` → `cloud/<service>/`; related crates, catalog. |
| IP-M01-MIGR-CONN-1 | `mail` | Mail service; corporate + personal mail remain strictly separated by context; SMTP/IMAP; legal-hold/eDiscovery. Inherits dual-context (Personal / Professional) as a cross-cutting field, not a binding. ADR-0132 carries the dissolution rationale. |
| IP-M01-MIGR-CONN-2 | `messenger` | Messenger service; personal and professional conversations remain strictly separated by context. |
| IP-M01-MIGR-CONN-3 | `calendar` | Calendar service deduplicated from legacy workspace surfaces. |
| IP-M01-MIGR-CONN-4 | `community` | Community service for org-wide announcements, Q&A, and KB threads. |
| IP-M01-MIGR-WS-1 | `docs` | workspace productivity composition surface (per ADR-0029, now dissolved by ADR-0132). |
| IP-M01-MIGR-WS-2 | `sheets` | as above |
| IP-M01-MIGR-WS-3 | `slides` | as above |
| IP-M01-MIGR-WS-4 | `drive` | as above (storage / file management) |
| IP-M01-MIGR-WS-5 | `meet` | as above (video conferencing) |
| IP-M01-MIGR-WS-6 | `forms` | as above |
| IP-M01-MIGR-WS-7 | `sites` | as above (web publishing) |
| IP-M01-MIGR-WS-8 | `tasks` | as above |
| IP-M01-MIGR-WS-9 | `notes` | as above |
| IP-M01-MIGR-WS-10 | `translate` | as above |
| IP-M01-MIGR-WS-11 | `recordings` | as above |
| IP-M01-MIGR-ENT-1 | `hr` | `docs/prds/hr.md`, related crates, catalog. Suite wrapper `/specs/microservices/tenant-rbac.json` retires (dissolved by ADR-0132); `hr` becomes flat. |
| IP-M01-MIGR-ENT-2 | `payroll` | `docs/prds/payroll.md`, related crates, catalog. As above. |
| IP-M01-MIGR-ENT-3 | `accounting` | `docs/prds/accounting.md`, related crates, catalog. As above. |

Total: 25 migration IPs (was ~63 under the per-check-* one-folder-each interim). Per the `governance` bundle in IP-M01-MIGR-014, all ~50 `oya-check-*` crates ship in a single ChangeSet.

### Migration completion gate

> 2026-06-02 amendment: the active completion path below is read through the `{oya,cloud}/<service>/`
> root amendment. Historical `microservices/<ms>/...` examples in the original ADR are migration provenance, not the
> current destination.

A µservice's migration is "done" only when ALL of these are true:

1. `{oya,cloud}/<service>/PRD.md` exists with frontmatter conforming to the PRD template.
2. `{oya,cloud}/<service>/README.md` exists.
3. Every old-path artifact has been removed (no zombie in `docs/prds/<ms>.md`, `docs/products/<ms>/`, `crates/oya-<ms>-*`, `/specs/microservices/<ms>*`, `.omc/plans/milestones/M0X/phases/P0Y/*` for this µservice).
4. `Cargo.toml` `[workspace.members]` references the new `{oya,cloud}/<service>/crates/<crate>` paths.
5. `cargo build --workspace` exits 0.
6. `cargo nextest run --workspace` exits 0 (no test references the old paths).
7. The cloud-ci/Rust gate packet `cross-ref-validity` exits 0 (no broken links to old paths).
8. The cloud-ci/Rust gate packet `per-service-layout --service <service>` exits 0.
9. Aggregation indices regenerated; working tree matches.

Repo-wide migration is complete only when every µservice in the table above has passed its completion gate AND `find docs/prds/ docs/products/ -type f -name '*.md' | grep -v INDEX | grep -v README` is empty AND `find .omc/plans/milestones/ -type d -empty -delete` empties the milestone-plans tree.

### Strangler pattern for runtime refs

Per ADR-0139's per-microservice release pointers (`release/<ms>/<env>`): the existing tree-wide `staging` and `production` refs retain **read-only** status during migration. They no longer fast-forward; they are tagged at the pre-migration HEAD and frozen. Removal happens only after every µservice has live `release/<ms>/staging` and `release/<ms>/production` pointers and a successor-IP cleanup ADR retires the legacy refs.

Per ADR-0119 (specs flat root topology), per-product spec files at `/specs/microservices/<product>.json` and `/specs/microservices/<product>/` directories move under their owning µservice; the `/specs/` root retains only cross-cutting specs.

Per ADR-0115 (registry consolidation), `registry/catalog/<crate>.yaml` becomes a *generated aggregation*; per-service `{oya,cloud}/<service>/catalog/<crate>.yaml` is the source path for generated aggregation.

Per ADR-0116 (retire external agent-coordination tooling), the `.omc/plans/milestones/...` directory tree empties as IPs migrate. After all migration IPs complete, the `.omc/plans/` subtree retires; `.omc/state/sessions/` retains its current function as an OMC plugin state location and is unaffected by this ADR.

### Integration via Workflow + Ontology

Not applicable. This ADR governs repo structure; it does not produce or consume Workflow events or Ontology Object Types. Each migrated µservice's PRD continues to declare its Workflow + Ontology integration points per the PRD template.

### Positive

- One µservice = one folder = one bounded context, end-to-end. An agent or engineer opens `{oya,cloud}/<service>/` and sees the entire narrative: intent (PRD), execution (phase + IPs), interface (contracts), behaviour (crates, tests), operation (runbooks, SLOs, IaC).
- Industry-conventional. New engineers and agents familiar with AWS/Google/Microsoft/Oracle/Stripe layouts find oyatie immediately legible.
- Aggregation indices (`docs/prds/INDEX.md`, etc.) become generated, with the per-service folder as the authoritative source path after promotion; no more "is `docs/prds/foundry.md` or `docs/products/foundry/PRD.md` canonical?" ambiguity (an ambiguity the present repo state contains).
- Service-scoped ADRs live next to the µservice they govern, removing the inflation of the central ADR ledger.
- ChangeSet boundary becomes physical: an IP affecting only `{oya,cloud}/<service>/**` is provably scoped, and the seam-discipline lane can refuse cross-µservice changesets at the path level.

### Negative

- One-time migration cost is substantial: every existing µservice/product requires a migration IP. Estimated ~15 migration IPs (one per current µservice + ~50 for the `oya-check-*` family if not bundled). Each IP is small mechanically (move files, update workspace `members`, update cross-references) but the sweep touches the whole repo.
- Every cross-reference inside the repo (Markdown links, Rust `path =` deps, CI workflow paths, doc-catalog entries) updates exactly once. The cloud-ci/Rust `cross-ref-validity` gate catches broken links during the sweep; legacy local `oya` mirrors are migration evidence only until that gate is live.
- Git history for moved files requires `git mv` (or per-file `git log --follow`) to remain navigable. Per-µservice migration IPs use `git mv` exclusively for files, never `rm`+`add`.
- Some external tooling (IDE workspace plugins, doc-publishing pipelines, dashboards) may have hardcoded paths to `docs/prds/`, `crates/`, etc. These break and must be updated. The migration plan owns this fan-out.
- Three open questions remain (see §Open Questions) — none blocking; all resolved during the migration sweep.

### Operational

- New cloud-ci/Rust gate packet: **`per-service-layout`**. Refuses:
  - New service without a `{oya,cloud}/<service>/` folder.
  - Authoring of PRD / phase-spec / IP / catalog row / runbook / threat-model / OpenSLO manifest / service-scoped spec at any location *other* than the owning µservice's folder.
  - Crate creation outside `{oya,cloud}/<service>/crates/` or shared `libs/<lib>/`.
  - Adding a row to a central aggregation index by hand (must be generated).
- New cloud-ci/Rust gate packet: **`aggregation-index-generation`**. Regenerates `docs/prds/INDEX.md`, `registry/catalog/<crate>.yaml` aggregation, and equivalents from per-µservice sources, then asserts the working tree matches.
- Cloud-ci/Rust gate packet: **`cross-ref-validity`** (successor to the existing ADR-0117 / repo-hygiene lane) extends to validate the new path shape.
- Workspace `Cargo.toml` `[workspace.members]` list rewrites to use `{oya,cloud}/<service>/crates/<crate>` and `libs/<lib>` paths. The rewrite is atomic per migration IP.
- Templates under `docs/templates/` update to reference per-service paths (`{oya,cloud}/<service>/IP-NNN-*.md` etc.), retiring the stale `.omc/plans/milestones/...` pointers in `impl-plan-template.md` and `phase-spec-template.md`. The retirement is captured in this ADR's `related:` field and executed by a single template-rewrite IP in M01.

## Clean Architecture Impact

This ADR is structural; it does not move code between clean-arch layers and does not change the dependency direction or port-location rules of `feedback_clean_architecture_requirements.md`. The 13-layer enum per ADR-0105 remains the authority for crate suffixes.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none — crate-level direction unchanged |
| `cross-product-refusal` (LEAN-A2) | Not affected | none — path-level cross-product refusal is *strengthened* (now also enforced at directory level: `{oya,cloud}/<a>/crates/**` never imports `{oya,cloud}/<b>/crates/**` except via Workflow / Ontology adapters), as a side benefit |
| `port-location` | Not affected | none — ports still live in `<ms>-kernel` crates |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |
| `per-service-layout` (NEW, this ADR) | New BLOCKER gate on dev | enforces folder shape; refuses out-of-layout artifacts |
| `aggregation-index-generation` (NEW, this ADR) | New BLOCKER gate on dev | enforces generated indices match per-service sources |
| `cross-ref-validity` (existing per ADR-0117) | Extended | adds the new path shape to its validator |

Port traits, kernel/domain/application/adapter layering, and Workflow + Ontology adapter discipline are unchanged. This ADR moves files; it does not move responsibilities.

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Are `oya-check-<topic>` crates each their own µservice (one folder each, ~50 folders) or bundled under `{oya,cloud}/governance/checks/`? | council-architecture | resolved in IP-M01-MIGR-014 plan; default to one-folder-each pending objection. |
| 2 | Does `.github/workflows/*.yml` migrate per-µservice (one workflow file per µservice under `{oya,cloud}/<service>/.github/`) or stay repo-root? | ops-sre-reliability | repo-root for now; GitHub Actions requires workflows at `.github/workflows/` and ignores nested locations. ADR-#### successor-IP if per-service triggering becomes needed. |
| 3 | How do regional packs (`regional-packs/<pack>/PACK.md` per `STANDARDS-AND-TEMPLATES.md`) interact with this layout? | regional-packs team | regional packs span multiple µservices; treat as cross-cutting and retain top-level `regional-packs/` directory. ADR-#### if rule changes. |

## Verification

- cloud-ci/Rust gate packet `per-service-layout` — exit 0; no out-of-layout artifacts.
- cloud-ci/Rust gate packet `aggregation-index-generation` — exit 0; generated indices match working tree.
- cloud-ci/Rust gate packet `cross-ref-validity` — exit 0; no broken cross-references after each migration IP.
- `cargo build --workspace` — exit 0; workspace `Cargo.toml` `[workspace.members]` resolves correctly under new paths.
- `cargo nextest run --workspace` — exit 0; no test references the old paths.
- After all migration IPs complete: `find docs/prds/ docs/products/ -type f -name '*.md'` returns only `INDEX.md` (generated) and `README.md`; `find .omc/plans/milestones/ -type d` returns empty.

## References

- ADR-0015: Repo structure (precedes; this ADR supersedes its docs/crates split for per-service ownership).
- ADR-0056: BNF v4.1 crate naming (unchanged; remains the authority for `oya-<ms>-<bc>-<layer>` inside `{oya,cloud}/<service>/crates/`).
- ADR-0105: 13-layer enum (unchanged).
- ADR-0110: ChangeSet state machine (each migration IP is one ChangeSet).
- ADR-0115: Registry consolidation flat singular (this ADR extends — registry becomes generated aggregation).
- ADR-0116: Retire external agent-coordination tooling (`.omc/plans/` content migrates per this ADR; `.omc/state/sessions/` unaffected).
- ADR-0119: Specs flat root topology (this ADR refines — `/specs/` retains only cross-cutting specs).
- ADR-0122: Ontology crate rename (precedent for cross-cutting rename ADR).
- ADR-0139: Agentic SLO-gated promotion (consumes this ADR for the `{oya,cloud}/observability/` active layout; old `microservices/observability/` references are historical only).
- `feedback_clean_architecture_requirements.md` — bounded-context cohesion principle.
- `feedback_quality_performance_scalability_bar.md` — hyperscaler-grade bar.
- `feedback_autonomous_decision_principles.md` — "nothing scheduled-for-distinct-tracked-work" principle.
- Industry sources cited inline in §Context.
- Memory: `feedback_flat_product_catalog.md`, `feedback_glossary_shared_not_platform.md`.
- Issues: scaffold branch `oya-microservice-flat-layout-buildout-2026-05-17` (PR opened against `dev` per CLAUDE.md Wave-B bootstrap; ADR-0116 explains the temporary seam).
