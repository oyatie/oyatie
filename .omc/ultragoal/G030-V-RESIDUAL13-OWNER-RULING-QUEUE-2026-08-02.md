# G030-V residual 13 owner-ruling queue — 2026-08-02

State: **PLANNING_ONLY — OWNER RULINGS REQUIRED; ZERO DELETE CANDIDATES**  
Authority: `origin/dev` at `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; ARC request declared `22Gi`, live request still `20Gi`).  
Supplements G030-T/U. No repository artifact, policy, fixture, or cluster state changed.

## Closed partition carried forward

```text
152 MACHINE_SSOT + 992 GRAPH_WIRED_INPUT + 32 POLICY_PROTECTED = 1176
32 residual = 13 non-fixture + 19 fixture
delete candidates = 0
```

This note turns the exact 13 non-fixture residual paths into an owner-ruling queue. It does not reclassify or delete them.

## Exact 13 tip paths

All 13 exist by `git ls-tree 0c1014b87... -- <path>` (re-probed post-#1529; set unchanged).

| # | Path | Tip evidence | Accountable owner / ruling |
|---:|---|---|---|
| 1 | `registry/accounts/schema.json` | Draft schema owned by `axis-foundry`; `registry/accounts/*.example.toml` exist; supervisor app reads directory, but no runtime/gate reader of schema bytes was proven; parser_ref points to absent historical path | `axis-foundry`: wire schema validation to live account loader under Rust/Buck2, update authority path, or retire schema claim |
| 2 | `registry/accounts/README.md` | Companion doc cites same absent historical parser path | `axis-foundry`: update to live `oya/intelligence/...FileAccountSnapshotProvider` contract or retire companion; no deletion without schema/loader ruling |
| 3 | `registry/vcs/concurrent-safe-paths.yaml` | Loader comment points to removed `oya-vcs-merge-queue-fix-loop-app`; empty strict-default map; ADR-0111 provenance | VCS/merge-queue owner: RETIRE/freeze under ADR-0363/0515 or bind to a live owned SCM/Tide policy reader |
| 4 | `registry/vcs/event-router.yaml` | Self-declared FROZEN historical evidence; rows target retired agents | `council-foundry-vcs` / platform-governance: preserve frozen or archive/delete via explicit ADR-0363 disposition; never reactivate silently |
| 5 | `registry/vcs/webhook-delivery-log.json` | Empty delivery ledger; no live Rust/Buck2 reader found | VCS/merge-queue owner: preserve historical, archive, or retire with retention/audit ruling |
| 6 | `registry/vcs/README.md` | Self-declared FROZEN; says not active/not deleted/not edited; still contains stale Jenkins-era prose | `council-foundry-vcs` / platform-governance: frozen companion ruling; G030 cannot contradict its own tombstone contract |
| 7 | `registry/foundation-bypasses/README.md` | Directory meaning is live: dev-cli foundation audit defaults ledger dir to `registry/foundation-bypasses`; README bytes/extensions are not selected by proven protected consumer | foundation-gate owner: keep companion and either document Rust/Buck2 enforcement destination or explicitly declass prose; not deletable while empty-directory semantics rely on it |
| 8 | `registry/capabilities/foundry-supervisor.toml` | Three driver rows; no exact runtime/Buck reader found; only unrelated historical/evidence name mentions | `axis-foundry` / cloud-intelligence owner: wire live driver catalog or retire/migrate the duplicate capability artifact |
| 9 | `registry/release/supply-chain/README.md` | Companion to release image/evidence contract; says pre-release empty is intentional | release/supply-chain owner: keep companion or make directory lifecycle machine-derived; no delete before release evidence producer/consumer contract proves independence |
| 10 | `registry/merge-queue-tick-log.json` | Scaffolded empty log owned `council-architecture`; purpose names removed merge-queue app; `specs/merge-queue-parked-pr.json` still claims every tick appends here | `council-architecture`: retire both stale producer claim and log, or bind successor writer/checker; dangling claim blocks deletion |
| 11 | `registry/claim-matrix/ops-portal.json` | Draft plan-stage matrix; registered in artifact-capabilities registry/defaults, but no semantic Rust/Buck reader proven | `council-foundry`: keep planning authority, bind claim validator, or retire with ops-plan authority; registration alone is not semantic consumption |
| 12 | `specs/policy/cedar-scope-schema.md` | Active prose specification; cited by `specs/cloud-enforceability-facets.json`; no byte-reading protected consumer proven | policy/PBAC owner: keep prose contract pending machine successor or promote an executable schema/check; citation does not authorize deletion |
| 13 | `specs/products/RETIREMENT.md` | Tombstone says directory retired and “preserved as a tombstone only” | platform-governance: keep until a machine-readable retirement pointer supersedes the human tombstone; no new files under directory |

## Important distinctions

### Accounts runtime directory is not schema wiring

`oya/intelligence/crates/oya-intelligence-supervisor-app/src/main.rs` constructs `FileAccountSnapshotProvider::new("registry/accounts")`, so the account **directory/data rows** participate in runtime. This does not prove `schema.json` bytes are read or validated. The schema and README remain policy-protected until the loader validates them or the owner retires them.

### Foundation empty-directory semantics are not README byte wiring

The dev-cli foundation audit reads the ledger directory and the README intentionally keeps an otherwise-empty control surface present. That supports retention, not promotion to GRAPH_WIRED_INPUT under the byte/semantic-consumer rule.

### Registry registration is not semantic execution

`ops-portal.json` has an artifact-capabilities registry row. Registration/lifecycle metadata does not prove its claims are evaluated by a Rust/Buck2 gate. It therefore remains protected, not graph-wired.

### Frozen/tombstone is a valid protected disposition, not delete authority

The VCS files and `specs/products/RETIREMENT.md` deliberately preserve historical/retirement context. They may be removed only when their accountable authority provides a successor pointer plus retention ruling. G030 census cannot infer DELETE from runtime inactivity.

## Required owner response format

One row per path:

```text
path | owner | decision(WIRE|KEEP_PROTECTED|RETIRE) | authority | successor_or_consumer | acceptance_check
```

Rules:

- `WIRE`: name exact owned-Rust/Buck2 target and protected fan-in; test must fail on an invalid/absent artifact.
- `KEEP_PROTECTED`: cite the live contract that requires retention and its review cadence.
- `RETIRE`: cite authority, retention/privacy consequences, successor pointer if any, and atomic deletion/migration scope.
- No coordinator inference substitutes for owner signature.

## Dependency and lane separation

- G028 #1529 MERGED as `0c1014b87` (declared 22Gi). Live observe still 20Gi — reconciler inert; prior founder ruling fixes class B / KEEP_CURRENT_LAB=true and independent design APPROVE remains pending (`OWNER-DECISION-G028-ABC-LIVE-22GI-2026-08-02.md`). No live apply authorized from this packet.
- #1526 cold FULL then #1523 restack remain blocked until live 22Gi on the FULL-running cell.
- G030-D nested `.omc/state` ruling is separate from this 13-path queue.
- The 19 fixture residual remains separately owned by calendar/CRATEADR/DR lanes in G030-U.
- G036/G037 activation must not be combined with residual cleanup.

## Non-actions

- No delete/move/edit of the 13 paths.
- No fixture mutation.
- No generated JSON edit.
- No new multispectrum evidence.
- No independent APPROVE claimed; review transport remains failed.
- No cluster or canonical dirty-checkout mutation.
