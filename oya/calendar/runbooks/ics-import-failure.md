---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: calendar
runbook_id: RB-ics-import-failure
status: Accepted
date: 2026-05-17
owner_team: axis-calendar
severity_applicable: [Sev-2, Sev-3]
related_failure_modes: [FM-05]
doc_status: published
---

# Runbook — .ics Import Failure

## When this runbook fires

- `calendar_ics_import_failure_rate` > 0.1% over 1h, OR
- Tenant reports import job `state=failed`, OR
- Suspected .ics injection attempt detected (T-T-02).

## Symptoms

- Import job stuck `failed`.
- Tenant cannot complete migration.
- Per-event parse errors logged.
- Possible attack signal (parser threw security error).

## Probable causes

1. .ics file violates RFC 5545 (vendor-specific extensions, CRLF errors, unfolding bugs).
2. File exceeds size bounds (> 100k events or > 8KB per line).
3. RRULE horizon exceeds 5y.
4. Injection attempt (crafted payload).
5. Parser version mismatch with vendor-emitted .ics.

## Triage (within 30 min)

1. Acknowledge OnCall page.
2. Identify affected job:
   ```bash
   oya calendar ics-import status --job-id <id>
   ```
3. Check per-event error report:
   ```bash
   oya calendar ics-import errors --job-id <id> --format json | jq '.errors[] | {line, error_kind, sample}'
   ```
4. If suspected injection, capture the payload to sandbox:
   ```bash
   oya calendar ics-import dump-payload --job-id <id> --sandbox-only --audit-reason "RB-ics-import-failure-attack-suspected"
   ```
5. Check parser version + RFC 5545 corpus pass rate:
   ```bash
   cargo run -p oya-calendar-ics-import-export-app -- parser version
   ```

## Mitigation steps

### Step 1 — Per-event partial success

If most events parse but some fail:

```bash
oya calendar ics-import retry --job-id <id> --skip-failed --audit-reason "RB-ics-import-failure-partial"
```

Tenant receives report of skipped events.

### Step 2 — If size bound exceeded

Suggest tenant split file:

```bash
oya calendar ics-import suggest-split --job-id <id>
```

### Step 3 — If injection attempt

1. Quarantine the affected job:
   ```bash
   oya calendar ics-import quarantine --job-id <id> --audit-reason "suspected-injection"
   ```
2. Notify ops-security.
3. Capture payload to fuzz-corpus:
   ```bash
   cargo fuzz add corpus/<short-hash> <quarantined-file>
   ```
4. Re-run fuzz suite locally; verify parser rejects the attack.
5. If parser had a vulnerability, patch + emit ADR; bump parser version.
6. Engage ops-security per `incident-response.md` ".ics injection attempt" playbook.

### Step 4 — If vendor extension causes failure

Identify which calendar system emitted the file:
```bash
oya calendar ics-import inspect-headers --job-id <id>
```

Common vendor extensions:
- Google: `X-GOOGLE-*`
- Outlook: `X-MICROSOFT-*`
- Apple: `X-APPLE-*`

Strategy:
- Strict mode (default): reject + log.
- Tenant may opt-in to "permissive mode": strip vendor extensions but import core fields.
- Permissive mode requires tenant audit-chain emission + UI disclosure.

### Step 5 — Tenant communication

```bash
oya calendar ics-import notify-tenant --job-id <id> --status failed --reason "<short>" --suggest "<remediation>"
```

## Recovery validation

| Metric | Target | After mitigation |
|---|---|---|
| `calendar_ics_import_failure_rate` | < 0.1% | within 1h |
| Tenant import success | resolved | per tenant |
| Fuzz corpus expanded | yes | if injection suspected |

## Post-incident review

- Was the parser bound appropriate?
- Was the vendor extension a new pattern? Update parser if needed.
- If injection: emit ADR + threat-model update for T-T-02.
- Update LEAN check `oya-check-ics-parser-conformance` if a new corpus entry was added.

## Drills

- Quarterly: replay fuzz corpus against deployed parser.
- Annual: simulate vendor-extension import; verify strict-mode rejection.

## References

- `failure-modes.md` FM-05.
- `threat-model.md` T-T-02.
- RFC 5545 (iCalendar) + RFC 5546 (iTIP).
- libical conformance corpus.
- `incident-response.md` ".ics injection attempt" playbook.
