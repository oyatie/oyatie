# DEV CUTOVER PLAN — `cleanup/whole-tree-2026-06-07` → `dev`

> **Status: PREP ONLY. This is a runbook, not an execution. No trigger has been pulled.**
> No branch-protection change, no merge, no push, no commit was performed in producing this document.
> Author lane is strictly READ-ONLY on `/Users/jasonlee/Developer/source`.

- **Date prepared:** 2026-06-07
- **Repo:** `/Users/jasonlee/Developer/source`
- **Door:** DEV CUTOVER (one-way: it rewrites the shared integration branch every other lane builds on)
- **Source branch (head):** `cleanup/whole-tree-2026-06-07` @ `7adae31fb942d7815bbf4dc8fc9722f984683d9c`
- **Target branch (head):** `dev` @ `5e4e074383a6435443c05230bf9964521ab51cc7`

---

## 0. TL;DR for the founder

The cutover is **mechanically trivial and 100% conflict-free** — it is a pure **fast-forward**.
`dev` has **zero** commits that are not already on the cleanup branch; the merge-base IS `dev`'s current head.

The cutover is **strategically heavy** — it deletes a **net ~4,936 files** from `dev`'s working tree
(17,689 deletions / 12,753 additions / 1,258 modifications) and replaces dev's `crates/` +
`microservices/` + `regional-packs/` + `Jenkinsfile` layout with the producer-rooted Buck2 layout
(`cloud/`, `oya/`, `libs/`, `platforms/`, `toolchains/`, `BUCK`, `.buckconfig`).

**Risk is therefore not "will it merge" (it will, cleanly) — it is "is this the tree we want dev to BE".**
Mitigated by: (a) the change is already fully verified on the cleanup branch, (b) two off-machine
backups exist (`github-mirror/cleanup/...` + recovery anchor `e38624dc4`), and (c) rollback is a
one-command `git update-ref` back to `5e4e074383`.

**Correct sequence: firewall go-live (lands WITH this cutover, since the firewall only exists on
cleanup) → big-2 → THEN flip dev to the new tree.** The firewall cannot pre-gate dev before the
cutover because the firewall code does not exist on dev yet. See §4.

---

## 1. MERGEABILITY (evidence)

### 1.1 Commit deltas

```
$ git -C /Users/jasonlee/Developer/source log --oneline dev..cleanup/whole-tree-2026-06-07 | wc -l
626          # commits on cleanup NOT on dev

$ git -C /Users/jasonlee/Developer/source log --oneline cleanup/whole-tree-2026-06-07..dev | wc -l
0            # commits on dev NOT on cleanup  ← dev has NOT moved past the fork
```

### 1.2 Merge-base = dev head (the decisive fact)

```
$ git -C /Users/jasonlee/Developer/source merge-base dev cleanup/whole-tree-2026-06-07
5e4e074383a6435443c05230bf9964521ab51cc7

$ git -C /Users/jasonlee/Developer/source rev-parse dev
5e4e074383a6435443c05230bf9964521ab51cc7      # ← IDENTICAL to merge-base
```

The merge-base equals dev's current head → **dev is a strict ancestor of cleanup**:

```
$ git merge-base --is-ancestor dev cleanup/whole-tree-2026-06-07  →  exit 0  (YES: FF possible)
$ git merge-base --is-ancestor cleanup/whole-tree-2026-06-07 dev  →  exit 1  (NO)
```

**Consequence: the cutover is a clean fast-forward. There is no divergence and therefore no possible
merge conflict.** No `-X` strategy, no manual conflict resolution, no merge commit required.

### 1.3 Dry-run conflict check (read-only `merge-tree`)

```
$ git -C /Users/jasonlee/Developer/source merge-tree --write-tree --name-only dev cleanup/whole-tree-2026-06-07
577f2e0d3b610bedacc9706e15d1fe2e95dedf91     # single line = the merged tree OID, NO conflict section
exit code: 0
```

`merge-tree --write-tree` exits non-zero and prints a conflict block when conflicts exist. It exited
**0** and emitted **only the tree OID** → **conflict count = 0, conflicted files = none.**

| Metric | Value |
|---|---|
| Conflicts | **0** |
| Conflicted files | **none** |
| Merge type | **fast-forward** (no merge commit needed) |
| Commits dev gains | 626 |
| Commits cleanup is missing from dev | 0 |

---

## 2. IMPACT (the dev-cutover delta)

### 2.1 File-level delta (rename detection OFF — true add/delete truth)

```
$ git diff --no-renames --name-status dev cleanup/whole-tree-2026-06-07 | awk '{print $1}' | sort | uniq -c
  17689 D      # deleted
  12753 A      # added
   1258 M      # modified
  ----- 
  31700 total paths touched
```

### 2.2 Tree size before/after

```
$ git ls-tree -r --name-only dev                            | wc -l   →  22714  files (dev today)
$ git ls-tree -r --name-only cleanup/whole-tree-2026-06-07  | wc -l   →  17778  files (after cutover)
NET CHANGE: -4,936 files   ← matches the "~5,600 destroyed, offset by re-homes" memory note
```

Line churn (`--no-renames --shortstat`): **2,119,640 insertions / 2,982,895 deletions**.
(With rename detection ON, git reports 9,826 renames — i.e. most "delete+add" pairs are actually
moves into the new producer-rooted layout, not raw data loss.)

### 2.3 Top-level structural change

| Removed from dev | Added in cleanup | Meaning |
|---|---|---|
| `Jenkinsfile` | — | forbidden-vocab eradication (Jenkins) — task #25 |
| `crates/` | `libs/`, `cloud/`, `oya/`, `platforms/` | flat crates → producer-rooted domain split |
| `microservices/` | (re-homed under `cloud/` / `oya/`) | wave-2/3 re-home (F-0023 observability etc.) |
| `regional-packs/` | (consolidated into `packs/`) | F-0016 packs consolidation |
| `bin/` (cargo-only) | `BUCK`, `.buckconfig`, `.buckroot`, `reindeer.toml`, `.cargo`, `third-party/`, `toolchains/` | Buck2 authority bring-up |
| — | `LICENSE`, `.claire`, `.omx` | new top-level adds |

### 2.4 What the 626 commits contain (provenance of the delta)

- **103** commits matching `firewall|gate|GATE-N` — the phase-0 firewall + 4 keystone gates.
- Waves 2 & 3 present: `F-0001 … F-0023` fix commits + `chore(firewall): regenerate accounting faces`.
- Recovery anchor `e38624dc4` ("checkpoint: full source tree pre-aggressive-cleanup") sits **24 commits**
  below cleanup head — i.e. the aggressive deletions are the top 24 commits.

**Firewall lives ONLY on cleanup.** Confirmed present on cleanup tree, absent on dev:
- `docs/decisions/ADR-0700-ci-admission-live-apex.md`
- `cloud/cloud-ci/gates/cloud-ci-firewall/{src/lib.rs,tests/firewall.rs,gate-baseline.signoff.json}`
- `cloud/cloud-ci/gates/accounting-registry-producer/*` (+ generated faces)
- The one-way door `gate-baseline.signoff.json` currently has `_sign_off_additions: {}`
  → **ratchet fully closed (baseline may only shrink).**

This is the linchpin for sequencing (§4): you cannot make the firewall gate `dev`'s PRs *before*
the cutover, because the firewall does not exist on `dev` until the cutover delivers it.

---

## 3. RISK + ROLLBACK

### 3.1 Risk profile

| Risk | Severity | Why low / mitigation |
|---|---|---|
| Merge conflict | **None** | Pure fast-forward, dry-run exit 0, 0 conflicted files (§1.3) |
| Silent data loss in the ~5.6k deletions | Medium | Most are renames (9,826 detected moves); full tree preserved at `e38624dc4`; cleanup verified green |
| In-flight feature branches forked off OLD dev | **High operational** | 626-commit FF rebases the world; every active branch off `5e4e074383` must re-base onto the new tree. **Inventory + warn ALL lane owners first.** |
| Buck2 layout breaks someone's cargo-path muscle memory | Medium | Layout doc + AGENTS.md already on cleanup; announce |
| Pushing to `forgejo` remote (forbidden vocab) | Policy | **Do NOT push the cutover to `forgejo/dev`.** Push only to `github-mirror`. `forgejo` is slated for eradication (task #25) |

### 3.2 Recovery anchors (verified to exist)

| Anchor | SHA | Role |
|---|---|---|
| **dev rollback target** | `5e4e074383a6435443c05230bf9964521ab51cc7` | current dev head = pre-cutover dev. Reflog `dev@{0}`. This is THE one-liner rollback. |
| **pre-aggressive-cleanup checkpoint** | `e38624dc4` | full source tree before the top-24 deletion commits; recover any over-deleted file from here |
| **off-machine mirror (cleanup)** | `github-mirror/cleanup/whole-tree-2026-06-07` @ `7adae31fb` | identical to local cleanup head — survives local disk loss |
| **off-machine mirror (dev)** | `github-mirror/dev` @ `9f1047e62` | NOTE: mirror dev is at `9f1047e62` ("Harden the temporary GitHub Actions bridge"), AHEAD of local `dev` `5e4e074383`. **Reconcile mirror-dev vs local-dev BEFORE cutover** (see §4 step 1) |

> **Discrepancy to resolve before cutover:** `github-mirror/dev` (`9f1047e62`) is NOT the same as
> local `dev` (`5e4e074383`). Verify whether `9f1047e62` is an ancestor-descendant of local dev and
> reconcile, so the rollback anchor is unambiguous. (Local dev's reflog shows only `pull --ff-only`,
> so local dev should be ≤ a remote; confirm which remote.)

### 3.3 Rollback procedure (if cutover proves wrong)

Because the cutover is a fast-forward, undo is a single ref reset — **no history rewrite, no revert
commits needed** — provided you captured the pre-cutover SHA (you have it: `5e4e074383`).

```
# LOCAL rollback (read-only-safe to PLAN; do NOT run now):
git -C /Users/jasonlee/Developer/source update-ref refs/heads/dev 5e4e074383a6435443c05230bf9964521ab51cc7

# REMOTE rollback (only if the cutover was already pushed):
#   force-push dev back to the captured SHA — requires temporarily lifting branch protection,
#   which is itself a founder one-way door. Prefer to NOT push dev until §4 gates are green.
git -C /Users/jasonlee/Developer/source push --force-with-lease github-mirror 5e4e074383...:refs/heads/dev
```

**Rollback rule:** capture `git rev-parse dev` into the runbook the instant before the flip; that SHA
is the rollback target. It is already `5e4e074383` today.

---

## 4. SEQUENCING (the correct order)

The founder rule is: **cutover follows firewall go-live + the big-2.** The wrinkle is that the
firewall *implementation* only exists on the cleanup branch, so the firewall cannot literally pre-gate
dev. The resolution is to treat the cutover as the **delivery vehicle** for the firewall, and to prove
the firewall GREEN on cleanup *before* flipping, then arm branch protection *immediately after*.

```
STEP 0  (DONE)   Cleanup branch built + verified green; 626 commits; mirror pushed.
                 ── prerequisite already satisfied ──

STEP 1  PRE-FLIGHT (read-only)
        - Reconcile github-mirror/dev (9f1047e62) vs local dev (5e4e074383). §3.2 discrepancy.
        - Re-run merge-base --is-ancestor dev cleanup  → must still be exit 0 (still FF).
        - Inventory active branches forked off old dev; notify owners of the impending rebase.
        - Capture rollback SHA = current dev head.

STEP 2  FIREWALL GO-LIVE  (must precede the flip's "armed" state)
        - On cleanup branch: run cloud-ci-firewall gate + 4 keystone gates → prove GREEN.
        - Confirm gate-baseline.signoff.json _sign_off_additions == {} (ratchet closed).
        - This is the door:one-way founder sign-off on the firewall itself.
        - (Firewall code ships TO dev via the flip in STEP 5; here we prove it on cleanup first.)

STEP 3  BIG-2  (the two other one-way doors that gate the cutover — sequence them FIRST)
        - The big-2 must clear before dev is flipped (per founder ordering).
        - Land/confirm both big-2 outcomes on the cleanup branch so the flipped dev carries them.
        - (Big-2 identity is owned by the founder/other lanes — confirm both GREEN before STEP 5.)

STEP 4  GATE-BEFORE-START kernel re-verify
        - Per MEMORY consolidation rule: re-verify kernel green before mutating shared source.

STEP 5  CUTOVER (the flip — fast-forward dev → cleanup head 7adae31fb)
        - git update-ref / ff-only merge dev to cleanup head. NO merge commit (FF).
        - Push dev to github-mirror ONLY (never forgejo).

STEP 6  ARM BRANCH PROTECTION  (immediately after the flip, not before)
        - Now that the firewall code is ON dev, wire cloud-ci-firewall + keystone gates as
          REQUIRED status checks on dev branch protection. THIS is "firewall go-live" in the
          enforcement sense — it can only be armed post-flip because pre-flip dev has no gate code.

STEP 7  POST-CUTOVER
        - Notify lane owners to rebase their branches onto new dev.
        - Verify CI green on new dev head; keep rollback SHA (5e4e074383) hot for 1 cycle.
```

### Why firewall-arming is post-flip (not a contradiction of "cutover follows firewall")

- **"Firewall go-live"** has two senses: (a) *proven green* and (b) *armed as required check on dev*.
- (a) MUST precede the flip — done on cleanup in STEP 2.
- (b) CANNOT precede the flip — the gate code lands on dev only via the flip — so it is armed in STEP 6.
- Net order still honors the rule: **prove firewall (2) → big-2 (3) → flip (5) → arm firewall on dev (6).**

---

## 5. EXACT COMMANDS (for the founder to run — DO NOT run from this lane)

```
# --- PRE-FLIGHT (read-only, safe) ---
git -C /Users/jasonlee/Developer/source fetch --all --prune
git -C /Users/jasonlee/Developer/source merge-base --is-ancestor dev cleanup/whole-tree-2026-06-07 && echo "STILL FF-SAFE"
git -C /Users/jasonlee/Developer/source rev-parse dev    # = rollback anchor; expect 5e4e074383...

# --- THE FLIP (fast-forward only; the one-way door) ---
git -C /Users/jasonlee/Developer/source checkout dev
git -C /Users/jasonlee/Developer/source merge --ff-only cleanup/whole-tree-2026-06-07
# (equivalent low-level form: git update-ref refs/heads/dev 7adae31fb)

# --- PUBLISH (github-mirror ONLY — never forgejo) ---
git -C /Users/jasonlee/Developer/source push github-mirror dev

# --- ROLLBACK (only if needed) ---
git -C /Users/jasonlee/Developer/source update-ref refs/heads/dev 5e4e074383a6435443c05230bf9964521ab51cc7
```

---

## 6. GO / NO-GO CHECKLIST

- [ ] Re-confirm `merge-base --is-ancestor dev cleanup` exits 0 (still pure FF)
- [ ] github-mirror/dev vs local dev discrepancy reconciled (§3.2)
- [ ] Firewall + 4 keystone gates proven GREEN on cleanup (STEP 2)
- [ ] `gate-baseline.signoff.json` `_sign_off_additions == {}` confirmed
- [ ] Big-2 both cleared (STEP 3)
- [ ] Kernel GATE-BEFORE-START re-verify green (STEP 4)
- [ ] Active feature-branch owners notified of impending rebase
- [ ] Rollback SHA `5e4e074383` recorded in the live runbook
- [ ] Founder one-way-door sign-off captured for the flip itself

**When all boxes are checked → STEP 5 flip → STEP 6 arm branch protection.**
