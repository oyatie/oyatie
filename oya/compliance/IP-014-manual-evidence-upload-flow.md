---
microservice: compliance
ip: IP-014
title: Manual evidence upload flow (pen-test reports + BAA inventory + signed artifacts)
status: Drafting
authority_tier: 3
owner: axis-compliance
co_owners: [axis-security]
date: 2026-05-18
related_adrs: [ADR-0181, ADR-0183, ADR-0207, ADR-0209]
---

# IP-014 — Manual evidence upload flow

## Purpose

Some evidence kinds cannot be auto-collected:

- Pen-test reports (annual external engagement; PDF).
- BAA inventory entries (legal review-gated; signed PDFs).
- Vendor-supplied compliance attestations (SOC 2 reports from sub-processors).
- Manual policy attestations (workforce training completion certificates).

Provide a Cedar-gated upload flow with cosign seal on upload, MIME-type allowlist, and audit-chain integration.

## Acceptance criteria

1. `POST /api/v1/evidence/manual-upload` accepts multipart form: artifact_kind + framework + file payload + metadata.
2. Cedar `compliance:admin` capability required.
3. MIME-type allowlist: PDF, JSON, signed-PDF (PAdES), zip-of-signed-PDFs.
4. File size cap: 100 MB.
5. SHA-256 + cosign seal applied on receipt.
6. UI: Backstage upload panel; WCAG 2.2 AA.
7. ≥ 5 integration tests: upload-happy-path + non-admin-rejected + oversize-rejected + non-allowed-mime-rejected + seal-emitted.

## Upload payload

```json
{
  "artifact_kind": "pen-test-report",
  "framework": "soc2-type-2",
  "tenant_id": "tenant_a",
  "metadata": {
    "engagement_id": "pentest-2026-q2",
    "vendor": "Synack",
    "report_date": "2026-04-15"
  },
  "file_payload": "<multipart-base64>"
}
```

## A11y

Per WCAG 2.2 AA. File-input has accessible label; progress bar has `aria-live`. Error messages programmatically associated.

## Risk + mitigation

- **Risk:** uploader uploads bogus pen-test report. **Mitigation:** Cedar `compliance:admin` role bound to known-identity workforce; signed PDF preferred.
- **Risk:** uploaded report contains exploitable secrets. **Mitigation:** PDF redaction tool integrated; manual review required for unredacted uploads.

## Acceptance evidence

`evidence/ip-014-manual-evidence-upload-flow-acceptance.json`.

## Cross-references

- ADR-0181 — image promotion (cosign).
- ADR-0183 — Cedar.
- ADR-0207 — a11y.
- ADR-0209 — substrate authority.
- IP-005 — audit-chain seal.
