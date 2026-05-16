mod audit;

use anyhow::Result;
use audit::{AuditRecord, emit_audit};
use chrono::Utc;
use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(
    name = "oya-tooling-agent-read",
    about = "Sanctioned read primitive for agent file access"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read a file, emit audit event, print contents
    Read { path: String },
    /// Regex search across files, emit audit, print matches grouped by file
    Search {
        pattern: String,
        #[arg(long, default_value = ".")]
        path: String,
    },
    /// List public Rust symbols in a file
    Symbols { path: String },
    /// Append a JSONL audit row to .audit/agent-read.jsonl
    AuditEmit {
        #[arg(long)]
        event: String,
        #[arg(long)]
        payload: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Read { path } => cmd_read(&path),
        Commands::Search { pattern, path } => cmd_search(&pattern, &path),
        Commands::Symbols { path } => cmd_symbols(&path),
        Commands::AuditEmit { event, payload } => cmd_audit_emit(&event, &payload),
    }
}

fn cmd_read(path: &str) -> Result<()> {
    let contents = fs::read_to_string(path)?;
    emit_audit(AuditRecord {
        timestamp: Utc::now(),
        agent_id: None,
        event: "read".to_string(),
        payload: serde_json::json!({ "path": path }),
    })?;
    print!("{}", contents);
    Ok(())
}

fn cmd_search(pattern: &str, dir: &str) -> Result<()> {
    let re = Regex::new(pattern)?;
    let mut results: std::collections::BTreeMap<String, Vec<(usize, String)>> =
        std::collections::BTreeMap::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        let Ok(content) = fs::read_to_string(path) else {
            continue;
        };
        let mut matches: Vec<(usize, String)> = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                matches.push((i + 1, line.to_string()));
            }
        }
        if !matches.is_empty() {
            results.insert(path.display().to_string(), matches);
        }
    }

    emit_audit(AuditRecord {
        timestamp: Utc::now(),
        agent_id: None,
        event: "search".to_string(),
        payload: serde_json::json!({ "pattern": pattern, "dir": dir, "file_count": results.len() }),
    })?;

    for (file, matches) in &results {
        println!("=== {} ===", file);
        for (lineno, line) in matches {
            println!("{}: {}", lineno, line);
        }
    }
    Ok(())
}

fn cmd_symbols(path: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let re = Regex::new(r"pub\s+(fn|struct|enum|trait|const)\s+(\w+)")?;

    emit_audit(AuditRecord {
        timestamp: Utc::now(),
        agent_id: None,
        event: "symbols".to_string(),
        payload: serde_json::json!({ "path": path }),
    })?;

    for (i, line) in content.lines().enumerate() {
        if let Some(cap) = re.find(line) {
            println!("{}:{}: {}", path, i + 1, cap.as_str());
        }
    }
    Ok(())
}

fn cmd_audit_emit(event: &str, payload_str: &str) -> Result<()> {
    let payload: serde_json::Value = serde_json::from_str(payload_str)?;
    emit_audit(AuditRecord {
        timestamp: Utc::now(),
        agent_id: None,
        event: event.to_string(),
        payload,
    })?;
    println!("audit event emitted: {}", event);
    Ok(())
}
