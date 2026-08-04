# G037-B hatch owner disposition packet — 2026-08-02

State: **PLANNING_ONLY — OWNER DISPOSITION REQUIRED; NO REGISTRY/HATCH/POLICY EDIT**  
Authority: `origin/dev` at `0c1014b87f0d881a821faa6a872b309deba0cfbf` (#1529 merged; ARC request declared `22Gi`, live request still `20Gi`).  
Supplements `G037-QUALITY-LANE-SOURCE-CENSUS-2026-08-02.md`. No lane bind/retire/activation.

## Why this packet exists

Half-(c) in `ci/facade/baseline-ratchet/tests/gate_registration.rs` already keeps unknown-unresolvable active lanes red and shrinks only when a hatch target starts resolving or the lane leaves `active` for a governance reason. It does **not** decide BIND vs RETIRE. That is an owner act.

G037-B therefore freezes the exact five hatched active rows, the two hatch keys, the tip existence proofs, and the minimum owner decision table. No mutation is authorized from this note.

## Exact hatch surface (tip)

```text
KNOWN_UNRESOLVABLE_LANE_TARGETS: 2 keys covering 5 active lanes
```

| Hatch key | Tip existence | Covered live lane id(s) |
|---|---|---|
| `cargo-package:oya-vcs-merge-queue-fix-loop-app` | **ABSENT** (`libs/`, `crates/`, package tree) | `oya-governance-merge-queue-ref-hygiene` |
| `repo-file:tools/governance/adr-0221-governance-gates.sh` | **ABSENT** | `oya-governance-vacuous-green`, `oya-governance-adr-orphan-citation`, `oya-governance-version-pin-source-citation`, `oya-governance-buildability-line-count` |

## Stale prose correction (not a third hatch)

Half-(c) comment text names lane `oya-governance-merge-queue-staging-ref-gc`. That id is **MISSING** from live `registry/quality/lanes.yaml` at tip.

Live row that still points at the dead cargo package is:

```text
id: oya-governance-merge-queue-ref-hygiene
status: active
stage: nightly
owner_team: axis-foundry
source: ADR-0111
check_command: cargo run -p oya-vcs-merge-queue-fix-loop-app -- --gc-staging-refs --max-age-seconds 3600
```

Disposition must address the live id. Comment-only rename inside half-(c) is optional hygiene and is **not** a disposition of the obligation.

## Exact five active hatched rows

### 1. merge-queue staging-ref hygiene

| Field | Tip value |
|---|---|
| id | `oya-governance-merge-queue-ref-hygiene` |
| status/stage | `active` / `nightly` |
| owner_team | `axis-foundry` |
| source | `ADR-0111` |
| check_command | `cargo run -p oya-vcs-merge-queue-fix-loop-app -- --gc-staging-refs --max-age-seconds 3600` |
| hatch | `cargo-package:oya-vcs-merge-queue-fix-loop-app` |
| measured defect | package removed with ADR-0363 VCS ratchet retirement; command is dark |

Owner choices (exactly one):

1. **RETIRE** — if ADR-0363 + ADR-0515 fully supersede the staging-ref GC obligation; cite those ADRs and leave `active` only after a real retirement governance edit (not a silent planned flip solely to clear half-(c)).
2. **BIND** — if the GC obligation still stands under plain-git/Tide admission, author an owned-Rust/Buck2 target that is fan-in reachable from `oya-ci-required` (or an existing required constituent), with RED/GREEN proof; then drop the hatch key only after the package/target resolves.

### 2–5. ADR-0221 shell-hook efficacy quartet

All four share hatch `repo-file:tools/governance/adr-0221-governance-gates.sh` and owner `council-architecture`.

| id | source | check_command suffix |
|---|---|---|
| `oya-governance-vacuous-green` | ADR-0221 M-06 | `vacuous-green` |
| `oya-governance-adr-orphan-citation` | ADR-0221 M-13 | `orphan-citation` |
| `oya-governance-version-pin-source-citation` | ADR-0221 M-01 | `version-pin` |
| `oya-governance-buildability-line-count` | ADR-0221 M-10 | `buildability-line-count` |

Tip facts:

- shell harness path is absent;
- half-(c) states none of the four names is a `gate validate` dispatch arm either;
- ADR-0523 zero-shell posture deleted the harness; no Rust replacement is claimed on tip.

Owner choices for the quartet (may be split per lane, but default is one class decision):

1. **RETIRE** each obligation that ADR-0221 no longer requires under current admission (plain git + `oya-ci-required` + Rust gate packets), with explicit source citation.
2. **BIND** each retained obligation to an owned-Rust/Buck2 check with fixture RED/GREEN and protected fan-in; only then remove the shared hatch key.

Do **not** flip `active → planned` solely to green half-(c). The registration test forbids that class of green-by-relabel.

## Planned five (not hatch; owner still required before activation)

These are not in the hatch set. They remain `planned` with empty `check_command` and must stay non-active until a resolvable target exists.

| id | owner_team | source |
|---|---|---|
| `lean-a10-regression` | `council-architecture` | ADR-0067 §5.5 |
| `quality-statelessness` | `council-architecture` | ADR-0062 |
| `quality-shardability` | `council-architecture` | ADR-0062 |
| `quality-perf-budget` | `council-architecture` | ADR-0062 |
| `quality-benchmark` | `council-architecture` | ADR-0062 |

Note: historical task docs still describe wiring four quality lanes into a retired aggregator path under old crate names. That is bridge/provenance only under CLI-retirement doctrine and is **not** protected-context admission. G037-D remains: leave planned until an owner writes a real protected target.

## Cardinality lock (unchanged)

```text
96 unique ids = 91 active + 5 planned
hatched active = 5
unknown-unresolvable active outside hatch = 0
```

## Accountable owners

| Owner | Why |
|---|---|
| `axis-foundry` | sole owner of the dead merge-queue GC lane |
| `council-architecture` | owner of the ADR-0221 quartet and all five planned rows |
| founder / platform-governance | only if owner teams deadlock on RETIRE vs BIND under ADR-0363/0515/0523 conflict |

No coordinator self-disposition is valid.

## Minimum safe next artifact after owner answers

One owner-signed table:

```text
lane_id | decision(BIND|RETIRE) | authority_citation | successor_target_or_none | hatch_key_action(drop|keep)
```

Only after that table and independent review may a mutation PR touch `registry/quality/lanes.yaml` and/or half-(c) hatch constants. G036 multi-root reachability remains a separate admission problem for the 36 shared-core bridge-only rows and must not be smuggled into hatch retirement.

## Dependency order

1. G028 #1529 MERGED as `0c1014b87` (declared request 22Gi). Live GitOps reconciler is **INERT** on `admin@oya-talos` (no Argo/CAPI; ARS/ERS still 20Gi). Owner must choose repair class A/B/C per `G028-GITOPS-BOOTSTRAP-GAP-2026-08-02.md` before any bootstrap mutation.
2. After live request 22Gi observed on the FULL-running cell: #1526 cold FULL green on corpus repair.
3. #1523 restack admit.
4. Then hatch disposition PR is eligible; not before the train can carry protected evidence.

## Non-actions

- No `lanes.yaml` edit.
- No half-(c) hatch shrink/expand.
- No `active → planned` relabel.
- No second quality registry.
- No baseline of darkness.
- No live G028 apply outside GitOps; historical transport failures are not APPROVE.
- No cluster mutation; no canonical dirty checkout mutation.
- No hand-edit of `*.generated.json`; no new multispectrum evidence files.
