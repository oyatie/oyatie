# G030-P singleton registry residual consumer proof — 2026-08-02

State: **PLANNING_ONLY — EIGHT SINGLETON ROWS GRAPH-WIRED; TWO RETAINED; NO REGISTRY/GATE EDIT**  
Authority: `origin/dev` at `b651080374113aeb57500eecbd9d1326f0404e48`.  
Supplements `G030-O-CHECK-EMPIRICAL-EVIDENCE-EXISTENCE-CONTRACT-PROOF-2026-08-02.md`.  
No registry row, gate, policy, PR, GitOps declaration, or cluster state was changed.

## Result

G030-G's residual family "singleton registry families = 10" maps exactly to ten tip paths. Exact Rust reader/writer defaults promote eight. Two remain POLICY_PROTECTED because the only machine citations are catalog/prose or planned checkers that are ABSENT on tip.

| Path | Measured consumer/retention evidence | Disposition |
|---|---|---|
| `registry/adr/inherited-bominal-adrs.yaml` | `adr-citation` defaults `--inheritance-registry` to this path; `read_inherited_adr_ids` parses `- id:` / `id:` ADR rows into the allowed pack set | `GRAPH_WIRED_INPUT — INHERITANCE ALLOWLIST` |
| `registry/ci-fix-loop-retry-budget.json` | `hyperscaler-maturity-claims` defaults and `read_json`s this path; `validate_ci_fix_loop_retry_budget` requires schema fields including shared-across-sources, escalation_action values, and forbids human-escalation | `GRAPH_WIRED_INPUT — RETRY BUDGET SCHEMA` |
| `registry/dependency-blessed-allowlist.json` | `DEFAULT_BLESSED_ALLOWLIST_PATH`; `read_blessed_allowlist` requires non-empty `blessed` object keys; scans workspace member Cargo.toml direct deps | `GRAPH_WIRED_INPUT — BLESSED ALLOWLIST` |
| `registry/graph/architecture-map.json` | architecture-map emit gate default `out_path`; architecture-map-app emits JSON here; freshness kernel fixtures treat the path as snapshot/input | `GRAPH_WIRED_INPUT — EMITTED MAP + FRESHNESS SNAPSHOT` |
| `registry/hyperscaler-scorecards/index.json` | ontology-scorecards-resolver `rollup_path` default; `--emit-rollup` rewrite and `--check` byte-identity compare | `GRAPH_WIRED_INPUT — SCORECARD ROLLUP` |
| `registry/merge-queue-admission-log.json` | pr-review-dispatcher-app `ADMISSION_LOG` const; `append_admission_event` reads/rebuilds schema `oya-merge-queue-admission-log/v1` events array | `GRAPH_WIRED_INPUT — ADMISSION APPEND LOG` |
| `registry/microservices.json` | architecture-map-app exact path load + `microservice_id` array parse into Microservice nodes; freshness kernel changed-path fixtures | `GRAPH_WIRED_INPUT — MICROSERVICE NODE SEED` |
| `registry/mistakes-ledger.json` | loop-recovery-patterns default; `read_mistakes_ledger_ids` requires top-level `entries[]` with `id`; join target for pattern mistake refs (G030-I promoted patterns, not this residual row) | `GRAPH_WIRED_INPUT — MISTAKES LEDGER JOIN SET` |
| `registry/merge-queue-tick-log.json` | only prose/spec citation (`specs/merge-queue-parked-pr.json` convergence_proof) + historical audit text; exact RS consumers 0; retired merge-queue app ABSENT | `POLICY_PROTECTED_MACHINE_ARTIFACT — PROSE-ONLY TICK LOG` |
| `registry/claim-matrix/ops-portal.json` | artifact-capabilities row + claim-matrix profile default only; declared checker/generator/healer planned and ABSENT on tip; exact RS consumers 0 | `POLICY_PROTECTED_MACHINE_ARTIFACT — CATALOG-ONLY CLAIM MATRIX` |

This promotes eight residual rows. Reconciled totals become **152 `MACHINE_SSOT` + 948 `GRAPH_WIRED_INPUT` + 76 `POLICY_PROTECTED_MACHINE_ARTIFACT` = 1,176**. Remaining protected queue: 19 fixture + 57 non-fixture. Delete candidates remain 0.

## Double-count check

Prior G030 slices referenced `registry/mistakes-ledger.json` only as the join set behind G030-I pattern promotion. The ledger path itself was never counted as a GRAPH_WIRED residual row. No other singleton path appears as MACHINE_SSOT/GRAPH_WIRED in G030-E..O durable proofs. Therefore all eight promotions are new.

## Consumer proof detail

### 1. ADR inheritance registry

`marketplace/facade/dev-cli/src/lib.rs`:

1. defaults `inheritance_registry` to `registry/adr/inherited-bominal-adrs.yaml`;
2. `read_inherited_adr_ids` opens the path when present;
3. line-parses `- id:` / `id:` values;
4. validates ADR-id shape;
5. extends the allowed pack ADR set used by `validate_adr_citations`.

Missing file is treated as empty inheritance (soft-open), but the committed tip path is the live default and is the gate's machine input when present. That is graph wiring, not delete authority.

### 2. CI fix-loop retry budget

`marketplace/facade/dev-cli/src/hyperscaler_maturity_claims_gate.rs`:

1. defaults `ci_fix_loop_retry_budget_path` to `registry/ci-fix-loop-retry-budget.json`;
2. `read_json`s it as a required maturity input;
3. `validate_ci_fix_loop_retry_budget` enforces schema fields, including escalation_action tokens and an explicit ban on human-escalation as the destination action.

### 3. Dependency blessed allowlist

`marketplace/facade/dev-cli/src/dependency_blessed_allowlist_gate.rs`:

1. `DEFAULT_BLESSED_ALLOWLIST_PATH = "registry/dependency-blessed-allowlist.json"`;
2. `read_blessed_allowlist` requires a non-empty top-level `blessed` object;
3. keys become the allow set for every workspace member's direct external deps;
4. default severity remains report-only (ADR-0092 D14); enforce is opt-in.

Comment-only Cargo.toml citations are not consumers. The Rust gate is.

### 4. Architecture map

Two executable edges:

- emit path: `architecture_map_emit_gate` defaults `--out` to `registry/graph/architecture-map.json` and writes via architecture-map-app `emit_json`;
- freshness path: `oya-governance-architecture-map-freshness-kernel` fixtures treat the same path as snapshot content and as a changed-path signal together with `registry/microservices.json`.

Emit-output status does not make the committed artifact deletable while freshness and emit both name it.

### 5. Hyperscaler scorecard rollup

`data/facade/ontology-scorecards-resolver/src/main.rs`:

1. `rollup_path(root)` = `root.join("registry/hyperscaler-scorecards/index.json")`;
2. `--emit-rollup` rewrites that path;
3. `--check` compares on-disk bytes to recomputed rollup.

### 6. Merge-queue admission log

`oya/intelligence/crates/oya-intelligence-pr-review-dispatcher-app`:

1. `const ADMISSION_LOG = "registry/merge-queue-admission-log.json"`;
2. after verdict, `append_admission_event` creates parents, reads existing events if present, rebuilds schema `oya-merge-queue-admission-log/v1`, and atomically writes;
3. tests assert append-across-runs and pending flags.

Append-writer is still executable graph wiring.

### 7. Microservices registry

`oya/intelligence/crates/oya-intelligence-architecture-map-app/src/lib.rs`:

1. joins exact `registry/microservices.json`;
2. if present, reads and parses `microservice_id` values into Microservice nodes;
3. freshness kernel fixtures also name the path as a changed input that invalidates architecture-map freshness.

### 8. Mistakes ledger

`marketplace/facade/dev-cli/src/loop_recovery_patterns_gate.rs`:

1. defaults `--mistakes-ledger` to `registry/mistakes-ledger.json`;
2. `read_mistakes_ledger_ids` requires top-level `entries` array and per-entry `id`;
3. pattern rows must reference known mistake IDs.

G030-I already promoted the pattern JSON rows and one empirical scorecard. This residual promotes the ledger residual itself.

## Retained POLICY_PROTECTED rows

### merge-queue-tick-log.json

Exact RS / BUCK consumers at tip: **0**.

Only machine-adjacent citations:

- `specs/merge-queue-parked-pr.json` prose convergence_proof naming the path;
- historical audit markdown describing a retired `oya-vcs-merge-queue-fix-loop-app` whose declared home is ABSENT.

No live appender, reader, or gate defaults to this path. Retain as protected historical/prose companion. Not delete authority without an owner ruling that the parked-PR convergence claim no longer needs the artifact.

### claim-matrix/ops-portal.json

Exact RS consumers at tip: **0**.

Machine catalog citations only:

- `registry/artifact-capabilities-registry.json` row `ops-portal-claim-matrix` with planned checker/generator/healer and `no live path`;
- `specs/artifact-profile-defaults.json` claim-matrix profile example path;
- openapi/evidence contract language for claim-matrix generally, not this residual loader.

Declared shell validator path and planned crates are ABSENT on tip. Catalog presence without an executable reader is POLICY_PROTECTED, not GRAPH_WIRED. Not delete authority while the artifact-capabilities row still names the path.

## Anti-vacuity and semantic boundary

Proven:

- tip singleton residual width = 10 exact paths;
- 8 have exact Rust default path + parse/write edge;
- 2 have zero exact RS consumers;
- none of the 8 were previously counted as GRAPH_WIRED residual rows;
- mistakes-ledger join role in G030-I did not already promote the ledger residual;
- planned claim-matrix checker and retired tick-log writer are ABSENT.

Not proven:

- protected required-context execution of every singleton gate on every `oya-ci-required` run;
- semantic freshness of architecture-map contents vs live tree;
- enforce-mode cleanliness of dependency allowlist;
- that admission-log append is currently consumed by a live Tide/admission controller;
- owner declassification of tick-log or claim-matrix.

These are enforcement/product gaps, not G030 delete authority.

## Verification boundary

Evidence came from immutable tip `ls-tree`, exact path grep across `*.rs`/`BUCK`, and reader/writer body inspection at `b651080374113aeb57500eecbd9d1326f0404e48`. No local CLI execution is used as merge authority.

Independent verifier transport remains fused (`encrypted_content` decrypt 400 / connection closed). This mechanical proof is **not** independent APPROVE.

## Non-actions and non-claims

- No registry singleton edited, emptied, or deleted.
- No gate default changed.
- No claim that emit-only or append-only artifacts are SSOT sources of truth beyond their executable edge.
- No claim that catalog-only claim-matrix or prose-only tick-log may be deleted.
- No move-plan JSON, generated face, or multispectrum evidence surface added.
- No independent APPROVE inferred from transport failure.
- G028 remains local-only unpushed at `051bc7ec6`; no cluster mutation.
