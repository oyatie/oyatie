use std::{env, ffi::OsString, fs, path::Path, sync::Mutex};

use intelligence_claude_agent_sdk::{
    GetSessionMessagesOptions, ImportSessionToStoreOptions, InMemorySessionStore, SessionKey,
    SessionStore, delete_session, fork_session, get_session_info, get_session_messages,
    get_session_messages_with_options, get_subagent_messages, import_session_to_store_with_options,
    list_sessions, list_subagents, project_key_for_directory, rename_session, tag_session,
};
use serde_json::{Value, json};
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

fn write_session(config_dir: &Path, project_dir: &Path, session_id: &str, lines: Vec<Value>) {
    let project_key = project_key_for_directory(project_dir);
    let session_dir = config_dir.join("projects").join(project_key);
    fs::create_dir_all(&session_dir).unwrap();
    let content = lines
        .into_iter()
        .map(|line| serde_json::to_string(&line).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(session_dir.join(format!("{session_id}.jsonl")), content).unwrap();
}

fn write_subagent_session(
    config_dir: &Path,
    project_dir: &Path,
    session_id: &str,
    subpath: &str,
    lines: Vec<Value>,
    metadata: Value,
) {
    let project_key = project_key_for_directory(project_dir);
    let session_dir = config_dir
        .join("projects")
        .join(project_key)
        .join(session_id)
        .join("subagents")
        .join(subpath);
    fs::create_dir_all(&session_dir).unwrap();
    let file_path = session_dir.join("agent-worker.jsonl");
    let content = lines
        .into_iter()
        .map(|line| serde_json::to_string(&line).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&file_path, content).unwrap();
    fs::write(
        file_path.with_extension("meta.json"),
        serde_json::to_string(&metadata).unwrap(),
    )
    .unwrap();
}

#[test]
fn project_key_matches_claude_transcript_directory_encoding() {
    assert_eq!(
        project_key_for_directory(Path::new("/Users/me/proj")),
        "-Users-me-proj"
    );
}

#[test]
fn lists_sessions_and_applies_metadata_mutations() -> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();
    let project_path = project.path().to_string_lossy().to_string();
    let session_id = "550e8400-e29b-41d4-a716-446655440000";

    write_session(
        config.path(),
        project.path(),
        session_id,
        vec![
            json!({
                "type": "user",
                "uuid": "00000000-0000-0000-0000-000000000001",
                "sessionId": session_id,
                "timestamp": "2026-06-03T01:02:03Z",
                "cwd": project_path,
                "gitBranch": "main",
                "message": {"role": "user", "content": "Build Rust SDK sessions"}
            }),
            json!({
                "type": "assistant",
                "uuid": "00000000-0000-0000-0000-000000000002",
                "parentUuid": "00000000-0000-0000-0000-000000000001",
                "sessionId": session_id,
                "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}
            }),
        ],
    );

    let sessions = list_sessions(Some(project.path()), Some(10), 0, false)?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, session_id);
    assert_eq!(sessions[0].summary, "Build Rust SDK sessions");
    assert_eq!(
        sessions[0].first_prompt.as_deref(),
        Some("Build Rust SDK sessions")
    );
    assert_eq!(sessions[0].git_branch.as_deref(), Some("main"));
    assert_eq!(sessions[0].cwd.as_deref(), Some(project_path.as_str()));
    assert!(sessions[0].file_size.is_some_and(|size| size > 0));
    assert!(sessions[0].created_at.is_some_and(|millis| millis > 0));

    rename_session(session_id, "  Refactor auth module  ", Some(project.path()))?;
    tag_session(
        session_id,
        Some("needs\u{200b}-review"),
        Some(project.path()),
    )?;

    let info = get_session_info(session_id, Some(project.path()))?.unwrap();
    assert_eq!(info.summary, "Refactor auth module");
    assert_eq!(info.custom_title.as_deref(), Some("Refactor auth module"));
    assert_eq!(info.tag.as_deref(), Some("needs-review"));

    tag_session(session_id, None, Some(project.path()))?;
    let info = get_session_info(session_id, Some(project.path()))?.unwrap();
    assert_eq!(info.tag, None);

    assert!(rename_session(session_id, "   ", Some(project.path())).is_err());
    assert!(tag_session(session_id, Some("\u{200b}"), Some(project.path())).is_err());
    Ok(())
}

#[test]
fn reads_visible_session_messages_in_parent_chain_with_pagination()
-> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440001";
    let u1 = "00000000-0000-0000-0000-000000000101";
    let a1 = "00000000-0000-0000-0000-000000000102";
    let u2 = "00000000-0000-0000-0000-000000000103";
    let a2 = "00000000-0000-0000-0000-000000000104";
    let s1 = "00000000-0000-0000-0000-000000000108";

    write_session(
        config.path(),
        project.path(),
        session_id,
        vec![
            json!({"type":"user","uuid":u1,"sessionId":session_id,"message":{"role":"user","content":"first"}}),
            json!({"type":"assistant","uuid":a1,"parentUuid":u1,"sessionId":session_id,"message":{"role":"assistant","content":[{"type":"text","text":"first answer"}]}}),
            json!({"type":"user","uuid":u2,"parentUuid":a1,"sessionId":session_id,"message":{"role":"user","content":[{"type":"text","text":"second"}]}}),
            json!({"type":"system","subtype":"compact_boundary","uuid":s1,"parentUuid":u2,"sessionId":session_id,"message":{"subtype":"compact_boundary"}}),
            json!({"type":"assistant","uuid":a2,"parentUuid":s1,"sessionId":session_id,"message":{"role":"assistant","content":[{"type":"text","text":"second answer"}]}}),
            json!({"type":"assistant","uuid":"00000000-0000-0000-0000-000000000105","parentUuid":a2,"sessionId":session_id,"isSidechain":true,"message":{"role":"assistant","content":[{"type":"text","text":"sidechain"}]}}),
            json!({"type":"user","uuid":"00000000-0000-0000-0000-000000000106","sessionId":session_id,"isMeta":true,"message":{"role":"user","content":"meta"}}),
            json!({"type":"progress","uuid":"00000000-0000-0000-0000-000000000107","parentUuid":a2,"sessionId":session_id}),
        ],
    );

    let messages = get_session_messages(session_id, Some(project.path()), None, 0)?;
    assert_eq!(messages.len(), 4);
    assert_eq!(
        messages
            .iter()
            .map(|message| message.message_type.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant", "user", "assistant"]
    );
    assert_eq!(messages[0].uuid, u1);
    assert_eq!(messages[3].uuid, a2);

    let with_system = get_session_messages_with_options(
        session_id,
        GetSessionMessagesOptions {
            directory: Some(project.path().to_path_buf()),
            include_system_messages: true,
            ..Default::default()
        },
    )?;
    assert_eq!(
        with_system
            .iter()
            .map(|message| message.message_type.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant", "user", "system", "assistant"]
    );
    assert_eq!(with_system[3].uuid, s1);
    assert_eq!(with_system[3].message["subtype"], json!("compact_boundary"));

    let page = get_session_messages(session_id, Some(project.path()), Some(2), 1)?;
    assert_eq!(
        page.iter()
            .map(|message| message.uuid.as_str())
            .collect::<Vec<_>>(),
        [a1, u2]
    );

    assert!(get_session_messages("not-a-uuid", Some(project.path()), None, 0)?.is_empty());
    assert!(get_session_info("not-a-uuid", Some(project.path()))?.is_none());
    Ok(())
}

#[test]
fn missing_mutation_target_reports_error() {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();

    let error = rename_session(
        "550e8400-e29b-41d4-a716-446655440099",
        "missing",
        Some(project.path()),
    )
    .unwrap_err()
    .to_string();
    assert!(error.contains("session not found"));
}

#[test]
fn local_subagent_helpers_and_delete_match_package_exports()
-> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440002";
    let root = "00000000-0000-0000-0000-000000000201";
    let reply = "00000000-0000-0000-0000-000000000202";

    write_session(
        config.path(),
        project.path(),
        session_id,
        vec![json!({
            "type": "user",
            "uuid": "00000000-0000-0000-0000-000000000200",
            "sessionId": session_id,
            "message": {"role": "user", "content": "main"}
        })],
    );
    write_subagent_session(
        config.path(),
        project.path(),
        session_id,
        "nested",
        vec![
            json!({
                "type": "user",
                "uuid": root,
                "sessionId": session_id,
                "message": {"role": "user", "content": "subtask"}
            }),
            json!({
                "type": "assistant",
                "uuid": reply,
                "parentUuid": root,
                "sessionId": session_id,
                "message": {"role": "assistant", "content": [{"type": "text", "text": "done"}]}
            }),
        ],
        json!({"toolUseId": "toolu_worker", "agentName": "worker"}),
    );

    assert_eq!(
        list_subagents(session_id, Some(project.path()))?,
        vec!["worker"]
    );
    let messages = get_subagent_messages(session_id, "worker", Some(project.path()), None, 0)?;
    assert_eq!(
        messages
            .iter()
            .map(|message| message.uuid.as_str())
            .collect::<Vec<_>>(),
        [root, reply]
    );
    assert_eq!(
        messages[0].parent_tool_use_id.as_deref(),
        Some("toolu_worker")
    );
    assert!(
        get_subagent_messages(session_id, "missing", Some(project.path()), None, 0)?.is_empty()
    );

    delete_session(session_id, Some(project.path()))?;
    assert!(get_session_info(session_id, Some(project.path()))?.is_none());
    let project_key = project_key_for_directory(project.path());
    let project_dir = config.path().join("projects").join(project_key);
    assert!(!project_dir.join(format!("{session_id}.jsonl")).exists());
    assert!(!project_dir.join(session_id).exists());

    let error = delete_session(session_id, Some(project.path()))
        .unwrap_err()
        .to_string();
    assert!(error.contains("session not found"));
    Ok(())
}

#[test]
fn forks_local_session_with_new_ids_and_parent_chain() -> intelligence_claude_agent_sdk::Result<()>
{
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440004";
    let u1 = "00000000-0000-0000-0000-000000000401";
    let a1 = "00000000-0000-0000-0000-000000000402";
    let u2 = "00000000-0000-0000-0000-000000000403";

    write_session(
        config.path(),
        project.path(),
        session_id,
        vec![
            json!({
                "type": "user",
                "uuid": u1,
                "sessionId": session_id,
                "timestamp": "2026-06-03T01:00:00Z",
                "message": {"role": "user", "content": "first"}
            }),
            json!({
                "type": "progress",
                "uuid": "00000000-0000-0000-0000-000000000499",
                "parentUuid": u1,
                "sessionId": session_id
            }),
            json!({
                "type": "assistant",
                "uuid": a1,
                "parentUuid": "00000000-0000-0000-0000-000000000499",
                "sessionId": session_id,
                "timestamp": "2026-06-03T01:00:01Z",
                "message": {"role": "assistant", "content": [{"type": "text", "text": "answer"}]}
            }),
            json!({
                "type": "user",
                "uuid": u2,
                "parentUuid": a1,
                "sessionId": session_id,
                "message": {"role": "user", "content": "not copied"}
            }),
        ],
    );

    let fork = fork_session(
        session_id,
        Some(project.path()),
        Some(a1),
        Some("Branch title"),
    )?;
    assert_ne!(fork.session_id, session_id);
    uuid::Uuid::parse_str(&fork.session_id).unwrap();

    let fork_messages = get_session_messages(&fork.session_id, Some(project.path()), None, 0)?;
    assert_eq!(
        fork_messages
            .iter()
            .map(|message| message.message_type.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant"]
    );
    assert_eq!(fork_messages[0].session_id, fork.session_id);
    assert_ne!(fork_messages[0].uuid, u1);
    assert_ne!(fork_messages[1].uuid, a1);

    let info = get_session_info(&fork.session_id, Some(project.path()))?.unwrap();
    assert_eq!(info.custom_title.as_deref(), Some("Branch title"));

    let project_key = project_key_for_directory(project.path());
    let fork_path = config
        .path()
        .join("projects")
        .join(project_key)
        .join(format!("{}.jsonl", fork.session_id));
    let raw = fs::read_to_string(fork_path).unwrap();
    let lines = raw
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[1]["parentUuid"], lines[0]["uuid"]);
    assert_eq!(lines[0]["forkedFrom"]["sessionId"], json!(session_id));
    assert_eq!(lines[1]["forkedFrom"]["messageUuid"], json!(a1));
    assert_eq!(lines[2]["type"], json!("custom-title"));
    Ok(())
}

#[test]
fn imports_local_session_and_subagents_to_store() -> intelligence_claude_agent_sdk::Result<()> {
    let _lock = ENV_LOCK.lock().unwrap();
    let config = tempdir().unwrap();
    let _env = ClaudeConfigEnvGuard::set(config.path());
    let project = tempdir().unwrap();
    let session_id = "550e8400-e29b-41d4-a716-446655440003";

    write_session(
        config.path(),
        project.path(),
        session_id,
        vec![
            json!({
                "type": "user",
                "uuid": "00000000-0000-0000-0000-000000000301",
                "sessionId": session_id,
                "message": {"role": "user", "content": "import me"}
            }),
            json!({
                "type": "assistant",
                "uuid": "00000000-0000-0000-0000-000000000302",
                "parentUuid": "00000000-0000-0000-0000-000000000301",
                "sessionId": session_id,
                "message": {"role": "assistant", "content": [{"type": "text", "text": "imported"}]}
            }),
        ],
    );
    write_subagent_session(
        config.path(),
        project.path(),
        session_id,
        "nested",
        vec![json!({
            "type": "assistant",
            "uuid": "00000000-0000-0000-0000-000000000303",
            "sessionId": session_id,
            "message": {"role": "assistant", "content": [{"type": "text", "text": "sub"}]}
        })],
        json!({"toolUseId": "toolu_import", "agentName": "worker"}),
    );

    let store = InMemorySessionStore::default();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .unwrap();
    runtime.block_on(import_session_to_store_with_options(
        session_id,
        &store,
        ImportSessionToStoreOptions {
            directory: Some(project.path().to_path_buf()),
            batch_size: Some(1),
            ..Default::default()
        },
    ))?;

    let project_key = project_key_for_directory(project.path());
    let main = runtime
        .block_on(store.load(SessionKey::new(project_key.clone(), session_id)))?
        .unwrap();
    assert_eq!(main.len(), 2);
    assert_eq!(main[0]["message"]["content"], json!("import me"));

    let subagent = runtime
        .block_on(store.load(SessionKey::with_subpath(
            project_key,
            session_id,
            "subagents/nested/agent-worker",
        )))?
        .unwrap();
    assert_eq!(subagent.len(), 2);
    assert_eq!(subagent[0]["message"]["content"][0]["text"], json!("sub"));
    assert_eq!(subagent[1]["type"], json!("agent_metadata"));
    assert_eq!(subagent[1]["toolUseId"], json!("toolu_import"));
    Ok(())
}
