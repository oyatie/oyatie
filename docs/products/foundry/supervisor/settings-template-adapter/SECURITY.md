---
doc_class: Standard
purpose: "Symlink defense, file permissions, and secret safety"
owner_team: axis-foundry
status: draft
---

# Settings-Template Adapter — Security

## Symlink Defense (v6 BLOCKER-6)

The adapter implements multi-layer symlink protection:

### Layer 1: Parent Directory Validation

```rust
// Open parent dir with O_NOFOLLOW
let parent_fd = open(parent_path, O_RDONLY|O_NOFOLLOW)?;
let stat = fstat(parent_fd)?;

// Verify it's a real directory (not a symlink)
assert!(!is_symlink(&stat));

// Verify ownership and permissions
assert!(stat.st_uid == getuid());
assert!(stat.st_mode & 0o077 == 0);  // no world/group rwx
```

### Layer 2: Target File Validation

```rust
// Open target with O_NOFOLLOW; reject if symlink
let target_fd = open(target_path, O_RDONLY|O_NOFOLLOW)?;
let stat = fstat(target_fd)?;

// Reject symlinks
if is_symlink(&stat) {
    return Err(SettingsRendererError::SymlinkDetected(target_path));
}
```

### Layer 3: Content-Addressed Verification

After write, blake3 hash ensures content integrity:

```
render() → RenderManifest { files: [{ target_path, content_blake3 }] }
verify() → read file → blake3 → compare against manifest
```

If symlink attacker redirects file after render, verify() detects content mismatch.

## File Permissions

All rendered files are created with mode `0o600` (rw-------):

```rust
fchmod(tmp_fd, 0o600)?;  // before rename
```

This ensures:
- Owner can read/write
- No group or world access
- No execute (even for owner)

## Secret Safety

The adapter:
- **Never stores** raw secrets in rendered files
- **Never resolves** `sref://` references (kernel layer passes them through)
- **Logs metadata only** (sref paths, not resolved values)

**Verified:** Grep for regex `sk-.*` (API key pattern) should find 0 matches in rendered files.

## Audit Trail

Every render and verify operation emits events:

```json
{
  "event_class": "foundry_supervisor_settings_render",
  "capability": "foundry.settings-template.render",
  "data_classes_touched": ["INTERNAL_ONLY"],
  "payload": {
    "provider_family": "Claude",
    "files_rendered": ["~/.claude/settings.json"],
    "manifest_hash": "blake3::..."
  }
}
```

Drift detection:

```json
{
  "event_class": "foundry_supervisor_settings_drift_exclude",
  "payload": {
    "account_id": "acct-xyz",
    "drifted_files": [
      { "path": "~/.claude/settings.json", "state": "Modified" }
    ]
  }
}
```

## References

- **v6 Amendments § BLOCKER-6:** Symlink defense + atomic write
- **POSIX security:** IEEE 1003.1-2017 (open, O_NOFOLLOW, stat)
