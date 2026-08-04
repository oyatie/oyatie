# G026 importer census — 2026-08-02

State: `PLANNING_ONLY_NOT_ACTIVATED`
Authority: `git grep` against **origin/dev** only (canonical checkout not trusted).

## oya-adapter-substitution-test-app

Live non-self references on origin/dev:
- `intelligence/adapters/account-adapter-inmemory/src/lib.rs:9` — documents this tools app as the substitution test consumer
- `ci/facade/module-membership/capability-membership-policy.json:473` — tools membership allowlist
- `specs/capability-registry.json:625` — tools absorb list
- `specs/crate-naming-audit.json:161` — naming audit proposed name
- docs audit inventory only (historical)

Disposition update: **MOVE** remains correct; preferred destination is **colocated under intelligence test surface** (adapter it tests) rather than generic tools. If colocation is blocked by face rules, fallback `ci/facade/adapter-substitution-test`. Not DELETE.

## oya-architecture-graph-generator-app

Live non-self / product references:
- `.github/workflows/docs-graph-drift.yml` — builds + tests the generator on path filters (lines 6,27,34,88-91)
- `ci/facade/generated-artifact-freshness/src/lib.rs:80` — hard-coded generator target label
- `ci/facade/generated-artifact-policy/src/lib.rs:2018` — policy fixture/target
- `registry/generated-artifact-control-plane.json:344` — `generator_target`
- membership/catalog policy rows

Disposition update: **MOVE** to `ci/facade/architecture-graph-generator` (or keep path until generator-control-plane cutover). **Not DELETE**. Live CI + freshness gate consumer. KEEP_PENDING only if destination face still ambiguous after control-plane rewrite plan; default class is delivery-fabric/ci.

## Non-claims
- Census is not a move plan.
- No code moved.
- Retirement scan for bot-autofix / tooling-agent-read / lane-supervisor is a separate background explore lane.
