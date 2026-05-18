---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M02-foundation
phase: P01-network-foundation
impl_plan_id: IP-010-inmail-bridge-bc
status: pending
execution_unit: ChangeSet
changeset_contract: claimable-verifiable-bundleable-promotable
owner: axis-network + axis-messenger
acceptance_lanes: [cargo-check, cargo-nextest, oya-governance-professional-context-isolation, oya-gate-inmail-bridge-contract]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-010: inmail-bridge BC end-to-end (Professional-tier bridge to messenger; ADR-NET-0003)

## Intent

Author the full `inmail-bridge` BC per ADR-NET-0003:

- gRPC contract to messenger µservice with `context_kind: Professional` + `inmail_target_channel: professional` invariant (PCI-09).
- Per-tenant rate budget enforced at REST handler (Redis token-bucket).
- foundry-runtime spam-classifier invoked before bridge dispatch; verdict surfaced to sender.
- Audit-chain seal on every send + delivery + read receipt.
- Backpressure via Redis Streams `network:inmail:bridge:queue:<tenant_id>` per `runbooks/inmail-fanout-degraded.md`.
- Minor-account FORBID: minor cannot receive InMail from unconnected adult (Cedar `tenant-scope.cedar`).

## Code Shape

```rust
// kernel/src/ports.rs
#[async_trait]
pub trait InMailBridge: Send + Sync {
    async fn send(&self, inmail: InMail) -> Result<InMailDeliveryReceipt, BridgeError>;
}

#[async_trait]
pub trait SpamClassifierClient: Send + Sync {
    async fn classify(&self, body: &InMailBody) -> Result<SpamVerdict, ClassifierError>;
}

#[async_trait]
pub trait RateBudgetEnforcer: Send + Sync {
    async fn consume(&self, tenant_id: &TenantId, user: &UserRef) -> Result<(), BudgetError>;
}
```

## Acceptance Gates

```bash
cargo nextest run -p oya-network-inmail-bridge-kernel
cargo nextest run -p oya-network-inmail-bridge-adapter-messenger-bridge
cargo run -p oya-dev-cli -- gate validate professional-context-isolation --microservice network
cargo run -p oya-dev-cli -- gate validate inmail-bridge-contract --microservice network
```

## Test Plan

- Synthetic InMail send: spam-classifier verdict surfaced; user confirms; messenger-bridge dispatch < 100ms (`slos/inmail-send-latency.openslo.yaml`).
- Personal-tier `messenger` response: runtime guard rejects + Sev-1 alert + audit-chain seal.
- Rate-budget exhaustion: 250/day cap; 251st send returns 429.
- Minor-account InMail FORBID: Cedar denial at REST handler.
- Backpressure: messenger-µservice Sev-2 → queue depth grows; per `runbooks/inmail-fanout-degraded.md` recovers on messenger restore.

## Halt Conditions

- Personal-tier delivery path discovered — Sev-1; this is PCI-09 violation; engage council-privacy.

## Next IP

[`IP-011-jobs-handoff-bc.md`](IP-011-jobs-handoff-bc.md)

## References

- ADR-NET-0003 (InMail bridge architecture).
- `microservices/network/policy/professional-context-isolation.md` PCI-09.
- `microservices/network/runbooks/inmail-fanout-degraded.md`.
- ADR-MSGR-0004 (sibling messenger Professional-tier surface).
