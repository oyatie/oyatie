---
doc_class: Spec
shape: anchor
length_cap: 200
authority_tier: 1
status: Accepted
date: 2026-05-12
purpose: |
  Write-side dark-launch (shadow traffic + diff-compare) for high-risk surfaces.
  Diff kernel: intelligence-shadow-diff-kernel. Aligned with Foundry RAG gate pattern.
planned_enforcement_ref:
  - governance-shadow-diff
related_adrs: [ADR-0040, ADR-0011, ADR-0024, ADR-0053, ADR-0055]
adr_citations: [ADR-0053, ADR-0055]
doc_status: published
---

# Dark-Launch Specification


## 1. What dark-launch means here

Dark-launch = the new code runs in production on real requests, but its outputs are **shadow-only** — not returned to the caller, not committed to durable state. We diff shadow outputs against baseline outputs and gate promotion on agreement.

Two flavours:

1. **Read-side dark-launch** — new read path runs alongside; outputs diffed; cheap, low-risk.
2. **Write-side dark-launch** — new write path runs alongside in a sandbox transaction (or against a shadow store); outputs diffed; **expensive** but mandatory for high-risk surfaces.

## 2. When mandatory (high-risk surfaces)

Write-side dark-launch is REQUIRED for:

- **Foundry capability publish** with replay-affecting changes ([ADR-0024](../../decisions/ADR-0024-intelligence-eval-harness-and-replay.md)).
- **Cross-axis contract** changes ([ADR-0011](../../decisions/ADR-0011-cross-axis-contract-registry.md)).
- **Cedar policy** changes ([ADR-0007](../../decisions/ADR-0007-cedar-authorization-policy-and-persona-tier.md)).
- **Audit-chain** schema changes ([ADR-0003](../../decisions/ADR-0003-audit-chain-and-evidence-emission.md)).
- **Billing / metering** logic ([ADR-0031](../../decisions/ADR-0031-ads-and-analytics-architecture.md)).
- **Search ranking** changes (per [`playbook-search.md`](playbook-search.md)).
- **DSR / proof-of-erasure** logic ([ADR-0038](../../decisions/ADR-0038-trust-framework-and-dsr-cascade-and-proof-of-erasure.md)).

## 3. The diff kernel: `intelligence-shadow-diff-kernel` (NEW)

Compares baseline-output and shadow-output records, classifies diffs, and emits a verdict. Inputs: pair-stream `(baseline_output, shadow_output, request_context)`. Outputs: per-pair classification + aggregate verdict.

### Diff classes

| Class | Meaning | Promotion impact |
|---|---|---|
| `identical` | Byte-equal output | Passes |
| `semantically-equivalent` | Different serialisation, same meaning (e.g. JSON key reorder) | Passes (with normaliser) |
| `expected-divergence` | Documented intentional change (e.g. new field added) | Passes if covered by allowlist |
| `unexpected-divergence` | Anything else | Blocks promotion |
| `error-only-shadow` | Shadow errored, baseline OK | Blocks promotion |
| `error-only-baseline` | Baseline errored, shadow OK | Reviewed (potential fix) |

### Threshold

`unexpected-divergence` + `error-only-shadow` together must be **≤ 0.01%** of sampled pairs across a minimum 10,000-pair window. Below threshold = promotion-eligible; above = block.

Sampling rate is configurable per surface; defaults: 100% for Cedar/audit/billing/DSR; 10% for cross-axis contracts; 1% for search ranking.

## 4. Alignment with Foundry RAG gate pattern

The Foundry RAG retrieval gate (commit 498b3ce) is the precedent: cross-tenant boundary gated before citations. Dark-launch generalises that pattern — the shadow gate runs before the new path is observable to callers. `intelligence-shadow-diff-kernel` follows the same kernel/api/adapter shape: pure diff logic in the kernel, transport adapters per surface.

## 5. Adapter crates

- `intelligence-shadow-diff-adapter-http` (NEW) — HTTP request/response pair capture.
- `intelligence-shadow-diff-adapter-grpc` (NEW) — gRPC unary/streaming pair capture.
- `intelligence-shadow-diff-adapter-event` (NEW) — outbox-pattern event pair capture ([ADR-0005](../../decisions/ADR-0005-eventing-backbone-outbox-pattern.md)).
- `intelligence-shadow-diff-adapter-cedar` (NEW) — Cedar evaluation pair capture (decision-only).

## 6. Write-side safety

Write-side dark-launch runs the new write path in one of two safe modes:

1. **Sandbox transaction** — write occurs inside a transaction that always rolls back. Side effects (events, external calls) are captured to a shadow log, not emitted.
2. **Shadow store** — write occurs against a parallel storage instance pre-seeded from baseline. Inspected, then discarded after diff.

External side-effects (emails, payment calls, webhooks) are MUST-be stubbed in the shadow path. Lane `governance-shadow-diff` refuses a dark-launch manifest that lacks the stub-list.

## 7. Promotion path

```
dark-launch start (shadow at 100% of sampled traffic)
   ↓ 10,000-pair window collected
diff-kernel verdict
   ↓ promotion-eligible
canary 1% (per canary-rail-spec)
   ↓ ... usual progression
100% promotion
```

Dark-launch sits **before** canary, not in place of it. Both are required for high-risk surfaces.

## 8. Hyperscaler equivalents

- Google "dark launches" (Production-grade testing in Google SRE Book §16).
- AWS Lambda alias-routing for shadow (1% traffic to new version, response discarded).
- Microsoft Azure Front Door rules-engine "shadow rules".
- Oracle Net Service "trace" mode for parallel-path testing.

## 9. Compliance gates

- `governance-shadow-diff` (NEW; HIGH for high-risk surfaces).
- `governance-canary-required` (NEW; BLOCKER — dark-launch supplements, never replaces canary).

## 10. ADR citations

- [ADR-0055](../../decisions/ADR-0055-four-layer-branch-pipeline.md) — dark-launch on `staging` feeds the shadow-diff evidence required before `prod-promoter` fires.
