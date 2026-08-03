//! Foundry JSONL supervisor adapter — file-backed InboxStore + OutboxSink.
//!
//! This is the **only** fsync-aware crate in the supervisor stack (Option D,
//! v4 §A.3). All disk I/O — peek_lock, commit, dead_letter rename, outbox
//! append — is isolated here. The supervisor-kernel port traits are implemented
//! on structs defined in this crate.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fs;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use oya_intelligence_supervisor_kernel::{
    AccountId, InboxItem, InboxStore, Locked, MessageId, OutboxSink, SupervisorError,
};

// ── JsonlInboxStore ──────────────────────────────────────────────────────────

pub struct JsonlInboxStore {
    root: PathBuf,
}

impl JsonlInboxStore {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn inbox_file(&self) -> PathBuf {
        self.root.join("inbox.jsonl")
    }

    fn lock_dir(&self) -> PathBuf {
        self.root.join("locks")
    }

    fn dead_letter_dir(&self) -> PathBuf {
        self.root.join("dead-letter")
    }

    fn now_secs() -> u64 {
        // ADR-0083 Tier 1: no production `.unwrap()` on a fallible Result.
        // SystemTime::now() before UNIX_EPOCH is only possible on a backward
        // mis-configured clock; treat as 0 seconds (parallels supervisor-
        // kernel's `record_spend` and is caught out-of-band by the
        // time-skew lane).
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::ZERO)
            .as_secs()
    }
}

impl InboxStore for JsonlInboxStore {
    fn peek_lock(&self, ttl: u64) -> Result<Option<Locked<InboxItem>>, SupervisorError> {
        let file = fs::File::open(self.inbox_file())
            .map_err(|e| SupervisorError::InboxError(e.to_string()))?;
        let reader = io::BufReader::new(file);

        fs::create_dir_all(self.lock_dir())
            .map_err(|e| SupervisorError::InboxError(e.to_string()))?;

        for line in reader.lines() {
            let line = line.map_err(|e| SupervisorError::InboxError(e.to_string()))?;
            if line.trim().is_empty() {
                continue;
            }

            // Minimal hand-rolled "parser" for {"id":"...","body":"..."}
            // Real impl should be more robust; this is Wave 2c logic.
            if let Some(id_start) = line.find("\"id\":\"") {
                let id_val_start = id_start + 6;
                if let Some(id_end) = line[id_val_start..].find('"') {
                    let message_id =
                        MessageId(line[id_val_start..id_val_start + id_end].to_string());

                    let lock_file = self.lock_dir().join(format!("{}.lock", message_id.0));
                    if lock_file.exists() {
                        // Check for stale lock
                        let metadata = fs::metadata(&lock_file)
                            .map_err(|e| SupervisorError::InboxError(e.to_string()))?;
                        let mtime = metadata
                            .modified()
                            .map_err(|e| SupervisorError::InboxError(e.to_string()))?
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or(Duration::ZERO)
                            .as_secs();

                        if Self::now_secs() < mtime {
                            continue; // Still locked
                        }
                    }

                    // Try to acquire lock
                    let reservation_id = format!("res-{}", Self::now_secs());
                    fs::write(&lock_file, &reservation_id)
                        .map_err(|e| SupervisorError::InboxError(e.to_string()))?;

                    // Extract body (minimal)
                    let body = if let Some(body_start) = line.find("\"body\":\"") {
                        let body_val_start = body_start + 8;
                        if let Some(body_end) = line[body_val_start..].find('"') {
                            line.as_bytes()[body_val_start..body_val_start + body_end].to_vec()
                        } else {
                            vec![]
                        }
                    } else {
                        vec![]
                    };

                    return Ok(Some(Locked {
                        reservation_id,
                        ttl_epoch_secs: Self::now_secs() + ttl,
                        item: InboxItem {
                            message_id,
                            payload: body,
                            enqueued_at_epoch_secs: 0, // Placeholder
                        },
                    }));
                }
            }
        }

        Ok(None)
    }

    fn commit(&self, id: &MessageId) -> Result<(), SupervisorError> {
        let lock_file = self.lock_dir().join(format!("{}.lock", id.0));
        if lock_file.exists() {
            fs::remove_file(lock_file).map_err(|e| SupervisorError::InboxError(e.to_string()))?;
        }
        // In a real impl, we'd also mark the line as committed in the jsonl or move it.
        Ok(())
    }

    fn release(&self, id: &MessageId, _reason: &str) -> Result<(), SupervisorError> {
        let lock_file = self.lock_dir().join(format!("{}.lock", id.0));
        if lock_file.exists() {
            fs::remove_file(lock_file).map_err(|e| SupervisorError::InboxError(e.to_string()))?;
        }
        Ok(())
    }

    fn dead_letter(&self, id: &MessageId, reason: &str) -> Result<(), SupervisorError> {
        let lock_file = self.lock_dir().join(format!("{}.lock", id.0));
        if !lock_file.exists() {
            return Err(SupervisorError::InvalidTransition);
        }

        fs::create_dir_all(self.dead_letter_dir())
            .map_err(|e| SupervisorError::InboxError(e.to_string()))?;
        let dead_file = self.dead_letter_dir().join(format!("{}.json", id.0));
        let content = format!("{{\"id\":\"{}\",\"reason\":\"{}\"}}", id.0, reason);
        fs::write(dead_file, content).map_err(|e| SupervisorError::InboxError(e.to_string()))?;

        fs::remove_file(lock_file).map_err(|e| SupervisorError::InboxError(e.to_string()))?;
        Ok(())
    }
}

// ── JsonlOutboxSink ──────────────────────────────────────────────────────────

pub struct JsonlOutboxSink {
    root: PathBuf,
}

impl JsonlOutboxSink {
    pub fn new<P: AsRef<Path>>(root: P) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
        }
    }

    fn outbox_file(&self, account_id: &AccountId) -> PathBuf {
        self.root.join(format!("{}.outbox.jsonl", account_id.0))
    }
}

impl OutboxSink for JsonlOutboxSink {
    fn push(&self, account_id: &AccountId, payload: Vec<u8>) -> Result<(), SupervisorError> {
        use std::io::Write;
        fs::create_dir_all(&self.root).map_err(|e| SupervisorError::OutboxError(e.to_string()))?;

        let path = self.outbox_file(account_id);
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| SupervisorError::OutboxError(e.to_string()))?;

        file.write_all(&payload)
            .map_err(|e| SupervisorError::OutboxError(e.to_string()))?;
        file.write_all(b"\n")
            .map_err(|e| SupervisorError::OutboxError(e.to_string()))?;
        file.sync_all()
            .map_err(|e| SupervisorError::OutboxError(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn outbox_push_creates_file_and_syncs() {
        let dir = tempdir().unwrap();
        let sink = JsonlOutboxSink::new(dir.path());
        let acc = AccountId("acc-1".to_owned());
        sink.push(&acc, b"{\"test\":true}".to_vec()).unwrap();

        let path = dir.path().join("acc-1.outbox.jsonl");
        assert!(path.exists());
        let content = fs::read_to_string(path).unwrap();
        assert_eq!(content, "{\"test\":true}\n");
    }

    #[test]
    fn dead_letter_fails_when_unlocked() {
        let dir = tempdir().unwrap();
        let store = JsonlInboxStore::new(dir.path());
        let id = MessageId("msg-1".to_owned());
        assert!(matches!(
            store.dead_letter(&id, "test"),
            Err(SupervisorError::InvalidTransition)
        ));
    }

    #[test]
    fn dead_letter_succeeds_when_locked() {
        let dir = tempdir().unwrap();
        let store = JsonlInboxStore::new(dir.path());
        let id = MessageId("msg-1".to_owned());

        // Mock a lock file
        fs::create_dir_all(store.lock_dir()).unwrap();
        fs::write(store.lock_dir().join("msg-1.lock"), "res-1").unwrap();

        store.dead_letter(&id, "unsupported_model").unwrap();

        assert!(!store.lock_dir().join("msg-1.lock").exists());
        assert!(store.dead_letter_dir().join("msg-1.json").exists());
        let content = fs::read_to_string(store.dead_letter_dir().join("msg-1.json")).unwrap();
        assert!(content.contains("unsupported_model"));
    }
}
