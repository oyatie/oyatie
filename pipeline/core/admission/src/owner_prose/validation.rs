use std::collections::BTreeSet;

use super::{OwnerProseNativeConsumer, OwnerProseProjection, OwnerProseWorkReference};

pub(super) fn semantic_claim_id(value: &str) -> bool {
    value.len() <= 128
        && value
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

pub(super) fn valid_work_reference(reference: &OwnerProseWorkReference) -> bool {
    let Some(rest) = reference.locator.strip_prefix("https://") else {
        return false;
    };
    let Some((authority, resource)) = rest.split_once('/') else {
        return false;
    };
    semantic_claim_id(&reference.system)
        && reference.locator.len() <= 2_048
        && !authority.is_empty()
        && authority.contains('.')
        && authority
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b':'))
        && !resource.is_empty()
        && !reference
            .locator
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
}

pub(super) fn valid_projection_target(
    projection: &OwnerProseProjection,
    sources: &BTreeSet<String>,
) -> bool {
    let path = projection.path.as_str();
    let parts: Vec<&str> = path.split('/').collect();
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || parts
            .iter()
            .any(|part| part.is_empty() || matches!(*part, "." | ".."))
        || sources.contains(path)
        || parts.contains(&"docs")
        || path.to_ascii_lowercase().ends_with(".md")
        || path.to_ascii_lowercase().ends_with(".markdown")
    {
        return false;
    }
    match projection.consumer {
        OwnerProseNativeConsumer::RustCompiler
        | OwnerProseNativeConsumer::RustTest
        | OwnerProseNativeConsumer::Runtime
        | OwnerProseNativeConsumer::Admission
        | OwnerProseNativeConsumer::Reconciler
        | OwnerProseNativeConsumer::SloController => path.ends_with(".rs"),
        OwnerProseNativeConsumer::CedarPolicyEngine => {
            path.ends_with(".cedar") || path.ends_with(".cedarschema")
        }
        OwnerProseNativeConsumer::ProtobufCompiler => path.ends_with(".proto"),
        OwnerProseNativeConsumer::Cargo => path.rsplit('/').next() == Some("Cargo.toml"),
        OwnerProseNativeConsumer::Buck => {
            path.rsplit('/').next() == Some("BUCK") || path.ends_with(".bzl")
        }
        OwnerProseNativeConsumer::OwnershipEnforcement => {
            path.rsplit('/').next() == Some("OWNERS") || path == ".github/CODEOWNERS"
        }
    }
}
