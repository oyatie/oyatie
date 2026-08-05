## G038 baseline reconciliation — approved execution handoff

Fresh read-only baseline: `origin/dev@b64eaaf4ab40f7428e3a27d7cd4b02930404eee9`. Implementation must use the existing clean #1558 worktree whose parent and merge-base are that exact head; the dirty primary checkout is excluded.

### Baseline and authority

- Legacy Stage 1 is closed: #1361 (`e4e7b3c`), #1362 (`cf72c1b`), #1363 (`70144ea`), and #1364 (`b29b92e`) are merged.
- #1558 is open/draft at `54b22d0c6470d8008012542eb37d0ff32b72e1b5`. Git reports `MERGEABLE`; protected admission is `BLOCKED` by draft status, absent independent approval, and failing exact-head checks. MERGEABLE is not merge-ready.
- #1559 is open/ready-for-review at `4d652c652e82c400793fa425e341fe6f470d2fbc`; its required check is green, but it has no review decision.
- Approved plan: `.omx/plans/cas-re-hyperscaler-capability-reorg-20260805.md`, SHA-256 `8833df33de2600f0bd960518f2402dce20b27ef828cb3cbf27878b4caeaebae5`, byte-identically preserved in issue #1560. This is execution guidance, not PR approval or runtime activation authority.

### Representative pilot

The only move plan is `specs/reorg/local-path-storage-move-plan.json`:

```json
{"capability":"storage","moves":[],"artifacts":[{"old_path":"infra/gitops/local-path-storage.yaml","new_path":"storage/adapters/local-path/local-path-storage.yaml"}]}
```

Always pass this plan explicitly: the source is branch-only and absent from the merge base, so automatic artifact-only discovery may classify it as already landed. Never commit or hand-edit `*.generated.json`.

The executable contract is `pilot-local-path-population/v1`:

- one structurally parsed `StorageClass/local-path` provider, at the old path before the move or the new path after it, never both;
- four exact runtime edges: NativeLink, registry, SeaweedFS, and `oya/observability/iac/k8s/helm/values.yaml#/grafana/persistence/storageClassName`;
- one structurally selected Argo `Application/local-path-storage` edge whose `spec.source.path` changes from `infra/gitops` to `storage/adapters/local-path` while `directory.include` remains `local-path-storage.yaml`.

Required result: `N_pre = N_post = N_promoted = 6 > 0`. The differently shaped StatefulSet probe must produce `6 -> 7 -> 6`.

### Hermetic Buck2 binding and preparation heads

`//storage/adapters/local-path:local-path-contract` must declare every live YAML input plus the probe through `resources` and one `env = { ...: "$(location <label>)" }` entry per input. It must not read the repository root. Canonical `corpus_yaml_facts_shards` declarations index each YAML-owning package but are not an implicit dependency of the Rust verdict.

Stage A is an SSH-signed immutable pre-harness commit atop #1558's current head:

- new canonical indexed/export packages: `infra/gitops/BUCK`, `infra/nativelink/BUCK`, `infra/registry/BUCK`, `infra/seaweedfs/BUCK`;
- one public single-file `//oya:observability-grafana-values` filegroup in existing `oya/BUCK`;
- destination `storage/adapters/local-path/BUCK`, Rust contract, and probe;
- provider binding `//infra/gitops:local-path-storage.yaml`.

Stage B is a later SSH-signed immutable head: apply the explicit codemod plan, remove the old export, bind provider `:local-path-storage.yaml`, update only the named GitOps Application, delete the unconsumed QEMU candidate, and keep the same Rust logic.

Four infra packages adopt ten existing YAML files. Expected final ratchet values are `baseline_unpackaged_yaml_files 448 -> 438`, packages `37/19 indexed`, YAML `5855 total / 5363 indexed / 438 unpackaged`, package/file coverage `5135/9159 bps`. These are expectations, not proof: the executor must use live gate output, lower the source policy only to measured values, and stop rather than fabricate or raise a ceiling.

### Exact commands

```text
# pre signed head
mkdir -p .omx/tmp/g039
test -z "$(git status --porcelain)"
test "$(git rev-parse HEAD^)" = "54b22d0c6470d8008012542eb37d0ff32b72e1b5"
COMMON_GIT_DIR=$(git rev-parse --git-common-dir); git -c gpg.ssh.allowedSignersFile="$COMMON_GIT_DIR/omx-local/allowed_signers" verify-commit HEAD
git rev-parse HEAD > .omx/tmp/g039/pre-harness-head.txt
set -o pipefail; PRE_SHA=$(git rev-parse HEAD); buck2 test //storage/adapters/local-path:local-path-contract 2>&1 | tee ".omx/tmp/g039/${PRE_SHA}.pre-local-path-contract.log"
buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- dry-run --plan specs/reorg/local-path-storage-move-plan.json --repo-root "$PWD"
buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- manifest --plan specs/reorg/local-path-storage-move-plan.json --repo-root "$PWD" --out .omx/tmp/g039/local-path-move-manifest.pre.generated.json

# apply
buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- apply --plan specs/reorg/local-path-storage-move-plan.json --repo-root "$PWD"

# post signed head
mkdir -p .omx/tmp/g039
test -z "$(git status --porcelain)"
test "$(git rev-parse HEAD^)" = "$(cat .omx/tmp/g039/pre-harness-head.txt)"
COMMON_GIT_DIR=$(git rev-parse --git-common-dir); git -c gpg.ssh.allowedSignersFile="$COMMON_GIT_DIR/omx-local/allowed_signers" verify-commit HEAD
git rev-parse HEAD > .omx/tmp/g039/post-move-head.txt
set -o pipefail; POST_SHA=$(git rev-parse HEAD); buck2 test //storage/adapters/local-path:local-path-contract 2>&1 | tee ".omx/tmp/g039/${POST_SHA}.post-local-path-contract.log"
buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- dry-run --revert --plan specs/reorg/local-path-storage-move-plan.json --repo-root "$PWD"
buck2 run //tools/oya-reorg-codemod-app:oya-reorg-codemod -- manifest --plan specs/reorg/local-path-storage-move-plan.json --repo-root "$PWD" --out .omx/tmp/g039/local-path-move-manifest.post.generated.json
buck2 test //ci/facade/corpus-index-coverage:ci-corpus-index-coverage-gate

# promoted proof
mkdir -p .omx/tmp/g039
git fetch origin dev
test ! -e ../g039-promoted-proof
git worktree add --detach ../g039-promoted-proof origin/dev
test -z "$(git -C ../g039-promoted-proof status --porcelain)"
test "$(git -C ../g039-promoted-proof rev-parse HEAD)" = "$(git rev-parse origin/dev)"
git -C ../g039-promoted-proof rev-parse HEAD > .omx/tmp/g039/promoted-head.txt
(cd ../g039-promoted-proof && buck2 test //storage/adapters/local-path:local-path-contract)
(cd ../g039-promoted-proof && buck2 test //ci/facade/corpus-index-coverage:ci-corpus-index-coverage-gate)
PROMOTED_SHA=$(cat .omx/tmp/g039/promoted-head.txt) && CHECK_URL=$(gh api "repos/jason931225/oyatie/commits/${PROMOTED_SHA}/check-runs" --jq '[.check_runs[] | select(.name=="oya-ci-required" and .conclusion=="success")][-1].html_url // empty') && test -n "$CHECK_URL" && printf '%s\n' "$CHECK_URL" | tee ".omx/tmp/g039/${PROMOTED_SHA}.oya-ci-required-url.txt"
```

Rust/Buck2/codemod evidence does not replace independent approval plus exact-head protected `oya-ci-required` green.

### Ownership and stops

- `axis-cloud-platform` via `storage/OWNERS`: destination provider, BUCK, Rust contract, and probe.
- `council-architecture` via `specs/OWNERS`: exact artifact-only plan and narrow reachability entry.
- `cloud-ci-platform` via `infra/arc/OWNERS`: ARC-only test cleanup.
- `cloud-ci-platform` via `infra/nativelink/OWNERS`: NativeLink package/export; manifest content remains read-only.
- `cloud-ci-platform` via root `OWNERS`: GitOps/registry/SeaweedFS package/export wiring, `oya/BUCK` export, source deletion, exact Application rewrite, and deletion of `infra/talos/qemu-cilium.patch.yaml`. Runtime consumer YAML content remains read-only.
- `cloud-ci-platform` via `ci/OWNERS`: measured shrink-only corpus policy update.
- One #1558 lane owns all writes; no concurrent lane consumes an unpromoted predecessor.

Accepted ADR-0515/0562/0614/0615 govern admission, placement, derived manifests, and boundaries. ADR-0562 records source date `2026-06-14` and formal acceptance `2026-07-10`; ADR-0614 records source date `2026-07-09` and acceptance `2026-08-01`. Proposed ADR-0560/0612/0630 are design input only. Stop on a second home, undeclared input, population other than six, failed reverse proof, absent approval, or non-green exact-head CI. Do not activate CAS until #1541 and legal/architecture/identity gates close; do not implement or activate RE under Proposed ADR-0612.

### Review history

Initial Architect: **APPROVE**. Initial Code Reviewer: **ITERATE**. Regression Architect: **BLOCK** on traceability dates only, with no architecture regression. Second Code Reviewer: **ITERATE** on hermetic input binding, complete ownership, exact evidence, and receipt consistency. Later command-safety reviews also required clean/signature/ancestry/SHA binding, pipefail for logged Buck2 tests, an explicit non-empty exact-head check assertion, and tests inside the detached promoted worktree; those findings are remediated. Final independent reviewer and verdict remain pending. No implementation/source file, GitHub object, credential, or cluster state was changed by G038 remediation.
