use std::cell::RefCell;
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditOutcome {
    pub event: String,
    pub verb: String,
    pub tool: String,
    pub args: Vec<String>,
    pub success: bool,
    pub exit_code: Option<i32>,
    pub message: String,
    pub timestamp_unix_ms: u128,
    pub agent_id: Option<String>,
}

impl AuditOutcome {
    pub fn success(verb: &str, tool: &str, args: &[String], exit_code: i32) -> Self {
        Self::new(
            verb,
            tool,
            args,
            true,
            Some(exit_code),
            "read command completed",
        )
    }

    pub fn failure(
        verb: &str,
        tool: &str,
        args: &[String],
        exit_code: Option<i32>,
        message: &str,
    ) -> Self {
        Self::new(verb, tool, args, false, exit_code, message)
    }

    fn new(
        verb: &str,
        tool: &str,
        args: &[String],
        success: bool,
        exit_code: Option<i32>,
        message: &str,
    ) -> Self {
        Self {
            event: format!(
                "EVT-AGENT-READ-{}",
                verb.to_ascii_uppercase().replace('-', "_")
            ),
            verb: verb.to_string(),
            tool: tool.to_string(),
            args: args.to_vec(),
            success,
            exit_code,
            message: message.to_string(),
            timestamp_unix_ms: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0),
            agent_id: env::var("OYATIE_AGENT_ID").ok().filter(|s| !s.is_empty()),
        }
    }

    pub fn to_json_line(&self) -> String {
        let exit = self
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "null".to_string());
        let agent = self
            .agent_id
            .as_ref()
            .map(|v| format!("\"{}\"", escape_json(v)))
            .unwrap_or_else(|| "null".to_string());
        let args = self
            .args
            .iter()
            .map(|a| format!("\"{}\"", escape_json(a)))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            "{{\"event\":\"{}\",\"verb\":\"{}\",\"tool\":\"{}\",\"args\":[{}],\"success\":{},\"exit_code\":{},\"message\":\"{}\",\"timestamp_unix_ms\":{},\"agent_id\":{}}}",
            escape_json(&self.event),
            escape_json(&self.verb),
            escape_json(&self.tool),
            args,
            self.success,
            exit,
            escape_json(&self.message),
            self.timestamp_unix_ms,
            agent
        )
    }
}

pub trait Auditor {
    fn emit(&self, outcome: &AuditOutcome) -> io::Result<()>;
}

pub struct FileAuditor {
    path: PathBuf,
}

impl FileAuditor {
    pub fn from_env() -> Self {
        let path = env::var("OYATIE_AGENT_READ_AUDIT_PATH")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(".audit/agent-read.jsonl"));
        Self { path }
    }
}

impl Auditor for FileAuditor {
    fn emit(&self, outcome: &AuditOutcome) -> io::Result<()> {
        if let Some(parent) = self.path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?;
        writeln!(file, "{}", outcome.to_json_line())
    }
}

pub fn emit_event(
    verb: &str,
    tool: &str,
    args: &[String],
    success: bool,
    exit_code: Option<i32>,
    message: &str,
) -> io::Result<()> {
    let outcome = if success {
        AuditOutcome::success(verb, tool, args, exit_code.unwrap_or(0))
    } else {
        AuditOutcome::failure(verb, tool, args, exit_code, message)
    };
    FileAuditor::from_env().emit(&outcome)
}

#[derive(Default)]
pub struct MemoryAuditor {
    records: RefCell<Vec<AuditOutcome>>,
}

impl MemoryAuditor {
    pub fn records(&self) -> Vec<AuditOutcome> {
        self.records.borrow().clone()
    }
}

impl Auditor for MemoryAuditor {
    fn emit(&self, outcome: &AuditOutcome) -> io::Result<()> {
        self.records.borrow_mut().push(outcome.clone());
        Ok(())
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_line_escapes_payload() {
        let outcome = AuditOutcome::failure(
            "pr-view",
            "gh",
            &["12\n34".to_string()],
            None,
            "bad \"input\"",
        );
        let line = outcome.to_json_line();
        assert!(line.contains("EVT-AGENT-READ-PR_VIEW"));
        assert!(line.contains("12\\n34"));
        assert!(line.contains("bad \\\"input\\\""));
    }
}
