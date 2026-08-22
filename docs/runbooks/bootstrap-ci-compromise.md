---
purpose: Oyatie Runbook — Stage-1 Bootstrap CI Runner Compromise
doc_status: published
---

# Oyatie Runbook — Stage-1 Bootstrap CI Runner Compromise

> **Status:** Active
> **Owner:** council-security + ops-sre-reliability
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during `oya verify` gate repair sweep)
> **Related ADRs:** ADR-0247 §D-5, ADR-0247 §D-4, ADR-0243 §D-5, ADR-0248 §D-2

---

## §A Trigger Conditions

This runbook applies **exclusively during the Stage-1 bootstrap window** (ADR-0247 §D-5, Stage 1): the ≤8h period between Stage 0.5 (bootstrap-replay log initialised) and Stage 2.0 (first Foundry-equivalent workflow running on the bootstrap cell). During this window, an external CI runner (GitHub Actions, temporary CircleCI, or temporary self-hosted runner) has elevated trust to deploy cosign-attested artifacts.

Per synthesis §5.3 (F5-247-02 fix), Stage-1 external CI runner identity is bound via SPIFFE workload identity issued by a one-shot offline-rooted CA, with cosign-attested attestations for every Stage-1 artifact, and an explicit ≤8h kill-switch.

Initiate when:

- **Stage-1 CI runner identity is compromised** — SPIFFE workload identity certificate for the Stage-1 runner is found on an unexpected host, or a runner is observed deploying artifacts not signed by the org root key.
- **Artifact attestation failure** — a Stage-1 artifact's cosign signature cannot be verified against the org root key chain (ADR-0243 §D-5).
- **Runner exfiltration detected** — evidence of secret exfiltration from the Stage-1 runner environment (e.g., OpenBao Shamir-share exposure, registry push credentials).
- **8h kill-switch fires automatically** — the Cedar fragment `pack-bootstrap-kill-switch` disables Stage-1 trust roots at T+8h regardless of Stage-2 readiness. This runbook covers the aftermath of an unexpected kill-switch firing that interrupted the bootstrap.
- **Anomalous artifact push** — container registry receives a push during Stage-1 with a signing chain that does not trace to the org root key.

**Important:** This runbook only applies during the bootstrap window. Post-bootstrap (Stage 2+), use `docs/runbooks/self-modification-rollback.md` for deployment compromise.

---

## §B Pre-Checks

Estimated time: **5 min** (act quickly — Stage-1 is time-bounded).

1. **Confirm bootstrap stage.** Verify the system is still in Stage-1:
   ```
   psql -c "SELECT stage, stage_started_at, stage_deadline_at, kill_switch_fired_at
     FROM bootstrap_state WHERE bootstrap_id = 'current';"
   ```
   If `stage` = `STAGE_2` or later, this runbook does not apply.

2. **Assess elapsed time.** Calculate time remaining in the 8h window:
   ```
   psql -c "SELECT stage_deadline_at - now() AS time_remaining FROM bootstrap_state;"
   ```
   If <30 min remaining, the kill-switch will fire automatically soon — proceed directly to §C Step 1 to fire it manually now.

3. **Identify the compromised runner.** List active Stage-1 SPIFFE identities:
   ```
   spire-cli bundle show | grep "stage-1"
   spire-cli entry show --selector "stage:bootstrap-stage-1"
   ```

4. **Identify which artifacts were produced by the compromised runner:**
   ```
   # Check container registry push logs for the bootstrap session:
   crane ls <REGISTRY>/<ORG>/bootstrap-artifacts | \
     xargs -I{} crane manifest <REGISTRY>/<ORG>/bootstrap-artifacts:{} | \
     jq '.annotations["dev.sigstore.cosign/bundle"]'
   ```
   Any artifact with a signature chain not tracing to `org-root-key-<TIMESTAMP>` is suspect.

5. **Declare incident.** SEV-1. Immediately notify `council-security` and the founding team. Bootstrap compromise has the potential to undermine the entire platform's trust chain.

---

## §C Procedure

### Step 1 — Activate kill-switch: disable Stage-1 trust roots (target: ≤60s)

The kill-switch is a Cedar fragment that disables Stage-1 SPIFFE trust roots at T+8h or on manual activation. Activate it immediately:

```
policy-engine-cli fragment activate \
  --fragment-id "pack-bootstrap-kill-switch" \
  --version "v1" \
  --operator oyatie.council-security.<operator-id> \
  --reason "stage-1-runner-compromise"
```

This fragment (per synthesis §5.3) contains:

```cedar
forbid (
  principal is SpiffeIdentity,
  action,
  resource
)
when {
  principal.spiffe_trust_domain == "bootstrap-stage-1"
};
```

Verify propagation to the bootstrap cell (≤30s):
```
policy-engine-cli fragment verify-active \
  --fragment-id "pack-bootstrap-kill-switch" \
  --cell bootstrap-cell --timeout 60s
```

The kill-switch fires: all Stage-1 SPIFFE identities are now forbidden from any action. The Stage-1 CI runner is operationally dead.

Emit:
```
audit-emit BootstrapKillSwitchActivated \
  --operator oyatie.council-security.<operator-id> \
  --reason "runner-compromise" \
  --elapsed-bootstrap-hours <HOURS_ELAPSED>
```

### Step 2 — Revoke Stage-1 SPIFFE certificates (target: ≤5 min)

Revoke all SPIFFE workload identity certificates issued to Stage-1 runners:

```
# Revoke via SPIRE server:
spire-cli entry delete --selector "stage:bootstrap-stage-1"

# Revoke the one-shot offline-rooted CA that issued Stage-1 SPIFFE certs:
# (This CA was issued a single-use cert for the ≤8h bootstrap window)
vault write pki/revoke \
  serial_number=<STAGE1_CA_CERT_SERIAL>
```

Publish CRL update so all trust bundle consumers see the revocation:
```
vault write pki/tidy safety_buffer=5m
```

### Step 3 — Quarantine all Stage-1 artifacts (target: ≤15 min)

Move all container artifacts produced during Stage-1 to a quarantine registry namespace:

```
microservices/cloud-iac/bin/quarantine-artifacts \
  --source-registry <REGISTRY>/<ORG>/bootstrap-artifacts \
  --quarantine-registry <REGISTRY>/<ORG>/quarantine/<INCIDENT_ID> \
  --session-tag "bootstrap-stage-1-<DATE>" \
  --operator oyatie.council-security.<operator-id>
```

Block access to the quarantine namespace except for security review:
```
cosign policy init \
  --namespace <REGISTRY>/<ORG>/quarantine/<INCIDENT_ID> \
  --deny-all
```

For any Stage-1 artifacts that were already deployed to the bootstrap cell (steps 1.3–1.9 of ADR-0247 §D-5 Stage 1):
```
kubectl get deployments --all-namespaces -o json | \
  jq '.items[] | select(.spec.template.spec.containers[].image | contains("bootstrap-stage-1"))' | \
  kubectl delete --wait=true -f -
```

### Step 4 — Assess bootstrap state integrity

Determine how far the Stage-1 sequence progressed before the compromise was detected:

```
psql -c "SELECT step, completed_at, artifact_ref, signed_by_key_id
  FROM bootstrap_sequence_log ORDER BY step;"
```

For each completed step, verify the artifact's cosign signature traces to `org-root-key-<TIMESTAMP>`:
```
for ARTIFACT_REF in $(psql -t -c "SELECT artifact_ref FROM bootstrap_sequence_log WHERE completed_at IS NOT NULL;"); do
  cosign verify --key /hsm/org-root-key.pub "${ARTIFACT_REF}" || echo "SUSPECT: ${ARTIFACT_REF}"
done
```

Any step with a `SUSPECT` artifact must be re-executed from scratch.

### Step 5 — Rebuild from offline-rooted CA (target: 2–4h)

If the compromise is confirmed (one or more SUSPECT artifacts), the bootstrap must restart from the last clean state. Determine the safe restart point:

```
# Find the last step where all artifacts verify clean:
SAFE_RESTART_STEP=$(psql -t -c "
  SELECT MAX(step) FROM bootstrap_sequence_log
  WHERE signed_by_key_id = '<ORG_ROOT_KEY_ID>'
    AND tamper_verified = true;")
```

Reset the bootstrap state machine to `SAFE_RESTART_STEP`:
```
psql -c "UPDATE bootstrap_state SET stage = 'STAGE_1',
  current_step = <SAFE_RESTART_STEP>,
  restart_reason = 'runner-compromise',
  restarted_at = now()
  WHERE bootstrap_id = 'current';"
```

Issue a new one-shot CA for the Stage-1 replacement runner. This CA has a fresh ≤8h validity window:
```
vault write pki/root/generate/internal \
  common_name="bootstrap-stage1-replacement-<DATE>" \
  ttl="8h" \
  key_type="ed25519"
```

Generate new SPIFFE identity for the replacement runner:
```
spire-cli entry create \
  --spiffe-id "spiffe://bootstrap-stage-1-replacement-<DATE>/ci-runner" \
  --selector "stage:bootstrap-stage-1-replacement-<DATE>" \
  --ttl 28800
```

Resume the bootstrap sequence from `SAFE_RESTART_STEP` using the new runner identity, building artifacts signed by the org root key chain.

### Step 6 — Retroactive audit log ingestion

Per ADR-0247 §D-5 Stage 0.5, the bootstrap-replay log must capture Stage-0 and Stage-1 actions. Ensure the quarantine and kill-switch events are appended to the bootstrap-replay log:

```
audit-chain-cli append-bootstrap-replay \
  --events kill-switch-activation,artifact-quarantine,runner-revocation \
  --incident-ref <INCIDENT_ID> \
  --operator oyatie.council-security.<operator-id>
```

This log will be ingested into the audit chain at Stage 2.6 when the bootstrap cell's audit stream is initialised.

---

## §D Verification

1. **Kill-switch fragment is ACTIVE:**
   ```
   policy-engine-cli fragment status --fragment-id "pack-bootstrap-kill-switch"
   ```

2. **No Stage-1 SPIFFE identities can authorize any action:**
   ```
   cedar-cli authorize \
     --principal "spiffe://bootstrap-stage-1/ci-runner" \
     --action "Substrate::Action::DeploySubstrateVersion" \
     --resource "*"
   ```
   Must return `DENY`.

3. **All suspect artifacts are in quarantine registry (not in active deployment):**
   ```
   kubectl get pods --all-namespaces -o jsonpath='{range .items[*]}{.spec.containers[*].image}{"\n"}{end}' | \
     grep "bootstrap-stage-1" | wc -l
   ```
   Must return `0`.

4. **Bootstrap state machine is at or before SAFE_RESTART_STEP:**
   ```
   psql -c "SELECT stage, current_step, restart_reason FROM bootstrap_state;"
   ```

5. **Audit-chain bootstrap-replay log updated with incident events.**

---

## §E Rollback

The kill-switch is designed to be one-way during the bootstrap window (fire-and-forget by design, per synthesis §5.3). There is no rollback of the kill-switch activation.

To resume bootstrap from a clean state, follow Step 5 (rebuild from offline-rooted CA). This is the intended recovery path: the bootstrap window simply restarts with a fresh Stage-1 runner identity and a new 8h budget.

If the bootstrap window has expired (8h elapsed) and Stage-2 is not yet reachable:
- Provision additional Tier 0 compute.
- File a deviation record explaining the extended bootstrap time.
- Restart the 8h window with documented rationale.

---

## §F Post-Incident

1. Root-cause analysis: how was the Stage-1 runner identity compromised? (Network exposure, CI secret leak, SPIFFE issuance policy gap.)
2. Update the Stage-1 runner isolation requirements in ADR-0247 §D-5 if a new hardening step is identified.
3. Verify that the org root key was not exposed during the Stage-1 runner compromise (it is used to sign artifacts passed to the runner, but should never be accessible from within the runner).
4. Post-mortem within 72h.
5. Update this runbook with any new failure mode observed.

---

## §G References

- ADR-0247 §D-5 (Bootstrap sequence stages; Stage-1 external CI runner)
- ADR-0247 §D-4 (Bootstrap minimum — Tier 0)
- ADR-0243 §D-5 (Bootstrap chain of trust — org root key)
- ADR-0248 §D-2 (Tier 1 bootstrap cell)
- Synthesis §5.3 (F5-247-02: Stage-1 SPIFFE identity binding + kill-switch requirement)
- `docs/runbooks/meta-trust-root-recovery.md`
- `docs/runbooks/self-modification-rollback.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
