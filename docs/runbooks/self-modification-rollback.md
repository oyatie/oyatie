---
purpose: Oyatie Runbook — Self-Modification Rollback (ADR-0247)
doc_status: published
---

# Oyatie Runbook — Self-Modification Rollback

> **Status:** Active
> **Owner:** ops-sre-reliability + council-security + axis-workflow-engine
> **Last updated:** 2026-05-20
> **Last verified:** 2026-05-20 (validated during retired `./bin/oya verify` gate repair sweep)
> **Related ADRs:** ADR-0247 §D-3, ADR-0247 §D-6, ADR-0247 §D-7, ADR-0247 §D-8, ADR-0243, ADR-0249

---

## §A Trigger Conditions

This runbook covers rollback of a **Foundry-executed self-modification** — any of the three modification classes defined in ADR-0247 §D-3:

- **Class 1:** `PublishWorkflowVersion` + `ActivateWorkflowVersion` — a new workflow version was activated and is causing incorrect behavior.
- **Class 2:** `PublishFragment` + `ActivateFragment` — a new Cedar fragment was authored and activated by `oyatie.foundry.fragment-author` and is producing incorrect policy decisions (see also `docs/runbooks/cedar-fragment-emergency-rollback.md` for the security-incident variant).
- **Class 3:** `DeploySubstrateVersion` — a new substrate µservice version was deployed via `oyatie.foundry.release-deploy` and is causing SLO breach, incorrect behavior, or security regression.

**Automatic rollback vs. manual rollback:**

Per ADR-0247 §D-6, `oyatie.foundry.rollback-controller` triggers **automatically** within 60s of SLO breach detection in `dev-tools-cell-prod`. When the automatic rollback fires and this runbook is invoked, proceed from §B to verify the automatic rollback completed correctly, then continue to §C Step 4 for the post-rollback procedure.

**Manual rollback triggers:**
- Automatic rollback did not fire (SLO breach not detected but incorrect behavior observed).
- A security finding requires rollback of a self-modification that did not cause an SLO breach.
- A compliance violation is detected post-activation.
- A post-mortem review finds the modification introduced technical debt or architectural regression.

---

## §B Pre-Checks

Estimated time: **5–10 min**.

1. **Identify the modification to roll back.** Find the relevant audit events:
   ```
   audit-chain-cli query \
     --stream "oyatie.foundry" \
     --event-class "WorkflowVersionActivated,CedarFragmentActivated,SubstrateVersionDeployed" \
     --window-start "<MODIFICATION_WINDOW_START>" \
     --output /tmp/modifications-<INCIDENT_ID>.json
   
   jq '.events[] | {event_type, modification_id, target, version, activated_at, activated_by}' \
     /tmp/modifications-<INCIDENT_ID>.json
   ```

2. **Identify the prior safe version.** For each modification type:

   **Class 1 (workflow version):**
   ```
   workflow-cli history \
     --workflow-id "<WORKFLOW_ID>" \
     --show-activation-history
   ```

   **Class 2 (Cedar fragment):**
   ```
   policy-engine-cli fragment history --fragment-id "<FRAGMENT_ID>"
   ```

   **Class 3 (substrate version):**
   ```
   kubectl rollout history deployment/<DEPLOYMENT_NAME> -n <NAMESPACE>
   ```

3. **Check if automatic rollback already fired:**
   ```
   audit-chain-cli query \
     --stream "oyatie.foundry" \
     --event-class "RollbackControllerFired,WorkflowVersionRolledBack,SubstrateVersionRolledBack" \
     --window-start "<SLO_BREACH_TIMESTAMP>"
   ```
   If automatic rollback fired and the prior version is now active, skip to §C Step 4.

4. **Assess blast radius.** Determine what was affected during the bad modification window:
   ```
   # For Class 1: which workflow instances ran under the bad version?
   workflow-cli instances --workflow-id <WORKFLOW_ID> --version <BAD_VERSION> --status any
   
   # For Class 2: what Cedar decisions were made under the bad fragment?
   # Use docs/runbooks/cedar-fragment-emergency-rollback.md §C Step 5
   
   # For Class 3: which cells received the bad substrate version?
   kubectl get pods -l app=<SUBSTRATE> -o jsonpath='{range .items[*]}{.spec.containers[0].image}{"\n"}{end}' \
     --all-namespaces
   ```

5. **Declare incident.** SEV-2 for SLO breach without security impact. SEV-1 for security regression. Notify `ops-sre-reliability`, `council-security`, `axis-workflow-engine`.

---

## §C Procedure

### Step 1 — Pause new self-modification operations (target: ≤60s)

Prevent additional modifications from being activated while the rollback is in progress:

```
cat > /tmp/self-mod-pause-<INCIDENT_ID>.cedar << 'EOF'
// TEMPORARY: self-modification pause during rollback
// EXPIRES: <ISO8601 +60min>
forbid (
  principal in Tenant::"oyatie".sub_scopes("foundry"),
  action in [
    Workflow::Action::ActivateWorkflowVersion,
    Cedar::Action::ActivateFragment,
    Substrate::Action::DeploySubstrateVersion
  ],
  resource
)
when { context.rollback_in_progress == true };
EOF

policy-engine-cli fragment publish \
  --fragment-path /tmp/self-mod-pause-<INCIDENT_ID>.cedar \
  --scope "oyatie/foundry/rollback-pause-<INCIDENT_ID>" \
  --ttl-seconds 3600 \
  --operator oyatie.council-security.<operator-id>
```

### Step 2 — Execute rollback per modification class (target: ≤5 min per class)

#### Class 1: Workflow version rollback

Per ADR-0247 §D-7, rollback is an atomic `ActivateWorkflowVersion` to the prior version:

```
workflow-cli activate-version \
  --workflow-id "<WORKFLOW_ID>" \
  --version "<PRIOR_SAFE_VERSION>" \
  --operator oyatie.council-security.<operator-id> \
  --reason "rollback-<INCIDENT_ID>"
```

Running instances of the bad version continue on their pinned version (per ADR-0247 §D-7 "per-instance version pinning") — they cannot be retroactively moved. Signal running bad-version instances to drain gracefully:

```
workflow-cli signal \
  --workflow-id <WORKFLOW_ID> \
  --version <BAD_VERSION> \
  --signal drain-graceful \
  --timeout 300s
```

For `oyatie.foundry.*` workflow instances that produced incorrect outputs (e.g., `oyatie.foundry.adr-drafter` that authored a bad ADR, or `oyatie.foundry.fragment-author` that published an incorrect fragment): their outputs must be remediated separately (see Step 4).

#### Class 2: Cedar fragment rollback

Follow `docs/runbooks/cedar-fragment-emergency-rollback.md` §C Steps 1–4. Key steps:

```
policy-engine-cli fragment emergency-revoke \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<BAD_VERSION>" \
  --reason "self-modification-rollback-<INCIDENT_ID>" \
  --operator oyatie.council-security.<operator-id>

policy-engine-cli fragment activate \
  --fragment-id "<FRAGMENT_ID>" \
  --version "<PRIOR_SAFE_VERSION>" \
  --operator oyatie.council-security.<operator-id>
```

Wait for propagation to all cells (≤30s per ADR-0243 §D-10).

#### Class 3: Substrate version rollback

Per ADR-0247 §D-3 Class 3, rollback uses `Substrate::Action::RollbackSubstrateVersion` via the rollback-controller:

```
workflow-cli start oyatie.foundry.rollback-controller \
  --substrate <SUBSTRATE_NAME> \
  --target-version "<PRIOR_SAFE_VERSION>" \
  --cells <AFFECTED_CELLS> \
  --operator oyatie.council-security.<operator-id>
```

The rollback-controller executes the canary rollback pattern in reverse:
- Step A: Roll back 10% of one cell's replicas; observe SLOs for 5 min.
- Step B: Roll back to 50%; observe 5 min.
- Step C: Roll back to 100% of first cell.
- Step D: Roll back cell-by-cell across affected cells per ADR-0248 topology.

Monitor rollback progress:
```
workflow-cli status --workflow oyatie.foundry.rollback-controller --instance <ROLLBACK_INSTANCE_ID>
```

### Step 3 — Mark rollback in audit-chain

Regardless of modification class, emit the rollback record:

```
audit-emit SelfModificationRolledBack \
  --modification-class "<class-1|class-2|class-3>" \
  --target "<WORKFLOW_ID|FRAGMENT_ID|SUBSTRATE_NAME>" \
  --bad-version "<BAD_VERSION>" \
  --restored-version "<PRIOR_SAFE_VERSION>" \
  --rollback-reason "<REASON>" \
  --incident-ref <INCIDENT_ID> \
  --operator oyatie.council-security.<operator-id>
```

This event in the `oyatie.foundry` audit stream is the authoritative record of the rollback per ADR-0247 §D-3.

### Step 4 — Remediate outputs produced during the bad modification window

The rollback stops new harm but does not undo work already done by the bad modification. For each class:

**Class 1 bad outputs:** Enumerate and assess each workflow instance that ran under the bad version:
```
workflow-cli instances \
  --workflow-id <WORKFLOW_ID> --version <BAD_VERSION> --status completed \
  | jq '.[] | {instance_id, completed_at, output_artifact_refs}'
```

For `oyatie.foundry.adr-drafter` bad outputs: revert any ADR documents committed during the bad window using git:
```
git log --since="<BAD_ACTIVATION_AT>" --until="<ROLLBACK_AT>" -- docs/decisions/ | \
  xargs git revert --no-commit
git commit -m "revert: rollback adr-drafter outputs from bad workflow version <BAD_VERSION> (incident <INCIDENT_ID>)"
```

For `oyatie.foundry.ci-build-and-test` bad outputs: mark all artifacts produced in the bad window as quarantined:
```
microservices/cloud-iac/bin/quarantine-artifacts \
  --session-tag "foundry-bad-version-<BAD_VERSION>-<TIMESTAMP_RANGE>" \
  --quarantine-registry <REGISTRY>/<ORG>/quarantine/<INCIDENT_ID>
```

**Class 2 bad Cedar grants:** Run evaluation replay per `docs/runbooks/cedar-fragment-emergency-rollback.md` §C Step 5 and remediate any unauthorized grants.

**Class 3 bad substrate behavior:** Assess whether any data-plane operations produced incorrect results during the bad-version window. File findings in `evidence/incidents/<INCIDENT_ID>/substrate-impact.json`.

### Step 5 — Post-mortem trigger

For all SEV-1 and SEV-2 rollbacks, trigger the post-mortem workflow:

```
workflow-cli start oyatie.foundry.post-mortem-trigger \
  --incident-ref <INCIDENT_ID> \
  --rollback-event-id <AUDIT_EVENT_ID> \
  --deadline "<NOW + 72h>"
```

The post-mortem is required within 72h per ADR-0247 §D-6.

### Step 6 — Remove rollback-pause Cedar fragment (target: ≤5s)

```
policy-engine-cli fragment deactivate \
  --scope "oyatie/foundry/rollback-pause-<INCIDENT_ID>" \
  --operator oyatie.council-security.<operator-id>
```

Verify self-modification operations resume normally (for legitimate operations, not the bad workflow/fragment/substrate):
```
sleep 10
workflow-cli status --workflow oyatie.foundry.ci-build-and-test --recent 3
```

---

## §D Verification

1. **Prior safe version is active:**

   Class 1:
   ```
   workflow-cli current-version --workflow-id <WORKFLOW_ID>
   ```
   Must return `<PRIOR_SAFE_VERSION>`.

   Class 2:
   ```
   policy-engine-cli fragment current-version --fragment-id <FRAGMENT_ID>
   ```

   Class 3:
   ```
   kubectl get deployment <DEPLOYMENT_NAME> -o jsonpath='{.spec.template.spec.containers[0].image}'
   ```

2. **SLO error budget recovering** on `microservices/observability/dashboards/cellular-topology.md`.

3. **Rollback-pause fragment is inactive:**
   ```
   policy-engine-cli fragment status --scope "oyatie/foundry/rollback-pause-<INCIDENT_ID>"
   ```

4. **`SelfModificationRolledBack` audit event present** with Merkle proof in `oyatie.foundry` stream.

5. **Post-mortem workflow initiated** (if SEV-1 or SEV-2):
   ```
   workflow-cli status --workflow oyatie.foundry.post-mortem-trigger --recent 1
   ```

---

## §E Rollback of the Rollback

If the prior safe version itself has issues (unusual but possible):
1. Do not activate the bad version again.
2. Identify the last version before the prior safe version that was known good.
3. Apply this runbook recursively to activate that version.
4. If no safe version exists in history (e.g., all versions of the workflow are broken), the workflow must be rebuilt from scratch via the standard authoring flow with multispectrum review.

---

## §F Post-Incident

1. Root-cause analysis: why did the multispectrum review (ADR-0247 §D-6 `staging → prod` gate) not catch the issue?
   - Was the staging soak period (24h) insufficient to surface the behavior?
   - Was the eval-runner parity report within tolerance despite the bad behavior?
   - Was the review facet scope incomplete?
2. Update eval criteria for the modified workflow/fragment/substrate to catch this regression class.
3. If automatic rollback fired correctly: document it as evidence that the ADR-0247 §D-6 SLO-breach → 60s auto-rollback is functioning.
4. If automatic rollback did not fire: investigate the SLO definition and breach-detection configuration. The 60s target (ADR-0247 §D-6) is a commitment; missed triggers are MFL findings.
5. Review whether `staging → prod` human-approval gate should have caught this:
   - Was the modification touching the self-modification permits fragment (requiring ≥2 council approvers)?
   - If yes and approval was bypassed: this is a security finding; escalate to `council-security`.
6. Post-mortem within 72h.

---

## §G References

- ADR-0247 §D-3 (Self-modification mechanics — 3 classes)
- ADR-0247 §D-6 (`dev-tools-cell-{dev,staging,prod}` environments; 60s auto-rollback commitment)
- ADR-0247 §D-7 (Workflow Engine substrate; `ActivateWorkflowVersion` atomicity)
- ADR-0247 §D-8 (Cedar fragment gating self-modification; meta-permit)
- ADR-0243 §D-10 (Hot-reload; ≤30s fragment propagation)
- ADR-0249 (Workflow Engine substrate — workflow versioning primitives)
- `docs/runbooks/cedar-fragment-emergency-rollback.md`
- `docs/runbooks/meta-trust-root-recovery.md`
- [INCIDENT-MANAGEMENT.md](../INCIDENT-MANAGEMENT.md)
