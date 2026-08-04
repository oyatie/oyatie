---
purpose: Oyatie — Mistakes-and-Fixes Ledger
doc_status: published
---

# Oyatie — Mistakes-and-Fixes Ledger

## Doctrinal authority — [decision-principles.json](../specs/decision-principles.json) + [forbidden-operations.json](../specs/forbidden-operations.json)


> **Owner:** `council-architecture` (curator). Per-team contributors.
> **Cadence:** per-incident + per-audit + per-quarter.
> **Companion:** [`standards/prevention-doctrine.md`](standards/prevention-doctrine.md), [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md), [RISK-REGISTER.md](RISK-REGISTER.md).

---

## 1. Why this ledger exists

Mistakes are unavoidable. Repeating the same mistake is preventable. This ledger captures every prevention-class learning so:

- Future engineers see the failure mode + prevention + when shipped
- Foundry capabilities can replay against past traces to verify continued prevention
- Auditors see chain of remediation
- Council can spot patterns across mistakes (e.g. recurring failure-mode class)

## 2. Entry format

```
| ID | Date | Mistake (1 line) | System gap (1 line) | Mechanical prevention | Shipped on | Link |
```

Each entry:
- **ID**: `MFL-NNNN` sequential
- **Date**: when the mistake surfaced (not when prevention shipped)
- **Mistake**: short description, no PII
- **System gap**: what system / process / contract was missing
- **Mechanical prevention**: the CI lane / hook / validator / fitness function name
- **Shipped on**: date prevention landed
- **Link**: PR / ADR / runbook / incident postmortem

## 3. Active ledger

| ID | Date | Mistake | System gap | Mechanical prevention | Shipped | Link |
|---|---|---|---|---|---|---|
| MFL-0001 | 2026-05-09 | Legacy ADRs cited in active consolidated docs after pack consolidation | No CI gate enforcing only-new-pack-citations | `oya-governance-adr-citation` lane | shipped 2026-05-10 (active gate) | per [ADR-CONSOLIDATION-PLAN.md](ADR-CONSOLIDATION-PLAN.md) |
| MFL-0002 | 2026-05-09 | Deprecated brand aliases or tautological brand-transition text persisted after standardization on Oyatie | No CI gate enforcing brand-residue check | `oya-governance-brand-residue` lane | shipped 2026-05-10 (active tautology gate) | per [ADR-0017 brand-naming](decisions/ADR-0017-brand-naming-and-repo-layout.md) |
| MFL-0003 | 2026-05-09 | M3/MVP vocabulary leaking from legacy docs into new docs after retirement | No CI gate for retired-vocab | `oya-governance-glossary` (extended for retired terms) | (target with W-Foundation gate) | per [GLOSSARY.md §11](GLOSSARY.md) |
| MFL-0004 | 2026-05-09 | CUG (Closed-User-Group) terminology persisted after Team rename | Same as MFL-0003 | Same lane | (target with W-Foundation gate) | per [GLOSSARY.md §11](GLOSSARY.md) |
| MFL-0005 | (future-prevention) | Cross-axis contract change without consumer-axis review | Cross-axis review label not auto-emitted | `oya-governance-blast-radius` per [DESIGN §3.0.5.3](DESIGN.md) blast-radius classifier | (target with W-Foundation gate) | per [ADR-0011 cross-axis-contract-registry](decisions/ADR-0011-cross-axis-contract-registry.md) |
| MFL-0006 | (future-prevention) | New external dep adopted without ledger entry | No CI gate on dep additions without ledger | `oya-governance-build-vs-buy` per [ADR-0014 build-vs-buy-policy](decisions/) | (target with W-Foundation gate) | per [VENDOR-PARTNER-LEDGER.md](VENDOR-PARTNER-LEDGER.md) |
| MFL-0007 | (future-prevention) | AGPL/GPL dep introduced into product code | License-class CI gate gap | `oya-governance-license` per [ADR-0013 product-license-policy](decisions/ADR-0013-product-license-policy.md) | (target with W-Foundation gate) | per License Policy ADR |
| MFL-0008 | (future-prevention) | Data class annotation missing on new struct field in kernel crate | Pre-commit data-class hook gap | `pre-commit-data-class.sh` + `oya-governance-data-class` per [ADR-0008 data-use-boundary](decisions/ADR-0008-data-use-boundary.md) | (target with W-Foundation gate) | per Data Use Boundary ADR |
| MFL-0009 | 2026-05-09 | Legacy mobile-clients ADR cluster silently dropped during pack consolidation; no consolidated successor authored | Regression-mapping had only enumerative coverage, no per-cluster successor-authoring discipline | `oya-governance-adr-citation` lane + per-PARTIAL-row authoring obligation in [`ADR-LEGACY-REGRESSION-MAPPING.md`](ADR-LEGACY-REGRESSION-MAPPING.md) | shipped 2026-05-09 (ADR-0051 mobile-and-native-client-strategy) | per [ADR-0051](decisions/ADR-0051-mobile-and-native-client-strategy.md) + Codex Round 2 verdict |
| MFL-0010 | 2026-05-09 | RUNBOOKS-INDEX referenced 49 P0 runbook files that did not exist on disk after the cleanup | No CI gate verifying RUNBOOKS-INDEX entries resolve to real files | `oya-governance-runbook-index-resolves` lane (planned) + per-runbook stub authoring at index-update time | shipped 2026-05-09 (49 P0 stubs under `runbooks/`) | per Codex Round 2 verdict + RUNBOOKS-INDEX §1 |
| MFL-0011 | 2026-05-09 | Brand-rebrand sed introduced false equality statements where both sides of a brand transition matched | Sed had no semantic awareness of historical-mention vs current-brand context | `oya-governance-brand-residue` lane catches tautological transition pairs | shipped 2026-05-10 (active gate) | per Codex Round 2 verdict |
| MFL-0012 | 2026-05-11 | Legacy `modules/`, `services/`, or `platform/` tree could be reintroduced after flat-crates migration work | Flat-crates lane checked role edges but not legacy top-level implementation directories or exact workspace package paths | `oya-governance-flat-crates` self-test + architecture boundary check | shipped 2026-05-11 (active gate) | per [ADR-0015](decisions/ADR-0015-architectural-flattening-target.md) + [flat-crates move runbook](runbooks/flat-crates-move-pr.md) |
| MFL-0013 | 2026-05-11 | OpenAPI 3.2 `query` or `additionalOperations` operations could be skipped by contract validation and runtime parity | OpenAPI validator hard-coded the pre-3.2 fixed-method set and did not traverse nested `additionalOperations` | `oya doc openapi` + `oya-intelligence-openapi-kernel` source/runtime parity tests for `query`, custom additional operations, fixed-method collisions, and response-schema parity | shipped 2026-05-11 (active gate) | per [API design standard](standards/api-design.md) + [SPEC.md](SPEC.md) |
| MFL-0014 | 2026-05-15 | GitHub Actions step pinned to a broken / unresolved `<action>@<sha>` SHA reached `main` and broke the PR-tests workflow before any other check could run | No preflight probe verifying every `uses: <action>@<sha>` resolves via `gh api /repos/<action>/commits/<sha>`; no fitness lane indexing the (`gha`, `broken-action-sha`) repeat-class | `oya-governance-mistakes-ledger-kernel` lane + `docs/runbooks/sanctioned-primitives/preflight.md` row #8 | shipped 2026-05-15 (M01-P17-IP-003) | per [pipeline-maturity audit Stage 6 + Stage 10](audits/../../evidence/audits/pipeline-maturity-audit-2026-05-15.md) |
| MFL-0015 | 2026-05-15 | `cargo nextest run --profile ci` failed because `[profile.ci]` was missing from `.config/nextest.toml`; one issue surfaced per CI cycle, not all at once | No preflight probe asserting `[profile.ci]` exists before invoking nextest; no fitness lane catching the (`nextest`, `missing-profile-ci`) repeat-class | `oya-governance-mistakes-ledger-kernel` lane + `docs/runbooks/sanctioned-primitives/preflight.md` row #9 | shipped 2026-05-15 (M01-P17-IP-003) | per [pipeline-maturity audit Stage 10 + Amendment §A](audits/../../evidence/audits/pipeline-maturity-audit-2026-05-15.md) |
| MFL-0016 | 2026-05-15 | Shell scripts under `scripts/` shipped without a `#!` shebang line; CI invoked them via `sh -c` which masked the regression for one full cycle | No preflight probe asserting every `*.sh` carries a shebang; no fitness lane catching the (`bash`, `missing-shebang`) repeat-class | `oya-governance-mistakes-ledger-kernel` lane + `docs/runbooks/sanctioned-primitives/preflight.md` row #10 | shipped 2026-05-15 (M01-P17-IP-003) | per [pipeline-maturity audit Stage 10](audits/../../evidence/audits/pipeline-maturity-audit-2026-05-15.md) |
| MFL-0017 | 2026-07-13 | A stable product graph could mask nondeterministic or stale direct masterplan projection output | The regenerate-twice canary compared the downstream graph but not the independently materialized masterplan bytes | `ci-generated-artifact-freshness` direct masterplan-and-graph determinism checks plus the stable-graph/masterplan-drift regression | (target with protected `oya-ci-required` admission) | per [ADR-0613](decisions/ADR-0613-de-commit-remaining-controller-materialized-projection-faces.md) and [pre-planning closure evidence](../evidence/consolidation/preplanning-authority-closure-20260713.json) |
| MFL-0018 | 2026-07-13 | Recorded sequencing ratification and an open planning hold could remain green with missing/stale digest proof or binding approval enabled | The cross-artifact gate checked ratification metadata and dispatch state without content-address or binding-approval parity | `ci-cross-artifact-agreement` three-way digest equality plus exact open-hold approval/dispatch regressions | (target with protected `oya-ci-required` admission) | per [sequencing ratification evidence](../evidence/goals/masterplan-v2-sequencing-founder-ratification-20260702.json) and [pre-planning closure evidence](../evidence/consolidation/preplanning-authority-closure-20260713.json) |
| MFL-0019 | 2026-08-03 | Registry-drift and crate-registration callers selected the first sorted committed reorg plan and passed it explicitly, bypassing the codemod's landed-plan filtering | Active-plan selection was duplicated in callers instead of remaining authoritative in the reorg codemod | Delete caller-side plan discovery; argument regressions prohibit `--plan`, while codemod tests preserve PARKED exclusion, landed-versus-active selection, multi-active fail-closed behavior, and zero-active canonical-empty output | (target with protected `oya-ci-required` admission) | per [`oya-reorg-codemod-app` selector](../tools/oya-reorg-codemod-app/src/manifest.rs) and its crate-registration / registry-drift callers |
| MFL-0020 | 2026-08-03 | Graph v2 dropped executable three-node/buried-cycle and policy-boundary regressions while a v1 projection falsely implied current full-topology parity | The replacement test corpus did not preserve every prior RED class, and no explicit claim ceiling distinguished the bounded 19-unit slice from the 24-capability target | `ci-dependency-graph-acyclicity-gate` named live-document mutations + explicit `current_parity_claim: false` + mandatory no-new-baseline `W0-C-TOPOLOGY-COVERAGE` | candidate; protected `oya-ci-required` admission pending | per [ADR-0635](decisions/ADR-0635-face-aware-substrate-dependency-graph-v2.md) and [ADR-0280 §D-13.F–H](decisions/ADR-0280-substrate-of-substrate-dependency-doctrine.md) |
| MFL-0021 | 2026-08-03 | Generated Talos machine configurations entered a private recovery branch during pre-wipe preservation | Root generated-output ignores did not name Talos bootstrap artifacts, and admission recognized unknown root files by path but not credential-bearing Talos structure after rename/relocation | exact root-anchored Talos output ignores + value-redacting `ci-repo-root-hygiene` structural regression over the tracked YAML corpus | candidate; protected `oya-ci-required` admission and independent security/operations review pending | [issue #1541](https://github.com/jason931225/oyatie/issues/1541) |

> **More entries** populate as mistakes surface. The ledger is the institutional memory of the project.

## 4. Pattern detection (review per quarter)

Per [`standards/prevention-doctrine.md §6`](standards/prevention-doctrine.md):
- Council reviews ledger quarterly
- Patterns (recurring failure-mode class) trigger meta-prevention
- Top-10-by-recurrence reported to Founder
- Foundry capability `oya.mistakes.detect-pattern` proposes meta-prevention

## 5. Per-prevention verification

Per `standards/prevention-doctrine.md §5 step 6`:
- Each shipped prevention tested against the original failure mode (replay or fuzz)
- Replay-as-eval per [ADR-0024 foundry-eval-harness-and-replay](decisions/ADR-0024-intelligence-eval-harness-and-replay.md)
- Per-quarter `oya.prevention.verify-coverage` capability run

## 6. Sources

- [`standards/prevention-doctrine.md`](standards/prevention-doctrine.md)
- [INCIDENT-MANAGEMENT.md](INCIDENT-MANAGEMENT.md)
- ADR-0003 (audit chain) for `EVT-PREVENTION-SHIPPED`
- All per-incident postmortems
- All per-audit findings
- CLAUDE.md prevention-doctrine pointer


---

> **§Note (2026-05-21 transition):** References to `oya-governance-*` in this historical document are intentional — they describe past state. New work uses `oya-governance-*` per the 2026-05-21 transition directive.
