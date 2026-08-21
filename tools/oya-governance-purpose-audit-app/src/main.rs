//! Purpose-audit tool (M01-P10-IP-001).
//!
//! Scans Markdown and JSON files for purpose declarations and reports violations.
//! Also supports backfilling purpose frontmatter if requested (v6 Directive 12).

use anyhow::{Context, Result};
use check_purpose_kernel::{PurposeNode, check};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        println!("Usage: oya-governance-purpose-audit-app <globs...>");
        return Ok(());
    }

    let mut nodes = Vec::new();
    for pattern in args {
        for entry in glob::glob(&pattern).context("invalid glob pattern")? {
            let path = entry.context("glob error")?;

            // Ignore transient state
            if path.components().any(|c| {
                c.as_os_str() == "state"
                    || c.as_os_str() == "sessions"
                    || c.as_os_str() == "handoffs"
                    || c.as_os_str() == "project-memory.json"
            }) {
                continue;
            }

            if path.is_file() {
                let node = scan_file(&path)?;
                nodes.push(node);
            }
        }
    }

    let report = check(&nodes).context("purpose check failed")?;

    if report.violations.is_empty() {
        println!(
            "Purpose audit green ({} nodes checked)",
            report.nodes_checked
        );
    } else {
        println!(
            "Purpose audit failed! {} violations found:",
            report.violations.len()
        );
        for v in report.violations {
            println!("  [MISSING PURPOSE] {}", v.path);
        }
        std::process::exit(1);
    }

    Ok(())
}

fn scan_file(path: &Path) -> Result<PurposeNode> {
    let content = fs::read_to_string(path).with_context(|| format!("failed to read {:?}", path))?;

    let path_str = path.to_string_lossy().to_string();
    let extension = path.extension().and_then(|s| s.to_str());

    let purpose = match extension {
        Some("md") => extract_markdown_purpose(&content),
        Some("json") => extract_json_purpose(&content),
        _ => None,
    };

    Ok(PurposeNode {
        path: path_str,
        purpose,
    })
}

fn extract_markdown_purpose(content: &str) -> Option<String> {
    if !content.starts_with("---") {
        return None;
    }
    let end_fm = content[3..].find("---")?;
    let fm_content = &content[3..3 + end_fm];
    let fm: serde_yaml::Value = serde_yaml::from_str(fm_content).ok()?;

    fm.get("purpose")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}

fn extract_json_purpose(content: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(content).ok()?;

    // Check _meta.purpose
    if let Some(p) = v
        .get("_meta")
        .and_then(|m| m.get("purpose"))
        .and_then(|p| p.as_str())
    {
        return Some(p.to_string());
    }

    // Check top-level purpose
    v.get("purpose")
        .and_then(|p| p.as_str())
        .map(|s| s.to_string())
}
