//! Emit the frozen `cloud-` name baseline for `ci/facade/cloud-name-ratchet`.
//!
//! Run this after a rename lands to record the burn-down:
//!
//! `cargo run -p ci-cloud-name-ratchet --bin oya-ci-cloud-name-baseline -- --repo-root . > ci/facade/cloud-name-ratchet/cloud-name-baseline.json`
//!
//! The gate REQUIRES the baseline to shrink in the same change as the rename, so the frozen file
//! never overstates the remaining debt. Regenerating is the intended, encouraged direction; the
//! gate fails on growth, not on shrink.
//!
//! `"_bootstrap": true` is emitted because the INTRODUCING change has no merge-base copy to
//! compare against. Once this file exists on the protected branch the gate reads the merge-base
//! copy instead, and the marker is inert — that is what stops a later PR regenerating its own
//! baseline to launder new debt into it.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut root = PathBuf::from(".");
    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--repo-root" => match args.next() {
                Some(v) => root = PathBuf::from(v),
                None => {
                    eprintln!("--repo-root needs a value");
                    return ExitCode::from(2);
                }
            },
            other => {
                eprintln!("unrecognized argument {other}");
                return ExitCode::from(2);
            }
        }
    }
    let names = census(&root);
    let rows: Vec<String> = names.iter().map(|n| format!("    {n:?}")).collect();
    println!(
        "{{\n  \"_comment\": \"FROZEN shrink-only baseline of deprecated `cloud-` names \
         (directories, Cargo package names, Helm chart names) present today. Growth is RED; \
         shrink is the point. Regenerate with `cargo run -p ci-cloud-name-ratchet --bin \
         oya-ci-cloud-name-baseline` in the SAME change as the rename, so this file never \
         overstates the remaining debt. NOTE: the oya-cloud-ci-* names are LIVE merge machinery \
         (the required test job binds oya-cloud-ci-accounting-registry-app via \
         OYA_CI_PRODUCER_BIN) — renaming one means moving that binding in the same change.\",\n  \
         \"_bootstrap\": true,\n  \"count\": {},\n  \"cloud_prefixed_names\": [\n{}\n  ]\n}}",
        names.len(),
        rows.join(",\n")
    );
    ExitCode::SUCCESS
}

/// Census every tracked-looking file under `root`.
fn census(root: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy().to_string();
            if entry.file_type().is_ok_and(|t| t.is_dir()) {
                if !matches!(
                    name.as_str(),
                    "target" | "buck-out" | ".git" | "node_modules" | ".jj"
                ) {
                    stack.push(path);
                }
                continue;
            }
            let Ok(relative) = path.strip_prefix(root) else {
                continue;
            };
            let relative = relative.to_string_lossy().to_string();
            let contents = if matches!(name.as_str(), "Cargo.toml" | "Chart.yaml") {
                std::fs::read_to_string(&path).unwrap_or_default()
            } else {
                String::new()
            };
            out.extend(ci_cloud_name_ratchet::findings(&relative, &contents));
        }
    }
    out
}
