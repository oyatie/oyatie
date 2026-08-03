# Next-wave lane plan after the serial G004 shared-producer lane

**Goal:** keep the next wave throughput-safe by fanning out only on disjoint, fresh `origin/dev` worktrees after the post-merge checks are green.

**Hard preconditions**
- No write fanout until `origin/dev` post-merge checks are green.
- One fresh `origin/dev` worktree per lane.
- No lane may touch generated JSON, `.github/workflows/**`, `oya-ci.toml`, root `specs/**`, `.omx/ultragoal/**`, or any hot/shared producer unless the lane is explicitly single-writer.

**Grounding**
- G003 is explicitly boundary/read-only: the operation contract says the backend actuation boundary is an intent-only marker, not runtime work (`specs/cloud-control-plane-operation-contract.json:61-68`, `366-375`).
- Workspace hygiene is inventory-by-default, with cleanup only behind explicit flags (`specs/workspace-hygiene.json:13-23`, `55-123`).
- G003 evidence already concluded there is no active GraphQL runtime crate in the owned scope (`.claude/worktrees/console-design-authority/evidence/wave-d-g003-g006/g003/graphql-boundary-evidence.md:5-18, 30-36`).
- G003 runtime-state evidence is inventory-only and keeps `.omx/ultragoal/**` out of cleanup interpretation (`.claude/worktrees/console-design-authority/evidence/wave-d-g003-g006/g003/runtime-state-classification.md:5-26`).
- G006 backlog rows are still Python legacy bridges and explicitly fenced as local-only until ported to Rust/Buck2 cloud-ci gates (`.claude/worktrees/console-design-authority/specs/language-discipline-registry.json:89-145`).
- The actual gate-app directories present in this checkout are leaf-gate dirs plus one shared producer dir (`cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/**`, `.../oya-cloud-ci-slo-coverage-app/**`, `.../oya-cloud-ci-generated-artifact-control-plane-app/**`, etc.).

## Ranked lanes

### 1) G003 — read-only boundary lane
**Parallelism:** read-only only; can run alongside other read-only inspection, but should not consume a writer slot.  
**Owned paths:** `evidence/wave-d-g003-g006/g003/**`  
**Inspect-only paths:** `oya/intelligence/crates/oya-intelligence-api-graphql-kernel/**`, `oya/intelligence/crates/oya-intelligence-api-graphql-adapter/**`

**Why first:** it is already truth-labeled as a no-active-GraphQL / runtime-boundary check, so it can prove “nothing to do” without spending a writer on source edits.

**Verify**
- `git status --short --branch`
- `git worktree list`
- `test -d oya/intelligence/crates/oya-intelligence-api-graphql-kernel || true`
- `test -d oya/intelligence/crates/oya-intelligence-api-graphql-adapter || true`
- `git ls-files 'oya/intelligence/crates/oya-intelligence-api-graphql-kernel/**' 'oya/intelligence/crates/oya-intelligence-api-graphql-adapter/**'`
- `buck2 targets 'oya/intelligence/crates/oya-intelligence-api-graphql-kernel/...' 'oya/intelligence/crates/oya-intelligence-api-graphql-adapter/...'`

**Stop / no-op**
- Stop if any in-scope GraphQL runtime crate reappears or the lane would need to edit root specs / generated files.
- No-op if the evidence still says “no active GraphQL runtime exists”; record the boundary note only.

### 2) G004 app-local hardening A — one gate app directory
**Parallelism:** 1 writer, isolated to one gate app directory.  
**Owned paths:** `cloud/cloud-ci/gates/oya-cloud-ci-slo-coverage-app/**`

**Why here:** it is a leaf gate already wired to the shared producer, but the gate app itself is self-contained and does not require shared producer edits.

**Verify**
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-slo-coverage-app:oya-cloud-ci-slo-coverage-app-unittest`
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-slo-coverage-app:oya-cloud-ci-slo-coverage-app-gate`

**Stop / no-op**
- Stop if the lane needs `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/**` edits.
- Stop if it would touch any generated JSON face or workflow file.

### 3) G004 app-local hardening B — one gate app directory
**Parallelism:** 1 writer, isolated to one gate app directory.  
**Owned paths:** `cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app/**`

**Why here:** another leaf gate with its own directory; safe to run in parallel with lane 2 as long as it stays app-local.

**Verify**
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app:oya-cloud-ci-generated-artifact-control-plane-app-unittest`
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-generated-artifact-control-plane-app:oya-cloud-ci-generated-artifact-control-plane-app-gate`

**Stop / no-op**
- Stop if the lane expands into `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/**` or any root policy/generated surface.

### 4) G006 tiny leaf lane A — legacy bridge evidence
**Parallelism:** tiny leaf-only; one writer max.  
**Owned paths:** `scripts/tests/cloud_observability_slo_evidence_check.py`

**Why here:** the registry still labels this validator as a local legacy bridge, so the safe move is a narrow leaf-only lane, not a broad rewrite.

**Verify**
- `python3 scripts/tests/cloud_observability_slo_evidence_check.py`
- `rg -n "cloud_observability_slo_evidence_check" specs/language-discipline-registry.json`

**Stop / no-op**
- Stop if the change requires any root spec rewrite or generated output.
- No-op if the script is already merely fenced and the only remaining work is a truth-label/evidence note.

### 5) G006 tiny leaf lane B — production-quality-kit backlog bridge
**Parallelism:** tiny leaf-only; one writer max.  
**Owned paths:** `scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`

**Why here:** same pattern as lane 4, but separate file ownership keeps the lane disjoint and cheap.

**Verify**
- `python3 scripts/tests/cloud_production_quality_kit_evidence_backlog_check.py`
- `rg -n "cloud_production_quality_kit_evidence_backlog_check" specs/language-discipline-registry.json`

**Stop / no-op**
- Stop if this would spill into the target/evidence JSONs or any generated face.

### 6) G005 serial holdback — shared producer lane
**Parallelism:** serial only; one writer; do not fan out.  
**Owned paths:** `cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app/**`

**Why last:** this is the shared producer. It serializes by design, so it should be queued only after the read-only and leaf lanes are either green or explicitly no-op.

**Verify**
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-unittest`
- `buck2 test //cloud/cloud-ci/gates/oya-cloud-ci-accounting-registry-app:oya-cloud-ci-accounting-registry-app-bin-unittest`

**Stop / no-op**
- Stop immediately on any shared-file request, any generated JSON touch, or any attempt to overlap another gate-app directory.
- No-op until the other lanes are green and the leader has a fresh `origin/dev` collision-free worktree.

## Wave-level stop / no-op criteria

Stop the wave if any of the following happens:
1. `origin/dev` post-merge checks are not green.
2. A lane needs a shared producer or any file on the forbidden list.
3. A lane would require more than one writer on the same gate app directory.
4. A lane can only be made green by editing generated JSON, root specs, workflows, or `oya-ci.toml`.
5. A lane claims progress without a fresh `git worktree list` / `git status --short --branch` snapshot.

No-op the wave if:
- G003 still reads as a no-active-GraphQL boundary.
- G006 is still only legacy-bridge evidence with no new local leaf work.
- The G005 producer lane is blocked by any shared-file collision.

