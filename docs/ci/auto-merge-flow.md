# Auto-Merge Flow — Forgejo gate-driven merge

This note describes how PRs auto-merge into `dev` on the Forgejo forge of record
(`forgejo.oya-forge.svc.cluster.local:3000`, repo `oya-admin/oyatie`) once the CI
gate goes green. It complements `docs/ci/forge-of-record.md` (which defines the
gating model).

## Two separate steps

Auto-merge is two distinct mechanisms — do not conflate them:

1. **Arm the gate (one-time, repo-level).** Branch protection on `dev` must
   require the CI gate context. `scripts/ci/arm-auto-merge.sh` enforces this
   idempotently:
   - `enable_status_check = true`
   - `status_check_contexts = ["oya-ci-gate"]` — the context the Jenkins
     `oya-ci-gate` pipeline posts to the Forgejo Commit Status API.
   - Re-running converges the rule (GET, then POST if absent / PATCH if present).

2. **Enable auto-merge (per-PR).** The PR author or a maintainer enables
   **"Auto Merge (when checks pass)"** on the individual PR. Forgejo then merges
   that PR automatically the moment all branch-protection-required checks
   (`oya-ci-gate`) report success.

## End-to-end flow

```
agent opens PR -> dev
        |
        v
ci-webhook-gateway fires Jenkins oya-ci-gate (ADR-0374 / ADR-0380)
        |
        v
oya-ci-gate posts Forgejo commit-status "oya-ci-gate" = pending -> success/failure
        |
        v
branch protection on dev requires status_check_contexts=["oya-ci-gate"]
        |
        v
PR has "Auto Merge (when checks pass)" enabled
        |
        +-- oya-ci-gate = success --> Forgejo merges automatically (squash, delete branch)
        +-- oya-ci-gate = failure --> merge blocked; auto-merge stays armed, retries on next green
```

## API calls

Arm the gate (what `scripts/ci/arm-auto-merge.sh` issues):

- `GET  /api/v1/repos/oya-admin/oyatie/branch_protections/dev` — probe.
- `POST /api/v1/repos/oya-admin/oyatie/branch_protections` — create when GET is 404.
- `PATCH /api/v1/repos/oya-admin/oyatie/branch_protections/dev` — update when GET is 200.

Create/update payload:

```json
{
  "branch_name": "dev",
  "enable_status_check": true,
  "status_check_contexts": ["oya-ci-gate"]
}
```

Enable auto-merge on a PR (per-PR, reference only — not run by the script):

- `POST /api/v1/repos/oya-admin/oyatie/pulls/{index}/merge`

```json
{
  "Do": "squash",
  "merge_when_checks_succeed": true,
  "delete_branch_after_merge": true
}
```

`Do` selects the merge style (`merge` | `rebase` | `rebase-merge` | `squash` |
`manually-merged`). `merge_when_checks_succeed` schedules the merge for when the
required checks pass; if they are already green the PR merges immediately. The
field name is `merge_when_checks_succeed` on Forgejo's `MergePullRequestOption`;
some older Gitea-derived builds expose `MergeWhenChecksSucceed` — confirm against
the deployed Forgejo's `/api/swagger` before scripting per-PR auto-merge.

## Auth

All calls send `Authorization: token ${FORGEJO_TOKEN}` (a Forgejo access token
with repo administration scope). The token is read from the environment and is
never echoed by the script or logged.

## References

- `scripts/ci/arm-auto-merge.sh` — idempotent gate-arming script
- `docs/ci/forge-of-record.md` — Forgejo as gating forge of record (ADR-0363)
- ADR-0374 — webhook-driven CI invocation
- ADR-0380 — CI loop closure on the Talos Jenkins farm (`oya-ci-gate` pipeline)
