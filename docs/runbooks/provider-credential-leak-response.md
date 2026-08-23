---
purpose: Oyatie Runbook — Leaked Platform-Default Provider Credentials Response
doc_status: published
---

# Oyatie Runbook — Leaked Platform-Default Provider Credentials Response

> **Status:** Active
> **Owner:** ops-security + council-security + ops-compliance
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0255 §D-4, ADR-0244 §D-3, ADR-0251 §D-8, ADR-0243

---

## §A Trigger Conditions

This runbook covers the leak of **platform-default provider credentials** — the API keys, subscription tokens, or service-account credentials that oyatie manages on behalf of tenants with `provider_credential_mode = 'platform_default'` (per ADR-0255 §D-4).

**Critical scoping note (ADR-0255 §D-4 + synthesis §4):** Platform-default credentials are used **only** for tenants who have not opted into provider-credential BYOK. Tenants with `provider_credential_mode = 'tenant-byok'` or `'tenant-subscription'` maintain their own provider credentials and are **isolated by design** — a platform-default credential leak does not affect them. This is a core isolation property of the SecretReference architecture.

Initiate when:

- **API key found in public repository** — platform-default Anthropic/OpenAI/Google key found in a public GitHub repo, paste site, or security scanner report.
- **Provider alerts unusual spend** — Anthropic/OpenAI/Google billing alert indicates requests far exceeding expected usage, suggesting credential misuse.
- **Security researcher disclosure** — responsible disclosure of a leaked platform-default credential.
- **Internal secret scanner alert** — `oyatie.foundry.security-scan` workflow (`trivy`, `grype`, `cosign-verify`, `cargo-audit` per ADR-0247 §D-2) reports a credential in a log file, artifact, or repository.
- **Provider breach notification** — the provider (Anthropic, OpenAI, Google, etc.) notifies oyatie of suspected key compromise.

**For provider-credential BYOK tenant credential leaks (ADR-0255 §D-4)**, use `docs/runbooks/byok-rotation-provider-tenant-duress.md` instead.

---

## §B Pre-Checks

Estimated time: **5 min**. Speed is critical — provider APIs are billed per-token and adversarial usage accumulates quickly.

1. **Identify the leaked credential(s).** Determine the provider and the `secret_ref_id`:
   ```
   psql -c "SELECT secret_ref_id, provider_type, openbao_path, created_at, last_rotated_at
     FROM secret_references
     WHERE owner_kind = 'platform-default'
       AND status = 'ACTIVE'
       AND provider_type = '<PROVIDER>';"
   ```

2. **Identify scope of tenant exposure.** List all tenants using platform-default credentials for this provider:
   ```
   psql -c "SELECT tenant_id, home_cell
     FROM tenants
     WHERE provider_credential_mode = 'platform_default';"
   ```
   Record `PLATFORM_DEFAULT_TENANT_COUNT`. provider-credential BYOK tenants are NOT affected (ADR-0255 §D-4; verify with `COUNT(*) WHERE provider_credential_mode IN ('tenant-byok','tenant-subscription')`).

3. **Pull recent provider usage.** Call provider billing API to get usage in the last 24h:
   - Anthropic: `GET https://api.anthropic.com/v1/usage?start_time=<24H_AGO>`
   - OpenAI: `GET https://api.openai.com/v1/usage?date=<TODAY>`
   - Google: Cloud Billing API `projects.billingAccounts.skus.list`

   Compare against expected baseline. Flag any delta >20% as anomalous.

4. **Declare incident.** SEV-1. Notify `council-security`, `ops-security`, `ops-compliance`. If anomalous usage suggests active exploitation, activate rate-limit fence (Step 1) immediately without waiting to complete all pre-checks.

---

## §C Procedure

### Step 1 — Rate-limit fence: throttle all platform-default inference (target: ≤60s)

Install a Cedar fragment that rate-limits LLM dispatch for all `platform_default` tenants to an emergency-minimum rate. This bounds ongoing cost during the incident while maintaining service:

```
cat > /tmp/platform-default-rate-limit-<INCIDENT_ID>.cedar << 'EOF'
// EMERGENCY: platform-default rate-limit fence
// EXPIRES: <ISO8601 +4h>
forbid (
  principal,
  action == Intelligence::Action::DispatchLLMCall,
  resource
)
when {
  context.provider_credential_mode == "platform_default"
  && context.calls_in_window_60s > 2
};
EOF

policy-engine-cli fragment publish \
  --fragment-path /tmp/platform-default-rate-limit-<INCIDENT_ID>.cedar \
  --scope "baseline/emergency-rate-limit-<INCIDENT_ID>" \
  --ttl-seconds 14400 \
  --operator oyatie.council-security.<operator-id>
```

Wait for propagation (≤30s per ADR-0243 §D-10).

### Step 2 — Provider-side emergency revocation (target: ≤10 min)

Contact the provider's security/support channel to revoke the leaked key immediately:

**Anthropic:**
```
# Via Anthropic Console (GUI: API Keys → Revoke) OR emergency security contact:
# security@anthropic.com with subject "Emergency Key Revocation"
# Include: key last-4 digits, account ID, incident reference
```

**OpenAI:**
```
# Via OpenAI Platform (GUI: API Keys → Delete) OR:
curl -X DELETE "https://api.openai.com/v1/organization/api_keys/<KEY_ID>" \
  -H "Authorization: Bearer <MGMT_KEY>"
```

**Google (Gemini / Vertex AI):**
```
gcloud iam service-accounts keys delete <KEY_ID> \
  --iam-account=<SERVICE_ACCOUNT_EMAIL>
```

**AWS Bedrock:**
```
aws iam delete-access-key --access-key-id <OLD_KEY_ID> \
  --user-name <BEDROCK_IAM_USER>
```

Document the revocation confirmation (API response or screenshot) in `evidence/incidents/<INCIDENT_ID>/`.

### Step 3 — Generate and deploy replacement credentials (target: ≤20 min)

Create new platform-default credentials at the provider, store in OpenBao, and create new SecretReference records:

```
# Create new API key at provider (via provider console or API)
# Store new secret in OpenBao:
vault kv put secret/platform/providers/<PROVIDER>/default \
  api_key="<NEW_API_KEY>" \
  created_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  rotation_incident="<INCIDENT_ID>"

# Create new SecretReference record:
psql -c "INSERT INTO secret_references
  (tenant_id, provider_type, owner_kind, openbao_path, openbao_kv_version, status, created_at)
  VALUES ('oyatie', '<PROVIDER>', 'platform-default', 'secret/platform/providers/<PROVIDER>/default',
          1, 'ACTIVE', now())
  RETURNING secret_ref_id;" AS NEW_SECRET_REF_ID

# Supersede old record:
psql -c "UPDATE secret_references SET status = 'SUPERSEDED', superseded_at = now()
  WHERE secret_ref_id = '<OLD_SECRET_REF_ID>';"
```

Emit:
```
audit-emit PlatformProviderCredentialRotated \
  --provider <PROVIDER> \
  --old-secret-ref-id <OLD_SECRET_REF_ID> \
  --new-secret-ref-id <NEW_SECRET_REF_ID> \
  --rotation-reason "credential-leak" \
  --incident-ref <INCIDENT_ID> \
  --operator oyatie.ops-security.<operator-id>
```

### Step 4 — Audit-row replay to detect misuse (target: ≤60 min; may run in background)

Run a replay of the audit stream to identify any LLM calls made during the exposure window that originated from the leaked credential. This determines whether PHI, PII, or other sensitive data was sent to the provider via the compromised key:

```
audit-chain-cli replay \
  --stream-class "intelligence.llm-dispatch" \
  --secret-ref-id <OLD_SECRET_REF_ID> \
  --window-start "<CREDENTIAL_CREATED_AT>" \
  --window-end "<REVOCATION_CONFIRMED_AT>" \
  --output /tmp/misuse-report-<INCIDENT_ID>.json
```

Analyze the report:
```
# Calls matching known-good patterns (oyatie.foundry.* workflows, expected tenant IDs):
jq '[.calls[] | select(.tenant_id as $t | ["expected-tenant-1","expected-tenant-2"] | index($t))] | length' \
  /tmp/misuse-report-<INCIDENT_ID>.json

# Anomalous calls (unexpected tenants, unexpected principals, unusual token counts):
jq '[.calls[] | select(.anomaly_score > 0.8)] | length' \
  /tmp/misuse-report-<INCIDENT_ID>.json
```

### Step 5 — Tenant impact assessment (target: ≤30 min)

Assess impact on `platform_default` tenants:

1. **provider-credential BYOK tenants are confirmed unaffected (ADR-0255 §D-4).** Document this explicitly:
   ```
   psql -c "SELECT COUNT(*) FROM tenants WHERE provider_credential_mode IN ('tenant-byok','tenant-subscription');"
   ```
   These tenants' calls used their own SecretReferences, not the platform-default. Record the count in the incident report.

2. **For platform_default tenants:** determine whether any calls in the misuse report involved regulated data classes (PHI, PII, financial data) by cross-referencing with the tenant's installed compliance packs:
   ```
   psql -c "SELECT t.tenant_id, array_agg(tcp.pack_id) as packs
     FROM tenants t
     JOIN tenant_compliance_packs tcp ON t.tenant_id = tcp.tenant_id
     WHERE t.provider_credential_mode = 'platform_default'
       AND tcp.status = 'ACTIVE'
     GROUP BY t.tenant_id;"
   ```

3. **If regulated data may have been in the leaked-credential calls:** immediately escalate to `council-legal` and `council-privacy` to assess breach notification obligation per ADR-0251 §D-8.

### Step 6 — Breach notification assessment (ADR-0251 §D-8)

If the audit replay confirms that calls under the leaked credential involved protected data:

1. Trigger the breach-notification workflow for each affected tenant's compliance pack:
   ```
   workflow-cli start oyatie.foundry.breach-notification \
     --affected-tenant-ids <TENANT_IDS_WITH_REGULATED_PACKS> \
     --trigger provider-credential-leak \
     --incident-ref <INCIDENT_ID>
   ```

2. Escalate to `docs/runbooks/breach-notification-council-escalation.md` for regulator communication.

**EU GDPR tenants:** 72h deadline from knowledge of breach (Article 33).
**KR-PIPA tenants:** 24h deadline (Article 34).
**HIPAA tenants:** 60-day deadline (§164.404); initial contact within 72h recommended.

### Step 7 — Remove rate-limit fence and restore service (target: ≤5 min)

Once the new credential is active and verified:

```
policy-engine-cli fragment deactivate \
  --scope "baseline/emergency-rate-limit-<INCIDENT_ID>" \
  --operator oyatie.council-security.<operator-id>
```

Verify LLM dispatch resumes at normal rates:
```
sleep 30
curl -s http://intelligence-metrics.<CELL_ID>/metrics | grep "llm_call_success_rate"
```

---

## §D Verification

1. **Old credential returns 401 from provider:**
   ```
   curl -H "x-api-key: <OLD_KEY>" https://api.anthropic.com/v1/messages \
     -d '{"model":"claude-3-haiku-20240307","max_tokens":1,"messages":[{"role":"user","content":"test"}]}'
   # Must return {"error":{"type":"authentication_error"}}
   ```

2. **New credential is ACTIVE and validates:**
   ```
   microservices/intelligence/bin/credential-probe \
     --secret-ref-id <NEW_SECRET_REF_ID> --live-test
   ```

3. **Rate-limit fence is inactive:**
   ```
   policy-engine-cli fragment status --scope "baseline/emergency-rate-limit-<INCIDENT_ID>"
   ```

4. **Audit trail complete:** Verify `PlatformProviderCredentialRotated` event with Merkle proof.

5. **Misuse report filed** in `evidence/incidents/<INCIDENT_ID>/misuse-report.json`.

---

## §E Rollback

There is no rollback for provider-side credential revocation — once revoked at the provider, the old key is gone. If the new credential fails (Step 3), create another new credential and retry.

If the rate-limit fence (Step 1) causes unacceptable service degradation before the new credential is ready, increase the per-window threshold in the fragment:
```
policy-engine-cli fragment update \
  --scope "baseline/emergency-rate-limit-<INCIDENT_ID>" \
  --new-threshold 10
```

---

## §F Post-Incident

1. Root-cause analysis: how was the platform-default credential exposed? (Log file, build artifact, repository scan miss, provider account misconfiguration.)
2. Update `oyatie.foundry.security-scan` workflow with detection rules for the exposure vector.
3. Assess whether platform-default credentials should be rotated on a scheduled cadence (quarterly recommended; monthly for high-traffic providers).
4. If misuse report shows anomalous calls from unexpected principals: determine if the leak was internal (insider threat) or external.
5. Update provider-credential BYOK adoption metrics — this incident motivates accelerating `provider_credential_mode = 'tenant-byok'` adoption for regulated tenants (per ADR-0255 §D-4 `provider_byok_required` flag on relevant compliance packs).
6. Post-mortem within 72h.

---

## §G References

- ADR-0255 §D-4 (provider-credential BYOK; `provider_credential_mode`; SecretReference)
- ADR-0244 §D-3 (`provider_credential_mode` distinct from `byok_enabled`)
- ADR-0251 §D-8 (Breach notification workflow)
- ADR-0243 §D-10 (Hot-reload for rate-limit fence)
- `docs/runbooks/byok-rotation-provider-tenant-duress.md`
- `docs/runbooks/breach-notification-council-escalation.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
