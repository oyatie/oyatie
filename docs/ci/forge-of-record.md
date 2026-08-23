# SCM of Record — CI Gating (ADR-0363)

GitHub (interim) at `github.com/jason931225/oyatie` is the **gating SCM of
record** for all PR merges to `dev`, per D-CLOUD-SCM. It is the merge authority:
required status contexts must be green before a PR can merge.

## How gating works

1. The ci controller posts GitHub Commit Status API entries for the
   `presubmit` context (crier pattern; see
   `oya/ci-controller/crates/ci-controller-app`).
2. The GitHub branch-protection rule for `dev` requires all contexts in
   `infra/branch-protection/dev.json` to be green before a PR can merge.
3. `.github/branch-protection.yaml` is the canonical branch-protection record.

## Phase-1 required status contexts

| Context | Producer | Description |
|---|---|---|
| `cargo-fmt` | `oyaCiLane` | `cargo fmt --check` |
| `cargo-check` | `oyaCiLane` | `cargo check --all-targets` |
| `cargo-clippy` | `oyaCiLane` | `cargo clippy -- -D warnings` |
| `cargo-nextest` | `oyaCiLane` | nextest test run |
| `cargo-deny` | `oyaCiLane` | OSI license + advisory + bans gate |
| `verify` | `oyaCiLane` | Rolled-up `./bin/oya verify --affected` verdict |

## Phase-2 (pending)

`pr-review` will be added back as a required context once the reviewer-agent HTTP
endpoint ships (currently returns HTTP 501). It was removed from the Phase-1 required
set to avoid deadlocking every PR. See `infra/branch-protection/dev.json` for the
tracking note.

## References

- ADR-0363: GitHub (interim) as gating SCM of record
- `infra/branch-protection/dev.json`: machine-readable required contexts
- `.github/branch-protection.yaml`: canonical branch-protection record
