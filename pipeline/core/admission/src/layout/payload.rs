//! Closed owner-local policy and desired-state payload grammars.

const FORBIDDEN_PAYLOAD_DIRS: &[&str] = &[
    "plan",
    "tasks",
    "helm",
    "charts",
    "chart",
    "tofu",
    "terraform",
    "kustomize",
];

pub(super) fn validate_cedar(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    validate_payload(file, parts, &["cedar", "cedarschema"], "Cedar", violations);
}

pub(super) fn validate_iac(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    validate_payload(
        file,
        parts,
        &["proto", "textproto"],
        "IaC protobuf IR",
        violations,
    );
}

fn validate_payload(
    file: &str,
    parts: &[&str],
    extensions: &[&str],
    kind: &str,
    violations: &mut Vec<String>,
) {
    let Some((name, directories)) = parts.split_last() else {
        violations.push(format!("{file}: {kind} requires a payload file"));
        return;
    };
    if let Some(forbidden) = directories
        .iter()
        .find(|directory| FORBIDDEN_PAYLOAD_DIRS.contains(directory))
    {
        violations.push(format!(
            "{file}: `{forbidden}/` is not allowed beneath owner {kind}"
        ));
        return;
    }
    if directories.iter().any(|directory| !segment_name(directory)) {
        violations.push(format!(
            "{file}: {kind} directories must use lowercase snake/kebab names"
        ));
        return;
    }
    if matches!(*name, "OWNERS" | "BUCK") {
        return;
    }
    let valid = extensions.iter().any(|extension| {
        name.strip_suffix(&format!(".{extension}"))
            .is_some_and(segment_name)
    });
    if !valid {
        violations.push(format!(
            "{file}: {kind} files must use one of these extensions: {}",
            extensions.join(", ")
        ));
    }
}

fn segment_name(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && !name.ends_with('-')
        && !name.ends_with('_')
        && !name.contains("--")
        && !name.contains("__")
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}
