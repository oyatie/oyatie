---
id: ADR-0131
status: Accepted
deciders: council-architecture, council-engineering, axis-foundry, ops-sre-reliability, ops-security
date: 2026-05-17
owner: council-architecture
supersedes:
  - "ADR-0015 (partial — supersedes the docs-vs-crates top-level split for per-service ownership; ADR-0015's BC and layer rules remain in force)"
  - "ADR-0119 (partial — supersedes the per-product slice of specs-flat-root; /specs/ retains only cross-cutting specs)"
superseded_by: []
related: [ADR-0015, ADR-0056, ADR-0105, ADR-0110, ADR-0115, ADR-0116, ADR-0119, ADR-0122, ADR-0139]
related_specs: [/specs/per-microservice-flat-layout.json, /specs/masterplan.json, /specs/master-plan-sequencing.json]
bominal_source: no Bominal equivalent
purpose: Mandate one universal artifact layout — per-microservice flat folder containing PRD, phase specs, IPs, service-scoped ADRs, contracts, catalog, specs, runbooks, threat model, IaC, OpenSLO, crates, tests — for every µservice and every product oyatie ships, matching the convention used by AWS, Google, Microsoft, Oracle, and Stripe.
---

# ADR-0131: Per-microservice flat layout (universal)

## Status

Accepted — 2026-05-17.

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

oyatie adopts one universal artifact layout: **per-microservice flat folder under a new top-level `microservices/` directory, mandatory for every µservice and every product in the repo.** The product-vs-substrate distinction collapses at the directory level; both shapes use the same folder structure. Sales segmentation remains a PRD-frontmatter field, not a directory split.

### Canonical folder shape

For a µservice named `<ms>` (kebab-case, per BNF v4.1 ADR-0056). The shape is **language-agnostic at the source root**: `src/` is mandatory and is the only place code lives, but its interior is chosen per language. Rust µservices use `src/crates/<crate>/` (matching BNF v4.1 layer split); TypeScript µservices use `src/packages/<pkg>/`; Python µservices use `src/<pkg>/` with `pyproject.toml`; mixed-language µservices have multiple top-level interior buckets. Tooling treats `src/` as the canonical code root for path-based ownership.

```text
microservices/<ms>/
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
    multispectrum/                                    # per-changeset multispectrum evidence (per docs/AGENTS.md §changeset)
```

The `src/` requirement is what lets the per-microservice-layout CI lane perform path-based change-ownership: a commit modifying `microservices/<a>/src/**` is owned by `<a>` and may not be bundled with a commit modifying `microservices/<b>/src/**`. Co-located docs (PRD, PHASE, IP, runbooks) sit *outside* `src/` so doc-only PRs don't trigger build pipelines.

### What stays central (cross-cutting only)

Five categories of artifact remain at repo-root locations, exclusively for items that are cross-cutting (govern more than one µservice or the repo as a whole). Authoring a per-service artifact at any of these locations is a CI violation under the new enforcement lane.

| Central location | Scope | What lives here |
|---|---|---|
| `docs/decisions/ADR-####-*.md` | Cross-cutting ADRs | Decisions that govern multiple µservices or the repo (ADR-0056 BNF v4.1; ADR-0105 layer enum; ADR-0110 ChangeSet state machine; ADR-0139 SLO gate; this ADR). Service-scoped ADRs move to `microservices/<ms>/decisions/`. |
| `docs/standards/<topic>.md` | Cross-cutting standards | Code style, commit message, API design, schema migration, error handling, observability/SLO authoring rules, etc. |
| `docs/templates/<artifact>.md` | Authoring templates | PRD template, ADR template, IP template, phase-spec template, runbook template, etc. |
| `/specs/<topic>.json` | Cross-cutting machine-readable specs | masterplan.json, master-plan-sequencing.json, hyperscaler-gates.json, per-microservice-flat-layout.json, agentic-slo-gated-promotion.json. Per-product specs (workflow.json, ontology.json, workflow-studio.json) move to `microservices/<ms>/specs/`. |
| `Cargo.toml` (workspace) + `.github/`, `.gitignore`, `CLAUDE.md`, `docs/AGENTS.md`, `docs/MASTERPLAN.md`, `docs/README.md` | Repo-wide infrastructure | Workspace manifest, CI workflows, contributor docs, root memory. Per-µservice CI workflow steps reference the µservice's IaC and tests via canonical relative paths. |

### What moves (every µservice and every product)

Every artifact currently scattered across the type-based folders **moves** into its owning µservice's folder. Concretely:

| Current location | Moves to |
|---|---|
| `crates/oya-<ms>-<bc>-<layer>/` | `microservices/<ms>/crates/oya-<ms>-<bc>-<layer>/` |
| `docs/prds/<ms>.md` | `microservices/<ms>/PRD.md` |
| `docs/products/<product>/PRD.md` | `microservices/<product>/PRD.md` |
| `docs/products/<product>/PHASE-NN-*.md` | `microservices/<product>/PHASE-NN-*.md` |
| `.omc/plans/milestones/M0X-*/phases/P0Y-*/IP-NNN-*.md` | `microservices/<ms>/IP-NNN-*.md` (denested; milestone+phase carried in IP frontmatter, not directory path) |
| `.omc/plans/milestones/M0X-*/phases/P0Y-*/README.md` | `microservices/<ms>/PHASE-NN-*.md` |
| `registry/catalog/<crate>.yaml` | `microservices/<ms>/catalog/<crate>.yaml` |
| `/specs/microservices/<product>.json` and per-product JSON specs | `microservices/<ms>/specs/<topic>.json` |
| `contracts/openapi/<surface>/*.yaml` (per-service) | `microservices/<ms>/contracts/openapi/<surface>.yaml` |
| `contracts/asyncapi/<surface>/*.yaml` (per-service) | `microservices/<ms>/contracts/asyncapi/<surface>.yaml` |
| `contracts/<surface>.proto` (per-service) | `microservices/<ms>/contracts/proto/<surface>.proto` |
| `docs/runbooks/<service>-*.md` | `microservices/<ms>/runbooks/<scenario>.md` |
| Threat models, DPIAs, OpenSLO manifests, IaC charts (where currently exist outside `microservices/`) | `microservices/<ms>/threat-model.md`, `microservices/<ms>/dpia.md`, `microservices/<ms>/slos/`, `microservices/<ms>/iac/` |
| Service-scoped ADRs currently at `docs/decisions/` (rare) | `microservices/<ms>/decisions/ADR-####-*.md`, with a redirect stub at the old path (RETIRED.md row) |
| Per-µservice multispectrum evidence currently at `/evidence/multispectrum/<change_id>-*.json` | `microservices/<ms>/evidence/multispectrum/<change_id>-*.json` |

The aggregation indices that previously lived as primary sources (`registry/catalog/`, `docs/prds/INDEX.md`, `/specs/microservices/`) become **generated views** sourced from the per-µservice folders. The generation lane is `oya-governance-aggregation-index-generation` (added by this ADR).

### Naming justification — the new top-level folder

```
NAME: microservices/
JUSTIFICATION:
- not a Rust crate; not subject to BNF v4.1 crate-name rule. Directory-naming convention only.
- "microservices" is the universal industry term across AWS/Google/Microsoft/Oracle/Stripe;
  see Context §"industry citation" above. Alternatives "services" (AWS internal) and "modules"
  (Google google3) were considered and rejected: "services" collides with `.github/services/`
  if introduced later; "modules" is overloaded with Rust's `mod` keyword.
- plural form matches the existing `crates/` (plural), `docs/` (plural), and `specs/` plural
  conventions in the repo.
- per `feedback_glossary_shared_not_platform.md` we deliberately avoid "platform" (retired);
  "microservices" carries no architectural baggage from the retired terminology.
- exemptions claimed: none.
```

Crate naming inside each `microservices/<ms>/crates/` subtree is unchanged: every crate continues to follow `oya-<microservice>[-<bc-tokens>]-<layer>` per ADR-0056 BNF v4.1, and the layer enum is the 13-value set per ADR-0105.

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

Per `/specs/microservice-migration-tooling.json` `cost_estimate` block, executed via `oya dev migrate-microservice`:

| Migration class | Per-IP files moved | Per-IP refs updated | Per-IP wall time |
|---|---|---|---|
| Small µservice (1–3 crates, 1 PRD) | ~5 | ~15 | ≤ 30 s |
| Medium µservice (5–10 crates, multiple PRDs + catalog rows) | ~25 | ~100 | ≤ 2 min |
| Large µservice (foundry-runtime / foundry-supervisor / etc.; 5–8 crates) | ~50 | ~200 | ≤ 5 min |
| Governance bundle (~50 `oya-check-*` crates with name rename) | ~250 | ~1000 | ≤ 30 min (atomic ChangeSet) |

Sum across all 30 migration IPs at the M01 launch tier: ≈ 1500 files moved + 3500 cross-refs updated + ≈ 3h cumulative wall time. Migrations run in parallel per the DAG below; end-to-end M01 migration window ≈ 1 working day with parallel execution.

Migration DAG (concurrency tiers; all IPs within one tier run in parallel). Per ADR-0132 (product-suite-dissolution), prior product bundles `foundry`, `workflow`, `cloud`, Connect Suite, Workspace Productivity Suite, Enterprise Suite are dissolved into 36 flat µservices below — matching AWS / Google / Microsoft / Stripe ship-as-separate-service precedent.

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
  IP-M01-MIGR-CONN-1  mail               ← from connect.mail + workspace.mail (dedup)
  IP-M01-MIGR-CONN-2  messenger          ← from connect.messenger + workspace.chat (dedup)
  IP-M01-MIGR-CONN-3  calendar           ← from connect.calendar + workspace.calendar (dedup)
  IP-M01-MIGR-CONN-4  community          ← from connect (org-wide threads, Q&A, KB)
  IP-M01-MIGR-WS-1    docs               ← from workspace.suite
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
  IP-M01-MIGR-ENT-1   hr                 ← unbundled from /specs/microservices/enterprise-suite.json
  IP-M01-MIGR-ENT-2   payroll
  IP-M01-MIGR-ENT-3   accounting
```

Total: 36 flat µservices, 30 migration IPs (six µservices ship natively, no migration). Compares against industry catalogs: AWS ~200+ services; Google Cloud ~100+; Azure ~200+; Stripe ~30; Linear ~8. 36 is mid-range for a multi-domain enterprise platform; each µservice is a meaningful unit of ownership, SLO, release pointer, and deploy cadence.

| Migration IP | µservice | Scope of move |
|---|---|---|
| IP-M01-MIGR-001 | `tenancy` | `docs/prds/tenancy.md` → `microservices/tenancy/PRD.md`; `crates/oya-tenancy-*` → `microservices/tenancy/src/crates/oya-tenancy-*`; contracts, catalog rows, runbooks, threat model. |
| IP-M01-MIGR-002 | `ontology` | `docs/prds/ontology.md`, `/specs/ontology.json`, related crates, contracts, catalog. |
| IP-M01-MIGR-012 | `audit-chain` | `crates/oya-audit-chain-*` family + any PRD/catalog/runbooks into `microservices/audit-chain/`. |
| IP-M01-MIGR-013 | `cell` | `crates/oya-cell-*` family. |
| IP-M01-MIGR-014 | `governance` (NEW name; replaces `foundry-fitness` working name) | **Bundles all ~50 `oya-check-*` crates into one µservice.** Moves `crates/oya-check-<topic>/` → `microservices/governance/src/crates/oya-check-<topic>/`. Crate renames to BNF v4.1 conformant `oya-governance-<topic>-<layer>` are staged inside this IP (atomic move+rename). The `governance` µservice owns the CI fitness lanes (architecture cohesion, supply-chain, license, ADR citation, doc coverage, naming conformance, SLO coverage, statelessness, shardability, etc.). Author single PRD describing the governance substrate; per-check evidence stays as lane output. |
| IP-M01-MIGR-003 | `workflow` | `docs/prds/workflow.md`, `/specs/workflow.json`, `/specs/microservices/workflow-studio.json`, related crates, contracts, catalog. |
| IP-M01-MIGR-NEW-1 | `observability` | NEW µservice; ships natively under this convention; no migration scope, only authoring. Cross-referenced from ADR-0139. |
| IP-M01-MIGR-008 | `application` | `docs/prds/application.md`, related crates, catalog. |
| IP-M01-MIGR-009 | `foundry` | Consolidate `docs/prds/foundry.md` + `docs/products/foundry/` into `microservices/foundry/`; related crates, catalog, contracts. |
| IP-M01-MIGR-010 | `cloud` | `docs/products/cloud/` → `microservices/cloud/`; related crates, catalog. |
| IP-M01-MIGR-CONN-1 | `mail` | From `connect.mail` + workspace `mail`; corporate + personal mail; SMTP/IMAP; legal-hold/eDiscovery. Inherits dual-context (Personal / Professional) as a cross-cutting field, not a binding. ADR-0132 carries the dissolution rationale. |
| IP-M01-MIGR-CONN-2 | `messenger` | From `connect.messenger` + workspace `chat` (deduplicated; same concept, two prior names). |
| IP-M01-MIGR-CONN-3 | `calendar` | From `connect.calendar` + workspace `calendar` (deduplicated). |
| IP-M01-MIGR-CONN-4 | `community` | From `connect` (org-wide announcements, Q&A, KB threads). |
| IP-M01-MIGR-WS-1 | `docs` | Workspace Productivity Suite surface (per ADR-0029, now dissolved by ADR-0132). |
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
| IP-M01-MIGR-ENT-1 | `hr` | `docs/prds/hr.md`, related crates, catalog. Suite wrapper `/specs/microservices/enterprise-suite.json` retires (dissolved by ADR-0132); `hr` becomes flat. |
| IP-M01-MIGR-ENT-2 | `payroll` | `docs/prds/payroll.md`, related crates, catalog. As above. |
| IP-M01-MIGR-ENT-3 | `accounting` | `docs/prds/accounting.md`, related crates, catalog. As above. |

Total: 25 migration IPs (was ~63 under the per-check-* one-folder-each interim). Per the `governance` bundle in IP-M01-MIGR-014, all ~50 `oya-check-*` crates ship in a single ChangeSet.

### Migration completion gate

A µservice's migration is "done" only when ALL of these are true:

1. `microservices/<ms>/PRD.md` exists with frontmatter conforming to the PRD template.
2. `microservices/<ms>/README.md` exists.
3. Every old-path artifact has been removed (no zombie in `docs/prds/<ms>.md`, `docs/products/<ms>/`, `crates/oya-<ms>-*`, `/specs/microservices/<ms>*`, `.omc/plans/milestones/M0X/phases/P0Y/*` for this µservice).
4. `Cargo.toml` `[workspace.members]` references the new `microservices/<ms>/src/crates/<crate>` paths.
5. `cargo build --workspace` exits 0.
6. `cargo nextest run --workspace` exits 0 (no test references the old paths).
7. `oya gate validate cross-ref-validity` exits 0 (no broken links to old paths).
8. `oya gate validate per-microservice-layout --microservice <ms>` exits 0.
9. Aggregation indices regenerated; working tree matches.

Repo-wide migration is complete only when every µservice in the table above has passed its completion gate AND `find docs/prds/ docs/products/ -type f -name '*.md' | grep -v INDEX | grep -v README` is empty AND `find .omc/plans/milestones/ -type d -empty -delete` empties the milestone-plans tree.

### Strangler pattern for runtime refs

Per ADR-0139's per-microservice release pointers (`release/<ms>/<env>`): the existing tree-wide `staging` and `production` refs retain **read-only** status during migration. They no longer fast-forward; they are tagged at the pre-migration HEAD and frozen. Removal happens only after every µservice has live `release/<ms>/staging` and `release/<ms>/production` pointers and a successor-IP cleanup ADR retires the legacy refs.

Per ADR-0119 (specs flat root topology), per-product spec files at `/specs/microservices/<product>.json` and `/specs/microservices/<product>/` directories move under their owning µservice; the `/specs/` root retains only cross-cutting specs.

Per ADR-0115 (registry consolidation), `registry/catalog/<crate>.yaml` becomes a *generated aggregation*; per-µservice `microservices/<ms>/catalog/<crate>.yaml` is the source of truth.

Per ADR-0116 (retire external agent-coordination tooling), the `.omc/plans/milestones/...` directory tree empties as IPs migrate. After all migration IPs complete, the `.omc/plans/` subtree retires; `.omc/state/sessions/` retains its current function as an OMC plugin state location and is unaffected by this ADR.

### Integration via Workflow + Ontology

Not applicable. This ADR governs repo structure; it does not produce or consume Workflow events or Ontology Object Types. Each migrated µservice's PRD continues to declare its Workflow + Ontology integration points per the PRD template.

### Positive

- One µservice = one folder = one bounded context, end-to-end. An agent or engineer opens `microservices/<ms>/` and sees the entire narrative: intent (PRD), execution (phase + IPs), interface (contracts), behaviour (crates, tests), operation (runbooks, SLOs, IaC).
- Industry-conventional. New engineers and agents familiar with AWS/Google/Microsoft/Oracle/Stripe layouts find oyatie immediately legible.
- Aggregation indices (`docs/prds/INDEX.md`, etc.) become generated, with the per-service folder as the single source of truth; no more "is `docs/prds/foundry.md` or `docs/products/foundry/PRD.md` canonical?" ambiguity (an ambiguity the present repo state contains).
- Service-scoped ADRs live next to the µservice they govern, removing the inflation of the central ADR ledger.
- ChangeSet boundary becomes physical: an IP affecting only `microservices/<ms>/**` is provably scoped, and the seam-discipline lane can refuse cross-µservice changesets at the path level.

### Negative

- One-time migration cost is substantial: every existing µservice/product requires a migration IP. Estimated ~15 migration IPs (one per current µservice + ~50 for the `oya-check-*` family if not bundled). Each IP is small mechanically (move files, update workspace `members`, update cross-references) but the sweep touches the whole repo.
- Every cross-reference inside the repo (Markdown links, Rust `path =` deps, CI workflow paths, doc-catalog entries) updates exactly once. The `oya-foundry-fitness-cross-ref-validity` lane catches broken links during the sweep; until it does, agents must run `oya gate validate cross-ref-validity` locally before pushing migration IPs.
- Git history for moved files requires `git mv` (or per-file `git log --follow`) to remain navigable. Per-µservice migration IPs use `git mv` exclusively for files, never `rm`+`add`.
- Some external tooling (IDE workspace plugins, doc-publishing pipelines, dashboards) may have hardcoded paths to `docs/prds/`, `crates/`, etc. These break and must be updated. The migration plan owns this fan-out.
- Three open questions remain (see §Open Questions) — none blocking; all resolved during the migration sweep.

### Operational

- New CI lane: **`oya-governance-per-microservice-layout`**. Refuses:
  - New µservice without a `microservices/<ms>/` folder.
  - Authoring of PRD / phase-spec / IP / catalog row / runbook / threat-model / OpenSLO manifest / service-scoped spec at any location *other* than the owning µservice's folder.
  - Crate creation outside `microservices/<ms>/crates/`.
  - Adding a row to a central aggregation index by hand (must be generated).
- New CI lane: **`oya-governance-aggregation-index-generation`**. Regenerates `docs/prds/INDEX.md`, `registry/catalog/<crate>.yaml` aggregation, and equivalents from per-µservice sources, then asserts the working tree matches.
- Lane: **`oya-foundry-fitness-cross-ref-validity`** (already exists per ADR-0117 / repo-hygiene) extends to validate the new path shape.
- Workspace `Cargo.toml` `[workspace.members]` list rewrites to use `microservices/<ms>/crates/<crate>` paths. The rewrite is atomic per migration IP.
- Templates under `docs/templates/` update to reference per-µservice paths (`microservices/<ms>/IP-NNN-*.md` etc.), retiring the stale `.omc/plans/milestones/...` pointers in `impl-plan-template.md` and `phase-spec-template.md`. The retirement is captured in this ADR's `related:` field and executed by a single template-rewrite IP in M01.

## Clean Architecture Impact

This ADR is structural; it does not move code between clean-arch layers and does not change the dependency direction or port-location rules of `feedback_clean_architecture_requirements.md`. The 13-layer enum per ADR-0105 remains the authority for crate suffixes.

| Lane | Impact | Action required |
|---|---|---|
| `dependency-direction` (LEAN-A1) | Not affected | none — crate-level direction unchanged |
| `cross-product-refusal` (LEAN-A2) | Not affected | none — path-level cross-product refusal is *strengthened* (now also enforced at directory level: `microservices/<a>/crates/**` never imports `microservices/<b>/crates/**` except via Workflow / Ontology adapters), as a side benefit |
| `port-location` | Not affected | none — ports still live in `<ms>-kernel` crates |
| `layer-correctness` | Not affected | none |
| `composition-root-only` | Not affected | none |
| `sdk-kernel-only` | Not affected | none |
| `per-microservice-layout` (NEW, this ADR) | New BLOCKER lane on dev | enforces folder shape; refuses out-of-layout artifacts |
| `aggregation-index-generation` (NEW, this ADR) | New BLOCKER lane on dev | enforces generated indices match per-µservice sources |
| `cross-ref-validity` (existing per ADR-0117) | Extended | adds the new path shape to its validator |

Port traits, kernel/domain/application/adapter layering, and Workflow + Ontology adapter discipline are unchanged. This ADR moves files; it does not move responsibilities.

## Open Questions

| # | Question | Owner | Target ADR / date |
|---|---|---|---|
| 1 | Are `oya-check-<topic>` crates each their own µservice (one folder each, ~50 folders) or bundled under `microservices/foundry/checks/`? | council-architecture | resolved in IP-M01-MIGR-014 plan; default to one-folder-each pending objection. |
| 2 | Does `.github/workflows/*.yml` migrate per-µservice (one workflow file per µservice under `microservices/<ms>/.github/`) or stay repo-root? | ops-sre-reliability | repo-root for now; GitHub Actions requires workflows at `.github/workflows/` and ignores nested locations. ADR-#### successor-IP if per-service triggering becomes needed. |
| 3 | How do regional packs (`regional-packs/<pack>/PACK.md` per `STANDARDS-AND-TEMPLATES.md`) interact with this layout? | regional-packs team | regional packs span multiple µservices; treat as cross-cutting and retain top-level `regional-packs/` directory. ADR-#### if rule changes. |

## Verification

- `cargo run -p oya-dev-cli -- gate validate per-microservice-layout` — exit 0; no out-of-layout artifacts.
- `cargo run -p oya-dev-cli -- gate validate aggregation-index-generation` — exit 0; generated indices match working tree.
- `cargo run -p oya-dev-cli -- gate validate cross-ref-validity` — exit 0; no broken cross-references after each migration IP.
- `cargo build --workspace` — exit 0; workspace `Cargo.toml` `[workspace.members]` resolves correctly under new paths.
- `cargo nextest run --workspace` — exit 0; no test references the old paths.
- After all migration IPs complete: `find docs/prds/ docs/products/ -type f -name '*.md'` returns only `INDEX.md` (generated) and `README.md`; `find .omc/plans/milestones/ -type d` returns empty.

## References

- ADR-0015: Repo structure (precedes; this ADR supersedes its docs/crates split for per-service ownership).
- ADR-0056: BNF v4.1 crate naming (unchanged; remains the authority for `oya-<ms>-<bc>-<layer>` inside `microservices/<ms>/crates/`).
- ADR-0105: 13-layer enum (unchanged).
- ADR-0110: ChangeSet state machine (each migration IP is one ChangeSet).
- ADR-0115: Registry consolidation flat singular (this ADR extends — registry becomes generated aggregation).
- ADR-0116: Retire external agent-coordination tooling (`.omc/plans/` content migrates per this ADR; `.omc/state/sessions/` unaffected).
- ADR-0119: Specs flat root topology (this ADR refines — `/specs/` retains only cross-cutting specs).
- ADR-0122: Ontology crate rename (precedent for cross-cutting rename ADR).
- ADR-0139: Agentic SLO-gated promotion (consumes this ADR for the `microservices/observability/` layout).
- `feedback_clean_architecture_requirements.md` — bounded-context cohesion principle.
- `feedback_quality_performance_scalability_bar.md` — hyperscaler-grade bar.
- `feedback_autonomous_decision_principles.md` — "nothing scheduled-for-distinct-tracked-work" principle.
- Industry sources cited inline in §Context.
- Memory: `feedback_flat_product_catalog.md`, `feedback_glossary_shared_not_platform.md`.
- Issues: scaffold branch `oya-microservice-flat-layout-buildout-2026-05-17` (PR opened against `dev` per CLAUDE.md Wave-B bootstrap; ADR-0116 explains the temporary seam).
