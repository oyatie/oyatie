---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: slides
runbook_id: theme-corruption
status: Accepted
severity: Sev-2 (Sev-1 if signing key compromise)
date: 2026-05-17
owner_team: axis-workspace + ops-security
related_artifacts:
  - microservices/slides/failure-modes.md FM-29
  - microservices/slides/threat-model.md T-T-05, T-SC-09
doc_status: published
---

# Runbook — Theme / template signed-bundle corruption

## When to use

- Tenant reports broken theme/template rendering.
- Ed25519 signature verification failure at slides-rest theme/template load.
- Signing key compromise alarm.

## Symptom triage

| Symptom | Likely cause | Run |
|---|---|---|
| Single theme fails | Bundle corruption OR signature mismatch | step 1 |
| All themes per pack fail | Per-pack signing key issue | step 2 |
| Sev-1 key-compromise alarm | External CA / OpenBao compromise | step 3 |

## Step 1 — Single-theme corruption

```bash
THEME_ID=<theme_id>

# Verify signature
oya vcs --service slides --action theme-verify --theme-id $THEME_ID

# If signature verification fails on a recently-uploaded theme: refuse + tenant notify; re-upload
oya vcs --service slides --action theme-revoke --theme-id $THEME_ID --reason "signature-verification-failed"

# CDN purge to prevent cache-serve of bad bundle
oya vcs --service slides --action cdn-purge-theme --theme-id $THEME_ID
```

## Step 2 — Per-pack signing key issue

```bash
PACK=<pack>

# Inspect signing key state
oya vcs --service slides --action theme-signing-key-status --pack $PACK

# If key rotated incorrectly: rebuild signed bundles
oya vcs --service slides --action theme-resign --pack $PACK --bulk
```

## Step 3 — Signing key compromise (Sev-1)

Per `threat-model.md` T-SC-09 + `failure-modes.md` FM-29.

```bash
PACK=<pack>

# Revoke compromised key immediately
oya vcs --service slides --action signing-key-revoke --pack $PACK --key-id <compromised_key_id>

# Distribute revocation list via CDN
oya vcs --service slides --action crl-distribute --pack $PACK

# Rotate to new key (OpenBao Transit + KMS-backed)
oya vcs --service slides --action signing-key-rotate --pack $PACK

# Re-sign all themes/templates with new key
oya vcs --service slides --action theme-resign --pack $PACK --bulk
oya vcs --service slides --action template-resign --pack $PACK --bulk

# Audit
oya vcs --service slides --action audit-tail --kind signing_key_rotation --since 1h
```

Tenant notification + DPO + legal escalation.

## Re-enable

```bash
# Health verify
oya vcs --service slides --action theme-health --pack $PACK
oya vcs --service slides --action template-health --pack $PACK
```

## Verification

- Theme/template signature verification success rate > 99.9%.
- Revocation list propagated to all CDN edges < 60s.
- Audit-chain seal of incident emitted.

## References

- threat-model.md T-T-05, T-SC-09.
- failure-modes.md FM-29.
- workflow-studio analogous "template-marketplace-quarantine" runbook.
