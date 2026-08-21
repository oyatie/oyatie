/// Cargo.lock name-rewrite / move-canonicalize CLI (§7.1.1 spec).
///
/// The pure, I/O-free transform lives in the shared [`cargo_lock_transform_kernel`] kernel
/// (single source of truth, consumed by both this xtask CLI and the reorg move codemod). This
/// module is the thin I/O layer: it loads the rename-map TSV + graph-additions JSON from disk,
/// invokes the kernel, and writes the result back (or prints it).
use anyhow::{Context, Result};
use cargo_lock_transform_kernel::{GraphAdditions, NewMember, move_lockfile, rewrite_lockfile};
use std::collections::HashMap;

/// Load a rename-map TSV (`old<TAB>new` per line) into a map. `reverse` swaps direction.
fn load_rename_map(rename_map_path: &str, reverse: bool) -> Result<HashMap<String, String>> {
    let map_content = std::fs::read_to_string(rename_map_path)
        .with_context(|| format!("reading rename map at {rename_map_path}"))?;

    let mut rename_map: HashMap<String, String> = HashMap::new();
    let mut seen_targets: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (lineno, line) in map_content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(2, '\t').collect();
        if parts.len() != 2 {
            anyhow::bail!(
                "rename map line {}: expected 'old<TAB>new', got: {:?}",
                lineno + 1,
                line
            );
        }
        let (old, new) = (parts[0].trim().to_owned(), parts[1].trim().to_owned());
        let (from, to) = if reverse { (new, old) } else { (old, new) };
        // Both-side injective (matching the move-manifest bijection's MUST-PASS #3): a
        // duplicate source is an ambiguous rename; a duplicate target would collapse two
        // distinct crates into one name — either silently drops a dependency edge. Fail-closed.
        if seen_targets.contains(&to) {
            anyhow::bail!(
                "rename map line {}: target {:?} appears more than once — non-injective rename \
                 (would collapse two crates), fail-closed",
                lineno + 1,
                to
            );
        }
        if rename_map.insert(from.clone(), to.clone()).is_some() {
            anyhow::bail!(
                "rename map line {}: source {:?} appears more than once — ambiguous rename, \
                 fail-closed",
                lineno + 1,
                from
            );
        }
        seen_targets.insert(to);
    }
    Ok(rename_map)
}

pub fn run_lockfile_rename(
    rename_map_path: &str,
    lockfile_path: &str,
    inplace: bool,
    reverse: bool,
) -> Result<()> {
    let rename_map = load_rename_map(rename_map_path, reverse)?;

    let lockfile_content = std::fs::read_to_string(lockfile_path)
        .with_context(|| format!("reading lockfile at {lockfile_path}"))?;

    let rewritten = rewrite_lockfile(&lockfile_content, &rename_map)?;

    if inplace {
        std::fs::write(lockfile_path, &rewritten)
            .with_context(|| format!("writing lockfile at {lockfile_path}"))?;
        println!("lockfile-rename: rewrote {lockfile_path} in place");
    } else {
        print!("{rewritten}");
    }

    Ok(())
}

fn json_str_array(value: &serde_json::Value, ctx: &str) -> Result<Vec<String>> {
    value
        .as_array()
        .with_context(|| format!("{ctx} must be an array"))?
        .iter()
        .enumerate()
        .map(|(j, x)| {
            x.as_str()
                .map(str::to_owned)
                .with_context(|| format!("{ctx}[{j}] must be a string"))
        })
        .collect()
}

/// Parse a `--graph-additions` JSON object:
/// `{"new_members": [{"name","version","dependencies":[..]}],
///   "add_dependencies": [{"package": .., "add": [..]}]}`. Both keys optional.
fn load_graph_additions(path: &str) -> Result<GraphAdditions> {
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("reading graph-additions at {path}"))?;
    let value: serde_json::Value = serde_json::from_str(&raw)
        .with_context(|| format!("parsing graph-additions JSON at {path}"))?;

    let mut new_members = Vec::new();
    if let Some(arr) = value.get("new_members") {
        for (i, item) in arr
            .as_array()
            .context("new_members must be an array")?
            .iter()
            .enumerate()
        {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .with_context(|| format!("new_members[{i}]: missing string field \"name\""))?
                .to_owned();
            let version = item
                .get("version")
                .and_then(|v| v.as_str())
                .with_context(|| format!("new_members[{i}]: missing string field \"version\""))?
                .to_owned();
            let dependencies = match item.get("dependencies") {
                None => Vec::new(),
                Some(d) => json_str_array(d, &format!("new_members[{i}].dependencies"))?,
            };
            new_members.push(NewMember {
                name,
                version,
                dependencies,
            });
        }
    }

    let mut add_dependencies: HashMap<String, Vec<String>> = HashMap::new();
    if let Some(arr) = value.get("add_dependencies") {
        for (i, item) in arr
            .as_array()
            .context("add_dependencies must be an array")?
            .iter()
            .enumerate()
        {
            let package = item
                .get("package")
                .and_then(|v| v.as_str())
                .with_context(|| {
                    format!("add_dependencies[{i}]: missing string field \"package\"")
                })?
                .to_owned();
            let add = json_str_array(
                item.get("add")
                    .with_context(|| format!("add_dependencies[{i}]: missing field \"add\""))?,
                &format!("add_dependencies[{i}].add"),
            )?;
            add_dependencies.entry(package).or_default().extend(add);
        }
    }

    Ok(GraphAdditions {
        new_members,
        add_dependencies,
    })
}

/// `lockfile-move` subcommand: rename + graph additions + canonicalize.
pub fn run_lockfile_move(
    rename_map_path: &str,
    graph_additions_path: Option<&str>,
    lockfile_path: &str,
    inplace: bool,
) -> Result<()> {
    let rename_map = load_rename_map(rename_map_path, false)?;
    let additions = match graph_additions_path {
        Some(p) => load_graph_additions(p)?,
        None => GraphAdditions::empty(),
    };

    let content = std::fs::read_to_string(lockfile_path)
        .with_context(|| format!("reading lockfile at {lockfile_path}"))?;
    let rewritten = move_lockfile(&content, &rename_map, &additions)?;

    if inplace {
        std::fs::write(lockfile_path, &rewritten)
            .with_context(|| format!("writing lockfile at {lockfile_path}"))?;
        println!(
            "lockfile-move: rewrote {lockfile_path} in place ({} renames, {} new members, {} edge targets)",
            rename_map.len(),
            additions.new_members.len(),
            additions.add_dependencies.len()
        );
    } else {
        print!("{rewritten}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A non-injective rename map (two olds → one new) fails closed at load.
    #[test]
    fn test_rename_map_non_injective_target_fails_closed() {
        let dir = tempfile::tempdir().unwrap();
        let map_path = dir.path().join("map.tsv");
        std::fs::write(&map_path, "old-a\tnew-x\nold-b\tnew-x\n").unwrap();
        let err = load_rename_map(map_path.to_str().unwrap(), false).unwrap_err();
        assert!(
            err.to_string().contains("non-injective"),
            "unexpected: {err}"
        );
    }
}
