# Laptop CAS — GitHub-hosted prelicense proof

**Outside** `oya-ci-required`. **Does not** flip `specs/cache-warm-license.json`.

## What it proves

GitHub-hosted runners can reach the founder laptop NativeLink CAS:

`GHA → cloudflared access tcp → cw|cr.oyatie.dev → tunnel TCP → :50151|:50152`

## Run

```bash
gh workflow run laptop-cas-gha-proof.yml --ref <branch>
# optional Buck2 smoke:
gh workflow run laptop-cas-gha-proof.yml --ref <branch> -f run_buck2=true
```

## Secrets

Bootstrapped by `~/oyatie-cas/gha/bootstrap-cas-secrets.sh` onto `jason931225/oyatie`.

## Local scripts

| Script | Role |
|--------|------|
| `infra/ci/laptop-cas/start-access-tcp.sh` | Docker Access TCP forwarders on `:55051`/`:55052` |
| `infra/ci/laptop-cas/reapi-proof.sh` | GetCapabilities + FindMissingBlobs |
| `infra/ci/laptop-cas/stop-access-tcp.sh` | Cleanup |

Overlays: `infra/ci/buckconfig/laptop-cas-github-{rw,ro}.buckconfig`.
