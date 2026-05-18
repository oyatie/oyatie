---
doc_class: ImplementationPlan
template_id: TPL-IMPL
milestone: M01-foundation
phase: P01-openbao-secretreference-substrate
impl_plan_id: IP-013-audit-emitter-bridge-to-audit-chain
status: pending
owner: axis-cloud-secrets + axis-governance
acceptance_lanes: [audit-seal-e2e]
---

<!-- Canonical-base: specs/ip/canonical-frontmatter-schema.json + docs/templates/ip-boilerplate-fragments.md (SWEEP-I Slice 6 per ADR-0064) -->

# IP-013: audit-emitter bridge to audit-chain

## Intent

Bridge the local OpenBao audit-device file → `audit-chain` µservice with Ed25519 signing per Bominal ADR-0028.

## ChangeSet boundary

Five new crates: kernel, usecase, api, adapter-audit-chain-bridge, app.

## Concrete File Targets

| Path | Action |
|---|---|
| `…/oya-cloud-secrets-audit-emitter-kernel/` | `SecretAuditEvent`, `AuditChainBridgeMessage` |
| `…/oya-cloud-secrets-audit-emitter-usecase/` | orchestrate file-tail → sign → bridge |
| `…/oya-cloud-secrets-audit-emitter-api/` | typed contracts |
| `…/oya-cloud-secrets-audit-emitter-adapter-audit-chain-bridge/` | bridge HTTP client (audit-chain) |
| `…/oya-cloud-secrets-audit-emitter-app/` | bridge worker binary |
| 5× catalog yamls | create |

## Acceptance Gates

```bash
cargo nextest run -p 'oya-cloud-secrets-audit-emitter-*'
# E2E with mock audit-chain
cargo nextest run --features audit-seal-e2e
```

## Test Plan

- Every SecretAccessed event reaches audit-chain within p99 ≤1s.
- audit-chain outage: local file durable; bridge resumes on recovery.
- Replay idempotent: dedup via `(event_id, signature)`.

## Halt Conditions

- Audit events sent unsigned — BLOCKER.
- Local file capped without rotation — BLOCKER.

## Next IP

`IP-014-observability-slo-branch-protection-hg-cloud-secrets.md`
