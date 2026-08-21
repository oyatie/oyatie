//! Pure hierarchy rules: namespace normalization, path derivation, the
//! parent-kind table, and edge validation. No I/O, no clock, no randomness —
//! every function is a total function of its arguments.

use crate::kernel::{SubScope, SubScopeId, SubScopeKernelError, SubScopeKind, SubScopePath};

/// Maximum number of edges between the tenant root and any scope.
///
/// The root sits at depth 0, so the deepest legal materialized path holds
/// `MAX_DEPTH + 1` segments. IP-016 §D.4 fixes this at 6.
pub const MAX_DEPTH: usize = 6;

/// Maximum length, in bytes, of one normalized path segment.
pub const MAX_SEGMENT_LEN: usize = 63;

/// The one separator used when a path is rendered as a string.
///
/// It is deliberately NOT legal inside a segment, so rendering is injective
/// and a rendered path can be split back into the same segments.
pub const PATH_SEPARATOR: char = '/';

/// Normalize and validate one path segment.
///
/// Normalization (applied first, never an error on its own):
/// - surrounding ASCII whitespace is trimmed;
/// - ASCII `A-Z` is folded to `a-z`, so `Atlas` and `atlas` are the same
///   segment and cannot both exist as siblings.
///
/// Validation (all rejections are [`SubScopeKernelError::NamespaceMalformed`]):
/// - 1..=[`MAX_SEGMENT_LEN`] bytes after normalization;
/// - the permitted character set is exactly ASCII `a-z`, `0-9` and `-`, so
///   `/`, `.`, `_`, spaces and every non-ASCII code point are refused (no
///   Unicode confusables in an authorization key);
/// - the first character is an ASCII letter and the last is alphanumeric;
/// - no `--` run, which keeps a segment safe to reuse as a DNS label.
///
/// # Errors
/// [`SubScopeKernelError::NamespaceMalformed`] when any rule above fails.
pub fn normalize_segment(raw: &str) -> Result<String, SubScopeKernelError> {
    let trimmed = raw.trim_matches(|c: char| c.is_ascii_whitespace());
    if trimmed.is_empty() || trimmed.len() > MAX_SEGMENT_LEN {
        return Err(SubScopeKernelError::NamespaceMalformed);
    }
    if !trimmed.is_ascii() {
        return Err(SubScopeKernelError::NamespaceMalformed);
    }
    let normalized = trimmed.to_ascii_lowercase();
    let bytes = normalized.as_bytes();
    let legal = bytes
        .iter()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
    if !legal {
        return Err(SubScopeKernelError::NamespaceMalformed);
    }
    let starts_ok = bytes.first().is_some_and(u8::is_ascii_lowercase);
    let ends_ok = bytes.last().is_some_and(u8::is_ascii_alphanumeric);
    if !starts_ok || !ends_ok || normalized.contains("--") {
        return Err(SubScopeKernelError::NamespaceMalformed);
    }
    Ok(normalized)
}

/// Normalize and validate every segment of a whole path.
///
/// # Errors
/// [`SubScopeKernelError::NamespaceMalformed`] if any segment is malformed or
/// the path is empty; [`SubScopeKernelError::DepthExceeded`] if it is deeper
/// than [`MAX_DEPTH`].
pub fn normalize_path(segments: &[String]) -> Result<SubScopePath, SubScopeKernelError> {
    if segments.is_empty() {
        return Err(SubScopeKernelError::NamespaceMalformed);
    }
    if segments.len() > MAX_DEPTH + 1 {
        return Err(SubScopeKernelError::DepthExceeded);
    }
    let mut normalized = Vec::with_capacity(segments.len());
    for segment in segments {
        normalized.push(normalize_segment(segment)?);
    }
    Ok(SubScopePath::new(normalized))
}

/// Derive a child's canonical path from its parent's path and its own segment.
///
/// This is the single derivation rule; a stored path that differs from this
/// result is refused as [`SubScopeKernelError::PathInconsistent`] rather than
/// stored, so the adjacency list and the materialized path can never drift.
///
/// # Errors
/// [`SubScopeKernelError::NamespaceMalformed`] for a bad segment;
/// [`SubScopeKernelError::DepthExceeded`] when the child would sit deeper
/// than [`MAX_DEPTH`].
pub fn canonical_path(
    parent_path: &SubScopePath,
    segment: &str,
) -> Result<SubScopePath, SubScopeKernelError> {
    let leaf = normalize_segment(segment)?;
    if parent_path.depth() + 1 > MAX_DEPTH {
        return Err(SubScopeKernelError::DepthExceeded);
    }
    let mut segments = parent_path.segments().to_vec();
    segments.push(leaf);
    Ok(SubScopePath::new(segments))
}

/// Rewrite `path` when its `old_prefix` moves to `new_prefix`.
///
/// Returns `None` when `old_prefix` is not a prefix of `path`. Used to keep a
/// whole subtree's materialized paths consistent after a rename or reparent.
#[must_use]
pub fn reroot_path(
    path: &SubScopePath,
    old_prefix: &SubScopePath,
    new_prefix: &SubScopePath,
) -> Option<SubScopePath> {
    let suffix = path.segments().get(old_prefix.segments().len()..)?;
    if !path.segments().starts_with(old_prefix.segments()) {
        return None;
    }
    let mut segments = new_prefix.segments().to_vec();
    segments.extend_from_slice(suffix);
    Some(SubScopePath::new(segments))
}

/// The parent-kind table.
///
/// Two rules, both from IP-016 §D.4 (AWS Organizations' OU constraints are
/// the precedent):
/// 1. a [`SubScopeKind::BusinessUnit`] may hang only under another
///    `BusinessUnit`, so the org tree stays contiguous from the root and an
///    engagement can never own a department;
/// 2. a [`SubScopeKind::Counterparty`] is terminal — nothing hangs under it,
///    because an external party's mirror scope must not grow internal
///    structure underneath it.
///
/// Every other combination is permitted.
#[must_use]
pub fn kind_may_parent(parent: SubScopeKind, child: SubScopeKind) -> bool {
    if matches!(parent, SubScopeKind::Counterparty) {
        return false;
    }
    if matches!(child, SubScopeKind::BusinessUnit) {
        return matches!(parent, SubScopeKind::BusinessUnit);
    }
    true
}

/// Validate a proposed parent -> child edge.
///
/// `parent_ancestors` is the parent's ancestor chain (nearest first) as read
/// through a tenant-scoped port; it is what makes the long, indirect cycle
/// detectable and not just self-parenting.
///
/// Checks run in a fixed order so the reported reason is stable: tenant
/// boundary, cycle, parent kind, depth.
///
/// # Errors
/// [`SubScopeKernelError::TenantBoundaryViolation`],
/// [`SubScopeKernelError::CycleRefused`],
/// [`SubScopeKernelError::ParentKindNotAllowed`] or
/// [`SubScopeKernelError::DepthExceeded`].
pub fn validate_new_edge(
    parent: &SubScope,
    child: &SubScope,
    parent_ancestors: &[SubScopeId],
) -> Result<(), SubScopeKernelError> {
    if parent.tenant_id != child.tenant_id {
        return Err(SubScopeKernelError::TenantBoundaryViolation);
    }
    if parent.id == child.id || parent_ancestors.contains(&child.id) {
        return Err(SubScopeKernelError::CycleRefused);
    }
    if !kind_may_parent(parent.kind, child.kind) {
        return Err(SubScopeKernelError::ParentKindNotAllowed);
    }
    if parent.depth() + 1 > MAX_DEPTH {
        return Err(SubScopeKernelError::DepthExceeded);
    }
    Ok(())
}

/// Reject a record whose materialized path does not match its parent chain.
///
/// A root (no parent) must own a single-segment path; a child's path must be
/// exactly `canonical_path(parent.path, child.leaf)`.
///
/// # Errors
/// [`SubScopeKernelError::PathInconsistent`] on any mismatch, or the error
/// raised by [`canonical_path`] / [`normalize_path`].
pub fn validate_path_consistency(
    parent: Option<&SubScope>,
    scope: &SubScope,
) -> Result<(), SubScopeKernelError> {
    let normalized = normalize_path(scope.path.segments())?;
    if normalized != scope.path {
        return Err(SubScopeKernelError::PathInconsistent);
    }
    let leaf = scope
        .path
        .leaf()
        .ok_or(SubScopeKernelError::NamespaceMalformed)?;
    match parent {
        None => {
            if scope.path.segments().len() == 1 {
                Ok(())
            } else {
                Err(SubScopeKernelError::PathInconsistent)
            }
        }
        Some(parent) => {
            if canonical_path(&parent.path, leaf)? == scope.path {
                Ok(())
            } else {
                Err(SubScopeKernelError::PathInconsistent)
            }
        }
    }
}

/// Maximum length, in bytes, of a tenant id.
pub const MAX_TENANT_ID_LEN: usize = 64;

/// Validate a tenant id.
///
/// A tenant id is opaque and minted upstream, so it is NOT normalized here —
/// case is significant and two ids that differ only in case are two tenants.
/// It is, however, held to a character set for the same reason a path segment
/// is: this crate is the namespace validator its consumers trust, and the
/// value is reused verbatim as a storage key (IP-023's `sub_scopes` rows), as
/// a policy entity id (`tenancy/policy/*.cedar`) and in log lines. A kernel
/// that advertises a validated namespace must not pass a quote, a semicolon,
/// a separator or a control character through to those.
///
/// The rule: 1..=[`MAX_TENANT_ID_LEN`] bytes, every byte in ASCII `A-Za-z`,
/// `0-9`, `_` or `-`, and the first byte alphanumeric. Whitespace, `/`, `.`,
/// quotes, backslashes and every C0 control byte are refused, as is every
/// non-ASCII code point (no Unicode confusables in a tenant key).
///
/// # Errors
/// [`SubScopeKernelError::TenantBoundaryViolation`] when the id breaks any
/// rule above — an unattributable request is by definition outside every
/// tenant, so the boundary error is the accurate one.
pub fn validate_tenant_id(tenant_id: &str) -> Result<(), SubScopeKernelError> {
    let bytes = tenant_id.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_TENANT_ID_LEN {
        return Err(SubScopeKernelError::TenantBoundaryViolation);
    }
    let legal = bytes
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'-');
    let starts_ok = bytes.first().is_some_and(u8::is_ascii_alphanumeric);
    if !legal || !starts_ok {
        return Err(SubScopeKernelError::TenantBoundaryViolation);
    }
    Ok(())
}
