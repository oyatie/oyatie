# FRIC-017 disk-preflight fix — READY TO LAND (between strangler moves)

Productizes task #50 = FRIC-017 recurrence (friction-ledger.jsonl line 17). buck2/affected-set lanes
intermittently false-RED with `No space left on device` at the buck-out RESTORE step (before any build runs).
Re-running is the work-around; THIS is the recurrence-preventing pipeline change (founder doctrine: fix the class).
Hit on PR #741 run 27658159382 job 81796720889 (and originally PR #638). FRIC-017's prescribed
`enforcement_fix` ("disk-space preflight gate at job start") was queued (`rerun-triggered-runner-hygiene-queued-G11`) but never shipped.

## Root cause (diagnosed, GitHub-hosted confirmed)
- Runners are GitHub-hosted `ubuntu-latest` (API: total self-hosted = 0). ~14 GiB free on `/`.
- buck-out cache blob = 5.78 GiB compressed → ~12–15 GiB decompressed, restored on top of a `fetch-depth:0`
  monorepo checkout (>10 GiB tree) → crosses the ~14 GiB cliff → ENOSPC kills even the runner's own `_diag` writer.
- NO free-disk/cleanup step exists anywhere in the workflow. `_diag` accrual is intra-run (not cross-run; not self-hosted).

## Fix (single file, two lanes)
File: `.github/workflows/oya-ci-required.yml` on **dev HEAD** (author against the post-cell-merge tip, NOT the stale local d705932).
Insert this step in BOTH heavy lanes — the `buck2` lane (~dev line 344) AND the `gate-affected-set` lane (~dev line 500) —
immediately AFTER "Install buck2" and BEFORE "Cache pinned Rust toolchain" (i.e. before the toolchain + buck-out restores):

```yaml
      # Reclaim preinstalled ubuntu-latest bloat (.NET/Android/GHC/CodeQL/preloaded Docker images:
      # ~25-30 GiB) BEFORE the toolchain + buck-out restores. This lane warms a multi-GB full-graph
      # buck-out (5.78 GiB compressed -> ~12-15 GiB on disk) on top of a fetch-depth:0 monorepo checkout,
      # exhausting the ~14 GiB free on /; FRIC-017 recurred on PR #741 run 27658159382 (No space left on
      # device at the buck-out restore, before any build ran). Hermetic: removes only vendor preinstall dirs
      # no oya/buck2 action consumes; touches NO repo content and NO cache, so the cold/warm integrity canary
      # (cold==warm) is unaffected. df is emitted so a real disk-NEED growth surfaces instead of being masked.
      - name: Reclaim runner disk before warm restore (FRIC-017 preflight)
        run: |
          set -euo pipefail
          echo "before:"; df -h /
          sudo rm -rf /usr/share/dotnet /usr/local/lib/android /opt/ghc \
                      /usr/local/.ghcup /opt/hostedtoolcache/CodeQL || true
          sudo docker image prune --all --force || true
          echo "after:"; df -h /
```

## Why hermetic + canonical (no false-green risk)
- Removes only FIXED vendor preinstall; emits `df -h /` before/after so a genuine disk-need-growth surfaces as a true RED, not masked.
- Touches ZERO repo content and ZERO buck2 cache state (buck-out / ~/.rustup / the restored blob untouched) → cold==warm bit-identity canary unaffected (ADR-0556 D1/D2).
- Single canonical workflow (ADR-0515), no new file, no generator, no third-party pinned action. Inline CI shell extension (founder permits minimal extension of existing CI shell; this is glue, not a new CLI surface).

## Rejected alternatives
(b) cap/prune buck-out cache — doesn't help restore-time peak (blob needed whole; already bounded, dev-push sole writer).
(c) larger mount — ubuntu-latest has no 2nd large writable mount. (d) ephemeral runners — already ephemeral. (a self-hosted reconciler) — N/A.

## Landing protocol
- Its own SMALL PR, BETWEEN moves (modifies shared CI used by all PRs; must not ride inside a reorg move).
- For `pull_request` same-repo branch, GitHub runs the HEAD's workflow YAML (merge commit) → the fix protects its OWN buck2 lane.
- Base on the current dev tip (content-assert: base == dev tip, no rebase). Independent review (small) → green → squash-merge → signature check → ledger row → post-merge push-tier verify.
- Follow-up (ledger hygiene, deferrable): flip FRIC-017 status to shipped; optionally add infra-red-vs-code-red label on the required context so a future disk-RED self-triages without an agent dispatch.

## Sequencing decision
Land AFTER cell #741 merges, BEFORE gateway move-7 → every subsequent move is disk-protected.
If #741's own rerun keeps hitting ENOSPC (can't get a genuine green), FLIP: land this fix first, then rebase+re-verify #741 onto it.
