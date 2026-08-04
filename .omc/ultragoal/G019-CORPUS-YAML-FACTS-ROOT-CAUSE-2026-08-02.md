# G019 promoted corpus failure — root-cause class — 2026-08-02

## Exact failing target (promoted tip evidence class)
- Target: `//oya:corpus-yaml-facts`
- Observed on completed promoted runs:
  - run `30743698064` (head `e26f2cc…`, #1522 push)
  - run `30747487757` (head `b6cebda…`, #1525 push)
- Log signature both times:
  - `Action failed: root//oya:corpus-yaml-facts … (genrule)`
  - `Local command returned non-zero exit code <no exit code>`
- Stage A promoted tip `b65108037` (#1527) operator artifact: FULL mode, binding phase `completed-check-exit-code` failure at job `91521498720` (~8.9 min step). Exact log still blocked while `oya-ci-required` aggregator remains queued; same target class is the strongest prior exact match.

## Pre-repair shape on origin/dev
```
genrule(
  name = "corpus-yaml-facts",
  srcs = glob(["**/*.yaml", "**/*.yml"]),
  out = "yaml-facts.json",
  cmd = "$(exe //governance/corpus/extract:yaml-facts) --target root//oya:corpus-yaml-facts --prefix oya --out $OUT $SRCS",
)
```
- Tracked Oya YAML count on origin/dev: **4103** files
- Path-list size alone ~245 KiB of argv text before content; this is the classic oversized same-package extraction face that dies with a blank non-zero and no structured exit code when the runner action spawn/argv path collapses.

## Repair candidate (PR #1526 head `fd2cb9d2f0d47f4bcd84c1c76e1953e7be440ecc`)
- Same-package sharding via `corpus_yaml_facts_shards(srcs=glob(...), shard_size=256)`
- Claims: 16 root faces + 6 nested package faces; configured coverage 4103; semantic output byte-identical
- Exact-head local verification (this session):
  - `buck2 build //oya:corpus-yaml-facts //ci/facade/corpus-index-coverage:` → BUILD SUCCEEDED
  - `buck2 test //ci/facade/corpus-index-coverage:ci-corpus-index-coverage-unittest //ci/facade/corpus-index-coverage:ci-corpus-index-coverage-gate` → PASS
- Candidate CI attempts 1–2: **RUNNER_LOST_COMMUNICATION** during merge-base baseline materialization (step still in_progress, later pending, logs 404), **before** candidate binding execution. Not a content disproof of the shard repair.
- Attempt 3: failed-jobs rerun queued at 2026-08-02T17:13:27Z; affected-set job `91524468086` still unassigned (runner capacity).

## Terminals held
| Object | Terminal |
|---|---|
| Promoted tip Stage A | `PROMOTED_BINDING_FAILURE` (class: corpus-yaml-facts) |
| PR #1526 attempts 1–2 | `RUNNER_LOST_COMMUNICATION` |
| PR #1526 local exact head | `LOCAL_EXACT_HEAD_GREEN_NOT_CANDIDATE_CI_GREEN` |
| PR #1526 attempt 3 | `QUEUED_NO_VERDICT` |
| PR #1524 | quarantined exact head `b1c4664d0570f26fcf492dcd48499a7c21db5470` DO NOT MERGE |
| Stage B | not activated |
| Materializer | `BLOCKED_NO_EXECUTABLE_MOVE_PLAN` |

## Non-claims
- Local green is not candidate protected green.
- Candidate protected green is not promoted green.
- No merge of #1526 until attempt-3 (or later) reaches terminal SUCCESS including oya-ci-required.
