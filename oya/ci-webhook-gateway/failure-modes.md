# Failure modes — CI Webhook Gateway

| Failure mode | Effect | Detection | Recovery |
|---|---|---|---|
| Webhook secret unset/empty | Every delivery fails closed (503). No PRs gated. | Startup `WARN` log; 503 deliveries in GitHub. | Provision `OYA_GITHUB_WEBHOOK_SECRET` (SETUP-RUNBOOK). |
| Secret divergence (GitHub vs OpenBao) | All deliveries 401. | 401 deliveries; `signature rejected` logs. | Re-sync the two secrets (SETUP-RUNBOOK §rotate). |
| Jenkins dispatch URL unset | Dispatch returns typed 502; no silent success. | Startup `WARN`; 502 on PR deliveries. | Set `OYA_JENKINS_DISPATCH_URL`. |
| Jenkins unreachable | Dispatch 502 (`connector`/`write`/`read` transport error). | 502 + `dispatch failed` logs. | Restore Jenkins reachability (ADR-0349 farm). |
| Malformed payload | 400 for that delivery; others unaffected. | 400 + `MalformedPayload` log. | Usually a sender bug; inspect the payload. |
| Unknown event class | 422 (logged, not dropped). | 422 + `unroutable` log. | Amend the closed router table via ADR if the event must be gated. |
| Gateway crash mid-delivery | GitHub redelivers (at-least-once). The kick is idempotent by `(pr, head_sha)`; a duplicate kick re-runs CI harmlessly. | Pod restart; redelivery. | Automatic; the head_sha keys the kick. |
| Reviewer-gate / merge-queue not built | 501 `unimplemented` if reached. | 501 + `placeholder_debt` token. | EXPECTED until the downstream is built (`adr-0374-*`). |

## Fault-isolation posture

The gateway holds **zero durable state** — it verifies, routes, and kicks. A
crash loses nothing; GitHub's at-least-once redelivery + the head_sha-keyed
idempotent kick make restart-replay safe. This mirrors the ADR-0113 "orchestrator
owns zero state" posture carried forward to the GitHub substrate.
