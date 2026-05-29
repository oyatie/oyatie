use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuditRecord {
    pub timestamp: DateTime<Utc>,
    pub agent_id: Option<String>,
    pub event: String,
    pub payload: serde_json::Value,
}

pub fn emit_audit(record: AuditRecord) -> Result<()> {
    let audit_dir = Path::new(".audit");
    if !audit_dir.exists() {
        fs::create_dir_all(audit_dir)?;
    }
    let audit_path = audit_dir.join("agent-read.jsonl");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_path)?;
    let line = serde_json::to_string(&record)?;
    writeln!(file, "{}", line)?;
    Ok(())
}
