---
doc_class: ImplementationPlan
impl_plan_id: IP-005-emission-usecase-and-adapter
status: pending
owner: axis-audit-chain
acceptance_lanes: [cargo-check, cargo-build, cargo-clippy, cargo-nextest, port-location, layer-correctness]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-005: emission-usecase + emission-api + emission-adapter + emission-rest + emission-sdk + emission-app

## Intent

Ship the full `emission` BC pipeline: usecase orchestrator + protocol-neutral api + adapter (WAL writer + event-id minter + idempotency dedup + SPIFFE binding) + REST surface + SDK + composition root.

## Crates introduced (6)

- `oya-audit-chain-emission-usecase`
- `oya-audit-chain-emission-api`
- `oya-audit-chain-emission-adapter`
- `oya-audit-chain-emission-rest`
- `oya-audit-chain-emission-sdk`
- `oya-audit-chain-emission-app`

## Concrete File Targets

Per crate per ADR-0105: `Cargo.toml`, `src/lib.rs` (or `src/main.rs` for app), per-crate modules. Catalog YAML entries.

## Code Shape (usecase)

```rust
// emission-usecase/src/lib.rs
pub async fn emit_event<E, W, P>(
    emitter: &E,
    wal: &W,
    principal_resolver: &P,
    spiffe_id: &str,
    raw_event: AuditEvent,
    idempotency_key: ulid::Ulid,
) -> Result<EmitReceipt, EmissionError>
where
    E: AuditEmitter,
    W: WalWriter,
    P: PrincipalResolver,
{
    // 1. Resolve principal from SPIFFE
    let principal = principal_resolver.resolve_from_spiffe(spiffe_id).await?;

    // 2. Verify tenant + pack binding (Cedar policy check)
    cedar_policy_check(&principal, &raw_event)?;

    // 3. Idempotency dedup (24h window)
    if let Some(existing_receipt) = check_idempotency(&idempotency_key).await? {
        return Ok(existing_receipt);
    }

    // 4. Build envelope (domain layer)
    let envelope = oya_audit_chain_emission_domain::build_envelope(&raw_event, &principal);

    // 5. Durable-write to WAL
    wal.write(envelope.clone()).await?;

    // 6. Stage raw blob to S3
    // ...

    // 7. Return receipt
    let receipt = EmitReceipt {
        event_id: envelope.event_id,
        period_id: envelope.period_id,
        pack: raw_event.pack,
        tenant_partition: envelope.tenant_partition,
        accepted_at: chrono::Utc::now(),
        sealed: false,
    };
    Ok(receipt)
}
```

## Test Plan

Per IP class: usecase ≥ 3 unit + ≥ 3 integration against mocked ports; adapter ≥ 2 integration against real Postgres + S3 (test containers); rest ≥ 2 happy + auth-fail + tenant-mismatch + size-overflow + idempotency-replay; sdk ≥ 5 (happy + retry + auth-fail + idempotency); app ≥ 1 smoke.

## Acceptance Gates

```bash
cargo nextest run -p oya-audit-chain-emission-usecase
cargo nextest run -p oya-audit-chain-emission-adapter --features integration
cargo nextest run -p oya-audit-chain-emission-rest --features integration
cargo nextest run -p oya-audit-chain-emission-sdk
cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice audit-chain
```

## References

- Bominal ADR-0003 §"Emission contract".
- `microservices/audit-chain/contracts/openapi/audit-chain.yaml` /emit path.
