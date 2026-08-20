use std::{env, ffi::OsString, fs, path::Path, sync::Mutex};

use intelligence_claude_agent_sdk::{
    ResolveSettingsOptions, ResolvedSettingSource, SDKSettingsParseError, SettingSource,
    filter_escalating_default_mode, resolve_settings,
};
use serde_json::{Map, Value, json, to_value};
use tempfile::tempdir;

static ENV_LOCK: Mutex<()> = Mutex::new(());

struct ClaudeConfigEnvGuard {
    previous: Option<OsString>,
}

impl ClaudeConfigEnvGuard {
    fn set(path: &Path) -> Self {
        let previous = env::var_os("CLAUDE_CONFIG_DIR");
        // SAFETY: These tests serialize access with ENV_LOCK and restore the
        // variable in Drop before releasing the lock.
        unsafe {
            env::set_var("CLAUDE_CONFIG_DIR", path);
        }
        Self { previous }
    }
}

impl Drop for ClaudeConfigEnvGuard {
    fn drop(&mut self) {
        // SAFETY: Protected by ENV_LOCK for the full lifetime of the guard.
        unsafe {
            match &self.previous {
                Some(value) => env::set_var("CLAUDE_CONFIG_DIR", value),
                None => env::remove_var("CLAUDE_CONFIG_DIR"),
            }
        }
    }
}

fn write_json(path: &Path, value: Value) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn settings(value: Value) -> Map<String, Value> {
    value.as_object().unwrap().clone()
}

#[test]
fn resolve_settings_merges_sources_and_records_provenance()
-> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();

    write_json(
        &config.path().join("settings.json"),
        json!({
            "model": "user-model",
            "permissions": {
                "allow": ["Bash(ls)"],
                "deny": ["WebFetch"]
            }
        }),
    );
    write_json(
        &project.path().join(".claude/settings.json"),
        json!({
            "model": "project-model",
            "cleanupPeriodDays": 10,
            "permissions": {
                "allow": ["Bash(ls)", "Read"],
                "defaultMode": "bypassPermissions"
            }
        }),
    );
    write_json(
        &project.path().join(".claude/settings.local.json"),
        json!({
            "model": "local-model",
            "permissions": {
                "ask": ["Bash(rm *)"]
            }
        }),
    );
    write_json(
        &config.path().join("managed-settings.json"),
        json!({
            "cleanupPeriodDays": 3,
            "permissions": {
                "deny": ["Bash(*)"]
            }
        }),
    );

    let resolved = resolve_settings(ResolveSettingsOptions {
        cwd: Some(project.path().to_path_buf()),
        setting_sources: Some(vec![
            SettingSource::User,
            SettingSource::Project,
            SettingSource::Local,
        ]),
        ..Default::default()
    })?;

    assert_eq!(resolved.effective["model"], json!("local-model"));
    assert_eq!(resolved.effective["cleanupPeriodDays"], json!(3));
    assert_eq!(
        resolved.effective["permissions"]["allow"],
        json!(["Bash(ls)", "Read"])
    );
    assert_eq!(
        resolved.effective["permissions"]["deny"],
        json!(["WebFetch", "Bash(*)"])
    );
    assert_eq!(
        resolved.effective["permissions"]["defaultMode"],
        json!("bypassPermissions")
    );
    assert_eq!(
        resolved.provenance["model"].source,
        ResolvedSettingSource::Local
    );
    assert_eq!(
        resolved.provenance["cleanupPeriodDays"].source,
        ResolvedSettingSource::Managed
    );
    assert_eq!(
        resolved
            .sources
            .iter()
            .map(|source| source.source)
            .collect::<Vec<_>>(),
        [
            ResolvedSettingSource::User,
            ResolvedSettingSource::Project,
            ResolvedSettingSource::Local,
            ResolvedSettingSource::Managed
        ]
    );

    let filtered = filter_escalating_default_mode(&resolved);
    assert_eq!(filtered["permissions"].get("defaultMode"), None);
    assert_eq!(
        filtered["permissions"]["deny"],
        json!(["WebFetch", "Bash(*)"])
    );
    Ok(())
}

#[test]
fn resolve_settings_can_skip_filesystem_sources_but_keep_managed()
-> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();

    write_json(
        &config.path().join("settings.json"),
        json!({"model": "user-model"}),
    );

    let resolved = resolve_settings(ResolveSettingsOptions {
        cwd: Some(project.path().to_path_buf()),
        setting_sources: Some(vec![]),
        managed_settings: Some(settings(json!({
            "permissions": {"deny": ["Bash(*)"]}
        }))),
        ..Default::default()
    })?;

    assert_eq!(resolved.effective.get("model"), None);
    assert_eq!(
        resolved.effective["permissions"]["deny"],
        json!(["Bash(*)"])
    );
    assert_eq!(
        resolved
            .sources
            .iter()
            .map(|source| source.source)
            .collect::<Vec<_>>(),
        [ResolvedSettingSource::Managed]
    );
    Ok(())
}

#[test]
fn resolve_settings_skips_malformed_files_and_reports_parse_errors()
-> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();

    fs::write(config.path().join("settings.json"), "{ invalid json").unwrap();
    fs::create_dir_all(project.path().join(".claude")).unwrap();
    fs::write(project.path().join(".claude/settings.json"), "[]").unwrap();
    write_json(
        &project.path().join(".claude/settings.local.json"),
        json!({"model": "local-model"}),
    );

    let resolved = resolve_settings(ResolveSettingsOptions {
        cwd: Some(project.path().to_path_buf()),
        setting_sources: Some(vec![
            SettingSource::User,
            SettingSource::Project,
            SettingSource::Local,
        ]),
        ..Default::default()
    })?;

    assert_eq!(resolved.effective["model"], json!("local-model"));
    assert_eq!(resolved.sources.len(), 1);
    assert_eq!(resolved.sources[0].source, ResolvedSettingSource::Local);
    assert_eq!(resolved.parse_errors.len(), 2);
    assert!(resolved.parse_errors.iter().any(|error| {
        error
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with("settings.json"))
            && error.path.is_empty()
            && !error.message.is_empty()
    }));
    assert!(resolved.parse_errors.iter().any(|error| {
        error
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with(".claude/settings.json"))
            && error.path.is_empty()
            && error.message.contains("must be an object")
    }));

    let parse_error = SDKSettingsParseError::new(Some("settings.json"), "", "invalid JSON");
    assert_eq!(
        to_value(parse_error)?,
        json!({"file": "settings.json", "path": "", "message": "invalid JSON"})
    );
    Ok(())
}
