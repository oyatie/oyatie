use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::Command,
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{ClaudeAgentError, Result};

const LITE_READ_BUF_SIZE: u64 = 65_536;
const MAX_SANITIZED_LENGTH: usize = 200;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SDKSessionInfo {
    pub session_id: String,
    pub summary: String,
    pub last_modified: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_branch: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub uuid: String,
    pub session_id: String,
    pub message: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_tool_use_id: Option<String>,
}

/// Options for retrieving local session messages.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GetSessionMessagesOptions {
    pub directory: Option<PathBuf>,
    pub limit: Option<usize>,
    pub offset: usize,
    pub include_system_messages: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForkSessionResult {
    pub session_id: String,
}

pub(crate) struct LiteSessionFile {
    pub(crate) mtime: u64,
    pub(crate) size: u64,
    pub(crate) head: String,
    pub(crate) tail: String,
}

/// Return the Claude project key used under `~/.claude/projects/<key>`.
///
/// The Agent SDK stores local transcripts under a sanitized canonical working
/// directory path: every non-ASCII-alphanumeric character becomes `-`, matching
/// the Python/TypeScript SDK session helpers and Claude Code docs.
pub fn project_key_for_directory(directory: impl AsRef<Path>) -> String {
    let canonical = canonicalize_path(directory.as_ref());
    sanitize_path(&canonical.to_string_lossy())
}

/// List local Claude transcript sessions.
///
/// When `directory` is provided, only that project directory is scanned
/// (including git worktrees when `include_worktrees` is true). When omitted,
/// all project directories under `CLAUDE_CONFIG_DIR/projects` or
/// `~/.claude/projects` are scanned. Results are sorted by `last_modified`
/// descending and then paginated with `offset` and `limit`.
pub fn list_sessions(
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
    include_worktrees: bool,
) -> Result<Vec<SDKSessionInfo>> {
    let sessions = match directory {
        Some(directory) => list_sessions_for_project(directory, limit, offset, include_worktrees),
        None => list_all_sessions(limit, offset),
    };
    Ok(sessions)
}

/// Read metadata for a single local transcript session.
pub fn get_session_info(
    session_id: &str,
    directory: Option<&Path>,
) -> Result<Option<SDKSessionInfo>> {
    if !is_valid_uuid(session_id) {
        return Ok(None);
    }

    let filename = format!("{session_id}.jsonl");
    match directory {
        Some(directory) => {
            let canonical = canonicalize_path(directory);
            for project_path in project_lookup_paths(&canonical) {
                if let Some(project_dir) = find_project_dir(&project_path)
                    && let Some(lite) = read_session_lite(&project_dir.join(&filename))
                {
                    return Ok(parse_session_info_from_lite(
                        session_id,
                        &lite,
                        Some(&project_path.to_string_lossy()),
                    ));
                }
            }
            Ok(None)
        }
        None => {
            for project_dir in all_project_dirs() {
                if let Some(lite) = read_session_lite(&project_dir.join(&filename)) {
                    return Ok(parse_session_info_from_lite(session_id, &lite, None));
                }
            }
            Ok(None)
        }
    }
}

/// Read visible user/assistant messages from a local transcript.
///
/// This follows the same parent chain Claude Code would resume from: find the
/// newest terminal message, walk `parentUuid` back to the root, then return only
/// visible `user` and `assistant` messages after pagination.
pub fn get_session_messages(
    session_id: &str,
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SessionMessage>> {
    get_session_messages_inner(session_id, directory, limit, offset, false)
}

/// Read visible local transcript messages using an options object.
///
/// Defaults match [`get_session_messages`]: system messages are excluded unless
/// `include_system_messages` is true.
pub fn get_session_messages_with_options(
    session_id: &str,
    options: GetSessionMessagesOptions,
) -> Result<Vec<SessionMessage>> {
    get_session_messages_inner(
        session_id,
        options.directory.as_deref(),
        options.limit,
        options.offset,
        options.include_system_messages,
    )
}

fn get_session_messages_inner(
    session_id: &str,
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
    include_system_messages: bool,
) -> Result<Vec<SessionMessage>> {
    if !is_valid_uuid(session_id) {
        return Ok(Vec::new());
    }

    let Some(content) = read_session_file(session_id, directory) else {
        return Ok(Vec::new());
    };
    if content.is_empty() {
        return Ok(Vec::new());
    }

    let entries = parse_transcript_entries(&content);
    Ok(entries_to_session_messages_with_system(
        &entries,
        limit,
        offset,
        include_system_messages,
    ))
}

/// Rename a session by appending a `custom-title` metadata entry.
pub fn rename_session(session_id: &str, title: &str, directory: Option<&Path>) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }
    let title = title.trim();
    if title.is_empty() {
        return Err(ClaudeAgentError::InvalidOption(
            "title must be non-empty".into(),
        ));
    }

    #[derive(Serialize)]
    struct CustomTitleEntry<'a> {
        #[serde(rename = "type")]
        entry_type: &'static str,
        #[serde(rename = "customTitle")]
        custom_title: &'a str,
        #[serde(rename = "sessionId")]
        session_id: &'a str,
    }

    let entry = CustomTitleEntry {
        entry_type: "custom-title",
        custom_title: title,
        session_id,
    };
    append_to_session(
        session_id,
        &(serde_json::to_string(&entry)? + "\n"),
        directory,
    )
}

/// Tag a session by appending a `tag` metadata entry. Pass `None` to clear it.
pub fn tag_session(session_id: &str, tag: Option<&str>, directory: Option<&Path>) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }
    let tag = match tag {
        Some(tag) => {
            let sanitized = sanitize_unicode_tag(tag).trim().to_owned();
            if sanitized.is_empty() {
                return Err(ClaudeAgentError::InvalidOption(
                    "tag must be non-empty (use None to clear)".into(),
                ));
            }
            sanitized
        }
        None => String::new(),
    };

    #[derive(Serialize)]
    struct TagEntry<'a> {
        #[serde(rename = "type")]
        entry_type: &'static str,
        tag: &'a str,
        #[serde(rename = "sessionId")]
        session_id: &'a str,
    }

    let entry = TagEntry {
        entry_type: "tag",
        tag: &tag,
        session_id,
    };
    append_to_session(
        session_id,
        &(serde_json::to_string(&entry)? + "\n"),
        directory,
    )
}

/// Delete a local Claude transcript session and its subagent transcript directory.
///
/// Mirrors the TypeScript SDK package helper: removes `<session_id>.jsonl` and
/// the sibling `<session_id>/` directory under the resolved Claude project
/// transcript directory. Empty transcript files are ignored and treated as not
/// found.
pub fn delete_session(session_id: &str, directory: Option<&Path>) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }

    let filename = format!("{session_id}.jsonl");
    for project_dir in session_project_dirs(directory) {
        let session_path = project_dir.join(&filename);
        let metadata = match fs::metadata(&session_path) {
            Ok(metadata) => metadata,
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
                ) =>
            {
                continue;
            }
            Err(error) => return Err(error.into()),
        };
        if metadata.len() == 0 {
            continue;
        }
        fs::remove_file(&session_path)?;
        match fs::remove_dir_all(project_dir.join(session_id)) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        return Ok(());
    }

    Err(ClaudeAgentError::SessionNotFound {
        session_id: session_id.to_owned(),
    })
}

/// List subagent IDs recorded under a local session's subagent transcripts.
pub fn list_subagents(session_id: &str, directory: Option<&Path>) -> Result<Vec<String>> {
    if !is_valid_uuid(session_id) {
        return Ok(Vec::new());
    }
    let Some(session_file) = find_session_file(session_id, directory) else {
        return Ok(Vec::new());
    };
    let mut seen = BTreeMap::new();
    for file in list_subagent_transcript_files(&session_file) {
        if let Some(agent_id) = agent_id_from_subagent_path(&file) {
            seen.entry(agent_id).or_insert(());
        }
    }
    Ok(seen.into_keys().collect())
}

/// Read visible user/assistant messages from a local subagent transcript.
pub fn get_subagent_messages(
    session_id: &str,
    agent_id: &str,
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SessionMessage>> {
    if !is_valid_uuid(session_id) || agent_id.is_empty() {
        return Ok(Vec::new());
    }
    let Some(session_file) = find_session_file(session_id, directory) else {
        return Ok(Vec::new());
    };
    let Some(subagent_file) = list_subagent_transcript_files(&session_file)
        .into_iter()
        .find(|path| agent_id_from_subagent_path(path).as_deref() == Some(agent_id))
    else {
        return Ok(Vec::new());
    };
    let content = match fs::read_to_string(&subagent_file) {
        Ok(content) => content,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            return Ok(Vec::new());
        }
        Err(error) => return Err(error.into()),
    };
    let parent_tool_use_id = read_subagent_tool_use_id(&subagent_file);
    let entries = parse_transcript_entries(&content);
    Ok(entries_to_session_messages_with_parent_tool_use_id(
        &entries,
        limit,
        offset,
        parent_tool_use_id,
    ))
}

/// Fork a local transcript into a new resumable session with fresh UUIDs.
///
/// Copies non-sidechain transcript messages from `session_id`, optionally
/// truncating at `up_to_message_id`, rewrites message UUIDs and parent links,
/// and appends a custom title entry for the fork.
pub fn fork_session(
    session_id: &str,
    directory: Option<&Path>,
    up_to_message_id: Option<&str>,
    title: Option<&str>,
) -> Result<ForkSessionResult> {
    validate_fork_inputs(session_id, up_to_message_id)?;
    let Some(session_file) = find_session_file(session_id, directory) else {
        return Err(ClaudeAgentError::SessionNotFound {
            session_id: session_id.to_owned(),
        });
    };
    let content = fs::read_to_string(&session_file)?;
    let entries = parse_json_lines(&content).collect::<Vec<_>>();
    let (forked_entries, forked_session_id) =
        fork_session_entries(&entries, session_id, up_to_message_id, title)?;
    let output = forked_entries
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n")
        + "\n";
    let forked_path = session_file.with_file_name(format!("{forked_session_id}.jsonl"));
    fs::write(forked_path, output)?;
    Ok(ForkSessionResult {
        session_id: forked_session_id,
    })
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

fn projects_dir() -> PathBuf {
    config_home_dir().join("projects")
}

fn canonicalize_path(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn sanitize_path(name: &str) -> String {
    let sanitized = name
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>();
    if sanitized.len() <= MAX_SANITIZED_LENGTH {
        sanitized
    } else {
        format!(
            "{}-{}",
            &sanitized[..MAX_SANITIZED_LENGTH],
            simple_hash(name)
        )
    }
}

fn simple_hash(value: &str) -> String {
    let mut hash: i64 = 0;
    for ch in value.chars() {
        hash = ((hash << 5) - hash + ch as i64) & 0xffff_ffff;
        if hash >= 0x8000_0000 {
            hash -= 0x1_0000_0000;
        }
    }
    let mut number = hash.abs();
    if number == 0 {
        return "0".into();
    }

    const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
    let mut out = Vec::new();
    while number > 0 {
        out.push(DIGITS[(number % 36) as usize] as char);
        number /= 36;
    }
    out.iter().rev().collect()
}

fn find_project_dir(project_path: &Path) -> Option<PathBuf> {
    let project_path = project_path.to_string_lossy();
    let sanitized = sanitize_path(&project_path);
    let exact = projects_dir().join(&sanitized);
    if exact.is_dir() {
        return Some(exact);
    }
    if sanitized.len() <= MAX_SANITIZED_LENGTH {
        return None;
    }

    let prefix = sanitized[..MAX_SANITIZED_LENGTH].to_owned();
    for entry in fs::read_dir(projects_dir()).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir()
            && entry
                .file_name()
                .to_string_lossy()
                .starts_with(&(prefix.clone() + "-"))
        {
            return Some(path);
        }
    }
    None
}

fn all_project_dirs() -> Vec<PathBuf> {
    fs::read_dir(projects_dir())
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

fn project_lookup_paths(canonical: &Path) -> Vec<PathBuf> {
    let mut paths = vec![canonical.to_path_buf()];
    for worktree in get_worktree_paths(canonical) {
        if !paths.iter().any(|existing| existing == &worktree) {
            paths.push(worktree);
        }
    }
    paths
}

fn session_project_dirs(directory: Option<&Path>) -> Vec<PathBuf> {
    match directory {
        Some(directory) => {
            let canonical = canonicalize_path(directory);
            project_lookup_paths(&canonical)
                .into_iter()
                .filter_map(|project_path| find_project_dir(&project_path))
                .collect()
        }
        None => all_project_dirs(),
    }
}

pub(crate) fn find_session_file(session_id: &str, directory: Option<&Path>) -> Option<PathBuf> {
    if !is_valid_uuid(session_id) {
        return None;
    }
    let filename = format!("{session_id}.jsonl");
    session_project_dirs(directory)
        .into_iter()
        .map(|project_dir| project_dir.join(&filename))
        .find(|path| {
            fs::metadata(path)
                .map(|metadata| metadata.is_file() && metadata.len() > 0)
                .unwrap_or(false)
        })
}

pub(crate) fn list_subagent_transcript_files(session_file: &Path) -> Vec<PathBuf> {
    let mut session_dir = session_file.to_path_buf();
    session_dir.set_extension("");
    let subagents_dir = session_dir.join("subagents");
    let mut files = Vec::new();
    collect_subagent_transcript_files(&subagents_dir, &mut files);
    files.sort();
    files
}

fn collect_subagent_transcript_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_subagent_transcript_files(&path, files);
        } else if file_type.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("agent-") && name.ends_with(".jsonl"))
        {
            files.push(path);
        }
    }
}

fn agent_id_from_subagent_path(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .and_then(|stem| stem.strip_prefix("agent-"))
        .filter(|agent_id| !agent_id.is_empty())
        .map(ToOwned::to_owned)
}

fn read_subagent_tool_use_id(path: &Path) -> Option<String> {
    let metadata_path = path.with_extension("meta.json");
    let content = fs::read_to_string(metadata_path).ok()?;
    serde_json::from_str::<Value>(&content)
        .ok()?
        .get("toolUseId")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

pub(crate) fn validate_fork_inputs(session_id: &str, up_to_message_id: Option<&str>) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }
    if let Some(up_to_message_id) = up_to_message_id
        && !is_valid_uuid(up_to_message_id)
    {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid up_to_message_id: {up_to_message_id}"
        )));
    }
    Ok(())
}

pub(crate) fn fork_session_entries(
    entries: &[Value],
    source_session_id: &str,
    up_to_message_id: Option<&str>,
    title: Option<&str>,
) -> Result<(Vec<Value>, String)> {
    let mut transcript = Vec::new();
    let mut content_replacements = Vec::new();
    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };
        let entry_type = object.get("type").and_then(Value::as_str);
        if matches!(
            entry_type,
            Some("user" | "assistant" | "attachment" | "system" | "progress")
        ) && object.get("uuid").and_then(Value::as_str).is_some()
        {
            transcript.push(entry.clone());
        } else if entry_type == Some("content-replacement")
            && object.get("sessionId").and_then(Value::as_str) == Some(source_session_id)
            && let Some(replacements) = object.get("replacements").and_then(Value::as_array)
        {
            content_replacements.extend(replacements.iter().cloned());
        }
    }

    let mut selected = transcript
        .into_iter()
        .filter(|entry| entry.get("isSidechain").and_then(Value::as_bool) != Some(true))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "session {source_session_id} has no messages to fork"
        )));
    }
    if let Some(up_to_message_id) = up_to_message_id {
        let Some(index) = selected
            .iter()
            .position(|entry| entry.get("uuid").and_then(Value::as_str) == Some(up_to_message_id))
        else {
            return Err(ClaudeAgentError::InvalidOption(format!(
                "message {up_to_message_id} not found in session {source_session_id}"
            )));
        };
        selected.truncate(index + 1);
    }

    let output_sources = selected
        .iter()
        .filter(|entry| entry.get("type").and_then(Value::as_str) != Some("progress"))
        .collect::<Vec<_>>();
    if output_sources.is_empty() {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "session {source_session_id} has no messages to fork"
        )));
    }

    let uuid_map = selected
        .iter()
        .filter_map(|entry| {
            entry
                .get("uuid")
                .and_then(Value::as_str)
                .map(|uuid| (uuid.to_owned(), uuid::Uuid::new_v4().to_string()))
        })
        .collect::<BTreeMap<_, _>>();
    let by_uuid = selected
        .iter()
        .filter_map(|entry| {
            entry
                .get("uuid")
                .and_then(Value::as_str)
                .map(|uuid| (uuid.to_owned(), (*entry).clone()))
        })
        .collect::<BTreeMap<_, _>>();

    let forked_session_id = uuid::Uuid::new_v4().to_string();
    let now = current_timestamp_iso_utc();
    let mut forked = Vec::new();
    for (index, source) in output_sources.iter().enumerate() {
        let old_uuid = source.get("uuid").and_then(Value::as_str).ok_or_else(|| {
            ClaudeAgentError::MessageParse {
                message: "fork source entry missing uuid".into(),
                data: (*source).clone(),
            }
        })?;
        let new_uuid =
            uuid_map
                .get(old_uuid)
                .cloned()
                .ok_or_else(|| ClaudeAgentError::MessageParse {
                    message: "fork source entry missing remapped uuid".into(),
                    data: (*source).clone(),
                })?;
        let mut object =
            source
                .as_object()
                .cloned()
                .ok_or_else(|| ClaudeAgentError::MessageParse {
                    message: "fork source entry must be an object".into(),
                    data: (*source).clone(),
                })?;

        object.insert("uuid".into(), Value::String(new_uuid));
        object.insert("sessionId".into(), Value::String(forked_session_id.clone()));
        object.insert("isSidechain".into(), Value::Bool(false));
        object.insert(
            "forkedFrom".into(),
            serde_json::json!({
                "sessionId": source_session_id,
                "messageUuid": old_uuid,
            }),
        );
        if index + 1 == output_sources.len() {
            object.insert("timestamp".into(), Value::String(now.clone()));
        }
        match nearest_remapped_parent(source, &by_uuid, &uuid_map) {
            Some(parent_uuid) => {
                object.insert("parentUuid".into(), Value::String(parent_uuid));
            }
            None => {
                object.insert("parentUuid".into(), Value::Null);
            }
        }
        if let Some(logical_parent) = source.get("logicalParentUuid") {
            match logical_parent.as_str().and_then(|uuid| uuid_map.get(uuid)) {
                Some(remapped) => {
                    object.insert("logicalParentUuid".into(), Value::String(remapped.clone()));
                }
                None if logical_parent.is_null() => {
                    object.insert("logicalParentUuid".into(), Value::Null);
                }
                None => {
                    object.insert("logicalParentUuid".into(), Value::Null);
                }
            }
        }
        for key in [
            "teamName",
            "agentName",
            "sessionKind",
            "slug",
            "sourceToolAssistantUUID",
        ] {
            object.remove(key);
        }
        forked.push(Value::Object(object));
    }

    if !content_replacements.is_empty() {
        forked.push(serde_json::json!({
            "type": "content-replacement",
            "sessionId": forked_session_id,
            "replacements": content_replacements,
            "uuid": uuid::Uuid::new_v4().to_string(),
            "timestamp": now,
        }));
    }

    let title = title
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| {
            format!(
                "{} (fork)",
                infer_fork_title(entries).unwrap_or_else(|| "Forked session".into())
            )
        });
    forked.push(serde_json::json!({
        "type": "custom-title",
        "sessionId": forked_session_id,
        "customTitle": title,
        "uuid": uuid::Uuid::new_v4().to_string(),
        "timestamp": current_timestamp_iso_utc(),
    }));

    Ok((forked, forked_session_id))
}

fn nearest_remapped_parent(
    entry: &Value,
    by_uuid: &BTreeMap<String, Value>,
    uuid_map: &BTreeMap<String, String>,
) -> Option<String> {
    let mut parent = entry.get("parentUuid").and_then(Value::as_str);
    while let Some(parent_uuid) = parent {
        let parent_entry = by_uuid.get(parent_uuid)?;
        if parent_entry.get("type").and_then(Value::as_str) != Some("progress") {
            return uuid_map.get(parent_uuid).cloned();
        }
        parent = parent_entry.get("parentUuid").and_then(Value::as_str);
    }
    None
}

fn infer_fork_title(entries: &[Value]) -> Option<String> {
    entries
        .iter()
        .rev()
        .find_map(|entry| {
            entry
                .get("customTitle")
                .or_else(|| entry.get("aiTitle"))
                .and_then(Value::as_str)
                .filter(|title| !title.is_empty())
                .map(ToOwned::to_owned)
        })
        .or_else(|| first_prompt_from_entries(entries))
}

fn first_prompt_from_entries(entries: &[Value]) -> Option<String> {
    for entry in entries {
        if entry.get("type").and_then(Value::as_str) != Some("user")
            || entry.get("isMeta").and_then(Value::as_bool) == Some(true)
            || entry.get("isCompactSummary").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }
        let Some(content) = entry
            .get("message")
            .and_then(|message| message.get("content"))
        else {
            continue;
        };
        for text in content_texts(content) {
            let prompt = text.replace('\n', " ").trim().to_owned();
            if !prompt.is_empty()
                && extract_command_name(&prompt).is_none()
                && !should_skip_first_prompt(&prompt)
            {
                return Some(truncate_prompt(prompt));
            }
        }
    }
    None
}

fn content_texts(content: &Value) -> Vec<&str> {
    match content {
        Value::String(text) => vec![text.as_str()],
        Value::Array(blocks) => blocks
            .iter()
            .filter_map(|block| {
                (block.get("type").and_then(Value::as_str) == Some("text"))
                    .then(|| block.get("text").and_then(Value::as_str))
                    .flatten()
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn get_worktree_paths(cwd: &Path) -> Vec<PathBuf> {
    let Ok(output) = Command::new("git")
        .args(["worktree", "list", "--porcelain"])
        .current_dir(cwd)
        .output()
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(PathBuf::from)
        .collect()
}

fn read_session_lite(path: &Path) -> Option<LiteSessionFile> {
    let mut file = File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    let size = metadata.len();
    if size == 0 {
        return None;
    }

    let mtime = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default();

    let head_len = size.min(LITE_READ_BUF_SIZE) as usize;
    let mut head_bytes = vec![0; head_len];
    file.read_exact(&mut head_bytes).ok()?;
    let head = String::from_utf8_lossy(&head_bytes).into_owned();

    let tail = if size <= LITE_READ_BUF_SIZE {
        head.clone()
    } else {
        file.seek(SeekFrom::Start(size - LITE_READ_BUF_SIZE)).ok()?;
        let mut tail_bytes = Vec::with_capacity(LITE_READ_BUF_SIZE as usize);
        file.read_to_end(&mut tail_bytes).ok()?;
        String::from_utf8_lossy(&tail_bytes).into_owned()
    };

    Some(LiteSessionFile {
        mtime,
        size,
        head,
        tail,
    })
}

fn parse_json_lines(text: &str) -> impl Iterator<Item = Value> + '_ {
    text.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line.trim()).ok())
}

fn extract_json_string_field(text: &str, key: &str) -> Option<String> {
    parse_json_lines(text).find_map(|entry| {
        entry
            .get(key)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
    })
}

fn extract_last_json_string_field(text: &str, key: &str) -> Option<String> {
    parse_json_lines(text).fold(None, |last, entry| {
        entry
            .get(key)
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .or(last)
    })
}

fn extract_last_tag(tail: &str) -> Option<String> {
    for line in tail.lines().rev() {
        let Ok(entry) = serde_json::from_str::<Value>(line.trim()) else {
            continue;
        };
        if entry.get("type").and_then(Value::as_str) == Some("tag") {
            return entry
                .get("tag")
                .and_then(Value::as_str)
                .filter(|tag| !tag.is_empty())
                .map(ToOwned::to_owned);
        }
    }
    None
}

fn extract_first_prompt_from_head(head: &str) -> Option<String> {
    let mut command_fallback = None;

    for entry in parse_json_lines(head) {
        if entry.get("type").and_then(Value::as_str) != Some("user")
            || entry.get("isMeta").and_then(Value::as_bool) == Some(true)
            || entry.get("isCompactSummary").and_then(Value::as_bool) == Some(true)
        {
            continue;
        }

        let Some(message) = entry.get("message") else {
            continue;
        };
        let Some(content) = message.get("content") else {
            continue;
        };

        let mut texts = Vec::new();
        match content {
            Value::String(text) => texts.push(text.as_str()),
            Value::Array(blocks) => {
                if blocks
                    .iter()
                    .any(|block| block.get("type").and_then(Value::as_str) == Some("tool_result"))
                {
                    continue;
                }
                for block in blocks {
                    if block.get("type").and_then(Value::as_str) == Some("text")
                        && let Some(text) = block.get("text").and_then(Value::as_str)
                    {
                        texts.push(text);
                    }
                }
            }
            _ => {}
        }

        for raw in texts {
            let prompt = raw.replace('\n', " ").trim().to_owned();
            if prompt.is_empty() {
                continue;
            }
            if let Some(command_name) = extract_command_name(&prompt) {
                command_fallback.get_or_insert(command_name);
                continue;
            }
            if should_skip_first_prompt(&prompt) {
                continue;
            }
            return Some(truncate_prompt(prompt));
        }
    }

    command_fallback
}

fn extract_command_name(text: &str) -> Option<String> {
    let start = text.find("<command-name>")? + "<command-name>".len();
    let end = text[start..].find("</command-name>")? + start;
    Some(text[start..end].to_owned())
}

fn should_skip_first_prompt(prompt: &str) -> bool {
    let trimmed = prompt.trim();
    trimmed.starts_with("<local-command-stdout>")
        || trimmed.starts_with("<session-start-hook>")
        || trimmed.starts_with("<tick>")
        || trimmed.starts_with("<goal>")
        || trimmed.starts_with("[Request interrupted by user")
        || (trimmed.starts_with("<ide_opened_file>") && trimmed.ends_with("</ide_opened_file>"))
        || (trimmed.starts_with("<ide_selection>") && trimmed.ends_with("</ide_selection>"))
}

fn truncate_prompt(prompt: String) -> String {
    if prompt.chars().count() <= 200 {
        return prompt;
    }
    let truncated = prompt
        .chars()
        .take(200)
        .collect::<String>()
        .trim_end()
        .to_owned();
    truncated + "…"
}

pub(crate) fn parse_session_info_from_lite(
    session_id: &str,
    lite: &LiteSessionFile,
    project_path: Option<&str>,
) -> Option<SDKSessionInfo> {
    let first_line = lite.head.lines().next().unwrap_or_default();
    if first_line.contains("\"isSidechain\":true") || first_line.contains("\"isSidechain\": true") {
        return None;
    }

    let custom_title = extract_last_json_string_field(&lite.tail, "customTitle")
        .or_else(|| extract_last_json_string_field(&lite.head, "customTitle"))
        .or_else(|| extract_last_json_string_field(&lite.tail, "aiTitle"))
        .or_else(|| extract_last_json_string_field(&lite.head, "aiTitle"));
    let first_prompt = extract_first_prompt_from_head(&lite.head);
    let summary = custom_title
        .clone()
        .or_else(|| extract_last_json_string_field(&lite.tail, "lastPrompt"))
        .or_else(|| extract_last_json_string_field(&lite.tail, "summary"))
        .or_else(|| first_prompt.clone())?;

    let git_branch = extract_last_json_string_field(&lite.tail, "gitBranch")
        .or_else(|| extract_json_string_field(&lite.head, "gitBranch"));
    let cwd = extract_json_string_field(&lite.head, "cwd")
        .or_else(|| project_path.map(ToOwned::to_owned));
    let created_at = extract_json_string_field(&lite.head, "timestamp")
        .and_then(|timestamp| parse_iso_epoch_ms(&timestamp));

    Some(SDKSessionInfo {
        session_id: session_id.to_owned(),
        summary,
        last_modified: lite.mtime,
        file_size: Some(lite.size),
        custom_title,
        first_prompt,
        git_branch,
        cwd,
        tag: extract_last_tag(&lite.tail),
        created_at,
    })
}

fn list_sessions_for_project(
    directory: &Path,
    limit: Option<usize>,
    offset: usize,
    include_worktrees: bool,
) -> Vec<SDKSessionInfo> {
    let canonical = canonicalize_path(directory);
    let mut paths = vec![canonical.clone()];
    if include_worktrees {
        for worktree in get_worktree_paths(&canonical) {
            if !paths.iter().any(|path| path == &worktree) {
                paths.push(worktree);
            }
        }
    }

    let mut sessions = Vec::new();
    for path in paths {
        if let Some(project_dir) = find_project_dir(&path) {
            sessions.extend(read_sessions_from_dir(
                &project_dir,
                Some(&path.to_string_lossy()),
            ));
        }
    }
    apply_sort_limit_offset(deduplicate_by_session_id(sessions), limit, offset)
}

fn list_all_sessions(limit: Option<usize>, offset: usize) -> Vec<SDKSessionInfo> {
    let sessions = all_project_dirs()
        .into_iter()
        .flat_map(|project_dir| read_sessions_from_dir(&project_dir, None))
        .collect();
    apply_sort_limit_offset(deduplicate_by_session_id(sessions), limit, offset)
}

fn read_sessions_from_dir(project_dir: &Path, project_path: Option<&str>) -> Vec<SDKSessionInfo> {
    fs::read_dir(project_dir)
        .map(|entries| {
            entries
                .flatten()
                .filter_map(|entry| {
                    let path = entry.path();
                    let name = path.file_name()?.to_string_lossy();
                    let session_id = name.strip_suffix(".jsonl")?;
                    if !is_valid_uuid(session_id) {
                        return None;
                    }
                    let lite = read_session_lite(&path)?;
                    parse_session_info_from_lite(session_id, &lite, project_path)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn deduplicate_by_session_id(sessions: Vec<SDKSessionInfo>) -> Vec<SDKSessionInfo> {
    let mut by_id: BTreeMap<String, SDKSessionInfo> = BTreeMap::new();
    for session in sessions {
        match by_id.get(&session.session_id) {
            Some(existing) if existing.last_modified >= session.last_modified => {}
            _ => {
                by_id.insert(session.session_id.clone(), session);
            }
        }
    }
    by_id.into_values().collect()
}

fn apply_sort_limit_offset(
    mut sessions: Vec<SDKSessionInfo>,
    limit: Option<usize>,
    offset: usize,
) -> Vec<SDKSessionInfo> {
    sessions.sort_by_key(|session| std::cmp::Reverse(session.last_modified));
    let sessions = if offset > 0 {
        sessions.into_iter().skip(offset).collect()
    } else {
        sessions
    };
    match limit {
        Some(limit) if limit > 0 => sessions.into_iter().take(limit).collect(),
        _ => sessions,
    }
}

fn read_session_file(session_id: &str, directory: Option<&Path>) -> Option<String> {
    let filename = format!("{session_id}.jsonl");
    match directory {
        Some(directory) => {
            let canonical = canonicalize_path(directory);
            for path in project_lookup_paths(&canonical) {
                if let Some(project_dir) = find_project_dir(&path)
                    && let Ok(content) = fs::read_to_string(project_dir.join(&filename))
                    && !content.is_empty()
                {
                    return Some(content);
                }
            }
            None
        }
        None => all_project_dirs().into_iter().find_map(|project_dir| {
            fs::read_to_string(project_dir.join(&filename))
                .ok()
                .filter(|content| !content.is_empty())
        }),
    }
}

fn parse_transcript_entries(content: &str) -> Vec<Value> {
    parse_json_lines(content)
        .filter(|entry| {
            matches!(
                entry.get("type").and_then(Value::as_str),
                Some("user" | "assistant" | "progress" | "system" | "attachment")
            ) && entry.get("uuid").and_then(Value::as_str).is_some()
        })
        .collect()
}

pub(crate) fn entries_to_session_messages(
    entries: &[Value],
    limit: Option<usize>,
    offset: usize,
) -> Vec<SessionMessage> {
    entries_to_session_messages_with_parent_tool_use_id(entries, limit, offset, None)
}

pub(crate) fn entries_to_session_messages_with_system(
    entries: &[Value],
    limit: Option<usize>,
    offset: usize,
    include_system_messages: bool,
) -> Vec<SessionMessage> {
    entries_to_session_messages_inner(entries, limit, offset, None, include_system_messages)
}

pub(crate) fn entries_to_session_messages_with_parent_tool_use_id(
    entries: &[Value],
    limit: Option<usize>,
    offset: usize,
    parent_tool_use_id: Option<String>,
) -> Vec<SessionMessage> {
    entries_to_session_messages_inner(entries, limit, offset, parent_tool_use_id, false)
}

fn entries_to_session_messages_inner(
    entries: &[Value],
    limit: Option<usize>,
    offset: usize,
    parent_tool_use_id: Option<String>,
    include_system_messages: bool,
) -> Vec<SessionMessage> {
    let chain = build_conversation_chain(entries);
    let messages = chain
        .iter()
        .filter(|entry| is_visible_message(entry, include_system_messages))
        .map(|entry| to_session_message(entry, parent_tool_use_id.as_deref()))
        .collect::<Vec<_>>();

    let messages = if offset > 0 {
        messages.into_iter().skip(offset).collect()
    } else {
        messages
    };
    match limit {
        Some(limit) if limit > 0 => messages.into_iter().take(limit).collect(),
        _ => messages,
    }
}

fn build_conversation_chain(entries: &[Value]) -> Vec<Value> {
    if entries.is_empty() {
        return Vec::new();
    }

    let mut by_uuid: BTreeMap<String, Value> = BTreeMap::new();
    let mut entry_index: BTreeMap<String, usize> = BTreeMap::new();
    let mut parent_uuids = BTreeSet::new();

    for (index, entry) in entries.iter().enumerate() {
        if let Some(uuid) = entry.get("uuid").and_then(Value::as_str) {
            by_uuid.insert(uuid.to_owned(), entry.clone());
            entry_index.insert(uuid.to_owned(), index);
        }
        if let Some(parent) = entry.get("parentUuid").and_then(Value::as_str) {
            parent_uuids.insert(parent.to_owned());
        }
    }

    let mut leaves = Vec::new();
    for entry in entries {
        let Some(uuid) = entry.get("uuid").and_then(Value::as_str) else {
            continue;
        };
        if parent_uuids.contains(uuid) {
            continue;
        }

        let mut current = Some(entry.clone());
        let mut seen = BTreeSet::new();
        while let Some(candidate) = current {
            let Some(candidate_uuid) = candidate.get("uuid").and_then(Value::as_str) else {
                break;
            };
            if !seen.insert(candidate_uuid.to_owned()) {
                break;
            }
            if matches!(
                candidate.get("type").and_then(Value::as_str),
                Some("user" | "assistant")
            ) {
                leaves.push(candidate);
                break;
            }
            current = candidate
                .get("parentUuid")
                .and_then(Value::as_str)
                .and_then(|parent| by_uuid.get(parent).cloned());
        }
    }

    if leaves.is_empty() {
        return Vec::new();
    }

    let main_leaves = leaves
        .iter()
        .filter(|leaf| {
            leaf.get("isSidechain").and_then(Value::as_bool) != Some(true)
                && leaf.get("teamName").is_none()
                && leaf.get("isMeta").and_then(Value::as_bool) != Some(true)
        })
        .cloned()
        .collect::<Vec<_>>();
    let leaf = pick_best_leaf(
        if main_leaves.is_empty() {
            &leaves
        } else {
            &main_leaves
        },
        &entry_index,
    );

    let mut chain = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = Some(leaf);
    while let Some(entry) = current {
        let Some(uuid) = entry.get("uuid").and_then(Value::as_str) else {
            break;
        };
        if !seen.insert(uuid.to_owned()) {
            break;
        }
        chain.push(entry.clone());
        current = entry
            .get("parentUuid")
            .and_then(Value::as_str)
            .and_then(|parent| by_uuid.get(parent).cloned());
    }
    chain.reverse();
    chain
}

fn pick_best_leaf(leaves: &[Value], entry_index: &BTreeMap<String, usize>) -> Value {
    leaves
        .iter()
        .max_by_key(|leaf| {
            leaf.get("uuid")
                .and_then(Value::as_str)
                .and_then(|uuid| entry_index.get(uuid))
                .copied()
                .unwrap_or_default()
        })
        .cloned()
        .unwrap_or(Value::Null)
}

fn is_visible_message(entry: &Value, include_system_messages: bool) -> bool {
    let visible_type = match entry.get("type").and_then(Value::as_str) {
        Some("user" | "assistant") => true,
        Some("system") => include_system_messages,
        _ => false,
    };
    visible_type
        && entry.get("isMeta").and_then(Value::as_bool) != Some(true)
        && entry.get("isSidechain").and_then(Value::as_bool) != Some(true)
        && entry.get("teamName").is_none()
}

fn to_session_message(entry: &Value, parent_tool_use_id: Option<&str>) -> SessionMessage {
    SessionMessage {
        message_type: match entry.get("type").and_then(Value::as_str) {
            Some("user") => "user".into(),
            Some("system") => "system".into(),
            _ => "assistant".into(),
        },
        uuid: entry
            .get("uuid")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        session_id: entry
            .get("sessionId")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        message: entry.get("message").cloned().unwrap_or(Value::Null),
        parent_tool_use_id: parent_tool_use_id.map(ToOwned::to_owned),
    }
}

fn append_to_session(session_id: &str, data: &str, directory: Option<&Path>) -> Result<()> {
    let filename = format!("{session_id}.jsonl");
    match directory {
        Some(directory) => {
            let canonical = canonicalize_path(directory);
            for path in project_lookup_paths(&canonical) {
                if let Some(project_dir) = find_project_dir(&path)
                    && try_append(&project_dir.join(&filename), data)?
                {
                    return Ok(());
                }
            }
            Err(ClaudeAgentError::SessionNotFound {
                session_id: session_id.to_owned(),
            })
        }
        None => {
            for project_dir in all_project_dirs() {
                if try_append(&project_dir.join(&filename), data)? {
                    return Ok(());
                }
            }
            Err(ClaudeAgentError::SessionNotFound {
                session_id: session_id.to_owned(),
            })
        }
    }
}

fn try_append(path: &Path, data: &str) -> Result<bool> {
    let mut file = match OpenOptions::new().append(true).open(path) {
        Ok(file) => file,
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound | std::io::ErrorKind::InvalidInput
            ) =>
        {
            return Ok(false);
        }
        Err(error) => return Err(error.into()),
    };
    if file.metadata()?.len() == 0 {
        return Ok(false);
    }
    file.write_all(data.as_bytes())?;
    Ok(true)
}

pub(crate) fn is_valid_uuid(session_id: &str) -> bool {
    uuid::Uuid::parse_str(session_id).is_ok()
}

fn current_timestamp_iso_utc() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let seconds = now.as_secs() as i64;
    let millis = now.subsec_millis();
    let days = seconds.div_euclid(86_400);
    let seconds_of_day = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}Z")
}

fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    (year as i32, month as u32, day as u32)
}

pub(crate) fn sanitize_unicode_tag(value: &str) -> String {
    value
        .chars()
        .filter(|&ch| !is_stripped_unicode(ch))
        .collect()
}

fn is_stripped_unicode(ch: char) -> bool {
    matches!(
        ch as u32,
        0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x206f | 0xfeff | 0xe000..=0xf8ff
    )
}

pub(crate) fn parse_iso_epoch_ms(value: &str) -> Option<u64> {
    let (date, time_with_zone) = value.split_once('T')?;
    let mut date_parts = date.split('-');
    let year = date_parts.next()?.parse::<i32>().ok()?;
    let month = date_parts.next()?.parse::<u32>().ok()?;
    let day = date_parts.next()?.parse::<u32>().ok()?;

    let (time, offset_minutes) = split_time_zone(time_with_zone)?;
    let mut time_parts = time.split(':');
    let hour = time_parts.next()?.parse::<i64>().ok()?;
    let minute = time_parts.next()?.parse::<i64>().ok()?;
    let second_fraction = time_parts.next()?;
    let (second_text, fraction) = second_fraction
        .split_once('.')
        .unwrap_or((second_fraction, ""));
    let second = second_text.parse::<i64>().ok()?;
    let millis = fraction_millis(fraction);

    let days = days_from_civil(year, month, day)?;
    let seconds = days * 86_400 + hour * 3_600 + minute * 60 + second - offset_minutes as i64 * 60;
    if seconds < 0 {
        return None;
    }
    Some(seconds as u64 * 1_000 + millis)
}

fn split_time_zone(time: &str) -> Option<(&str, i32)> {
    if let Some(time) = time.strip_suffix('Z') {
        return Some((time, 0));
    }
    let zone_index = time
        .char_indices()
        .skip(1)
        .find_map(|(index, ch)| matches!(ch, '+' | '-').then_some(index));
    let Some(index) = zone_index else {
        return Some((time, 0));
    };
    let sign = if time.as_bytes()[index] == b'+' {
        1
    } else {
        -1
    };
    let zone = &time[index + 1..];
    let mut parts = zone.split(':');
    let hours = parts.next()?.parse::<i32>().ok()?;
    let minutes = parts.next().unwrap_or("0").parse::<i32>().ok()?;
    Some((&time[..index], sign * (hours * 60 + minutes)))
}

fn fraction_millis(fraction: &str) -> u64 {
    let mut digits = fraction
        .chars()
        .take(3)
        .filter_map(|ch| ch.to_digit(10))
        .collect::<Vec<_>>();
    while digits.len() < 3 {
        digits.push(0);
    }
    digits.into_iter().fold(0, |acc, digit| acc * 10 + digit) as u64
}

fn days_from_civil(year: i32, month: u32, day: u32) -> Option<i64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let year = year - i32::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let yoe = year - era * 400;
    let month = month as i32;
    let day = day as i32;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era as i64 * 146_097 + doe as i64 - 719_468)
}
