---
doc_class: Runbook
microservice: feature-flags
runbook_id: RB-FF-005
status: Accepted
date: 2026-05-20
related_adrs:
  - ADR-0159
  - ADR-0243
  - ADR-0251
companion_docs:
  - microservices/feature-flags/runbooks/killswitch-engaged.md
  - microservices/feature-flags/compliance.md
  - microservices/feature-flags/policy/pack-flag-override.cedar
planned_enforcement_ref: governance-adr-adherence-matrix
---

# Runbook: Pack Override Cascade

## A. Trigger conditions

- A new compliance pack was activated for a tenant; pack-mandated flag overrides applying unexpectedly.
- `PackFlagOverrideApplied` events firing for flags that should not be affected.
- Tenant admin reports that flag mutations are being rejected (pack-locked fields).
- Multiple tenants' flags changed simultaneously after a platform-wide pack update.
- A pack was incorrectly activated on the wrong tenant tier.

## B. Pre-checks (≤3 minutes)

1. Identify which pack triggered the cascade:
   ```bash
   oya audit query --event-class PackFlagOverrideApplied \
     --since 30m \
     --tenant <tenant_id>
   # Output: pack_id, flag_key, override_value, previous_value for each override
   ```
2. Check tenant's active packs:
   ```bash
   oya tenancy get-packs --tenant <tenant_id>
   ```
3. Verify the pack overlay roster (`compliance.md §pack-overlay-roster`) — was this pack supposed to override this flag for this tenant?
4. Check if `pack-overlay-agent` attestation was valid:
   ```bash
   oya audit query --event-class PackFlagOverrideApplied \
     --since 30m \
     --field foundry_attestation_present=true
   ```

## C. Procedure

### Case A — Pack correctly applied; tenant surprised

Educate tenant: pack-mandated overrides are non-negotiable regulatory requirements (e.g., HIPAA `phi-exposure-flag = off`). Tenant admins cannot override these. Refer to `compliance.md §pack-overlay-roster` for the authoritative list.

### Case B — Wrong pack applied to wrong tenant

```bash
# Identify the incorrectly activated pack
oya tenancy get-packs --tenant <tenant_id>

# Deactivate incorrect pack (requires platform-admin + dual-approval)
oya packs deactivate <pack_id> --tenant <tenant_id> \
  --approver-1 <principal_1> \
  --approver-2 <principal_2>

# Pack deactivation triggers pack-overlay-agent to re-evaluate and remove overrides
# Wait ≤60s for re-evaluation
oya flags propagation-status <affected_flag_key> --tenant <tenant_id>
```

### Case C — Pack overlay agent malfunction (applying wrong values)

```bash
# Check pack-overlay-agent logs
kubectl logs -n feature-flags -l app=pack-overlay-agent --since=30m

# Check PackOverrideTamperAttempt detection signals
oya audit query --event-class PackOverrideTamperAttempt --since 30m

# If agent is malfunctioning: stop the agent (requires platform-admin)
kubectl scale deployment pack-overlay-agent -n feature-flags --replicas=0

# Manually restore affected flags to correct state
for flag_key in <affected_flags>; do
  oya flags update $flag_key \
    --tenant <tenant_id> \
    --restore-from-audit  # Restores to pre-override value from audit trail
done

# Restart agent after fix
kubectl scale deployment pack-overlay-agent -n feature-flags --replicas=2
```

### Step — Verify correct state (≤5 minutes)

```bash
# Verify pack overrides are correct per compliance.md §pack-overlay-roster
oya flags list --tenant <tenant_id> --has-pack-override

# For each flag: verify override_value matches expected pack-mandated value
oya compliance check-pack-overlays --tenant <tenant_id>
```

## D. Verification

- `PackFlagOverrideApplied` events show correct `pack_id` + `override_value` matching `compliance.md §pack-overlay-roster`.
- No `PackOverrideTamperAttempt` events in the last 5 minutes.
- Tenant's active packs match their contracted compliance tier.

## E. Rollback

Pack overlays cannot be manually rolled back by tenants (by design). Rollback requires:
1. Platform-admin deactivates incorrect pack.
2. Pack-overlay-agent re-evaluates and removes incorrect overrides.
3. Correct pack (if any) activated with dual-approval.

## F. Post-incident

- Was the incorrect pack activation a human error or an automation bug?
- Review dual-control requirement for production pack activations (`compliance.md §insider-threat-controls`).
- Add test: pack activation should trigger dry-run of overrides before applying to production.

## G. References

- `compliance.md §pack-overlay-roster` — authoritative pack override roster.
- `policy/pack-flag-override.cedar` — Cedar policy for pack overrides.
- `policy/pack-overlay-authorization.cedar` — Cedar policy for pack-overlay-agent authorization.
- ADR-0251 — compliance packs.
