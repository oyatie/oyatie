---
purpose: Laptop CAS — GitHub-hosted prelicense proof (Access-TCP; lab tip only)
doc_status: published
---

# Laptop CAS — GitHub-hosted prelicense proof

> **Status:** Active

**Outside** `oya-ci-required`. **Does not** flip `specs/cache-warm-license.json`.

## What it proves

GitHub-hosted runners can reach the founder laptop NativeLink CAS:

`GHA → cloudflared access tcp → cw|cr.oyatie.dev → tunnel TCP → :50151|:50152`

## Standing proof (do not land as new workflow shell)

Rust-first automation (`workflow_inline_shell` + non-Rust `.sh` exception baselines) is **shrink-only**.
A new `.github/workflows/*.yml` with `run:` steps (or new `infra/ci/**/*.sh`) is born-blocking until either:

1. reviewed baseline growth, or
2. Rust/Buck2 productization of those steps.

So the working Access-TCP proof stays on branch tip, not on `dev`:

| Field | Value |
|-------|-------|
| Branch | `lab/laptop-cas-gha-proof` |
| Tip with green REAPI job | `6f03d0977` |
| Green run | https://github.com/jason931225/oyatie/actions/runs/31555198011 |

```bash
# optional: re-run from the lab tip (workflow file exists on that ref only)
gh workflow run laptop-cas-gha-proof.yml --ref lab/laptop-cas-gha-proof
gh workflow run laptop-cas-gha-proof.yml --ref lab/laptop-cas-gha-proof -f run_buck2=true
```

Day-to-day lab canary (preferred):

```bash
~/oyatie-cas/canary/reapi-access-canary.sh
```

## Secrets

Bootstrapped by `~/oyatie-cas/gha/bootstrap-cas-secrets.sh` onto `jason931225/oyatie` and `jason931225/console`.

## Fleet warm

Keep `specs/cache-warm-license.json` → `warm_reads_licensed: false` until a separate reviewed license-flip PR after a green dispatch proof is productized.
