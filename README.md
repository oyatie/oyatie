# Oyatie

Oyatie is a cohesive ecosystem-as-a-service across SaaS, Workspace, Vertical, Foundry, Cloud, Search, and Ads + Analytics.

Canonical guidance and machine-readable specs live under [`docs/`](docs/) and [`.omc/`](.omc/). Humans typically start here; agents read [`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) for the canonical entry-point pointers into machine-readable artifacts.

## Run

```sh
git fetch github-mirror dev
git worktree add /tmp/oyatie-lane-<slug> -b chore/<slug> github-mirror/dev
python3 scripts/ci/assert-repo-hygiene-automation.py --json
buck2 build //:repo-hygiene-automation-check
buck2 build //:github-lane-unlocker-bridge-check //:buck2-authority-policy-check //:repo-hygiene-automation-check
infra/ci/buck2-affected-gate.sh github-mirror/dev HEAD
# Local Cargo commands are advisory only when a dual Cargo+Buck2 setup is intentionally maintained.
```

The current runnable slice is W-Foundation: tenancy, identity, data-use boundary, cell routing, audit chain, capability policy, regional packs, Object Graph, and idempotent outbox.

## Current dev-lane bridge

ADR-0516 records the temporary GitHub/GitHub Actions lane-unlocker for highly parallel product, infra, and cloud work; dev is gated by `github-lane-unlocker-required` during the bridge. Buck2 remains build/test/check authority; the native destination is cloud native, Kubernetes-native, and hyperscaler native.

Shared repo surfaces stay thin: root docs, indexes, and registries point to lane-owned shards instead of carrying large mutable content. Cloud auth/shared substrate and Oyatie product auth/shared substrate are decoupled now; no shared contract or shared surface until a later rewrite and rewire of Oyatie products to consume the Cloud IdP.


## Repo hygiene automation

[`specs/repo-hygiene-automation.json`](specs/repo-hygiene-automation.json) is the P00 automation contract for git/worktree, branch/merge, repository publication, disk/workspace, Kubernetes workload, and documentation-sprawl hygiene. The native Sapling-inspired SCM must expose GitHub public/private publication and status adapters; GitHub remains a bridge, not the durable source of truth.

Documentation sprawl rule: shared docs stay pointer-thin, new Markdown defaults to registered/lane-owned or archived, and stale docs older than 3 days are audit candidates before deletion.

## License

This repository is proprietary and all rights are reserved. See [`LICENSE`](LICENSE)
for the repository-default IP posture. Third-party materials and files/components
with explicit license notices remain governed by those notices for those materials
only.
