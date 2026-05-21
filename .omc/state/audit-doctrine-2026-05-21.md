# Audit Doctrine — Wave 15-corpus-audit (2026-05-21)

**Single canonical reference. Every audit subagent reads this once before any work. Owner: orchestrator session 8f603fc7. Status: ACTIVE.**

---

## §0. How to use this document

You are an audit codex or Claude agent. Before doing any work:

1. Read this entire doctrine document (this file).
2. Read the assignment block in your dispatch prompt.
3. Read your assigned µservice's manifest.json + PRD.md + ARCHITECTURE.md to ground yourself.
4. Read the canonical ADRs cited per §1 authority chain.
5. Apply the §3 three-tier audit method to each assigned µservice/artifact.
6. Write findings in the §8 findings format with §9 severity rating.
7. Honor §7 boundaries.
8. Run §10 verification gates before declaring complete.

If you find yourself unsure between two interpretations: this doctrine wins. If the doctrine is silent: defer to the §1 authority chain, then ask in a finding note rather than guess.

**This is a substantive audit. Reasoning depth must be xhigh. Output bespoke per-µservice findings — no template-stamping.**

---

## §1. Authority chain (precedence order, highest wins)

```
1. CURRENT-SESSION USER DIRECTIVES IN MEMORY
   (~/.claude/projects/-Users-jasonlee-oyatie/memory/feedback_*.md)
        ↓
2. KEYSTONE ADRs (2026-05-20 foundational doctrine bundle + Wave 15 doctrine)
   ADR-0242..0255 (14-ADR keystone bundle)
   ADR-0329 tier-system-retired
   ADR-0330 tenant_class + composable billing_components
   ADR-0331 cross-µservice tenant_class adoption template
   ADR-0332 healthcare-domain-decomposition
   ADR-0333 cell-µservice-retired-pattern-not-service
   ADR-0334 shorts-µservice-merged-into-social
   ADR-0335 foundry-µservice-retired-absorbed-by-intelligence
   ADR-0336 valkey-not-redis-substrate
        ↓
3. OTHER ADRs (chronological; newer wins unless explicit supersession)
        ↓
4. CANONICAL SPECS
   specs/master-plan-sequencing.json
   specs/microservices/<ms>.json
   specs/root-hub-pointers.json
   specs/manifests-index.json
        ↓
5. PER-µSERVICE MANIFEST + PRD + ARCHITECTURE
        ↓
6. IMPLEMENTATION (src/ + crates/)
        ↓
7. HISTORICAL DOCS + RETIRED ARTIFACTS
   RETIRED.md markers, archive dirs, old ADRs with status: Retired/Substantially-Rewritten
```

Conflicts resolve top-down. If memory says X and an ADR says Y, X wins because memory captures the most recent user directive.

**Special precedence rules:**
- **Cedar policy** wins over code claims when interpreting authorization decisions (per ADR-0243).
- **OpenSLO YAML** wins over markdown SLO prose (per ADR-0245).
- **manifest.json** wins over README markdown for substrate_dependencies and supported_oses.

---

## §2. Scope

### §2.1 In-scope µservices (77 active, post-retirements)

```
analytics, api-gateway, application, audit-chain, calendar, cloud-billing, cloud-billing-tax,
cloud-data, cloud-iac, cloud-iam, cloud-k8s, cloud-kms, cloud-network, cloud-network-dns,
cloud-secrets, cloud-storage, comms-email, community, compliance, connect, consent-graph,
contact-center, contract-lifecycle-management, crm, data-pipeline, data-warehouse,
design-collaboration, detection, developer-sdk, diagnostics, docs, drive, emergency, emr,
feature-flags, financial-planning, finops-portal, forms, global-trade, governance,
healthcare-integration, identity, imaging, incident-management, intelligence, itsm,
learning-management, mail, marketing-automation, marketplace, meet, messenger, notes,
observability, ontology, ops-dashboard-control-center, patient-monitoring, payments,
performance-management, pharmacy, plant-maintenance, plugin-app-store, production-planning,
quality-management, real-estate, recordings, sheets, sites, slides, social,
supply-chain-planning, tasks, tenancy, translate, treasury, warehouse, whiteboard,
workflow-engine, workflow-studio, workplace-integration
```

### §2.2 Retired µservices (SKIP — verify RETIRED.md present)

```
foundry → absorbed by intelligence per ADR-0335
network → absorbed by community per Wave 15K
cell → absorbed by tenancy + cloud-iac + observability per ADR-0333
shorts → absorbed by social per ADR-0334
```

If you find a retired µservice still has substantive non-RETIRED content, flag as P0.

### §2.3 Cross-corpus artifacts (full corpus audit)

- All 336+ ADRs at `docs/decisions/ADR-*.md`
- All specs at `specs/**/*.json` (master-plan-sequencing, microservices/<ms>.json, manifests-index, root-hub-pointers, etc.)
- All standards docs at `docs/standards/**/*.md`
- Canonical primitives at `tools/hooks/_canonical-primitives.md`
- Glossary at `docs/GLOSSARY.md` + `docs/machine-readable/glossary.json`
- Machine-readable doctrine at `docs/machine-readable/**/*.json`

### §2.4 Per-µservice artifact types (16 types per µservice)

```
1.  PRD.md
2.  ARCHITECTURE.md
3.  manifest.json
4.  IP-*.md (base IPs + journey IPs)
5.  contracts/openapi*.yaml (must be 3.2.0)
6.  contracts/asyncapi*.yaml (must be 3.1.0)
7.  contracts/*.proto (must be proto3)
8.  policy/*.cedar
9.  slos/*.openslo.yaml
10. iac/<context>/*.tf (OpenTofu, NOT Terraform)
11. capabilities/*.yaml
12. REMEDIATION-NOTES-*.md
13. README.md + onboarding/ + runbooks/
14. threat-model.md + dpia.md + compliance.md + benchmarks/ + migration-playbooks/
15. src/**.rs OR crates/oya-<ms>-*/src/**.rs (Rust code)
16. data-model + structural-map artifacts (data-model.md, ER diagrams, schema migrations under db/migrations/, entity catalog) — see §4.16
```

---

## §3. Three-tier audit method

For each assigned µservice, walk all three tiers in order. Do not skip a tier.

### Tier 1: STRUCTURAL (presence check)

For each of the 16 artifact types in §2.4, verify:
- Does it exist on disk?
- Is the file non-empty (≥10 lines for prose; ≥1 valid record for JSON/YAML)?
- Does its frontmatter declare the expected doc_class / template_id?
- Does it pass syntax validation (`yq empty` / `jq empty` / proto3 `protoc --lint` / cedar `cedar validate`)?

Missing artifact = P0 (CA-VERIFY classification: RED if structural).

### Tier 2: COHERENCE (cross-artifact alignment)

Compare claims across artifacts. Drift = finding.

Examples:
- PRD §C user stories must map to IPs (each user story → at least one IP slice)
- IPs must reference real crate names that exist in `crates/`
- manifest.json `substrate_dependencies` must match the actual deps in src/Cargo.toml
- Cedar entity types referenced in IPs must be declared in policy/*.cedar
- OpenAPI endpoints declared in PRD/ARCH must appear in contracts/openapi*.yaml
- SLO targets in PRD must appear in slos/*.openslo.yaml
- supported_oses in manifest.json must have a per-OS CI lane
- tenant_class adoption (per ADR-0330/0331) must be consistent across PRD + Cedar + manifest + contracts

Coherence drift = P0 if user-visible (e.g. contract mismatch) / P1 if internal.

### Tier 3: SUBSTANCE (intern-test)

For each µservice, simulate a day-1-ship intern (see §5). Ask:
- Can the intern read the docs in <2h and form a coherent mental model?
- Can they run `cargo check -p oya-<ms>-*` clean?
- Can they implement one IP slice end-to-end without asking "what does X mean here?"
- Can they open a PR that passes CI lanes?

For each unanswerable question, file a finding.

Substance gap = P1 (high-friction onboarding) / P2 (style/polish).

---

## §4. Per-artifact-type audit checklists

### §4.1 PRD.md

**Required sections** (per `templates/PRD.md`):
- Frontmatter: doc_class=PRD, template_id=TPL-PRD, prd_id, microservice, status (Accepted/Proposed/Draft), milestone_first_ship, related_adrs (array), related_specs, owner_team, date
- §Purpose: who-what-why in 3-5 paragraphs; reference top-3 counterparts
- §Tenant Value: per-capability tenant value; reference tenant_class (demo_trial, paid) + billing_components (per ADR-0330)
- §Functional Requirements: FR-X1..FR-Xn cross-cutting invariants + per-BC FR matrix
- §Non-Functional Requirements: Performance (p99 latency targets) + Security (Cedar/mTLS/SPIFFE) + Audit+Compliance + Availability+SLO + Data residency
- §Bounded Contexts: per ADR-0137 (each BC owns contracts + Cedar + kernels)
- §Data residency: per ADR-0117 tenant jurisdiction_code
- §Counterparts: top-3 anchor products + parity-matrix reference
- §Risks: explicit list with mitigations

**P0 if:** missing entire PRD; PRD is stamped boilerplate; tenant_class adoption missing; Bronze/Silver/Gold/Platinum tier vocabulary present (ADR-0329); claims contradict ADRs at higher precedence

### §4.2 ARCHITECTURE.md

**Required:**
- Layer enum declared per ADR-0105 13-layer canonical (kernel/domain/usecase/adapter/api/app/infrastructure/cli/rest/grpc/graphql/sdk/worker)
- Per-BC architecture diagram (ASCII or PNG with `.alt` text)
- Data flow: inward-only per ADR-0105 §D-1
- Cell topology per ADR-0248 (Tier-0..Tier-3 placement)
- Provider-agnostic + multi-context (AWS-guest / OCI-guest / on-prem / colo / Oyatie-cloud-provider) per `feedback_multi_context_provider_agnostic_2026_05_20`
- HTTP/3+QUIC default per ADR-0253
- HLC default; TrueTime opt-in per ADR-0252
- Cell affinity + shuffle sharding references per oya-shuffle-sharding crate

**P0 if:** declares old 12-layer enum; outward dependencies; missing layer; missing cell topology

### §4.3 manifest.json

**Required keys:**
- microservice (name matches dir)
- bounded_contexts (array, each with name + crates + contracts paths)
- substrate_dependencies (must list: valkey NOT redis per ADR-0336; postgres; opentofu; cedar)
- supported_oses (must list at least: talos, rhel, oracle-linux, ubuntu-lts, debian, rocky, almalinux, centos-stream, amazon-linux, flatcar, photon, macos-apple-silicon-m5-plus — per `feedback_os_support_matrix_2026_05_20`)
- deployment_contexts (must list all 5: aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider — per `feedback_multi_context_provider_agnostic_2026_05_20`)
- tenant_class_eligibility (demo_trial / paid; per ADR-0330)
- paid_billing_components_emitted (subset of revenue_share, per_seat, per_usage; per ADR-0330)
- related_adrs (array)
- compliance_packs (per ADR-0251 — HIPAA/GDPR/SOC2/CSAP/PCI/EU-AI-Act etc. that apply)
- absorbed_microservices (array — if this µservice absorbed retired µservices per ADR-0333/0334/0335)
- JSON valid (`jq empty` passes)

**P0 if:** substrate_dependencies missing or lists redis; supported_oses missing OS; deployment_contexts incomplete; tenant_class_eligibility missing

### §4.4 IP-*.md

**Required sections** (Wave 15-IP-substance bar):
- §A Problem: bespoke description of the specific gap THIS IP closes — not generic
- §B Approach: technical mechanism (algorithm/data structure/API/Cedar/Terraform) specific to µservice's actual code
- §C Deliverables: specific files/types/functions/contracts to be created or modified — references real paths
- §D Implementation: 5+ ordered concrete steps; each verifiable; references real files/types
- §E Acceptance: bespoke verifications referencing real test files / contract endpoints / Cedar rules / SLO docs
- §F Evidence: actual links to counterparts, ADRs, prior IPs, or external benchmarks
- §G Counterparts: explicit Stripe/Salesforce/Snowflake/etc. comparison row(s)

**P0 if:** stamped shell (line count 30-79 with identical labels across many IPs); no counterpart references; hallucinated artifacts (declared crates/types that don't exist anywhere); template-loop in §C/§D/§E with 30+ rows differing only in 1-2 token substitutions

### §4.5 contracts/openapi*.yaml

- Must declare `openapi: 3.2.0` (NOT 3.0.0 / 3.1.0 / 3.3.0 — per canonical primitives)
- Every endpoint must have operationId + request/response schemas
- Every schema must have description
- security must be declared per endpoint
- servers must include all 5 deployment_contexts

**P0 if:** wrong version; missing operationId; endpoints undocumented

### §4.6 contracts/asyncapi*.yaml

- Must declare `asyncapi: 3.1.0` (NOT 2.x / 3.0.0)
- Every channel must declare messages with payload schema
- Bindings declared for transport (Kafka per ADR-0050 / NATS / etc.)

**P0 if:** wrong version; missing payload schemas

### §4.7 contracts/*.proto

- proto3 syntax (NOT proto2)
- Every service declares RPCs with request + response messages
- Every message has field tags + types
- Reserved tags marked for retired fields

**P0 if:** proto2 syntax; missing tags

### §4.8 policy/*.cedar

- Declare entity types up front
- Declare action types
- Permit / Forbid rules cite entity types + actions
- Cross-reference Cedar rules from IPs

**P0 if:** entity types referenced in IPs not declared here; syntax error (`cedar validate` fails)

### §4.9 slos/*.openslo.yaml

- OpenSLO 1.0 schema
- objective per critical user journey (latency p50/p99; availability)
- error budget declared
- SLO indicators reference real metric names (must be emitted by code)

**P0 if:** missing SLO for a critical user journey

### §4.10 iac/<context>/*.tf

- OpenTofu (NOT Terraform) per `feedback_zero_handroll_opentofu_only_2026_05_20`
- One subdir per deployment_context (aws-guest, oci-guest, on-prem, colo, oyatie-as-cloud-provider, plus oci-guest/always-free per `feedback_oci_always_free_maximization_2026_05_20`)
- Modules signed (cosign signature present)
- No hardcoded secrets — secrets resolved via cloud-secrets OpenBao

**P0 if:** filename is `terraform-*.tf` instead of OpenTofu module; hardcoded secrets; missing deployment_context

### §4.11 capabilities/*.yaml

- Per-capability declaration with capability_id + autonomy_tier per ADR-0022
- tenant_class_eligibility per capability
- compliance_packs gates per capability
- Cedar entity reference per capability

**P0 if:** Bronze/Silver/Gold/Platinum tier reference in capability metadata; missing autonomy_tier

### §4.12 REMEDIATION-NOTES-*.md

- One per remediation wave
- Append-only (do not delete prior remediation history)
- Each section dated + cites wave name
- Lists files modified + decisions made + follow-ups

**P0 if:** missing for a µservice that was rewritten

### §4.13 README + onboarding + runbooks

**README.md:**
- 30-100 lines (concise)
- Links: PRD, ARCH, manifest, contracts, runbooks
- One-paragraph µservice purpose
- Day-1 quick-start command (`./bin/oya <ms> <command>` or `cargo run -p oya-<ms>-*`)

**onboarding/**:
- New-engineer first-week guide
- Reading order for docs
- Local dev setup commands
- Common pitfalls

**runbooks/**:
- One file per on-call alert
- Steps to triage + escalate + remediate
- Reference Grafana dashboards + log queries

**P0 if:** missing onboarding for substrate µservices; runbooks for declared SLO alerts missing

### §4.14 threat-model + dpia + compliance + benchmarks + migration-playbooks

**threat-model.md:** STRIDE per BC + Cedar policy boundary diagram + key adversary classes
**dpia.md:** Per Article 35 GDPR data protection impact assessment (if applicable)
**compliance.md:** Per compliance_pack (HIPAA / SOC2 / EU-AI-Act / etc.) mapping
**benchmarks/**: Real performance numbers; cite measurement methodology
**migration-playbooks/**: from-<counterpart>.md for each top-3 counterpart

**P0 if:** missing for µservices declared in compliance_packs; benchmarks are stub

### §4.15 src/**.rs OR crates/oya-<ms>-*/src/**.rs

- Rust strict-only per `feedback_rust_strict_only_no_python_2026_05_20`
- Each crate maps to one layer of ADR-0105 13-layer enum (kernel/domain/usecase/adapter/api/app/infrastructure/cli/rest/grpc/graphql/sdk/worker)
- Inward-only dependencies (kernel doesn't import app)
- Cedar entities used in code must be declared in policy/
- `cargo check -p oya-<ms>-*` PASS

**P0 if:** non-Rust backend code present (Python/JS/Ruby/etc.); outward deps; cargo check fails

### §4.16 Data model + structural map (NEW per user directive 2026-05-21)

**Every µservice must have a data model + structural map artifact:**

- `data-model.md` OR `docs/data-model.md` — ER diagram + entity table + relationships + cardinalities + ownership
- Schema migrations under `db/migrations/<NNNN>-<name>.sql` (one file per change; never edit shipped migrations; only add forward migrations)
- Entity catalog: each entity declares name + bounded_context_owner + primary_key + foreign_keys + tenant_scoping (per ADR-0244) + retention_class
- Structural map: bounded contexts ↔ crates ↔ contracts ↔ Cedar entities (cross-reference table)
- ADR-0099 data-class-registry mapping per entity (which data class applies: tenant_data, oyatie_data, public, system, etc.)

**Required entries:**

```
| Entity | BC owner | PK | FKs | tenant_scoping | data_class | retention | Cedar entity |
|---|---|---|---|---|---|---|---|
| Invoice | invoicing | uuid | tenant_id, customer_id | tenant_scoped | tenant_data | 7y | Invoice |
```

**Structural map example:**

```
BC: invoicing
├── Crates: oya-cloud-billing-domain (entities), oya-cloud-billing-kernel (logic),
│           oya-cloud-billing-tax-app (composition)
├── Contracts: contracts/openapi-invoicing.yaml + asyncapi-billing-events.yaml
├── Cedar:    policy/invoicing-authz.cedar
├── DB:       db/migrations/0001_invoice.sql .. 0017_revenue_recognition.sql
├── Audit:    InvoiceIssued / InvoiceVoided / ChargeCompleted / RefundIssued
└── SLOs:     slos/invoicing-p99-latency.openslo.yaml
```

**P0 if:**
- No data-model.md or equivalent
- Entity table missing tenant_scoping or data_class
- Migrations missing per declared entity
- Structural map missing bounded_context ↔ crate ↔ contract chain
- Data class not declared per ADR-0099

**P1 if:**
- ER diagram absent (entity table only)
- Migrations ungrouped by BC
- Cedar entity name doesn't match the Rust type name

---

## §5. Intern-test definition + acceptance bar

**Intern profile (Day-1 ship-a-slice bar):**

- Mid-level engineer
- 6 months Rust experience
- No prior Oyatie context
- Has read `tools/hooks/_canonical-primitives.md` once
- Has access to the µservice's docs + src + contracts + Cedar

**Bar (cumulative — all must pass):**

1. **Read in <2h** — µservice docs (PRD + ARCH + 3-5 representative IPs + manifest + README) ⇒ coherent mental model
2. **Build clean** — `cargo check -p oya-<ms>-*` PASS without dep fixes
3. **Test clean** — `cargo test -p oya-<ms>-*` PASS
4. **Implement one slice** — pick one IP, follow §D Implementation steps, ship the slice (commit + push) with TDD red→green→refactor
5. **PR passes CI** — without help on what the system means
6. **Onboarding answers** — common questions answered by docs (NOT by asking a human)

**Acceptance question for the auditor:**
> "If a competent mid-level Rust engineer joined the team Monday morning and was assigned this µservice's IP-007, could they have a green PR open by Friday EOD without asking me a single 'what does this mean' question?"

If YES across the µservice: substance bar passes.
If NO: file P1 findings on the specific friction points.

---

## §6. Cross-artifact coherence rules

Check these pairwise alignments. Drift = finding.

| Source | Target | Coherence rule |
|---|---|---|
| PRD §FR | IPs | every FR-X maps to at least one IP slice (cross-reference) |
| PRD §counterparts | manifest competitor_anchors | exact match |
| PRD §Bounded Contexts | manifest bounded_contexts | exact match |
| ARCHITECTURE.md layers | src/ subdir layout | layers in ARCH appear as crates/dirs in src |
| ARCH layer enum | ADR-0105 | matches 13-layer canonical |
| manifest substrate_deps | src/Cargo.toml | every substrate dep in manifest has a Cargo dep |
| manifest substrate_deps | iac/<context>/ | every substrate dep has IaC module |
| IPs §G counterparts | manifest competitor_anchors | each IP counterpart in manifest's anchor list |
| IPs §C deliverables | src/ + contracts/ + policy/ | every declared artifact exists or is in IMPL-truth-up scaffold list |
| Cedar entity types | IP §B/§D references | entities in Cedar = entities cited in IPs |
| Cedar entity types | data-model.md entities | match (entity == Cedar entity) |
| OpenAPI endpoints | PRD §FR API surface | every public FR has OpenAPI endpoint |
| AsyncAPI channels | IP §C events | every emitted event has AsyncAPI channel |
| OpenSLO objectives | PRD §NFR SLO | match latency targets |
| manifest tenant_class | Cedar tenant_class context | match |
| manifest deployment_contexts | iac/<context>/ dirs | every context has dir |
| manifest supported_oses | CI lanes | every OS has lane |
| data-model entities | contract schemas | every entity exposed via API has OpenAPI schema |
| migration files | data-model entities | every entity has at least one migration |

For each rule, write a coherence finding when violated. Severity: see §9.

---

## §7. Boundaries

### Always do
- Read this doctrine before any work
- Cite specific file:line in every finding
- Run `cargo check` / `jq empty` / `yq empty` / `cedar validate` for syntax verifications
- Append to REMEDIATION-NOTES rather than overwriting
- Preserve historical content with cross-reference rather than deleting
- Use `oya git` for git ops (per canonical primitives)

### Ask first
- Removing entire artifact files (vs scrubbing)
- Renaming crates that have downstream dependencies
- Changing OpenAPI/AsyncAPI/proto3 versions
- Adding new ADR (must be Wave-N coordinated)
- Touching Cedar policy in ways that change authorization decisions

### Never do
- Delete RETIRED.md markers (foundry/network/cell/shorts must keep their RETIRED.md)
- Scaffold artifacts you can't ground in real source/contract evidence (no hallucination)
- Bypass workspace Cargo.toml (every new crate must be a workspace member)
- Skip hooks with --no-verify
- Make destructive git operations (reset --hard / force push) outside your assigned worktree
- Use `unimplemented!()` / `todo!()` that compiles but panics at runtime (use `#[allow(dead_code)]` minimal stubs instead)
- Edit memory files outside your scope (memory updates go to feedback_<topic>_<date>.md and MEMORY.md index)

---

## §8. Findings format

Every finding follows this format:

```
F-<MS-PREFIX>-<NNN> | <severity P0/P1/P2/INFO> | <one-line summary>
  Tier: <STRUCTURAL|COHERENCE|SUBSTANCE>
  Artifact type: <one of §2.4 16 types>
  Evidence: <file_path:line_number>, <file_path:line_number>, ...
  Authority: <ADR-NNNN §X-Y> | <memory:feedback_*.md> | <this doctrine §N>
  Recommendation: <specific action — scaffold X / edit Y / delete Z / cross-reference W>
  Fix-IP-candidate: <IP-NNN title if a remediation IP should be filed>
```

Example:

```
F-CRM-014 | P0 | Stamped competitor-parity-matrix with 327 identical rows
  Tier: SUBSTANCE
  Artifact type: §4.14 benchmarks (competitor-parity-matrix.md)
  Evidence: microservices/crm/competitor-parity-matrix.md:1-327
  Authority: ADR-0324 anti-template-stamping; this doctrine §3 Tier 3 + §4.4 IP-*.md rules
  Recommendation: Discard stamped content; author 8-15 bespoke comparison rows referencing Salesforce + HubSpot + Microsoft Dynamics 365 + Pipedrive + Zoho actual capability claims; cross-link to specific PRD §FR / IP / contract endpoints.
  Fix-IP-candidate: IP-CRM-COMPARISON-MATRIX-REWRITE-2026-05-21
```

---

## §9. Severity

### P0 (BLOCKER)
- Structural: required artifact missing entirely
- Coherence: user-visible contract drift (OpenAPI endpoint claims schema A, code returns B)
- Substance: template stamping detected; tier vocabulary present (ADR-0329); kernel-ahead-of-spec; tenant_class missing
- Authority: violates a keystone ADR (0242-0255 / 0329-0336)
- Hyperscaler: SLO target missing for critical user journey; supported_oses incomplete
- Data model: no data-model.md; entity not in catalog; data_class not declared

**P0 must be remediated before µservice can promote past dev.**

### P1 (HIGH)
- Coherence: internal drift (manifest bounded_contexts diverges from src/ but contracts/ still match)
- Substance: intern-test fails on >1 friction point per µservice
- Style: ADR cross-references stale (cite retired ADRs)
- Maintainability: 24-month risk (eg. concrete bare-metal IP details that won't survive provider rotation)

**P1 must be remediated before µservice can promote past staging.**

### P2 (NORMAL)
- Style: docs ergonomics (missing TOC, inconsistent heading style)
- Polish: ER diagram absent but entity table present
- Minor coherence (Cedar action name vs IP action verb differ by 1 word)

**P2 can be batched into next quarterly remediation cycle.**

### INFO (no fix required)
- Observation: a stylistic choice that the auditor noticed but is not actionable
- Historical: documented preservation rationale

---

## §10. Verification gates

Before declaring complete:

1. ✅ All 16 artifact types reviewed for assigned µservice
2. ✅ All 3 tiers walked (Structural → Coherence → Substance)
3. ✅ All §6 coherence rules checked
4. ✅ All findings formatted per §8
5. ✅ All findings have severity per §9
6. ✅ Cross-reference table mapping findings → fix-IP candidates
7. ✅ Verification commands run (cargo check / jq empty / yq empty / cedar validate)
8. ✅ Output written to `microservices/<ms>/AUDIT-2026-05-21.md` (per-µservice audit report)
9. ✅ Aggregate findings appended to `.omc/state/wave-15-audit-aggregation-2026-05-21.md`

---

## §11. Anti-patterns

These trigger immediate P0 findings.

| Anti-pattern | What to flag | Fix direction |
|---|---|---|
| **Template-stamping** | Identical sections across N+ artifacts in same µservice | Discard + author bespoke per-artifact content |
| **Tier vocabulary** | Bronze/Silver/Gold/Platinum/capability_tier/max_tier | Per ADR-0329/0330 → tenant_class + billing_components |
| **Kernel-ahead-of-spec** | Substantive Rust kernel + zero/stub PRD/ARCH/IPs | Spec-sprint IP authoring; cite real code |
| **Counterpart mis-anchor** | Top-3 anchors don't match actual µservice purpose | Re-anchor; cite Big-8 if applicable |
| **Hallucinated artifacts** | IP declares crate/type/endpoint that doesn't exist | Scaffold OR trim IP claim |
| **Stamped journey-IP** | 30+ table rows differing only in 1-2 tokens | Bespoke per-row substance OR delete row as un-grounded |
| **Wrong contract version** | OpenAPI != 3.2.0; AsyncAPI != 3.1.0; proto2 | Bump version + validate |
| **Terraform-named-IaC** | iac/ filename starts with `terraform` | Rename to OpenTofu pattern |
| **Outward layer dep** | kernel imports app | Reverse to inward-only |
| **Old 12-layer enum** | hardcoded LAYER_ENUM_12 array | Replace with ADR-0105 13-layer |
| **Missing tenant_class** | manifest/Cedar/PRD silent on tenant_class | Add per ADR-0330/0331 |
| **Hermes terminology** | "Hermes" in active doc (not retirement citation) | Drop per ADR-0335 |
| **Redis substrate** | "redis" / "Redis" outside counterpart-fact context | Valkey per ADR-0336 |
| **Missing data-model** | No data-model.md or equivalent | Author per §4.16 |
| **Untagged proto** | proto3 fields without tags | Add tags |
| **Mock-DB in tests** | Tests use mock instead of real DB | Replace with real DB per past incident |

---

## §12. Fixer authority + anti-hallucination guardrails

(Applies to fixer agents in batch-fix phase AFTER audit complete.)

### Authorized actions
- **AUTHOR** new artifacts (PRD/ARCH/IP/contracts/Cedar/SLO/runbooks/data-model) when audit flagged missing
- **EDIT** existing artifacts to remediate findings
- **DELETE** stamped/incoherent/duplicate artifacts (with REMEDIATION-NOTES audit trail)
- **SCAFFOLD** Rust crates as workspace members (minimal stub compiling clean)
- **RENAME** via `git mv` (preserving history)

### Mandatory grounding (anti-hallucination)
- Every authored artifact must cite a REAL source: existing src/ code, existing contracts/, existing Cedar fragments, existing tests, OR explicit Wave-N IP scaffold marker
- Speculative content forbidden — if you don't have grounding, mark as `<TODO Wave-N>` cross-referenced to a fix-IP
- Counterpart references must be verifiable (real product, real feature, real version date)
- Cargo crates must compile clean (`cargo check -p <crate>` PASS)
- Contracts must validate (OpenAPI 3.2.0 strict shape; AsyncAPI 3.1.0 strict; proto3)
- Cedar must validate (`cedar validate` PASS)
- migration files must apply clean against PostgreSQL 17 + Valkey 8.x

### Audit trail
- Every fixer action must append to `microservices/<ms>/REMEDIATION-NOTES-2026-05-21.md` with:
  - Section name `## Wave 15-corpus-audit-fix (2026-05-21)`
  - Files authored / edited / deleted
  - Cited audit findings (F-NNN)
  - Cited authority (ADR-NNN / memory)
  - cargo check + validate status

---

## §13. Dispatch sequencing relative to in-flight work

**Currently in flight (do NOT touch their µservices during audit):**

- WAVE-B: 10 codex Valkey corpus rewrite (V-BUCKET-1..8 + 2 RED) + 1 Claude workspace audit + 1 Claude cloud-billing spec sprint
- WAVE-C (queued): 5 Claude contract-side IMPL-truth-up

**Audit dispatch GATE: wait for all in-flight to land before audit codex starts.**

Once gate clears:
- Audit batch 1: 12 codex × ~6-7 µservices = full corpus audit in 1 batch
- Audit batch 2: cross-corpus artifacts (ADRs/specs/standards) — 4-6 codex
- Audit batch 3: aggregate findings + author canonical aggregation report — 1 Claude
- Then: batch-fix waves dispatched by P0/P1 severity

---

## §14. Verification checklist (before declaring audit DONE)

For per-µservice audit codex:
- [ ] All 16 artifact types reviewed
- [ ] 3 tiers walked
- [ ] §6 coherence rules checked
- [ ] §11 anti-patterns scanned
- [ ] §4.16 data-model + structural map audited
- [ ] Findings formatted per §8 with severity per §9
- [ ] `microservices/<ms>/AUDIT-2026-05-21.md` written
- [ ] Findings appended to aggregation file
- [ ] cargo/jq/yq/cedar verifications run

For cross-corpus audit codex:
- [ ] All 336+ ADRs scanned for retired-ADR-still-cited drift
- [ ] specs/ JSON files validated
- [ ] docs/standards/* reviewed for keystone-ADR alignment
- [ ] tools/hooks/_canonical-primitives.md current
- [ ] docs/GLOSSARY.md + docs/machine-readable/glossary.json current
- [ ] Findings aggregated

For aggregation Claude:
- [ ] Per-µservice AUDIT-2026-05-21.md files combined
- [ ] Cross-cutting patterns identified
- [ ] P0/P1/P2 severity rollup
- [ ] Top-N hotspots flagged
- [ ] Fix-IP candidates catalogued
- [ ] Canonical aggregation report at `.omc/state/wave-15-audit-aggregation-2026-05-21.md`
- [ ] Memory file updated with audit outcome

---

## §15. Cross-references

**Memory:**
- `feedback_valkey_not_redis_2026_05_21`
- `feedback_codex_dispatch_canonical_2026_05_21` — every codex MUST use `-c model_reasoning_effort=xhigh`
- `feedback_no_capability_tiers_2026_05_20`
- `feedback_tenant_class_demo_trial_vs_paid_per_seat_usage_2026_05_20`
- `feedback_docs_substance_not_scaffold_2026_05_20`
- `feedback_verify_deliverables_not_just_line_count_2026_05_20`
- `feedback_microservice_ownership_coherence_2026_05_20`
- `feedback_clean_architecture_requirements`
- `feedback_quality_performance_scalability_bar`
- `feedback_layer_enum_adr_0105_13_canonical`
- `feedback_realignment_review_findings_2026_05_21`

**ADRs (keystone):**
- ADR-0105 (13-layer enum)
- ADR-0131 (per-µservice flat layout)
- ADR-0132 (no-suite policy)
- ADR-0242 (oyatie-is-a-tenant)
- ADR-0243 (Cedar as universal gate)
- ADR-0244 (tenant as universal scoping primitive)
- ADR-0245 (substrate vs product layering)
- ADR-0248 (Amazon-shape cellular architecture)
- ADR-0251 (compliance-pack primitive)
- ADR-0253 (HTTP/3 + QUIC default)
- ADR-0255 (intelligence two-layer)
- ADR-0329 (tier system retired)
- ADR-0330 (tenant_class + composable billing_components)
- ADR-0331 (cross-µservice tenant_class adoption)
- ADR-0335 (foundry retired absorbed by intelligence)
- ADR-0336 (Valkey not Redis)

**Specs:**
- `specs/master-plan-sequencing.json` (canonical wave-numbering)
- `tools/hooks/_canonical-primitives.md` (canonical primitives cheat sheet)

**State files:**
- `.omc/state/wave-14-aggregation.md` (mid-stream findings rollup)
- `.omc/state/wave-15-progress-2026-05-21.md` (Wave 15 progress snapshot)
- `.omc/state/wave-15-ca-verify-2026-05-21.md` (initial CA-VERIFY findings)
- `.omc/state/realignment-review-2026-05-21.md` (orchestrator analysis)

---

**End of Audit Doctrine. Every audit subagent reads this once. Do the work bespoke per µservice.**
