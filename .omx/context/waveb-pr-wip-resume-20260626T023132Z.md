# Wave B PR + WIP resume context

## Outcome
Close the current Wave B safely: preserve WIP, fix/check/merge open PRs #882/#883/#884 after review plus green `oya-ci-required`, and continue issue #879 without losing uncommitted work.

## Constraints
- One worktree per lane; one writer per branch/hot file.
- Do not hand-edit `*.generated.json`.
- Backend verification uses Buck2 only; Cargo-based test authority is retired.
- No new dependencies unless explicitly required.
- GitHub Actions is transitional; CI must remain productized, universal, hermetic, comprehensive, cloud-native/API-driven.
- Merge only after review plus green `oya-ci-required`; leader owns final merge.

## Current PR branches
- #882 cloud-ci: branch `agent/waveB-cloud-ci-873-20260626`, worktree `/Users/jasonlee/oyatie-worktrees/waveA-cloud-ci-20260625173629`, latest pushed `b640a4102f41e650dc6c8945b265e3e4ba1d597c`; targeted Buck evidence green after dev merge.
- #883 CRM: branch `agent/waveB-crm-descriptor-metadata-878`, worktree `/Users/jasonlee/oyatie-worktrees/waveA-crm-marketing-20260625173629`, latest pushed `05a492ca3a8048654fba736d471a9c5024200aa1`; targeted Buck evidence green after dev merge.
- #884 transport: branch `agent/waveB-iac-transport-773-20260626`, worktree `/Users/jasonlee/oyatie-worktrees/waveA-iac-k8s-20260625173629`, latest pushed `1f9dbb94f03761fe5c442df9de3079f75be0839c`; rustfmt fix and targeted Buck evidence green after dev merge.

## WIP preserved
- #879 same-tenant PDP WIP branch: `agent/waveB-pdp-same-tenant-879`, worktree `/Users/jasonlee/oyatie-worktrees/waveA-kms-iam-20260625173629`.
- Uncommitted WIP patch snapshot saved at `/Users/jasonlee/oyatie-worktrees/_wip-preserve/20260626T022802Z/waveB-pdp-same-tenant-879.patch`.
- Modified files at snapshot: `iam/adapters/identity-workload-authz-cedar/src/lib.rs`, `iam/core/identity-workload-domain/src/lib.rs`, `k8s/adapters/tenant-quota-adapter-cedar/src/lib.rs`.

## Lane assignment
1. PR #882 lane: monitor CI for #882, if red inspect logs and fix only branch/worktree above; otherwise report green evidence for leader merge.
2. PR #883 lane: monitor CI for #883, if red inspect logs and fix only branch/worktree above; otherwise report green evidence for leader merge.
3. PR #884 lane: monitor CI for #884, if red inspect logs and fix only branch/worktree above; otherwise report green evidence for leader merge.
4. #879 WIP lane: finish Cedar/PDP DSL same-tenant condition on the existing WIP branch. Preserve patch first, add tests, use Buck2 verification, push PR only when coherent.
5. Backlog/readiness lane: classify remaining open issues (#778/#777/#776/#774/#771) into next disjoint wave after current PRs merge; do not edit files.
