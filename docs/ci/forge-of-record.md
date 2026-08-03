# SCM of Record — CI Gating (ADR-0363)

GitHub (interim) at `github.com/jason931225/oyatie` is the **gating SCM of
record** for all PR merges to `dev`, per D-CLOUD-SCM. It is the merge authority:
required status contexts must be green before a PR can merge.

## How gating works

1. `.github/workflows/oya-ci-required.yml` fans out the current cloud-ci,
   Buck2, generated-output, app-shell, and PR reviewer-evidence lanes, then fans
   them in to the single job named `oya-ci-required`.
2. The GitHub branch-protection rule for `dev` requires the one context in
   `infra/branch-protection/dev.json` to be green before a PR can merge.
3. `.github/branch-protection.yaml` is the canonical branch-protection shadow
   record. It intentionally keeps GitHub Review API approval count at 0/null;
   reviewer-agent approval is enforced through the required context path, not
   through GitHub's `reviewDecision` field.

## Current required status context

| Context | Producer | Description |
|---|---|---|
| `oya-ci-required` | GitHub Actions workflow fan-in | Green iff every constituent workflow lane succeeds, including the PR-body reviewer-evidence lane on `pull_request` events. |

## Reviewer evidence enforcement

GitHub human approving reviews are not live merge authority for `dev`, and an
empty GitHub `reviewDecision` is expected while `required_pull_request_reviews`
remains null. Merge-ready pull requests still require reviewer-agent evidence:
the `pr-reviewer-evidence` lane inside `oya-ci-required` extracts
`pull_request.body` and runs:

```bash
cargo run --locked -p oya-dev-cli -- gate validate pr-traceability \
  --pr-body "$PR_BODY_PATH" \
  --require-code-review
```

That validator fails closed when `## Code Review` is missing or when the section
lacks field-style `Reviewer agent`, `Verdict: APPROVE`, `Resolved items`, and
`Deferred items` evidence. Kanban-only approval is not sufficient for protected
merge readiness; local hook output is advisory unless the same policy is green
inside the protected `oya-ci-required` context.

## Phase-2 (pending)

`oya-pr-review` may be added back as a separate required context once the
reviewer-agent HTTP endpoint ships as a trusted producer (currently HTTP 501).
Until then, reviewer evidence remains enforced by `pr-reviewer-evidence` inside
`oya-ci-required` to avoid deadlocking every PR on a non-live producer.

## References

- ADR-0363: GitHub (interim) as gating SCM of record
- `infra/branch-protection/dev.json`: machine-readable required contexts
- `.github/branch-protection.yaml`: canonical branch-protection record
