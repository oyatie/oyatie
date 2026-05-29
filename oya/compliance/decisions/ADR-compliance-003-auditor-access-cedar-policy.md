---
id: ADR-compliance-003
status: Accepted
deciders: axis-compliance, axis-security
date: 2026-05-18
related_adrs: [ADR-0183, ADR-0209]
---

# ADR-compliance-003 — Auditor access Cedar policy (per-engagement; read-only; auto-revoke)

## Decision

- Per-engagement Cedar role binding (`Auditor::"engagement-<id>"`).
- Read-only — write/upload actions forbidden via Cedar `forbid` rule.
- Engagement window: open + close timestamps; reads outside window denied.
- Engagement-end webhook revokes the role binding; integration test asserts revoke.

See `capabilities/auditor-engagement-read.cedar`.
