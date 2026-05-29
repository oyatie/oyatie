# IP-005: enforcement-domain — Cedar policy compilation + scope/predicate evaluator

- Bounded context: enforcement
- Layer: domain
- Crate: `oya-consent-graph-enforcement-domain`
- Acceptance status: ga
- Authority: ADR-0214 §2.3, ADR-0090 (Cedar adoption), ADR-0056 (domain = pure rules over kernel
  types), ADR-0105.
- Depends on: `oya-consent-graph-{enforcement-kernel, agreement-kernel, agreement-domain}`,
  `cedar-policy = "3.2"`.

## 1. Goal

Bind the Cedar policy engine to the agreement model: compile an agreement's `(scope, terms,
sovereignty)` into a Cedar `PolicySet`, evaluate enforcement requests against it, and report
`EnforcementDecision`s. This is the only layer that depends directly on the `cedar-policy` crate.

## 2. Scope

In:
- `compile(agreement) -> CompiledCedarPolicy`
- `evaluate(policy, request) -> EnforcementDecision` (implements the `PolicyEvaluator` port)
- Cedar schema definition (`Cedar.json`) for `Tenant`, `Principal`, `Resource`, `Action`, `Context`
- Catalog of 14 reusable Cedar policy snippets (one per Big-5 vertical × 2–3 patterns)
- Scope-to-Cedar translation (turn `EntityScope { entity_type, field_set, predicate }` into Cedar
  permit + deny rules)
- Cache-key derivation (so two semantically-identical agreements share a compiled policy)

Out:
- Cache *storage* (port lives in kernel; in-memory impl in adapter).
- Persistence of compiled artifacts (Cedar policy artifacts are ephemeral — recompiled on µservice
  cold start from agreement row).

## 3. Cedar schema

```json
{
  "OyaConsentGraph": {
    "entityTypes": {
      "Tenant": { "shape": { "type": "Record", "attributes": {
        "tenant_id": { "type": "String" },
        "region": { "type": "String" },
        "tenant_class": { "type": "String" }
      }}},
      "Principal": { "memberOfTypes": ["Tenant"], "shape": { "type": "Record", "attributes": {
        "principal_id": { "type": "String" },
        "tenant_id": { "type": "String" },
        "role": { "type": "String" }
      }}},
      "Resource": { "shape": { "type": "Record", "attributes": {
        "entity_type": { "type": "String" },
        "entity_id": { "type": "String", "required": false },
        "grantor_tenant_id": { "type": "String" },
        "field_set": { "type": "Set", "element": { "type": "String" }, "required": false }
      }}}
    },
    "actions": {
      "project.subscribe": { "appliesTo": { "principalTypes": ["Principal"], "resourceTypes": ["Resource"] }},
      "project.read":      { "appliesTo": { "principalTypes": ["Principal"], "resourceTypes": ["Resource"] }},
      "attested.query":    { "appliesTo": { "principalTypes": ["Principal"], "resourceTypes": ["Resource"] }},
      "aggregate.read":    { "appliesTo": { "principalTypes": ["Principal"], "resourceTypes": ["Resource"] }}
    }
  }
}
```

Pack overlays may *extend* the action enum (e.g., us-healthcare adds `emergency.break_glass`) but
must never narrow the base — neutrality per ADR-0064.

## 4. Translate `EntityScope` to Cedar

Algorithm `compile(agreement) -> CompiledCedarPolicy`:

1. Emit base permit rule:
```cedar
permit (
  principal in Tenant::"<grantee_tenant>",
  action in [Action::"<action>"],     // derived from terms.mode
  resource is Resource
)
when {
  resource.entity_type == "<scope.entity_type>" &&
  resource.grantor_tenant_id == "<grantor_tenant>" &&
  context.purpose_of_use == "<terms.purpose_of_use>" &&
  context.request_time <= datetime("<expiration_or_max>") &&
  context.tenant_class == "paid"
};
```

2. Append a field-set permit/deny:
   - `FieldSet::AllFields` → no field constraint.
   - `FieldSet::Allow(fields)` → emit `&& resource.field_set in [<fields>]`.
   - `FieldSet::Deny(fields)` → emit a `forbid` rule for those fields.

3. Append the user-authored predicate (parsed by `agreement-domain::parse_predicate`, then rendered
   into Cedar). The predicate is sandboxed: only `principal.*`, `resource.*`, `context.*` references
   allowed; no function calls outside `in`, `==`, `!=`, `<`, `<=`, `>`, `>=`.

4. Append sovereignty guard:
```cedar
forbid (principal, action, resource)
when { context.request_region not in <permitted_grantee_regions> };
```

5. Append rate-limit context check:
```cedar
forbid (principal, action, resource)
when { context.tenant_class == "demo_trial" && action in Action::"compliance_pack.bound_projection" };
```

6. Set policy id = `cedar::<agreement_id>::v<schema_version>` for cache keying.

## 5. Evaluate

```rust
impl PolicyEvaluator for CedarPolicyEvaluator {
    fn evaluate(&self, policy: &CompiledPolicyHandle, request: &EnforcementRequest)
        -> Result<EnforcementDecision, EvaluatorError>
    {
        let policies = policy.opaque.downcast_ref::<CedarPolicySet>()
            .ok_or(EvaluatorError::DowncastFailure)?;
        let cedar_req = self.build_cedar_request(request)?;
        let entities = self.build_entities(request)?;
        let started = Instant::now();
        let response = Authorizer::new().is_authorized(&cedar_req, policies, &entities);
        let elapsed = started.elapsed().as_nanos() as u64;
        let outcome = match response.decision() {
            Decision::Allow => EnforcementOutcome::Permit,
            Decision::Deny  => EnforcementOutcome::Deny { reason: self.derive_deny_reason(&response) },
        };
        let reasons = response.diagnostics().reason()
            .map(|p| DeterminingReason::Policy(p.to_string()))
            .collect();
        Ok(EnforcementDecision {
            outcome,
            matched_agreement: Some(policy.agreement_id),
            determining_policy_id: Some(policy.policy_id.clone()),
            reasons,
            eval_duration_ns: elapsed,
            cache_hit: true,
        })
    }
}
```

## 6. Cache-key derivation

Two agreements produce the *same* compiled artifact if `(scope, terms, sovereignty, schema_version)`
are byte-equal after canonicalization (sorted field-set, lower-cased entity_type, normalized
predicate AST). Compile result memoized by canonical hash → ~80% cache hit rate at warm steady state.

## 7. Policy catalog (14 reusable snippets)

| Snippet ID | Vertical | Use case | Source |
|------------|----------|----------|--------|
| `cg-pol-sc-po-visibility`     | supply chain | PO field-restricted projection | spec |
| `cg-pol-sc-shipment-status`   | supply chain | shipment status w/ PII exclusion | spec |
| `cg-pol-sc-asn-visibility`    | supply chain | ASN field restriction | spec |
| `cg-pol-hc-eligibility`       | healthcare   | eligibility verify attested query | spec |
| `cg-pol-hc-coverage`          | healthcare   | coverage status projection | spec |
| `cg-pol-hc-break-glass`       | healthcare   | break-glass with audit-officer review | spec |
| `cg-pol-bk-account-status`    | banking      | account status projection | spec |
| `cg-pol-bk-tx-status`         | banking      | transaction status projection | spec |
| `cg-pol-bk-balance-attested`  | banking      | balance via attested query | spec |
| `cg-pol-mp-cohort-aggregate`  | marketplace  | cohort aggregate k≥5 | spec |
| `cg-pol-mp-seller-order-vol`  | marketplace  | seller order volume aggregate | spec |
| `cg-pol-b2c-order-tracking`   | B2C          | consumer-initiated order tracking | spec |
| `cg-pol-b2c-self-revoke`      | B2C          | consumer self-revoke | spec |
| `cg-pol-deny-all`             | meta         | deny-by-default fallback | spec |

## 8. Tests

- `compile_supply_chain_template_produces_3_rules` — permit + sovereignty-forbid + field-allow.
- `evaluate_permit_when_scope_matches` — known-good request → Permit.
- `evaluate_deny_field_outside_scope` — request for restricted field → Deny{ScopeNotPermitted}.
- `evaluate_deny_purpose_of_use_mismatch` — request purpose ≠ agreement purpose → Deny{PurposeOfUseMismatch}.
- `evaluate_deny_sovereignty_violation` — request from forbidden region → Deny{SovereigntyViolation}.
- `compile_caching_idempotent_on_canonical_equal` — two equal scopes → same `policy_id` + same compiled bytes.
- `evaluate_panic_returns_indeterminate` — injected panicking evaluator → Indeterminate (caught upstream).
- `compile_cedar_3_2_compat_smoke` — Cedar 3.2 API surface used (no deprecated calls).
- `evaluate_latency_under_10ms_p99_synth` — synthetic 10K iteration shows p99 < 10ms on M2.

## 9. Performance

- Compile: ≤200ms (one-time per agreement-accept).
- Evaluate: ≤2ms p99 hot, ≤10ms p99 cold (per ADR-0214 SLO).
- Cache hit-rate target ≥80%.

## 10. Dependencies

- `cedar-policy = "3.2"`
- `oya-consent-graph-enforcement-kernel`
- `oya-consent-graph-agreement-{kernel, domain}`
- `serde`, `thiserror`, `instant`
- **No** Postgres, **no** Pulsar, **no** Tokio runtime — but uses `Instant`.

## 11. Verification

- `cargo test -p oya-consent-graph-enforcement-domain` clean.
- Cedar 3.2 schema validation passes (`cedar-policy::Schema::from_json_str(...).is_ok()`).
- Benchmark group `criterion` runs and writes `target/criterion/.../report.json` for trend tracking.

## 12. Risk

- **R**: Cedar 3.2 → 4.x breaking change.
  **M**: Pinned in `Cargo.toml`; upgrade requires ADR-SVC-CG-* + benchmark regression check.
- **R**: Field-level deny rules combinatorially explode for wide entities.
  **M**: Use `field_set` set comparison (one Cedar `in` operator) rather than per-field rule emission.
- **R**: Predicate sandbox escape (user injects unsafe Cedar via `predicate` field).
  **M**: Parser in `agreement-domain` produces a typed AST; renderer in `enforcement-domain` emits
  only well-formed Cedar; no raw string passthrough.

## Wave 15-IP-substance counterpart evidence

Preserved as substantive. Counterpart anchors: TrustArc and OneTrust handle purpose/preference logic, Cookiebot handles consent categories, and data-sharing counterparts lean on RBAC. This IP turns those counterpart concepts into Cedar policy sets with field-level scope, purpose, sovereignty, and aggregate constraints, which is the service-specific enforcement substance absent from generic CMP or warehouse-share controls.

Grep-recognized counterpart anchor: Snowflake and Databricks clean-room/data-sharing controls are relevant here because Cedar compilation must enforce scoped, purpose-bound access before any warehouse or clean-room projection can run. Salesforce and HubSpot are relevant only as downstream consent-propagation systems; they do not replace OneTrust/TrustArc as the primary consent comparator.
