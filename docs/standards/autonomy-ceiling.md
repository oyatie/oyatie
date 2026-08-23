---
purpose: Cross-cutting autonomy-ceiling standard. Defines the T1 / T2 / T3 / T4 capability tiers, the Cedar-policy binding per capability, the per-capability autonomy record, and the explicit prohibition of config-flag uplift.
doc_status: published
---

---
doc_class: Standard
shape: ~
length_cap: 250
authority_tier: 2
status: Accepted
date: 2026-05-12
purpose: |
  Cross-cutting autonomy-ceiling standard. Defines the T1 / T2 / T3 / T4 capability
  tiers, the Cedar-policy binding per capability, the per-capability autonomy
  record, and the explicit prohibition of config-flag uplift. Implements
  `forbidden-operations.json` FO-10 and the AGENTS.md §Pre-flight checklist
  Item 4 ("Confirm autonomy ceiling").
canonical_authority: /specs/decision-principles.json + /specs/forbidden-operations.json
planned_enforcement_ref: governance-autonomy-ceiling
enforcement_status:
  governance-autonomy-ceiling: F-PENDING-AUTONOMY-CEILING (crate missing; tracked in registry/stub-audit/2026-05-17/missing-fitness-crates.json)
  governance-capability-publish: F-PENDING-CAPABILITY-PUBLISH (crate missing)
meta_policy: ADR-0133 (chained-enforcement planning contract, pending)
companion_docs:
  - docs/standards/security-review.md
  - docs/standards/data-class.md
  - docs/standards/observability.md
  - docs/AGENTS.md
related_adrs:
  - ADR-0053
  - ADR-0052
  - ADR-0054
---

# Autonomy Ceiling

## Doctrinal authority — [decision-principles.json](../../specs/decision-principles.json) + [forbidden-operations.json](../../specs/forbidden-operations.json)

Per [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-10:

> No autonomy-tier uplift without policy + runtime gate. Capability
> bindings declare T1 / T2 / T3 / T4; uplift requires a Cedar policy and
> a runtime check, not a config flag.

This standard names the tiers, the binding contract, the policy shape,
and the runtime gate.

## 1. The tier taxonomy

| Tier | Autonomy | Examples | Out-of-band approval? |
|---|---|---|---|
| **T1** | Read-only, no side effects | RAG retrieval, fact lookup, semantic search | NO |
| **T2** | Tenant-scoped writes, reversible | Draft document, attach metadata, queue a task | NO |
| **T3** | Tenant-scoped writes, irreversible OR cross-pillar | Send email, post to external API, mint capability, KYC submit | YES (per-invocation human-in-loop OR pre-signed policy) |
| **T4** | Cross-tenant, financial, regulated, destructive | Bulk export, payment send, model-fine-tune, schema migration | YES (escalation + ratification per capability record) |

Tier defines the **maximum** autonomy of the capability. A capability
operating below its tier ceiling is fine; uplift past the ceiling is
**not** permitted via runtime flag.

Sources: ADR-0024 (capability ceiling), ADR-0025 (Foundry consolidation).

## 2. The capability record

Per [`docs/AGENTS.md`](../AGENTS.md) D5, every capability published
under `registry/capability-templates/` carries a record with:

```yaml
---
capability_id: foundry.rag.semantic-search
version: 1.4.0
owner_team: axis-foundry
autonomy_tier: T1
status: stable
data_classes_consumed:
  - tenant-data
  - public
data_classes_produced:
  - internal
cedar_policy: registry/capability-templates/policies/foundry.rag.semantic-search.cedar
runtime_gate: intelligence-runtime-rag::gate::semantic_search
audit_topic: EVT-CAPABILITY-INVOKED
eval_set: registry/capability-templates/eval-sets/foundry.rag.semantic-search/
  - golden.jsonl
  - adversarial.jsonl
  - linguistic.jsonl
cosign_signature: ...
threat_model: docs/threat-models/foundry.rag.semantic-search.md
last_reviewed: 2026-05-08
---
```

Lane: `governance-capability-publish` validates every record
shape on PR.

## 3. Cedar policy binding

Every capability declares a Cedar policy in
`registry/capability-templates/policies/<capability-id>.cedar`. The policy is
**authoritative**; the runtime gate enforces.

```cedar
permit (
    principal in Agent::"runtime",
    action in Action::"Invoke",
    resource is Capability
) when {
    resource.autonomy_tier == "T1" &&
    principal.tenant == resource.tenant &&
    context.consent_token != null
};

permit (
    principal in Agent::"runtime",
    action in Action::"Invoke",
    resource is Capability
) when {
    resource.autonomy_tier == "T3" &&
    principal.tenant == resource.tenant &&
    context.consent_token != null &&
    context.human_approval_id != null &&
    context.human_approval_expires_at > context.now
};
```

Source: [Cedar — Policy Language](https://www.cedarpolicy.com/),
[Amazon Verified Permissions](https://aws.amazon.com/verified-permissions/).

## 4. Runtime gate

Every capability binding is invoked through `intelligence-runtime-*::invoke`,
which performs five non-bypassable steps in order:

1. **Verify capability record signature** (Cosign attestation of the record).
2. **Evaluate Cedar policy** against `{principal, action, resource, context}`;
   on `Deny`, return `AutonomyError::Denied { tier, reasons }`.
3. **Emit `EVT-CAPABILITY-INVOKED`** to the audit chain before execution.
4. **Execute** the capability.
5. **Emit `EVT-CAPABILITY-COMPLETED`** with the response summary.

The signature is `async fn invoke<C: Capability>(cap: &C, req: C::Request,
ctx: InvocationContext) -> Result<C::Response, AutonomyError>`. The gate IS
the runtime contract; calls that bypass it fail review.

Rules:

1. Steps 1–5 are NEVER bypassed. The gate IS the runtime contract.
2. Tier uplift = changing the capability record's `autonomy_tier` +
   updating the Cedar policy + landing the PR through
   `capability-reviewer` + Council-Privacy + Council-Architecture.
3. Config flags MAY influence within-tier behavior; they MUST NOT bump
   tier semantics.

## 5. Tier-uplift PR shape

A PR that changes `autonomy_tier` from `Tn` to `T(n+1)`:

- MUST include a Cedar-policy diff.
- MUST include a runtime-gate test that proves the policy rejects out-
  of-scope contexts.
- MUST include a threat-model update (per
  [`security-review.md`](security-review.md) §3).
- MUST cite ADR(s) authorizing the tier change.
- MUST be reviewed by `capability-reviewer` + `security-reviewer` +
  Council-Privacy + Council-Architecture.
- Emits `EVT-AUTONOMY-UPLIFT` on merge.

Lane: `governance-autonomy-ceiling` refuses tier changes
missing any of the above.

## 6. T3 / T4 out-of-band approval

For T3 and T4 invocations, the gate consumes a `human_approval_id` from
the context. Approval can be:

1. **Per-invocation human-in-loop**: an operator approves the specific
   request in a UI surface (Foundry approval inbox). The approval is
   bound to (`subject`, `action`, `nonce`) and single-use.
2. **Pre-signed policy**: a capability owner pre-approves a request
   class (e.g., "send email to customer-service domain") with a Cedar
   rule that includes the bound class. Per-instance approval not
   required.

Approval tokens carry a TTL (default 5 min for per-invocation; 24h for
pre-signed). Expired tokens fail the gate.

## 7. Audit-chain integration

Every invocation emits `EVT-CAPABILITY-INVOKED` and
`EVT-CAPABILITY-COMPLETED` per [`observability.md`](observability.md) §4
with:

- `capability_id`, `capability_version`.
- `autonomy_tier`, `cedar_decision_id`.
- `tenant_id`, `actor_id`, `consent_token`, `human_approval_id`
  (when applicable).
- `trace_id`, `span_id` (W3C).

Tier uplift emits `EVT-AUTONOMY-UPLIFT` linking the old + new tier and
the ADR cite.

## 8. Eval-set requirements

Per AGENTS.md D5, every capability publishes:

- **Golden set**: known-good (request, response) pairs.
- **Adversarial set**: jailbreak / prompt-injection / boundary probes.
- **Linguistic set**: paraphrase coverage; cross-lingual where
  applicable.

Tier uplift requires:

- Adversarial set ≥ 100 cases at T3; ≥ 500 at T4.
- Linguistic set ≥ 50 paraphrases per primary intent.
- Replay-as-eval: every prior incident attributable to the capability
  has a regression entry.

Lane: `governance-foundry-eval` (per DOC-CATALOG.md §4).

## 9. Cross-tenant invocations

T1/T2/T3 capabilities are tenant-scoped by default. Cross-tenant
invocations require T4 (e.g., a platform-aggregator capability that
rolls up billing across tenants). The Cedar policy explicitly grants
the cross-tenant principal class.

The cross-tenant probe fitness lane
(`cross-tenant-access-fuzz` per DOC-CATALOG.md §4) deterministically
proves cross-tenant access fails closed on T1/T2/T3 capabilities.

## 10. Anti-patterns

1. **Config flag uplift** (e.g., "temporarily" allowing a T2 capability
   to write cross-tenant). Refused.
2. **`permit (...) when true`** in a Cedar policy. Refused by the lane.
3. **Skipping the gate** (direct call bypassing `runtime::invoke`).
4. **Tier-down without ADR** — down-tiering changes posture of every
   caller.
5. **Reusing a human-approval token** across invocations beyond TTL.

## 11. Sources scanned

- [`forbidden-operations.json`](../../specs/forbidden-operations.json) FO-10;
  [`docs/AGENTS.md`](../AGENTS.md) §Pre-flight Item 4 + D5;
  [`docs/DOC-CATALOG.md`](../DOC-CATALOG.md) §4 (foundry-eval,
  cross-tenant-access-fuzz).
- ADR-0021 (Cedar policy), ADR-0022 (runtime gate), ADR-0024 (capability
  ceiling), ADR-0025 (Foundry consolidation).
- [Cedar — Policy Language](https://www.cedarpolicy.com/),
  [Amazon Verified Permissions](https://aws.amazon.com/verified-permissions/).
- KR PIPC + EU AI Act + US Executive Order 14110 (jurisdiction-specific
  autonomy-tier bindings live in regional packs).
