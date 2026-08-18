#[path = "support_fake_cli.rs"]
mod support;
use support::{expect_json_line, fake_cli, read_json_line, write_json_line};

use futures::StreamExt;
use intelligence_claude_agent_sdk::{
    ClaudeAgentOptions, FoldSessionSummaryOptions, ForkSessionOptions, GetSessionInfoOptions,
    GetSessionMessagesOptions, GetSubagentMessagesOptions, InMemorySessionStore,
    ListSessionsOptions, ListSubagentsOptions, Message, SessionKey, SessionMutationOptions,
    SessionStore, SharedSessionStore, delete_session_via_store, delete_session_with_options,
    fold_session_summary, fold_session_summary_with_options, fork_session_via_store,
    fork_session_with_options, get_session_info_from_store, get_session_info_with_options,
    get_session_messages_from_store, get_session_messages_from_store_with_options,
    get_subagent_messages_with_options, list_sessions_from_store, list_sessions_with_options,
    list_subagents_with_options, project_key_for_directory, query, rename_session_via_store,
    rename_session_with_options, tag_session_via_store, tag_session_with_options,
};
use serde_json::json;
use tempfile::tempdir;

const SESSION_ID: &str = "550e8400-e29b-41d4-a716-446655440010";

#[test]
fn fold_session_summary_mtime_is_adapter_stamped_and_preserved() {
    let key = SessionKey::new("proj", SESSION_ID);
    let first_entry = json!({
        "type": "user",
        "uuid": "00000000-0000-0000-0000-000000000101",
        "sessionId": SESSION_ID,
        "timestamp": "2026-06-03T01:02:03Z",
        "message": {"role": "user", "content": "first prompt"}
    });
    let retitle_entry = json!({
        "type": "custom-title",
        "uuid": "00000000-0000-0000-0000-000000000102",
        "sessionId": SESSION_ID,
        "customTitle": "Renamed"
    });

    let summary = fold_session_summary_with_options(
        None,
        &key,
        &[first_entry],
        FoldSessionSummaryOptions { mtime: Some(1234) },
    );
    assert_eq!(summary.mtime, 1234);
    assert_eq!(summary.data["first_prompt"], json!("first prompt"));

    let preserved = fold_session_summary(Some(&summary), &key, &[]);
    assert_eq!(preserved.mtime, 1234);

    let restamped = fold_session_summary_with_options(
        Some(&summary),
        &key,
        &[retitle_entry],
        FoldSessionSummaryOptions { mtime: Some(5678) },
    );
    assert_eq!(restamped.mtime, 5678);
    assert_eq!(restamped.data["custom_title"], json!("Renamed"));
}

#[tokio::test]
async fn in_memory_session_store_matches_core_contracts()
-> intelligence_claude_agent_sdk::Result<()> {
    let store = InMemorySessionStore::default();
    let key = SessionKey::new("proj", SESSION_ID);
    let subkey = SessionKey::with_subpath("proj", SESSION_ID, "subagents/agent-a");

    assert_eq!(store.load(key.clone()).await?, None);
    assert_eq!(store.load(subkey.clone()).await?, None);

    let user = json!({
        "type": "user",
        "uuid": "00000000-0000-0000-0000-000000000201",
        "sessionId": SESSION_ID,
        "timestamp": "2026-06-03T01:02:03Z",
        "cwd": "/tmp/proj",
        "gitBranch": "main",
        "message": {"role": "user", "content": "Build store parity"}
    });
    let assistant = json!({
        "type": "assistant",
        "uuid": "00000000-0000-0000-0000-000000000202",
        "parentUuid": "00000000-0000-0000-0000-000000000201",
        "sessionId": SESSION_ID,
        "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}
    });

    store.append(key.clone(), vec![user.clone()]).await?;
    store.append(key.clone(), vec![assistant.clone()]).await?;
    assert_eq!(
        store.load(key.clone()).await?.unwrap(),
        vec![user.clone(), assistant.clone()]
    );

    store.append(key.clone(), vec![]).await?;
    assert_eq!(
        store.load(key.clone()).await?.unwrap(),
        vec![user.clone(), assistant.clone()]
    );

    let subagent_entry = json!({
        "type": "assistant",
        "uuid": "00000000-0000-0000-0000-000000000203",
        "sessionId": SESSION_ID,
        "message": {"role": "assistant", "content": [{"type": "text", "text": "sub"}]}
    });
    store
        .append(subkey.clone(), vec![subagent_entry.clone()])
        .await?;
    store
        .append(
            SessionKey::new("other", SESSION_ID),
            vec![json!({"type": "user"})],
        )
        .await?;

    let listing = store.list_sessions("proj").await?.unwrap();
    assert_eq!(listing.len(), 1);
    assert_eq!(listing[0].session_id, SESSION_ID);
    assert!(listing[0].mtime > 1_000_000_000_000);

    let subkeys = store
        .list_subkeys(SessionKey::new("proj", SESSION_ID))
        .await?
        .unwrap();
    assert_eq!(subkeys, vec!["subagents/agent-a"]);

    let summaries = store.list_session_summaries("proj").await?.unwrap();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].session_id, SESSION_ID);
    assert_eq!(
        summaries[0].data["first_prompt"],
        json!("Build store parity")
    );
    assert_eq!(summaries[0].mtime, listing[0].mtime);

    store.delete(subkey.clone()).await?;
    assert_eq!(store.load(subkey.clone()).await?, None);
    assert!(store.load(key.clone()).await?.is_some());

    store.append(subkey.clone(), vec![subagent_entry]).await?;
    store.delete(key.clone()).await?;
    assert_eq!(store.load(key).await?, None);
    assert_eq!(store.load(subkey).await?, None);
    Ok(())
}

#[tokio::test]
async fn store_backed_session_helpers_roundtrip() -> intelligence_claude_agent_sdk::Result<()> {
    let store = InMemorySessionStore::default();
    let project = tempdir().unwrap();
    let project_key = project_key_for_directory(project.path());
    let key = SessionKey::new(project_key, SESSION_ID);

    store
        .append(
            key,
            vec![
                json!({
                    "type": "user",
                    "uuid": "00000000-0000-0000-0000-000000000301",
                    "sessionId": SESSION_ID,
                    "timestamp": "2026-06-03T01:02:03Z",
                    "cwd": project.path().to_string_lossy(),
                    "gitBranch": "main",
                    "message": {"role": "user", "content": "List sessions from store"}
                }),
                json!({
                    "type": "system",
                    "subtype": "compact_boundary",
                    "uuid": "00000000-0000-0000-0000-000000000303",
                    "parentUuid": "00000000-0000-0000-0000-000000000301",
                    "sessionId": SESSION_ID,
                    "message": {"subtype": "compact_boundary"}
                }),
                json!({
                    "type": "assistant",
                    "uuid": "00000000-0000-0000-0000-000000000302",
                    "parentUuid": "00000000-0000-0000-0000-000000000303",
                    "sessionId": SESSION_ID,
                    "message": {"role": "assistant", "content": [{"type": "text", "text": "done"}]}
                }),
            ],
        )
        .await?;

    let sessions = list_sessions_from_store(&store, Some(project.path()), Some(10), 0).await?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].session_id, SESSION_ID);
    assert_eq!(sessions[0].summary, "List sessions from store");
    assert_eq!(sessions[0].git_branch.as_deref(), Some("main"));

    let messages =
        get_session_messages_from_store(&store, SESSION_ID, Some(project.path()), None, 0).await?;
    assert_eq!(
        messages
            .iter()
            .map(|message| message.message_type.as_str())
            .collect::<Vec<_>>(),
        ["user", "assistant"]
    );

    let with_system = get_session_messages_from_store_with_options(
        &store,
        SESSION_ID,
        GetSessionMessagesOptions {
            directory: Some(project.path().to_path_buf()),
            include_system_messages: true,
            ..Default::default()
        },
    )
    .await?;
    assert_eq!(
        with_system
            .iter()
            .map(|message| message.message_type.as_str())
            .collect::<Vec<_>>(),
        ["user", "system", "assistant"]
    );
    assert_eq!(with_system[1].message["subtype"], json!("compact_boundary"));

    rename_session_via_store(&store, SESSION_ID, "  Store title  ", Some(project.path())).await?;
    tag_session_via_store(
        &store,
        SESSION_ID,
        Some("needs\u{200b}-review"),
        Some(project.path()),
    )
    .await?;

    let info = get_session_info_from_store(&store, SESSION_ID, Some(project.path()))
        .await?
        .unwrap();
    assert_eq!(info.summary, "Store title");
    assert_eq!(info.custom_title.as_deref(), Some("Store title"));
    assert_eq!(info.tag.as_deref(), Some("needs-review"));

    tag_session_via_store(&store, SESSION_ID, None, Some(project.path())).await?;
    let info = get_session_info_from_store(&store, SESSION_ID, Some(project.path()))
        .await?
        .unwrap();
    assert_eq!(info.tag, None);

    let missing = rename_session_via_store(
        &store,
        "550e8400-e29b-41d4-a716-446655440099",
        "missing",
        Some(project.path()),
    )
    .await
    .unwrap_err()
    .to_string();
    assert!(missing.contains("session not found"));

    delete_session_via_store(&store, SESSION_ID, Some(project.path())).await?;
    assert!(
        get_session_info_from_store(&store, SESSION_ID, Some(project.path()))
            .await?
            .is_none()
    );
    let missing_delete = delete_session_via_store(&store, SESSION_ID, Some(project.path()))
        .await
        .unwrap_err()
        .to_string();
    assert!(missing_delete.contains("session not found"));
    Ok(())
}

#[tokio::test]
async fn store_backed_fork_session_remaps_transcript_entries()
-> intelligence_claude_agent_sdk::Result<()> {
    let store = InMemorySessionStore::default();
    let project = tempdir().unwrap();
    let project_key = project_key_for_directory(project.path());
    let key = SessionKey::new(project_key.clone(), SESSION_ID);
    let u1 = "00000000-0000-0000-0000-000000000501";
    let a1 = "00000000-0000-0000-0000-000000000502";

    store
        .append(
            key,
            vec![
                json!({
                    "type": "user",
                    "uuid": u1,
                    "sessionId": SESSION_ID,
                    "message": {"role": "user", "content": "fork from store"}
                }),
                json!({
                    "type": "assistant",
                    "uuid": a1,
                    "parentUuid": u1,
                    "sessionId": SESSION_ID,
                    "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}
                }),
            ],
        )
        .await?;

    let fork = fork_session_via_store(
        &store,
        SESSION_ID,
        Some(project.path()),
        None,
        Some("Store fork"),
    )
    .await?;
    assert_ne!(fork.session_id, SESSION_ID);
    uuid::Uuid::parse_str(&fork.session_id).unwrap();

    let forked = store
        .load(SessionKey::new(project_key, &fork.session_id))
        .await?
        .unwrap();
    assert_eq!(forked.len(), 3);
    assert_eq!(forked[0]["sessionId"], json!(fork.session_id));
    assert_ne!(forked[0]["uuid"], json!(u1));
    assert_eq!(forked[1]["parentUuid"], forked[0]["uuid"]);
    assert_eq!(forked[0]["forkedFrom"]["sessionId"], json!(SESSION_ID));
    assert_eq!(forked[1]["forkedFrom"]["messageUuid"], json!(a1));
    assert_eq!(forked[2]["customTitle"], json!("Store fork"));
    Ok(())
}

#[tokio::test]
async fn session_mutation_options_route_to_store_helpers()
-> intelligence_claude_agent_sdk::Result<()> {
    let store = InMemorySessionStore::default();
    let project = tempdir().unwrap();
    let project_key = project_key_for_directory(project.path());
    let u1 = "00000000-0000-0000-0000-000000000701";
    let a1 = "00000000-0000-0000-0000-000000000702";

    store
        .append(
            SessionKey::new(project_key.clone(), SESSION_ID),
            vec![
                json!({
                    "type": "user",
                    "uuid": u1,
                    "sessionId": SESSION_ID,
                    "message": {"role": "user", "content": "options wrapper"}
                }),
                json!({
                    "type": "assistant",
                    "uuid": a1,
                    "parentUuid": u1,
                    "sessionId": SESSION_ID,
                    "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}
                }),
            ],
        )
        .await?;

    let mutation_options = SessionMutationOptions {
        directory: Some(project.path().to_path_buf()),
        session_store: Some(SharedSessionStore::new(store.clone())),
    };

    rename_session_with_options(SESSION_ID, "  Options title  ", mutation_options.clone()).await?;
    tag_session_with_options(SESSION_ID, Some("options-tag"), mutation_options.clone()).await?;
    let info = get_session_info_from_store(&store, SESSION_ID, Some(project.path()))
        .await?
        .unwrap();
    assert_eq!(info.custom_title.as_deref(), Some("Options title"));
    assert_eq!(info.tag.as_deref(), Some("options-tag"));

    let fork = fork_session_with_options(
        SESSION_ID,
        ForkSessionOptions {
            directory: Some(project.path().to_path_buf()),
            session_store: Some(SharedSessionStore::new(store.clone())),
            title: Some("Options fork".into()),
            ..Default::default()
        },
    )
    .await?;
    assert_ne!(fork.session_id, SESSION_ID);
    let fork_info = get_session_info_from_store(&store, &fork.session_id, Some(project.path()))
        .await?
        .unwrap();
    assert_eq!(fork_info.custom_title.as_deref(), Some("Options fork"));

    delete_session_with_options(SESSION_ID, mutation_options).await?;
    assert!(
        get_session_info_from_store(&store, SESSION_ID, Some(project.path()))
            .await?
            .is_none()
    );
    assert!(
        get_session_info_from_store(&store, &fork.session_id, Some(project.path()))
            .await?
            .is_some()
    );
    Ok(())
}

#[tokio::test]
async fn session_read_options_route_to_store_helpers() -> intelligence_claude_agent_sdk::Result<()>
{
    let store = InMemorySessionStore::default();
    let project = tempdir().unwrap();
    let project_key = project_key_for_directory(project.path());
    let u1 = "00000000-0000-0000-0000-000000000801";
    let a1 = "00000000-0000-0000-0000-000000000802";

    store
        .append(
            SessionKey::new(project_key.clone(), SESSION_ID),
            vec![
                json!({
                    "type": "user",
                    "uuid": u1,
                    "sessionId": SESSION_ID,
                    "timestamp": "2026-06-03T01:02:03Z",
                    "cwd": project.path().to_string_lossy(),
                    "message": {"role": "user", "content": "options list"}
                }),
                json!({
                    "type": "assistant",
                    "uuid": a1,
                    "parentUuid": u1,
                    "sessionId": SESSION_ID,
                    "message": {"role": "assistant", "content": [{"type": "text", "text": "ok"}]}
                }),
            ],
        )
        .await?;
    store
        .append(
            SessionKey::with_subpath(project_key, SESSION_ID, "subagents/agent-worker"),
            vec![json!({
                "type": "assistant",
                "uuid": "00000000-0000-0000-0000-000000000803",
                "sessionId": SESSION_ID,
                "message": {"role": "assistant", "content": [{"type": "text", "text": "sub"}]}
            })],
        )
        .await?;

    let session_store = SharedSessionStore::new(store.clone());
    let sessions = list_sessions_with_options(ListSessionsOptions {
        directory: Some(project.path().to_path_buf()),
        limit: Some(5),
        session_store: Some(session_store.clone()),
        ..Default::default()
    })
    .await?;
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].summary, "options list");

    let info = get_session_info_with_options(
        SESSION_ID,
        GetSessionInfoOptions {
            directory: Some(project.path().to_path_buf()),
            session_store: Some(session_store.clone()),
        },
    )
    .await?
    .unwrap();
    assert_eq!(info.first_prompt.as_deref(), Some("options list"));

    let subagents = list_subagents_with_options(
        SESSION_ID,
        ListSubagentsOptions {
            directory: Some(project.path().to_path_buf()),
            session_store: Some(session_store.clone()),
        },
    )
    .await?;
    assert_eq!(subagents, vec!["worker"]);

    let messages = get_subagent_messages_with_options(
        SESSION_ID,
        "worker",
        GetSubagentMessagesOptions {
            directory: Some(project.path().to_path_buf()),
            limit: Some(1),
            session_store: Some(session_store),
            ..Default::default()
        },
    )
    .await?;
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].message["content"][0]["text"], json!("sub"));
    Ok(())
}

#[test]
fn session_store_options_emit_mirror_and_reject_file_checkpointing() {
    let options = ClaudeAgentOptions::builder()
        .session_store(InMemorySessionStore::default())
        .build();
    let args = options.to_cli_args().unwrap();
    assert!(args.iter().any(|arg| arg == "--session-mirror"));

    let mut invalid = options;
    invalid.enable_file_checkpointing = true;
    let error = invalid.to_cli_args().unwrap_err().to_string();
    assert!(error.contains("session_store cannot be combined"));
}

#[tokio::test]
async fn query_mirrors_transcript_frames_without_yielding_them()
-> intelligence_claude_agent_sdk::Result<()> {
    let config = tempdir().unwrap();
    let config_dir = config.path().to_string_lossy().into_owned();
    let store = InMemorySessionStore::default();
    let store_clone = store.clone();
    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(move |mut r, mut w, opts| {
            let store = store_clone.clone();
            async move {
                // The SDK must pass --session-mirror in args
                assert!(opts.args.iter().any(|a| a == "--session-mirror"), "expected --session-mirror in args");
                let init = expect_json_line(&mut r).await;
                write_json_line(&mut w, &json!({
                    "type":"control_response",
                    "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
                })).await;
                let user = expect_json_line(&mut r).await;
                let content = user["message"]["content"].clone();
                // The filePath must be under CLAUDE_CONFIG_DIR/projects/<key>/<session>.jsonl
                // so the SDK's file_path_to_session_key can parse it.
                let config_dir = opts.env.get("CLAUDE_CONFIG_DIR").cloned()
                    .expect("CLAUDE_CONFIG_DIR must be set in spawn env");
                let file_path = format!("{}/projects/myproj/{}.jsonl", config_dir, SESSION_ID);
                let frame = json!({
                    "type":"transcript_mirror",
                    "filePath": file_path,
                    "entries":[{"type":"user","uuid":"u1","sessionId":SESSION_ID,"message":{"content":content}}]
                });
                write_json_line(&mut w, &frame).await;
                write_json_line(&mut w, &json!({
                    "type":"assistant","session_id":SESSION_ID,
                    "message":{"model":"claude-test","content":[{"type":"text","text":"ok"}]}
                })).await;
                write_json_line(&mut w, &frame).await;
                write_json_line(&mut w, &json!({
                    "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                    "is_error":false,"num_turns":1,"session_id":SESSION_ID,"result":"done"
                })).await;
                // Wait for the SDK to process the mirror frames before the task exits
                while read_json_line(&mut r).await.is_some() {}
                drop(store);
            }
        }))
        .env("CLAUDE_CONFIG_DIR", config_dir)
        .session_store(store.clone())
        .build();
    let mut stream = query("hello", options)?;

    let first = stream.next().await.unwrap()?;
    assert!(matches!(first, Message::Assistant(_)));
    let second = stream.next().await.unwrap()?;
    assert!(matches!(second, Message::Result(_)));
    assert!(stream.next().await.is_none());

    let mirrored = store
        .load(SessionKey::new("myproj", SESSION_ID))
        .await?
        .unwrap();
    assert_eq!(mirrored.len(), 2);
    assert_eq!(mirrored[0]["message"]["content"], json!("hello"));
    Ok(())
}

#[tokio::test]
async fn resume_materializes_store_session_before_spawn()
-> intelligence_claude_agent_sdk::Result<()> {
    let project = tempdir().unwrap();
    let store = InMemorySessionStore::default();
    store
        .append(
            SessionKey::new(project_key_for_directory(project.path()), SESSION_ID),
            vec![json!({
                "type": "user",
                "uuid": "00000000-0000-0000-0000-000000000401",
                "sessionId": SESSION_ID,
                "message": {"role": "user", "content": "resume me"}
            })],
        )
        .await?;

    let options = ClaudeAgentOptions::builder()
        .spawn_claude_code_process(fake_cli(|mut r, mut w, opts| async move {
            // The SDK should pass --resume <SESSION_ID> in args
            let resume_idx = opts.args.iter().position(|a| a == "--resume")
                .expect("expected --resume in args");
            let resume_id = opts.args[resume_idx + 1].clone();
            assert_eq!(resume_id, SESSION_ID);
            // The SDK should have materialized the transcript into CLAUDE_CONFIG_DIR/projects/<key>/<SESSION_ID>.jsonl
            let config = opts.env.get("CLAUDE_CONFIG_DIR").cloned().unwrap_or_default();
            let projects_dir = std::path::PathBuf::from(&config).join("projects");
            let target_file = format!("{}.jsonl", SESSION_ID);
            let mut found_path = None;
            if let Ok(entries) = std::fs::read_dir(&projects_dir) {
                for entry in entries.flatten() {
                    let candidate = entry.path().join(&target_file);
                    if candidate.exists() {
                        found_path = Some(candidate);
                        break;
                    }
                }
            }
            let found_path = found_path.expect("materialized transcript missing");
            let contents = std::fs::read_to_string(&found_path).unwrap();
            assert!(contents.contains("resume me"), "transcript does not contain 'resume me'");
            let init = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"control_response",
                "response":{"subtype":"success","request_id":init["request_id"],"response":{"ok":true}}
            })).await;
            let _user = expect_json_line(&mut r).await;
            write_json_line(&mut w, &json!({
                "type":"result","subtype":"success","duration_ms":1,"duration_api_ms":1,
                "is_error":false,"num_turns":1,"session_id":resume_id,"result":"resumed"
            })).await;
        }))
        .cwd(project.path())
        .session_store(store)
        .resume(SESSION_ID)
        .build();
    let mut stream = query("continue", options)?;
    let result = stream.next().await.unwrap()?;
    assert!(
        matches!(result, Message::Result(message) if message.result.as_deref() == Some("resumed"))
    );
    Ok(())
}
