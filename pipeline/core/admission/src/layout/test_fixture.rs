//! Closed grammar for integration tests and their loaded data fixtures.

const DATA_EXTENSIONS: &[&str] = &["json", "txt"];
const FORBIDDEN_DIRS: &[&str] = &[".cargo", "bin", "plan", "src", "tasks"];

pub(super) fn validate_test_tree(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    let Some((name, directories)) = parts.split_last() else {
        return;
    };
    if name.ends_with(".rs") {
        validate_rust_test(file, name, directories, violations);
    } else {
        validate_data_fixture(file, name, directories, violations);
    }
}

fn validate_rust_test(file: &str, name: &str, directories: &[&str], violations: &mut Vec<String>) {
    if directories
        .iter()
        .any(|directory| FORBIDDEN_DIRS.contains(directory) || !snake_case(directory))
        || !name.strip_suffix(".rs").is_some_and(snake_case)
    {
        violations.push(format!(
            "{file}: integration-test modules must use snake_case `.rs` paths"
        ));
    }
}

fn validate_data_fixture(
    file: &str,
    name: &str,
    directories: &[&str],
    violations: &mut Vec<String>,
) {
    let bounded_location = directories.is_empty()
        || directories.first() == Some(&"fixtures") && directories.len() <= 2;
    let safe_directories = directories
        .iter()
        .all(|directory| !FORBIDDEN_DIRS.contains(directory) && fixture_component(directory));
    let safe_name = name.rsplit_once('.').is_some_and(|(stem, extension)| {
        fixture_component(stem) && DATA_EXTENSIONS.contains(&extension)
    });
    if !bounded_location || !safe_directories || !safe_name {
        violations.push(format!(
            "{file}: test data must be direct or `fixtures/<case>/` lowercase `.json`/`.txt` input"
        ));
    }
}

fn snake_case(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && !name.ends_with('_')
        && !name.contains("__")
        && name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn fixture_component(name: &str) -> bool {
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric)
        && !["--", "__", "-_", "_-"]
            .iter()
            .any(|separator| name.contains(separator))
        && name.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}
