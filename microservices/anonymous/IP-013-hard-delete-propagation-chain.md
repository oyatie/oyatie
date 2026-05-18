---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-shared-substrate
phase: P02-anonymous-foundation
impl_plan_id: IP-013-hard-delete-propagation-chain
status: pending
execution_unit: ChangeSet
owner: axis-anonymous + ops-data
acceptance_lanes: [cargo-test, oya-governance-propagation-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: Hard-delete propagation chain (Postgres → Redis → Meilisearch → audit-chain tombstone)

## Intent

Implement the cross-BC propagation chain ensuring a hard-delete on the post-thread BC propagates to feed-timeline (Redis), search-index (Meilisearch), and audit-chain (tombstone seal) within p99 ≤ 5s. The propagation is the load-bearing surface for the I3 invariant.

## ChangeSet

- Cross-BC test harness in `tests/e2e/hard-delete-propagation.rs`
- Saga-style propagation pattern (no XA; per-step idempotency)
- Tombstone Merkle proof verifier

## Acceptance

- E2E propagation test: delete a post; verify within 5s that:
  - Postgres no longer returns the post
  - Redis feed cache no longer returns the post
  - Meilisearch index no longer returns the post
  - Audit-chain tombstone seal verifiable
- Failure injection: kill propagation worker mid-flight; verify resumption
- Per `runbooks/hard-delete-tombstone-corruption.md` runbook tested
