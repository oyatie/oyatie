//! Git plumbing for the occupancy collector.
//!
//! Two readers, deliberately distinct: one returns an object id and
//! validates that it is one, the other returns file content and must not.

use std::process::{Command, Output};

use pipeline_admission::{GitChangePaths, git_change_paths_from_name_status_z};

use super::REMOTE;

pub(crate) fn git_change_paths(
    token: &str,
    merge_base: &str,
    head: &str,
) -> Result<GitChangePaths, String> {
    let output = git_output(
        token,
        &["diff", "--name-status", "-z", "-M", merge_base, head, "--"],
    )?;
    git_change_paths_from_name_status_z(&output.stdout).map_err(|error| error.message())
}

/// Reads a git BLOB. Deliberately does no object-id validation: the content is
/// arbitrary text. `git_text` cannot serve this purpose — it rejects any byte
/// that is not hex, so `.gitattributes` fails at its first character.
pub(crate) fn git_blob_text(token: &str, args: &[&str]) -> Result<String, String> {
    let output = git_output(token, args)?;
    String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8 output", args[0]))
}

/// Reads a git OBJECT ID, and validates that it is one.
pub(crate) fn git_text(token: &str, args: &[&str]) -> Result<String, String> {
    let output = git_output(token, args)?;
    let value = String::from_utf8(output.stdout)
        .map_err(|_| format!("git {} returned non-UTF-8 output", args[0]))?;
    let value = value.trim_end_matches(['\r', '\n']);
    if !is_object_id(value) {
        return Err(format!("git {} returned an invalid object id", args[0]));
    }
    Ok(value.to_owned())
}

pub(crate) fn is_object_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn git_output(token: &str, args: &[&str]) -> Result<Output, String> {
    let auth = format!(
        "AUTHORIZATION: basic {}",
        base64(format!("x-access-token:{token}").as_bytes())
    );
    let mut command = Command::new("git");
    command
        .args(args)
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "http.https://github.com/.extraheader")
        .env("GIT_CONFIG_VALUE_0", auth);
    command_output(command, &format!("git {}", args[0]))
}

pub(crate) fn command_output(mut command: Command, label: &str) -> Result<Output, String> {
    let output = command
        .output()
        .map_err(|error| format!("start {label}: {error}"))?;
    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("{label} failed: {}", stderr.trim()))
    }
}

pub(crate) fn base64(input: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut encoded = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let a = chunk[0];
        let b = *chunk.get(1).unwrap_or(&0);
        let c = *chunk.get(2).unwrap_or(&0);
        encoded.push(TABLE[(a >> 2) as usize] as char);
        encoded.push(TABLE[(((a & 0x03) << 4) | (b >> 4)) as usize] as char);
        if chunk.len() > 1 {
            encoded.push(TABLE[(((b & 0x0f) << 2) | (c >> 6)) as usize] as char);
        } else {
            encoded.push('=');
        }
        if chunk.len() > 2 {
            encoded.push(TABLE[(c & 0x3f) as usize] as char);
        } else {
            encoded.push('=');
        }
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_content_is_not_an_object_id() {
        // `git_text` validates its output IS a hex object id, because it was
        // written for `merge-base`. Reading `.gitattributes` through it always
        // failed — at the first byte, `#` — and paired with a defaulting
        // unwrap that made the whole exemption silently empty while every gate
        // stayed green. `git_blob_text` exists for content; this pins why.
        assert!(is_object_id("a355428b265db665a18c29e4fc0a35872fbd0053"));
        assert!(!is_object_id(""));
        assert!(
            !is_object_id("# Cargo.lock is generated"),
            "prose must never pass the object-id validator"
        );
        assert!(!is_object_id("Cargo.lock merge=cargo-lock"));
    }

    #[test]
    fn basic_auth_encoding_matches_git_https() {
        assert_eq!(
            base64(b"x-access-token:test"),
            "eC1hY2Nlc3MtdG9rZW46dGVzdA=="
        );
    }
}
