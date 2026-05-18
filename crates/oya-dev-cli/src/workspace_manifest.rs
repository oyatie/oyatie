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
    let manifest = fs::read_to_string(path)
        .map_err(|error| format!("workspace manifest unreadable: {error}"))?;
    let members_start = manifest
        .find("members")
        .ok_or_else(|| "workspace manifest missing members array".to_string())?;
    let after_members = &manifest[members_start..];
    let list_start = after_members
        .find('[')
        .ok_or_else(|| "workspace manifest members missing '['".to_string())?;
    let after_list_start = &after_members[list_start + 1..];
    let list_end = after_list_start
        .find(']')
        .ok_or_else(|| "workspace manifest members missing ']'".to_string())?;
    let members = &after_list_start[..list_end];
    let member_paths = members
        .split(',')
        .filter_map(|entry| {
            let trimmed = entry.trim();
            let first_quote = trimmed.find('"')?;
            let rest = &trimmed[first_quote + 1..];
            let second_quote = rest.find('"')?;
            let path = &rest[..second_quote];
            if path.is_empty() {
                None
            } else {
                Some(path.to_string())
            }
        })
        .collect::<Vec<_>>();
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

    use super::read_workspace_member_paths;

    #[test]
    fn workspace_member_parser_ignores_comments_in_members_array() {
        let dir = std::env::temp_dir().join(format!(
            "oya-workspace-manifest-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("fixture dir created");
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
}
