// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod lockfile_rename;
mod metadata;

#[derive(Parser)]
#[command(
    name = "xtask-metadata-augment",
    about = "Workspace metadata augmentation and lockfile rename tooling"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Augment [package.metadata.oya] blocks in all workspace manifests.
    MetadataAugment {
        /// Dry-run: print changes without writing.
        #[arg(long)]
        check: bool,
        /// Apply changes in-place.
        #[arg(long)]
        apply: bool,
        /// Shard label (e.g. tools-xtask-metadata-augment).
        #[arg(long)]
        shard: Option<String>,
    },
    /// Rewrite Cargo.lock crate names per a rename-map TSV.
    LockfileRename {
        /// Path to TSV file: old-name<TAB>new-name per line.
        #[arg(long)]
        rename_map: String,
        /// Path to Cargo.lock to rewrite.
        #[arg(long)]
        lockfile: String,
        /// Rewrite in place (default: print to stdout).
        #[arg(long)]
        inplace: bool,
        /// Reverse the rename map (new→old).
        #[arg(long)]
        reverse: bool,
    },
    /// Move-aware Cargo.lock maintenance: rename crates, register newly-created
    /// local members, and re-canonicalize into Cargo's package/dependency order
    /// — the owned replacement for `cargo metadata` in a capability move (no
    /// version resolution, no Cargo in the authoring loop).
    LockfileMove {
        /// Path to TSV file: old-name<TAB>new-name per line.
        #[arg(long)]
        rename_map: String,
        /// Path to JSON graph-additions object:
        /// `{"new_members":[{"name","version","dependencies":[..]}],
        ///   "add_dependencies":[{"package","add":[..]}]}`. Optional.
        #[arg(long)]
        graph_additions: Option<String>,
        /// Path to Cargo.lock to rewrite.
        #[arg(long)]
        lockfile: String,
        /// Rewrite in place (default: print to stdout).
        #[arg(long)]
        inplace: bool,
    },
    /// Check that all workspace crates have valid [package.metadata.oya] blocks.
    RegistryCheck,
    /// Check that [lib] name matches snake_case of [package] name for all crates.
    LibNameCheck,
    /// Generate /tmp/old-crate-names.txt and /tmp/rename-map.tsv from §3 audit table.
    GenerateRenameMap {
        /// Path to the v4 plan markdown file.
        #[arg(
            long,
            default_value = "docs/plans/rename-plan-v4-clean-arch-2026-05-13.md"
        )]
        plan: String,
        /// Output path for old-crate-names.txt.
        #[arg(long, default_value = "/tmp/old-crate-names.txt")]
        names_out: String,
        /// Output path for rename-map.tsv.
        #[arg(long, default_value = "/tmp/rename-map.tsv")]
        map_out: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::MetadataAugment {
            check,
            apply,
            shard,
        } => metadata::run_metadata_augment(check, apply, shard.as_deref()),
        Commands::LockfileRename {
            rename_map,
            lockfile,
            inplace,
            reverse,
        } => lockfile_rename::run_lockfile_rename(&rename_map, &lockfile, inplace, reverse),
        Commands::LockfileMove {
            rename_map,
            graph_additions,
            lockfile,
            inplace,
        } => lockfile_rename::run_lockfile_move(
            &rename_map,
            graph_additions.as_deref(),
            &lockfile,
            inplace,
        ),
        Commands::RegistryCheck => registry_check(),
        Commands::LibNameCheck => lib_name_check(),
        Commands::GenerateRenameMap {
            plan,
            names_out,
            map_out,
        } => generate_rename_map(&plan, &names_out, &map_out),
    }
}

fn registry_check() -> Result<()> {
    let root_toml = std::fs::read_to_string("Cargo.toml").context("reading root Cargo.toml")?;
    let doc: toml_edit::DocumentMut = root_toml.parse().context("parsing root Cargo.toml")?;

    let members = doc["workspace"]["members"]
        .as_array()
        .context("workspace.members not found")?;

    let mut errors: Vec<String> = Vec::new();
    for member in members.iter() {
        let path = member.as_str().context("member is not a string")?;
        let manifest_path = format!("{path}/Cargo.toml");
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(e) => {
                errors.push(format!("cannot read {manifest_path}: {e}"));
                continue;
            }
        };
        let manifest_doc: toml_edit::DocumentMut = match manifest.parse() {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("cannot parse {manifest_path}: {e}"));
                continue;
            }
        };
        let meta = &manifest_doc["package"]["metadata"]["oya"];
        if meta.is_none() {
            errors.push(format!(
                "{manifest_path}: missing [package.metadata.oya] block"
            ));
        }
    }

    if errors.is_empty() {
        println!("registry-check: OK ({} members checked)", members.len());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("ERROR: {e}");
        }
        anyhow::bail!("registry-check: {} error(s) found", errors.len())
    }
}

fn lib_name_check() -> Result<()> {
    let root_toml = std::fs::read_to_string("Cargo.toml").context("reading root Cargo.toml")?;
    let doc: toml_edit::DocumentMut = root_toml.parse().context("parsing root Cargo.toml")?;

    let members = doc["workspace"]["members"]
        .as_array()
        .context("workspace.members not found")?;

    let mut errors: Vec<String> = Vec::new();
    for member in members.iter() {
        let path = member.as_str().context("member is not a string")?;
        let manifest_path = format!("{path}/Cargo.toml");
        let manifest = match std::fs::read_to_string(&manifest_path) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let manifest_doc: toml_edit::DocumentMut = match manifest.parse() {
            Ok(d) => d,
            Err(_) => continue,
        };

        let pkg_name = manifest_doc["package"]["name"]
            .as_str()
            .unwrap_or("")
            .to_owned();
        if pkg_name.is_empty() {
            continue;
        }

        let lib = &manifest_doc["lib"];
        if lib.is_none() {
            continue;
        }

        let lib_name = lib["name"].as_str().unwrap_or("").to_owned();
        if lib_name.is_empty() {
            continue;
        }

        let expected = pkg_name.replace('-', "_");
        if lib_name != expected {
            errors.push(format!(
                "{manifest_path}: [lib] name = \"{lib_name}\" but expected \"{expected}\" (snake of \"{pkg_name}\")"
            ));
        }
    }

    if errors.is_empty() {
        println!("lib-name-check: OK ({} members checked)", members.len());
        Ok(())
    } else {
        for e in &errors {
            eprintln!("ERROR: {e}");
        }
        anyhow::bail!("lib-name-check: {} mismatch(es) found", errors.len())
    }
}

fn generate_rename_map(plan_path: &str, names_out: &str, map_out: &str) -> Result<()> {
    let plan = std::fs::read_to_string(plan_path)
        .with_context(|| format!("reading plan at {plan_path}"))?;

    // Extract rename pairs from the §3 audit tables.  Two column schemas appear:
    //
    // 11-column (§3.2 Cloud, §3.3 Foundry non-check, §3.3.2 check crates):
    //   # | current_name | vertical | bounded_context | kind | layer | layer_evidence | proposed_name | bc_registry_status | risk | dep_edges_affected
    //   proposed_name at index 7
    //
    // 9-column (§3.1 Platform, §3.4 Connect/Workspace, §3.5 Foundation+Tooling):
    //   # | current_name | microservice | bounded_context | layer | layer_evidence | proposed_name | risk | dep_edges_affected
    //   proposed_name at index 6
    //
    // PROTOCOL-UNKNOWN rows (26 deferred to Shard 1.5) have proposed_name starting with
    // "PROTOCOL-UNKNOWN" — these are skipped.
    let mut rename_pairs: Vec<(String, String)> = Vec::new();

    for line in plan.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(str::trim)
            .collect();

        // Need at least 9 cells for the narrower schema.
        if cells.len() < 9 {
            continue;
        }

        let row_num = cells[0];
        // Skip header rows and separator rows.
        if row_num.is_empty() || row_num.starts_with('-') || row_num == "#" || row_num == "--:" {
            continue;
        }
        // Row number must parse as an integer (e.g. "1", "29", "138").
        if row_num.trim_start_matches('*').parse::<u32>().is_err() {
            continue;
        }

        let current = cells[1].trim_matches('`').trim_matches('*').trim();

        // Determine proposed_name index based on column count:
        // 11 columns → index 7; 9 columns → index 6.
        let proposed_idx = if cells.len() >= 11 { 7 } else { 6 };
        let proposed = cells[proposed_idx]
            .trim_matches('`')
            .trim_matches('*')
            .trim();

        // Skip PROTOCOL-UNKNOWN deferred rows and non-crate entries.
        if current.is_empty()
            || proposed.is_empty()
            || proposed.starts_with("PROTOCOL-UNKNOWN")
            || proposed.starts_with("STUB")
            || !current.starts_with("oya-")
            || !proposed.starts_with("oya-")
        {
            continue;
        }

        if current != proposed {
            rename_pairs.push((current.to_owned(), proposed.to_owned()));
        }
    }

    // Deduplicate
    rename_pairs.sort();
    rename_pairs.dedup();

    // Write names file (old names, one per line)
    let names_content = rename_pairs
        .iter()
        .map(|(old, _)| old.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(names_out, names_content + "\n")
        .with_context(|| format!("writing {names_out}"))?;

    // Write TSV (old<TAB>new)
    let map_content = rename_pairs
        .iter()
        .map(|(old, new)| format!("{old}\t{new}"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(map_out, map_content + "\n").with_context(|| format!("writing {map_out}"))?;

    println!(
        "generate-rename-map: {} rename pairs written to {} and {}",
        rename_pairs.len(),
        names_out,
        map_out
    );
    Ok(())
}
