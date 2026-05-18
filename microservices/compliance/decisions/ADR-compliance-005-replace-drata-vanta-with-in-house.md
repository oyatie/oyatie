---
id: ADR-compliance-005
status: Accepted
deciders: council-architecture, axis-compliance, axis-finops
date: 2026-05-18
related_adrs: [ADR-0173, ADR-0209]
---

# ADR-compliance-005 — Replace Drata / Vanta with in-house pipeline (rationale)

## Context

Per ADR-0173 vendor-lock-in avoidance + ADR-0209 compliance evidence automation: we build the compliance pipeline in-house rather than wrap Drata / Vanta / Tugboat / AuditBoard / ServiceNow GRC.

## Why

1. **Direct auditor relationship** — no SaaS proxy between us and the auditor.
2. **Verifiable tamper evidence** — cosign keyless OIDC chain; auditor can verify independently of our infrastructure.
3. **Sovereignty preserved** — evidence never leaves operator-controlled cluster (critical for KR / UAE / EU sovereignty packs).
4. **Cost stable** — engineering time amortized; no per-employee SaaS fee.
5. **Differentiation** — direct auditor relationship + verifiable seals + sovereignty are sellable differentiators vs commercial GRC.

## Consequences

We own the on-call rotation for the pipeline. Mitigation: existing observability backplane covers paging; existing incident-response runbooks apply.

See `competitor-parity-matrix.md` for feature-by-feature comparison.
