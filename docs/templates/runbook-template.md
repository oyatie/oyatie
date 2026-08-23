---
doc_status: published
---

# Runbook: <name>

> **Owner:** `<team-id>` from [`teams/`](../teams/)
> **Severity supported:** Sev 1 | Sev 2 | Sev 3 | Sev 4
> **Last verified:** YYYY-MM-DD by <author> in drill / production
> **Related:** [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md), [SLO-CATALOG.md](../SLO-CATALOG.md), [security-program.json](../security-program/security-program.json)

---

## Trigger

The exact symptom or alert that should cause an operator (or agent) to open this runbook. Be specific:
- Alert name(s)
- SLI breach pattern
- Customer report shape

---

## Pre-checks (5 minutes max)

Verifiable conditions before executing the runbook. Each is a binary check.

- [ ] Pre-check 1: <condition> — verify by <command/dashboard>
- [ ] Pre-check 2: <condition>
- [ ] Pre-check 3: <condition>

If any pre-check fails, **STOP** and route to a different runbook (cite which) or escalate.

---

## Steps

Numbered steps. Each step:
- ☐ One sentence + the exact command / dashboard / API call
- Expected outcome (one sentence)
- If outcome differs, escalate

```
1. ☐ Verify the affected surface — `oya ops ... <args>`
   Expected: <output shape>
   If differs: ...

2. ☐ Quarantine ... — `oya ops ... <args>`
   Expected: ...
   If differs: ...

...
```

---

## Rollback

What to undo if the steps above made things worse. Always reversible if possible.

---

## Verification

Conditions that prove the issue is resolved.

- [ ] Verification 1: ...
- [ ] Verification 2: ...
- [ ] SLO returns to within budget

---

## Post-incident updates

After this runbook is invoked, update:
- [ ] Incident postmortem per [`templates/incident-postmortem-template.md`](incident-postmortem-template.md)
- [ ] [`docs/MISTAKES-LEDGER.md`](../MISTAKES-LEDGER.md) entry if a new prevention is identified
- [ ] This runbook with new steps / pre-checks / verification if the incident exposed a gap

---

## Audit-chain emission

Per ADR-0003, every runbook invocation emits to `oyatie.ops.runbook.invoked` with:
- runbook-id
- invoker-id
- timestamp
- outcome (resolved / escalated / unresolved)
- affected tenant(s) / surface(s)

---

## Sources scanned

- Per-service playbook (linked)
- Related ADR / issue refs
- Industry references (e.g. Google SRE workbook chapter)
