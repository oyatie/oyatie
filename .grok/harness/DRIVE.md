# mm-drive — autonomous outer loop

Ralph/ultragoal **behavior** on the single mm-delivery pipeline.  
**Not** omc/omx/gjc. Merge is **conditional**: independent formal APPROVE + green + no critical blockers (never self-approve).

## Loop

```text
Stop hook → mm-drive stop-hook → block|allow
                ↓
           mm-drive tick/status
                ↓
        resolvable → fix CI / implement
        merge_ready → merge-check → merge (squash)
                ↓
        R3 post-merge packet on promoted SHA
                ↓
        SCORE_GRADE / LEARN / mm-quant
```

## Commands

```sh
.grok/bin/mm-drive status --json
.grok/bin/mm-drive tick --json
.grok/bin/mm-drive briefs --json
.grok/bin/mm-drive stop-hook   # stdin = Grok Stop event JSON
.grok/bin/mm-drive merge-check --pr N
.grok/bin/mm-drive merge --pr N [--dry-run]
.grok/bin/mm-drive packet --pr N [--program PROG] [--dry-run]
.grok/bin/mm-packet --pr N     # same R3 packet surface
.grok/bin/mm-drive checkpoint-check --quality-gate-json PATH [--goal-id G…] [--evidence …]
```

## Autonomous merge

When `merge-check` returns `ok: true`:

1. Independent non-author `APPROVED` review (or `reviewDecision=APPROVED` with non-author approver)
2. `oya-ci-required` SUCCESS on exact head
3. Not draft; mergeable; not CHANGES_REQUESTED; no unresolved review threads
4. Then `mm-drive merge --pr N` (squash)

Until then: class stays `human_blocked` — wait for reviewer, do not thrash.
After merge: run **R3 packet** automation from **promoted** merge commit + trunk CI (not PR-head alone). See **D5** below.

## D5 — R3 post-merge packet (`mm-drive packet` / `mm-packet`)

Automates the product-completion packet template after squash to `origin/dev`.

**Canon:** `programs/hyperscaler-delivery-lanes/R3-postmerge-packet-template.md`  
**Example filled:** `programs/cas-fabric/evidence/1559-post-merge-completion-packet.json`

### Fail-closed gates

| Gate | Required |
|------|----------|
| PR state | `MERGED` (or `--sha` with associated merged PR / SHA-only path) |
| Promoted commit | PR `mergeCommit.oid` (not PR head) |
| Trunk CI | `oya-ci-required` **SUCCESS** on **exact** promoted SHA |

Exit **2** when gates fail. Does **not** claim ultragoal complete (`ultragoal_complete: false` always). **Not merge authority.**

### Usage

```sh
# Dry-run against a known merged PR (example #1559)
.grok/bin/mm-packet --pr 1559 --program cas-fabric --dry-run

# Same via mm-drive
.grok/bin/mm-drive packet --pr 1559 --program cas-fabric --dry-run

# Write under .grok/programs/<program>/evidence/<pr>-post-merge-completion-packet.json
.grok/bin/mm-packet --pr 1559 --program cas-fabric

# SHA path (promoted tip); optional --pr when known
.grok/bin/mm-packet --sha a4a5ace5fcba343ee979f7f1d4fa885ca41b9ff0 --program cas-fabric --dry-run

# Explicit output path + human narrative fields
.grok/bin/mm-packet --pr 1559 --out /tmp/packet.json \
  --rollout-class docs-only \
  --rollback-note 'Revert promoted SHA on dev' \
  --observability-check 'N/A docs-only' \
  --browser-user-story 'N/A' \
  --release-impact 'N/A' \
  --harvest-cards none \
  --harvest-rationale 'No new defect cards' \
  --closed

# Open / unmerged PR → fail-closed
.grok/bin/mm-packet --pr 1561 --dry-run   # exit 2, pr_not_merged:OPEN
```

### Behavior notes

- Machine-fills: `pr`, `title`, `promoted_commit`, `oya_ci_required.{status,run_url,head_sha,completed_at}`, `promoted_at`, `pr_state`, `merge_method`.
- Human fields (rollout / rollback / observability / browser / release / harvest) preserved from an existing packet at `--out` unless `--no-merge-existing`.
- `completion_packet_closed` stays `false` unless `--closed` **and** gates pass **and** human fields are non-empty.
- Default program path: `hyperscaler-delivery-lanes` when `--program` omitted.
- Optional `MM_PACKET_FETCH=1` runs `git fetch origin dev` before tip compare; `MM_PACKET_NO_CLOBBER=1` refuses overwrite without `--force`.
- Never edits product `oya-ci-required` workflows; never squash-merges; never closes G039/G001/ultragoal.

## Fine-tune

Edit `harness/drive.v1.json` (max stop blocks, whether waiting_ci blocks Stop, human gate classes).

## D3 — checkpoint-check (drive ↔ goals)

Fail-closed preflight **before** `mm-goals checkpoint --status complete`.  
`mm-goals checkpoint` **invokes** `mm-drive checkpoint-check` and refuses to mutate `goals.json` when `ok=false`.

Prevents false ultragoal / G039 / G001 completion from draft-only, placeholder, or local-green-only evidence.

### Policy (`harness/drive.v1.json` → `aggregate_complete`)

| Key | Default | Effect |
|-----|---------|--------|
| `require_quality_gate` | true | Refuse complete if `--quality-gate-json` missing |
| `forbid_from_draft_only` | true | Refuse if evidence is draft-only without promoted proof |
| `forbid_from_local_green_only` | true | Refuse if evidence is local-green only without promoted proof |

### Usage

```sh
# Refuse: no quality gate
.grok/bin/mm-drive checkpoint-check --goal-id G039

# Refuse: pass!=true
.grok/bin/mm-drive checkpoint-check \
  --goal-id G039 \
  --quality-gate-json '{"pass":false}' \
  --evidence 'local green'

# Refuse: local green without promoted sha / run URL
.grok/bin/mm-drive checkpoint-check \
  --quality-gate-json '{"pass":true}' \
  --evidence 'local green'

# Refuse: terminal goal without promoted markers
.grok/bin/mm-drive checkpoint-check \
  --goal-id G039 \
  --quality-gate-json '{"pass":true}' \
  --evidence 'revalidated on origin/dev tip'

# OK: quality gate pass + promoted proof markers
.grok/bin/mm-drive checkpoint-check \
  --goal-id G039 \
  --quality-gate-json path/to/gate.json \
  --evidence 'promoted_commit=a4a5ace5fdocs(adr)… run_url=https://github.com/…/actions/runs/123 oya-ci-required SUCCESS post-merge packet'
```

### Output

Always prints JSON:

```json
{"ok": true|false, "reasons": ["quality_gate_missing", "…"], "goal_id": "…", "status": "complete", …}
```

| Reason | Meaning |
|--------|---------|
| `quality_gate_missing` | No `--quality-gate-json` while policy requires it |
| `quality_gate_pass_not_true` | Gate loaded but `pass` is not strictly `true` |
| `quality_gate_json_invalid` | Path/inline not parseable JSON |
| `evidence_missing` | No evidence text, path content, or goal evidence |
| `evidence_placeholder_only` | Evidence is TODO/TBD/WIP/empty placeholder |
| `evidence_local_green_only` | Claims local green without promoted sha / run URL |
| `evidence_draft_only` | Claims draft-only green without promotion proof |
| `terminal_goal_requires_promoted_proof` | G039/G001/ultragoal complete without promoted markers |
| `evidence_path_draft_packet` | Draft packet path without promoted proof |

Promoted-proof markers (any one accepts the heuristic): 40-char SHA, `promoted_commit` / `promoted_sha`, `post-merge`, `origin/dev`, `oya-ci-required`, `actions/runs/<id>`, `run_url`, GitHub Actions run URL.

Optional: `MM_DRIVE_CHECKPOINT_EXIT=1` maps `ok=false` → process exit 1 for shell fail-closed.

Non-`complete` `--status` values are not gated (`ok: true`, note `non_complete_status_not_gated`).

**Not merge authority.** This check does not merge, undraft, or self-approve.

## D4 — briefs path-overlap (parallel lanes)

`mm-drive briefs` / `tick` / `status` report path-overlap among **resolvable** + **merge_ready** lanes so parallel spawns do not dual-write.

Sources (in order): lane `surfaces`/`paths`, matching `PROGRAM.json` lane surfaces, then id heuristics (kit → `.grok/…`, CAS → `ci/`/`infra/`, …).

```sh
MM_DRIVE_KIT=1 .grok/bin/mm-drive briefs --offline --json
# exit 1 when path_overlap.fail_closed and pairs non-empty
# opt out: MM_DRIVE_PATH_OVERLAP_EXIT=0
```

```json
{
  "briefs": [{"id": "R8-…", "path_overlap_conflict": true, "parallel_ok": false, …}],
  "path_overlap": {
    "ok": false,
    "fail_closed": true,
    "pairs": [{"a": "R4-…", "b": "R8-…", "reasons": ["worktree_collision", "surface_overlap"], …}]
  }
}
```

Policy: `harness/parallelism.v1.json` → `path_overlap.fail_closed` (default true).  
Overlap uses the same prefix/* rules as `mm-paths`. **Not merge authority.**

## Enable hooks

Project file: `.grok/hooks/mm-drive-stop.json`  
Trust once: `/hooks-trust` in this repo.

## Hard stops

- Never auto-merge or self-approve  
- Never complete G039/G001/ultragoal without quality-gate evidence  
- Cap stop continuations (default 3/turn; Grok also caps at 8)  
- Hook classifies only; agents implement under lane hard_stops  

## Separated hyperscaler lanes

SSOT: `programs/hyperscaler-delivery-lanes/LANES.md` (session `.grok`).

| Lane | Agent? | Note |
|------|--------|------|
| R1 runner capacity | Ops/human | Short-term queue fix |
| R2 pre-merge shape | Yes, own PR | Path-filter / materialize-once |
| R3 post-merge trunk | Yes, own PR | Packets / trusted writers |
| R4 local assist | Yes, kit | See `memory/tips/local-prepush-new-yaml.md` |
| R5 CAS #1558 | Yes, cas worktree | Not RE; not warm without #1541 |
| R6 RE | Blocked | After R5 terminal |
| R7 k8s #1561 | Human review | CI already green |
| R8 mm-drive kit | Yes | This doc + drive.v1.json |
| R9 #1541 | Human security | No secrets automation |

Do not couple R1+R2+R5 in one PR.
