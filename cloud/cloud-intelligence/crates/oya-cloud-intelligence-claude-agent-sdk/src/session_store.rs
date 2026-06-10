use std::{
    collections::BTreeMap,
    env, fs,
    future::Future,
    path::{Component, Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures::future::{BoxFuture, FutureExt, ready};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tokio::time::{sleep, timeout};

use crate::{
    error::{ClaudeAgentError, Result},
    options::ClaudeAgentOptions,
    sessions::{
        ForkSessionResult, GetSessionMessagesOptions, LiteSessionFile, SDKSessionInfo,
        SessionMessage, delete_session, entries_to_session_messages,
        entries_to_session_messages_with_system, find_session_file, fork_session,
        fork_session_entries, get_session_info, get_subagent_messages, list_sessions,
        list_subagent_transcript_files, list_subagents, parse_iso_epoch_ms,
        parse_session_info_from_lite, project_key_for_directory, rename_session,
        sanitize_unicode_tag, tag_session, validate_fork_inputs,
    },
};

const DEFAULT_STORE_TIMEOUT_MS: u64 = 60_000;
const MIRROR_APPEND_TIMEOUT: Duration = Duration::from_secs(60);
const MIRROR_APPEND_BACKOFFS: [Duration; 2] =
    [Duration::from_millis(200), Duration::from_millis(800)];
const MAX_PENDING_MIRROR_ENTRIES: usize = 500;
const MAX_PENDING_MIRROR_BYTES: usize = 1 << 20;

/// Opaque JSON-safe transcript entry stored by a [`SessionStore`].
pub type SessionStoreEntry = Value;

/// Address of a mirrored transcript in a [`SessionStore`].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SessionKey {
    pub project_key: String,
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
}

impl SessionKey {
    pub fn new(project_key: impl Into<String>, session_id: impl Into<String>) -> Self {
        Self {
            project_key: project_key.into(),
            session_id: session_id.into(),
            subpath: None,
        }
    }

    pub fn with_subpath(
        project_key: impl Into<String>,
        session_id: impl Into<String>,
        subpath: impl Into<String>,
    ) -> Self {
        Self {
            project_key: project_key.into(),
            session_id: session_id.into(),
            subpath: Some(subpath.into()),
        }
    }

    fn storage_key(&self) -> String {
        match &self.subpath {
            Some(subpath) if !subpath.is_empty() => {
                format!("{}/{}/{}", self.project_key, self.session_id, subpath)
            }
            _ => format!("{}/{}", self.project_key, self.session_id),
        }
    }
}

/// Listing row returned by [`SessionStore::list_sessions`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionStoreListEntry {
    pub session_id: String,
    /// Unix epoch milliseconds from the backing storage clock.
    pub mtime: u64,
}

/// Incremental session summary sidecar for stores that can list metadata cheaply.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionSummaryEntry {
    pub session_id: String,
    /// Unix epoch milliseconds from the same backing storage clock as listings.
    pub mtime: u64,
    #[serde(default)]
    pub data: Map<String, Value>,
}

/// Adapter-supplied metadata for [`fold_session_summary_with_options`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoldSessionSummaryOptions {
    /// Unix epoch milliseconds from the same backing storage clock as listings.
    ///
    /// This is not derived from transcript entry timestamps. Pass `Some` when
    /// folding a newly persisted batch; omit only when re-folding an existing
    /// sidecar and preserving its previous modification time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mtime: Option<u64>,
}

/// When mirrored transcript entries should be flushed to [`SessionStore::append`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStoreFlushMode {
    #[default]
    Batched,
    Eager,
}

/// Options object for listing sessions.
///
/// When `session_store` is set, listing routes through the async store-backed
/// implementation. Otherwise it reads local Claude transcript files.
#[derive(Debug, Clone)]
pub struct ListSessionsOptions {
    /// Project directory used to resolve local transcripts or a store project key.
    pub directory: Option<PathBuf>,
    /// Maximum number of sessions to return.
    pub limit: Option<usize>,
    /// Number of sessions to skip from the sorted result set.
    pub offset: usize,
    /// Include local git worktree transcript paths when reading from files.
    pub include_worktrees: bool,
    /// Optional external transcript store to list instead of local files.
    pub session_store: Option<SharedSessionStore>,
}

impl Default for ListSessionsOptions {
    fn default() -> Self {
        Self {
            directory: None,
            limit: None,
            offset: 0,
            include_worktrees: true,
            session_store: None,
        }
    }
}

/// Options object for reading session metadata by ID.
#[derive(Debug, Clone, Default)]
pub struct GetSessionInfoOptions {
    /// Project directory used to resolve local transcripts or a store project key.
    pub directory: Option<PathBuf>,
    /// Optional external transcript store to read instead of local files.
    pub session_store: Option<SharedSessionStore>,
}

/// Options object for listing subagents under a session.
#[derive(Debug, Clone, Default)]
pub struct ListSubagentsOptions {
    /// Project directory used to resolve local transcripts or a store project key.
    pub directory: Option<PathBuf>,
    /// Optional external transcript store to read instead of local files.
    pub session_store: Option<SharedSessionStore>,
}

/// Options object for retrieving subagent messages.
#[derive(Debug, Clone, Default)]
pub struct GetSubagentMessagesOptions {
    /// Project directory used to resolve local transcripts or a store project key.
    pub directory: Option<PathBuf>,
    /// Maximum number of messages to return.
    pub limit: Option<usize>,
    /// Number of messages to skip from the start.
    pub offset: usize,
    /// Optional external transcript store to read instead of local files.
    pub session_store: Option<SharedSessionStore>,
}

/// Options object for importing a local session into a [`SessionStore`].
#[derive(Debug, Clone)]
pub struct ImportSessionToStoreOptions {
    /// Project directory used to find the local transcript.
    pub directory: Option<PathBuf>,
    /// Include subagent transcripts and metadata while importing.
    pub include_subagents: bool,
    /// Maximum number of entries per store append call.
    pub batch_size: Option<usize>,
}

impl Default for ImportSessionToStoreOptions {
    fn default() -> Self {
        Self {
            directory: None,
            include_subagents: true,
            batch_size: None,
        }
    }
}

/// Options object for helpers that mutate a session by ID.
///
/// When `session_store` is set, mutation helpers route through the async
/// store-backed implementation. Otherwise they mutate local Claude transcript
/// files in `directory`.
#[derive(Debug, Clone, Default)]
pub struct SessionMutationOptions {
    /// Project directory used to resolve the local transcript project key.
    pub directory: Option<PathBuf>,
    /// Optional external transcript store to mutate instead of local files.
    pub session_store: Option<SharedSessionStore>,
}

/// Options object for forking a session.
///
/// This mirrors [`SessionMutationOptions`] and adds the optional fork boundary
/// and title fields used by [`fork_session_with_options`].
#[derive(Debug, Clone, Default)]
pub struct ForkSessionOptions {
    /// Project directory used to resolve the local transcript project key.
    pub directory: Option<PathBuf>,
    /// Optional external transcript store to fork instead of local files.
    pub session_store: Option<SharedSessionStore>,
    /// Optional message UUID at which to truncate the fork source.
    pub up_to_message_id: Option<String>,
    /// Optional custom title to append to the forked session.
    pub title: Option<String>,
}

/// Async external transcript mirror/load interface.
///
/// Required methods are [`append`](SessionStore::append) and
/// [`load`](SessionStore::load). Optional methods return `Ok(None)` by default;
/// implement them when helpers such as [`list_sessions_from_store`] or resume
/// materialization need to enumerate sessions/subkeys efficiently.
pub trait SessionStore: Send + Sync {
    fn append<'a>(
        &'a self,
        key: SessionKey,
        entries: Vec<SessionStoreEntry>,
    ) -> BoxFuture<'a, Result<()>>;

    fn load<'a>(&'a self, key: SessionKey)
    -> BoxFuture<'a, Result<Option<Vec<SessionStoreEntry>>>>;

    fn list_sessions<'a>(
        &'a self,
        _project_key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<SessionStoreListEntry>>>> {
        ready(Ok(None)).boxed()
    }

    fn list_session_summaries<'a>(
        &'a self,
        _project_key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<SessionSummaryEntry>>>> {
        ready(Ok(None)).boxed()
    }

    fn delete<'a>(&'a self, _key: SessionKey) -> BoxFuture<'a, Result<Option<()>>> {
        ready(Ok(None)).boxed()
    }

    fn list_subkeys<'a>(&'a self, _key: SessionKey) -> BoxFuture<'a, Result<Option<Vec<String>>>> {
        ready(Ok(None)).boxed()
    }
}

/// Cloneable trait-object wrapper used by [`ClaudeAgentOptions`].
#[derive(Clone)]
pub struct SharedSessionStore(Arc<dyn SessionStore>);

impl SharedSessionStore {
    pub fn new<S>(store: S) -> Self
    where
        S: SessionStore + 'static,
    {
        Self(Arc::new(store))
    }

    pub fn from_arc(store: Arc<dyn SessionStore>) -> Self {
        Self(store)
    }

    pub fn as_store(&self) -> &(dyn SessionStore + 'static) {
        self.0.as_ref()
    }
}

impl std::fmt::Debug for SharedSessionStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("SharedSessionStore(..)")
    }
}

impl From<Arc<dyn SessionStore>> for SharedSessionStore {
    fn from(value: Arc<dyn SessionStore>) -> Self {
        Self::from_arc(value)
    }
}

/// In-memory reference [`SessionStore`] for tests and development.
#[derive(Debug, Clone, Default)]
pub struct InMemorySessionStore {
    inner: Arc<Mutex<InMemoryState>>,
}

#[derive(Debug, Default)]
struct InMemoryState {
    entries: BTreeMap<String, Vec<SessionStoreEntry>>,
    mtimes: BTreeMap<String, u64>,
    summaries: BTreeMap<(String, String), SessionSummaryEntry>,
    last_mtime: u64,
}

impl InMemorySessionStore {
    pub fn get_entries(&self, key: SessionKey) -> Vec<SessionStoreEntry> {
        self.inner
            .lock()
            .map(|state| {
                state
                    .entries
                    .get(&key.storage_key())
                    .cloned()
                    .unwrap_or_default()
            })
            .unwrap_or_default()
    }

    pub fn size(&self) -> usize {
        self.inner
            .lock()
            .map(|state| {
                state
                    .entries
                    .keys()
                    .filter(|key| key.split('/').count() == 2)
                    .count()
            })
            .unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut state) = self.inner.lock() {
            state.entries.clear();
            state.mtimes.clear();
            state.summaries.clear();
            state.last_mtime = 0;
        }
    }
}

impl SessionStore for InMemorySessionStore {
    fn append<'a>(
        &'a self,
        key: SessionKey,
        entries: Vec<SessionStoreEntry>,
    ) -> BoxFuture<'a, Result<()>> {
        async move {
            if entries.is_empty() {
                return Ok(());
            }

            let mut state = lock_state(&self.inner)?;
            let storage_key = key.storage_key();
            state
                .entries
                .entry(storage_key.clone())
                .or_default()
                .extend(entries.clone());
            let now_ms = state.next_mtime();
            state.mtimes.insert(storage_key, now_ms);
            if key.subpath.is_none() {
                let summary_key = (key.project_key.clone(), key.session_id.clone());
                let folded = fold_session_summary_with_options(
                    state.summaries.get(&summary_key),
                    &key,
                    &entries,
                    FoldSessionSummaryOptions {
                        mtime: Some(now_ms),
                    },
                );
                state.summaries.insert(summary_key, folded);
            }
            Ok(())
        }
        .boxed()
    }

    fn load<'a>(
        &'a self,
        key: SessionKey,
    ) -> BoxFuture<'a, Result<Option<Vec<SessionStoreEntry>>>> {
        async move {
            let state = lock_state(&self.inner)?;
            Ok(state.entries.get(&key.storage_key()).cloned())
        }
        .boxed()
    }

    fn list_sessions<'a>(
        &'a self,
        project_key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<SessionStoreListEntry>>>> {
        async move {
            let state = lock_state(&self.inner)?;
            let prefix = format!("{project_key}/");
            let mut entries = Vec::new();
            for storage_key in state.entries.keys() {
                let Some(rest) = storage_key.strip_prefix(&prefix) else {
                    continue;
                };
                if rest.contains('/') {
                    continue;
                }
                entries.push(SessionStoreListEntry {
                    session_id: rest.to_owned(),
                    mtime: state.mtimes.get(storage_key).copied().unwrap_or_default(),
                });
            }
            Ok(Some(entries))
        }
        .boxed()
    }

    fn list_session_summaries<'a>(
        &'a self,
        project_key: &'a str,
    ) -> BoxFuture<'a, Result<Option<Vec<SessionSummaryEntry>>>> {
        async move {
            let state = lock_state(&self.inner)?;
            Ok(Some(
                state
                    .summaries
                    .iter()
                    .filter_map(|((stored_project, _), summary)| {
                        (stored_project == project_key).then_some(summary.clone())
                    })
                    .collect(),
            ))
        }
        .boxed()
    }

    fn delete<'a>(&'a self, key: SessionKey) -> BoxFuture<'a, Result<Option<()>>> {
        async move {
            let mut state = lock_state(&self.inner)?;
            let storage_key = key.storage_key();
            state.entries.remove(&storage_key);
            state.mtimes.remove(&storage_key);
            if key.subpath.is_none() {
                state
                    .summaries
                    .remove(&(key.project_key.clone(), key.session_id.clone()));
                let prefix = format!("{}/{}/", key.project_key, key.session_id);
                let subkeys = state
                    .entries
                    .keys()
                    .filter(|candidate| candidate.starts_with(&prefix))
                    .cloned()
                    .collect::<Vec<_>>();
                for subkey in subkeys {
                    state.entries.remove(&subkey);
                    state.mtimes.remove(&subkey);
                }
            }
            Ok(Some(()))
        }
        .boxed()
    }

    fn list_subkeys<'a>(&'a self, key: SessionKey) -> BoxFuture<'a, Result<Option<Vec<String>>>> {
        async move {
            let state = lock_state(&self.inner)?;
            let prefix = format!("{}/{}/", key.project_key, key.session_id);
            Ok(Some(
                state
                    .entries
                    .keys()
                    .filter_map(|candidate| candidate.strip_prefix(&prefix).map(ToOwned::to_owned))
                    .collect(),
            ))
        }
        .boxed()
    }
}

impl InMemoryState {
    fn next_mtime(&mut self) -> u64 {
        let mut now_ms = unix_epoch_ms();
        if now_ms <= self.last_mtime {
            now_ms = self.last_mtime + 1;
        }
        self.last_mtime = now_ms;
        now_ms
    }
}

fn lock_state(
    inner: &Arc<Mutex<InMemoryState>>,
) -> Result<std::sync::MutexGuard<'_, InMemoryState>> {
    inner
        .lock()
        .map_err(|_| ClaudeAgentError::Connection("session store lock poisoned".into()))
}

/// Fold appended main-transcript entries into an incremental summary sidecar.
pub fn fold_session_summary(
    previous: Option<&SessionSummaryEntry>,
    key: &SessionKey,
    entries: &[SessionStoreEntry],
) -> SessionSummaryEntry {
    fold_session_summary_with_options(previous, key, entries, FoldSessionSummaryOptions::default())
}

/// Fold appended main-transcript entries into an incremental summary sidecar.
///
/// The summary `mtime` is adapter-stamped from `options.mtime`; when omitted,
/// an existing summary keeps its previous `mtime` and a new summary starts at
/// `0`.
pub fn fold_session_summary_with_options(
    previous: Option<&SessionSummaryEntry>,
    key: &SessionKey,
    entries: &[SessionStoreEntry],
    options: FoldSessionSummaryOptions,
) -> SessionSummaryEntry {
    let mut summary = previous.cloned().unwrap_or_else(|| SessionSummaryEntry {
        session_id: key.session_id.clone(),
        mtime: 0,
        data: Map::new(),
    });
    if let Some(mtime) = options.mtime {
        summary.mtime = mtime;
    }

    for entry in entries {
        let Some(object) = entry.as_object() else {
            continue;
        };

        if !summary.data.contains_key("is_sidechain") {
            summary.data.insert(
                "is_sidechain".into(),
                Value::Bool(object.get("isSidechain").and_then(Value::as_bool) == Some(true)),
            );
        }

        if !summary.data.contains_key("created_at")
            && let Some(ms) = object
                .get("timestamp")
                .and_then(Value::as_str)
                .and_then(parse_iso_epoch_ms)
        {
            summary.data.insert("created_at".into(), json!(ms));
        }

        if !summary.data.contains_key("cwd")
            && let Some(cwd) = object.get("cwd").and_then(Value::as_str)
            && !cwd.is_empty()
        {
            summary
                .data
                .insert("cwd".into(), Value::String(cwd.to_owned()));
        }

        fold_first_prompt(&mut summary.data, object);

        for (source, dest) in [
            ("customTitle", "custom_title"),
            ("aiTitle", "ai_title"),
            ("lastPrompt", "last_prompt"),
            ("summary", "summary_hint"),
            ("gitBranch", "git_branch"),
        ] {
            if let Some(value) = object.get(source).and_then(Value::as_str) {
                summary
                    .data
                    .insert(dest.to_owned(), Value::String(value.to_owned()));
            }
        }

        if object.get("type").and_then(Value::as_str) == Some("tag") {
            match object.get("tag").and_then(Value::as_str) {
                Some(tag) if !tag.is_empty() => {
                    summary
                        .data
                        .insert("tag".into(), Value::String(tag.to_owned()));
                }
                _ => {
                    summary.data.remove("tag");
                }
            }
        }
    }

    summary
}

/// Convert a store-maintained summary sidecar into public session metadata.
pub fn summary_entry_to_sdk_info(
    entry: &SessionSummaryEntry,
    project_path: Option<&str>,
) -> Option<SDKSessionInfo> {
    if entry
        .data
        .get("is_sidechain")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }

    let first_prompt = if entry
        .data
        .get("first_prompt_locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        data_string(&entry.data, "first_prompt")
    } else {
        data_string(&entry.data, "command_fallback")
    };
    let custom_title =
        data_string(&entry.data, "custom_title").or_else(|| data_string(&entry.data, "ai_title"));
    let summary = custom_title
        .clone()
        .or_else(|| data_string(&entry.data, "last_prompt"))
        .or_else(|| data_string(&entry.data, "summary_hint"))
        .or_else(|| first_prompt.clone())?;

    Some(SDKSessionInfo {
        session_id: entry.session_id.clone(),
        summary,
        last_modified: entry.mtime,
        file_size: None,
        custom_title,
        first_prompt,
        git_branch: data_string(&entry.data, "git_branch"),
        cwd: data_string(&entry.data, "cwd").or_else(|| project_path.map(ToOwned::to_owned)),
        tag: data_string(&entry.data, "tag"),
        created_at: entry.data.get("created_at").and_then(Value::as_u64),
    })
}

/// List sessions from a [`SessionStore`], using sidecar summaries when present.
pub async fn list_sessions_from_store(
    session_store: &(impl SessionStore + ?Sized),
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SDKSessionInfo>> {
    let project_path = canonical_project_path(directory);
    let project_path_string = project_path.to_string_lossy().to_string();
    let project_key = project_key_for_directory(&project_path);

    if let Some(summaries) = session_store.list_session_summaries(&project_key).await? {
        let listing = session_store
            .list_sessions(&project_key)
            .await?
            .unwrap_or_default();
        let known_mtimes = listing
            .iter()
            .map(|entry| (entry.session_id.clone(), entry.mtime))
            .collect::<BTreeMap<_, _>>();
        let mut summary_ids = Vec::new();
        let mut sessions = Vec::new();
        for summary in summaries {
            if let Some(known_mtime) = known_mtimes.get(&summary.session_id)
                && summary.mtime < *known_mtime
            {
                continue;
            }
            summary_ids.push(summary.session_id.clone());
            if let Some(info) = summary_entry_to_sdk_info(&summary, Some(&project_path_string)) {
                sessions.push(info);
            }
        }
        for listing in listing {
            if summary_ids.iter().any(|id| id == &listing.session_id) {
                continue;
            }
            if let Some(mut info) =
                get_session_info_from_store(session_store, &listing.session_id, directory).await?
            {
                info.last_modified = listing.mtime;
                sessions.push(info);
            }
        }
        return Ok(apply_sort_limit_offset(sessions, limit, offset));
    }

    let Some(listing) = session_store.list_sessions(&project_key).await? else {
        return Err(ClaudeAgentError::InvalidOption(
            "session_store implements neither list_session_summaries nor list_sessions".into(),
        ));
    };

    let mut sessions = Vec::new();
    for entry in listing {
        if let Some(mut info) =
            get_session_info_from_store(session_store, &entry.session_id, directory).await?
        {
            info.last_modified = entry.mtime;
            sessions.push(info);
        }
    }
    Ok(apply_sort_limit_offset(sessions, limit, offset))
}

/// Read metadata for one session from a [`SessionStore`].
pub async fn get_session_info_from_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    directory: Option<&Path>,
) -> Result<Option<SDKSessionInfo>> {
    if !is_valid_uuid(session_id) {
        return Ok(None);
    }
    let project_path = canonical_project_path(directory);
    let project_path_string = project_path.to_string_lossy().to_string();
    let key = SessionKey::new(project_key_for_directory(&project_path), session_id);
    let Some(entries) = session_store.load(key).await? else {
        return Ok(None);
    };
    if entries.is_empty() {
        return Ok(None);
    }
    let lite = entries_to_lite(&entries, None);
    Ok(parse_session_info_from_lite(
        session_id,
        &lite,
        Some(&project_path_string),
    ))
}

/// List sessions using either local transcripts or a configured store.
pub async fn list_sessions_with_options(
    options: ListSessionsOptions,
) -> Result<Vec<SDKSessionInfo>> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        list_sessions_from_store(
            session_store.as_store(),
            directory,
            options.limit,
            options.offset,
        )
        .await
    } else {
        list_sessions(
            directory,
            options.limit,
            options.offset,
            options.include_worktrees,
        )
    }
}

/// Read one session's metadata using either local transcripts or a configured store.
pub async fn get_session_info_with_options(
    session_id: &str,
    options: GetSessionInfoOptions,
) -> Result<Option<SDKSessionInfo>> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        get_session_info_from_store(session_store.as_store(), session_id, directory).await
    } else {
        get_session_info(session_id, directory)
    }
}

/// Read visible user/assistant messages from a store-backed session.
pub async fn get_session_messages_from_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SessionMessage>> {
    get_session_messages_from_store_with_options(
        session_store,
        session_id,
        GetSessionMessagesOptions {
            directory: directory.map(Path::to_path_buf),
            limit,
            offset,
            include_system_messages: false,
        },
    )
    .await
}

/// Read store-backed session messages using an options object.
///
/// Defaults match [`get_session_messages_from_store`]: system messages are
/// excluded unless `include_system_messages` is true.
pub async fn get_session_messages_from_store_with_options(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    options: GetSessionMessagesOptions,
) -> Result<Vec<SessionMessage>> {
    if !is_valid_uuid(session_id) {
        return Ok(Vec::new());
    }
    let project_key =
        project_key_for_directory(canonical_project_path(options.directory.as_deref()));
    let key = SessionKey::new(project_key, session_id);
    let Some(entries) = session_store.load(key).await? else {
        return Ok(Vec::new());
    };
    Ok(entries_to_session_messages_with_system(
        &filter_transcript_entries(&entries),
        options.limit,
        options.offset,
        options.include_system_messages,
    ))
}

/// Rename a session using either a local transcript or a configured store.
pub async fn rename_session_with_options(
    session_id: &str,
    title: &str,
    options: SessionMutationOptions,
) -> Result<()> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        rename_session_via_store(session_store.as_store(), session_id, title, directory).await
    } else {
        rename_session(session_id, title, directory)
    }
}

/// Tag or clear a session tag using either a local transcript or a configured store.
pub async fn tag_session_with_options(
    session_id: &str,
    tag: Option<&str>,
    options: SessionMutationOptions,
) -> Result<()> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        tag_session_via_store(session_store.as_store(), session_id, tag, directory).await
    } else {
        tag_session(session_id, tag, directory)
    }
}

/// Delete a session using either a local transcript or a configured store.
pub async fn delete_session_with_options(
    session_id: &str,
    options: SessionMutationOptions,
) -> Result<()> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        delete_session_via_store(session_store.as_store(), session_id, directory).await
    } else {
        delete_session(session_id, directory)
    }
}

/// Fork a session using either a local transcript or a configured store.
pub async fn fork_session_with_options(
    session_id: &str,
    options: ForkSessionOptions,
) -> Result<ForkSessionResult> {
    let directory = options.directory.as_deref();
    let up_to_message_id = options.up_to_message_id.as_deref();
    let title = options.title.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        fork_session_via_store(
            session_store.as_store(),
            session_id,
            directory,
            up_to_message_id,
            title,
        )
        .await
    } else {
        fork_session(session_id, directory, up_to_message_id, title)
    }
}

/// List subagent IDs using either local transcripts or a configured store.
pub async fn list_subagents_with_options(
    session_id: &str,
    options: ListSubagentsOptions,
) -> Result<Vec<String>> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        list_subagents_from_store(session_store.as_store(), session_id, directory).await
    } else {
        list_subagents(session_id, directory)
    }
}

/// Read subagent messages using either local transcripts or a configured store.
pub async fn get_subagent_messages_with_options(
    session_id: &str,
    agent_id: &str,
    options: GetSubagentMessagesOptions,
) -> Result<Vec<SessionMessage>> {
    let directory = options.directory.as_deref();
    if let Some(session_store) = options.session_store.as_ref() {
        get_subagent_messages_from_store(
            session_store.as_store(),
            session_id,
            agent_id,
            directory,
            options.limit,
            options.offset,
        )
        .await
    } else {
        get_subagent_messages(
            session_id,
            agent_id,
            directory,
            options.limit,
            options.offset,
        )
    }
}

/// Append a store-backed custom title entry.
pub async fn rename_session_via_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    title: &str,
    directory: Option<&Path>,
) -> Result<()> {
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
    ensure_store_session_exists(session_store, session_id, directory).await?;
    let key = SessionKey::new(
        project_key_for_directory(canonical_project_path(directory)),
        session_id,
    );
    session_store
        .append(
            key,
            vec![json!({
                "type": "custom-title",
                "customTitle": title,
                "sessionId": session_id,
                "uuid": format!("meta-{}", uuid::Uuid::new_v4().simple()),
                "timestamp": current_timestamp_iso_utc(),
            })],
        )
        .await
}

/// Append a store-backed tag entry. Pass `None` to clear the tag.
pub async fn tag_session_via_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    tag: Option<&str>,
    directory: Option<&Path>,
) -> Result<()> {
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
    ensure_store_session_exists(session_store, session_id, directory).await?;
    let key = SessionKey::new(
        project_key_for_directory(canonical_project_path(directory)),
        session_id,
    );
    session_store
        .append(
            key,
            vec![json!({
                "type": "tag",
                "tag": tag,
                "sessionId": session_id,
                "uuid": format!("meta-{}", uuid::Uuid::new_v4().simple()),
                "timestamp": current_timestamp_iso_utc(),
            })],
        )
        .await
}

/// Delete a store-backed session when the adapter implements deletion.
pub async fn delete_session_via_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    directory: Option<&Path>,
) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }
    ensure_store_session_exists(session_store, session_id, directory).await?;
    let key = SessionKey::new(
        project_key_for_directory(canonical_project_path(directory)),
        session_id,
    );
    match session_store.delete(key).await? {
        Some(()) => Ok(()),
        None => Err(ClaudeAgentError::InvalidOption(
            "session_store does not implement delete".into(),
        )),
    }
}

/// List subagent IDs for a store-backed session.
pub async fn list_subagents_from_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    directory: Option<&Path>,
) -> Result<Vec<String>> {
    if !is_valid_uuid(session_id) {
        return Ok(Vec::new());
    }
    let key = SessionKey::new(
        project_key_for_directory(canonical_project_path(directory)),
        session_id,
    );
    let Some(subkeys) = session_store.list_subkeys(key).await? else {
        return Err(ClaudeAgentError::InvalidOption(
            "session_store does not implement list_subkeys".into(),
        ));
    };
    let mut seen = BTreeMap::new();
    for subkey in subkeys {
        let Some(rest) = subkey.strip_prefix("subagents/") else {
            continue;
        };
        let last = rest.rsplit('/').next().unwrap_or_default();
        let Some(agent_id) = last.strip_prefix("agent-") else {
            continue;
        };
        seen.entry(agent_id.to_owned()).or_insert(());
    }
    Ok(seen.into_keys().collect())
}

/// Read messages for a store-backed subagent transcript.
pub async fn get_subagent_messages_from_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    agent_id: &str,
    directory: Option<&Path>,
    limit: Option<usize>,
    offset: usize,
) -> Result<Vec<SessionMessage>> {
    if !is_valid_uuid(session_id) || agent_id.is_empty() {
        return Ok(Vec::new());
    }
    let project_key = project_key_for_directory(canonical_project_path(directory));
    let direct = format!("subagents/agent-{agent_id}");
    let mut candidates = vec![direct.clone()];
    if let Some(subkeys) = session_store
        .list_subkeys(SessionKey::new(project_key.clone(), session_id))
        .await?
    {
        candidates = subkeys
            .into_iter()
            .filter(|subkey| {
                subkey.starts_with("subagents/")
                    && subkey
                        .rsplit('/')
                        .next()
                        .and_then(|last| last.strip_prefix("agent-"))
                        == Some(agent_id)
            })
            .collect();
        if candidates.is_empty() {
            candidates.push(direct);
        }
    }

    for subpath in candidates {
        let key = SessionKey::with_subpath(project_key.clone(), session_id, subpath);
        if let Some(entries) = session_store.load(key).await?
            && !entries.is_empty()
        {
            return Ok(entries_to_session_messages(
                &filter_transcript_entries(&entries),
                limit,
                offset,
            ));
        }
    }
    Ok(Vec::new())
}

/// Fork a store-backed session into a new session key with remapped UUIDs.
pub async fn fork_session_via_store(
    session_store: &(impl SessionStore + ?Sized),
    session_id: &str,
    directory: Option<&Path>,
    up_to_message_id: Option<&str>,
    title: Option<&str>,
) -> Result<ForkSessionResult> {
    validate_fork_inputs(session_id, up_to_message_id)?;
    let project_key = project_key_for_directory(canonical_project_path(directory));
    let key = SessionKey::new(project_key.clone(), session_id);
    let Some(entries) = session_store.load(key).await? else {
        return Err(ClaudeAgentError::SessionNotFound {
            session_id: session_id.to_owned(),
        });
    };
    if entries.is_empty() {
        return Err(ClaudeAgentError::SessionNotFound {
            session_id: session_id.to_owned(),
        });
    }
    let (forked_entries, forked_session_id) =
        fork_session_entries(&entries, session_id, up_to_message_id, title)?;
    session_store
        .append(
            SessionKey::new(project_key, forked_session_id.clone()),
            forked_entries,
        )
        .await?;
    Ok(ForkSessionResult {
        session_id: forked_session_id,
    })
}

/// Copy a local JSONL session into a [`SessionStore`].
///
/// The main transcript is appended under `SessionKey::new(project_key,
/// session_id)`. When `include_subagents` is true, subagent transcripts under
/// `<session_id>/subagents/**/agent-*.jsonl` are appended under matching
/// subpaths, and sibling `*.meta.json` files are preserved as
/// `agent_metadata` entries.
pub async fn import_session_to_store(
    session_id: &str,
    session_store: &(impl SessionStore + ?Sized),
    directory: Option<&Path>,
    include_subagents: bool,
    batch_size: Option<usize>,
) -> Result<()> {
    if !is_valid_uuid(session_id) {
        return Err(ClaudeAgentError::InvalidOption(format!(
            "invalid session_id: {session_id}"
        )));
    }
    let Some(session_file) = find_session_file(session_id, directory) else {
        return Err(ClaudeAgentError::SessionNotFound {
            session_id: session_id.to_owned(),
        });
    };

    let project_key = project_key_for_directory(canonical_project_path(directory));
    let batch_size = batch_size.filter(|size| *size > 0).unwrap_or(500);
    append_jsonl_file_to_store(
        session_store,
        &session_file,
        SessionKey::new(project_key.clone(), session_id),
        batch_size,
    )
    .await?;

    if !include_subagents {
        return Ok(());
    }

    for subagent_file in list_subagent_transcript_files(&session_file) {
        let Some(subpath) = subagent_subpath(&session_file, &subagent_file) else {
            continue;
        };
        let key = SessionKey::with_subpath(project_key.clone(), session_id, subpath);
        append_jsonl_file_to_store(session_store, &subagent_file, key.clone(), batch_size).await?;
        if let Some(metadata) = read_agent_metadata(&subagent_file)? {
            session_store.append(key, vec![metadata]).await?;
        }
    }
    Ok(())
}

/// Copy a local JSONL session into a [`SessionStore`] using an options object.
pub async fn import_session_to_store_with_options(
    session_id: &str,
    session_store: &(impl SessionStore + ?Sized),
    options: ImportSessionToStoreOptions,
) -> Result<()> {
    import_session_to_store(
        session_id,
        session_store,
        options.directory.as_deref(),
        options.include_subagents,
        options.batch_size,
    )
    .await
}

/// Convert a transcript path emitted by `--session-mirror` into a store key.
pub fn file_path_to_session_key(
    file_path: impl AsRef<Path>,
    projects_dir: impl AsRef<Path>,
) -> Option<SessionKey> {
    let relative = file_path
        .as_ref()
        .strip_prefix(projects_dir.as_ref())
        .ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            _ => return None,
        }
    }
    if parts.len() < 2 {
        return None;
    }
    let project_key = parts[0].clone();
    let second = &parts[1];
    if parts.len() == 2 {
        let session_id = second.strip_suffix(".jsonl")?.to_owned();
        return Some(SessionKey::new(project_key, session_id));
    }
    if parts.len() >= 4 {
        let mut subpath_parts = parts[2..].to_vec();
        if let Some(last) = subpath_parts.last_mut()
            && let Some(stripped) = last.strip_suffix(".jsonl")
        {
            *last = stripped.to_owned();
        }
        return Some(SessionKey::with_subpath(
            project_key,
            second.clone(),
            subpath_parts.join("/"),
        ));
    }
    None
}

async fn append_jsonl_file_to_store(
    session_store: &(impl SessionStore + ?Sized),
    path: &Path,
    key: SessionKey,
    batch_size: usize,
) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let mut batch = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        batch.push(serde_json::from_str::<Value>(trimmed)?);
        if batch.len() >= batch_size {
            session_store
                .append(key.clone(), std::mem::take(&mut batch))
                .await?;
        }
    }
    if !batch.is_empty() {
        session_store.append(key, batch).await?;
    }
    Ok(())
}

fn subagent_subpath(session_file: &Path, subagent_file: &Path) -> Option<String> {
    let mut session_dir = session_file.to_path_buf();
    session_dir.set_extension("");
    let relative = subagent_file.strip_prefix(&session_dir).ok()?;
    let mut parts = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().to_string()),
            _ => return None,
        }
    }
    let last = parts.last_mut()?;
    *last = last.strip_suffix(".jsonl")?.to_owned();
    Some(parts.join("/"))
}

fn read_agent_metadata(subagent_file: &Path) -> Result<Option<Value>> {
    let metadata_path = subagent_file.with_extension("meta.json");
    let content = match fs::read_to_string(metadata_path) {
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
    let value = serde_json::from_str::<Value>(&content)?;
    let mut object = value.as_object().cloned().unwrap_or_default();
    object.insert("type".into(), Value::String("agent_metadata".into()));
    Ok(Some(Value::Object(object)))
}

pub(crate) struct TranscriptMirrorBatcher {
    store: SharedSessionStore,
    projects_dir: PathBuf,
    flush_mode: SessionStoreFlushMode,
    pending: Vec<(SessionKey, Vec<SessionStoreEntry>)>,
    pending_entries: usize,
    pending_bytes: usize,
}

impl TranscriptMirrorBatcher {
    pub(crate) fn new(
        store: SharedSessionStore,
        projects_dir: PathBuf,
        flush_mode: SessionStoreFlushMode,
    ) -> Self {
        Self {
            store,
            projects_dir,
            flush_mode,
            pending: Vec::new(),
            pending_entries: 0,
            pending_bytes: 0,
        }
    }

    pub(crate) async fn enqueue_frame(&mut self, frame: &Value) -> Vec<(SessionKey, String)> {
        let Some(file_path) = frame
            .get("filePath")
            .or_else(|| frame.get("file_path"))
            .and_then(Value::as_str)
        else {
            return Vec::new();
        };
        let Some(key) = file_path_to_session_key(file_path, &self.projects_dir) else {
            return Vec::new();
        };
        let entries = frame
            .get("entries")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        if entries.is_empty() {
            return Vec::new();
        }
        let entry_count = entries.len();
        let byte_count = serde_json::to_vec(&entries).map_or(0, |bytes| bytes.len());
        self.pending.push((key, entries));
        self.pending_entries += entry_count;
        self.pending_bytes += byte_count;
        if self.flush_mode == SessionStoreFlushMode::Eager
            || self.pending_entries > MAX_PENDING_MIRROR_ENTRIES
            || self.pending_bytes > MAX_PENDING_MIRROR_BYTES
        {
            self.flush().await
        } else {
            Vec::new()
        }
    }

    pub(crate) async fn flush(&mut self) -> Vec<(SessionKey, String)> {
        if self.pending.is_empty() {
            return Vec::new();
        }
        let pending = std::mem::take(&mut self.pending);
        self.pending_entries = 0;
        self.pending_bytes = 0;
        let mut coalesced: Vec<(SessionKey, Vec<SessionStoreEntry>)> = Vec::new();
        for (key, entries) in pending {
            if let Some((_, existing)) = coalesced
                .iter_mut()
                .find(|(existing_key, _)| existing_key == &key)
            {
                existing.extend(entries);
            } else {
                coalesced.push((key, entries));
            }
        }

        let mut errors = Vec::new();
        for (key, entries) in coalesced {
            if let Err(error) = append_with_retry(self.store.as_store(), key.clone(), entries).await
            {
                errors.push((key, error.to_string()));
            }
        }
        errors
    }
}

async fn append_with_retry(
    store: &(dyn SessionStore + 'static),
    key: SessionKey,
    entries: Vec<SessionStoreEntry>,
) -> Result<()> {
    let mut last_error = None;
    for attempt in 0..3 {
        if attempt > 0 {
            sleep(MIRROR_APPEND_BACKOFFS[attempt - 1]).await;
        }
        match timeout(
            MIRROR_APPEND_TIMEOUT,
            store.append(key.clone(), entries.clone()),
        )
        .await
        {
            Ok(Ok(())) => return Ok(()),
            Ok(Err(error)) => last_error = Some(error),
            Err(_) => {
                return Err(ClaudeAgentError::ControlTimeout(format!(
                    "SessionStore.append() for {}/{}",
                    key.project_key, key.session_id
                )));
            }
        }
    }
    Err(last_error
        .unwrap_or_else(|| ClaudeAgentError::Connection("SessionStore.append() failed".into())))
}

pub(crate) struct MaterializedSession {
    config_dir: PathBuf,
    pub(crate) resume_session_id: String,
}

impl MaterializedSession {
    pub(crate) fn config_dir(&self) -> &Path {
        &self.config_dir
    }
}

impl Drop for MaterializedSession {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.config_dir);
    }
}

pub(crate) async fn materialize_resume_session(
    options: &ClaudeAgentOptions,
) -> Result<Option<MaterializedSession>> {
    let Some(store) = &options.session_store else {
        return Ok(None);
    };
    if options.resume.is_none() && !options.continue_conversation {
        return Ok(None);
    }

    let timeout_duration =
        Duration::from_millis(options.load_timeout_ms.unwrap_or(DEFAULT_STORE_TIMEOUT_MS));
    let project_key = project_key_for_directory(options.cwd.as_deref().unwrap_or(Path::new(".")));
    let resolved = if let Some(session_id) = &options.resume {
        if !is_valid_uuid(session_id) {
            return Ok(None);
        }
        load_candidate(store.as_store(), &project_key, session_id, timeout_duration).await?
    } else {
        resolve_continue_candidate(store.as_store(), &project_key, timeout_duration).await?
    };
    let Some((session_id, entries)) = resolved else {
        return Ok(None);
    };

    let config_dir = create_private_temp_dir()?;
    let project_dir = config_dir.join("projects").join(&project_key);
    if let Err(error) = (|| -> Result<()> {
        fs::create_dir_all(&project_dir)?;
        write_jsonl(&project_dir.join(format!("{session_id}.jsonl")), &entries)?;
        copy_auth_files(&config_dir, &options.env)?;
        Ok(())
    })() {
        let _ = fs::remove_dir_all(&config_dir);
        return Err(error);
    }

    if let Err(error) = materialize_subkeys(
        store.as_store(),
        &project_dir,
        &project_key,
        &session_id,
        timeout_duration,
    )
    .await
    {
        let _ = fs::remove_dir_all(&config_dir);
        return Err(error);
    }

    Ok(Some(MaterializedSession {
        config_dir,
        resume_session_id: session_id,
    }))
}

async fn load_candidate(
    store: &(dyn SessionStore + 'static),
    project_key: &str,
    session_id: &str,
    timeout_duration: Duration,
) -> Result<Option<(String, Vec<SessionStoreEntry>)>> {
    let key = SessionKey::new(project_key, session_id);
    let entries = timeout(timeout_duration, store.load(key))
        .await
        .map_err(|_| ClaudeAgentError::ControlTimeout("SessionStore.load()".into()))??;
    Ok(entries
        .filter(|entries| !entries.is_empty())
        .map(|entries| (session_id.to_owned(), entries)))
}

async fn resolve_continue_candidate(
    store: &(dyn SessionStore + 'static),
    project_key: &str,
    timeout_duration: Duration,
) -> Result<Option<(String, Vec<SessionStoreEntry>)>> {
    let Some(mut sessions) = timeout(timeout_duration, store.list_sessions(project_key))
        .await
        .map_err(|_| ClaudeAgentError::ControlTimeout("SessionStore.list_sessions()".into()))??
    else {
        return Ok(None);
    };
    sessions.sort_by_key(|entry| std::cmp::Reverse(entry.mtime));
    for session in sessions {
        if !is_valid_uuid(&session.session_id) {
            continue;
        }
        let Some(candidate) =
            load_candidate(store, project_key, &session.session_id, timeout_duration).await?
        else {
            continue;
        };
        if candidate
            .1
            .first()
            .and_then(Value::as_object)
            .and_then(|entry| entry.get("isSidechain"))
            .and_then(Value::as_bool)
            == Some(true)
        {
            continue;
        }
        return Ok(Some(candidate));
    }
    Ok(None)
}

async fn materialize_subkeys(
    store: &(dyn SessionStore + 'static),
    project_dir: &Path,
    project_key: &str,
    session_id: &str,
    timeout_duration: Duration,
) -> Result<()> {
    let Some(subkeys) = timeout(
        timeout_duration,
        store.list_subkeys(SessionKey::new(project_key, session_id)),
    )
    .await
    .map_err(|_| ClaudeAgentError::ControlTimeout("SessionStore.list_subkeys()".into()))??
    else {
        return Ok(());
    };
    let session_dir = project_dir.join(session_id);
    for subpath in subkeys {
        if !is_safe_subpath(&subpath) {
            continue;
        }
        let key = SessionKey::with_subpath(project_key, session_id, subpath.clone());
        let Some(entries) = timeout(timeout_duration, store.load(key))
            .await
            .map_err(|_| ClaudeAgentError::ControlTimeout("SessionStore.load() subkey".into()))??
        else {
            continue;
        };
        let mut metadata = Vec::new();
        let mut transcript = Vec::new();
        for entry in entries {
            if entry.get("type").and_then(Value::as_str) == Some("agent_metadata") {
                metadata.push(entry);
            } else {
                transcript.push(entry);
            }
        }
        let transcript_path = session_dir.join(&subpath).with_extension("jsonl");
        if !transcript.is_empty() {
            write_jsonl(&transcript_path, &transcript)?;
        }
        if let Some(last_metadata) = metadata.last() {
            let mut object = last_metadata.as_object().cloned().unwrap_or_default();
            object.remove("type");
            let metadata_path = transcript_path.with_extension("meta.json");
            if let Some(parent) = metadata_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(metadata_path, Value::Object(object).to_string())?;
        }
    }
    Ok(())
}

pub(crate) fn effective_projects_dir(env_override: &BTreeMap<String, String>) -> PathBuf {
    effective_config_dir(env_override).join("projects")
}

fn effective_config_dir(env_override: &BTreeMap<String, String>) -> PathBuf {
    env_override
        .get("CLAUDE_CONFIG_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("CLAUDE_CONFIG_DIR").map(PathBuf::from))
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".claude")))
        .unwrap_or_else(|| PathBuf::from(".claude"))
}

fn copy_auth_files(config_dir: &Path, env_override: &BTreeMap<String, String>) -> Result<()> {
    let source_config_dir = effective_config_dir(env_override);
    if let Ok(credentials) = fs::read_to_string(source_config_dir.join(".credentials.json")) {
        let destination = config_dir.join(".credentials.json");
        fs::write(&destination, redact_refresh_token(credentials))?;
        set_private_file_permissions(&destination)?;
    }

    let claude_json_source = if env_override.contains_key("CLAUDE_CONFIG_DIR")
        || env::var_os("CLAUDE_CONFIG_DIR").is_some()
    {
        source_config_dir.join(".claude.json")
    } else {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude.json")
    };
    if claude_json_source.exists() {
        let destination = config_dir.join(".claude.json");
        fs::copy(claude_json_source, &destination)?;
        set_private_file_permissions(&destination)?;
    }
    Ok(())
}

fn create_private_temp_dir() -> Result<PathBuf> {
    for _ in 0..100 {
        let candidate =
            env::temp_dir().join(format!("claude-resume-{}", uuid::Uuid::new_v4().simple()));
        match create_private_dir(&candidate) {
            Ok(()) => return Ok(candidate),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ClaudeAgentError::Connection(
        "failed to create private resume materialization directory".into(),
    ))
}

#[cfg(unix)]
fn create_private_dir(path: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    fs::DirBuilder::new().mode(0o700).create(path)
}

#[cfg(not(unix))]
fn create_private_dir(path: &Path) -> std::io::Result<()> {
    fs::create_dir(path)
}

#[cfg(unix)]
fn set_private_file_permissions(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_file_permissions(_path: &Path) -> Result<()> {
    Ok(())
}

fn redact_refresh_token(credentials: String) -> String {
    let Ok(mut value) = serde_json::from_str::<Value>(&credentials) else {
        return credentials;
    };
    if let Some(object) = value.as_object_mut()
        && let Some(oauth) = object
            .get_mut("claudeAiOauth")
            .and_then(Value::as_object_mut)
    {
        oauth.remove("refreshToken");
    }
    value.to_string()
}

fn write_jsonl(path: &Path, entries: &[SessionStoreEntry]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut content = String::new();
    for entry in entries {
        content.push_str(&serde_json::to_string(entry)?);
        content.push('\n');
    }
    fs::write(path, content)?;
    Ok(())
}

fn entries_to_lite(entries: &[SessionStoreEntry], mtime: Option<u64>) -> LiteSessionFile {
    let jsonl = entries_to_jsonl(entries);
    let size = jsonl.len() as u64;
    LiteSessionFile {
        mtime: mtime.unwrap_or_else(|| mtime_from_entries(entries).unwrap_or_else(unix_epoch_ms)),
        size,
        head: jsonl.clone(),
        tail: jsonl,
    }
}

fn entries_to_jsonl(entries: &[SessionStoreEntry]) -> String {
    entries
        .iter()
        .filter_map(|entry| serde_json::to_string(entry).ok())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn mtime_from_entries(entries: &[SessionStoreEntry]) -> Option<u64> {
    entries
        .iter()
        .rev()
        .find_map(|entry| entry.get("timestamp").and_then(Value::as_str))
        .and_then(parse_iso_epoch_ms)
}

fn filter_transcript_entries(entries: &[SessionStoreEntry]) -> Vec<Value> {
    entries
        .iter()
        .filter(|entry| {
            matches!(
                entry.get("type").and_then(Value::as_str),
                Some("user" | "assistant" | "progress" | "system" | "attachment")
            ) && entry.get("uuid").and_then(Value::as_str).is_some()
        })
        .cloned()
        .collect()
}

fn ensure_store_session_exists<'a>(
    session_store: &'a (impl SessionStore + ?Sized),
    session_id: &'a str,
    directory: Option<&'a Path>,
) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        let project_key = project_key_for_directory(canonical_project_path(directory));
        let key = SessionKey::new(project_key, session_id);
        match session_store.load(key).await? {
            Some(entries) if !entries.is_empty() => Ok(()),
            _ => Err(ClaudeAgentError::SessionNotFound {
                session_id: session_id.to_owned(),
            }),
        }
    })
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

fn canonical_project_path(directory: Option<&Path>) -> PathBuf {
    let path = directory.unwrap_or_else(|| Path::new("."));
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn data_string(data: &Map<String, Value>, key: &str) -> Option<String> {
    data.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn fold_first_prompt(data: &mut Map<String, Value>, entry: &Map<String, Value>) {
    if data
        .get("first_prompt_locked")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || entry.get("type").and_then(Value::as_str) != Some("user")
        || entry.get("isMeta").and_then(Value::as_bool) == Some(true)
        || entry.get("isCompactSummary").and_then(Value::as_bool) == Some(true)
    {
        return;
    }

    let Some(message) = entry.get("message").and_then(Value::as_object) else {
        return;
    };
    let Some(content) = message.get("content") else {
        return;
    };
    if content.as_array().is_some_and(|blocks| {
        blocks
            .iter()
            .any(|block| block.get("type").and_then(Value::as_str) == Some("tool_result"))
    }) {
        return;
    }

    for raw in content_texts(content) {
        let prompt = raw.replace('\n', " ").trim().to_owned();
        if prompt.is_empty() {
            continue;
        }
        if let Some(command_name) = extract_command_name(&prompt) {
            data.entry("command_fallback")
                .or_insert(Value::String(command_name));
            continue;
        }
        if should_skip_first_prompt(&prompt) {
            continue;
        }
        data.insert(
            "first_prompt".into(),
            Value::String(truncate_prompt(prompt)),
        );
        data.insert("first_prompt_locked".into(), Value::Bool(true));
        return;
    }
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

fn is_valid_uuid(session_id: &str) -> bool {
    uuid::Uuid::parse_str(session_id).is_ok()
}

fn unix_epoch_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or_default()
}

fn current_timestamp_iso_utc() -> String {
    let now = SystemTime::now()
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

fn is_safe_subpath(subpath: &str) -> bool {
    !subpath.is_empty()
        && subpath.starts_with("subagents/")
        && !subpath.contains('\0')
        && !subpath.contains('\\')
        && !subpath.contains(':')
        && subpath
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
}
