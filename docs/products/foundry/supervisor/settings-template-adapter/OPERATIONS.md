---
doc_class: Runbook
purpose: "Render/verify workflows, reconciliation, and backup management"
owner_team: axis-foundry
status: draft
doc_status: published
---

# Settings-Template Adapter — Operations

## Manual Render

To manually render settings for all accounts:

> **Retired path:** `templates/foundry-supervisor/` was deleted (hooks pointed at
> missing `tools/foundry-supervisor-*` binaries). Pass a live `--templates-root`
> when one exists; do not recreate the deleted tree.

```bash
cargo run -p dev-cli -- settings-template render \
  --templates-root <live-templates-root> \
  --accounts-root registry/accounts \
  --home-dir ~

# Output: .omc/state/render-manifest.json
```

## Drift Detection

To check for drift across all accounts:

```bash
cargo run -p dev-cli -- gate validate settings-drift \
  --templates-root <live-templates-root> \
  --accounts-root registry/accounts \
  --report-out .omc/state/settings-drift-report.json

# Exit 0 if no drift
# Exit 1 if drift detected
```

### Interpreting the Report

```json
{
  "timestamp": "2026-05-15T10:30:45Z",
  "accounts": [
    {
      "account_id": "acct-xyz",
      "provider_family": "Claude",
      "entries": [
        {
          "target_path": "~/.claude/settings.json",
          "state": "Modified",
          "diff": "< hooks: [...]\n> hooks: [new_hook]"
        }
      ]
    }
  ]
}
```

**States:**
- `Match`: file matches manifest
- `Modified`: file changed since render
- `Missing`: should exist but doesn't
- `Extra`: exists but not in manifest

## Reconciliation

If drift is detected, trigger reconciliation:

```bash
export OYATIE_SUPERVISOR_SETTINGS_RENDERER_MODE=Reconcile
systemctl restart intelligence-supervisor

# Next tick will render all drifted accounts
# Check: .omc/state/settings-drift-report.json should now be all "Match"
```

## Backup Management

After every render, a backup is created:

```bash
ls -la ~/.claude/settings.json.omc-settings-bak.*
ls -la ~/.codex/config.toml.omc-settings-bak.*
```

### Restore from Backup

```bash
#!/bin/bash
# Restore Claude settings to prior state
bak_file="$HOME/.claude/settings.json.omc-settings-bak.1715812345"
if [ -f "$bak_file" ]; then
  cp "$bak_file" "$HOME/.claude/settings.json"
  echo "Restored from $bak_file"
fi
```

Or use the provided undo script:

```bash
./tools/undo-settings-render.sh ~/.claude/settings.json.omc-settings-bak.1715812345
```

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| "Permission denied" on render | Renderer doesn't have write access | Check `chmod ~/.claude/` |
| "Symlink detected" error | ~/.claude is a symlink | Remove symlink; set up real directory |
| Drift never resolves | RendererMode is VerifyOnly | Change to Reconcile; restart |
| Backup files piling up | No cleanup script | Add cleanup to cron: `find ~ -name ".omc-settings-bak.*" -mtime +30 -delete` |

## References

- **Atomic write pattern:** `docs/products/foundry/supervisor/settings-template-adapter/README.md` (v6 BLOCKER-6)
- **v5 Plan § B.5:** CI lane spec
