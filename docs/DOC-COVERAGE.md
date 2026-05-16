---
doc_class: DocCoverageMatrix
shape: snapshot
status: Living
date: 2026-05-13
auto_emitted_by: oya-check-documentation-cli (LEAN-A5)
authority_chain: "docs/MASTERPLAN.md \xA713.5 \u2192 ADR-0063 \u2192 this file"
purpose: "This file is the canonical coverage snapshot for the oyatie documentation suite contract. Per ADR-0063, every µservice in `[workspace.metadata.oya.microservices]` must have a complete documentation suite."
doc_status: published
---
# Documentation Suite Coverage Matrix

This file is the canonical coverage snapshot for the oyatie documentation suite contract. Per ADR-0063, every µservice in `[workspace.metadata.oya.microservices]` must have a complete documentation suite. This file is **auto-emitted by `oya-check-documentation-cli`** (LEAN-A5) on every PR; hand-edits get overwritten.

## Status legend

- 🟢 **Full** — PRD + Microservice record + Naming-scope ADR + BC registrations + ≥1 Phase-Spec + ≥1 Impl-Plan, all section-complete per ADR-0063 §4
- 🟡 **Partial** — some artifacts exist, but the suite is incomplete (e.g., PRD exists but no microservice record, or PRD missing required section)
- 🔴 **Stub** — registered in `[workspace.metadata.oya.microservices]` (or planned in MASTERPLAN §2.1) but no canonical docs yet
- ⚪ **Not yet registered** — planned in MASTERPLAN §2.1 but not yet a workspace member; not enforced until first introducing-phase claims it

## Required artifacts (per ADR-0063)

### Canonical suite (per µservice)

1. `docs/microservices/<microservice>.md`
2. `docs/prds/<microservice>.md`
3. `docs/decisions/ADR-NNNN-microservice-<microservice>.md`
4. `docs/bounded-contexts/<microservice>-<bc>.md` (one per BC)
5. ≥1 `.omc/plans/milestones/M*/phases/*/phase-spec.md` referencing the µservice
6. ≥1 `.omc/plans/milestones/M*/phases/*/impl-plan.md` referencing the µservice

### Per-pack overlay (per active pack × µservice in pack scope)

1. `docs/prds/<microservice>-<pack>.md` (when pack adds material scope)
2. `docs/decisions/ADR-NNNN-<pack>-<microservice>-regulatory.md`
3. `docs/localization-packs/<pack>/evidence/<microservice>.md`

---

## Coverage matrix

### Customer-facing — Healthcare cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `medical` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P01 |
| `pharmacy` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P02 |
| `patient` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P04 |
| `emergency` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P05 |
| `clinical` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P01 |
| `healthcare-portal` | 🔴 stub | 🔴 stub | M04 | ⚪ awaits M04-P04 |

### Customer-facing — Workforce cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `hr` | 🟡 PRD only | 🔴 stub | M03 | 🟡 PRD exists; needs microservice record + naming ADR + BC registrations |
| `payroll` | 🟡 PRD only | 🔴 stub | M03 | 🟡 PRD exists; same gap |
| `accounting` | 🟡 PRD only | 🔴 stub | M03 | 🟡 PRD exists; same gap |
| `ats` | 🔴 stub | 🔴 stub | M08 | ⚪ awaits M08-P01 |
| `grc` | 🔴 stub | 🔴 stub | M08 | ⚪ awaits M08-P02 |
| `performance` | 🔴 stub | 🔴 stub | M08 | ⚪ awaits M08-P03 |
| `workforce-analytics` | 🔴 stub | 🔴 stub | M08 | ⚪ awaits M08-P04 |

### Customer-facing — Industrial cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `manufacturing` | 🔴 stub | 🔴 stub | M07 | ⚪ awaits M07-P01 |
| `logistics` | 🔴 stub | 🔴 stub | M07 | ⚪ awaits M07-P02 |
| `facility-ops` | 🔴 stub | 🔴 stub | M07 | ⚪ awaits M07-P03 |
| `procurement` | 🔴 stub | 🔴 stub | M07 | ⚪ awaits M07-P04 |
| `security` | 🔴 stub | 🔴 stub | M07 | ⚪ awaits M07-P05 |

### Customer-facing — FinTech cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `payments` | 🔴 stub | 🔴 stub | M06 | ⚪ awaits M06-P01 |
| `insurance` | 🔴 stub | 🔴 stub | M06 | ⚪ awaits M06-P04 |
| `finance-quant` | 🔴 stub | 🔴 stub | M06 | ⚪ awaits M06-P05 |
| `settlement` | 🔴 stub | 🔴 stub | M06 | ⚪ awaits M06-P03 |

### Customer-facing — Connect & Social cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `connect` | 🟡 PRD only | 🔴 stub | M03 (Pro) / M05 (Personal) | 🟡 PRD exists (covers dual-context); needs microservice record + naming ADR + BC registrations |
| `community` | 🔴 stub | 🔴 stub | M05 | ⚪ awaits M05-P04 |
| `social-graph` | 🔴 stub | 🔴 stub | M05 | ⚪ awaits M05-P04 |
| `profile-personal` | 🔴 stub | 🔴 stub | M05 | ⚪ awaits M05-P04 |

### Customer-facing — Hospitality / Niche cluster

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `hospitality` | 🔴 stub | 🔴 stub | H4 backlog | ⚪ M12+ |
| `dining` | 🔴 stub | 🔴 stub | H4 backlog | ⚪ M12+ |
| `cellar` | 🔴 stub | 🔴 stub | H4 backlog | ⚪ M12+ |

### Adapter plane — Workflow + Ontology

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `workflow` | 🟡 PRD only | N/A (pack-neutral) | M02 / M03 | 🟡 PRD exists (28.5K hero PRD); needs microservice record + naming ADR + BC registrations |
| `ontology` | 🟡 PRD only | N/A (pack-neutral) | M02 | 🟡 PRD exists; same gap |

### Substrate (always-on)

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `tenancy` | 🟡 PRD only | N/A | M02 | 🟡 PRD exists |
| `identity` | 🔴 stub | N/A | M02-P03 | ⚪ awaits M02-P03 |
| `audit-chain` | 🔴 stub | N/A | M02-P04 | ⚪ awaits M02-P04 |
| `eventing` | 🔴 stub | N/A | M02-P05 | ⚪ awaits M02-P05 |
| `secrets` | 🔴 stub | N/A | M02-P06 | ⚪ awaits M02-P06 |
| `observability` | 🔴 stub | N/A | M02-P07 | ⚪ awaits M02-P07 |
| `kms` | 🔴 stub | N/A | M02-P08 | ⚪ awaits M02-P08 |
| `policy` | 🔴 stub | N/A | M02-P14 | ⚪ awaits M02-P14 |
| `search` | 🔴 stub | N/A | M02-P09 | ⚪ awaits M02-P09 |
| `vector` | 🔴 stub | N/A | M02-P10 | ⚪ awaits M02-P10 |
| `data-boundary` | 🔴 stub | N/A | M02-P15 | ⚪ awaits M02-P15 |
| `finance-library` | 🔴 stub | N/A (pack-neutral; KR uses via seam) | M02-P11 | ⚪ awaits M02-P11 |
| `capability-registry` | 🔴 stub | N/A | M02-P17 | ⚪ awaits M02-P17 |
| `records` | 🔴 stub | KR FHIR R5 cross-walk | M02-P16 / M04 | ⚪ awaits M02-P16 + M04-P03 |
| `ads` | 🔴 stub | N/A | M12+ backlog | ⚪ |
| `analytics` | 🔴 stub | N/A | M12+ backlog | ⚪ |

### Cloud substrate

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `cloud-tenancy` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-iam` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-kms` | 🔴 stub | N/A | M02-P08 | ⚪ awaits M02-P08 |
| `cloud-compute` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-storage` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-network` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-billing` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-cell` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-region` | 🔴 stub | N/A | M02-P18 | ⚪ awaits M02-P18 |
| `cloud-observability` | 🔴 stub | N/A | M02-P07 / M02-P18 | ⚪ |

### Foundry + Application

| µservice | Canonical | KR pack | Lead milestone | Status |
|---|---|---|---|---|
| `foundry` | 🔴 stub | N/A (internal-only) | M01-P05 + M02-P01 | 🟡 partial — phase docs exist; PRD pending |
| `application` | 🟡 PRD only | N/A (pack-neutral shell) | M02-P19 | 🟡 PRD exists; needs microservice record + naming ADR + BC registrations |

---

## Pack overlay coverage

### `kr` pack (planned/foundational — pack #1)

Status: `planned/foundational`. Flips to `active` when all promotion blockers in `kr/pack.yaml` are green (corpus.lock signed + ≥1 µservice overlay shipped + ≥1 acceptance evidence signed + 1 paying tenant live).

**Scope source of truth**: `docs/localization-packs/kr/pack.yaml > microservices_in_scope` (27 µservices across Workforce / Healthcare / FinTech / Industrial / Connect / Hospitality clusters). The `material_scope: bool` per-µservice flag in the manifest determines whether an overlay PRD is required (per ADR-0063 §2).

| Pack artifact | Path | Status |
|---|---|---|
| Manifest (authoritative) | `docs/localization-packs/kr/pack.yaml` | 🟢 authored |
| Overview | `docs/localization-packs/kr.md` | 🟢 authored |
| Corpus lock | `docs/localization-packs/kr/corpus.lock` | 🔴 TBD (promotion blocker) |
| Evidence dir | `docs/localization-packs/kr/evidence/` | 🟢 directory exists; 🔴 0 µservice evidence docs yet |
| Per-µservice overlay PRDs (material_scope=true; 18 µservices) | `docs/prds/<microservice>-kr.md` | 🔴 0/18 |
| Per-µservice regulatory ADRs (every µservice in scope) | `docs/decisions/ADR-NNNN-kr-<microservice>-regulatory.md` | 🔴 0/27 |
| Per-µservice acceptance evidence | `docs/localization-packs/kr/evidence/<microservice>.md` | 🔴 0/27 |

### `us` / `eu` / `jp` / `sea-*` / `mena-*` packs

⚪ All deferred to H3 / H4. Catalog entries planned but not yet active.

---

## Milestone-level artifacts

| Milestone | README | Acceptance evidence | Status |
|---|---|---|---|
| M01-foundation | 🔴 TBD | 🟡 partial (P05 hardening committed) | 🟡 |
| M02-substrate | 🔴 TBD | 🔴 TBD | 🔴 |
| M03-first-tenant | 🔴 TBD | 🔴 TBD (P08 evidence bundle planned) | 🔴 |
| M04-healthcare-kr | 🔴 TBD | 🔴 TBD | ⚪ |
| M05-connect-personal | 🔴 TBD | 🔴 TBD | ⚪ |
| M06-fintech-kr | 🔴 TBD | 🔴 TBD | ⚪ |
| M07-industrial-kr | 🔴 TBD | 🔴 TBD | ⚪ |
| M08-enterprise-breadth | 🔴 TBD | 🔴 TBD | ⚪ |
| M09-us-expansion | 🔴 TBD | 🔴 TBD | ⚪ |
| M10-eu-expansion | 🔴 TBD | 🔴 TBD | ⚪ |
| M11-healthcare-intl | 🔴 TBD | 🔴 TBD | ⚪ |
| M12-hyperscaler-maturity | 🔴 TBD | 🔴 TBD | ⚪ |

---

## Section-completeness audit

Required sections per ADR-0063 §4 (every PRD must have these):

- `## Competitive Benchmark`
- `## Performance Targets`
- `## Horizontal Scalability`
- `## Bounded Contexts`

Required sections per Impl-Plan: `## Concrete File Targets`, `## Code Shape`, `## Acceptance Gates`, `## Load test`, `## Grit Claim Symbols`, `## ICM Rows to Emit`.

Required Phase-Spec frontmatter: `acceptance_lanes:`, `depends_on:`, `entry_gate:`, `exit_gate:`.

**Lane will report per-PRD / per-Phase-Spec / per-Impl-Plan section gaps once `oya-check-documentation-cli` is operational (M02-P20).**

---

## Tracking-issue model

Each 🔴 / 🟡 row above translates 1:1 to a doc-authoring task. The dispatch model:

1. Author per-cluster sweeps (one executor per cluster) to fill canonical-suite gaps
2. Per-pack sweep (one executor per pack) to fill pack overlay gaps
3. Each executor: write all artifacts in scope, commit per `grit claim → work → grit done` protocol
4. After commit, this matrix re-emits via `oya-check-documentation-cli` and reflects new green status

---

## References

- [ADR-0063 Documentation suite coverage (CI enforced)](decisions/ADR-0063-documentation-suite-coverage.md)
- [ADR-0064 Canonical base + localization seams/adapters/packs](decisions/ADR-0064-canonical-base-and-localization-packs.md)
- [MASTERPLAN §13.5](MASTERPLAN.md)
- [Localization packs INDEX](localization-packs/INDEX.md)
- [KR pack (#1)](localization-packs/kr.md)
- Lane registry: `registry/quality/lanes.yaml` (lane id: `lean-a5-documentation`)
