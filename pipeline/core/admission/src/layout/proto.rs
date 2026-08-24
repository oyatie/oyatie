//! Protobuf package identity checks for changed sold-facade contracts.

use super::path_parts;

pub fn proto_package_violations(path: &str, contents: &str) -> Vec<String> {
    let Some(expected) = expected_proto_package(path) else {
        return Vec::new();
    };
    let packages = declared_packages(contents);
    if packages.as_slice() == [expected.as_str()] {
        Vec::new()
    } else {
        vec![format!(
            "{path}: protobuf package must be exactly `{expected}`, got {}",
            if packages.is_empty() {
                "<missing>".to_owned()
            } else {
                packages.join(", ")
            }
        )]
    }
}

fn expected_proto_package(path: &str) -> Option<String> {
    let parts = path_parts(path);
    let (owner, facade_index) = if parts.first() == Some(&"app") {
        (*parts.get(1)?, 2)
    } else {
        (*parts.first()?, 1)
    };
    let tail = parts.get(facade_index..)?;
    if tail.len() != 6
        || tail[0] != "facade"
        || tail[1] != "proto"
        || tail[2] != owner
        || tail[4] != "v1"
        || !tail[5].ends_with(".proto")
    {
        return None;
    }
    Some(format!("{owner}.{}.v1", tail[3]))
}

fn declared_packages(contents: &str) -> Vec<String> {
    let source = without_comments_and_strings(contents);
    let mut packages = Vec::new();
    let mut statement = String::new();
    let mut brace_depth = 0_u32;
    for character in source.chars() {
        match character {
            '{' => {
                if brace_depth == 0 {
                    statement.clear();
                }
                brace_depth += 1;
            }
            '}' => {
                brace_depth = brace_depth.saturating_sub(1);
                if brace_depth == 0 {
                    statement.clear();
                }
            }
            ';' if brace_depth == 0 => {
                if let Some(package) = package_statement(&statement) {
                    packages.push(package);
                }
                statement.clear();
            }
            _ if brace_depth == 0 => statement.push(character),
            _ => {}
        }
    }
    packages
}

fn package_statement(statement: &str) -> Option<String> {
    let rest = statement.trim().strip_prefix("package")?;
    if !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let package: String = rest
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect();
    (!package.is_empty()).then_some(package)
}

fn without_comments_and_strings(contents: &str) -> String {
    let bytes = contents.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match (bytes[index], bytes.get(index + 1).copied()) {
            (b'/', Some(b'/')) => {
                output.push(b' ');
                index += 2;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
            }
            (b'/', Some(b'*')) => {
                output.push(b' ');
                index += 2;
                while index + 1 < bytes.len() && (bytes[index] != b'*' || bytes[index + 1] != b'/')
                {
                    index += 1;
                }
                index = (index + 2).min(bytes.len());
            }
            (quote @ (b'"' | b'\''), _) => {
                output.push(b' ');
                index += 1;
                while index < bytes.len() {
                    if bytes[index] == b'\\' {
                        index = (index + 2).min(bytes.len());
                    } else if bytes[index] == quote {
                        index += 1;
                        break;
                    } else {
                        index += 1;
                    }
                }
            }
            (byte, _) => {
                output.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&output).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_must_match_the_sold_proto_path() {
        let path = "network/facade/proto/network/edge/v1/edge_service.proto";
        assert!(
            proto_package_violations(path, "syntax='proto3'; package network.edge.v1;").is_empty()
        );
        assert!(!proto_package_violations(path, "package iam.edge.v1;").is_empty());
        assert!(
            !proto_package_violations(path, "// package network.edge.v1;\nmessage Edge {}")
                .is_empty()
        );
        assert!(
            !proto_package_violations(
                path,
                "option fake = ';package network.edge.v1;'; message Edge {}",
            )
            .is_empty()
        );
        assert!(
            !proto_package_violations(
                path,
                "message Edge { string name = 1; package network.edge.v1; }",
            )
            .is_empty()
        );
        assert!(
            proto_package_violations(path, "package/* package whitespace */network.edge.v1;")
                .is_empty()
        );
    }
}
