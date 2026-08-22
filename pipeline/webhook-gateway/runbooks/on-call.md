# Runbook — CI Webhook Gateway on-call

## What this service does

Receives GitHub webhooks, verifies the HMAC, and kicks the Jenkins `oyaCiLane`
pipeline for PRs against `dev`. If it is down, PRs against `dev` stop being
gated automatically (no CI kicks) — do NOT fall back to admin-relax-merge; fix
the gateway.

## Symptoms → actions

### PRs against dev are not getting CI runs

1. Check the gateway is up: `kubectl -n ci get pods -l app=ci-webhook-gateway`
   and `curl http://<pod>:8099/healthz`.
2. Check GitHub webhook deliveries: GitHub repo → Settings → Webhooks →
   Recent Deliveries. A red delivery shows the response code:
   - **401** — signature mismatch. The webhook secret in GitHub and the
     `OYATIE_GITHUB_WEBHOOK_SECRET` (from `sref://openbao/oya/ci/github-webhook-secret`)
     have diverged. Re-sync per SETUP-RUNBOOK.md §"Rotate the webhook secret".
   - **422** — unroutable event. GitHub is sending an event we don't gate;
     usually harmless. If a NEW event class must be gated, amend `event.rs`'s
     router table (closed set; ADR amendment required).
   - **502** — Jenkins dispatch failed. Check `OYATIE_JENKINS_DISPATCH_URL` and
     that Jenkins is reachable from the gateway pod.
   - **503** — webhook secret unavailable. The OpenBao/ESO injection failed;
     check the `github-webhook-secret` Secret exists in the namespace.
3. Check gateway logs (JSON): `kubectl -n ci logs -l app=ci-webhook-gateway`.
   Each delivery logs `delivery`, `outcome`, and (on dispatch) `pr`/`sha`.

### A delivery is stuck redelivering

GitHub retries failed deliveries. A persistent 401/503 will redeliver. Fix the
secret (401) or the injection (503); GitHub will succeed on the next retry. A
200/202/422 stops redelivery.

### Suspected webhook spoofing

Any unsigned/badly-signed delivery is rejected with 401 BEFORE parsing
(fail-closed). Confirm in logs (`webhook signature rejected`). The HMAC secret
is the authenticator; rotate it (SETUP-RUNBOOK §rotate) if you suspect leak.

## Escalation

- Gateway crashloop / bind failure → ops-platform.
- Repeated dispatch 502 with Jenkins healthy → CI-farm owner (ADR-0349).
- Reviewer-gate / merge-queue "501 unimplemented" responses are EXPECTED until
  those downstreams are built (placeholder-debt `adr-0374-*`); not an incident.
