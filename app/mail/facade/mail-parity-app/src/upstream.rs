use super::{PIN, compare, git, inspect};
use serde_json::{Value, json};
use std::{path::Path, time::Duration};

fn stable_tag(release: &Value) -> Result<&str, String> {
    let tag = release["tag_name"].as_str().ok_or("release tag missing")?;
    if release["draft"] != false || release["prerelease"] != false {
        return Err("release is not stable".into());
    }
    let parts: Vec<_> = tag.strip_prefix('v').unwrap_or("").split('.').collect();
    if parts.len() != 3
        || parts
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()))
        || tag.len() > 64
    {
        return Err("unrecognized stable release tag".into());
    }
    Ok(tag)
}

pub fn refresh(checkout: &Path) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("oyatie-mail-parity")
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;
    let release: Value = client
        .get("https://api.github.com/repos/stalwartlabs/stalwart/releases/latest")
        .send()
        .and_then(reqwest::blocking::Response::error_for_status)
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    let tag = stable_tag(&release)?;
    git(
        checkout,
        &[
            "fetch",
            "--depth",
            "1",
            "https://github.com/stalwartlabs/stalwart.git",
            &format!("refs/tags/{tag}:refs/tags/{tag}"),
            "refs/heads/main:refs/remotes/stalwart/main",
        ],
    )?;
    let baseline = inspect(checkout, PIN)?;
    let stable = inspect(checkout, tag)?;
    let main = inspect(checkout, "refs/remotes/stalwart/main")?;
    let stable_diff = compare(&baseline, &stable)?;
    let main_diff = compare(&stable, &main)?;
    let changed = [&stable_diff, &main_diff].iter().any(|r| {
        r["changes"].as_array().is_none_or(|a| !a.is_empty())
            || r["target_mapping_complete"] != true
    });
    println!(
        "{}",
        serde_json::to_string_pretty(
            &json!({"latest_stable":tag,"stable":stable_diff,"main":main_diff,"full_parity":false})
        )
        .map_err(|e| e.to_string())?
    );
    if changed {
        Err("upstream changes require compatibility implementation and qualification".into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_stable_numeric_tags_can_be_used_as_fetch_refspecs() {
        assert_eq!(
            stable_tag(&json!({"tag_name":"v0.16.22","draft":false,"prerelease":false})).unwrap(),
            "v0.16.22"
        );
        for tag in [
            "main",
            "--upload-pack=bad",
            "v1.2.3:refs/heads/dev",
            "v1.2.3-rc1",
            "v1.2",
            "v1..3",
        ] {
            assert!(stable_tag(&json!({"tag_name":tag,"draft":false,"prerelease":false})).is_err());
        }
        assert!(stable_tag(&json!({"tag_name":"v1.2.3","draft":false,"prerelease":true})).is_err());
    }
}
