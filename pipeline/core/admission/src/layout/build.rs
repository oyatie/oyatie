//! Build meta-root grammar. Provenance: ADR-0719 D-8.

use super::inner::validate_owner_path;
use super::{FACES, manifest};

pub(super) fn validate_build_path(file: &str, parts: &[&str], violations: &mut Vec<String>) {
    let Some(child) = parts.get(1).copied() else {
        return;
    };
    if child == "port-engine" {
        if parts.last() == Some(&"PACKAGE") {
            violations.push(format!(
                "{file}: PACKAGE is allowed only at build-script dependency-declarations crate roots"
            ));
        }
        return;
    }
    if child == "dependency-declarations" {
        let (Some(face), Some(leaf)) = (parts.get(2).copied(), parts.get(3).copied()) else {
            violations.push(format!(
                "{file}: dependency-declarations requires a frozen face/crate pair"
            ));
            return;
        };
        let Some((_, allows_build_script)) = manifest::dependency_declarations_package(face, leaf)
        else {
            violations.push(format!(
                "{file}: `{face}/{leaf}` is not one of the six dependency-declarations crates"
            ));
            return;
        };
        if parts.len() == 5 && parts[4] == "PACKAGE" && allows_build_script {
            return;
        }
        if !allows_build_script && parts.get(4) == Some(&"build.rs") {
            violations.push(format!(
                "{file}: std-only port must not use root `build.rs`"
            ));
            return;
        }
        validate_owner_path(file, parts, 2, violations);
        return;
    }
    if parts.last() == Some(&"PACKAGE") {
        violations.push(format!(
            "{file}: PACKAGE is allowed only at build-script dependency-declarations crate roots"
        ));
    } else if FACES.contains(&child) {
        violations.push(format!(
            "{file}: meta root `build` cannot contain owner Cargo face `{child}`"
        ));
    } else if parts.get(2).is_some_and(|face| FACES.contains(face)) {
        violations.push(format!(
            "{file}: unapproved nested Build subsystem cannot contain owner face `{}`",
            parts[2]
        ));
    } else if parts[1..].iter().any(|part| {
        matches!(*part, "Cargo.toml" | "Cargo.lock" | "build.rs") || part.ends_with(".rs")
    }) {
        violations.push(format!(
            "{file}: nested Build Cargo/Rust content is allowed only under `build/port-engine` or `build/dependency-declarations`"
        ));
    }
}
