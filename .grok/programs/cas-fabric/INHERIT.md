# Inherited handoff — CAS-first fabric (parallel program)

**Inherited:** 2026-08-05 (this session)  
**Source:** https://github.com/jason931225/oyatie/issues/1560#issuecomment-5189694302  
**Recovery commit:** `fd2c9a5262b8b6f1266eef582e3eb78bf4885ea4` (`chore(recovery): bind handoff URL and ledger checkpoint`)  
**Plan SSOT:** `evidence/approved-plan-cas-re-20260805.md`  
**Plan SHA-256:** `8833df33de2600f0bd960518f2402dce20b27ef828cb3cbf27878b4caeaebae5` (**verified**)  

**Orchestration rule for this program:** use `.grok` harness only (`mm-goals`, `mm-pipeline`, workflows).  
**Do not** drive this with `gjc` / `omc` / `omx` / `hermes` CLIs. Ultragoal snapshots under `evidence/` are **read-only provenance**, not live schedulers.

---

## Parallel program board

| Lane | Program | Status | Hard stops |
|------|---------|--------|------------|
| **A** | Multi-agent delivery harness kit (`.grok/`) | Active on worktree `agent/mm-harness-20260805` | Not merge authority; portable kit |
| **B** | CAS/RE hyperscaler fabric (issue #1560) | **G039 in_progress** | Do not merge #1558 as-is; do not couple #1561 |
| **C** | Talos credential incident #1541 | Open / security | Separate lane; rebuild tracking open |
| **D** | K8s Go→Rust W0-A #1561 | Draft PR, another lane | **Do not touch** |

Ultragoal aggregate remains **active/checkpointing**, not completed. Only a future G044-class audit may complete the CAS program after quality gates.

---

## Live re-query (this inherit)

| Fact | Value |
|------|--------|
| `origin/dev` | `a4a5ace5fcba343ee979f7f1d4fa885ca41b9ff0` (#1559 squash: ADR amenders) |
| Plan/G038 baseline pin | still `b64eaaf4a…` (plan intentionally older) — **re-query ancestry before mutation** |
| #1559 post-merge CI | run [30990439972](https://github.com/jason931225/oyatie/actions/runs/30990439972) — status **queued** → **completion packet NOT closed** |
| #1558 | OPEN draft, head `54b22d0c…`, BLOCKED, `oya-ci-required` **FAILURE** |
| #1561 | OPEN draft, head `587ac30d…`, BLOCKED, checks mixed/queued — **out of scope** |
| Primary checkout | `agent/cas-live-proof-20260804` @ `06b501806` — **preservation only**, not implement base |
| Recovery ref | `archive/prewipe-20260805/fresh-agent-handoff` @ `fd2c9a526…` |

### Workspace preservation (from handoff)

- 1,739/1,780 paths preserved; 41 deliberate exclusions (generated / local metadata / secret-bearing)
- Final scan: zero findings (as of handoff capture — re-scan before wipe claims)
- Talos recovery archives + checksums verified; security/rebuild still open in **#1541**

---

## G039 — representative #1558 (active story)

**Story:** Run the representative #1558 storage-owned reorganization pilot  
**PR:** https://github.com/jason931225/oyatie/pull/1558  
**Safety boundary (PR body):** declarative prerequisites only — **does not** activate CAS/RE/ARC credentials or live cluster.

### PR #1558 files (exact)

- `infra/arc/tests/ci_workspace_capacity.rs` (+88)
- `infra/gitops/local-path-storage.yaml` (+177)
- `infra/talos/qemu-cilium.patch.yaml` (+8)

### Exact-head failures (run 30977703798)

1. `cloud-ci-firewall` (baseline ratchet + gate-registration) — FAILURE  
2. `buck2` (hermetic build + affected gate tests) — FAILURE  
3. `gate · affected-set` (ADR-0554) — FAILURE  
4. aggregate `oya-ci-required` — FAILURE  

Handoff note: Buck lane indicated **unindexed YAML** / corpus-index coverage after new YAML — **do not** assume all three failures share one root cause without re-diagnosis on a clean worktree.

### Next actions for Lane B (when implementing)

1. `git fetch origin dev` → new isolated worktree from **`a4a5ace5…`** (or fresher dev)  
2. Checkout PR head or recreate branch; **never** use dirty primary checkout  
3. Re-run failing gates locally / inspect logs; fix corpus index + firewall/affected-set as needed  
4. Keep draft until green + independent review; hard STOP on merge of current shape  
5. Do **not** mark G039 complete until exact-head `oya-ci-required` green + evidence checkpointed  

---

## Goals inheritance (status — not completed)

From recovered ultragoal snapshot (active/in_progress subset):

| ID | Status | Title |
|----|--------|-------|
| G001 | in_progress | Complete Oyatie through small merge-safe waves |
| G019 | in_progress | Stage-1 protected admission / open PR train |
| **G039** | **in_progress** | Representative #1558 storage pilot |
| G040–G044 | pending | NativeLink/CAS, credentials, production CAS, RE decision, program audit |
| G038 | complete | Reconcile Stage-1 + bind CAS/RE baseline |

Snapshots: `evidence/ultragoal-goals.snapshot.json`, `evidence/ultragoal-ledger.snapshot.jsonl`.

**mm-goals** run for this program: see `mm-runs/` id recorded in `PROGRAM.json`.

---

## Coupling to multi-agent harness (Lane A)

| Use harness for | Do not use harness for |
|-----------------|------------------------|
| Lens-plan diagnosis of #1558 CI failures | Claiming merge readiness |
| Dual-critic review of fix plan | Driving omx ultragoal scheduler |
| Score/grade of fix worktree | Touching #1561 or #1541 without assignment |
| Portability of kit to other repos | Implementing from dirty preservation checkout |

---

## Forbidden

- Reset/clean/rebase primary dirty checkout as implement base  
- Merge #1558 while draft + red  
- Couple #1561 into G039  
- Call G039/G044 complete without quality gates  
- Invoke `gjc` / `omc` / `omx` / `hermes` as control plane for this inherit  

---

## First five minutes for any fresh agent

1. Read this file + `PROGRAM.json`  
2. `git fetch origin dev && git rev-parse origin/dev`  
3. `gh pr view 1558` / `1561` / `gh run view 30990439972`  
4. Recover evidence from recovery ref if needed (already copied under `evidence/`)  
5. Choose lane A (harness) or B (#1558) — **isolated worktree only for B**  


---

## Re-query after session compact (2026-08-05T09:06ZZ)

Live facts re-verified (do not trust prior snapshot alone):

| Fact | Value |
|------|--------|
| handoff comment | https://github.com/jason931225/oyatie/issues/1560#issuecomment-5189694302 |
| recovery commit | `fd2c9a5262b8b6f1266eef582e3eb78bf4885ea4` (present; quarantine only) |
| plan SHA-256 | `8833df33…ae5` (matches recovery plan blob) |
| origin/dev | `a4a5ace5fcba343ee979f7f1d4fa885ca41b9ff0` |
| #1559 post-merge CI | run 30990439972 still **queued** → completion packet **open** |
| #1558 | draft, head `54b22d0c…`, three red lanes (firewall/buck2/affected-set) |
| #1561 | draft, head `587ac30d…` — **do not touch** |
| #1541 | open security/rebuild |
| G002 diagnosis | complete → `evidence/G039-DIAGNOSIS-1558.md` |
| G003 | **active** next (OWNERS + Buck package for 2 YAML) |
| Ultragoal | **active/checkpointing — not completed** |
| primary checkout | `agent/cas-live-proof-20260804` @ `06b501806` — preservation only |
| #1558 implement worktree | `oyatie-lanes-20260805/cas-proof-bootstrap` @ exact head |
| diag worktree | `oyatie-g039-1558-diag` |
