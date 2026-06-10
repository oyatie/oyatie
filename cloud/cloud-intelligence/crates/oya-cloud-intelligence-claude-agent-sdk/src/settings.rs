use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{error::Result, options::SettingSource};

pub type Settings = Map<String, Value>;

/// Settings file parse or validation error.
///
/// When a settings JSON file cannot be parsed or does not have the expected
/// object shape, Claude Code skips that file rather than applying a partial
/// ruleset. This mirrors the package-exported TypeScript
/// `SDKSettingsParseError` shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SDKSettingsParseError {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    pub path: String,
    pub message: String,
}

impl SDKSettingsParseError {
    pub fn new(
        file: Option<impl Into<String>>,
        path: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            file: file.map(Into::into),
            path: path.into(),
            message: message.into(),
        }
    }
}

/// Settings scope literals used by Claude Code config operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConfigScope {
    Local,
    User,
    Project,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResolveSettingsOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setting_sources: Option<Vec<SettingSource>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_settings: Option<Settings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_managed_settings: Option<Settings>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolvedSettings {
    pub effective: Settings,
    #[serde(default)]
    pub provenance: BTreeMap<String, ProvenanceEntry>,
    #[serde(default)]
    pub sources: Vec<ResolvedSettingsSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parse_errors: Vec<SDKSettingsParseError>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenanceEntry {
    pub source: ResolvedSettingSource,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_origin: Option<PolicySettingsOrigin>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedSettingsSource {
    pub source: ResolvedSettingSource,
    pub settings: Settings,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_origin: Option<PolicySettingsOrigin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ResolvedSettingSource {
    User,
    Project,
    Local,
    Managed,
    Flag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicySettingsOrigin {
    Helper,
    Remote,
    Plist,
    Hklm,
    File,
    Parent,
    Hkcu,
}

/// Resolve effective Claude Code settings from local JSON sources.
///
/// This mirrors the documented SDK shape for filesystem/user-project-local
/// sources, host-provided managed settings, and managed settings JSON files.
/// It does not execute admin `policyHelper` programs or query OS MDM registries;
/// those deployment-specific sources should be supplied via
/// `server_managed_settings` or `managed_settings`.
pub fn resolve_settings(options: ResolveSettingsOptions) -> Result<ResolvedSettings> {
    let cwd = canonicalize_path(options.cwd.as_deref().unwrap_or_else(|| Path::new(".")));
    let allowed_sources = options.setting_sources.clone().unwrap_or_else(|| {
        vec![
            SettingSource::User,
            SettingSource::Project,
            SettingSource::Local,
        ]
    });

    let mut layers = Vec::new();
    let mut parse_errors = Vec::new();
    for source in [
        SettingSource::User,
        SettingSource::Project,
        SettingSource::Local,
    ] {
        if !allowed_sources.contains(&source) {
            continue;
        }
        let path = settings_path_for_source(source, &cwd);
        if let Some(settings) = read_settings_file(&path, &mut parse_errors)? {
            layers.push(ResolvedSettingsSource {
                source: source.into(),
                settings,
                path: Some(path.to_string_lossy().to_string()),
                policy_origin: None,
            });
        }
    }

    if let Some((settings, origin)) = managed_settings_from_sources(&options, &mut parse_errors)? {
        layers.push(ResolvedSettingsSource {
            source: ResolvedSettingSource::Managed,
            settings,
            path: None,
            policy_origin: Some(origin),
        });
    }

    let mut effective = Settings::new();
    let mut provenance = BTreeMap::new();
    for layer in &layers {
        merge_settings(&mut effective, &layer.settings);
        for key in layer.settings.keys() {
            provenance.insert(
                key.clone(),
                ProvenanceEntry {
                    source: layer.source,
                    path: layer.path.clone(),
                    policy_origin: layer.policy_origin,
                },
            );
        }
    }

    Ok(ResolvedSettings {
        effective,
        provenance,
        sources: layers,
        parse_errors,
    })
}

/// Apply the documented trust filter for escalating project default modes.
pub fn filter_escalating_default_mode(resolved: &ResolvedSettings) -> Settings {
    let Some(default_mode) = resolved
        .effective
        .get("permissions")
        .and_then(Value::as_object)
        .and_then(|permissions| permissions.get("defaultMode"))
        .and_then(Value::as_str)
    else {
        return resolved.effective.clone();
    };
    if !matches!(default_mode, "bypassPermissions" | "auto" | "acceptEdits") {
        return resolved.effective.clone();
    }

    for source in resolved.sources.iter().rev() {
        let has_default_mode = source
            .settings
            .get("permissions")
            .and_then(Value::as_object)
            .is_some_and(|permissions| permissions.get("defaultMode").is_some());
        if !has_default_mode {
            continue;
        }
        if source.source == ResolvedSettingSource::Project {
            let mut filtered = resolved.effective.clone();
            if let Some(permissions) = filtered
                .get_mut("permissions")
                .and_then(Value::as_object_mut)
            {
                permissions.remove("defaultMode");
            }
            return filtered;
        }
        return resolved.effective.clone();
    }

    resolved.effective.clone()
}

impl From<SettingSource> for ResolvedSettingSource {
    fn from(value: SettingSource) -> Self {
        match value {
            SettingSource::User => Self::User,
            SettingSource::Project => Self::Project,
            SettingSource::Local => Self::Local,
        }
    }
}

fn settings_path_for_source(source: SettingSource, cwd: &Path) -> PathBuf {
    match source {
        SettingSource::User => config_home_dir().join("settings.json"),
        SettingSource::Project => cwd.join(".claude").join("settings.json"),
        SettingSource::Local => cwd.join(".claude").join("settings.local.json"),
    }
}

fn managed_settings_from_sources(
    options: &ResolveSettingsOptions,
    parse_errors: &mut Vec<SDKSettingsParseError>,
) -> Result<Option<(Settings, PolicySettingsOrigin)>> {
    let mut managed = Settings::new();
    let mut origin = None;

    if let Some(file_settings) = read_managed_settings_from_disk(parse_errors)? {
        merge_settings(&mut managed, &file_settings);
        origin = Some(PolicySettingsOrigin::File);
    }
    if let Some(server_managed_settings) = &options.server_managed_settings {
        merge_settings(&mut managed, server_managed_settings);
        origin = Some(PolicySettingsOrigin::Remote);
    }
    if let Some(parent_managed_settings) = &options.managed_settings {
        merge_settings(&mut managed, parent_managed_settings);
        origin.get_or_insert(PolicySettingsOrigin::Parent);
    }

    if managed.is_empty() {
        Ok(None)
    } else {
        Ok(Some((
            managed,
            origin.unwrap_or(PolicySettingsOrigin::File),
        )))
    }
}

fn read_managed_settings_from_disk(
    parse_errors: &mut Vec<SDKSettingsParseError>,
) -> Result<Option<Settings>> {
    let config_dir = config_home_dir();
    let mut merged = Settings::new();
    let mut found = false;

    if let Some(settings) =
        read_settings_file(&config_dir.join("managed-settings.json"), parse_errors)?
    {
        merge_settings(&mut merged, &settings);
        found = true;
    }

    let managed_dir = config_dir.join("managed-settings.d");
    let entries = match fs::read_dir(&managed_dir) {
        Ok(entries) => entries,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            return Ok(found.then_some(merged));
        }
        Err(error) => return Err(error.into()),
    };
    let mut paths = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".json") && !name.starts_with('.'))
        })
        .collect::<Vec<_>>();
    paths.sort();
    for path in paths {
        if let Some(settings) = read_settings_file(&path, parse_errors)? {
            merge_settings(&mut merged, &settings);
            found = true;
        }
    }

    Ok(found.then_some(merged))
}

fn read_settings_file(
    path: &Path,
    parse_errors: &mut Vec<SDKSettingsParseError>,
) -> Result<Option<Settings>> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            return Ok(None);
        }
        Err(error) => return Err(error.into()),
    };
    if content.trim().is_empty() {
        return Ok(Some(Settings::new()));
    }
    let value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(error) => {
            parse_errors.push(settings_parse_error(path, "", error.to_string()));
            return Ok(None);
        }
    };
    value_to_settings(value, path, parse_errors)
}

fn value_to_settings(
    value: Value,
    path: &Path,
    parse_errors: &mut Vec<SDKSettingsParseError>,
) -> Result<Option<Settings>> {
    match value {
        Value::Object(object) => Ok(Some(object)),
        _ => {
            parse_errors.push(settings_parse_error(
                path,
                "",
                format!("settings JSON must be an object: {}", path.display()),
            ));
            Ok(None)
        }
    }
}

fn settings_parse_error(
    path: &Path,
    field_path: impl Into<String>,
    message: impl Into<String>,
) -> SDKSettingsParseError {
    SDKSettingsParseError::new(
        Some(path.to_string_lossy().to_string()),
        field_path,
        message,
    )
}

fn merge_settings(target: &mut Settings, source: &Settings) {
    for (key, value) in source {
        match target.get_mut(key) {
            Some(existing) => merge_value(existing, value.clone()),
            None => {
                target.insert(key.clone(), value.clone());
            }
        }
    }
}

fn merge_value(target: &mut Value, source: Value) {
    match (target, source) {
        (Value::Object(target), Value::Object(source)) => {
            for (key, value) in source {
                match target.get_mut(&key) {
                    Some(existing) => merge_value(existing, value),
                    None => {
                        target.insert(key, value);
                    }
                }
            }
        }
        (Value::Array(target), Value::Array(source)) => {
            append_unique_values(target, source);
        }
        (target, source) => {
            *target = source;
        }
    }
}

fn append_unique_values(target: &mut Vec<Value>, source: Vec<Value>) {
    for value in source {
        if !target.iter().any(|existing| existing == &value) {
            target.push(value);
        }
    }
}

fn config_home_dir() -> PathBuf {
    if let Some(config_dir) = env::var_os("CLAUDE_CONFIG_DIR") {
        return PathBuf::from(config_dir);
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}

fn canonicalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}
