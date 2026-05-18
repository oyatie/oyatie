---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-005-resolver-usecase
status: pending
owner: axis-cloud-secrets
acceptance_lanes: [cargo-test, lean-a1, lean-a2]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: oya-cloud-secrets-secret-reference-resolver-usecase

## Intent

Orchestrate the resolve flow: parse URI → check cache → on miss query OpenBao → policy-eval → emit audit → cache result → return.

## ChangeSet boundary

One new crate; depends on `-kernel` + `-domain`.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-secret-reference-resolver-usecase/Cargo.toml` | create |
| `…/src/lib.rs` | create |
| `…/src/resolve.rs` | create — orchestrator: `pub struct ResolveUseCase<O, C, A> { openbao: O, cache: C, audit: A }` |
| `…/src/list.rs` | create — list orchestrator |
| `…/src/policy_eval.rs` | create — Cedar policy hook |
| `microservices/cloud-secrets/catalog/oya-cloud-secrets-secret-reference-resolver-usecase.yaml` | create |

## Code Shape

```rust
pub async fn resolve<O, C, A>(
    deps: &Deps<O, C, A>,
    reference: &SecretReference,
    principal: &Principal,
) -> Result<ResolvedSecret, ResolveError>
where
    O: OpenBaoClient,
    C: SecretCache,
    A: AuditEmitter,
{
    deps.policy.evaluate(principal, "resolve_secret_reference", reference)?;

    if let Some(cached) = deps.cache.get(reference).await {
        deps.audit.emit_accessed(reference, principal, AccessOutcome::CacheHit).await?;
        return Ok(cached);
    }

    let resolved = deps.openbao.read(reference).await?;
    let ttl = clamp_ttl(resolved.suggested_ttl());
    deps.cache.put(reference.clone(), resolved.clone(), ttl).await;
    deps.audit.emit_accessed(reference, principal, AccessOutcome::CacheMiss).await?;
    Ok(resolved)
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-cloud-secrets-secret-reference-resolver-usecase
cargo run -p oya-dev-cli -- gate validate lean-a1 --crate oya-cloud-secrets-secret-reference-resolver-usecase
cargo run -p oya-dev-cli -- gate validate lean-a2 --crate oya-cloud-secrets-secret-reference-resolver-usecase
```

## Test Plan

- Mock ports; cover paths: cache hit, cache miss, policy deny, OpenBao error.
- Property: every resolve → exactly one audit emission.
- Property: every cache write uses clamped TTL.

## Halt Conditions

- Audit emission skippable — BLOCKER (security invariant).

## Next IP

`IP-006-resolver-adapter-openbao.md`
