---
purpose: Ship the ProviderAccountPool pure-kernel crate that coordinates rotation across multiple ProviderAccount records (already owned by P00 account-auth).
---

---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-001-provider-account-pool-kernel
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: pending approval
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
changeset_split_rule: split-before-execution-if-unrelated-lock-scope-or-deployable
purpose: |
  Ship the ProviderAccountPool pure-kernel crate that coordinates rotation across multiple
  ProviderAccount records (already owned by P00 account-auth). Pool is a thin coordination
  layer — it stores no account-level state, only membership + routing strategy + verdict.
  This is the Rust counterpart to ccproxy-api `credential_balancer/manager.py` rotation +
  health logic, refactored to pure value objects so the runtime stays adapter-agnostic
  (MASTERPLAN Directive 4) and final-shape from day one (Directive 3).
grit_claim_symbols:
  - "crates/oya-foundry-provider-pool-kernel/src/lib.rs::ProviderAccountPool"
  - "crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolRoutingStrategy"
  - "crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolRoutingDecision"
  - "crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolMembershipChange"
  - "crates/oya-foundry-provider-pool-kernel/src/lib.rs::pick_account"
agent_prerequisites:
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md
  - docs/AGENTS.md
  - /specs/cross-cutting/decision-principles.json
  - /specs/cross-cutting/forbidden-operations.json
  - .omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md
  - .omc/standards/dependency-policy.md
final_shape_compliance: true
dependency_additions:
  - { crate: "serde 1.0", lts: true, adr_exception: null }
  - { crate: "thiserror 2.0", lts: true, adr_exception: null }
  - { crate: "time 0.3", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the special-case branch "single-account == no pool"
  by representing single-account as a pool of size 1 with `RoundRobin`. The pick_account
  function therefore has no `if members.len() == 1` branch — the data shape removes it.
authority_chain_declaration: |
  /specs/cross-cutting/decision-principles.json + /specs/cross-cutting/forbidden-operations.json > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-001-provider-account-pool-kernel: ProviderAccountPool kernel + value types

## Purpose

Ships the pure-value-type `ProviderAccountPool` kernel + the deterministic `pick_account`
decision function as the foundation of P02. Pool kernel sits *above* the P00 account state
machine — it never mutates ProviderAccount; it only reads usage snapshots, applies a routing
strategy, and emits a `PoolRoutingDecision { account_id, reason, fallback_chain }`. This
isolation is what lets the adapter crates (IP-002 Anthropic-compat, IP-003 OpenAI-compat)
share one rotation kernel without provider-specific branching (Master Plan Directive 4 +
ccproxy-api `credential_balancer/manager.py` lesson applied with stricter typing).

## Symbols to grit-claim

```
crates/oya-foundry-provider-pool-kernel/src/lib.rs::ProviderAccountPool
crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolRoutingStrategy
crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolRoutingDecision
crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolMembershipChange
crates/oya-foundry-provider-pool-kernel/src/lib.rs::pick_account
crates/oya-foundry-provider-pool-kernel/src/lib.rs::PoolRoutingReason
crates/oya-foundry-provider-pool-kernel/src/error.rs::PoolError
```

Pure value types only — no async, no I/O, no allocator games. All fields carry
`#[oyatie(data_class = "<class>")]` annotations per security-review.md §5.

### Shape

```
struct ProviderAccountPool {
    id: PoolId,                                   // data_class = Internal
    provider: ProviderFamily,                     // data_class = Internal
    tier: ProviderTier,                           // data_class = Internal
    tenant_id: TenantId,                          // data_class = TenantScoped
    members: BTreeSet<ProviderAccountId>,         // data_class = TenantScoped
    routing_strategy: PoolRoutingStrategy,        // data_class = Internal
    anti_correlation_window: Duration,            // data_class = Internal
    tos_acknowledgment_ref: Option<TosAckId>,     // populated by IP-006
}

enum PoolRoutingStrategy { RoundRobin, LeastUsed, LeastLatency, LeastRemaining, Sticky(SessionId) }
enum PoolRoutingReason { Healthy, FailoverFrom(ProviderAccountId), Sticky, QuotaPreserve, LeastUsedTieBreak }

struct PoolRoutingDecision {
    account_id: ProviderAccountId,
    reason: PoolRoutingReason,
    fallback_chain: Vec<ProviderAccountId>,       // ordered, capped at members.len()-1
    decided_at: OffsetDateTime,
}

fn pick_account(
    pool: &ProviderAccountPool,
    request: &RequestMetadata,
    usage: &UsageSnapshotMap,
    health: &AccountHealthMap,
    now: OffsetDateTime,
) -> Result<PoolRoutingDecision, PoolError>;
```

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 provider-account-pool-kernel ccproxy-api" --limit 5` and read.
2. Read `.omc/plans/MASTERPLAN.md §2` Directives 3, 4, 7 (final-shape, provider-agnostic, Linus).
3. Read `./INDEX.md` and the ccproxy-api parity matrix at `./ccproxy-api-parity-matrix.md`.
4. Read `docs/AGENTS.md §Pre-flight checklist` and `/specs/cross-cutting/decision-principles.json` (DP-01..DP-10).
5. Confirm no other agent has claimed `crates/oya-foundry-provider-pool-kernel/src/lib.rs::*` via `oya-tooling-agent-read grit-status crates/oya-foundry-provider-pool-kernel`.
6. Read `.omc/scratch/foundry-salvage-from-ultragoal-2026-05-12.md §B` to inherit the P00 state machine; pool kernel MUST NOT duplicate ProviderAccount-level state.
<!-- agent-instructions:end -->

**Human path:** read the same files; run `oya gate validate plan-hierarchy --ip IP-001-provider-account-pool-kernel` to confirm parent pointers + frontmatter.

## Acceptance test commands

```
$ cargo nextest run -p oya-foundry-provider-pool-kernel --all-features            # expect: PASS, 0 failures
$ cargo clippy -p oya-foundry-provider-pool-kernel --all-features -- -D warnings  # expect: PASS, 0 warnings
$ cargo-semver-checks check-release -p oya-foundry-provider-pool-kernel           # expect: PASS (kernel is public API)
$ cargo deny check                                                                # expect: PASS
$ oya gate validate oya-foundry-fitness-provider-coupling                         # expect: PASS (no upstream HTTP types here)
$ oya gate validate oya-foundry-fitness-no-placeholder                            # expect: PASS
$ oya-tooling-agent-read run-evidence "cargo test -p oya-foundry-provider-pool-kernel -- --nocapture" # expect: 25+ pool-routing property tests green
```

Property tests required (proptest):
- `pick_account` deterministic given identical inputs.
- `RoundRobin` over N members produces every member within N calls.
- `LeastUsed` selects strictly-lowest-usage member when unique; tie-breaks via `BTreeSet` order.
- `LeastRemaining` never picks an account with `reserve_remaining_pct < threshold`.
- All-unhealthy → `PoolError::NoHealthyMembers` with fallback_chain empty.

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done` (no orphan claims).
- [ ] `docs/AGENTS.md §Done-Definition checklist` D1-D18 walked.
- [ ] All acceptance commands PASS; outputs captured in PR `## Verification`.
- [ ] Dependency additions cleared `cargo deny check` (serde, thiserror, time — all LTS).
- [ ] `icm store -t context-foundry -c "<payload>" -i high` emitted (§Icm-store-payload).
- [ ] Audit-chain `EVT-PROVIDER-POOL-KERNEL-SHIPPED` emitted; ID pasted in PR `## Evidence`.
- [ ] Phase INDEX `§Implementation Plans` row updated to `merged`.
- [ ] `data_class` annotation present on every public field; lane `oya-foundry-fitness-data-class` PASS.

## Rollback procedure

1. Identify rollback boundary: git revert of the new crate scaffold + workspace member removal.
2. Execute: `grit revert <claim-id>` to release symbols; remove `members = [..., "crates/oya-foundry-provider-pool-kernel"]` row from root `Cargo.toml`; revert PR.
3. Verify: `cargo check --workspace` green; no downstream crate references `oya-foundry-provider-pool-kernel`.
4. Postmortem trigger threshold: Sev-3 (kernel-only; no live traffic yet).

## Next IP pointer

`IP-002-anthropic-compat-adapter.md` (consumes `pick_account` from this kernel).

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-001-provider-account-pool-kernel merged at <git-sha>; grit symbols released: ProviderAccountPool, pick_account, PoolRoutingStrategy, PoolRoutingDecision, PoolMembershipChange; acceptance lanes green: oya-foundry-fitness-provider-coupling, -no-placeholder, -data-class; next IP: IP-002-anthropic-compat-adapter" \
  -i high \
  -k "M02,P02,IP-001,pool-kernel,ccproxy-parity"
```

## Decision log (Linus good-taste row)

Eliminated the `if members.len() == 1` branch by representing the single-account case as a
pool-of-1 with `RoundRobin`. The data shape removes the special case from `pick_account`.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 3, 4, 7, 8.
- Phase INDEX: `./INDEX.md`.
- Parent contract: `oyatie/docs/products/foundry/PHASE-00-SPEC.md` (when lifted) — ProviderAccount state machine.
- ADR-0053 — sanctioned primitives (grit-claim discipline).
- ADR-0054 — grit-scaffold-claim pattern (new-crate scaffold is the first claimable artifact).
- Progressive-delivery composer output: `.omc/advanced-cicd/progressive-delivery/playbook-foundry.md` (kernel ships dark-launch behind feature flag).
- Branch-pipeline composer output for promotion gates.
- ccproxy-api source of inspiration: https://github.com/CaddyGlow/ccproxy-api/tree/main/ccproxy/plugins/credential_balancer (Python `manager.py` rotation; this IP is the typed Rust equivalent).
