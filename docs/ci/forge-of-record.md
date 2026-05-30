# Forge of Record — CI Gating (ADR-0363)

Forgejo (self-hosted at `forgejo.oya-forge.svc.cluster.local`) is the **gating forge
of record** for all PR merges to `dev`. GitHub (`github.com/jason931225/oyatie`) is the
**bootstrap mirror** only — it holds a read-only copy of the repository and is not the
merge authority.

## How gating works

1. Jenkins pipelines (`oyaCiLane.groovy`) post Forgejo Commit Status API entries for
   each required context via `postForgeStatus()` (see
   `infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`).
2. The Forgejo branch-protection rule for `dev` requires all contexts in
   `infra/branch-protection/dev.json` to be green before a PR can merge.
3. GitHub branch-protection (`.github/branch-protection.yaml`) mirrors the same
   required-context list for consistency but is not the enforcement point.

## Phase-1 required status contexts

| Context | Producer | Description |
|---|---|---|
| `cargo-fmt` | `oyaCiLane` | `cargo fmt --check` |
| `cargo-check` | `oyaCiLane` | `cargo check --all-targets` |
| `cargo-clippy` | `oyaCiLane` | `cargo clippy -- -D warnings` |
| `cargo-nextest` | `oyaCiLane` | nextest test run |
| `cargo-deny` | `oyaCiLane` | OSI license + advisory + bans gate |
| `oya-verify` | `oyaCiLane` | Rolled-up `./bin/oya verify --affected` verdict |

## Phase-2 (pending)

`oya-pr-review` will be added back as a required context once the reviewer-agent HTTP
endpoint ships (currently returns HTTP 501). It was removed from the Phase-1 required
set to avoid deadlocking every PR. See `infra/branch-protection/dev.json` for the
tracking note.

## References

- ADR-0363: Forgejo as gating forge of record
- `infra/branch-protection/dev.json`: machine-readable required contexts
- `.github/branch-protection.yaml`: canonical branch-protection record
- `infra/ci/jenkins/reported-status-contexts.json`: all contexts posted by Jenkins
