---
doc_class: Audit Report
shape: Reference
status: Published
date: 2026-05-21
authority_tier: 2
audited_against:
  - docs/standards/documentation-rigor.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
  - docs/architecture/corpus-rigor-audit-2026-05-20.md
related_adrs:
  - ADR-0105
  - ADR-0212
  - ADR-0242
  - ADR-0243
  - ADR-0244
  - ADR-0245
  - ADR-0246
  - ADR-0247
  - ADR-0248
  - ADR-0249
  - ADR-0250
  - ADR-0251
  - ADR-0252
  - ADR-0253
  - ADR-0254
  - ADR-0255
  - ADR-0257
  - ADR-0258
  - ADR-0263
  - ADR-0272
  - ADR-0273
  - ADR-0276
  - ADR-0280
  - ADR-0284
  - ADR-0292
  - ADR-0293
  - ADR-0294
  - ADR-0295
  - ADR-0296
  - ADR-0297
planned_enforcement_ref: governance-doc-rigor
companion_docs:
  - docs/standards/documentation-rigor.md
  - docs/architecture/corpus-rigor-audit-2026-05-20.md
  - docs/architecture/keystone-bundle-2026-05-20-synthesis.md
---

# Microservices Corpus Line Audit — 2026-05-21

Line-by-line audit of all 46 µservices at `microservices/*/`. Each µservice
examined across four file classes: `PRD.md`, `ARCHITECTURE.md`, `compliance.md`,
`manifest.json`. Audited against `documentation-rigor.md` §1.1, §1.2, §2
PRD-row, §3.1, §3.2.1 28-row matrix, §3.2.2 invariants 1–10, §3.2.3, §3.2.4,
§3.2.5, §3.2.6. Complements `corpus-rigor-audit-2026-05-20.md` with per-µservice
line-level evidence for every section.

---

## §1 Scope

**46 µservices audited.** Four file classes per µservice.

### §1.1 File class floors (documentation-rigor.md §2)

| File class | Floor |
|---|---|
| `PRD.md` | ≥1500 lines; ≥40 `### US-` stories; ≥6 UX flows; ≥3 B2C + ≥3 B2B personas; §A–§J structure; P50/P95/P99 + conversion + retention metrics; compliance-pack mapping |
| `ARCHITECTURE.md` | 14 required `## §<slug>` anchors (§3.2.1); every ANCHOR-INJECTED marker fully replaced with µservice-specific prose; ≥15 keystone-bundle ADR cross-references |
| `compliance.md` | 15 required anchors (§3.2.1); substantive prose per anchor; no boilerplate-only sections |
| `manifest.json` | Valid JSON; 6 required fields: `tier`, `audience_type`, `layer_enum_conformance`, `cell_eligibility`, `substrate_dependencies`, `compliance_packs`; `naming_justifications` block (BNF v4.1) |

### §1.2 Reference bar PRDs (Slice-8)

The four Slice-8 reference PRDs used as the quality bar throughout this audit:

| µservice | Lines | Stories | Sections | Metrics |
|---|---:|---:|---|---:|
| `payments` | 1612 | 42 | §A–§J all present | 14 |
| `identity` | 1642 | 42 | §A–§J all present | 14 |
| `workflow-engine` | 1596 | 42 | §A–§J all present | 18 |
| `ontology` | 1539 | 42 | §A–§J all present | 24 |

### §1.3 Audit methodology

- Line counts: `wc -l` per file.
- Story counts: `grep -c "^### US-"` per PRD.md.
- Section detection: `grep "^## §"` for ARCHITECTURE.md and compliance.md;
  `grep "^## [A-J]"` for PRD §A–§J structure.
- REVISE-PENDING count: `grep -c "REVISE-PENDING\|ANCHOR-INJECTED"` per ARCHITECTURE.md.
- ADR citations: unique `ADR-NNNN` pattern count per file.
- manifest.json fields: Python `json.load` + key inspection.
- Retired citations: `grep` for `ADR-0145`, `retired VCS ratchet`, `grit claim`, `rtk proxy`.
- Hard-coded strings: `grep` for `"oyatie"`, `oyatie.com` in substantive sections.
- placeholder markers: `grep -ci "placeholder marker\|placeholder marker\|placeholder marker\|placeholder marker\|code-only deferral"`.

### §1.4 Rating scales

**PRD rating:**
- PASS — ≥1500 lines AND ≥40 `### US-` stories AND full §A–§J AND ≥1 metric threshold.
- BORDERLINE — 800–1499 lines OR 20–39 stories OR partial §A–§J (missing ≤3 sections).
- STUB — <800 lines OR <20 stories OR <4 required sections. Requires full Wave-3-D rewrite.

**ARCHITECTURE.md verdict:**
- APPROVE-WITH-FINDINGS — all 14 required anchors present; ≤3 REVISE-PENDING; ≥15 ADR cites.
- REVISE — any required anchor missing; OR majority anchors are ANCHOR-INJECTED boilerplate (>3 REVISE-PENDING).

**compliance.md verdict:**
- APPROVE-WITH-FINDINGS — all 15 required anchors present; substantive prose.
- REVISE — any of the 15 required anchors missing.

**manifest.json verdict:**
- PASS — all 6 required fields + `naming_justifications` present.
- APPROVE-WITH-FINDINGS — 4–5 of 6 fields present.
- REVISE — 0–3 fields present (old schema).

---

## §2 PRD Audit Table

### §2.1 Per-µservice PRD measurements

| µservice | Lines | `### US-` stories | §A–§J present | Metrics hits | Rating |
|---|---:|---:|---|---:|---|
| analytics | 113 | 0 | none detected | 7 | STUB |
| anonymous | 387 | 0 | A B C D F H I | 8 | STUB |
| api-gateway | 117 | 0 | A C F | 3 | STUB |
| application | 382 | 0 | A B C F H I | 4 | STUB |
| audit-chain | 400 | 0 | A B C F H I | 6 | STUB |
| calendar | 326 | 0 | A B C F H I | 12 | STUB |
| cell | 425 | 0 | A B C F H I | 6 | STUB |
| cloud-iac | 443 | 0 | A B C F H I | 9 | STUB |
| cloud-k8s | 387 | 0 | A B C F H I | 6 | STUB |
| cloud-secrets | 363 | 0 | A B C F H I | 10 | STUB |
| comms-email | 183 | 0 | none detected | 5 | STUB |
| community | 1449 | 20 | numbered 1–19 (not §A–§J) | 8 | BORDERLINE |
| compliance | 127 | 0 | A C F G | 5 | STUB |
| connector | 321 | 0 | A B C D E F G H I J | 9 | STUB |
| consent-graph | 280 | 0 | none detected | 9 | STUB |
| developer-sdk | 194 | 0 | A B C F | 13 | STUB |
| docs | 387 | 0 | A B C F H I | 13 | STUB |
| drive | 409 | 0 | A B C F H I | 12 | STUB |
| feature-flags | 116 | 0 | A C F | 4 | STUB |
| finops-portal | 116 | 0 | C I | 2 | STUB |
| forms | 234 | 0 | none detected | 11 | STUB |
| foundry | 388 | 0 | A B C F H I | 3 | STUB |
| governance | 419 | 0 | A B C F H I | 3 | STUB |
| identity | 1642 | 42 | A B C D E F G H I J | 14 | **PASS** |
| intelligence | 38 | 0 | §Purpose §Scope only | 0 | STUB |
| mail | 1545 | 0 | numbered 1–16 (not §A–§J) | 6 | BORDERLINE |
| meet | 357 | 0 | A B C F H I | 3 | STUB |
| messenger | 1718 | 0 | numbered 1–19 (not §A–§J) | 11 | BORDERLINE |
| network | 462 | 0 | A B C F H I | 7 | STUB |
| notes | 400 | 0 | A B C F H I | 7 | STUB |
| observability | 309 | 0 | A B C F H I | 2 | STUB |
| ontology | 1539 | 42 | A B C D E F G H I J | 24 | **PASS** |
| ops-dashboard-control-center | 49 | 0 | §Purpose §Scope §Users §Acceptance §Exit | 0 | STUB |
| payments | 1612 | 42 | A B C D E F G H I J | 14 | **PASS** |
| plugin-app-store | 205 | 0 | A B C F | 13 | STUB |
| recordings | 469 | 0 | A B C F H I | 7 | STUB |
| sheets | 597 | 0 | A B C F H I | 16 | STUB |
| shorts | 418 | 0 | A B C F H I | 9 | STUB |
| sites | 400 | 0 | A B C F H I | 18 | STUB |
| slides | 518 | 0 | A B C F H I | 9 | STUB |
| social | 397 | 0 | A B C F H I | 7 | STUB |
| tasks | 383 | 0 | A B C F H I | 14 | STUB |
| tenancy | 511 | 0 | A B C F H I | 8 | STUB |
| translate | 311 | 0 | A B C F | 15 | STUB |
| workflow-engine | 1596 | 42 | A B C D E F G H I J | 18 | **PASS** |
| workflow-studio | 528 | 0 | A B C F H I | 9 | STUB |

### §2.2 PRD distribution

| Rating | Count | µservices |
|---|---:|---|
| PASS | 4 | `identity`, `ontology`, `payments`, `workflow-engine` |
| BORDERLINE | 3 | `community`, `mail`, `messenger` |
| STUB | 39 | All remaining |

**39 of 46 PRDs (85%) are stubs.** The 4 PASS PRDs are the Slice-8 reference-bar
µservices. The 3 BORDERLINE PRDs each fail on distinct axes.

### §2.3 Per-BORDERLINE gap analysis

**community (1449 lines, 20 stories):**
Line count is 51 below the 1500 floor. Story count is 20 — exactly half the 40
required. Stories use `### US-NN` format correctly but only 20 exist. Sections
use numbered headings (`## 1. Purpose`, `## 8. User Stories`) rather than the
canonical `## A Problem` / `## B Target users` / `## C User stories` etc. The
numbered format does not satisfy the §A–§J structure check. The PRD has genuine
content depth (feature-matrix-vs-benchmarks, moderation pipeline description,
tenant mode distinctions) but is 20 stories and 51 lines short. Recommendation:
add ≥20 more stories, bump to ≥1500 lines, migrate headings to §A–§J canonical
structure.

**mail (1545 lines, 0 `### US-` stories):**
Line count passes (1545 ≥ 1500). However, zero `### US-` story anchors exist.
The PRD has a `## 6. User Stories (20+, step-by-step)` section with narrative
prose but stories are not formatted with the required `### US-NNN` prefix
parseable by `governance-doc-rigor`. The section structure uses numbered
headings (1–16) not §A–§J. Missing §D (Functional requirements) and §E
(Non-functional requirements) canonical section names. The PRD also lacks the
6-dimension matrix (§1.2) in any named §E. Recommendation: reformat 20+ stories
with `### US-NNN` anchors; migrate section headings to §A–§J; add §E with
six-dimension matrix; total lines likely adequate once reformatted.

**messenger (1718 lines, 0 `### US-` stories):**
Line count passes (1718 ≥ 1500). Zero `### US-` story anchors. The PRD has
`## 6. User Stories (20+: 10 Personal + 10 Work)` in narrative form without
`### US-NNN` prefixes. Section structure uses numbered headings (1–19). Critical
gap: no MLS/E2EE (RFC 9420) compliance section despite this being the MLS
surface described in ADR-0246 and `feedback_mls_rfc_9420_e2ee_personal_messenger`.
Section `## 14. References` does not cite ADR-0246 (MLS) or ADR-0253 (transport).
Recommendation: add `### US-NNN` prefixes to all stories; add MLS/E2EE
compliance section citing ADR-0246; migrate to §A–§J structure.

### §2.4 Critical stub detail: worst-case PRDs

**intelligence (38 lines):** The shortest PRD in the corpus. Consists of YAML
frontmatter (citing only ADR-0215, ADR-0219, ADR-0220 — none are keystone-bundle
2026-05-20 ADRs), a `## Purpose` paragraph (3 sentences), a `## Scope` list
(3 items each), and `## Acceptance` (3 bullets). No user stories, no personas,
no UX flows, no metrics, no §A–§J structure, no compliance-pack mapping, no
ADR-0255 citation (the primary ADR for the intelligence substrate). The entire
PRD is shorter than the frontmatter of the reference-bar PRDs. The intelligence
substrate serves every AI-dispatch call across all 46 µservices and its PRD
is 38 lines.

**ops-dashboard-control-center (49 lines):** Design-anchor document masquerading
as a PRD. Uses `doc_class: Product-Requirements` with `status: accepted-design-anchor`.
Contains only §Purpose, §Scope (in/out lists), §Users (5 operator roles in
prose), §Acceptance criteria (7 numbered ACs), and §Exit claim boundary. No
user stories, no personas, no metrics, no UX flows, no compliance-pack mapping.
The acceptance criteria are well-formed but not in the required `### US-NNN`
format. The exit-claim boundary explicitly states "Runtime exit remains blocked
until implementation crates, policy tests... land" — this is a placeholder, not
a PRD.

**api-gateway (117 lines) and feature-flags (116 lines):** Both below 120 lines.
Both have `## A`, `## C`, `## F` sections only. These are scaffolding stubs from
PR-143 with no user-facing content at all. The api-gateway and feature-flags are
critical infrastructure layers; their stub PRDs mean no intern can understand
the product requirements for either.

**finops-portal (116 lines):** Has only `## C` (User stories — empty) and `## I`
(Open questions). Two sections, 116 lines, 0 metrics. The FinOps portal is
the primary cost-visibility product surface; its PRD is non-functional.

### §2.5 Structural gaps common to all 39 stubs

Every stub PRD shares the same structural deficiencies:

1. **Zero `### US-` stories** — the parser cannot count stories; the CI lane
   reports 0/40 for all 39.
2. **Missing §D (Functional requirements)** — present only in the 5 full PRDs
   and `connector`. The remaining 41 PRDs collapse functional requirements into
   §A or §C prose without the required structured section.
3. **Missing §E (Non-functional requirements / six-dimension matrix)** — the
   maintainability/observability/scalability/performance/optimization/code-quality
   matrix from §1.2 is absent in 41 PRDs. Only the 4 reference-bar PRDs and
   `connector` carry §E.
4. **Missing §G (Success metrics with conversion + retention)** — many stubs
   have P50/P95/P99 mentions in prose but no structured §G with quantified
   conversion percentages and retention targets.
5. **Missing §J (Out-of-scope)** — 41 PRDs omit this section entirely.
6. **Compliance-pack mapping absent** — the PRD §H (Compliance impact) section
   is either missing or contains only generic placeholders in 41 PRDs.

---

## §3 ARCHITECTURE.md Anchor Coverage Table

### §3.1 The 14 required anchors

Per documentation-rigor.md §3.2.1, every ARCHITECTURE.md must contain these
14 section anchors (as `## §<slug>` headers):

1. `§principals` — ADR-0242 answer
2. `§cedar-gates` — ADR-0243 answer
3. `§tenant-scoping` — ADR-0244 answer
4. `§substrate-product-binding` — ADR-0245 answer
5. `§policy-evaluation` — ADR-0246+amendment answer
6. `§cell-eligibility` — ADR-0248 answer
7. `§intelligence-dispatch` — ADR-0255+amendment answer
8. `§ontology-read-path` — ADR-0257+amendment answer
9. `§transport` — ADR-0253 answer
10. `§deployment-shape` — ADR-0254 answer
11. `§observability` — ADR-0263 answer
12. `§abuse-defence` — ADR-0297 answer
13. `§critical-path-edge-cases` — §3.2.5 answer
14. `§credential-isolation` — ADR-0296 answer

### §3.2 Per-µservice ARCHITECTURE.md coverage

| µservice | Lines | `## §` anchors | Required-14 present | Missing required | REVISE-PENDING | ADR cites | Verdict |
|---|---:|---:|---:|---|---:|---:|---|
| analytics | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| anonymous | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| api-gateway | 321 | 20 | 14 | none | 2 | 23 | APPROVE-WITH-FINDINGS |
| application | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| audit-chain | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| calendar | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| cell | 111 | 13 | 12 | `intelligence-dispatch` `ontology-read-path` | 13 | 13 | REVISE |
| cloud-iac | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| cloud-k8s | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| cloud-secrets | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| comms-email | 164 | 21 | 14 | none | 2 | 22 | APPROVE-WITH-FINDINGS |
| community | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| compliance | 183 | 24 | 14 | none | 2 | 24 | APPROVE-WITH-FINDINGS |
| connector | 226 | 18 | 14 | none | 2 | 24 | APPROVE-WITH-FINDINGS |
| consent-graph | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| developer-sdk | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| docs | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| drive | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| feature-flags | 525 | 18 | 14 | none | 0 | 34 | APPROVE-WITH-FINDINGS |
| finops-portal | 164 | 20 | 14 | none | 2 | 21 | APPROVE-WITH-FINDINGS |
| forms | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| foundry | 127 | 15 | 14 | none | 15 | 15 | REVISE |
| governance | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| identity | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| intelligence | 484 | 17 | 14 | none | 0 | 36 | APPROVE-WITH-FINDINGS |
| mail | 182 | 24 | 14 | none | 2 | 25 | APPROVE-WITH-FINDINGS |
| meet | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| messenger | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| network | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| notes | 186 | 24 | 14 | none | 2 | 25 | APPROVE-WITH-FINDINGS |
| observability | 103 | 12 | 12 | `intelligence-dispatch` `ontology-read-path` | 12 | 12 | REVISE |
| ontology | 177 | 21 | 14 | none | 2 | 28 | APPROVE-WITH-FINDINGS |
| ops-dashboard-control-center | 346 | 23 | 14 | none | 0 | 26 | APPROVE-WITH-FINDINGS |
| payments | 500 | 34 | 14 | none | 3 | 30 | APPROVE-WITH-FINDINGS |
| plugin-app-store | 127 | 15 | 14 | none | 15 | 15 | REVISE |
| recordings | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| sheets | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| shorts | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| sites | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| slides | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| social | 231 | 24 | 14 | none | 2 | 25 | APPROVE-WITH-FINDINGS |
| tasks | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| tenancy | 195 | 25 | 14 | none | 2 | 25 | APPROVE-WITH-FINDINGS |
| translate | 119 | 14 | 14 | none | 14 | 14 | REVISE |
| workflow-engine | 127 | 15 | 14 | none | 15 | 15 | REVISE |
| workflow-studio | 127 | 15 | 14 | none | 15 | 15 | REVISE |

### §3.3 ARCHITECTURE.md distribution

| Verdict | Count | µservices |
|---|---:|---|
| APPROVE-WITH-FINDINGS | 14 | `api-gateway`, `comms-email`, `compliance`, `connector`, `feature-flags`, `finops-portal`, `intelligence`, `mail`, `notes`, `ontology`, `ops-dashboard-control-center`, `payments`, `social`, `tenancy` |
| REVISE | 32 | All remaining |

### §3.4 ANCHOR-INJECTED boilerplate analysis

The `<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->` marker was injected
into ARCHITECTURE.md files as scaffolding on 2026-05-21. Example from
`analytics/ARCHITECTURE.md §principals`:

```
<!-- ANCHOR-INJECTED-2026-05-21 — REVISE-PENDING -->

This µservice operates under `oyatie.analytics.<role>` principals per ADR-0242
(oyatie-is-a-tenant). Tenant-scoped callers invoke per their `audience_type`
per ADR-0244. See `manifest.json:principals` and the Cedar entity-types in
`policy/*.cedar` for the authoritative principal roster.
```

This pattern is identical for every ANCHOR-INJECTED µservice — only the
µservice name slug changes. The boilerplate:

- Names no actual Cedar entity types (only `oyatie.<name>.<role>` placeholder).
- Cites no concrete principal slugs.
- Defers to `manifest.json:principals` — a field that does not exist in any
  manifest (all 46 manifests use the old schema without a `principals` key).
- Defers to `policy/*.cedar` — the Cedar files exist in the artifact tree but
  the ARCHITECTURE.md contains no cross-reference to specific fragment paths.
- Contains no intern-buildable information: an intern reading `§principals` for
  `calendar` learns only that it "operates under `oyatie.calendar.<role>`
  principals" — zero actionable information.

The same boilerplate pattern applies to all 14 injected sections for the 32
all-boilerplate ARCHITECTURE.md files. The `§transport` boilerplate declares
"defaults to HTTP/3 + QUIC per ADR-0253" with no IaC file paths; the
`§intelligence-dispatch` boilerplate declares "library-first via
`shared-intelligence-dispatch`" with no actual call sites; the
`§ontology-read-path` boilerplate declares a `freshness_floor` "in the
read-path configuration" without specifying any value.

**Contrast with `feature-flags/ARCHITECTURE.md` (0 REVISE-PENDING, 34 ADR cites,
525 lines):** The `§principals` section names specific principal slugs including
`oyatie.feature-flags.flag-manager`, `oyatie.feature-flags.flag-evaluator`,
`oyatie.feature-flags.killswitch-operator`, `oyatie.feature-flags.experiment-designer`,
`oyatie.feature-flags.pack-overlay-agent`, and `oyatie.feature-flags.audit-emitter`
— each with their specific permission scope. The `§cedar-gates` section includes
actual PERMIT and FORBID rule shapes with step-up-auth class requirements.
This is the target state for all 32 REVISE ARCHITECTURE.md files.

### §3.5 Missing `intelligence-dispatch` and `ontology-read-path` anchors

11 µservices are missing both `§intelligence-dispatch` and `§ontology-read-path`:
`anonymous`, `application`, `audit-chain`, `cell`, `cloud-iac`, `cloud-k8s`,
`cloud-secrets`, `consent-graph`, `developer-sdk`, `network`, `observability`.

These are infrastructure-tier µservices where the scaffold template assumed they
do not call Intelligence or read the Ontology and therefore omitted those anchors.
However, documentation-rigor.md §3.2.1 rows 14 and 15 require an explicit
declaration for ALL µservices — including the negative declaration ("this µservice
does NOT call Intelligence; it is a pure infrastructure layer and no AI-dispatch
path exists") as evidence that the question was considered and answered. Absence
of the section fails the 28-row matrix check regardless of whether the µservice
actually calls those substrates.

### §3.6 ADR-adherence 28-row matrix — per-µservice scores

The 28-row matrix (§3.2.1) checks whether each ADR from ADR-0242 through
ADR-0297 is cited in ARCHITECTURE.md + compliance.md combined. Column order
matches the 28-row sequence. `Y` = cited at least once; `.` = not cited.

ADR sequence (columns left to right):
`0242 0243 0244 0245 0246 0247 0248 0249 0250 0251 0252 0253 0254 0255
 0257 0258 0263 0272 0273 0276 0280 0284 0292 0293 0294 0295 0296 0297`

| µservice | Score | Citation bitmap |
|---|---:|---|
| analytics | 17/28 | `YYYYY.Y.YY.YYYYYY....Y....YY` |
| anonymous | 15/28 | `YYYYY.Y.YY.YY...Y....YY...YY` |
| api-gateway | 28/28 | `YYYYYYYYYYYYYYYYYYYYYYYYYYYY` |
| application | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| audit-chain | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| calendar | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| cell | 17/28 | `YYYYYYY.YY.YY...Y....Y.Y.YYY` |
| cloud-iac | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| cloud-k8s | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| cloud-secrets | 15/28 | `YYYYY.Y.YY.YY...Y....Y...YYY` |
| comms-email | 22/28 | `YYYYY.Y.YYYYYYYYYYY..Y..YYYY` |
| community | 18/28 | `YYYYY.Y.YY.YYYYYY....YY...YY` |
| compliance | 24/28 | `YYYYY.YYYYYYYYYYY..Y.YYYYYYY` |
| connector | 27/28 | `YYYYYYYYYYYYYYY.YYYYYYYYYYYY` |
| consent-graph | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| developer-sdk | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| docs | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| drive | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| feature-flags | 28/28 | `YYYYYYYYYYYYYYYYYYYYYYYYYYYY` |
| finops-portal | 21/28 | `YYYYY.Y.YYYYYYYYY..Y.Y..YYYY` |
| forms | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| foundry | 20/28 | `YYYYYYY.YY.YYYYYY....Y.Y.YYY` |
| governance | 17/28 | `YYYYY.Y.YY.YYYYYY....Y....YY` |
| identity | 17/28 | `YYYYY.Y.YY.YYYY.Y....Y...YYY` |
| intelligence | 28/28 | `YYYYYYYYYYYYYYYYYYYYYYYYYYYY` |
| mail | 26/28 | `YYYYY.YYYYYYYYYYYYYY.YYYYYYY` |
| meet | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| messenger | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| network | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| notes | 25/28 | `YYYYY.YYYYYYYYYYYY.Y.YYYYYYY` |
| observability | 14/28 | `YYYYY.Y.YY.YY...Y....Y....YY` |
| ontology | 25/28 | `YYYYY.YYYYYYYYYYY.Y.YYYYYYYY` |
| ops-dashboard-control-center | 27/28 | `YYYYYYYYYYYYYYY.YYYYYYYYYYYY` |
| payments | 27/28 | `YYYYYYYYYYYYYYYYYYYYYYYYYYY.` |
| plugin-app-store | 18/28 | `YYYYY.YYYY.YYYY.Y....YY...YY` |
| recordings | 18/28 | `YYYYY.Y.YY.YYYYYY....YY...YY` |
| sheets | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| shorts | 18/28 | `YYYYY.Y.YY.YYYYYY....YY...YY` |
| sites | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| slides | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| social | 25/28 | `YYYYY.YYYYYYYYYYYY.Y.YYYYYYY` |
| tasks | 17/28 | `YYYYY.Y.YY.YYYY.Y....YY...YY` |
| tenancy | 25/28 | `YYYYYYYYYYYYYYYYY..Y.YYYYYYY` |
| translate | 18/28 | `YYYYY.Y.YY.YYYYYY....YY...YY` |
| workflow-engine | 17/28 | `YYYYY.YYYY.YYYY.Y....Y....YY` |
| workflow-studio | 18/28 | `YYYYY.YYYY.YYYY.Y....YY...YY` |

### §3.7 28-row matrix: most commonly missing ADRs

| ADR | Question gated | µservices missing citation | Count |
|---|---|---|---:|
| ADR-0247 (self-modification) | Meta-trust-root attestation path | analytics, anonymous, api-gateway*, application, audit-chain, calendar, cloud-iac, cloud-k8s, cloud-secrets, consent-graph, developer-sdk, docs, drive, forms, governance, identity*, meet, messenger, network, observability, recordings, sheets, shorts, sites, slides, tasks, translate | 27 |
| ADR-0258 (API versioning) | SemVer policy + deprecation cadence | analytics, anonymous, application, audit-chain, calendar, cell*, cloud-iac, cloud-k8s, cloud-secrets*, consent-graph, developer-sdk, docs, drive, forms, foundry*, governance, identity*, meet, messenger, network, observability, sheets, sites, slides, social*, tasks, workflow-engine | 27 |
| ADR-0280 (substrate-of-substrate dependency) | DAG position | 33 µservices | 33 |
| ADR-0272 (cookie consent per-purpose) | Per-purpose consent surface | 22 µservices | 22 |
| ADR-0273 (per-tenant DKIM/SPF/DMARC) | Mail-emitting deliverability | 18 µservices | 18 |
| ADR-0276 (backup portability GDPR Art.20) | Per-tenant backup-export format | 22 µservices | 22 |
| ADR-0293 (meta-trust-root) | Foundry-touching attestation | 22 µservices | 22 |
| ADR-0294 (Cedar fragment soak) | ≥60s soak window | 22 µservices | 22 |

*These µservices cite ADR-0247 via compliance.md §self-modification but not
ARCHITECTURE.md. Both files are searched in the score above; the `.` in the
bitmap means neither file cites the ADR.

**ADR-0280 (substrate-of-substrate dependency DAG) is the most universally
missing ADR:** Only 13 of 46 µservices cite it anywhere. This means 33 µservices
have not declared their substrate-dependency DAG position — a prerequisite for
the six-hops graph traversal (§3.1) and the `manifest.json:substrate_dependencies`
field (which is also missing from 44 manifests). The two gaps are linked.

---

## §4 compliance.md Anchor Coverage Table

### §4.1 The 15 required anchors

Per documentation-rigor.md §3.2.1, every compliance.md must contain:

1. `§day-one-cert-readiness` — ADR-0250 answer
2. `§pack-overlay-roster` — ADR-0251 answer
3. `§detection-substrate-binding` — §3.2.6 row 1
4. `§ml-model-lifecycle` — §3.2.6 row 2
5. `§detection-fairness-audit` — §3.2.6 row 3
6. `§investigation-binding` — §3.2.4 D7
7. `§insider-threat-controls` — §3.2.4 D8
8. `§threat-intelligence-feeds` — §3.2.4 D9
9. `§key-rotation-cadence` — §3.2.4 D16
10. `§crypto-agility-plan` — §3.2.4 D20
11. `§pentest-and-bounty-cadence` — §3.2.4 D21
12. `§facility-controls` — §3.2.4 D22
13. `§supply-chain-risk` — §3.2.4 D23
14. `§critical-path-edge-cases` — §3.2.5 mandatory
15. `§data-classification` — §3.2.6 row 5

### §4.2 Per-µservice compliance.md coverage

| µservice | Lines | Total anchors | Required-15 present | Missing required | Verdict |
|---|---:|---:|---:|---|---|
| analytics | 183 | 16 | 15 | none | APPROVE-WITH-FINDINGS |
| anonymous | 313 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| api-gateway | 267 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| application | 281 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| audit-chain | 304 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| calendar | 480 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| cell | 339 | 17 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| cloud-iac | 403 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| cloud-k8s | 413 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| cloud-secrets | 352 | 25 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| comms-email | 249 | 16 | 15 | none | APPROVE-WITH-FINDINGS |
| community | 281 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| compliance | 211 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| connector | 196 | 21 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| consent-graph | 270 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| developer-sdk | 152 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| docs | 442 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| drive | 402 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| feature-flags | 376 | 24 | 15 | none | APPROVE-WITH-FINDINGS |
| finops-portal | 119 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| forms | 295 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| foundry | 234 | 19 | 15 | none | APPROVE-WITH-FINDINGS |
| governance | 331 | 16 | 15 | none | APPROVE-WITH-FINDINGS |
| identity | 240 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| intelligence | 386 | 17 | 9 | `investigation-binding` `pentest-and-bounty-cadence` `facility-controls` `supply-chain-risk` `critical-path-edge-cases` `data-classification` | REVISE |
| mail | 427 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| meet | 335 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| messenger | 307 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| network | 419 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| notes | 326 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| observability | 403 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| ontology | 409 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| ops-dashboard-control-center | 237 | 23 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| payments | 435 | 40 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| plugin-app-store | 160 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| recordings | 300 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| sheets | 402 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| shorts | 402 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| sites | 381 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| slides | 370 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| social | 362 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| tasks | 400 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| tenancy | 421 | 17 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| translate | 351 | 17 | 15 | none | APPROVE-WITH-FINDINGS |
| workflow-engine | 362 | 14 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |
| workflow-studio | 394 | 15 | 13 | `ml-model-lifecycle` `detection-fairness-audit` | REVISE |

### §4.3 compliance.md distribution

| Verdict | Count | µservices |
|---|---:|---|
| APPROVE-WITH-FINDINGS | 12 | `analytics`, `comms-email`, `community`, `feature-flags`, `foundry`, `governance`, `mail`, `recordings`, `shorts`, `social`, `translate`, `tenancy`* |
| REVISE | 34 | All remaining |

*`tenancy` compliance.md is APPROVE-WITH-FINDINGS on anchor count but still
REVISE on the 28-row matrix score (missing ADR-0272, ADR-0273).

### §4.4 Dominant gap: `ml-model-lifecycle` + `detection-fairness-audit`

**34 of 46 µservices (74%) missing both `§ml-model-lifecycle` AND
`§detection-fairness-audit`.** This is a corpus-wide template deficiency —
the compliance.md scaffold that was applied to most µservices omitted these
two anchors. Neither is in the 14 anchors of the "base" template; they were
added to the documentation-rigor.md §3.2.1 required list as part of the
2026-05-20 keystone bundle.

The 12 µservices that DO have these anchors received them via one of:
(a) a more recent compliance template (feature-flags, intelligence, ops-dashboard),
(b) explicit authoring during Slice 8 (mail, notes, social, ontology*),
(c) ML-adjacent services that prompted their addition (recordings, shorts, translate).

*ontology compliance.md is missing both despite being a Slice-8 reference
service — the compliance.md was not updated with the keystone-bundle additions.

### §4.5 intelligence compliance.md: worst-case substrate

`intelligence/compliance.md` (386 lines) is missing 6 of 15 required anchors:
`investigation-binding`, `pentest-and-bounty-cadence`, `facility-controls`,
`supply-chain-risk`, `critical-path-edge-cases`, `data-classification`. The
file has substantive content in the anchors it does have — the
`§pack-overlay-roster` section includes a rich framework×pack matrix with 20
frameworks × 14 packs, specific HIPAA control implementations, FedRAMP High
deployment constraints, and KR-PIPA/KR-CSAP overlay rules. The depth in the
present sections is high. The gap is that the six missing anchors were never
authored; the file ends after `§portability` with no trailing sections.

The `§critical-path-edge-cases` gap is the most serious: the intelligence
substrate is in the critical path of every AI-dispatch call across 46 µservices.
Its failure modes (model timeout, provider circuit-open, context-window overflow,
policy-refusal cascade, BYOK credential expiry) are not documented anywhere in
its compliance.md. This is a P0 finding.

### §4.6 payments compliance.md: richest file, still REVISE

`payments/compliance.md` (435 lines, 40 anchors) is the richest compliance.md
in the corpus. It has 11 numbered pack-overlay sections (PCI-DSS L1 v4,
KR-FSS, EU PSD2+SCA, US-state MTL, CCPA/CPRA-2023, AU AML/CTF, BR LGPD+BACEN,
IN RBI, CN PIPL, COPPA+KOSA, SOX-ITGC) plus consent, email-deliverability,
self-modification, meta-trust-attestation, platform-owner-indirection,
bootstrap-trust-chain, credential-isolation, minor-protection, cross-references,
and then a full second sweep of the 15 standard anchors. Despite this depth, it
is REVISE because `§ml-model-lifecycle` and `§detection-fairness-audit` are
absent. The payments µservice uses ML for fraud detection — both anchors are
mandatory for any ML-touching service.

---

## §5 manifest.json Field Coverage Table

### §5.1 Required fields

Per documentation-rigor.md §1 invariant #1:
`tier`, `audience_type`, `layer_enum_conformance`, `cell_eligibility`,
`substrate_dependencies`, `compliance_packs` — plus `naming_justifications`
(BNF v4.1, per `feedback_naming_justification`).

### §5.2 Per-µservice manifest.json coverage

| µservice | JSON valid | `tier` | `audience_type` | `layer_enum_conformance` | `cell_eligibility` | `substrate_dependencies` | `compliance_packs` | Fields/6 | `naming_justifications` | Verdict |
|---|---|---|---|---|---|---|---|---:|---|---|
| analytics | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| anonymous | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| api-gateway | Yes | substrate | all | — | ["tier-0"] | — | — | 4/6 | Yes | APPROVE-WITH-FINDINGS |
| application | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| audit-chain | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| calendar | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| cell | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| cloud-iac | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| cloud-k8s | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| cloud-secrets | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| comms-email | Yes | — | — | — | — | — | — | 0/6 | Yes | REVISE — old schema |
| community | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| compliance | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| connector | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| consent-graph | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — has `cross_microservice_dependencies` extension |
| developer-sdk | Yes | external-facing | — | — | — | — | — | 1/6 | No | REVISE — non-standard `tier` value |
| docs | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| drive | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| feature-flags | Yes | substrate | — | — | {object} | — | — | 3/6 | Yes | REVISE — `cell_eligibility` non-enum; `audience_type` missing |
| finops-portal | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| forms | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| foundry | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| governance | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| identity | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| intelligence | Yes | substrate | [array-6] | — | {object} | — | — | 4/6 | Yes | REVISE — `cell_eligibility` non-enum; missing 2 fields |
| mail | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| meet | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| messenger | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema; MLS µservice without `compliance_packs` |
| network | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| notes | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| observability | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — canonical exemplar with failing manifest |
| ontology | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| ops-dashboard-control-center | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| payments | Yes | substrate | — | — | — | — | — | 2/6 | No | REVISE — only `tier` + 1 partial field |
| plugin-app-store | Yes | external-facing | — | — | — | — | — | 1/6 | No | REVISE — non-standard `tier` value |
| recordings | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| sheets | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| shorts | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| sites | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| slides | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| social | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| tasks | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| tenancy | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| translate | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |
| workflow-engine | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — reference-bar PRD; manifest entirely old schema |
| workflow-studio | Yes | — | — | — | — | — | — | 0/6 | No | REVISE — old schema |

### §5.3 manifest.json distribution

| Verdict | Count |
|---|---:|
| PASS — all 6 fields + naming_justifications | 0 |
| APPROVE-WITH-FINDINGS — 4–5 fields | 2 (`api-gateway` 4/6, `intelligence` 4/6) |
| REVISE — 0–3 fields | 44 |

**Zero of 46 manifests pass the full 6-field check.** The root cause is a
schema mismatch: 39 µservices use the old PR-143 manifest schema
(`schema_version`, `bounded_contexts`, `layers`, `contracts`, `capabilities`,
`slos`, `ips`, `regulatory_packs`, `lts_pins`, `adrs`, `hyperscaler_inv_coverage`,
`audit_chain`, `secrets_substrate`, `ontology_projections`, `mesh_layering`)
which predates documentation-rigor.md §1's 6-field requirement. The old schema
is not wrong per se but does not satisfy the new required fields.

### §5.4 Old schema vs new schema field mapping

The old schema fields do not map 1:1 to the required fields:

| Required field | Closest old-schema field | Equivalent? |
|---|---|---|
| `tier` | none | No — the old schema has no tier classification |
| `audience_type` | none | No — not present |
| `layer_enum_conformance` | `layers[].layer` | No — old schema lists layer slugs per BC but has no explicit ADR-0105 conformance declaration |
| `cell_eligibility` | none | No — not present |
| `substrate_dependencies` | none — `adrs` lists ADRs, not dependency µservices | No |
| `compliance_packs` | `regulatory_packs` | Partial — `regulatory_packs` uses different id format than `compliance_packs` |

The `regulatory_packs` → `compliance_packs` rename is not just cosmetic: the
old field uses short-form pack names (e.g., `kr-pipa`, `gdpr-eu`) while the
required `compliance_packs` field uses the canonical registry ids
(e.g., `pack-kr`, `pack-eu`, `pack-pci-dss-l1-v4`). The CI lane
`governance-cross-consistency` reads `compliance_packs`; it will not
find `regulatory_packs`.

### §5.5 `cell_eligibility` type inconsistency

Three µservices have `cell_eligibility` present but in different shapes:

- `api-gateway`: `["tier-0"]` — string array of tier labels.
- `feature-flags`: `{"tier_min": 2, "sovereign_packs_supported": [...], "cells": [...]}` — structured object.
- `intelligence`: `{"default_tier": "Tier-1", "regulated_tier": "Tier-3", "tier_selection_rule": "...", "authority_adr": "ADR-0248"}` — structured object with different keys.

ADR-0248 §D-1 defines `cell_eligibility` as an enum with values
`Tier 0` / `Tier 1` / `Tier 2` / `Tier 3`. None of the three present
implementations uses this enum. The `feature-flags` implementation
(structured object) is the richest but is an ad-hoc extension not aligned to
the canonical enum. The `governance-cross-consistency` lane invariant 5
check will fail for all three.

### §5.6 `naming_justifications` coverage

Only 4 of 46 manifests contain a `naming_justifications` block:
`api-gateway`, `comms-email`, `feature-flags`, `intelligence`.

Per `feedback_naming_justification`: "Every new name must carry one-line
justification proving v4 BNF + 12-layer-enum conformance at scaffold time."
The remaining 42 manifests carry no naming justification. The `api-gateway`
manifest's `naming_justifications` block is the reference form; the other 42
must be brought to parity.

---

## §6 Cross-µservice Consistency Findings (§3.2.2 invariants 1–10)

### §6.1 Invariant 1 — Field naming consistency

**FAIL corpus-wide.** The manifest schema is non-uniform across 46 µservices:
39 use old-schema, 5 use partial new-schema, 2 use hybrid forms. The following
field names appear in different forms across µservices:

- `regulatory_packs` (old) vs `compliance_packs` (required) — 44 µservices use
  the old form.
- `layers` (old, array of {layer, crates}) vs `layer_enum_conformance` (required,
  explicit ADR-0105 declaration) — 39 µservices use the old form; 0 use the
  required form.
- `tier` value inconsistency: `api-gateway` uses `"substrate"`;
  `developer-sdk` and `plugin-app-store` use `"external-facing"` (not a valid
  ADR-0245 tier value — valid values are `substrate` and `product`).

In Cedar and OpenAPI, field naming consistency was not directly audited (the
contract files exist at `contracts/*.yaml` but were not line-examined in this
audit). However, given that the ARCHITECTURE.md `§tenant-scoping` sections are
all ANCHOR-INJECTED boilerplate in 32 µservices, the `tenant_id`,
`provider_credential_mode`, and `audience_type` field declarations have not been
verified per-µservice.

### §6.2 Invariant 2 — Audit-event-class taxonomy consistency

**CANNOT FULLY VERIFY.** The central audit-event-class registry (ADR-0263 §D-N)
was not found at a canonical path during this audit. The `§observability`
sections in 32 of 46 ARCHITECTURE.md files are ANCHOR-INJECTED boilerplate
deferring to `dashboards/*.json` for the authoritative event-class list.
Without the central registry file, cross-µservice consistency cannot be
mechanically verified. Finding: each µservice declares its audit event classes
in its ARCHITECTURE.md `§observability` section; without a central registry
these cannot be checked for collisions or gaps. The ADR-0263 registry creation
(if not yet done) is a prerequisite for this invariant.

### §6.3 Invariant 3 — OpenAPI 3.2.0 / AsyncAPI 3.1.0 / proto3

**PASS (no violations found in existing contract files).** Grep over all
`microservices/*/contracts/openapi*.yaml` and
`microservices/*/contracts/asyncapi*.yaml` found zero files at non-3.2.0 or
non-3.1.0 versions. **Caveat:** many µservices have not yet authored their
contract files — stub µservices list them as deferred artifacts. The absence of
a violation reflects partial absence of the files, not confirmed corpus-wide
conformance. Contract files must be audited once created.

### §6.4 Invariant 4 — OpenBao SecretReference path shape consistency

**CANNOT VERIFY.** All 32 ANCHOR-INJECTED ARCHITECTURE.md `§credential-isolation`
sections reference `iac/<env>-ingress.yaml` for OpenBao path details but
contain no actual `${openbao:secret/...}` path strings. The IaC files
themselves were not in scope for this audit. A dedicated IaC audit pass is
required for this invariant.

### §6.5 Invariant 5 — Cell-tier-conformance consistency

**FAIL.** Three µservices with `cell_eligibility` use three incompatible shapes
(see §5.5). 43 µservices have no `cell_eligibility` at all. The ADR-0248 §D-1
canonical enum (`Tier 0` / `Tier 1` / `Tier 2` / `Tier 3`) is used by zero
µservices in their manifest. This invariant is a corpus-wide fail.

### §6.6 Invariant 6 — Compliance-pack-id consistency

**FAIL.** Three different pack-id naming conventions are in active use:

| Convention | Example | Used by |
|---|---|---|
| Short without prefix | `gdpr-eu`, `kr-isms-p`, `fedramp-high` | `feature-flags` manifest cells[].sovereign_packs |
| Hyphen-prefixed | `pack-kr`, `pack-eu`, `pack-us-healthcare` | `cloud-secrets` compliance.md |
| Versioned full-form | `pack-pci-dss-l1-v4`, `pack-eu-psd2-sca`, `pack-kr-fss` | `payments` compliance.md |

The central pack registry (ADR-0251) is the canonical source. No µservice
references the central registry path explicitly in their compliance.md or
manifest. The CI lane `governance-cross-consistency` invariant 6 check
reads `compliance_packs[]` from manifests — 44 manifests have no such field,
and the 2 that have related fields (`feature-flags` via `sovereign_packs`,
`payments` via old `regulatory_packs` field) use different id formats. This
invariant requires the manifest migration (§10.1 batch action 1) plus a
pack-id normalization pass.

### §6.7 Invariant 7 — Layer-enum consistency (ADR-0105 13-layer)

**CANNOT VERIFY AT MANIFEST LEVEL.** The `layer_enum_conformance` field is
absent from 44 of 46 manifests. The `layers` arrays in old-schema manifests
contain crate-layer slugs (e.g., `kernel`, `domain`, `application`, `api`) but
without an explicit `layer_enum_conformance` declaration these cannot be
validated as conforming to ADR-0105's 13-layer canonical set. The A1-BL-1 fix
(referenced in documentation-rigor.md §3.2.2 invariant 7) is meant to close
per-ADR layer forks — but without the conformance declaration field populated
it cannot be enforced. A dedicated ADR-0105 conformance audit is required.

### §6.8 Invariant 8 — Naming-justification tables

**FAIL.** 42 of 46 manifests lack `naming_justifications`. See §5.6.

### §6.9 Invariant 9 — Six-hops graph traversability

**STRUCTURAL PRECONDITIONS BROKEN for 39 µservices.** Six-hops traversal from
any PRD into the substrate dependency layer requires `manifest.json:substrate_dependencies`
edges. These edges are absent from 44 manifests. An intern reading
`community/PRD.md` cannot traverse to `cell`, `tenancy`, `identity`, or
`foundry` via manifest links — the edges do not exist in the JSON.

Additionally, 32 ARCHITECTURE.md files are ANCHOR-INJECTED boilerplate that
defer all cross-references to other files. An intern following those references
reaches dead ends: `manifest.json:principals` does not exist in 44 manifests;
`policy/*.cedar` files exist but are not cross-referenced from ARCHITECTURE.md
with specific fragment names. The six-hops graph has broken edges at the
ARCHITECTURE.md→manifest and ARCHITECTURE.md→policy transition points for
32 µservices.

The `tools/doc-graph-walker/` tool (if operational) would report BFS failure
from any of the 32 boilerplate ARCHITECTURE.md files within ≤2 hops.

### §6.10 Invariant 10 — BYOK terminology consistency

**PARTIALLY FAIL.** ADR-0255 §D-4 disambiguates provider-BYOK (opt-in LLM/provider
credential isolation, `provider_credential_mode ∈ {platform_default, byok, byok_required_by_pack}`)
from encryption-BYOK (ADR-0251 §D-10, customer-managed keys for data at rest).

The `provider_credential_mode` field:
- Absent from all 46 manifests (no manifest has this field).
- Referenced in ANCHOR-INJECTED `§tenant-scoping` boilerplate as "declared in
  `manifest.json`" — which is factually incorrect for 44 manifests.
- Present with correct semantics only in `feature-flags/compliance.md`
  (which uses `byok_required_by_pack` values in Cedar rule shapes) and
  `intelligence/compliance.md` (which uses `byok-gating.cedar` in the
  HIPAA pack section).

The disambiguation requirement (provider-BYOK vs encryption-BYOK) is met in
only 2 of 46 µservices. The remaining 44 either mention BYOK without
disambiguation or do not mention it at all.

---

## §7 Staleness Findings

### §7.1 Retired ADR and tool citations

#### §7.1.1 ADR-0145 (Workflow+Ontology forced-adapter — retired)

ADR-0145 was retired per the 2026-05-18 direct-gRPC reform (`feedback_workflow_objectgraph_adapter_layer`).
The following files cite ADR-0145 in contexts that assume the retired pattern
is still active:

**ARCHITECTURE.md files:**
- `connect/ARCHITECTURE.md` — cites ADR-0145 in transport section
- `payments/ARCHITECTURE.md` — cites ADR-0145 in cross-references
- `intelligence/ARCHITECTURE.md` — cites ADR-0145 in ontology-read-path section

**PRD.md files (via frontmatter `related_adrs` lists):**
`api-gateway`, `audit-chain`, `calendar`, `comms-email`, `community`,
`compliance`, `connector`, `docs`, `drive`, `feature-flags`, `forms`, `foundry`,
`identity`, `mail`, `meet`, `messenger`, `network`, `observability`, `ontology`,
`payments`, `plugin-app-store`, `sheets`, `shorts`, `sites`, `slides`, `social`,
`tasks`, `tenancy`, `translate`, `workflow-engine`, `workflow-studio` (31 PRDs)

**compliance.md files:**
`audit-chain`, `calendar`, `cell`, `cloud-iac`, `cloud-k8s`, `comms-email`,
`compliance`, `docs`, `drive`, `mail`, `messenger`, `network`, `observability`,
`ontology`, `payments`, `sheets`, `shorts`, `sites`, `social`, `tasks`,
`tenancy`, `workflow-engine`, `workflow-studio` (23 files)

**Root cause:** ADR-0145 appears in the `related_adrs` list of many PRD
frontmatter blocks as a historical architecture citation. Since ADR-0145's
retirement post-dates most PRD authoring, these citations were not updated.
The ARCHITECTURE.md substantive citations (connect, payments, intelligence) are
the higher priority — they affect runtime reasoning. The PRD frontmatter
citations are lower priority but still fail the completeness invariant
("every retired doc is retired explicitly").

#### §7.1.2 `retired VCS ratchet` / external-agent-coordination tool citations

Per `feedback_deprecate_external_agent_coord_tooling` (2026-05-16): raw
git/gh canonical; external-agent-coordination tooling retired. The string
`retired VCS ratchet` appears in compliance.md boilerplate blocks referencing "coordination
via retired VCS ratchet claim/work/done cycle" in at least 23 compliance.md files (the same
set with the ADR-0145 citation above). These compliance.md files were scaffolded
before the 2026-05-16 deprecation and retain the stale reference.

### §7.2 Hard-coded `oyatie` strings (ADR-0284 violation)

ADR-0284 requires platform-owner name indirection — no hard-coded `"oyatie"`
strings in source or specs that would prevent white-labeling or tenant
rebranding.

**compliance.md (45 of 46 µservices):** All compliance.md files contain the
string `oyatie` in the `§platform-owner-indirection` section's audit statement.
The section reads: "This µservice has audited its source for hard-coded `oyatie`
strings and migrated them to the platform-owner indirection primitive." This is
ironic — the audit statement itself contains the string — but this is acceptable
per ADR-0284 because the section is explicitly about the migration audit, not a
functional string binding. CI grep checks will flag it; a per-µservice exemption
may be needed.

**ARCHITECTURE.md — substantive violations (5 files):** These are not in
the indirection-audit section and represent genuine ADR-0284 violations:
- `api-gateway/ARCHITECTURE.md` — 2 occurrences in principal slug examples
  (`oyatie.api-gateway.*`)
- `comms-email/ARCHITECTURE.md` — 1 occurrence in example sender principal
- `community/ARCHITECTURE.md` — 1 occurrence in example Cedar entity path
- `compliance/ARCHITECTURE.md` — 1 occurrence in namespace example
- `intelligence/ARCHITECTURE.md` — 1 occurrence in audience tag example

All 5 must be replaced with the platform-owner indirection primitive before
the `governance-cross-consistency` lane goes BLOCKER.

### §7.3 placeholder marker / placeholder marker / placeholder marker / placeholder marker / "code-only deferral" occurrences

Per completeness invariant 7: zero such occurrences in canonical doc bodies.

| µservice | Count | File(s) | Nature |
|---|---:|---|---|
| community | 16 | PRD.md | Mix of placeholder marker story stubs and placeholder marker metric placeholders in §8 and §12 |
| tasks | 13 | compliance.md | placeholder marker items in `§investigation-binding` and `§key-rotation-cadence` |
| messenger | 10 | PRD.md | placeholder marker items in MLS/E2EE section and placeholder marker in UX flows |
| social | 5 | compliance.md | placeholder marker items in `§detection-substrate-binding` |
| identity | 2 | PRD.md + compliance.md | placeholder marker in story acceptance criteria |
| ontology | 2 | PRD.md + compliance.md | placeholder marker in §I Open questions |
| payments | 2 | ARCHITECTURE.md + PRD.md | REVISE-PENDING (counted separately above) + 1 placeholder marker in §J |
| sites | 2 | PRD.md + compliance.md | placeholder marker in metrics section |
| workflow-engine | 2 | PRD.md + compliance.md | placeholder marker in §I Open questions |
| analytics | 1 | PRD.md | placeholder marker in success metrics |
| calendar | 1 | PRD.md | placeholder marker in §G Success metrics |
| drive | 1 | PRD.md | placeholder marker in UX flows |
| forms | 1 | PRD.md | placeholder marker in acceptance criteria |
| mail | 1 | compliance.md | placeholder marker in `§email-deliverability` |
| **Total** | **68** | — | All violations of completeness invariant 7 |

All 68 occurrences trigger `governance-doc-link-resolves` (BLOCKER
from 2026-07-16). The `tasks` compliance.md placeholder marker count (13) is particularly
problematic — the compliance document's key security sections are unfinished.

### §7.4 Date and vintage staleness

**ANCHOR-INJECTED sections:** All 32 all-boilerplate ARCHITECTURE.md files
carry the timestamp `2026-05-21` in their `<!-- ANCHOR-INJECTED-2026-05-21 —
REVISE-PENDING -->` markers. These are not stale by date but stale by content
quality — the timestamp proves auto-generation rather than authoring.

**analytics/manifest.json:** Uses a unique schema with `framework_scorecards`,
`ip_pack`, and `mesh_layering` fields found in no other manifest. No `date`
field, no `schema_version` field. This appears to pre-date the PR-143 schema
and is the oldest-vintage manifest in the corpus.

**PRD frontmatter citing ADR-0145:** All 31 PRDs with ADR-0145 in their
`related_adrs` list were authored before the 2026-05-18 ADR-0145 retirement.
Their `date` fields range from 2026-05-17 to 2026-05-20. None have been
updated post-retirement.

---

## §8 Critical-Path Coverage Gap (§3.2.5)

### §8.1 §3.2.5 scenario applicability map

Documentation-rigor.md §3.2.5 enumerates critical-path edge cases that MUST be
documented in `compliance.md §critical-path-edge-cases` and
`ARCHITECTURE.md §critical-path-edge-cases`. The following table maps each
scenario to applicable µservices and assesses coverage:

| Scenario | Applicable µservices | Coverage |
|---|---|---|
| Cedar evaluation timeout / policy-engine latency spike | All 46 | NONE — no µservice documents Cedar timeout fallback policy in §critical-path-edge-cases; all 32 boilerplate ARCHITECTURE.md files defer to other docs |
| OpenBao key-rotation race / credential-TTL-expiry cascade | `cloud-secrets`, `identity`, `foundry`, `cell`, `tenancy` | NONE — all 5 have boilerplate ARCHITECTURE.md; `cloud-secrets` compliance.md has §key-rotation-cadence but no specific race-condition fallback |
| Cross-tenant data leak via shared cache / projection cache | `tenancy`, `identity`, `ontology`, `intelligence` | PARTIAL — `payments` and `ontology` ARCHITECTURE.md have non-boilerplate sections with tenant-isolation guards; `intelligence` has compliance.md pack-isolation but no explicit cache-leak scenario |
| Regional cell unreachability / multi-region split-brain | All substrate µservices | PARTIAL — `payments` §critical-path-edge-cases (REVISE-PENDING) links to `multi-region.md`; all other substrate µservices boilerplate |
| Compliance pack activation race (pack mid-activation) | `compliance`, `tenancy`, `cell` | NONE — `compliance` ARCHITECTURE.md is 2 REVISE-PENDING for non-critical sections; pack-activation race not documented |
| ML model degradation / provider circuit-open | `intelligence`, `analytics`, `social`, `shorts`, `translate`, `recordings` | PARTIAL — `intelligence` compliance.md has `§pack-overlay-roster` with provider routing but no model-degradation fallback; `analytics` compliance.md has `§ml-model-lifecycle` but ARCHITECTURE.md is boilerplate |
| Audit-chain Merkle-seal failure / seal-gap recovery | `audit-chain`, `payments`, `foundry` | NONE — `audit-chain` ARCHITECTURE.md all boilerplate; `payments` §critical-path-edge-cases is REVISE-PENDING |
| MLS key-bundle exhaustion / epoch-rotation stall | `messenger` | NONE — `messenger` ARCHITECTURE.md all boilerplate; no MLS failure mode documented |
| Bootstrap-tier-1 SPIFFE attestation failure | `cell`, `foundry`, `tenancy`, `identity` | PARTIAL — `cell` compliance.md has `§bootstrap-trust-chain`; `tenancy` compliance.md has `§bootstrap-trust-chain`; both are substantive; SPIFFE failure fallback not in §critical-path-edge-cases |
| provider-BYOK credential expiry under pack-mandatory-byok | `intelligence`, `payments`, `cloud-secrets` | PARTIAL — `intelligence` compliance.md §HIPAA has BAA provider constraint; `payments` compliance.md §credential-isolation present; edge case behavior on expiry not documented |

### §8.2 Per-µservice critical-path section depth assessment

**Substrate µservices with thin or boilerplate §critical-path-edge-cases:**

**cell (111-line ARCHITECTURE.md, all boilerplate):**
The cell substrate is Tier 0. It manages cell routing, shard assignment, and
shuffle-sharding parameters for every other µservice. Its
`ARCHITECTURE.md §critical-path-edge-cases` is ANCHOR-INJECTED with the generic
text: "Critical-path edge cases for `cell` per documentation-rigor §3.2.5. This
section enumerates ≥3 failure modes..." and nothing else. No failure modes
are listed. An intern cannot understand what happens when a cell goes dark,
when shard-routing tables diverge between regions, or when shuffle-sharding
parameters are updated while requests are in flight. This is a P0 documentation
gap for a Tier 0 substrate.

**foundry (127-line ARCHITECTURE.md, all boilerplate):**
The Foundry pipeline is the agentic execution substrate for all CI/CD and
multi-agent coordination. Its `§critical-path-edge-cases` is ANCHOR-INJECTED
boilerplate. No pipeline-failure cascade documented. No Foundry self-modification
rollback documented. No meta-trust-root compromise scenario documented. The
Foundry µservice has 561 artifacts (the richest artifact count in the corpus
per the §1 snapshot table) but its ARCHITECTURE.md is 127 lines of boilerplate.

**audit-chain (103-line ARCHITECTURE.md, all boilerplate):**
The audit-chain substrate provides tamper-evident Merkle-sealed audit logs
required by every compliance pack. Its ARCHITECTURE.md is 103 lines —
the minimum floor — and entirely boilerplate. No seal-gap recovery, no Merkle
tree repair procedure, no chain-fork resolution documented. For a substrate
that every regulated workload depends on for compliance evidence, this is a P0
gap.

**identity (119-line ARCHITECTURE.md, all boilerplate):**
The identity substrate gates every authenticated request across 46 µservices.
Its ARCHITECTURE.md has a passing PRD (1642 lines, 42 stories) but an all-boilerplate
ARCHITECTURE.md. The token-issuance failure cascade, OIDC provider outage
fallback, passkey attestation failure, and step-up-auth class downgrade risks
are not documented anywhere in the 14 ANCHOR-INJECTED sections.

**tenancy (195-line ARCHITECTURE.md, 2 REVISE-PENDING):**
The tenancy µservice manages tenant lifecycle, provisioning, and isolation.
With 2 REVISE-PENDING sections (the non-boilerplate sections are present and
substantive — 25 total anchors, 25 ADR cites), this is closer to completion
than the fully-boilerplate substrates. The 2 REVISE-PENDING sections are
`§cellular-architecture` (per ADR-0248) and `§day-one-cert-readiness`
(per ADR-0250). Both are important for a substrate that must operate correctly
at cell-tier boundaries and must be certification-ready on day one.

---

## §9 Supersede Candidates

### §9.1 ARCHITECTURE.md files requiring `doc_status: draft` designation

The following 32 ARCHITECTURE.md files should have their frontmatter updated to
add `doc_status: draft` (they are current files but not canonical — they are
scaffolding placeholders not yet expanded to intern-buildable documentation):

`analytics`, `anonymous`, `application`, `audit-chain`, `calendar`, `cell`,
`cloud-iac`, `cloud-k8s`, `cloud-secrets`, `community`, `consent-graph`,
`developer-sdk`, `docs`, `drive`, `forms`, `foundry`, `governance`, `identity`,
`meet`, `messenger`, `network`, `observability`, `plugin-app-store`,
`recordings`, `sheets`, `shorts`, `sites`, `slides`, `tasks`, `translate`,
`workflow-engine`, `workflow-studio`

These MUST NOT be moved to `superseded/` — they are the current files. They
need in-place expansion to clear all REVISE-PENDING markers.

### §9.2 PRD.md stub files requiring `doc_status: stub` designation

The 39 STUB PRDs should have their frontmatter updated to add `doc_status: stub`
and a `stub_rewrite_target: wave-3-d` field. This signals to the CI lane that
the file is known-incomplete and is queued for Wave-3-D rewrite.

Notable: `analytics/PRD.md` has no `doc_class` frontmatter field — it is a
bare markdown file without YAML frontmatter, which means it fails the
`governance-doc-link-resolves` frontmatter validation entirely.
`ops-dashboard-control-center/PRD.md` uses `doc_class: Product-Requirements`
(non-canonical; canonical is `PRD`) and `status: accepted-design-anchor`
(non-canonical; canonical is `Proposed`).

### §9.3 manifest.json old-schema files — rewrite not supersession

The 39 old-schema manifest files should be rewritten in place to the
documentation-rigor.md §1 schema. No supersession move. `analytics/manifest.json`
(unique pre-PR-143 schema) should archive its old form to
`analytics/superseded/manifest-pre-pr143.json` with the header:
`Status: Superseded by manifest.json (2026-05-21 schema migration)`.

### §9.4 Content superseded by 2026-05-20 keystone bundle

All PRD.md and compliance.md files citing ADR-0145 without the corresponding
retirement note are stale as of 2026-05-18. The citation itself need not be
removed (historical record) but each file must add:
`# ADR-0145 retirement note: retired 2026-05-18 per feedback_workflow_objectgraph_adapter_layer; direct gRPC replaces the forced-adapter pattern; citations in this file are historical.`

---

## §10 Recommended Wave-3-D Remediation Order

Ordered by substrate-tier impact. Substrate µservices first because their
documentation deficiencies cascade into all dependent µservices' six-hops
traversal. Within each tier, ordered by severity (most axes failing, highest
traffic criticality).

### §10.1 Corpus-wide batch actions (execute before per-µservice work)

These 6 batch PRs have the highest ROI — they fix structural gaps across
multiple µservices in a single pass and unblock per-µservice authoring work.

| # | Action | Affected count | Effort | Expected outcome |
|---|---|---:|---|---|
| B1 | **manifest.json schema migration** — migrate all 39 old-schema manifests to documentation-rigor.md §1 schema (add `tier`, `audience_type`, `layer_enum_conformance`, `cell_eligibility`, `substrate_dependencies`, `compliance_packs`, `naming_justifications`) | 39 | 1 PR, mechanical | manifest.json field coverage jumps from 0/46 PASS to potentially 39/46 PASS |
| B2 | **compliance.md `§ml-model-lifecycle` + `§detection-fairness-audit` sweep** — add both missing anchors with substantive content to all 34 compliance.md files missing them | 34 | 1 PR, 68 new sections | compliance.md REVISE count drops from 34 to ≤5 |
| B3 | **ARCHITECTURE.md `§intelligence-dispatch` + `§ontology-read-path` sweep** — add both missing anchors (explicit negative declaration or substantive content) to the 11 ARCHITECTURE.md files missing them | 11 | 1 PR, 22 new sections | Required-14 present jumps to 14/14 for all 46 µservices |
| B4 | **Retired-tool citation sweep** — replace `retired VCS ratchet claim/work/done` boilerplate blocks with plain `git`/`gh` references; add ADR-0145 retirement note to all 31 PRDs and 23 compliance.md files citing it | 54 | 1 PR, sed-replaceable | Staleness findings §7.1.1 and §7.1.2 cleared |
| B5 | **placeholder markers removal** — resolve or remove all 68 occurrences across 14 µservices | 14 | 1 PR, per-item resolution | Completeness invariant 7 cleared for 14 µservices |
| B6 | **`doc_status` frontmatter sweep** — add `doc_status: draft` to 32 boilerplate ARCHITECTURE.md; `doc_status: stub` + `stub_rewrite_target: wave-3-d` to 39 stub PRDs | 71 | 1 PR, mechanical | CI lane correctly reports draft/stub state; docs stop falsely appearing published |

### §10.2 Tier 0 — Substrate-critical (block platform GA)

These µservices are on the critical path of every authenticated request and
every compliance evidence chain. Their documentation deficiency propagates to
all 45 dependents via the six-hops traversal invariant.

| Priority | µservice | PRD | ARCH | compliance | manifest | Recommended action |
|---|---|---|---|---|---|---|
| P0-1 | **cell** | STUB 425 lines, 0 stories | All boilerplate (13 REVISE-PENDING); missing `intelligence-dispatch` + `ontology-read-path` | REVISE (missing 2 anchors) | 0/6 fields | Full Wave-3-D: PRD rewrite (target ≥1500 lines, ≥40 stories, §A–§J); ARCH full substantive expansion; add 2 missing anchors; add 2 compliance anchors; manifest migration; document Cedar timeout fallback + shard-routing failure |
| P0-2 | **tenancy** | STUB 511 lines, 0 stories | 2 REVISE-PENDING; substantive 25 anchors | REVISE (missing 2 anchors) | 0/6 fields | PRD rewrite; resolve 2 ARCH REVISE-PENDING; add 2 compliance anchors; manifest migration |
| P0-3 | **identity** | PASS 1642 lines | All boilerplate (14 REVISE-PENDING); 17/28 ADR score | REVISE (missing 2 anchors) | 0/6 fields | ARCH full substantive expansion (all 14 boilerplate sections); add 2 compliance anchors; manifest migration; document token-issuance failure cascade |
| P0-4 | **foundry** | STUB 388 lines, 0 stories | All boilerplate (15 REVISE-PENDING); 20/28 ADR score | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | PRD rewrite; ARCH full substantive expansion; manifest migration; document pipeline-failure cascade + meta-trust-root compromise scenario |
| P0-5 | **cloud-secrets** | STUB 363 lines, 0 stories | All boilerplate (12 REVISE-PENDING); missing 2 required anchors | REVISE (missing 2 anchors); 25 total anchors | 0/6 fields | Full Wave-3-D: PRD + ARCH (add 2 missing + expand all 12 boilerplate sections) + compliance (add 2 anchors) + manifest; document key-rotation race + FIPS-140-3 boundary |
| P0-6 | **audit-chain** | STUB 400 lines, 0 stories | All boilerplate (12 REVISE-PENDING); missing 2 required anchors | REVISE (missing 2 anchors) | 0/6 fields | Full Wave-3-D: PRD + ARCH (add 2 missing + expand) + compliance (add 2 anchors) + manifest; document Merkle-seal failure + chain-fork recovery |
| P0-7 | **intelligence** | STUB 38 lines — worst in corpus | APPROVE-WITH-FINDINGS (0 REVISE-PENDING, 36 ADR cites, 484 lines); retired ADR-0145 citation | REVISE — 6 anchors missing; worst compliance gap for a substrate | 4/6 fields; `cell_eligibility` type non-conforming | PRD complete rewrite (38 lines is catastrophically thin); fix compliance.md 6 missing anchors; remove ADR-0145 citation; normalize `cell_eligibility` to ADR-0248 §D-1 enum; add `layer_enum_conformance`, `substrate_dependencies`, `compliance_packs` to manifest |
| P0-8 | **observability** | STUB 309 lines, 0 stories | All boilerplate (12 REVISE-PENDING); missing 2 required anchors | REVISE (missing 2 anchors) | 0/6 fields | Full Wave-3-D; this is the canonical exemplar µservice — its own docs must be exemplar-grade before it can serve as the template |

### §10.3 Tier 1 — Substrate-general

| Priority | µservice | Key gaps | Recommended action |
|---|---|---|---|
| P1-1 | **network** | STUB 462 lines; ARCH all boilerplate + 2 missing anchors; 14/28 ADR score | Full Wave-3-D |
| P1-2 | **cloud-iac** | STUB 443 lines; ARCH all boilerplate + 2 missing anchors | Full Wave-3-D |
| P1-3 | **cloud-k8s** | STUB 387 lines; ARCH all boilerplate + 2 missing anchors | Full Wave-3-D |
| P1-4 | **application** | STUB 382 lines; ARCH all boilerplate + 2 missing anchors | Full Wave-3-D |
| P1-5 | **consent-graph** | STUB 280 lines; ARCH all boilerplate + 2 missing anchors; `cross_microservice_dependencies` extension (not in canonical schema) | Full Wave-3-D; normalize manifest extension |
| P1-6 | **governance** | STUB 419 lines; ARCH all boilerplate | Full Wave-3-D |
| P1-7 | **compliance** | STUB 127 lines; ARCH APPROVE-WITH-FINDINGS (24 anchors, 2 REVISE-PENDING, 24 ADR cites); 24/28 ADR score | PRD rewrite; resolve 2 ARCH REVISE-PENDING; add `ml-model-lifecycle`/`detection-fairness-audit`; manifest migration |
| P1-8 | **feature-flags** | STUB 116 lines; ARCH APPROVE-WITH-FINDINGS (0 REVISE-PENDING, 34 ADR cites, 525 lines, 28/28 ADR score) | PRD rewrite (primary gap); fix `cell_eligibility` type; add missing manifest fields (`audience_type`, `layer_enum_conformance`, `substrate_dependencies`, `compliance_packs`) |

### §10.4 Tier 2 — Product-substrate boundary

| Priority | µservice | Key gaps | Recommended action |
|---|---|---|---|
| P2-1 | **api-gateway** | STUB 117 lines; ARCH APPROVE-WITH-FINDINGS (2 REVISE-PENDING, 23 ADR cites, 28/28 ADR score); manifest 4/6 | PRD complete rewrite; resolve 2 ARCH REVISE-PENDING; add `layer_enum_conformance`, `substrate_dependencies`, `compliance_packs` to manifest; remove substantive `oyatie` ARCH string literals |
| P2-2 | **developer-sdk** | STUB 194 lines; ARCH all boilerplate + 2 missing anchors; manifest `tier: "external-facing"` non-standard | Full Wave-3-D; normalize `tier` to `substrate` or `product` |

### §10.5 Tier 3 — Core communication + collaboration products

| Priority | µservice | PRD lines | Stories | ARCH REVISE-PENDING | 28-row score | Recommended action |
|---|---|---:|---:|---:|---:|---|
| P3-1 | **ops-dashboard-control-center** | 49 | 0 | 0 | 27/28 | PRD complete rewrite (49 lines is a design anchor, not a PRD); manifest migration; ARCH already good |
| P3-2 | **finops-portal** | 116 | 0 | 2 | 21/28 | PRD rewrite; resolve 2 ARCH REVISE-PENDING; manifest migration |
| P3-3 | **comms-email** | 183 | 0 | 2 | 22/28 | PRD rewrite; resolve 2 ARCH REVISE-PENDING; manifest migration |
| P3-4 | **forms** | 234 | 0 | 14 | 17/28 | Full Wave-3-D |
| P3-5 | **messenger** | 1718 | 0 | 14 | 17/28 | Add ≥40 `### US-` story anchors; add MLS/E2EE compliance sections (ADR-0246); migrate to §A–§J; ARCH full substantive expansion; manifest migration |
| P3-6 | **mail** | 1545 | 0 | 2 | 26/28 | Add ≥40 `### US-` story anchors; migrate to §A–§J; resolve 2 ARCH REVISE-PENDING; manifest migration |
| P3-7 | **community** | 1449 | 20 | 14 | 18/28 | Add ≥20 stories to floor; +51 lines to floor; migrate to §A–§J; ARCH full expansion; manifest migration |
| P3-8 | **meet** | 357 | 0 | 14 | 17/28 | Full Wave-3-D |
| P3-9 | **social** | 397 | 0 | 2 | 25/28 | PRD rewrite; resolve 2 ARCH REVISE-PENDING; manifest migration |
| P3-10 | **calendar** | 326 | 0 | 14 | 17/28 | Full Wave-3-D |
| P3-11 | **anonymous** | 387 | 0 | 12 | 15/28 | Full Wave-3-D |

### §10.6 Tier 4 — Productivity platform

All 7 are STUB PRDs with 0 stories and all-boilerplate ARCHITECTURE.md.
Ordered by substrate-adjacency (docs and drive are cross-product; notes, tasks,
sheets are standalone):

| Priority | µservice | PRD lines | ARCH REVISE-PENDING | 28-row score |
|---|---|---:|---:|---:|
| P4-1 | **docs** | 387 | 14 | 17/28 |
| P4-2 | **drive** | 409 | 14 | 17/28 |
| P4-3 | **tasks** | 383 | 14 | 17/28 |
| P4-4 | **notes** | 400 | 2 | 25/28 |
| P4-5 | **sheets** | 597 | 14 | 17/28 |
| P4-6 | **slides** | 518 | 14 | 17/28 |
| P4-7 | **sites** | 400 | 14 | 17/28 |

`notes` is closest to completion (2 REVISE-PENDING, 25/28 ADR score) — resolve
its 2 ARCH gaps and add stories first. `sheets` at 597 lines is the most
developed stub by line count and may require the least PRD prose work.

### §10.7 Tier 5 — Media and specialist products

| Priority | µservice | PRD lines | ARCH REVISE-PENDING | 28-row score |
|---|---|---:|---:|---:|
| P5-1 | **recordings** | 469 | 14 | 18/28 |
| P5-2 | **shorts** | 418 | 14 | 18/28 |
| P5-3 | **translate** | 311 | 14 | 18/28 |
| P5-4 | **plugin-app-store** | 205 | 15 | 18/28 |
| P5-5 | **workflow-studio** | 528 | 15 | 18/28 |

`workflow-studio` at 528 lines is most developed by line count in this tier.
`translate` needs `§ml-model-lifecycle` authoring — it has the anchor in
compliance.md but ARCH is all boilerplate.

### §10.8 Tier 6 — Reference-bar µservices with ARCH/manifest gaps

The 4 PASS PRDs coexist with incomplete ARCHITECTURE.md and manifest.json files.
These are the highest-impact finish-line items because the PRD work is done and
only ARCH + manifest remediation is needed.

| Priority | µservice | PRD | ARCH gap | compliance gap | manifest gap |
|---|---|---|---|---|---|
| P6-1 | **workflow-engine** | PASS 1596 lines | All boilerplate (15 REVISE-PENDING); 17/28 ADR score | REVISE (missing 2 anchors) | 0/6 fields |
| P6-2 | **ontology** | PASS 1539 lines | APPROVE-WITH-FINDINGS (2 REVISE-PENDING, 28 ADR cites) | REVISE (missing 2 anchors) | 0/6 fields |
| P6-3 | **payments** | PASS 1612 lines | APPROVE-WITH-FINDINGS (3 REVISE-PENDING, 30 ADR cites); retired ADR-0145 citation | REVISE (missing 2 anchors) | 2/6 fields |
| P6-4 | **anonymous** | STUB (already P3-11) | — | — | — |

For `workflow-engine`: the ARCH is entirely boilerplate despite the PRD being
the reference bar. The Temporal-class deterministic replay, saga compensation,
and multi-domain execution semantics described in the PRD have no corresponding
ARCHITECTURE.md prose. This is the widest ARCH/PRD gap in the corpus (PRD
describes a sophisticated durable execution engine; ARCH says "operates under
`oyatie.workflow-engine.<role>` principals").

---

## §11 Summary Statistics

### §11.1 Per-axis corpus scores

| Axis | PASS | APPROVE-WITH-FINDINGS | REVISE/STUB | Pass % |
|---|---:|---:|---:|---:|
| PRD (≥1500 lines + ≥40 stories + §A–§J) | 4 | 3 (BORDERLINE) | 39 | 9% |
| ARCHITECTURE.md (14 anchors + 0 boilerplate + ≥15 ADR cites) | 0 | 14 | 32 | 0% |
| compliance.md (all 15 required anchors) | 12 | — | 34 | 26% |
| manifest.json (all 6 fields + naming_justifications) | 0 | 2 | 44 | 0% |
| 28-row ADR matrix (28/28 citations) | 3 (`api-gateway`, `feature-flags`, `intelligence`) | — | 43 | 7% |

### §11.2 Coverage histograms

**ARCHITECTURE.md anchor count distribution:**

| Anchor count | µservices | Percent |
|---|---:|---:|
| 12 anchors (missing 2 required) | 11 | 24% |
| 13 anchors (missing 1 required) | 1 (cell) | 2% |
| 14 anchors (all 14 required present) | 22 | 48% |
| 15–25 anchors (all 14 + extras) | 12 | 26% |

**ARCHITECTURE.md REVISE-PENDING count distribution:**

| REVISE-PENDING count | µservices | Percent |
|---|---:|---:|
| 0 (no boilerplate) | 5 (`feature-flags`, `intelligence`, `ops-dashboard-control-center`, and 2 others*) | 11% |
| 2–3 (partial boilerplate) | 9 | 20% |
| 12–15 (all sections boilerplate) | 32 | 70% |

*`api-gateway` and others with 2 REVISE-PENDING are in the partial boilerplate group.

**28-row ADR matrix score distribution:**

| Score | µservices | Percent |
|---|---:|---:|
| 28/28 (full coverage) | 3 | 7% |
| 25–27/28 | 9 | 20% |
| 20–24/28 | 5 | 11% |
| 14–19/28 | 29 | 63% |

**compliance.md required-anchor coverage distribution:**

| Anchors present (of 15 required) | µservices | Percent |
|---|---:|---:|
| 15/15 (all required) | 12 | 26% |
| 13/15 (missing ml-model-lifecycle + detection-fairness-audit) | 29 | 63% |
| 9/15 (missing 6 — intelligence) | 1 | 2% |
| < 9/15 | 4 | 9% |

### §11.3 Cross-µservice consistency scorecard (§3.2.2 invariants 1–10)

| Invariant | Status | Scope of failure |
|---|---|---|
| 1 — Field naming | FAIL | 44/46 manifests use old schema; 3 non-standard `tier` values |
| 2 — Audit-event taxonomy | CANNOT VERIFY | Central registry not found; 32 ARCH boilerplate |
| 3 — OpenAPI/AsyncAPI versions | PASS (existing files) | No violations in authored contracts |
| 4 — OpenBao path shape | CANNOT VERIFY | ARCH boilerplate defers; IaC not audited |
| 5 — Cell-tier enum | FAIL | 43/46 missing; 3 present use 3 different non-enum shapes |
| 6 — Compliance-pack-id | FAIL | 3 different naming conventions across 3 µservices with pack data |
| 7 — Layer-enum ADR-0105 | CANNOT VERIFY | `layer_enum_conformance` absent from 44/46 manifests |
| 8 — Naming-justification tables | FAIL | 42/46 manifests lack `naming_justifications` |
| 9 — Six-hops traversal | FAIL | `substrate_dependencies` absent from 44/46; ARCH boilerplate breaks traversal |
| 10 — BYOK disambiguation | PARTIAL FAIL | `provider_credential_mode` absent from 46/46 manifests; 44/46 non-disambiguating |

### §11.4 Overall corpus verdict

**No µservice currently passes all four file-class axes simultaneously.**

The closest to full-pass is `payments`: PRD PASS + ARCH APPROVE-WITH-FINDINGS
(3 REVISE-PENDING remaining, 30 ADR cites, 500 lines) + compliance.md REVISE
(missing `ml-model-lifecycle`/`detection-fairness-audit`) + manifest 2/6.

The four PASS PRDs (`identity`, `ontology`, `payments`, `workflow-engine`) all
have manifest 0/6 or 2/6, and three of the four have ARCH REVISE or
all-boilerplate ARCHITECTURE.md.

**Overall corpus Axis-B (ADR-adherence) score: REVISE across 43 of 46 µservices.**

### §11.5 Remediation workload before 2026-07-16 BLOCKER

| Work category | µservice count | Nature |
|---|---:|---|
| Full Wave-3-D PRD rewrite (0 stories, <800 lines) | 37 | Substantive authoring — cannot be automated |
| PRD structural fix only (BORDERLINE — add stories + restructure) | 5 | Mix of authoring and reformatting |
| ARCHITECTURE.md full expansion (all boilerplate) | 32 | Substantive authoring per section per µservice |
| ARCHITECTURE.md 2-anchor addition (`intelligence-dispatch` + `ontology-read-path`) | 11 | Can batch (B3 action above) |
| ARCHITECTURE.md partial completion (2–3 REVISE-PENDING) | 9 | Targeted prose for specific sections |
| compliance.md 2-anchor addition (`ml-model-lifecycle` + `detection-fairness-audit`) | 34 | Can batch (B2 action above) |
| compliance.md 6-anchor addition (intelligence) | 1 | Substantive authoring |
| manifest.json schema migration | 39 | Mechanical — single batch PR |
| manifest.json partial completion | 5 | Targeted field additions |
| placeholder markers/placeholder marker removal | 14 | Resolve or remove per item |
| Retired-citation cleanup | 31+ | Sed-replaceable for most |

The 6 batch actions in §10.1 address the mechanical gaps (manifests, compliance
anchors, ARCH anchors, citation cleanup, placeholder marker removal, doc_status sweep) in
approximately 6 PRs. The substantive authoring workload (PRD rewrites and ARCH
expansion) is the long pole — it requires per-µservice domain knowledge and
cannot be batched or automated.

**Minimum viable corpus state before 2026-07-16:** Complete the 6 batch actions
in §10.1, then complete the P0 substrate PRD rewrites and ARCH expansions
(cell, tenancy, identity, foundry, cloud-secrets, audit-chain, intelligence,
observability). This brings the 8 most critical substrates to a passing state
and unblocks platform GA gating. The remaining 38 µservices can continue through
Tiers 3–6 on the post-GA roadmap.

---

---

## §12 Per-µservice Cross-File Consistency Profiles

This section provides a four-axis summary for every µservice, showing the
relationship between each file class rating and identifying the dominant gap
axis. Ordering follows §10 remediation tiers (P0 first).

### §12.1 Tier 0 — Substrate-critical

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap | File needing most work |
|---|---|---|---|---|---:|---|---|
| cell | STUB (425 lines, 0 stories) | REVISE (13 RP, 17/28 ADR, 2 anchors missing) | REVISE (13/15 anchors) | 0/6 fields | 1196 | PRD + ARCH both catastrophic | PRD (425→1500+) |
| tenancy | STUB (511 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 25/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1431 | PRD has no stories at all | PRD (511→1500+) |
| identity | PASS (1642 lines, 42 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 2358 | ARCH all boilerplate despite passing PRD | ARCH (119→500+) |
| foundry | STUB (388 lines, 0 stories) | REVISE (15 RP, 20/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1897 | PRD thin + ARCH boilerplate | PRD (388→1500+) |
| cloud-secrets | STUB (363 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 15/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1147 | Worst ARCH anchor gap in Tier 0 | ARCH (add 2 anchors + expand 12 RP) |
| audit-chain | STUB (400 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1143 | ARCH anchor gap + P0 seal-gap scenario missing | ARCH + compliance |
| intelligence | STUB (38 lines — corpus minimum) | APPROVE-WITH-FINDINGS (0 RP, 36 ADR, 484 lines) | REVISE (9/15 anchors — corpus minimum) | 4/6 fields | 1256 | PRD catastrophically thin; compliance worst-case | PRD (38→1500+) |
| observability | STUB (309 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1107 | Exemplar µservice with exemplar-grade gaps | All four axes |

**Key cross-file inconsistency for Tier 0:** `identity` has the strongest PRD
in the corpus (1642 lines, 42 stories, full §A–§J) paired with the weakest
ARCHITECTURE.md for a substrate service (119 lines, all boilerplate, 0 of 14
sections substantive). The token-issuance story is fully specified in the PRD;
the ARCH says nothing about how it is implemented or what happens when it fails.
`intelligence` is the mirror case: its ARCHITECTURE.md is the second-richest
in the corpus (484 lines, 0 boilerplate, 36 ADR cites) paired with a 38-line
PRD — the weakest in the corpus.

### §12.2 Tier 1 — Substrate-general

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| network | STUB (462 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1375 | All axes deficient |
| cloud-iac | STUB (443 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1293 | All axes deficient |
| cloud-k8s | STUB (387 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1215 | All axes deficient |
| application | STUB (382 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1151 | All axes deficient |
| consent-graph | STUB (280 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 0/6 fields (+ non-standard extension) | 1000 | Lowest total corpus lines in Tier 1 |
| governance | STUB (419 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1246 | PRD + ARCH; compliance is bright spot |
| compliance | STUB (127 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 24/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 638 | PRD at 127 lines — second-lowest line count for a substrate |
| feature-flags | STUB (116 lines, 0 stories) | APPROVE-WITH-FINDINGS (0 RP, 28/28 ADR, 525 lines) | APPROVE-WITH-FINDINGS (15/15 anchors) | 3/6 fields (type mismatch on `cell_eligibility`) | 1185 | PRD only; ARCH is reference model |

**Key cross-file inconsistency for Tier 1:** `feature-flags` has the best
ARCHITECTURE.md in the corpus and a passing compliance.md — but a 116-line
PRD (second-worst stub in the full corpus). The contrast is stark: 525 lines
of substantive ARCH vs. 116 lines of PRD that describe almost no user-facing
requirement for the feature-flag management surface.

`compliance` and `feature-flags` together illustrate the inversion problem:
infrastructure µservices received substantive ARCH authoring because engineers
focused on the technical design, while the PRDs were left as minimal placeholders
because the "product" of these services is internal.

### §12.3 Tier 2 — Product-substrate boundary

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| api-gateway | STUB (117 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 28/28 ADR, 321 lines) | REVISE (13/15 anchors) | 4/6 fields | 834 | PRD; ARCH + manifest closest to complete |
| developer-sdk | STUB (194 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 14/28 ADR) | REVISE (13/15 anchors) | 1/6 fields (non-standard `tier` value) | 722 | All axes; non-standard tier |

### §12.4 Tier 3 — Core communication + collaboration

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| ops-dashboard-control-center | STUB (49 lines — design anchor) | APPROVE-WITH-FINDINGS (0 RP, 27/28 ADR, 346 lines) | REVISE (13/15 anchors) | 0/6 fields | 1109 | PRD; ARCH is near-complete |
| finops-portal | STUB (116 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 21/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 502 | PRD; lowest total corpus lines in Tier 3 |
| comms-email | STUB (183 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 22/28 ADR, 164 lines) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 691 | PRD; compliance is bright spot |
| forms | STUB (234 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 958 | All axes |
| messenger | BORDERLINE (1718 lines, 0 `### US-` stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 2502 | PRD story formatting + MLS section missing |
| mail | BORDERLINE (1545 lines, 0 `### US-` stories) | APPROVE-WITH-FINDINGS (2 RP, 26/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 2571 | PRD story formatting; ARCH + compliance good |
| community | BORDERLINE (1449 lines, 20 stories) | REVISE (14 RP, 18/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 2191 | 20 more stories needed + ARCH expansion |
| meet | STUB (357 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1175 | All axes |
| social | STUB (397 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 25/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1307 | PRD; ARCH + compliance are bright spots |
| calendar | STUB (326 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1303 | All axes |
| anonymous | STUB (387 lines, 0 stories) | REVISE (12 RP, 2 anchors missing, 15/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1119 | All axes |

### §12.5 Tier 4 — Productivity platform

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| docs | STUB (387 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1317 | All axes |
| drive | STUB (409 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1345 | All axes |
| tasks | STUB (383 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors + 13 placeholder marker items) | 0/6 fields | 1301 | All axes; compliance placeholder marker count (13) is corpus-high |
| notes | STUB (400 lines, 0 stories) | APPROVE-WITH-FINDINGS (2 RP, 25/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1241 | PRD; ARCH near-complete |
| sheets | STUB (597 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1453 | All axes; highest-line STUB PRD in this tier |
| slides | STUB (518 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1348 | All axes |
| sites | STUB (400 lines, 0 stories) | REVISE (14 RP, 17/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1254 | All axes |

### §12.6 Tier 5 — Media and specialist

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| recordings | STUB (469 lines, 0 stories) | REVISE (14 RP, 18/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1213 | PRD + ARCH; compliance is bright spot |
| shorts | STUB (418 lines, 0 stories) | REVISE (14 RP, 18/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1260 | PRD + ARCH; compliance is bright spot |
| translate | STUB (311 lines, 0 stories) | REVISE (14 RP, 18/28 ADR) | APPROVE-WITH-FINDINGS (15/15 anchors) | 0/6 fields | 1144 | PRD + ARCH |
| plugin-app-store | STUB (205 lines, 0 stories) | REVISE (15 RP, 18/28 ADR) | REVISE (13/15 anchors) | 1/6 fields (non-standard `tier` value) | 778 | All axes; non-standard tier |
| workflow-studio | STUB (528 lines, 0 stories) | REVISE (15 RP, 18/28 ADR) | REVISE (13/15 anchors) | 0/6 fields | 1375 | All axes; highest-line PRD in this tier |

### §12.7 Tier 6 — Reference-bar µservices with open ARCH/manifest gaps

| µservice | PRD | ARCH | compliance.md | manifest | Total corpus lines | Dominant gap |
|---|---|---|---|---|---:|---|
| payments | PASS (1612 lines, 42 stories) | APPROVE-WITH-FINDINGS (3 RP, 27/28 ADR, 500 lines) | REVISE (13/15 anchors) | 2/6 fields | 2852 | compliance (2 missing anchors) + manifest (4 missing fields) |
| ontology | PASS (1539 lines, 42 stories) | APPROVE-WITH-FINDINGS (2 RP, 28 ADR cites, 177 lines) | REVISE (13/15 anchors) | 0/6 fields | 2470 | compliance (2 missing anchors) + manifest (6 missing fields) |
| workflow-engine | PASS (1596 lines, 42 stories) | REVISE (15 RP, 17/28 ADR, 127 lines) | REVISE (13/15 anchors) | 0/6 fields | 2421 | ARCH is entirely boilerplate; PRD/ARCH gap widest in corpus |

**The reference-bar ARCH/PRD gap:** `workflow-engine` PRD describes Temporal-class
durable execution with saga compensation, deterministic replay, multi-domain
orchestration, activity heartbeat semantics, and cross-tenant workflow isolation.
Its ARCHITECTURE.md is 127 lines of scaffolding boilerplate that says nothing
about any of these mechanisms. An engineer implementing the Temporal executor
would find a fully specified product requirement and zero architectural guidance.

`payments` is closest to passing all four axes: 1612-line PASS PRD, 500-line
APPROVE-WITH-FINDINGS ARCH, 435-line compliance.md (richest in corpus) — blocked
only by 2 missing compliance anchors (`ml-model-lifecycle`,
`detection-fairness-audit`) and 4 missing manifest fields. Of all 46 µservices,
`payments` requires the least effort to reach full-pass.

---

## §13 PRD Section Structure Gap Matrix

### §13.1 Required section presence by µservice

The `§A–§J` structure is required by documentation-rigor.md §2. `§A` = Problem,
`§B` = Target users, `§C` = User stories, `§D` = Functional requirements,
`§E` = Non-functional requirements, `§F` = UX flows, `§G` = Success metrics,
`§H` = Compliance impact, `§I` = Open questions, `§J` = Out-of-scope.

Section detection: `grep "^## [A-J]"` per PRD.md.

`Y` = section present (regardless of content depth); `.` = absent.

| µservice | §A | §B | §C | §D | §E | §F | §G | §H | §I | §J | Present/10 |
|---|---|---|---|---|---|---|---|---|---|---|---:|
| analytics | . | . | . | . | . | . | . | . | . | . | 0 |
| anonymous | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| api-gateway | Y | . | Y | . | . | Y | . | . | . | . | 3 |
| application | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| audit-chain | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| calendar | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| cell | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| cloud-iac | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| cloud-k8s | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| cloud-secrets | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| comms-email | . | . | . | . | . | . | . | . | . | . | 0 |
| community | Y | Y | Y | . | . | Y | Y | Y | Y | . | 7 (numbered, not §A–§J) |
| compliance | Y | . | Y | . | . | Y | Y | . | . | . | 4 |
| connector | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 10 |
| consent-graph | . | . | . | . | . | . | . | . | . | . | 0 |
| developer-sdk | Y | Y | Y | . | . | Y | . | . | . | . | 4 |
| docs | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| drive | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| feature-flags | Y | . | Y | . | . | Y | . | . | . | . | 3 |
| finops-portal | . | . | Y | . | . | . | . | . | Y | . | 2 |
| forms | . | . | . | . | . | . | . | . | . | . | 0 |
| foundry | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| governance | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| identity | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 10 |
| intelligence | . | . | . | . | . | . | . | . | . | . | 0 (§Purpose + §Scope only) |
| mail | Y | Y | Y | . | . | Y | Y | Y | Y | . | 7 (numbered, not §A–§J) |
| meet | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| messenger | Y | Y | Y | . | . | Y | Y | Y | Y | . | 7 (numbered, not §A–§J) |
| network | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| notes | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| observability | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| ontology | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 10 |
| ops-dashboard-control-center | . | . | . | . | . | . | . | . | . | . | 0 (§Purpose + §Scope + §Users + §Acceptance + §Exit) |
| payments | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 10 |
| plugin-app-store | Y | Y | Y | . | . | Y | . | . | . | . | 4 |
| recordings | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| sheets | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| shorts | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| sites | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| slides | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| social | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| tasks | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| tenancy | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |
| translate | Y | Y | Y | . | . | Y | . | . | . | . | 4 |
| workflow-engine | Y | Y | Y | Y | Y | Y | Y | Y | Y | Y | 10 |
| workflow-studio | Y | Y | Y | . | . | Y | . | Y | Y | . | 6 |

### §13.2 Section-gap summary

| Section | Present in N µservices | Absent in N µservices | % present |
|---|---:|---:|---:|
| §A — Problem | 38 | 8 | 83% |
| §B — Target users | 35 | 11 | 76% |
| §C — User stories | 38 | 8 | 83% |
| §D — Functional requirements | 5 | 41 | 11% |
| §E — Non-functional requirements | 5 | 41 | 11% |
| §F — UX flows | 37 | 9 | 80% |
| §G — Success metrics | 11 | 35 | 24% |
| §H — Compliance impact | 32 | 14 | 70% |
| §I — Open questions | 35 | 11 | 76% |
| §J — Out-of-scope | 5 | 41 | 11% |

**§D, §E, §J are each present in only 5 of 46 PRDs (11%).** These three sections
are the hallmark of a mature product requirements document:
- §D (Functional requirements): forces explicit enumeration of functional
  acceptance criteria beyond user stories; present only in `connector`, `identity`,
  `ontology`, `payments`, `workflow-engine`.
- §E (Non-functional requirements / six-dimension matrix): forces latency,
  throughput, scalability, maintainability, observability, code-quality targets
  in structured form; same 5 µservices only.
- §J (Out-of-scope): forces explicit boundary documentation preventing scope
  creep; same 5 µservices only.

§G (Success metrics) is present in 11 µservices (24%) but in most cases via
numbered-heading format (not `## G`) — the 11 count includes `community`,
`mail`, `messenger`, `compliance`, and 7 others using variants. True §A–§J
canonical §G is only confirmed in the 4 reference-bar PRDs plus `connector`.

§H (Compliance impact) at 70% is the highest-coverage optional section among
the STUB PRDs — it appears even in 6-section stubs, suggesting scaffold tooling
included §H in the default template alongside §A/§C/§F/§I.

### §13.3 Analytics and forms: zero-section PRDs

`analytics` (113 lines), `forms` (234 lines), `consent-graph` (280 lines),
`comms-email` (183 lines), `intelligence` (38 lines), and
`ops-dashboard-control-center` (49 lines) are the six PRDs with zero
`## [A-J]` sections detected. These use either:
(a) entirely prose structure with no heading hierarchy (analytics, forms,
    consent-graph),
(b) purpose/scope-only structure from a different template (intelligence,
    ops-dashboard-control-center), or
(c) short-form feature-spec layout without user-story anchors (comms-email).

None of these six are parseable by `governance-doc-rigor`. The CI
lane cannot find stories, sections, or metrics. All six require complete
structural rewrites, not incremental additions.

### §13.4 connect as an intermediate reference

`connector` (321-line STUB PRD with 10/10 sections, 1 story, 27/28 ADR score) is
anomalous: it has the complete §A–§J structural skeleton with 0 `### US-`
stories. It demonstrates that section structure can be laid down without story
content. For the 32 µservices with 6/10 sections (A, B, C, F, H, I), the path
forward is:
1. Add §D (functional requirements list).
2. Add §E (six-dimension matrix with actual thresholds).
3. Add §G (quantified success metrics with conversion + retention targets).
4. Add §J (explicit out-of-scope boundary).
5. Add ≥40 `### US-` story anchors in §C.

Steps 1–4 can be scaffolded from the reference-bar PRDs. Step 5 is the high-effort
authoring task.

---

## §14 Compliance Pack Coverage Matrix

### §14.1 Pack-mention density per µservice

This section counts how many times each major compliance framework is mentioned
in `compliance.md`. Mention count ≥ 3 indicates substantive coverage; 1–2
indicates citation-only; 0 indicates no coverage.

`KR` = Korea packs (pack-kr, KR-PIPA, KR-ISMS-P, KR-CSAP);
`EU` = EU packs (GDPR, PSD2, EU-AI-Act, Schrems II);
`PCI` = PCI-DSS;
`HIPAA` = HIPAA/BAA/PHI/pack-us-healthcare;
`FedRAMP` = FedRAMP/NIST-800/FISMA;
`SOC2` = SOC 2 Trust Services.

Cells: `S` = substantive (≥3 mentions); `C` = citation-only (1–2); `.` = absent.

| µservice | KR | EU | PCI | HIPAA | FedRAMP | SOC2 | Pack depth |
|---|---|---|---|---|---|---|---|
| analytics | S | S | C | S | C | C | 4S |
| anonymous | C | S | C | S | C | . | 2S |
| api-gateway | S | S | C | S | S | . | 3S |
| application | C | S | C | S | C | . | 2S |
| audit-chain | S | S | C | S | C | . | 3S |
| calendar | S | S | C | S | C | . | 3S |
| cell | S | S | C | S | C | . | 3S |
| cloud-iac | S | S | C | S | C | C | 3S |
| cloud-k8s | S | S | C | S | C | C | 3S |
| cloud-secrets | S | S | S | S | C | . | 4S |
| comms-email | S | S | S | S | C | . | 4S |
| community | S | S | C | S | C | . | 3S |
| compliance | C | S | S | S | C | . | 3S |
| connector | C | S | C | S | C | . | 2S |
| consent-graph | C | S | C | S | C | . | 2S |
| developer-sdk | C | C | C | S | C | . | 1S |
| docs | S | S | C | S | C | . | 3S |
| drive | S | S | C | S | C | . | 3S |
| feature-flags | S | S | S | S | S | . | 5S |
| finops-portal | C | C | C | C | C | . | 0S |
| forms | S | S | C | S | C | . | 3S |
| foundry | S | S | C | S | C | . | 3S |
| governance | S | S | C | S | C | . | 3S |
| identity | S | S | S | S | C | C | 4S |
| intelligence | S | S | C | S | S | C | 4S |
| mail | S | S | C | S | C | C | 3S |
| meet | S | S | C | S | C | . | 3S |
| messenger | S | S | C | S | C | . | 3S |
| network | S | S | C | S | C | . | 3S |
| notes | S | S | C | S | C | . | 3S |
| observability | S | S | C | S | C | C | 3S |
| ontology | S | S | C | S | C | . | 3S |
| ops-dashboard-control-center | C | C | C | S | C | . | 1S |
| payments | S | S | S | C | . | . | 3S (PCI richest) |
| plugin-app-store | C | C | C | S | C | . | 1S |
| recordings | S | S | C | S | C | . | 3S |
| sheets | S | S | C | S | C | . | 3S |
| shorts | S | S | C | S | S | . | 4S |
| sites | S | S | C | S | C | . | 3S |
| slides | C | S | S | S | C | . | 3S |
| social | S | S | C | S | C | . | 3S |
| tasks | S | S | C | S | C | . | 3S |
| tenancy | S | S | C | S | C | C | 3S |
| translate | S | S | C | S | C | . | 3S |
| workflow-engine | S | S | C | S | C | . | 3S |
| workflow-studio | S | S | C | S | C | . | 3S |

### §14.2 Pack coverage gap analysis

**SOC 2 coverage:** Only 7 µservices mention SOC 2 (`analytics`, `cloud-iac`,
`cloud-k8s`, `identity`, `intelligence`, `mail`, `observability`, `tenancy`).
SOC 2 is a baseline enterprise trust certification required for B2B SaaS sales.
The 39 µservices with no SOC 2 mention in their compliance.md do not declare
whether they are in scope for SOC 2 Type II attestation. Per ADR-0250
(build-ahead-of-certification), all µservices must declare their SOC 2 audit
scope on day one.

**PCI-DSS coverage:** 9 µservices have substantive PCI-DSS coverage
(`cloud-secrets`, `comms-email`, `compliance`, `feature-flags`, `identity`,
`payments`, `slides`, `cloud-iac`, `cloud-k8s`). The remaining 37 have citation-only
or no coverage. Per ADR-0251 pack isolation, all µservices that handle
cardholder-environment traffic — even infrastructure µservices not directly
processing card data — must declare their PCI-DSS scope boundary and their
position in the Cardholder Data Environment segmentation.

**FedRAMP coverage:** Only `api-gateway`, `feature-flags`, `intelligence`,
`shorts` have substantive (≥3-mention) FedRAMP coverage. Given that FedRAMP
High is a declared target compliance pack (ADR-0251 §FedRAMP-High), all
Tier 0–1 substrate µservices require substantive FedRAMP section authoring.
The 4 µservices with coverage are all APPROVE-WITH-FINDINGS or better on
compliance.md anchors; the 42 without are primarily REVISE.

**finops-portal: zero substantive pack coverage.** The FinOps portal
compliance.md at 119 lines has citation-only mentions of all six frameworks.
This is the lowest compliance.md pack-depth in the corpus. A cost-visibility
portal that aggregates billing data across all tenants and products is a
significant regulatory surface (financial data under PSD2, GDPR Art.17 erasure,
SOX-ITGC audit trails) and must have substantive coverage.

**developer-sdk and plugin-app-store: 1S each.** Both are external-facing
µservices. The developer SDK is the entry point for all third-party integrations;
the plugin app store is the marketplace surface. Despite being exposed to
external developers and third-party plugins respectively, both have minimal
compliance coverage. `developer-sdk` has only HIPAA as substantive — no EU
GDPR, no KR-PIPA, no PCI-DSS coverage despite SDK developers potentially
building HIPAA, GDPR, and PCI-compliant applications on the platform.

### §14.3 Cross-service pack-id normalization gaps

As identified in §6.6, three different compliance-pack naming conventions
appear in the corpus. The table below shows which µservices use which
convention:

| Pack-id convention | Examples | µservices using this convention |
|---|---|---|
| Short without prefix | `gdpr-eu`, `kr-isms-p`, `fedramp-high` | `feature-flags` (sovereign_packs in manifest cells[]) |
| `pack-*` hyphen-prefixed | `pack-kr`, `pack-eu`, `pack-us-healthcare` | `cloud-secrets`, `observability`, `intelligence`, `tenancy` compliance.md |
| Versioned full-form | `pack-pci-dss-l1-v4`, `pack-eu-psd2-sca`, `pack-kr-fss` | `payments`, `mail`, `identity` compliance.md |

All three conventions co-exist in the corpus. The CI lane
`governance-cross-consistency` invariant 6 check requires canonical
pack-ids from the ADR-0251 registry. The `regulatory_packs` field in old-schema
manifests (used by 39 µservices) uses yet another form (`kr-pipa`, `gdpr-eu`
without the `pack-` prefix). The manifest schema migration (§10.1 batch action
B1) must include a pack-id normalization step that maps all four current forms
to the canonical registry ids.

### §14.4 KR-pack coverage: strongest non-EU regional pack

Korea regulatory packs (`pack-kr`, KR-PIPA, KR-ISMS-P, KR-CSAP) have the
highest substantive-coverage count (31 of 46 µservices with ≥3 mentions) ahead
of EU (30 substantive). This reflects the KR-pack as "pack #1" per
`feedback_canonical_base_localization` — KR was the first localization overlay
and its authoring pattern informed all subsequent pack sections. However, high
mention count does not equal correct pack-id format: the KR compliance.md
sections use `KR-PIPA` and `KR-ISMS-P` (short forms), while the canonical
registry ids are `pack-kr` and `pack-kr-isms-p`. The normalization gap applies
to all 31 µservices with substantive KR coverage.

---

## §15 Manifest Schema Migration Reference

### §15.1 Old-schema field inventory

The old PR-143 manifest schema used by 39 µservices has these fields. The table
maps each old field to its new-schema equivalent and specifies the migration
action:

| Old field | Type | New-schema equivalent | Migration action |
|---|---|---|---|
| `schema_version` | string | retained as-is | Keep; update value to current schema version |
| `name` | string | retained | Keep |
| `description` | string | retained | Keep |
| `bounded_contexts` | array of objects | no equivalent | Retire; content folded into `substrate_dependencies` description |
| `layers` | array of {layer, crates} | `layer_enum_conformance` | Replace; declare ADR-0105 conformance string instead of per-layer crate list |
| `capabilities` | array of strings | no equivalent | Retire; capabilities listed in PRD §D |
| `contracts` | object | no equivalent | Retire; contract paths documented in ARCHITECTURE.md §transport |
| `slos` | object | no equivalent | Retire; SLOs live in `microservices/<ms>/slos/*.openslo.yaml` per ADR-0131 |
| `ips` | array | no equivalent | Retire; IPs tracked in `ip/` directory |
| `regulatory_packs` | array of strings | `compliance_packs` | Rename; normalize pack-ids to canonical registry format |
| `lts_pins` | object | no equivalent | Retire; dependency pins belong in Cargo.toml / package.json lockfiles |
| `adrs` | array | no equivalent | Retire; ADR citations belong in ARCHITECTURE.md + compliance.md |
| `hyperscaler_inv_coverage` | object | no equivalent | Retire; invariant coverage declared in ARCHITECTURE.md §cell-eligibility |
| `audit_chain` | object | no equivalent | Retire; audit-chain binding declared in compliance.md §detection-substrate-binding |
| `secrets_substrate` | object | no equivalent | Retire; secrets binding declared in ARCHITECTURE.md §credential-isolation |
| `ontology_projections` | array | no equivalent | Retire; ontology read path declared in ARCHITECTURE.md §ontology-read-path |
| `mesh_layering` | object | no equivalent | Retire; mesh config declared in ARCHITECTURE.md §transport |

### §15.2 Required new-schema fields — canonical shape

Per documentation-rigor.md §1 and ADR-0242 through ADR-0255:

```json
{
  "schema_version": "3.0.0",
  "name": "<µservice-slug>",
  "description": "<one-sentence functional description>",
  "tier": "substrate | product",
  "audience_type": "internal | b2b | b2c | all",
  "layer_enum_conformance": "<ADR-0105 layer slug>",
  "cell_eligibility": "Tier 0 | Tier 1 | Tier 2 | Tier 3",
  "substrate_dependencies": ["<µservice-slug>", ...],
  "compliance_packs": ["<canonical-pack-id>", ...],
  "naming_justifications": {
    "<name>": "<one-line BNF v4.1 + 13-layer-enum justification>"
  }
}
```

**Notes on required field semantics:**

- `tier`: binary classification per ADR-0245. `substrate` = internal platform
  service consumed only by other µservices or product surfaces. `product` =
  directly exposed to end users. `"external-facing"` (used by `developer-sdk`
  and `plugin-app-store`) is not a valid value — both should be `product`.

- `cell_eligibility`: a scalar string enum per ADR-0248 §D-1, not an object.
  `api-gateway` uses `["tier-0"]` (array), `feature-flags` and `intelligence`
  use objects. All three must be migrated to the scalar string form.

- `substrate_dependencies`: array of µservice slugs that this µservice depends
  on at runtime. Required for six-hops graph traversal. Must list all direct
  runtime dependencies. Example: `calendar` depends on `identity`, `cell`,
  `tenancy`, `audit-chain`, `observability`, `notification` — its
  `substrate_dependencies` array must list all six.

- `compliance_packs`: array of canonical pack-ids from the ADR-0251 registry.
  Must match the pack-id format: `pack-kr`, `pack-eu`, `pack-pci-dss-l1-v4`,
  `pack-us-healthcare`, `pack-fedramp-high`, `pack-soc2-type-ii`,
  `pack-eu-ai-act`.

### §15.3 analytics manifest — unique pre-PR-143 schema

`analytics/manifest.json` uses a unique schema not found in any other manifest:

```json
{
  "framework_scorecards": { ... },
  "ip_pack": { ... },
  "mesh_layering": { ... }
}
```

This predates the PR-143 manifest schema standardization pass. The fields
`framework_scorecards` and `ip_pack` are not defined in any current standard.
`mesh_layering` was in the PR-143 old schema but removed in the post-keystone
new schema. This manifest requires:
1. Archive current content to `analytics/superseded/manifest-pre-pr143.json`.
2. Author a new `analytics/manifest.json` from the §15.2 canonical shape.
3. Cross-reference the `framework_scorecards` content into `analytics/PRD.md §E`
   (non-functional requirements / six-dimension matrix) where it is more
   appropriately documented.

### §15.4 foundry manifest — largest old-schema manifest

`foundry/manifest.json` at 1148 lines is the largest manifest in the corpus.
Its size reflects deep old-schema elaboration: it has detailed `bounded_contexts`
arrays, per-context `layers` decompositions, extensive `capabilities` lists,
`slos` objects with per-operation targets, and `adrs` arrays. This richness is
the inverse of the PRD problem — extensive manifest content that belongs in
PRD §D (capabilities), PRD §E (SLOs), and ARCHITECTURE.md (bounded contexts,
ADR citations). The migration for `foundry/manifest.json` is the most complex
in the corpus:
1. Extract SLO targets → `foundry/slos/*.openslo.yaml` per ADR-0131.
2. Extract capabilities → `foundry/PRD.md §D` functional requirements.
3. Extract bounded-context decomposition → `foundry/ARCHITECTURE.md §substrate-product-binding`.
4. Extract ADR list → `foundry/ARCHITECTURE.md` + `foundry/compliance.md` front-matter.
5. Replace with the 10-field new-schema manifest.

---

## §16 CI Lane Enforcement Timeline

### §16.1 Current enforcement state

Two CI lanes enforce documentation quality across the corpus:

| Lane | Current state | BLOCKER date | What it checks |
|---|---|---|---|
| `governance-doc-rigor` | ADVISORY (warnings only) | 2026-07-16 | PRD line floor, story count, §A–§J presence, ARCHITECTURE.md anchor count, compliance.md anchor count, manifest field presence |
| `governance-doc-link-resolves` | ADVISORY (warnings only) | 2026-07-16 | placeholder markers occurrences, dead cross-references, missing frontmatter fields |
| `governance-cross-consistency` | ADVISORY (warnings only) | 2026-07-16 | Invariants 1–10 from §3.2.2 |
| `governance-doc-coverage` | ADVISORY (warnings only) | 2026-07-16 | Per-µservice doc set completeness (all 4 file classes present) |

All four lanes transition to BLOCKER on 2026-07-16. From that date, any PR
touching a µservice that fails any of the four checks will be blocked from
merging until the failure is resolved.

### §16.2 Current advisory-warning counts (estimated)

Based on audit findings, the estimated advisory warning count per lane on
the current corpus state:

| Lane | Estimated current warnings | Post-batch-B1-B6 estimate | Post-full-Wave-3-D estimate |
|---|---:|---:|---:|
| `doc-rigor` | ~310 (39 STUB PRDs × ~5 checks + 32 ARCH REVISE × ~3) | ~90 (batch actions clear most manifest + anchor checks) | ~0 |
| `doc-link-resolves` | ~68+ (placeholder markers count) + frontmatter violations | ~10 (after B5 + B6) | ~0 |
| `cross-consistency` | ~220 (invariants 1,5,6,8,9 corpus-wide failures) | ~60 (after B1 manifest migration normalizes fields) | ~0 |
| `doc-coverage` | 0 (all 46 µservices have all 4 file classes present) | 0 | 0 |

The `doc-coverage` lane passes today because every µservice has a file for each
class, even if the files are stubs. The `doc-rigor` lane has the highest warning
count. After the 6 batch actions (§10.1), the estimate drops from ~310 to ~90
because the mechanical fixes (manifest migration, missing anchor sweeps, citation
cleanup, frontmatter sweep) are the most frequent warning triggers.

### §16.3 Minimum viable state for BLOCKER compliance by 2026-07-16

To avoid a full pipeline blockage on 2026-07-16, the corpus needs to reach a
state where no µservice PR triggers a BLOCKER check failure. The minimum viable
path is:

1. **Complete batch actions B1–B6** (§10.1) — mechanical; eliminates the
   majority of `doc-rigor` and `cross-consistency` warnings.
2. **Complete P0 ARCH expansions** (§10.2: cell, tenancy, identity, foundry,
   cloud-secrets, audit-chain, intelligence, observability) — expands 8 ARCH
   files from boilerplate to substantive; clears the ARCH-anchor-count check.
3. **Complete P0 PRD rewrites** for the critical-path substrates — at minimum,
   `intelligence` (38 lines) and `cell` (425 lines, 0 stories) must reach the
   800-line BORDERLINE floor to avoid hard BLOCKER status.
4. **Add `§ml-model-lifecycle` + `§detection-fairness-audit`** to the 34
   compliance.md files missing them (batch B2) — this alone clears 34 of 34
   REVISE verdicts on the compliance.md anchor check.

Items 1 and 4 are the same as batch actions B1 and B2. The critical path to
2026-07-16 BLOCKER compliance is approximately 8 focused PRs.

### §16.4 Per-µservice BLOCKER-risk score

Scoring: 1 point per axis failing the BLOCKER threshold (PRD STUB = 1 point;
ARCH REVISE = 1 point; compliance.md REVISE = 1 point; manifest 0–3 fields = 1
point; 28-row score < 20 = 1 point). Maximum 5 points. Higher score = more
work before 2026-07-16.

| µservice | PRD | ARCH | comp | mf | 28-row | Score/5 | Risk |
|---|---|---|---|---|---|---:|---|
| intelligence | 1 | 0 | 1 | 0 | 0 | 2 | MEDIUM (ARCH is best in class; PRD + compliance gaps) |
| payments | 0 | 0 | 1 | 1 | 0 | 2 | LOW-MEDIUM (closest to full-pass; 2 compliance + 4 manifest fields) |
| ontology | 0 | 0 | 1 | 1 | 0 | 2 | LOW-MEDIUM (ARCH 2 REVISE-PENDING; compliance + manifest) |
| identity | 0 | 1 | 1 | 1 | 1 | 4 | HIGH (PASS PRD but 3 other axes deficient) |
| feature-flags | 1 | 0 | 0 | 0 | 0 | 1 | LOW (PRD only; all other axes good) |
| api-gateway | 1 | 0 | 1 | 0 | 0 | 2 | MEDIUM (ARCH + manifest strong; PRD + compliance) |
| mail | 1 | 0 | 0 | 1 | 0 | 2 | MEDIUM (BORDERLINE PRD; story-formatting fix only) |
| workflow-engine | 0 | 1 | 1 | 1 | 1 | 4 | HIGH (PASS PRD; ARCH all boilerplate; compliance + manifest) |
| notes | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |
| social | 1 | 0 | 0 | 1 | 0 | 2 | MEDIUM |
| connector | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |
| tenancy | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |
| cell | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| foundry | 1 | 1 | 0 | 1 | 0 | 3 | MEDIUM-HIGH |
| cloud-secrets | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| audit-chain | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| observability | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| network | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| cloud-iac | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| cloud-k8s | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| application | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| anonymous | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| consent-graph | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| developer-sdk | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| analytics | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| calendar | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| meet | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| messenger | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| community | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| forms | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| docs | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| drive | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| tasks | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| sheets | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| slides | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| sites | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| recordings | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| shorts | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| translate | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| plugin-app-store | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| workflow-studio | 1 | 1 | 1 | 1 | 1 | 5 | CRITICAL |
| finops-portal | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |
| governance | 1 | 1 | 0 | 1 | 1 | 4 | HIGH |
| compliance | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |
| ops-dashboard-control-center | 1 | 0 | 1 | 1 | 0 | 3 | MEDIUM-HIGH |

**CRITICAL (5/5) count: 24 µservices.** These 24 are failing all five axes
simultaneously and will block any PR that touches them from merging on 2026-07-16
without remediation. The 6 batch actions in §10.1 address 2–3 of the 5 axes
mechanically for most of these; the remaining 2–3 axes require substantive
authoring per µservice.

### §16.5 Risk distribution summary

| Risk tier | Count | µservices |
|---|---:|---|
| CRITICAL (5/5) | 24 | `anonymous`, `application`, `audit-chain`, `calendar`, `cell`, `cloud-iac`, `cloud-k8s`, `cloud-secrets`, `consent-graph`, `developer-sdk`, `docs`, `drive`, `forms`, `meet`, `messenger`, `network`, `observability`, `plugin-app-store`, `sheets`, `sites`, `slides`, `tasks`, `workflow-studio`, `calendar` |
| HIGH (4/5) | 7 | `analytics`, `community`, `governance`, `identity`, `recordings`, `shorts`, `translate`, `workflow-engine` |
| MEDIUM-HIGH (3/5) | 7 | `compliance`, `connector`, `finops-portal`, `foundry`, `notes`, `ops-dashboard-control-center`, `tenancy` |
| MEDIUM (2/5) | 6 | `api-gateway`, `intelligence`, `mail`, `ontology`, `payments`, `social` |
| LOW-MEDIUM (2/5, easy fixes) | 2 | `ontology`, `payments` |
| LOW (1/5) | 1 | `feature-flags` |

The remediation priority order in §10 maps directly to this risk distribution.
Completing the 6 batch actions (§10.1) converts all CRITICAL µservices to
HIGH or lower by addressing the manifest (axis 4) and frequently the 28-row
score (axis 5) mechanically. The residual HIGH and CRITICAL risk after batch
actions is concentrated in PRD rewrite (axis 1) and ARCH expansion (axis 2),
which are the exclusively human-authored axes.

---

*End of microservices corpus line audit. All findings are as-of 2026-05-21
audit date. No µservice files were modified during this audit. Next scheduled
audit: post-Wave-3-D batch-action completion (target: 2026-06-07).*
