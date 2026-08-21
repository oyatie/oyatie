use std::fs;
use std::path::Path;

pub(crate) fn read_workspace_member_crate_ids(path: &Path) -> Result<Vec<String>, String> {
    let crate_ids = read_workspace_member_paths(path)?
        .into_iter()
        .filter_map(|path| {
            Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    if crate_ids.is_empty() {
        Err("workspace manifest members array is empty".to_string())
    } else {
        Ok(crate_ids)
    }
}

pub(crate) fn read_workspace_member_paths(path: &Path) -> Result<Vec<String>, String> {
    let repo_root = path
        .parent()
        .ok_or_else(|| format!("workspace manifest has no parent: {}", path.display()))?;
    let member_paths = workspace_members_kernel::resolve_member_dirs(repo_root)
        .map_err(|error| format!("workspace manifest members unresolved: {error}"))?;
    if member_paths.is_empty() {
        Err("workspace manifest members array is empty".to_string())
    } else {
        Ok(member_paths)
    }
}

pub(crate) fn read_package_license(path: &Path) -> Result<String, String> {
    let manifest = fs::read_to_string(path)
        .map_err(|error| format!("package manifest unreadable: {error}"))?;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("license") {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "license" {
            continue;
        }
        return Ok(value.trim().trim_matches('"').to_string());
    }
    Err(format!(
        "package manifest missing license: {}",
        path.display()
    ))
}

pub(crate) fn read_package_name(path: &Path) -> Result<String, String> {
    let manifest = fs::read_to_string(path)
        .map_err(|error| format!("package manifest unreadable: {error}"))?;
    let mut in_package_section = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package_section = trimmed == "[package]";
            continue;
        }
        if !in_package_section {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        let package_name = value.trim().trim_matches('"').to_string();
        if package_name.is_empty() {
            break;
        }
        return Ok(package_name);
    }
    Err(format!(
        "package manifest missing package name: {}",
        path.display()
    ))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::read_workspace_member_paths;

    fn write_member(root: &Path, relative: &str) {
        let dir = root.join(relative);
        fs::create_dir_all(&dir).expect("member dir created");
        fs::write(
            dir.join("Cargo.toml"),
            format!(
                "[package]\nname = \"{}\"\nedition = \"2024\"\nversion = \"0.1.0\"\nlicense = \"Apache-2.0\"\n",
                relative.replace('/', "-")
            ),
        )
        .expect("member manifest written");
    }

    #[test]
    fn workspace_member_parser_ignores_comments_in_members_array() {
        let dir = std::env::temp_dir().join(format!(
            "oya-workspace-manifest-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("fixture dir created");
        write_member(&dir, "crates/one");
        write_member(&dir, "crates/two");
        let manifest = dir.join("Cargo.toml");
        fs::write(
            &manifest,
            r#"[workspace]
members = [
  "crates/one",
  # comment inside TOML array
  "crates/two",
  # another comment
]
"#,
        )
        .expect("manifest written");

        let members = read_workspace_member_paths(&manifest).expect("members parsed");
        assert_eq!(members, vec!["crates/one", "crates/two"]);

        fs::remove_file(manifest).expect("manifest removed");
        fs::remove_dir_all(dir).expect("fixture dir removed");
    }

    #[test]
    fn workspace_member_parser_expands_globs_and_honors_exclude() {
        let dir = std::env::temp_dir().join(format!(
            "oya-workspace-manifest-glob-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("fixture dir created");
        write_member(&dir, "libs/oya-one");
        write_member(&dir, "libs/oya-two");
        write_member(&dir, "cloud/cloud-kernel/crates/oya-excluded-kernel");
        let manifest = dir.join("Cargo.toml");
        fs::write(
            &manifest,
            r#"[workspace]
members = [
  "libs/oya-*",
  "cloud/*/crates/oya-*",
]
exclude = [
  "cloud/cloud-kernel",
]
"#,
        )
        .expect("manifest written");

        let members = read_workspace_member_paths(&manifest).expect("members parsed");
        assert_eq!(members, vec!["libs/oya-one", "libs/oya-two"]);

        fs::remove_file(manifest).expect("manifest removed");
        fs::remove_dir_all(dir).expect("fixture dir removed");
    }
}
