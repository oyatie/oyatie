---
doc_class: Standard
purpose: "Secret handling, Cedar autonomy enforcement, idempotency, and symlink defense"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Supervisor Kernel — Security

## Secret Handling Policy

### Rule: OpenBao Only

All secrets (API tokens, MCP server credentials, provider account tokens) are resolved **at spawn time** via OpenBao, never embedded in kernel types.

```
SessionTicket (kernel)
  ├─ account_id: AccountId           ✓ (public)
  ├─ provider_family: ProviderFamily  ✓ (public)
  ├─ autonomy_tier: AutonomyTier      ✓ (public)
  └─ message_id: MessageId             ✓ (ULID; idempotency only)
  
✗ NEVER:
  ├─ api_token: String                ✗ (use OpenBao sref://)
  ├─ secret_ref: SecretReference      ✗ (resolved at spawn, not transport)
  └─ raw_credential: Vec<u8>          ✗ (audit-trail sensitive)
```

### `SecretReference` Usage

Settings templates declare secrets via `sref://` references:

```toml
# Illustrative settings-template fragment (templates/foundry-supervisor/ deleted —
# hooks pointed at missing tools/foundry-supervisor-* binaries).
[[mcp_servers]]
name = "oya-mcp-gateway"
env_secret_refs = [
    { env_var = "OYA_MCP_TOKEN", sref = "sref://openbao/foundry/mcp-gateway/token" }
]
```

Resolution happens in `supervisor-app` at session spawn:

```rust
// In SessionDriver::spawn_for_message():
for (env_var, sref) in &account.secret_refs {
    let secret = openbao_client.resolve(sref).await?;
    env::set_var(env_var, secret);  // never printed, never logged
}
```

**Audit trail:** No secret values appear in any log, audit event, or span attribute. The audit event records only the `sref://` path, not the resolved value.

## Cedar Autonomy Enforcement

### Tier Hierarchy

```
┌─────────────────────────────────────────┐
│ AutonomyTier (Cedar policy decision)    │
├─────────────────────────────────────────┤
│ L0: Admin          (no usage ceiling)   │
│ L1: Unrestricted   (N/A; not a tier)    │
│ L2: Standard       (use_limit: 1M tokens) │
│ L3: Restricted     (use_limit: 100K tokens) │
│ L4: Locked         (use_limit: 0)       │
└─────────────────────────────────────────┘
```

### Enforcement Point

Every `tick_once()` call checks the ceiling via `UsageEnforcement::check_limit()`:

```rust
pub async fn tick_once(...) -> Result<TickOutcome> {
    // Step 7: enforce usage window + autonomy ceiling
    let projection = usage_window.project_p95();
    enforcement.check_limit(&ticket, &projection)?;  // ← Cedar decision
    
    if enforcement_result == Err(OverLimit) {
        return Ok(TickOutcome::Quarantined(ticket.message_id));
    }
}
```

**Invariant:** If `AutonomyTier::Locked`, no session spawns. Cedar enforces this via a policy denial.

### Audit Trail

Every enforcement decision emits:

```json
{
  "event_class": "foundry_supervisor_degrade_account",
  "capability": "foundry.supervisor.enforce_usage",
  "autonomy_tier_at_decision": "L2",
  "enforcement_projection": {
    "projected_tokens_p95": 1050000,
    "use_limit": 1000000
  },
  "principal": "account-uuid",
  "data_classes_touched": ["TENANT_SCOPED"]
}
```

## Request-ID Idempotency

### Invariant

Every `SessionTicket` carries a unique `RequestId` (opaque string, no structural requirements).

```rust
pub struct RequestId(pub String);  // opaque; never parsed by kernel
```

### Idempotency Guarantee

If the same `RequestId` is submitted twice:

1. **First submission:** Message is picked from inbox, spawned, spend recorded, message committed.
2. **Second submission (within replay window):** Message is not re-picked; supervisor recognizes the duplicate and returns the prior result.

**Implementation:** The adapter (JSONL InboxStore) maintains a `RequestId` → `TickOutcome` cache with a bounded TTL (default: 24 hours).

```bash
# Verify cache:
ls -la .omc/state/idempotency-cache/
find .omc/state/idempotency-cache -name "req-*.json" | wc -l
```

### Audit Trail

Replayed requests (same `RequestId`) are logged as:

```json
{
  "event_class": "foundry_supervisor_idempotent_replay",
  "request_id": "req-abc123xyz",
  "prior_outcome": "Spawned(msg-uuid)",
  "principal": "account-uuid"
}
```

## Symlink Defense (v6 BLOCKER-6)

### Problem

The settings-template renderer writes to user home directories (`~/.claude/`, `~/.codex/`, `~/.gemini/`). A compromised account could create symlinks to exfiltrate or corrupt settings files.

### Defense

All renderer implementations enforce:

```rust
// In SettingsRenderer::render():
1. Open parent dir with O_NOFOLLOW
2. stat: assert dir ∧ owner==current_uid ∧ mode <= 0755
3. Reject if symlink
4. Open target file with O_NOFOLLOW|O_CLOEXEC
5. Reject if existing file is symlink
6. Read-merge existing content via fd (NOT fresh path open)
7. cp to backup file `{target}.omc-settings-bak.{epoch}`
8. Write tempfile; fchmod 0600 BEFORE rename
9. atomic rename(2)
```


### Backup Recovery

If a render is interrupted:

```bash
# List backups:
ls -la ~/.claude/settings.json.omc-settings-bak.*

# Restore prior state:
./tools/undo-settings-render.sh ~/.claude/settings.json.omc-settings-bak.1715812345
```

The `undo` script logs every restore to the audit chain.

## Data Classes

Types in the kernel are annotated with their data class per ADR-0008:

```rust
/// data_class: INTERNAL_ONLY
pub struct SessionTicket { ... }

/// data_class: INTERNAL_ONLY (state machine; no tenant payload)
pub enum InboxState { ... }

/// data_class: TENANT_SCOPED (bridges to spend_to_usage_record)
pub struct SpendRecord { ... }

/// data_class: INTERNAL_ONLY
pub struct UsageWindowSnapshot { ... }

/// data_class: INTERNAL_ONLY (request idempotency; opaque to tenant)
pub struct RequestId(pub String);
```

**Privacy policy:** Only `TENANT_SCOPED` types (spend records) are visible to tenant audits. `INTERNAL_ONLY` types are supervisor-internal and never returned to the tenant.

## References

- **ADR-0008:** Data use boundary + privacy classes
- **ADR-0024:** Autonomy tier + Cedar enforcement
- **ADR-0003:** Audit chain + evidence emission
- **Kernel source:** `crates/oya-intelligence-supervisor-kernel/src/lib.rs`
- **v4 Plan § BLOCKER-3:** Audit-chain ADR-0003 conformance + data_class annotations
- **v6 Plan § BLOCKER-6:** Settings-template renderer symlink defense
