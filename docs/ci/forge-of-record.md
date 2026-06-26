# SCM of Record — CI Gating (ADR-0363)

GitHub (interim) at `github.com/jason931225/oyatie` is the **gating SCM of
record** for all PR merges to `dev`, per D-CLOUD-SCM. Merge readiness is the
protected `oya-ci-required` status context; local Cargo/`oya verify` output is
advisory bridge evidence only and is never protected-branch authority.

## How gating works

1. The transitional GitHub Actions workflow `.github/workflows/oya-ci-required.yml`
   posts the single `oya-ci-required` verdict today; the owned oya-ci controller
   is the cloud-native successor producer for the same context.
2. The GitHub branch-protection rule for `dev` requires all contexts in
   `infra/branch-protection/dev.json` to be green before a PR can merge.
3. `.github/branch-protection.yaml` is the canonical branch-protection record.

## Phase-1 required status contexts

| Context | Producer | Description |
|---|---|---|
| `oya-ci-required` | GitHub Actions transition; owned oya-ci/cloud-ci successor | Single fan-in status over Buck2/cloud-ci gate jobs. |

## Phase-2 (pending)

`oya-pr-review` will be added back as a required context once the reviewer-agent HTTP
endpoint ships (currently returns HTTP 501). It was removed from the Phase-1 required
set to avoid deadlocking every PR. See `infra/branch-protection/dev.json` for the
tracking note.

## References

- ADR-0363: GitHub (interim) as gating SCM of record
- `infra/branch-protection/dev.json`: machine-readable required contexts
- `.github/branch-protection.yaml`: canonical branch-protection record
