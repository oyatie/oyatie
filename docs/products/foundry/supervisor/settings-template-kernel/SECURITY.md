---
doc_class: Standard
purpose: "Secret reference handling and data class annotations"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Kernel — Security

## Secret Reference Handling

### `SecretReference` (sref://)

The kernel defines a `SecretReference` value type (imported from account-kernel):

```rust
pub struct SecretReference {
    pub scheme: String,      // "sref"
    pub service: String,     // "openbao"
    pub path: String,        // "foundry/mcp-gateway/token"
}
```

**Usage in templates:**
```toml
[[mcp_servers]]
name = "mcp-gateway"
env_secret_refs = [
    { env_var = "OYATIE_MCP_TOKEN", sref = "sref://openbao/foundry/mcp-gateway/token" }
]
```

### Resolution at Spawn Time

**Critically:** The kernel NEVER resolves secrets. Resolution happens in `supervisor-app` at session spawn:

```rust
// In supervisor-app (NOT in kernel)
for (env_var, sref) in &account.secret_refs {
    let secret_value = openbao_client.resolve(sref).await?;
    env::set_var(env_var, secret_value);  // ← never logged
}
```

**Principle:** Kernel types are audit-transportable (can cross process boundaries). Secrets never enter the kernel layer.

## Data Class Annotations

All kernel types are annotated per ADR-0008:

```rust
/// data_class: INTERNAL_ONLY
/// Kernel-only values for session routing and state management.
pub struct SettingsTemplate { ... }

/// data_class: INTERNAL_ONLY
/// Hook event enumeration; no tenant data.
pub enum HookEvent { ... }

/// data_class: INTERNAL_ONLY
/// MCP server reference; credentials resolved at spawn time, not stored.
pub struct McpServerRef { ... }

/// data_class: INTERNAL_ONLY
/// Manifest of rendered files; no secrets, no tenant data.
pub struct RenderManifest { ... }

/// data_class: INTERNAL_ONLY
/// Drift detection state; no secrets.
pub struct DriftReport { ... }
```

**Privacy Policy:** All kernel types are `INTERNAL_ONLY`. No kernel data is visible to tenant audits.

## Audit Trail

When drift is detected and reconciled:

```json
{
  "event_class": "foundry_supervisor_settings_drift_exclude",
  "capability": "foundry.supervisor.verify_settings",
  "data_classes_touched": ["INTERNAL_ONLY"],
  "payload": {
    "account_id": "acct-xyz",
    "provider_family": "Claude",
    "drifted_files": [
      { "path": "~/.claude/settings.json", "state": "Modified" }
    ]
  }
}
```

## References

- **ADR-0008:** Data use boundary + privacy classes
- **Account kernel:** `crates/intelligence-account-kernel/src/lib.rs` (SecretReference def)
- **v5 Plan § B.2:** Public API (kernel contracts)
