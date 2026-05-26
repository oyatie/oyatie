# Capability catalog — CI Webhook Gateway

| Capability | Description | Surface |
|---|---|---|
| `webhook.receive` | Receive Forgejo/Gitea/GitHub webhook deliveries over HTTP. | `POST /webhook/forgejo` |
| `webhook.verify-hmac` | Verify the HMAC-SHA256 signature on the raw body, fail-closed, constant-time. | `signature.rs` |
| `pr-event.parse` | Parse `pull_request` events (opened/reopened/synchronized) against the gated branch. | `event.rs` |
| `pipeline.dispatch` | Kick the Jenkins `oyaCiLane` pipeline (admission → `oya gate run-all`). | `dispatch.rs` |
| `boundary.report` | Report the typed `Unimplemented` boundary for not-yet-built downstream stages. | `error.rs` |
| `liveness.probe` | Expose a `/healthz` endpoint for k8s probes. | `GET /healthz` |

## Capability boundaries

- The gateway does NOT post commit statuses (Jenkins does, per
  `oyaCiLane.groovy`).
- The gateway does NOT run governance gates (Jenkins, the trusted runner, does
  — ADR-0367).
- The gateway does NOT review code (the Intelligence-service reviewer gate does
  — ADR-0367 D2; not yet wired, tracked as placeholder-debt).
