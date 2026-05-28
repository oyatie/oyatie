---
runbook_id: cloud-iac/restore-drill-quarterly
authored: 2026-05-18
oncall: axis-cloud-iac
adr_authority: ADR-0197
cadence: quarterly
---

# Runbook — Quarterly restore drill

## Scope

Per ADR-0197 D-6, oyatie runs a quarterly restore drill on one µservice
per quarter on a rotation. The drill exercises Velero (K8s state + PV)
+ pgBackRest (Postgres PITR) end-to-end into the `dr-staging`
environment.

## Procedure

### Day -7 (preparation)

1. Pick the target µservice from the rotation
   (`registry/restore-drill-rotation.yaml`).
2. Notify the µservice owner; confirm `dr-staging` namespace is
   available.
3. Confirm the most-recent backups exist:
   ```sh
   velero get backups | grep <microservice>
   pgbackrest --stanza=<microservice> info
   ```

### Day 0 (drill execution)

1. **Restore Postgres** from pgBackRest to a PITR target of
   `T-now - 5 minutes`:
   ```sh
   pgbackrest --stanza=<microservice> \
              --target="$(date -u -d '5 minutes ago' '+%Y-%m-%d %H:%M:%S')" \
              restore
   ```
2. **Restore K8s state + PVs** via Velero:
   ```sh
   velero restore create drill-$(date +%s) \
          --from-backup <most-recent-daily> \
          --namespace-mappings <microservice>-prod:<microservice>-dr-staging
   ```
3. **Apply the µservice Helm chart** against `dr-staging`:
   ```sh
   helm install <microservice> microservices/<microservice>/iac/helm/<microservice>/ \
        --namespace <microservice>-dr-staging \
        --values microservices/<microservice>/iac/helm/<microservice>/values-dr-staging.yaml
   ```
4. **Run the µservice's smoke-test set** against `dr-staging`:
   ```sh
   cargo test -p oya-<microservice>-app --test smoke -- --include-ignored
   ```

### Day +1 (evidence + scoring)

1. Measure observed RPO + RTO; compare to the workload-class targets
   per ADR-0197 D-4.
2. Emit `class: BackupRestoreDrill` to the audit chain with the
   measurements, signed by the chaos-substrate key.
3. Update `registry/restore-drill-rotation.yaml` to advance the
   rotation pointer.

## Failure handling

- **Drill fails to complete** → SEV-2; µservice owner has 7 days to
  remediate; another drill is scheduled at the end of the 7 days. If
  remediation fails, the µservice is downgraded one promotion tier per
  ADR-0181.
- **Drill completes but RPO/RTO exceeds workload-class target** → SEV-2;
  same remediation procedure.

## Evidence

- Audit-chain class `BackupRestoreDrill` is the canonical evidence.
- Drill outcomes feed the regulator-evidence quarterly emit per
  ADR-0174.

## References

- ADR-0197 D-6 — restore drill cadence.
- ADR-0165 — chaos engineering substrate (drill runner).
- ADR-0181 — container image promotion pipeline (tier-downgrade
  authority).
- `docs/standards/backup-canonical.md`.
