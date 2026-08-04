# G019/G028 affected-set eviction root cause — 2026-08-02

State: **CONFIRMED INFRASTRUCTURE BLOCKER — NO ADMISSION, NO LIVE MUTATION**

## Exact candidate and trunk facts

- Current `origin/dev`: `b651080374113aeb57500eecbd9d1326f0404e48`.
- PR #1526 candidate: `fd2cb9d2f0d47f4bcd84c1c76e1953e7be440ecc`.
- PR #1528 candidate: `da46906d02408cef255f3a678ff5e047fe8a3d44`.
- Trunk run `30757272048` failed its binding FULL build at exact target `root//oya:corpus-yaml-facts` after 421 seconds. That is the target PR #1526 shards and repairs.
- The trunk operator artifact records a real `decision.tier=FULL`, `will_run=true`, and `binding-build-test=completed-check-exit-code`; this is a code verdict, not a blank run.

## PR failure signature

Two independent PR jobs on two different owned runners had the same shape:

| PR/job | Runner | Step-7 start | Job end | Step-7 elapsed | Typed terminal shape |
|---|---|---|---|---:|---|
| #1528 / `91523009385` | `qj8fm` | 17:01:20Z | 17:22:16Z | 20.93 min | job `failure`; step 7 still `in_progress`, null conclusion; later steps pending |
| #1526 attempt 3 / `91524468086` | `vm2d4` | 17:23:53Z | 17:45:01Z | 21.13 min | job `failure`; step 7 still `in_progress`, null conclusion; later steps pending |

The workflow gives step 7 a 55-minute timeout and the job a 120-minute timeout. Neither timeout fired. Repository comments already define this typed shape as pod death before GitHub can attach a step verdict.

## Cluster evidence

Read-only Kubernetes events confirm ephemeral-storage eviction:

- `qj8fm`: evicted 2026-08-02T17:12:31Z; runner using `32,419,048 Ki`; request `20Gi`; node below ephemeral-storage threshold.
- `vm2d4`: evicted 2026-08-02T17:34:58Z; runner using `30,922,628 Ki`; request `20Gi`; node below ephemeral-storage threshold.
- `wsll2`: same class at `32,428,476 Ki`.
- Worker-2 transitioned to `NodeHasDiskPressure`, then recovered after eviction.

The active runner declaration requests `20Gi` ephemeral storage and limits at `60Gi`. Worker-2 allocatable ephemeral storage is `45,909,593,217` bytes (42.756 GiB binary). The reported runner usages convert to 29.490–30.926 GiB binary (`Ki × 1024 ÷ 1024³`). Therefore Kubernetes can schedule two 20Gi-request runners on one worker even though two measured peaks cannot fit. This is a scheduling-accounting defect, not evidence that a 60Gi-per-pod hard limit is too small.

## Trusted-baseline causal loop

The PR fast path can reuse exact push-to-dev build/test baseline artifacts. Current trunk’s binding FULL run fails on `//oya:corpus-yaml-facts`, so its baseline upload steps are skipped. The PR therefore correctly fails closed to a cold merge-base rebuild. The cold rebuild creates a second worktree and whole-graph Buck output, reaches ~31Gi, and is evicted when co-located.

PR #1526 is thus blocked by a circular admission condition unless runner scheduling is fixed:

1. trunk cannot publish a trusted baseline because the corpus target fails;
2. #1526 exists to repair that target;
3. #1526 must cold-rebuild the merge base because no trusted baseline exists;
4. the cold fallback is evicted because runner disk requests understate measured peak demand.

## Smallest safe repair

Change the owned ARC runner’s ephemeral-storage **request** to the smallest integer Gi value for which two requests exceed every current worker’s ~42.75Gi allocatable capacity. That is `22Gi` (`2 × 22Gi = 44Gi > 42.75Gi`), though operational headroom and byte/Gi arithmetic must be verified in the implementation lane before choosing the committed value. Keep the `60Gi` limit, `maxRunners: 3`, required workflow semantics, and gate fail-closed behavior unchanged.

This schedules at most one runner per current worker and lets the existing three-runner fleet use the two worker nodes serially where necessary. It does not make a failed target green and does not weaken the merge-base ratchet.

## Review status

The independent tracer terminated on transport/decryption failure before a verdict. No approval is inferred. A separate writer lane is preparing the declaration-only change and must self-verify, then receive independent review before push/admission.

## Implementation status (2026-08-02 later)

- Local declaration-only commit on branch `g028-runner-disk-request-20260802`: `051bc7ec603d49b838a400471a01778b966b2b8c`.
- Change: `requests.ephemeral-storage` `20Gi` → `22Gi`; limit `60Gi` and `maxRunners: 3` unchanged.
- Self-verified binary arithmetic and YAML parse invariants.
- Independent review pending/transport-failed; **not pushed**, **not applied** to the live fleet.
- Measured usage comment corrected to binary Gi `29.49–30.93` (Ki×1024/1024³), not the raw Ki×1e-6 misread.
