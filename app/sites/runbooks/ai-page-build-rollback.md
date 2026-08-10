---
doc_class: Runbook
template_id: TPL-RUNBOOK
microservice: sites
runbook_id: RB-SITES-AI-PAGE-BUILD-ROLLBACK
severity_class: sev-2
related_adrs: [ADR-SITES-0006]
related_slos: [ai-page-build-latency]
owner_team: axis-sites + council-privacy
date: 2026-05-17
doc_status: published
---

# Runbook: AI-page-build rollback

## Symptom

A T2 AI-page-build action has produced unsafe, hostile, or
non-compliant output, and the user accepted the output before
realising the issue. Or: a T2 AI-page-build call has been confirmed
to be in a refused-context (HR/legal/medical) despite Cedar policy.
Visible as:

- Tenant report: "AI generated discriminatory content / hateful
  language / off-brand text / hallucinated facts."
- `oya_sites_ai_page_build_refused_post_publish_total` increments
  (post-publish safety classifier flag).
- LEAN lane `oya-check-ai-page-build-context-refusal` fires on a CI
  test exposing cross-context drift.
- EU AI Act regulator inquiry (pack-eu).
- Council-privacy / DPO escalation.

## Severity

**Sev-2** by default. **Sev-1** if:
- pack-eu tenant + HR/legal/medical context confirmed in scope.
- Patient-portal page (pack-us-healthcare) with PHI implications.
- Cross-tenant prompt-leak detected.

## First responder

axis-sites on-call. Escalate to council-privacy + ops-security for
EU AI Act / privacy-implication cases.

## Diagnosis

### Step 1 — Identify the AI-page-build run

```bash
cargo run -p oya-dev-cli -- vcs ai-page-build-history \
  --microservice sites \
  --tenant <tenant_id> \
  --site <site_id> \
  --limit 10
```

Look for: `build_id`, `model_id`, `prompt_hash`, `output_hash`,
`accepted_by_user_id`, `accepted_at`, `eu_ai_act_classification`,
`reversibility_window_expires_at`.

### Step 2 — Determine if reversibility window is still open

If `now < reversibility_window_expires_at`, the user can cancel via
in-product banner — no admin action needed.

If window closed, admin reversal is needed.

### Step 3 — Check audit-chain seal for the build

```bash
cargo run -p oya-dev-cli -- audit-chain verify \
  --microservice sites \
  --event AiPageBuildAccepted \
  --build-id <build_id>
```

## Mitigation

### Case A — Reversibility window still open (no admin action)

User can cancel via the in-product banner. No action; document the
event.

### Case B — Window closed; revert page

```bash
# Find the prior-publish version
cargo run -p oya-dev-cli -- vcs page-history \
  --microservice sites \
  --page <page_id> \
  --before <accepted_at>

# Revert
cargo run -p oya-dev-cli -- vcs page-revert \
  --microservice sites \
  --tenant <tenant_id> \
  --page <page_id> \
  --to-version <prior_version> \
  --reason "AI-page-build rollback per RB-SITES-AI-PAGE-BUILD-ROLLBACK; build_id <build_id>"

# Emit audit-chain reversal event
cargo run -p oya-dev-cli -- audit-chain emit \
  --microservice sites \
  --event AiPageBuildPostHocReverted \
  --build-id <build_id> \
  --reverter-user-id <admin_user_id> \
  --reason "<reason>"
```

### Case C — HR/legal/medical-context drift (EU AI Act Annex III §3)

1. Page council-privacy.
2. Identify all T2 builds for the tenant in the suspect context:
   ```bash
   cargo run -p oya-dev-cli -- vcs ai-page-build-context-audit \
     --microservice sites \
     --tenant <tenant_id> \
     --context-overlay hr,legal,medical,employment_decision,credit_decision
   ```
3. Refuse subsequent T2 calls for the tenant until council-privacy
   review:
   ```bash
   cargo run -p oya-dev-cli -- vcs t2-tenant-disable \
     --microservice sites \
     --tenant <tenant_id> \
     --reason "EU AI Act §3 review pending"
   ```
4. Determine if EU AI Act conformity assessment is needed for tenant's
   use case → escalate to council-privacy + external AI compliance firm
   per `compliance.md`.
5. Regulator notification per `incident-response.md` if material.

### Case D — Cross-tenant prompt-leak (Sev-1)

1. **Page ops-security immediately.**
2. Identify the affected build(s):
   ```bash
   cargo run -p oya-dev-cli -- vcs ai-page-build-leak-trace \
     --microservice sites \
     --build-id <build_id>
   ```
3. Disable T2 globally pending root-cause:
   ```bash
   cargo run -p oya-dev-cli -- vcs t2-cell-disable \
     --microservice sites \
     --cell <cell_id> \
     --reason "Sev-1 cross-tenant prompt leak; ops-security"
   ```
4. Open forensic ticket; engage external red-team.
5. Tenant notification per `incident-response.md` 72h chain (GDPR Art. 33).

### Case E — Hostile / discriminatory output (post-classifier flag)

1. Quarantine the build output.
2. Revert the page.
3. Engage post-publish safety classifier review:
   ```bash
   cargo run -p oya-dev-cli -- vcs ai-page-build-safety-review \
     --microservice sites \
     --build-id <build_id>
   ```
4. If pattern emerges (same model_id producing hostile output across
   tenants), engage council-privacy + foundry-runtime for model
   re-evaluation per ADR-SITES-0006.

## Verification

After mitigation:

```bash
# Verify the reverted page-render
curl -sI https://<tenant-domain>/<page-path>

# Verify audit-chain reversal event sealed
cargo run -p oya-dev-cli -- audit-chain verify \
  --microservice sites \
  --event AiPageBuildPostHocReverted \
  --build-id <build_id>

# ai-page-build-latency SLO (if applicable; not safety-bound)
cargo run -p oya-dev-cli -- gate validate slo --microservice sites --slo ai-page-build-latency
```

## Post-incident

- If Case C, file ADR-SITES-XXXX conformity assessment as pending work.
- If Case D, post-mortem + structural fix (model isolation, prompt
  scoping).
- If Case E, evaluate model swap or fine-tune adjustment via
  foundry-runtime.
- Update T2 capability YAML at `capabilities/T2-auto.yaml` if behaviour
  bound needs tightening.

## References

- ADR-SITES-0006 — AI-page-build bounds (EU AI Act).
- EU AI Act Regulation (EU) 2024/1689 — Annex III §3; Arts. 14, 50.
- GDPR Art. 33 (breach notification).
- `microservices/sites/capabilities/T2-auto.yaml`.
- `microservices/sites/incident-response.md`.
- `microservices/sites/policy/editor-isolation.md` Invariant 4.
- `microservices/sites/compliance.md` §"pack-eu" overlay.
