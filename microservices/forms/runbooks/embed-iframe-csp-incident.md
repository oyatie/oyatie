---
doc_class: Runbook
title: Embed iframe CSP incident (cross-origin / CSP violation)
microservice: forms
severity: "Sev-2"
status: Accepted
owner_team: ops-security + axis-forms + council-design-system
date: 2026-05-17
related_artifacts:
  - microservices/forms/threat-model.md §"T-I-02" + §"T-I-03"
  - microservices/forms/failure-modes.md FM-05
  - microservices/forms/policy/public-read.cedar
doc_status: published
---

# Runbook: Embed iframe CSP incident

## Purpose

Forms supports tenant-owned-page embedding via iframe + JS widget. CSP misconfiguration on either side (Forms-served frame OR parent page) can expose submitter data, enable XSS, or break legitimate embeds. This runbook covers incident detection + recovery.

## Trigger

ONE of:

1. **`oya_forms_embed_csp_violation_total > 0`** ≥ 5 min.
2. **External report**: tenant reports embed not rendering OR submitter data appearing in unexpected DOM.
3. **`oya_forms_embed_postmessage_origin_mismatch_total > 0`** (parent origin doesn't match tenant allow-list).
4. **`oya_forms_iframe_xss_attempt_total > 0`** — Trusted Types violation in renderer.

## Severity

- CSP violation reports without confirmed exploitation: Sev-3.
- Tenant reports broken legitimate embed: Sev-2.
- Confirmed XSS exploit / data exfiltration: Sev-1 → escalate to `pii-leak-incident-p0.md`.

## Impact

- Embed may fail to render (false positive CSP block).
- Submitter data may be exposed to attacker-controlled parent (true positive XSS).
- Tenant trust impact.

## Pre-checks

1. CSP report ingest: `dashboards/embed-and-distribution.json` panel "CSP violations top-N".
2. Identify the form_id + tenant + parent_origin.
3. Compare parent_origin to tenant's declared allow-list: `cargo run -p oya-dev-cli -- forms embed-allow-list --tenant <id>`.
4. Check Trusted Types policy violations in `oya_forms_iframe_xss_attempt_total`.

## Recovery Path A — Legitimate parent origin missing from allow-list

| Step | Action |
|---|---|
| 1 | Verify tenant intent via gtm-customer-success. |
| 2 | Add parent_origin to tenant allow-list: `cargo run -p oya-dev-cli -- forms embed-allow-list --tenant <id> --add <origin>`. |
| 3 | CSP `frame-ancestors` header re-issued; CDN cache invalidated. |
| 4 | Embed re-renders successfully (within CDN TTL). |
| 5 | Tenant comms. |

## Recovery Path B — Suspected XSS in tenant-authored form label

Cause: tenant authored a form with a label containing HTML that bypasses sanitisation.

| Step | Action |
|---|---|
| 1 | Identify the form + offending field. |
| 2 | Block the form publish: `cargo run -p oya-dev-cli -- forms publish-block --form <id> --reason xss-suspect`. |
| 3 | Tenant comms: their form is paused pending review. |
| 4 | Reproduce the XSS in staging; verify Trusted Types catches it. |
| 5 | If sanitiser bug: open hotfix; deploy; lift block; tenant resumes. |
| 6 | If tenant deliberate: escalate per ToS + abuse policy. |

## Recovery Path C — Confirmed cross-origin data exfiltration

Cause: attacker-controlled parent page embeds Forms iframe; submitter data exfiltrated via postMessage / clickjacking.

| Step | Action |
|---|---|
| 1 | Declare Sev-1; escalate to `runbooks/pii-leak-incident-p0.md`. |
| 2 | Block the affected form publish. |
| 3 | Audit-chain: identify how many submitters affected; subject hashes. |
| 4 | Per-pack regulatory notification (GDPR Art. 33-34; HIPAA §164.404; etc.). |
| 5 | Affected submitters notified directly. |
| 6 | Post-incident: review CSP defaults; tighten Trusted Types policy; pen-test successor-IP. |

## Recovery Path D — CSP report-only mode flood

Cause: tenant operator added overly permissive CSP via config; flood of report-only violations.

| Step | Action |
|---|---|
| 1 | Roll back tenant CSP config to default. |
| 2 | Tenant comms: review CSP policy via builder UI. |
| 3 | Restore report ingest rate-limit if overwhelmed. |

## Invariant: Strict default CSP

Forms-served iframe defaults to:

```
Content-Security-Policy:
  default-src 'self';
  script-src 'self';
  style-src 'self' 'unsafe-inline';   /* tenant theme classes only; sanitised */
  img-src 'self' data: https:;
  connect-src 'self' https://*.oyatie.dev;
  frame-ancestors <tenant allow-list>;
  base-uri 'self';
  form-action 'self';
  require-trusted-types-for 'script';
  trusted-types forms-renderer;
```

Tenant CSP can ONLY narrow (never broaden). `oya-forms-csp-default-baseline` CI lane asserts.

## Verification

After recovery:
- `oya_forms_embed_csp_violation_total` ≤ baseline (per-tenant noise).
- No Trusted Types violations.
- All published forms render in tenant-declared parent origins.

## Post-incident updates

- Postmortem within 5 business days.
- CSP defaults review per `policy/public-read.cedar`.
- Trusted Types policy review.
- If XSS confirmed: pen-test successor-IP + ADR for sanitiser gap.

## References

- `threat-model.md` T-I-02, T-I-03.
- `policy/public-read.cedar`.
- W3C CSP Level 3 — `www.w3.org/TR/CSP3/`.
- W3C Trusted Types — `www.w3.org/TR/trusted-types/`.
- OWASP CSP cheat sheet — `cheatsheetseries.owasp.org/cheatsheets/Content_Security_Policy_Cheat_Sheet.html`.
