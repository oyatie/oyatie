//! Inner owner grammar for ADR-0719 D-8.

use super::payload::{validate_cedar, validate_iac};
use super::{FORBIDDEN_NAMES, cap_root_file_ok, face_dir_ok};

// D-41 explicitly amends the earlier closed list with a tiny owned build.rs
// that writes generated module membership to OUT_DIR.
const CRATE_FILES: &[&str] = &["Cargo.toml", "OWNERS", "BUCK", "build.rs"];
const DOC_DIRS: &[&str] = &["concepts", "runbooks", "design"];
const INNER_DUMP_DIRS: &[&str] = &["plan", "tasks"];

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
            validate_face_path(
                file,
                parts[child_index - 1],
                child,
                &parts[child_index + 1..],
                violations,
            );
        }
        "cedar" => validate_cedar(file, &parts[child_index + 1..], violations),
        "iac" => validate_iac(file, &parts[child_index + 1..], violations),
        "observability" => {
            validate_observability(file, &parts[child_index + 1..], violations);
        }
        "docs" => validate_docs(file, &parts[child_index + 1..], violations),
        _ => violations.push(format!("{file}: `{child}` is not an owner face")),
    }
}

fn validate_face_path(
    file: &str,
    owner: &str,
    face: &str,
    rest: &[&str],
    violations: &mut Vec<String>,
) {
    if face == "facade" && rest.first() == Some(&"proto") {
        validate_proto(file, owner, rest, violations);
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
    validate_crate_path(file, face, crate_path, violations);
}

fn validate_crate_path(file: &str, face: &str, parts: &[&str], violations: &mut Vec<String>) {
    let Some((crate_name, tail)) = parts.split_first() else {
        violations.push(format!("{file}: face requires a crate directory"));
        return;
    };
    if !crate_leaf_ok(face, crate_name) {
        violations.push(format!(
            "{file}: `{crate_name}` does not match the `{face}` crate-leaf grammar"
        ));
    }
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
    } else if matches!(*entry, "src" | "tests") {
        validate_rust_tree(file, face, entry, descendants, violations);
    } else {
        violations.push(format!(
            "{file}: crate content must live under `src/` or `tests/`"
        ));
    }
}

fn crate_leaf_ok(face: &str, name: &str) -> bool {
    if !kebab_case(name)
        || name.starts_with("cloud-")
        || name.starts_with("oyatie-")
        || name.ends_with("-rs")
        || name.ends_with("-rust")
        || matches!(face, "ports" | "adapters") && name.ends_with("-draft")
    {
        return false;
    }
    match face {
        "adapters" => name.contains('-'),
        "facade" => {
            name == "app"
                || name
                    .strip_suffix("-app")
                    .is_some_and(|surface| !surface.is_empty() && kebab_case(surface))
        }
        "core" | "ports" => true,
        _ => false,
    }
}

fn kebab_case(name: &str) -> bool {
    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn validate_rust_tree(
    file: &str,
    face: &str,
    tree: &str,
    parts: &[&str],
    violations: &mut Vec<String>,
) {
    let Some((source, directories)) = parts.split_last() else {
        return;
    };
    if tree == "src" && directories.first() == Some(&"bin") {
        violations.push(format!(
            "{file}: `src/bin/` bypasses the canonical face entry point"
        ));
        return;
    }
    if tree == "src" && directories.is_empty() && *source == "main.rs" && face != "facade" {
        violations.push(format!(
            "{file}: `{face}` is a library face and must use `src/lib.rs`, not `src/main.rs`"
        ));
        return;
    }
    for directory in directories {
        if INNER_DUMP_DIRS.contains(directory) {
            violations.push(format!("{file}: forbidden inner directory `{directory}`"));
            return;
        }
        if !snake_case(directory) {
            violations.push(format!(
                "{file}: Rust module directory `{directory}` must be snake_case"
            ));
            return;
        }
    }
    let valid_source = source.strip_suffix(".rs").is_some_and(snake_case);
    if !valid_source {
        violations.push(format!(
            "{file}: crate source and integration-test files must be snake_case `.rs` files"
        ));
    }
}

fn validate_proto(file: &str, owner: &str, parts: &[&str], violations: &mut Vec<String>) {
    let valid_shape = parts.len() == 5
        && parts[0] == "proto"
        && parts[1] == owner
        && snake_case(parts[1])
        && snake_case(parts[2])
        && parts[3] == "v1";
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
    stem != "v1" && snake_case(stem)
}

fn snake_case(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && !name.ends_with('_')
        && !name.contains("__")
        && name
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
    } else if parts[1..parts.len() - 1]
        .iter()
        .any(|part| INNER_DUMP_DIRS.contains(part))
    {
        violations.push(format!(
            "{file}: owner docs must not contain `plan/` or `tasks/` dumps"
        ));
    }
}
