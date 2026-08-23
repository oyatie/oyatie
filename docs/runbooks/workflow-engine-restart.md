---
purpose: Oyatie Runbook — Workflow Engine Restart
doc_status: published
---

# Oyatie Runbook — Workflow Engine Restart

> **Status:** Link-continuity page; the canonical SaaS operator procedure is [`docs/runbooks/saas/workflow-engine-deadlock.md`](saas/workflow-engine-deadlock.md).
> **Owner:** `axis-saas`
> **Authority:** `docs/products/saas-platform/PRD.md`, `specs/masterplan.json` M03-P04/M03-P08, and the canonical SaaS runbook linked above.
> **Last verified:** 2026-06-09 (root path checked against the canonical SaaS procedure and SSOT chain).

## Operator action

Use [`docs/runbooks/saas/workflow-engine-deadlock.md`](saas/workflow-engine-deadlock.md) for workflow engine stall, deadlock, replay, and recovery. This root path is retained for existing inbound links only and does not define a separate incident authority.

## Guardrails

- Do not execute an alternate recovery flow from this page.
- Do not treat workstation checks as production authority; the canonical procedure requires cloud control-plane status, sealed audit evidence, and `presubmit` evidence where merge readiness is involved.
- If the canonical procedure and this link page diverge, the canonical SaaS procedure wins.

## Sources

[`docs/runbooks/saas/workflow-engine-deadlock.md`](saas/workflow-engine-deadlock.md), [`docs/products/saas-platform/PRD.md`](../products/saas-platform/PRD.md), `specs/masterplan.json` M03-P04/M03-P08, and `HANDOFF.md`.
