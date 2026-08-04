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

## Throughput and concurrency control

- `oya-ci-required.yml` is intentionally fan-out / fan-in and now supports configurable
  runner placement:
  - Set repository/org variable `OYA_CI_RUNNER_LABELS` to a JSON array of labels for all
    CI jobs (for example `["ubuntu-latest"]` locally, or `["self-hosted","linux","talos"]`
    for the Talos worker pool).
  - Set repository/org variable `OYA_CI_MAX_PARALLEL` (JSON number) to control gate matrix
    parallelism (default: `8`).
- If you observe only one active runner, that usually reflects available runner capacity or
  label matching (not a workflow defect), not necessarily fan-in/fan-out logic.
- If a job is limited by available CPU/memory in a single lane, split by worker class first
  (more runners), then increase `OYA_CI_MAX_PARALLEL` only after cache / runner saturation
  signals are healthy.
- For sustained high throughput, run Buck2 and cargo caches as designed:
  - `.buck2-ci` cache key is based on `infra/ci/install-buck2.sh`.
  - cargo registry/git caches are now shared per lane family to reduce repeated downloads.

## Self-hosted runner path (Talos)

You can switch CI jobs to Talos-hosted runners by setting `OYA_CI_RUNNER_LABELS` above to
your Talos runner labels, without changing lane semantics. This keeps `oya-ci-required` as the
single required context and preserves the same fan-in contract.

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

## CI caching + BUCK2 RE note

The `buck2` lane supports a cache-reproducibility optimization path:

- `infra/ci/buck2-affected-gate.sh` provides an advisory affected-target run.
- `infra/ci/warm-buck2-cache.sh` optionally writes `.buckconfig.local` when
  `vars.OYA_CI_RE_CACHE_MODE` is set to `ro` or `rw`.
- `toolchains/cache/` contains the cache-only execution platform that maps RE
  cache knobs from `[oya_cache]`.

The repository now defaults CI to cache-only CAS (`mode=ro`) for `buck2`:
read/write uploads remain off by default until `nativelink-cas` behavior and tenancy
isolation evidence is green. The `buck2` required lane remains stable by default;
override `vars.OYA_CI_RE_CACHE_MODE` to `off` for emergency rollback, or `rw` once
upload policy is approved.
