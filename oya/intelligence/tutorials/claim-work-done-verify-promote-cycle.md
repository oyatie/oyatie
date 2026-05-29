# Tutorial — Walk the full claim → work → done → verify → promote cycle

Goal: take a small but real change through the Foundry pipeline end-to-end. You'll edit two files in a doc lane, run the protocol
verbs, watch the admission gate execute, and observe the merge.

Pre-reqs:
- A loopback foundry cell: `make dev-cell.up CELL=foundry-loopback-1 PROFILE=foundry-dev`
- An isolated worktree: `./bin/oya git worktree-add --base dev --branch tutorial/$USER-foundry .worktrees/$USER-foundry`
- `cd .worktrees/$USER-foundry`

## Step 1 — claim

```bash
./bin/oya vcs claim \
  --agent tutorial-$USER \
  --intent foundry-tutorial-2026-05-20 \
  docs/architecture/notes microservices/intelligence/scratch
```

Expected:
```
oya vcs claim accepted: action=claim-lock agent=tutorial-$USER scopes=2 evidence=0
```

The claim is now active. Other agents cannot touch `docs/architecture/notes/**` or `microservices/intelligence/scratch/**` while you hold it.

## Step 2 — work (edit files)

```bash
mkdir -p docs/architecture/notes microservices/intelligence/scratch
cat > docs/architecture/notes/$USER-tutorial.md <<EOF
# $USER foundry-tutorial 2026-05-20

Walks the claim/work/done/verify/promote cycle end-to-end. No content beyond demonstrating the pipeline.
EOF

cat > microservices/intelligence/scratch/$USER-tutorial.json <<EOF
{ "actor": "$USER",
  "scenario": "foundry-tutorial-2026-05-20",
  "completed_steps": ["claim"] }
EOF
```

Inspect:
```bash
./bin/oya vcs status --agent tutorial-$USER
```
Expected: 2 modified files staged for the claim.

## Step 3 — verify (run local validators)

```bash
./bin/oya vcs verify \
  --agent tutorial-$USER \
  --evidence "files:2 lanes:doc-coverage,no-secrets" \
  docs/architecture/notes microservices/intelligence/scratch
```

Expected:
```
oya vcs verify: lanes=2 passed=2 failed=0
  lean-a5-doc-coverage   PASS
  lean-a4-secret-cleartext PASS
```

If a lane fails, fix the underlying issue (do not skip the lane). Re-run.

## Step 4 — done

```bash
./bin/oya vcs done \
  --agent tutorial-$USER \
  --evidence "files:2 lanes:doc-coverage,no-secrets verify:green" \
  docs/architecture/notes microservices/intelligence/scratch
```

Expected:
```
oya vcs done accepted: action=close-claim agent=tutorial-$USER scopes=2 evidence=4
audit_chain_event_id: ce-2026-05-20T08:51:32.214Z-…
```

The claim is now closed; you can no longer modify those scopes without re-claiming.

## Step 5 — open the PR

```bash
git add docs/architecture/notes microservices/intelligence/scratch
git commit -m "tutorial: $USER foundry pipeline walkthrough 2026-05-20"
git push -u origin tutorial/$USER-foundry
gh pr create --base dev --title "tutorial: $USER foundry walkthrough" --body "Documents the claim/work/done/verify/promote cycle."
```

## Step 6 — watch the admission gate

```bash
gh pr checks --watch
```

You'll see (in order):
1. `foundry / preflight` (≤ 30 s)
2. `foundry / lean-a* lanes` (parallel; ≤ 4 min)
3. `foundry / build` (≤ 8 min for a doc-only change)
4. `foundry / tests` (skipped for doc-only paths)
5. `reviewer-agent / multispectrum-v2.4.0` (≤ 6 min)
6. `foundry / projected-merge-state` (≤ 30 s)
7. `foundry / merge` — fast-forwards `dev` to your PR tip

If any check fails, the PR returns to draft with a structured conflict report; iterate via fresh `claim → work → done → verify`
cycles.

## Step 7 — promote to dev

(`gh pr merge --auto --squash` already invoked this implicitly via the auto-merge label, but you can also run it manually for
non-auto-merge flows.)

```bash
./bin/oya vcs promote \
  --agent tutorial-$USER \
  --bundle tutorial-$USER-foundry-walkthrough-2026-05-20 \
  --environment dev \
  --evidence "files:2 admit:green merge:green" \
  docs/architecture/notes microservices/intelligence/scratch
```

Expected:
```
oya vcs promote accepted: action=promote-to-environment env=dev bundle=tutorial-…
audit_chain_event_id: ce-2026-05-20T09:02:14.319Z-…
```

## Step 8 — verify the audit chain

```bash
./bin/oya audit query --agent tutorial-$USER --window 1h
```

You should see linked events:
```
1. claim-lock          ts=08:48:11Z
2. lane-pass (×2)      ts=08:50:21Z
3. close-claim         ts=08:51:32Z
4. pr-admit            ts=08:51:35Z
5. lane-pass (×7)      ts=08:53…56Z
6. reviewer-verdict    ts=08:57:01Z
7. merge               ts=08:57:42Z
8. promote-to-env=dev  ts=09:02:14Z
```

Each event's `curr_hash` matches the next event's `prev_hash` — the chain is intact.

## Step 9 — cleanup the worktree

```bash
cd /Users/$USER/oyatie  # back to the main checkout
./bin/oya git worktree-remove --path .worktrees/$USER-foundry --force
```

## What you proved

- The full Foundry protocol is 5 verbs (claim/work/done/verify/promote) + a PR.
- Validators run on projected merge state, not branch tip.
- The reviewer-agent verdict is an explicit gate, not just a hint.
- Every step writes a linked audit-chain event.
- Promotion is a separate Cedar-gated primitive, distinct from PR merge.
