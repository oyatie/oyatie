# Provision Secrets — CI Webhook Gateway

Operator runbook for provisioning the secrets required by the ci-webhook-gateway
before ArgoCD syncs the Helm chart. All steps require human credentials.
Substrate: OpenBao + ESO + GitHub (ADR-0387, ADR-0363, ADR-0374).

---

## 1. Generate the Ed25519 keypair for GitHub webhook signing  **[HUMAN-AUTH]**

The gateway verifies GitHub webhook deliveries using an Ed25519 public key.
Generate a dedicated keypair for this purpose (do not reuse the committer signing
key from §3 of SETUP-RUNBOOK.md):

```sh
ssh-keygen -t ed25519 -C "ci-webhook-gateway@oyatie" \
  -f ~/.ssh/ci_webhook_github_ed25519 -N ""
# Outputs:
#   ~/.ssh/ci_webhook_github_ed25519       (PRIVATE key — keep secret)
#   ~/.ssh/ci_webhook_github_ed25519.pub   (public key — store in OpenBao + GitHub)
```

Note the public key value:

```sh
cat ~/.ssh/ci_webhook_github_ed25519.pub
# e.g.: ssh-ed25519 AAAA... ci-webhook-gateway@oyatie
```

---

## 2. Provision all three secrets into OpenBao  **[HUMAN-AUTH]**

The ExternalSecret in the Helm chart pulls these three keys from a single KV
path. Provision them in one `vault kv put` call so ESO sees a consistent
secret version:

```sh
vault kv put secret/oya/ci-webhook-gateway \
  github_ed25519_pub="$(cat ~/.ssh/ci_webhook_github_ed25519.pub)" \
  jenkins_api_token="<JENKINS_API_TOKEN>" \
  github_token="<GH_TOKEN>"
```

- `github_ed25519_pub`: the full public key string from step 1 (including the
  `ssh-ed25519 AAAA...` prefix).
- `jenkins_api_token`: a Jenkins API token scoped to the `ci-gate` job.
  Generate via Jenkins UI → user → Configure → API Token → Add new token.
- `github_token`: a GitHub PAT (or fine-grained token) with
  `repo:status` write scope for `admin/oyatie` (used to post commit statuses).

Confirm the write:

```sh
vault kv get secret/oya/ci-webhook-gateway
```

---

## 3. Configure the GitHub webhook  **[HUMAN-AUTH]**

In GitHub, as a repo admin of `admin/oyatie`:

- Repo → **Settings → Webhooks → Add Webhook → GitHub (Gitea-compatible)**.
- **Target URL**: `https://ci-webhook-gateway.oyatie.com/webhook/github`
  (or in-cluster: `http://ci-webhook-gateway.ci-webhook-gateway.svc.cluster.local:8081/webhook/github`).
- **HTTP Method**: POST. **Content type**: `application/json`.
- **Signing key type**: Ed25519. **Public key**: paste the content of
  `~/.ssh/ci_webhook_github_ed25519.pub` (matching what you stored in OpenBao).
  The gateway reads `OYATIE_CI_WEBHOOK_GITHUB_ED25519_PUB` from the ESO secret
  and verifies the `X-GitHub-Signature` header on every delivery.
- **Trigger events**: Custom events → **Pull Request** (opened, reopened,
  synchronized). The gateway ignores all other event types.
- **Active**: checked. Save, then **Test Delivery** — expect `200 ignored`
  (ping) or `202 dispatched` on a real PR.

---

## 4. Verify end-to-end  **[HUMAN-AUTH]**

1. Open a test PR against `dev` in GitHub.
2. GitHub webhook delivery shows `202` in the GitHub delivery log.
3. The ci-webhook-gateway pod logs show `signature: verified` and
   `dispatched: ci-gate`.
4. Jenkins shows the `ci-gate` build triggered with the correct
   `PR_NUMBER`, `PR_SHA`, `REPO_OWNER`, `REPO_NAME` variables.
5. On a green gate run, the GitHub commit status for the PR SHA shows
   `ci-gate: success`.
6. GitHub auto-merge (if enabled) merges the PR on green without admin override.

If step 2 shows a signature error, confirm:
- The public key stored in OpenBao matches the private key used by GitHub.
- The ESO ExternalSecret has synced (`kubectl get externalsecret -n ci-webhook-gateway`).
- The pod has been restarted after the secret sync.

---

## 5. Secret rotation  **[HUMAN-AUTH]**

Ed25519 keypair rotation (recommended annually or on suspected compromise):

1. Generate a new keypair (step 1 above).
2. Update OpenBao with the new public key: `vault kv patch secret/oya/ci-webhook-gateway github_ed25519_pub="<NEW_PUB>"`.
3. Update the GitHub webhook signing key to the new public key — do this
   **in the same change window** as the OpenBao update to minimise signature
   verification failures.
4. Wait for ESO to refresh (default: 1h; force via
   `kubectl annotate externalsecret ci-webhook-gateway-secrets -n ci-webhook-gateway force-sync=$(date +%s)`).
5. Confirm delivery succeeds with the new keypair via Test Delivery in GitHub.

Jenkins API token and GitHub token rotation follow the same pattern:
`vault kv patch secret/oya/ci-webhook-gateway jenkins_api_token="<NEW>"` or
`github_token="<NEW>"`, then force ESO refresh.
