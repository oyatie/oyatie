# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Intelligence, Cloud, Search, and Ads + Analytics.

Canonical guidance and machine-readable specs live under [`docs/`](docs/) and [`.omc/`](.omc/). Humans typically start here; agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the canonical entry-point pointers into machine-readable artifacts.

## Run

```sh
git fetch github-mirror dev
git worktree add /tmp/oyatie-lane-<slug> -b chore/<slug> github-mirror/dev
buck2 build //:repo-hygiene-automation-check
buck2 build //:kubernetes-native-anti-pattern-check
buck2 build //:buck2-authority-policy-check //:rust-llvm-coverage-runner-contract-check //:rust-llvm-coverage-smoke-check
buck2 build //:repo-hygiene-automation-check //:buck2-authority-policy-check
# Local Cargo commands are advisory only when a dual Cargo+Buck2 setup is intentionally maintained.
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.

## Current CI and dev-lane authority

ADR-0513 is the current CI authority: Oyatie CI is a Rust, Prow-shaped, Kubernetes-native control plane that posts `oya-ci-required` from trusted controller state. GitHub remains the pull-request/publication adapter while the native Sapling-inspired SCM matures; GitHub Actions artifacts are compatibility/shadow checks, not first-class CI direction. Buck2 remains build/test/check authority.

Shared repo surfaces stay thin: root docs, indexes, and registries point to lane-owned shards instead of carrying large mutable content. Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now; no shared contract or shared surface until a later rewrite and rewire of Oyatie products to consume the Cloud IdP.


## Repo hygiene automation

[`specs/repo-hygiene-automation.json`](specs/repo-hygiene-automation.json) is the P00 automation contract for git/worktree, branch/merge, repository publication, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene. The native Sapling-inspired SCM must expose GitHub public/private publication and status adapters; GitHub remains a bridge, not the durable source of truth.

Documentation sprawl rule: shared docs stay pointer-thin, new Markdown defaults to registered/lane-owned or archived, and stale docs older than 3 days are audit candidates before deletion.

Rust purity rule: active CI/governance automation should be Rust + Buck2 + Prow/Kubernetes-native. Python and shell are migration targets unless they are narrowly scoped one-time bootstrap or host-prelude glue with deletion criteria.

## License

This repository is proprietary and all rights are reserved. See [`LICENSE`](LICENSE)
for the repository-default IP posture. Third-party materials and files/components
with explicit license notices remain governed by those notices for those materials
only.
