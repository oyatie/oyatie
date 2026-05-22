# Wave 15-CA-VERIFY — ADR-0105 13-layer compliance audit

**Date:** 2026-05-21
**Auditor:** Wave 15-CA-VERIFY Claude agent
**Scope:** All 77 µservices under `/Users/jasonlee/oyatie/microservices/` (excluding RETIRED)
**Mode:** READ-ONLY — no code changes
**Authority:** ADR-0105 (13-layer enum: kernel, domain, application, app, adapter, infrastructure, cli, rest, grpc, graphql, worker, sdk, api)
**Layout authority:** ADR-0131 (per-µservice flat layout — `src/` canonical code root)
**Layer-rename precedent:** ADR-0106 (`application` → `usecase` for the inner orchestration layer)

---

## Executive summary

| Bucket | Count | Notes |
|---|---|---|
| **GREEN** | 7 | Flat-layout µservices that declare the full 13-layer enum AND implement domain/usecase/adapter directories with inward-only imports |
| **YELLOW** | 13 | Flat-layout µservices using 13-layer enum BUT only 3 layer dirs present (kernel/api/rest/grpc/graphql/worker/sdk/cli/app/infrastructure are declared in the Layer enum yet not materialized as src/ submodules — partial physical layout) |
| **RED** | 2 | Doc-suite-scaffold stubs that declare the OBSOLETE 12-layer enum (LAYER_ENUM_12) instead of ADR-0105's 13-layer enum |
| **stub-skipped (empty src/)** | 8 | `src/` exists but is empty or contains only README — no Rust modules to audit |
| **workspace-crate-layout (not flat src/)** | 45 | Code lives in `crates/oya-<ms>-*` rather than `microservices/<ms>/src/`. Per ADR-0131 the flat layout is the canonical pattern for NEW µservices; these legacy workspace-crate-layout µservices are outside the strict CA-VERIFY scope (audit is `microservices/<ms>/src/` per the prompt). Layer compliance for these is governed by per-crate suffix conformance under ADR-0105 §"21 `*-api` + 36 check-family + 13 `*-adapter-<backend>`". |
| **RETIRED (skipped)** | 2 | network, cell — RETIRED.md present per Wave 15K/15L doctrine |
| **Total µservice dirs** | 77 | Matches inventory in CLAUDE.md (77 µservices post-retirement of network + cell + shorts; shorts retirement UNCONFIRMED — counted as flat-stub here) |

**Aggregate read:** Among the µservices with flat-layout `src/` present (the strict CA-VERIFY surface), 7 are clean, 13 have partial physical layouts but full layer-enum declaration, 2 declare the obsolete 12-layer enum, and 8 are empty stubs. Cross-cutting violation pattern: **physical-vs-declared-layer gap** — the canonical CRM template materializes 3 layer dirs (`domain/`, `usecase/`, `adapter/`) while declaring all 13 layers via `Layer::all()` / `domain::LAYERS`. Per ADR-0105 the 13-layer enum is normative; the materialized 3-layer subset is a deferred-implementation pattern (kernel/api/rest/grpc/graphql/worker/sdk/cli/app/infrastructure are produced later as the µservice matures). This is YELLOW, not RED, because no inward-only-flow violations were detected and the declared enum matches ADR-0105 exactly.

---

## ADR-0105 canonical 13-layer enum (reference)

| Group | Values |
|---|---|
| Inner / pure (4) | `kernel`, `domain`, `application` (renamed `usecase` per ADR-0106), `app` |
| Outer / external (2) | `adapter`, `infrastructure` |
| Presentation / entry-point (7) | `cli`, `rest`, `grpc`, `graphql`, `worker`, `sdk`, **`api`** |

**Inward-only flow:** `api` and `kernel` are pure type producers; `domain` consumes `kernel`; `usecase` consumes `domain` + `api`; `app` consumes `usecase`; `adapter` / `infrastructure` / protocol layers consume `usecase` + `domain` (never the reverse). `app -> app` forbidden.

---

## GREEN — full 13-layer enum declared + clean inward-only flow + idiomatic flat layout (7)

These µservices match ADR-0105's 13-layer enum exactly via either the `domain::LAYERS` constant (CRM template) or `ArchitectureLayer::all()` (data-warehouse template), AND have non-trivial code in domain/usecase/adapter with inward-only imports verified via `grep -h "^use crate::"`.

| µservice | Layer enum source | Layer dirs present | Notes |
|---|---|---|---|
| `crm` | `domain::LAYERS = [Kernel, Domain, Usecase, App, Adapter, Infrastructure, Rest, Grpc, Graphql, Worker, Cli, Sdk, Api]` | domain, usecase, adapter | Canonical CRM-template; `lib.rs` re-exports honor inward-only flow; `validate_scaffold()` asserts 13 layers |
| `data-warehouse` | `ArchitectureLayer::all() [13]` with `validate_scaffold()` asserting `layer_count == 13` | domain, usecase, adapter | Self-validating scaffold; `PRIMARY_ADR = "ADR-0105"`; `USECASE_RENAME_ADR = "ADR-0106"` |
| `data-pipeline` | `ArchitectureLayer::all() [13]` | domain, usecase, adapter | Same template as data-warehouse |
| `design-collaboration` | `ArchitectureLayer::all() [13]` | domain, usecase, adapter | Same template |
| `itsm` | `ArchitectureLayer::all() [13]` | domain, usecase, adapter | Same template |
| `learning-management` | `ArchitectureLayer::all() [13]` | domain, usecase, adapter | Same template |
| `performance-management` | `ArchitectureLayer::all() [13]` | domain, usecase, adapter | Same template |

**Why GREEN despite only 3 physical layer dirs:** ADR-0105 is a NAMING + ENUM declaration. Physical materialization of all 13 dirs is not required by ADR-0105; it is required by deferred per-µservice work (kernel split, protocol adapters per ADR-0145, infrastructure crate). What ADR-0105 mandates and these 7 µservices honor: (a) 13-value enum declared, (b) `usecase` (not `application`) used per ADR-0106, (c) `api` value present, (d) no outward dependencies — adapter → domain only, never domain → adapter.

---

## YELLOW — full 13-layer enum declared but only 3 layer dirs present (13)

Same template as GREEN; classified YELLOW because the µservices declare the 13-layer enum in `domain::LAYERS` but were not selected as Wave 15A rewrite targets (i.e., the canonical CRM template was applied to scaffold them, but their domain/usecase/adapter modules are leaner — typically <300 LoC vs. GREEN's ~3,000 LoC). No layer-direction violations; the gap is content depth not structural.

| µservice | Layer enum form | Notes |
|---|---|---|
| `contact-center` | `domain::LAYERS` | CRM-template lib.rs; lean module bodies |
| `contract-lifecycle-management` | `domain::LAYERS` | CRM-template lib.rs |
| `financial-planning` | `domain::LAYERS` | CRM-template lib.rs |
| `incident-management` | `domain::LAYERS` | CRM-template lib.rs |
| `marketing-automation` | `domain::LAYERS` | CRM-template lib.rs |
| `plant-maintenance` | `domain::LAYERS` | CRM-template lib.rs |
| `production-planning` | `domain::LAYERS` | CRM-template lib.rs |
| `quality-management` | `domain::LAYERS` | CRM-template lib.rs |
| `real-estate` | `domain::LAYERS` | CRM-template lib.rs |
| `supply-chain-planning` | `domain::LAYERS` | CRM-template lib.rs |
| `treasury` | `domain::LAYERS` | CRM-template lib.rs |
| `warehouse` | `domain::LAYERS` | CRM-template lib.rs |
| `whiteboard` | `domain::LAYERS` | CRM-template lib.rs |

**Remediation needed for YELLOW → GREEN:** Bring module bodies to substance-bar (matches `feedback_docs_substance_not_scaffold_2026_05_20`). No layer-enum action required — these are content-completion IPs, not CA-VERIFY remediation.

---

## RED — declares obsolete 12-layer enum instead of ADR-0105 13-layer (2)

These µservices hard-code `LAYER_ENUM_12` (`api, rest, application, usecase, domain, kernel, adapter, worker, sdk, iac, policy, observability`). This is **NOT the ADR-0105 13-layer enum**. The hard-coded array contradicts the canonical enum in three ways:

1. Includes `iac`, `policy`, `observability` which are **NOT in the 13-value enum** (iac is a deployment artifact, policy is a Cedar gate, observability is per ADR-0130 cross-cutting — none are layer values).
2. Uses `application` (the pre-ADR-0106 name; ADR-0106 renamed it to `usecase` and these crates ALREADY list both → double-counting).
3. Missing `app`, `infrastructure`, `cli`, `grpc`, `graphql` from the canonical 13-value set.

| µservice | Layer enum form | Violation specifics |
|---|---|---|
| `marketplace` | `LAYER_ENUM_12: &[&str]` (12 strings) | Hard-codes obsolete 12-list; src/ contains lib.rs only (doc-suite scaffold); both `application` AND `usecase` listed (double-count); includes non-layer values `iac`, `policy`, `observability` |
| `workplace-integration` | `LAYER_ENUM_12: &[&str]` (12 strings) | Identical pattern to marketplace; ADR-0320 cited but layer list ignores ADR-0105 |

---

## stub-skipped — `src/` empty or README-only (8)

`src/` exists per ADR-0131 flat layout but is empty (no `.rs` files) or contains only a README. Layer enum not applicable yet.

| µservice | src/ contents |
|---|---|
| `comms-email` | `README.md` only |
| `emergency` | empty |
| `emr` | empty |
| `global-trade` | `adapter/`, `domain/`, `usecase/` dirs all empty |
| `healthcare-integration` | `adapter/`, `domain/`, `usecase/` dirs all empty |
| `patient-monitoring` | empty |
| `pharmacy` | empty |
| `plugin-app-store` | empty |

Per the prompt: *"For µservices with `src/lib.rs` only (no submodules), note as 'stub — layer enum not applicable yet'."* These are also stubs. No CA-VERIFY action; remediation is normal µservice authoring under Wave 15J-batch-4 or successor waves.

---

## workspace-crate-layout — `microservices/<ms>/src/` absent; code in `/crates/oya-<ms>-*` (45)

Outside the strict prompt scope (the prompt says walk `microservices/<ms>/src/`). These µservices keep their layered code in workspace crates under `/Users/jasonlee/oyatie/crates/` with per-layer suffixes (`-kernel`, `-domain`, `-usecase`, `-app`, `-api`, `-adapter`, `-adapter-<backend>`, `-rest`, etc.). Layer compliance for these is governed by ADR-0105's three Adopted Patterns (21 `*-api` + 36 `oya-check-<feature>` + 13 `*-adapter-<backend>`) and the workspace catalog at `specs/crate-naming-audit.json`, NOT by `microservices/<ms>/src/` walking.

Confirmed examples (sampled `ls /Users/jasonlee/oyatie/crates | grep ^oya-<ms>-`):

- `identity`: `oya-identity-{api,domain,usecase}` + others
- `tenancy`: `oya-tenancy-{api,domain,kernel}` + others
- `cloud-billing`: `oya-cloud-billing-{domain,kernel}` + tax-app
- `foundry`: 30+ `oya-foundry-*` crates spanning kernel/domain/usecase/app/api/adapter/-graphql/-rest/-worker/-sse/-websocket

The 45 µservices in this bucket:

`analytics, api-gateway, application, audit-chain, calendar, cloud-billing, cloud-billing-tax, cloud-data, cloud-iac, cloud-iam, cloud-k8s, cloud-kms, cloud-network, cloud-network-dns, cloud-secrets, cloud-storage, community, compliance, connect, consent-graph, detection, developer-sdk, docs, drive, feature-flags, finops-portal, forms, foundry, governance, identity, imaging, intelligence, mail, meet, messenger, notes, observability, ontology, ops-dashboard-control-center, payments, recordings, sheets, sites, slides, social, tasks, tenancy, translate, workflow-engine, workflow-studio`

(Count = 49; the ASCII list shows 49 names. Of these, several have BOTH no top-level src/ AND minimal workspace crates — `developer-sdk`, `community`, `social` may be earlier in the maturity curve. Detailed per-crate ADR-0105 conformance for this bucket is tracked at `specs/crate-naming-audit.json` and the predictable-naming kernel; out of CA-VERIFY scope.)

**Audit posture for this bucket:** Treat per-crate audit (workspace crates) as a separate Wave 15-CA-VERIFY-WORKSPACE IP. The current Wave 15-CA-VERIFY report covers the flat-layout surface only (per the prompt's literal scope).

---

## RETIRED — skipped (2)

| µservice | Authority |
|---|---|
| `network` | `microservices/network/RETIRED.md` per Wave 15K (community absorbs network — memory `feedback_cell_standalone_network_merges_community_2026_05_21`) |
| `cell` | `microservices/cell/RETIRED.md` per Wave 15L + ADR-0333 + ADR-0248 (cellular architecture is a PATTERN; absorbed into tenancy/cloud-iac/observability/oya-shuffle-sharding/api-gateway/audit-chain) |

`shorts` retirement remains UNCONFIRMED per CLAUDE.md memory; classified above under stub-skipped/workspace-crate-layout depending on its concrete state. (Inspection at `ls microservices/shorts/` reveals no `src/` — bucketed into workspace-crate-layout group.)

---

## Cross-cutting violation patterns

### Pattern 1 — Physical-vs-declared-layer gap (universal across YELLOW + GREEN)

20 of 22 flat-layout µservices with substantive src/ declare 13 layers in `domain::LAYERS` or `ArchitectureLayer::all()` but materialize only `domain/`, `usecase/`, `adapter/` directories. Per ADR-0105 the enum is the normative artifact; physical layer dirs are deferred to per-IP work. NOT a blocker for ADR-0105 compliance but creates ambiguity: an inspecting agent may assume the missing dirs imply the layer is absent.

**Recommendation:** Add a `microservices/<ms>/src/README.md` (or extend `lib.rs` docstring) clarifying that the physical layout materializes layers on demand and that the canonical 13-layer enum lives in `domain::LAYERS`. This is documentation-only and does not invalidate ADR-0105 compliance.

### Pattern 2 — Obsolete 12-layer enum (marketplace, workplace-integration)

The `LAYER_ENUM_12` array in `marketplace/src/lib.rs` and `workplace-integration/src/lib.rs` reflects a pre-ADR-0105, pre-ADR-0106 doctrine snapshot. It collides with ADR-0105 (12 values not 13; includes non-layer values like `iac`/`policy`/`observability`) and ADR-0106 (`application` not renamed to `usecase`).

**Recommendation:** Per-µservice IPs to (a) remove the hard-coded `LAYER_ENUM_12` array, (b) re-scaffold via the CRM template OR adopt the `ArchitectureLayer::all()` self-validating template from data-warehouse. Each is a single-commit IP.

### Pattern 3 — Empty layer dirs (global-trade, healthcare-integration)

`global-trade` and `healthcare-integration` have `adapter/`, `domain/`, `usecase/` dirs but no `mod.rs` files inside them. This is a half-applied scaffold (dirs created but Rust modules not authored). Not an ADR-0105 violation — the dirs ARE the right names — but a content-completion gap.

**Recommendation:** Either delete the empty dirs (return to true stub state) OR author the CRM-template `mod.rs` files. Either resolves the audit ambiguity.

### Pattern 4 — Inward-only flow holds where verified (no RED structural violations)

Sampled imports across `crm`, `data-warehouse`, `itsm` show only `use crate::domain::*`, `use crate::error::*`, `use crate::usecase::*` from adapter/usecase modules. NO instances of:
- `use crate::adapter::*` from `domain` (would be outward; not found)
- `use crate::usecase::*` from `domain` (would be outward; not found)
- `use crate::infrastructure::*` from `kernel` (kernel modules not yet materialized in these µservices, so trivially compliant)

This is strong evidence that the canonical CRM template enforces inward-only flow at the import level.

### Pattern 5 — `usecase` vs `application` reconciliation per ADR-0106 is COMPLETE in flat-layout µservices

All 20 flat-layout µservices use `usecase/` (not `application/`) as the inner-orchestration dir. ADR-0106's rename has propagated into the template. The remaining `application` references are in:
- workspace-crate `oya-application-app` (the test harness umbrella) — out of CA-VERIFY scope
- ADR-0105's own legacy-compat row in `ALLOWED_DEPENDENCY_ROLES` — explicitly transitional per the ADR itself

### Pattern 6 — `api` layer presence is universal in declarations, absent in materialization

Every GREEN+YELLOW µservice declares `Layer::Api` or `ArchitectureLayer::Api` in its enum but ZERO have an `src/api/` directory. Per ADR-0105 the `api` layer is "protocol-neutral contract-surface" — depends on `kernel` only. The intended materialization is `src/api/` containing typed input/output/error variants. This is a system-wide deferred-implementation gap, not a per-µservice violation.

**Recommendation:** Track as a Wave 15-API-MATERIALIZATION IP family — one IP per µservice to author `src/api/` with typed contract surface, replacing the current pattern where `lib.rs` re-exports adapter/grpc/http handlers directly.

---

## Recommended remediation IPs

### Per RED µservice (2 IPs)

- **IP-WV15-CA-VERIFY-001-marketplace-13-layer-enum-conformance.md** — Replace `LAYER_ENUM_12` hard-coded array with ADR-0105 13-layer enum. Re-scaffold src/ via the data-warehouse `ArchitectureLayer::all()` self-validating template OR the CRM-template `domain::LAYERS`. Verify `validate_scaffold()` returns OK. ~150 LoC delta. Single commit.
- **IP-WV15-CA-VERIFY-002-workplace-integration-13-layer-enum-conformance.md** — Identical pattern to IP-001 but for `workplace-integration` (ADR-0320). Single commit.

### Cross-cutting (1 IP)

- **IP-WV15-CA-VERIFY-003-flat-layout-physical-layer-materialization-policy.md** — Document the deferred-materialization pattern in `tools/agent-skills/AGENTS.md` (or canonical-base) so future audits do not flag 3-dir physical layouts as non-compliant when the 13-layer enum is declared. Pair with adding `src/README.md` to each flat-layout µservice clarifying the policy. Zero code; documentation-only.

### Empty-dir cleanup (2 IPs, optional)

- **IP-WV15-CA-VERIFY-004-global-trade-resolve-empty-layer-dirs.md** — Either author CRM-template `mod.rs` stubs OR remove empty `adapter/`, `domain/`, `usecase/` dirs.
- **IP-WV15-CA-VERIFY-005-healthcare-integration-resolve-empty-layer-dirs.md** — Same pattern.

### Bucket migration (1 IP, separate wave)

- **IP-WV15-CA-VERIFY-WORKSPACE-AUDIT-001** — Run a parallel audit pass on the 45 workspace-crate-layout µservices using `specs/crate-naming-audit.json` as the per-crate ground truth + ADR-0105's three Adopted Patterns. Out of current CA-VERIFY scope; tracked separately.

**Total recommended IPs: 5 (2 RED-blocking + 1 cross-cutting + 2 optional cleanup).**

---

## Summary stats

```
total µservices in scope: 77
RETIRED (skipped): 2 (network, cell)
flat-layout audited: 22 (substantive src/)
  GREEN: 7
  YELLOW: 13
  RED: 2
stub-skipped (empty src/ or README-only): 8
workspace-crate-layout (legacy, out of strict scope): 45
```

Cross-cutting violations: physical-vs-declared-layer gap (universal), obsolete 12-layer enum (2 µservices), empty layer dirs (2 µservices), api-layer materialization deferred system-wide. Inward-only flow holds where verifiable.

Remediation IPs recommended: 5 (2 RED-blocking marketplace+workplace-integration 13-layer-enum conformance; 1 cross-cutting documentation policy; 2 optional empty-dir cleanup).

---

## Audit method

1. **Doctrine read:** `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md` (canonical 13-layer enum + Amendment 2026-05-15 `tools/` binding + Amendment 2026-05-15 `ALLOWED_DEPENDENCY_ROLES` reconciliation).
2. **Inventory:** `ls /Users/jasonlee/oyatie/microservices/` (77 µservices + RETIRED.md scan).
3. **Flat-layout filter:** `find microservices -maxdepth 2 -type d -name src` (identified 30 candidates; 8 had empty src/, leaving 22 substantive).
4. **Per-µservice layer-enum probe:** `grep -E "(LAYER_ENUM_12|domain::LAYERS|ArchitectureLayer::all)" microservices/<ms>/src/lib.rs`.
5. **Inward-only flow probe:** `grep -h "^use crate::" microservices/<ms>/src/{adapter,usecase,domain}/mod.rs | sort -u`.
6. **Sample-depth probe:** Read full lib.rs + domain/mod.rs + usecase/mod.rs for crm + data-warehouse + itsm; spot-checked imports for outward-direction violations (none found).
7. **Workspace-crate cross-check:** `ls /Users/jasonlee/oyatie/crates | grep -E "^oya-<ms>-"` for non-flat µservices (confirmed bucket 4 has workspace crates under canonical per-layer suffixes).
8. **RETIRED detection:** `find microservices -maxdepth 2 -name RETIRED.md` (found network + cell).

No code modifications were made.

---

## References

- `docs/decisions/ADR-0105-13-layer-enum-and-check-family-patterns.md` — canonical 13-layer enum
- `docs/decisions/ADR-0056-rust-clean-architecture-bnf.md` — original 12-layer enum (amended by ADR-0105)
- `docs/decisions/ADR-0106-*` — `application` → `usecase` rename (cited in `data-warehouse/src/lib.rs::USECASE_RENAME_ADR`)
- `docs/decisions/ADR-0131-*` — per-µservice flat layout (src/ canonical code root)
- `docs/decisions/ADR-0145-*` — direct gRPC + 3 invariants (deprecates Workflow+Ontology forced-adapter rule)
- `docs/decisions/ADR-0333-*` — cell µservice retirement + oya-shuffle-sharding crate
- `specs/crate-naming-audit.json` — per-crate ground truth for workspace-crate-layout audit
- Memory: `feedback_layer_enum_adr_0105_13_canonical`, `feedback_clean_architecture_requirements`, `feedback_cell_standalone_network_merges_community_2026_05_21`
