# Threat model — CI Webhook Gateway

STRIDE over the webhook receiver surface. The gateway is internet-adjacent (an
ingress forwards Forgejo deliveries) so the threat surface is the untrusted
inbound webhook.

| Threat | Vector | Mitigation |
|---|---|---|
| **Spoofing** | An attacker POSTs a forged `pull_request` event to trigger a CI run or poison state. | HMAC-SHA256 verification on the RAW body, **fail-closed**, BEFORE any parsing/routing. No valid signature → 401, no dispatch. Constant-time compare (`subtle`) prevents secret recovery via timing. |
| **Tampering** | Payload mutated in transit. | Any single-bit change invalidates the HMAC → 401. |
| **Repudiation** | "I didn't send that delivery." | Every delivery decision emits an audit-chain row keyed by `delivery_id` (ADR-0263). |
| **Information disclosure** | Secret leaks via logs/Debug. | `WebhookSecret` is redacted in `Debug`; the secret is only read from `OYA_FORGEJO_WEBHOOK_SECRET` (injected from OpenBao via `sref`); never written to disk. |
| **Denial of service** | Flood of deliveries / oversized bodies. | Body read is bounded; verification is O(body) and cheap; the dispatch kick is bounded + `Connection: close`. Ingress rate-limit + the per-changeset fan-in cap (ADR-0112 carried-forward) bound runaway loops. |
| **Elevation of privilege** | Use the gateway to reach tenant data. | The gateway has NO tenant data path; Cedar `forbid`s `tenant.data.read` (cedar/policies.cedar). It can only kick the Jenkins lane. |

## Trust boundaries

- **Untrusted**: the inbound HTTP request (until the HMAC verifies).
- **Authenticated**: a delivery whose HMAC matches the OpenBao-held secret.
- **Trusted runner**: Jenkins (re-executes + signs evidence per ADR-0367); the
  gateway trusts neither the author nor its own parse for the merge decision —
  it only KICKS the trusted runner.

## Key rotation

The HMAC secret rotates per SETUP-RUNBOOK.md §"Rotate the webhook secret"
(update Forgejo + OpenBao together). ADR-0112's 90-day rotation TTL guidance
applies; a future `oya-governance-secret-rotation-lifecycle` lane can enforce.
