---
doc_class: Runbook
title: Migration Execution (Per-µservice + DR Failover)
microservice: governance
status: Accepted
classification: INTERNAL_ONLY
date: 2026-05-17
owner_team: ops-sre-reliability + axis-foundry
severity_default: Sev-3 (planned migration); Sev-1/2 if DR failover
related_failure_modes: [F-05]
related_artifacts:
  - microservices/governance/multi-region.md
  - microservices/governance/failure-modes.md
review_cadence: quarterly + per-pack onboarding
doc_status: published
---

# Runbook: Migration Execution

## When to invoke

- New `oya-check-*` crate migrating to `microservices/governance/src/crates/` per ADR-0131 IP-M01-MIGR-014 (per-batch IPs IP-002, IP-003, P02..P04).
- New pack onboarding (Wave 2..11 per `multi-region.md` Roadmap).
- DR failover triggered (F-05 Postgres failover OR regional outage).
- Workspace `Cargo.toml` member path update due to ADR-0131 layout.

## Decision tree

```text
                Migration type?
                  ├─ Per-crate within governance      → §A
                  ├─ Per-pack onboarding              → §B
                  ├─ DR failover (emergency)          → §C
                  └─ Workspace member rebase          → §D
```

## §A — Per-crate migration (e.g., one `oya-check-*` → `microservices/governance/`)

This pattern executes once per batch of ~5 crates (per IP).

### Pre-flight

- Source path: `crates/oya-check-<topic>/`.
- Target path: `microservices/governance/src/crates/oya-check-<topic>/`.
- ChangeSet contract per ADR-0110: one PR; atomic.
- All cross-refs to source path identified.

### Steps

1. **Identify** cross-refs:
   ```bash
   rg "crates/oya-check-<topic>" --type-not lock > /tmp/xrefs-<topic>.txt
   ```

2. **Move** via `git mv` (preserves history):
   ```bash
   git mv crates/oya-check-<topic> microservices/governance/src/crates/oya-check-<topic>
   ```

3. **Update workspace** `Cargo.toml` `[workspace.members]`:
   ```toml
   - "crates/oya-check-<topic>",
   + "microservices/governance/src/crates/oya-check-<topic>",
   ```

4. **Update cross-refs** in Rust + docs (via structured search-replace; per ADR-0131 §"No-blanket-sed" principle, use `cargo-edit` or ast-grep, not raw sed):
   ```bash
   ast-grep --pattern 'crates/oya-check-<topic>' --rewrite 'microservices/governance/src/crates/oya-check-<topic>' -r .
   ```

5. **Author** catalog row at `microservices/governance/catalog/oya-check-<topic>.yaml`.

6. **Verify** workspace integrity:
   ```bash
   cargo check --workspace --all-features
   cargo build --workspace --all-features
   cargo nextest run --workspace --all-features
   cargo run -p oya-dev-cli -- gate validate cross-ref-validity
   cargo run -p oya-dev-cli -- gate validate per-microservice-layout --microservice governance
   ```

7. **Commit + PR**:
   ```bash
   git commit -m "feat(governance): migrate oya-check-<topic> to microservices/governance/ (ADR-0131 IP-M01-MIGR-014)"
   gh pr create --title "Migrate oya-check-<topic>" --body "Per ADR-0131 §IP-M01-MIGR-014. Atomic ChangeSet."
   ```

8. **Verify** PR's full ~50-lane suite passes (including self-application of the migrated lane).

### Per-batch IPs

Tier-A first 10 crates → IP-002 + IP-003 (5 each).
Tier-B next batch → IP-012 + IP-013.
Tier-C remainder → IP-014 + IP-015 + phases P02..P04.

## §B — Per-pack onboarding

Per `multi-region.md` §"Pack-onboarding gate".

### Pre-flight

- ADR-NNNN-pack-<pack>-onboarding accepted.
- OCI region provisioned (compute + network + IAM).
- pack-specific KMS keyring + S3 bucket + Postgres replica ready.
- Cedar policy fragment reviewed for pack-specific overrides.

### Steps

1. **Deploy** governance cluster via per-pack overlay:
   ```bash
   kubectl apply -k iac/kustomize/overlays/pack-<pack>/
   ```

2. **Verify** all components healthy:
   ```bash
   kubectl get pods -n governance-pack-<pack> -o wide
   kubectl get pvc -n governance-pack-<pack>
   kubectl get ingress -n governance-pack-<pack>
   ```

3. **Cedar deploy** with pack-specific policy:
   ```bash
   cargo run -p oya-dev-cli -- governance policy deploy --pack <pack>
   ```

4. **Compliance check**:
   - ROPA in `compliance.md` updated for the new pack.
   - DPIA addendum signed by council-privacy.
   - Per-pack residency rules in `policy/data-residency.md` updated.

5. **DR drill** before accepting production traffic:
   ```bash
   cargo run -p oya-dev-cli -- governance dr-drill --pack <pack> --mode rehearsal
   ```

6. **Tenant migration** (if relocating existing tenants):
   - Per-tenant consent + ADR.
   - Per-data-class migration plan.
   - Run `oya-check-data-residency --pack <pack>` lane.

7. **Cut-over**:
   ```bash
   cargo run -p oya-dev-cli -- governance pack-traffic-enable --pack <pack>
   ```

8. **Verify** end-to-end synthetic PR in the new pack.

## §C — DR failover (emergency; Sev-1/2)

Per `multi-region.md` §"Failover Procedures → Standard DR failover".

### Steps

1. **Confirm** primary region unavailability (Postgres + S3 health-check fail; Grafana alert).
2. **Decision** by ops-sre-reliability on-call + axis-foundry on-call within 5 min.
3. **Promote** DR Postgres replica:
   ```bash
   kubectl exec -n governance-dr postgres-replica -- patronictl failover
   ```
4. **Switch DNS** via Cloudflare:
   ```bash
   cargo run -p oya-dev-cli -- governance dr-switch --pack <pack> --to dr
   ```
5. **Scale up** DR ARC pool from 50% → 100%:
   ```bash
   kubectl scale -n governance-dr deployment/arc-runner-pool --replicas=50
   ```
6. **Backfill** in-flight lane runs from primary outbox:
   ```bash
   cargo run -p oya-dev-cli -- governance outbox replay --since <unix>
   ```
7. **Verify** end-to-end:
   ```bash
   cargo run -p oya-dev-cli -- governance synthetic-pr --pack <pack>
   ```
8. **Notify** stakeholders; open postmortem.

### Stand-down

- All in-flight lane runs completed via outbox replay.
- Synthetic PR passes through DR-promoted primary.
- Grafana posture green.

### Post-failover

- RCA per `incident-response.md`.
- Recovery to primary region when conditions allow (off-peak window; coordinated cut-back).

## §D — Workspace member rebase

When ADR-0131 layout changes require Cargo.toml updates (e.g., move all `crates/` → `microservices/<ms>/src/crates/`):

1. **Stage** the move via per-µservice migration IP.
2. **Update** workspace `Cargo.toml` member list.
3. **Run** `cargo metadata --offline` to verify integrity.
4. **Update** `oya gate validate per-microservice-layout` to pass.
5. **PR** the workspace update separately from the move (so reviewers see each ChangeSet clearly).

## Stand-down criteria

- All targeted artifacts in their new location.
- `cargo build --workspace` exits 0.
- `cargo nextest run --workspace` exits 0.
- `oya gate validate cross-ref-validity` exits 0.
- `oya gate validate per-microservice-layout` exits 0.
- Synthetic PR end-to-end check passes.

## Post-action

- Update `multi-region.md` Roadmap row (for §B).
- Update postmortem if applicable (for §C).
- File successor-IP IPs for any structural improvement opportunity surfaced.

## References

- ADR-0131 §"Migration DAG → IP-M01-MIGR-014".
- ADR-0117 (data-residency).
- `microservices/governance/multi-region.md` (DR + pack-onboarding).
- `microservices/governance/failure-modes.md` F-05.
- `microservices/governance/incident-response.md`.
