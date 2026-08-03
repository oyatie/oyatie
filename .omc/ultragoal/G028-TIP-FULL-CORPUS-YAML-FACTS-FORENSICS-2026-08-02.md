# G028 tip FULL `//oya:corpus-yaml-facts` forensics — 2026-08-02

State: **READ-ONLY CLASSIFICATION — NO RERUN / NO CLUSTER MUTATION / NO APPROVE**  
Authority tip: `origin/dev` `0c1014b87f0d881a821faa6a872b309deba0cfbf`  
Run: `30767156146` · Job: `91547799524` · Runner: `oya-arm64-lh4ch-runner-9d5zl`  
URL: https://github.com/jason931225/oyatie/actions/runs/30767156146/job/91547799524

## Proven facts

| Fact | Evidence |
|---|---|
| event | push to `dev`; before `b651080…`, after `0c1014b87…` (#1529) |
| run window | started `2026-08-02T21:06:57Z`, updated `21:19:26Z`, attempt 1 |
| job window | started `21:09:17Z`, completed `21:18:20Z`, conclusion failure |
| failed step | #8 Binding affected-set build + test (`21:11:05Z`→`21:18:16Z`) |
| tier | FULL; mode full; baseline_source cold-rebuild; binding phase `completed-check-exit-code` |
| failed action | `root//oya:corpus-yaml-facts` genrule at `21:16:53.954Z` |
| signature | `Local command returned non-zero exit code <no exit code>` |
| concurrent work at failure | ~90 other local actions still waiting (e.g. `aws-lc-sys` buildscript) |
| post-failure runner health | step 11 Upload affected-set operator artifacts **success** at `21:18:16Z` (847 B); Complete job success |
| annotation | only `Process completed with exit code 1` on `.github` |
| live ARC | ARS + ERS still request `20Gi` on `admin@oya-talos` |
| related open PR | #1526 corpus YAML shard faces; still blocked for cold FULL rerun |

Operator artifact `affected-set-binding-decision.json` (artifact id `8839438030`):

```json
"mode": "full",
"tier": "FULL",
"baseline_source": "cold-rebuild",
"binding-build-test": "completed-check-exit-code"
```

## What is **not** proven

- No log line for OOM, DiskPressure, eviction, ENOSPC, SIGKILL, or runner lost-communication on this job.
- No Kubernetes pod terminal-state / cgroup / node-condition retention for the runner window in this packet (not collected as durable evidence here).
- No proof that 22Gi would have made this FULL green (necessary ≠ sufficient).
- No proof this is a pure corpus logic/input defect: genrule emitted no stderr/exit code under keep-going concurrent FULL compile.
- Workflow `wf_9c2680fb-269` collectors all FAILED_TRANSPORT; its empty-evidence synthesis is **discarded** and is not authority.

## Narrow classification

```text
FULL_COLD_CORPUS_YAML_FACTS_CHILD_ACTION_NO_EXIT_CODE
runner_survived=true
pod_eviction_or_OOM=NOT_PROVEN
live_22Gi=false
rerun_authorized=false
```

Discriminating observation: the **child genrule** died without an exit code while the **job runner continued** ~83s later to upload artifacts and complete cleanly. That refutes whole-pod immediate death as the sole explanation, but does not identify the child’s kill mechanism.

## Codepath at tip (immutable)

`root//oya:corpus-yaml-facts` is a single package genrule:

```text
oya/BUCK
  genrule name=corpus-yaml-facts
  srcs = glob(["**/*.yaml", "**/*.yml"])   # tip count = 4103 paths under oya/
  cmd = $(exe //governance/corpus/extract:yaml-facts)
        --target root//oya:corpus-yaml-facts --prefix oya --out $OUT $SRCS
```

Extractor (`governance/corpus/extract/src/yaml_facts_main.rs`):

- reads every declared input fully into a `String`;
- runs pure `corpus_yaml_kernel::extract` per file (saphyr YAML walk → nodes/edges/opaque);
- accumulates **all** nodes/edges/opaque for the shard in memory;
- serializes one canonical JSON face and writes `$OUT`;
- on normal errors prints `corpus-yaml-facts: …` to stderr and returns `ExitCode::FAILURE`.

Measured tip input mass (blob sizes via `git ls-tree -l`, not runtime RSS):

| Metric | Value |
|---|---|
| YAML path count under `oya/` | 4103 |
| sum of YAML blob sizes | 6.48 MiB |
| estimated abs `$SRCS` argv payload | ~0.36 MiB |
| ARG_MAX-class failure | **refuted** at this scale |

So a clean logic failure should have left stderr + a normal nonzero exit. The observed `<no exit code>` under concurrent FULL `buck2 build //... --keep-going` is therefore still consistent with **abnormal child termination** (signal/cgroup/runner action kill) or a buck2 local-execute reporting gap — not with ARG_MAX, and not proven as OOM/DiskPressure without pod/cgroup evidence.

## Related open repair (#1526) — not authorized to rerun

Head `fd2cb9d2f0d47f4bcd84c1c76e1953e7be440ecc` replaces the single genrule with:

```text
corpus_yaml_facts_shards(srcs=glob(...), shard_size=256)
→ 17 faces (corpus-yaml-facts + corpus-yaml-facts-shard-0001… )
```

That bounds configured action inputs/working set per face. It is a plausible mitigator for single-action pressure, **not** proof of root cause, and **not** admission while live ARC remains 20Gi and independent review/transport cannot clear the PR train.

## Unblock criteria (unchanged train)

1. Prior founder ruling selects GitOps class B / KEEP_CURRENT_LAB=true; real independent design APPROVE lands.
2. Admitted reconciler path makes live ARS/ERS request `22Gi`.
3. Only then authorize #1526 cold FULL rerun with retained job+pod+node telemetry.
4. #1523 restack push only after #1526 promoted green path is healthy.
5. Do not weaken request below 22Gi; do not helm-upgrade ARC from scratchpad.

## Non-actions

- No CI rerun from this note.
- No cluster mutation.
- No #1523 push / #1524 touch / canonical dirty-checkout mutation.
- No transport failure treated as APPROVE.
- No DiskPressure/OOM claim without stronger discriminating evidence.
