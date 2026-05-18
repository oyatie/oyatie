# Runbook — Manual evidence upload rejected (Sev-4)

## Trigger

User reports upload failure.

## Triage

| Symptom | Cause | Resolution |
|---|---|---|
| 403 | Cedar `compliance:admin` capability missing | Add to admin's role binding |
| 413 | File > 100 MB | Ask for redaction + re-upload |
| 415 | MIME-type not allowed | Convert to PDF/JSON/zip |
| 500 | Cosign signing failed | Check Fulcio reachability |

## Cross-references

- IP-014 — manual upload flow.
