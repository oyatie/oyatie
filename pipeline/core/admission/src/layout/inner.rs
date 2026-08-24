//! Inner owner grammar for ADR-0719 D-8.

use super::{FORBIDDEN_NAMES, cap_root_file_ok, face_dir_ok};

const CRATE_FILES: &[&str] = &["Cargo.toml", "OWNERS", "BUCK"];
const DOC_DIRS: &[&str] = &["concepts", "runbooks", "design"];

pub(super) fn validate_owner_path(
    file: &str,
    parts: &[&str],
    child_index: usize,
    violations: &mut Vec<String>,
) {
    let child = parts[child_index];
    if FORBIDDEN_NAMES.contains(&child) {
        violations.push(format!("{file}: forbidden child `{child}`"));
        return;
    }
    if parts.len() == child_index + 1 {
        if face_dir_ok(child) {
            violations.push(format!("{file}: `{child}` must be a directory"));
        } else if !cap_root_file_ok(child) {
            violations.push(format!(
                "{file}: `{child}` is not an owner face or law file"
            ));
        }
        return;
    }

    match child {
        "core" | "ports" | "adapters" | "facade" => {
            validate_face_path(file, child, &parts[child_index + 1..], violations);
        }
        "cedar" | "iac" => {}
        "observability" => {
            validate_observability(file, &parts[child_index + 1..], violations);
        }
        "docs" => validate_docs(file, &parts[child_index + 1..], violations),
        _ => violations.push(format!("{file}: `{child}` is not an owner face")),
    }
}

fn validate_face_path(file: &str, face: &str, rest: &[&str], violations: &mut Vec<String>) {
    if face == "facade" && rest.first() == Some(&"proto") {
        validate_proto(file, rest, violations);
        return;
    }
    let crate_path = if matches!(face, "ports" | "adapters") && rest.first() == Some(&"draft") {
        if rest.len() == 1 {
            violations.push(format!("{file}: `{face}/draft` must be a directory"));
            return;
        }
        &rest[1..]
    } else {
        rest
    };
    validate_crate_path(file, crate_path, violations);
}

fn validate_crate_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    let Some((_crate_name, tail)) = parts.split_first() else {
        violations.push(format!("{file}: face requires a crate directory"));
        return;
    };
    let Some((entry, descendants)) = tail.split_first() else {
        violations.push(format!("{file}: crate name must be a directory"));
        return;
    };
    if descendants.is_empty() {
        if matches!(*entry, "src" | "tests") {
            violations.push(format!("{file}: `{entry}` must be a directory"));
        } else if !CRATE_FILES.contains(entry) {
            violations.push(format!("{file}: `{entry}` is not allowed at a crate root"));
        }
    } else if !matches!(*entry, "src" | "tests") {
        violations.push(format!(
            "{file}: crate content must live under `src/` or `tests/`"
        ));
    }
}

fn validate_proto(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    let valid_shape = parts.len() == 5 && parts[0] == "proto" && parts[3] == "v1";
    let valid_file =
        valid_shape && (matches!(parts[4], "OWNERS" | "BUCK") || snake_case_proto(parts[4]));
    if !valid_file {
        violations.push(format!(
            "{file}: facade proto must be `proto/<owner>/<api>/v1/<snake_case>.proto`"
        ));
    }
}

fn snake_case_proto(name: &str) -> bool {
    let Some(stem) = name.strip_suffix(".proto") else {
        return false;
    };
    !stem.is_empty()
        && stem
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_observability(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts.first() != Some(&"slos") {
        violations.push(format!(
            "{file}: owner observability content must live under `observability/slos/`"
        ));
    } else if parts.len() == 1 {
        violations.push(format!("{file}: `observability/slos` must be a directory"));
    }
}

fn validate_docs(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    if parts == ["README.md"] {
        return;
    }
    let Some(section) = parts.first() else {
        violations.push(format!("{file}: owner docs require a canonical entry"));
        return;
    };
    if !DOC_DIRS.contains(section) {
        violations.push(format!("{file}: `{section}` is not an owner docs section"));
    } else if parts.len() == 1 {
        violations.push(format!("{file}: `docs/{section}` must be a directory"));
    }
}
