---
microservice: compliance
ip: IP-010
title: Attestation aggregator (cross-µservice attestation rollup per framework)
status: Drafting
authority_tier: 3
owner: axis-compliance
date: 2026-05-18
related_adrs: [ADR-0145, ADR-0209]
---

# IP-010 — Attestation aggregator

## Purpose

Roll up per-µservice attestations into per-framework attestation packs that an auditor can review in one place. Each µservice in the fleet emits a quarterly self-attestation (signed by µservice owner); the aggregator gathers these + per-collector evidence + per-incident postmortems into a single auditor-readable pack.

## Acceptance criteria

1. `oya-compliance-attestation-aggregator` job runs quarterly per framework.
2. Output: `evidence/attestation-packs/<framework>-<YYYY-QQ>.json` with per-µservice attestation + linked evidence artifacts + per-incident postmortem refs.
3. Cosign keyless OIDC seal on the aggregated pack.
4. Backstage auditor portal renders the pack with drill-down to each artifact.
5. Per-µservice attestation form template at `policy/attestation-form-template.md`.
6. ≥ 5 integration tests.

## Pack shape

```json
{
  "framework": "soc2-type-2",
  "period": "2026-Q2",
  "microservices": [
    {
      "microservice": "identity",
      "owner": "axis-identity",
      "attestation_text": "...",
      "attestation_signed_by_spiffe_id": "spiffe://...",
      "artifacts": ["evt_x1", "evt_x2", ...],
      "incidents": ["INC-2026-04-12-pagerduty-flap"]
    }
  ],
  "aggregator_seal_hex": "..."
}
```

## Risk + mitigation

- **Risk:** µservice owner misses attestation → blocks audit. **Mitigation:** automated reminder at T-30 / T-7; escalation at T-0; substitute attestation from cell-owner.
- **Risk:** aggregator pack grows unboundedly. **Mitigation:** drill-down by reference; pack carries summary + ids, not full payloads.

## Acceptance evidence

`evidence/ip-010-attestation-aggregator-acceptance.json`.

## Cross-references

- ADR-0145 — substrate.
- ADR-0209 — substrate authority.
- IP-007 — auditor portal (consumer).
