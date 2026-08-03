# G004 baseline decision — 20260627T193540Z

Decision: source-write fan-out may proceed from fresh `origin/dev` worktrees.

Evidence:
- `origin/dev`: `ba4f6347905ac6c31cde941d5383741ddd0318a1`.
- Branch protection required status checks: `oya-ci-required` only (GitHub app id 15368), strict=false.
- Commit check-runs on `ba4f6347905ac6c31cde941d5383741ddd0318a1`: pending=0, failures=0, one completed success run `oya-check-substrate-dependency-dag-acyclicity`.
- Commit check-suites on `ba4f6347905ac6c31cde941d5383741ddd0318a1`: failures=0. Non-required suites for Cursor/Claude/GitHub Actions are queued/pending with `latest_check_runs_count=0`; they are not merge authority and have no executable failure evidence.

Guardrails for next fan-out:
- Fresh isolated worktree branch per lane/team worker from `origin/dev`.
- Max two writers plus one read-only reviewer.
- One writer per app subtree; no shared producer/workflow/root policy/generated face/.omx ultragoal edits.
- No generated JSON hand edits; materialized gate runs only in disposable/cleaned worktrees.
- Rust + Buck2 verification is authoritative; Cargo is not merge authority.
