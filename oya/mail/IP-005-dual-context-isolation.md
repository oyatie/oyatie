---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M03-connect-dissolution
phase: P01-mail-dissolution-from-connect
impl_plan_id: IP-005-dual-context-isolation
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-mail + council-privacy
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness, oya-governance-dual-context-cross-boundary, oya-governance-mail-context-immutability]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-mail-dual-context-isolation-{kernel,domain,usecase,api,adapter,app}

## Intent

Scaffold the dual-context-isolation BC. Six crates (kernel + domain + usecase + api + adapter + app) implementing the `ContextBoundaryGuard` port + kernel-immutable `ContextKind` + cross-context refusal at the API boundary per `policy/dual-context-isolation.md` Invariants DCI-01..DCI-08.

## ChangeSet boundary

6 Rust crates; one Cedar schema fragment; integration with `auditor-scope.cedar` + `tenant-scope.cedar`.

## Concrete File Targets

| Path | Action |
|---|---|
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-kernel/` | create (port + entities) |
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-domain/` | create (pure logic) |
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-usecase/` | create (orchestrators) |
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-api/` | create (typed contracts) |
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-adapter/` | create (Cedar evaluator integration) |
| `microservices/mail/src/crates/oya-mail-dual-context-isolation-app/` | create (composition root) |
| `microservices/mail/policy/schema.cedarschema` | create | Cedar schema for ContextKind + Mailbox + LegalHoldScope |
| `microservices/mail/catalog/oya-mail-dual-context-isolation-{...}.yaml` × 6 | create | catalog rows |

## Crate Naming

```
NAME: oya-mail-dual-context-isolation-{layer}
JUSTIFICATION:
- microservice = mail
- bc-tokens = dual-context-isolation (sibling BC)
- layer = per ADR-0105 13-value enum
- exemptions claimed: none
```

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait ContextBoundaryGuard: Send + Sync + Sealed {
    fn assert(&self, principal_ctx: ContextKind, resource_ctx: ContextKind)
        -> Result<(), ContextBoundaryError>;
}

// usecase/src/lib.rs
pub struct DefaultContextBoundaryGuard;

#[async_trait]
impl ContextBoundaryGuard for DefaultContextBoundaryGuard {
    fn assert(&self, p: ContextKind, r: ContextKind) -> Result<(), ContextBoundaryError> {
        if p != r {
            audit_chain::emit("mail_cross_context_routing_refused", ...);
            metric::inc("mail_cross_context_routing_refused_total");
            return Err(ContextBoundaryError::CrossBoundary { from: p, to: r });
        }
        Ok(())
    }
}
```

```rust
// LEAN check verification: every kernel struct uses #[non_exhaustive] + no setter on context_kind
// Tests assert deserialization rejects context_kind mutation.
```

## Acceptance Gates

```bash
cargo nextest run -p oya-mail-dual-context-isolation-domain
cargo nextest run -p oya-mail-dual-context-isolation-usecase
buck2 build //:quality-lane-registry-authority-check # lane=dual-context-cross-boundary --microservice mail
buck2 build //:quality-lane-registry-authority-check # lane=mail-context-immutability --microservice mail
buck2 build //:quality-lane-registry-authority-check # lane=personal-pillar-kms-scope
buck2 build //:quality-lane-registry-authority-check # lane=personal-pillar-hold-forbidden
buck2 build //:quality-lane-registry-authority-check # lane=search-index-context-partition --microservice mail
buck2 build //:quality-lane-registry-authority-check # lane=migration-context-tagging --microservice mail
```

## Test Plan

- Unit: cross-context routing refused with metric + audit emission.
- Property-based: ANY combination of (principal_ctx, resource_ctx) where ≠ → refused; where = → permitted.
- Cedar policy: 8 test cases covering Personal → Professional, Professional → Personal, hold on Personal forbidden, search partition.
- E2E `tests/e2e/cross-context-refusal.sh`: Professional API call with Personal mailbox ID → 403 + audit-emit.

## Halt Conditions

- LEAN check finds any `mailbox_store.read()` not preceded by `context_boundary_guard.assert()`.
- Personal-pillar KMS scope test finds any role with subject != user.user_id can decrypt.
- Any code path bypasses the four-eyes check for legal-hold engage.


## Sustainability emission (per ADR-0344)
- Per-call audit row emission: populate `cost_usd_minor_units`, `co2_grams`, and `watt_hours` with provider and region on every audit-chain row.
- Carbon-aware scheduling eligibility: opt-in only; do not defer Tier 0/1 workloads or realtime-mandated compliance-pack workloads (`eu-ai-act-annex-iii`, `hipaa-em-incident-response`, `pci-dss-realtime-fraud-detection`).
- finops-portal rollup axes affected: tenant / product / capability / provider / cell / compliance_pack.
- Surface evidence: `microservices/mail/IP-005-dual-context-isolation.md` matched `emission`; anchors `microservices/mail/manifest.json, crates/oya-shared-email-comms-kernel/src/lib.rs`; type anchor `crates/oya-shared-email-comms-kernel/src/lib.rs::OutboundMessage`.

## Next IP

[`IP-006-inbound-smtp.md`](IP-006-inbound-smtp.md)

## References

- `microservices/mail/policy/dual-context-isolation.md` Invariants DCI-01..DCI-08
- Bominal ADR-0208 (dual-context unified channel hub)
- Bominal ADR-0215 (retention/legal-hold dual-context)
- ADR-0135 (super-app dissolution authority)
- ADR-0140 (retired per ADR-0145) (Cedar policy enforcement)
- Cedar policy language — `cedarpolicy.com`
