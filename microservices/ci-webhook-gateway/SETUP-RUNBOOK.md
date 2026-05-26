# SETUP-RUNBOOK — CI Webhook Gateway

This runbook lists **exactly what a human must provision** to make PRs against
`dev` gated by REAL automated checks and to retire the manual admin-relax-merge
seam. Every step that needs human credentials/authority is flagged
**[HUMAN-AUTH]**. Nothing here is auto-applied by the gateway; you run it.

Substrate: git + Jenkins + self-hosted Forgejo (ADR-0363). Design: ADR-0374.

---

## 0. Prerequisites (already standing per `infra/forge/README.md`)

- Self-hosted Forgejo running in namespace `oya-forge` (repo `oya-admin/oyatie`).
- Jenkins running in namespace `oya-ci-jenkins` with the `oyaCiLane` shared
  library (`infra/ci/jenkins/shared-library/vars/oyaCiLane.groovy`).
- The `forgejo-ci-token` Jenkins credential (so Jenkins can POST commit
  statuses) — created per `infra/forge/jenkins-forgejo-token.secret.template.yaml`.

If those are not up, stand them up first; this gateway is the *trigger* that
sits in front of them.

---

## 1. Generate + store the webhook HMAC secret  **[HUMAN-AUTH]**

The gateway verifies `HMAC-SHA256(secret, raw_body)` fail-closed. The SAME secret
must live in two places: Forgejo (which signs) and OpenBao (which the gateway
reads from). They MUST match exactly.

1. Generate a high-entropy secret (32 bytes hex):

   ```
   openssl rand -hex 32
   ```

2. **[HUMAN-AUTH]** Store it in OpenBao at the canonical path (the gateway reads
   it from here via External Secrets Operator → env `OYA_FORGEJO_WEBHOOK_SECRET`):

   ```
   bao kv put secret/oya/ci/forgejo-webhook-secret value='<SECRET_FROM_STEP_1>'
   ```

   The canonical reference is `sref://openbao/oya/ci/forgejo-webhook-secret`
   (ADR-0043 SecretReference contract). Do NOT commit the secret; gitleaks (run
   in the Jenkins lane) blocks it.

3. Confirm the ExternalSecret that projects it into the gateway namespace
   (`oya-ci`) exists and syncs (see §4 deploy).

---

## 2. Register the Forgejo webhook  **[HUMAN-AUTH]**

**[HUMAN-AUTH]** In Forgejo, as a repo admin of `oya-admin/oyatie`:

- Repo → **Settings → Webhooks → Add Webhook → Forgejo (Gitea-compatible)**.
- **Target URL**: the gateway's reachable address, e.g.
  `https://ci-webhook-gateway.<your-domain>/webhook/forgejo`
  (in-cluster: `http://ci-webhook-gateway.oya-ci.svc.cluster.local:8099/webhook/forgejo`).
  Note: Forgejo's `webhook.ALLOWED_HOST_LIST` must permit the gateway host —
  `infra/forge/forgejo.yaml` currently allows `*.cluster.local,oya-ci-jenkins`;
  add the gateway host if you front it with an external URL.
- **HTTP Method**: POST. **Content type**: `application/json`.
- **Secret**: paste the SAME secret from §1.
- **Trigger events**: choose **Custom events → Pull Request** (opened,
  reopened, synchronized). The gateway ignores everything else safely.
- **Active**: checked. Save, then **Test Delivery** — you should get a `200`
  `ignored` (ping) or, on a real PR, a `202` `dispatched`.

The closed router table (`src/event.rs`) only acts on `pull_request` against the
gated branch; a `ping` returns 200, an unknown event returns 422.

---

## 3. Enable commit signing so `required_signatures` is REAL  **[HUMAN-AUTH]**

`infra/branch-protection/dev.json` sets `required_signatures: true`. Today the
relax-merge hack sidesteps this. To satisfy it with REAL signed commits
(Ed25519 per the ADR-0039 signed-commits discipline):

### 3a. Each committer (human + agent identity) generates a signing key **[HUMAN-AUTH]**

```
ssh-keygen -t ed25519 -C "agent-or-human@oyatie" -f ~/.ssh/oya_signing_ed25519
```

Configure git to sign every commit with it:

```
git config --global gpg.format ssh
git config --global user.signingkey ~/.ssh/oya_signing_ed25519.pub
git config --global commit.gpgsign true
git config --global tag.gpgsign true
```

### 3b. **[HUMAN-AUTH]** Register the PUBLIC key in Forgejo

Forgejo → **Settings → SSH / GPG Keys → Add Key → Signing Keys** — paste
`~/.ssh/oya_signing_ed25519.pub`. Forgejo then marks commits signed by that key
as **Verified**, which is what `required_signatures` consults.

### 3c. Confirm

Push a signed commit to a branch and open a test PR; Forgejo should show the
commit as **Verified**. With every commit verified, branch protection's
signed-commit requirement is met **without** disabling `enforce_admins`.

> The gateway removes the *required-checks* half of the relax-merge hack;
> commit signing removes the *signed-commit* half. After both, `dev` merges on
> real signed-green, no admin override.

---

## 4. Deploy the gateway  **[HUMAN-AUTH]**

The gateway is one small stateless pod in namespace `oya-ci`. Provide:

- `OYA_FORGEJO_WEBHOOK_SECRET` — from the ExternalSecret projecting
  `sref://openbao/oya/ci/forgejo-webhook-secret` (§1).
- `OYA_JENKINS_DISPATCH_URL` — the Jenkins endpoint that kicks `oyaCiLane`,
  e.g. the Generic Webhook Trigger or build-token URL:
  `http://jenkins.oya-ci-jenkins.svc.cluster.local:8080/generic-webhook-trigger/invoke?token=<JOB_TOKEN>`.
  **[HUMAN-AUTH]** Create the Jenkins job token (Jenkins → job → Configure →
  Build Triggers → Generic Webhook Trigger / Trigger builds remotely → token).
- `OYA_GATEWAY_TARGET_BRANCH` — defaults to `dev` (override only if gating a
  different branch).
- `OYA_GATEWAY_BIND_ADDR` — defaults to `0.0.0.0:8099`.

Build + run locally to smoke-test:

```
cd microservices/ci-webhook-gateway
cargo build
OYA_FORGEJO_WEBHOOK_SECRET='<secret>' \
OYA_JENKINS_DISPATCH_URL='http://localhost:8080/generic-webhook-trigger/invoke?token=t' \
cargo run
# then: curl localhost:8099/healthz  -> {"status":"ok"}
```

Container/Helm packaging follows the per-microservice `iac/k8s/helm/` convention
(ADR-0349); the deployment must mount the ExternalSecret as the env var above
and expose port 8099 behind a Service (and an ingress if Forgejo posts from
outside the cluster).

> **Liveness/readiness**: probe `GET /healthz`.
> **Fail-closed proof**: if `OYA_FORGEJO_WEBHOOK_SECRET` is unset the gateway
> logs a WARN and rejects every delivery with 503 — it will not accept unsigned
> traffic. Provision the secret before going live.

---

## 5. Rotate the webhook secret  **[HUMAN-AUTH]**

Per the ADR-0112 90-day rotation guidance (carried forward):

1. Generate a new secret (`openssl rand -hex 32`).
2. **[HUMAN-AUTH]** Update OpenBao (`bao kv put …`) AND the Forgejo webhook
   Secret field — **in the same change window** (they must match; a mismatch
   causes 401s until re-synced).
3. Restart/roll the gateway pod so it re-reads the env (or rely on the
   ExternalSecret refresh + pod restart policy).

---

## 6. Cutover checklist (retire the relax-merge seam)  **[HUMAN-AUTH]**

- [ ] Webhook secret in OpenBao + Forgejo (match).
- [ ] Forgejo `pull_request` webhook registered + Test Delivery green.
- [ ] Gateway deployed; `/healthz` green; `OYA_JENKINS_DISPATCH_URL` set.
- [ ] A real test PR against `dev` → gateway 202 → Jenkins runs → all 14
      contexts posted to Forgejo + `oya-pr-review` from the reviewer.
- [ ] Every committer's Ed25519 signing key registered in Forgejo (commits show
      Verified).
- [ ] Forgejo **auto-merge** ("merge when checks succeed") enabled on the repo.
- [ ] Confirm a green PR auto-merges **without** toggling `enforce_admins`.
- [ ] Stop using the admin-merge path (the seam is now retired).

> NOTE — the reviewer gate (Intelligence-service CI stage, ADR-0367 D2) and the
> merge-queue (ADR-0111) are NOT yet built; the gateway returns a typed 501 for
> them and they are tracked in `registry/placeholder-debt/adr-follow-ups.yaml`
> (`adr-0374-reviewer-gate-dispatch`, `adr-0374-merge-queue-admit`). Until the
> reviewer stage exists, `oya-pr-review` is posted by the current reviewer agent
> as it is today; the gateway does not regress that.
