---
doc_class: ImplementationPlan
template_id: TPL-IP
ip_id: IP-006-tos-policy-audit-chain
parent: ./INDEX.md
milestone: M02
phase: P02-multi-subscription-pool
status: pending approval
purpose: |
  Extend `oya-foundry-policy-kernel` with the `ToSAcknowledgment` record + `PoolingPolicyCheck`
  function that gates pool-membership > 1 on an explicit per-(tenant, provider) ToS-ack.
  Every routing decision emits `EVT-PROVIDER-POOL-ROUTING` to the audit chain with the
  account_id, pool_id, reason, and a reference to the ToS-ack record. Anti-correlation:
  the routing strategy may consider ToS-flagged dimensions (different IPs, distinct OAuth
  identities, time-skew between rotation cycles) without ever exposing raw secret material.
  This IP is what makes multi-subscription pooling legally defensible vs ccproxy-api's
  implicit posture.
grit_claim_symbols:
  - "crates/oya-foundry-policy-kernel/src/lib.rs::ToSAcknowledgment"
  - "crates/oya-foundry-policy-kernel/src/lib.rs::PoolingPolicyCheck"
  - "crates/oya-foundry-policy-kernel/src/lib.rs::TenantPoolingPolicy"
  - "crates/oya-foundry-policy-kernel/src/lib.rs::AntiCorrelationRule"
  - "crates/oya-foundry-policy-kernel/src/audit.rs::emit_pool_routing_event"
agent_prerequisites:
  - .omc/plans/MASTERPLAN.md
  - ./INDEX.md
  - ./IP-001-provider-account-pool-kernel.md
  - docs/AGENTS.md
  - docs/CONSTITUTION.md
  - .omc/standards/security-review.md
  - .omc/specs/foundry-salvage-from-ultragoal-2026-05-12.md
final_shape_compliance: true
dependency_additions:
  - { crate: "cedar-policy 4.4", lts: true, adr_exception: null }
  - { crate: "serde 1.0", lts: true, adr_exception: null }
  - { crate: "time 0.3", lts: true, adr_exception: null }
decision_log: |
  Linus good-taste row: eliminated the implicit-acknowledgment branch — there is no
  "we'll prompt later" path. ToS-ack is either present (a row in the ledger with tenant_id,
  provider, version, accepted_at, accepted_by, evidence_hash) or pool-membership > 1 is
  refused. One predicate; no branching.
authority_chain_declaration: |
  docs/CONSTITUTION.md > rest of docs/ > catalog records > Redirect-class > working drafts.
---

# IP-006-tos-policy-audit-chain: ToS-acknowledgment policy + pool-routing audit emission

## Purpose

Ships the policy + audit primitives that make multi-subscription pooling auditable per
tenant. The kernel exposes `PoolingPolicyCheck(tenant, provider, pool_size) -> Allow | Deny`;
behind it is a `TenantPoolingPolicy` record carrying the operative `ToSAcknowledgment`. Every
`pick_account` decision emitted by IP-001 funnels through `emit_pool_routing_event` so the
audit chain records the full provenance (pool_id, account_id, routing_reason, ToS-ack ref,
trace_context). Operator console renders the audit feed; regulators read directly from the
immutable ledger.

## Symbols to grit-claim

```
crates/oya-foundry-policy-kernel/src/lib.rs::ToSAcknowledgment
crates/oya-foundry-policy-kernel/src/lib.rs::TenantPoolingPolicy
crates/oya-foundry-policy-kernel/src/lib.rs::PoolingPolicyCheck
crates/oya-foundry-policy-kernel/src/lib.rs::AntiCorrelationRule
crates/oya-foundry-policy-kernel/src/lib.rs::PoolingPolicyVerdict
crates/oya-foundry-policy-kernel/src/audit.rs::emit_pool_routing_event
crates/oya-foundry-policy-kernel/cedar/foundry.pooling.cedar::ToSAckRequired
```

### Shape

```
struct ToSAcknowledgment {
    tenant_id: TenantId,                  // data_class = TenantScoped
    provider: ProviderFamily,             // data_class = Internal
    upstream_tos_version: String,         // data_class = Internal
    accepted_at: OffsetDateTime,          // data_class = Internal
    accepted_by: ActorId,                 // data_class = Internal (no PII in the kernel)
    evidence_hash: Sha256Hash,            // hash of signed acceptance bundle
    revoked_at: Option<OffsetDateTime>,
}

struct TenantPoolingPolicy {
    tenant_id: TenantId,
    max_pool_size_per_provider: u8,
    anti_correlation_rules: Vec<AntiCorrelationRule>,
    tos_acks: BTreeMap<ProviderFamily, ToSAcknowledgment>,
}

enum AntiCorrelationRule {
    DistinctSourceIp,
    DistinctOAuthIdentity,
    MinRotationInterval(Duration),
    BlocklistedDualUse,                   // ToS-flagged anti-pattern; forbidden
}

fn PoolingPolicyCheck(
    policy: &TenantPoolingPolicy,
    provider: ProviderFamily,
    pool_size: u8,
) -> PoolingPolicyVerdict;
```

### Audit event

```
EVT-PROVIDER-POOL-ROUTING
  pool_id, tenant_id, provider, chosen_account_id (redacted to last 4),
  routing_reason, fallback_chain (redacted), tos_ack_ref, trace_id,
  decided_at, autonomy_tier
```

## Agent prerequisites

<!-- agent-instructions:start -->
Before `grit claim`, the agent **MUST**:
1. `icm recall-context "P02 tos acknowledgment pool routing audit cedar" --limit 5`.
2. Confirm IP-001 merged.
3. Read `.omc/standards/security-review.md §5` (data-class) and `§6` (autonomy ceiling).
4. Read `docs/AGENTS.md §Pre-flight checklist`.
5. Read parent INDEX `./INDEX.md` and the parity matrix `./ccproxy-api-parity-matrix.md`.
6. Confirm symbols unclaimed.
<!-- agent-instructions:end -->

**Human path:** tenant onboarding wizard surfaces a "Multi-subscription pooling acknowledgment" panel that links to the upstream provider's ToS; ack signature creates the `ToSAcknowledgment` record; without that record, `oya foundry pool add-member` refuses with `PoolingPolicyVerdict::Deny(ToSAckRequired)`.

## Acceptance test commands

```
$ cargo nextest run -p oya-foundry-policy-kernel --all-features                  # expect: PASS, 0 failures
$ cargo clippy -p oya-foundry-policy-kernel -- -D warnings                       # expect: PASS, 0 warnings
$ cargo deny check                                                               # expect: PASS
$ oya gate validate oya-foundry-fitness-tos-acknowledgment                       # expect: PASS
$ oya gate validate oya-foundry-fitness-pool-routing-honor                       # expect: PASS
$ oya-tooling-agent-read run-evidence "scripts/smoke/audit-chain-pool-routing.sh" # expect: 100 routing decisions → 100 EVT-PROVIDER-POOL-ROUTING entries in audit ledger
```

Property tests required:
- `PoolingPolicyCheck(no ack, pool_size = 2) -> Deny(ToSAckRequired)`.
- `PoolingPolicyCheck(revoked ack, pool_size = 2) -> Deny(ToSAckRevoked)`.
- `PoolingPolicyCheck(valid ack, pool_size = max_pool_size_per_provider) -> Allow`.
- `PoolingPolicyCheck(valid ack, pool_size > max) -> Deny(PoolSizeExceeded)`.
- Cedar policy file `foundry.pooling.cedar` evaluates identically to the Rust check (cross-check property test).

## Done criteria

- [ ] All `grit_claim_symbols` claimed → work → `grit done`.
- [ ] D1-D18 done-definition walked.
- [ ] All acceptance commands PASS.
- [ ] `cargo deny check` + cargo vet for cedar-policy.
- [ ] `icm store -t context-foundry` emitted.
- [ ] Audit-chain `EVT-TOS-POLICY-ACTIVE` emitted at first PoolingPolicyCheck merge.
- [ ] Council-Privacy reviewer-agent verdict: APPROVE.
- [ ] Cedar policy unit tests green; runtime + Cedar cross-check property test green.
- [ ] Operator-console onboarding wizard surfaces the ack panel (UI hand-off to `oya-foundry-console-tos-wizard`).

## Rollback procedure

1. Identify rollback boundary: feature flag `foundry.pooling.tos_ack_required = false` is **NOT permitted** (ToS-ack is a regulatory floor, not a feature toggle); rollback means reverting the entire P02 phase deployment to single-account-per-provider mode.
2. Execute: `oya foundry pool quarantine --pool-size-floor 1`; existing multi-member pools enter Degraded state per the P00 state machine; audit emits `EVT-POOL-QUARANTINED`.
3. Verify: no pool with >1 member serves traffic; `oya foundry pool list` shows all quarantined.
4. Postmortem trigger: Sev-1 if ToS-ack absence was the cause of a customer-impacting incident (regulatory exposure); Sev-2 otherwise.

## Next IP pointer

End of phase P02. Next phase: `phases/P03-gates-validators-evidence/INDEX.md`.

## Icm-store-payload

```
icm store \
  -t context-foundry \
  -c "IP-006-tos-policy-audit-chain merged at <git-sha>; grit symbols released: ToSAcknowledgment, TenantPoolingPolicy, PoolingPolicyCheck, AntiCorrelationRule, emit_pool_routing_event; acceptance lanes green: -tos-acknowledgment (BLOCKER), -pool-routing-honor (BLOCKER); phase P02 complete; next phase: P03-gates-validators-evidence" \
  -i high \
  -k "M02,P02,IP-006,tos-policy,audit-chain,phase-complete,ccproxy-parity-gap-closure"
```

## Decision log (Linus good-taste row)

Eliminated the "implicit acceptance" / "we'll prompt later" branch — ToS-ack is binary
(present or absent). One predicate; one verdict; no branching.

## Cross-references

- Master Plan: `.omc/plans/MASTERPLAN.md` §2 Directives 2, 6, 9.
- Phase INDEX: `./INDEX.md`.
- ADR-0053 — sanctioned primitives.
- ADR-0043 — secrets management (audit emission must not leak secret material).
- `.omc/standards/security-review.md §5, §6, §7` — data-class, autonomy ceiling, secret handling.
- Progressive-delivery + branch-pipeline composers.
- ccproxy-api gap closure: ccproxy-api documents subscription pooling but ships no explicit ToS-ack ledger; this IP closes that gap.
- Anthropic Acceptable Use / Usage Policies: https://www.anthropic.com/legal/aup.
- OpenAI Terms of Use: https://openai.com/policies/row-terms-of-use/.
- Google AI Studio / Gemini Terms: https://ai.google.dev/gemini-api/terms.
