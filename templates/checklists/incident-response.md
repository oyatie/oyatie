---
doc_status: published
---

# Checklist: Incident Response

> **When:** Sev 1/2 (or escalated Sev 3) detected. Mechanically driven by [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md).
> **Owner:** Incident Manager (IM) on duty.
> **Validator:** `incident-template-completeness` (post-incident).

---

## 0. Detection (≤ 5 min from signal)

1. ☐ **Signal received** — alert / customer-report / synthetic / security-event
2. ☐ **Triage triggered** — IM assigned per on-call rotation
3. ☐ **Severity declared** — per [INCIDENT-MANAGEMENT.md §1](../INCIDENT-MANAGEMENT.md) taxonomy
4. ☐ **Bridge opened** — voice + chat; pinned in #incidents-<id>

## 1. Response (≤ 15 min for Sev 1; ≤ 1h Sev 2)

5. ☐ **SME(s) paged** per affected axis / surface
6. ☐ **Comms Manager (CM)** paged for Sev 1
7. ☐ **Privacy Lead** paged if Sev 1 with data-class touch
8. ☐ **Security Lead** paged if security-class
9. ☐ **Founder** notified for Sev 1
10. ☐ **Runbook invoked** per [RUNBOOKS-INDEX.md](../RUNBOOKS-INDEX.md) for the affected surface
11. ☐ **Per-affected-tenant impact estimated** (count + class + data-class touched)
12. ☐ **Per-cell containment** if cross-tenant risk (per [ADR-0009-cell-architecture](../decisions/ADR-0009-cell-architecture-per-tenant-per-region.md))

## 2. Regulatory clock (start immediately for Sev 1)

13. ☐ **PIPA Art 34** — KR FSS notification within 24h (if KR tenant + data-class touched)
14. ☐ **PIPC** — within 72h (KR personal info breach)
15. ☐ **GDPR Art 33** — Supervisory Authority within 72h (EU)
16. ☐ **HIPAA Breach Notification** — HHS within 60d (US healthcare)
17. ☐ **PCI-DSS** — per acquirer SLA (often 24h)
18. ☐ **Per-pack regulator** per [COMPLIANCE-MATRIX.md](../COMPLIANCE-MATRIX.md) row

## 3. Mitigation (target: stop customer impact)

19. ☐ **Runbook step-by-step executed** with verification per step
20. ☐ **Customer impact stopped or contained**
21. ☐ **Audit-chain emission verified** for the mitigation actions

## 4. Resolution (target: root cause fixed)

22. ☐ **Root cause identified** (5-Whys / Causal-Tree)
23. ☐ **Fix deployed** (with rollback path documented)
24. ☐ **Post-deploy verification** — affected surface SLO returns to within budget
25. ☐ **Bridge closed** + handed off to postmortem owner

## 5. Comms

26. ☐ **Customer notification** drafted by CM, sent ≤ 24h Sev 1 / ≤ 48h Sev 2
27. ☐ **Status page updated** ≤ 1h after detection (Sev 1/2)
28. ☐ **Trust portal incident page** live for Sev 1
29. ☐ **Internal stakeholder update** every 30min (Sev 1) / 60min (Sev 2) until resolved

## 6. Postmortem (≤ 30d post-resolution)

30. ☐ **Postmortem draft** per [`templates/incident-postmortem-template.md`](../templates/incident-postmortem-template.md)
31. ☐ **Blameless** review with team + cross-team observers
32. ☐ **Action items** (prevention mechanical + mitigation process) with owners + ETAs
33. ☐ **Trust portal mirror** within 60d
34. ☐ **Mistakes-and-fixes ledger** entry per [`MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md)
35. ☐ **Prevention loop** — mechanical fix shipped within 30d (Sev 1) / 60d (Sev 2)

## 7. Anti-patterns

- Skipping the regulatory clock — never; even if mitigation is fast, the clock starts at detection
- Closing the bridge before resolution — never
- Action items without owners + ETAs — never
- Process-only prevention (no mechanical fix) — anti-pattern; per [`standards/prevention-doctrine.md`](../standards/prevention-doctrine.md)
- Founder bypass on Sev 1 customer/regulator notification language — never
