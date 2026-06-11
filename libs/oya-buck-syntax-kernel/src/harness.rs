//! Fixer self-validation harness (ADR-0549): the write-through guard every BUCK/manifest fixer
//! must route rewrites through before persisting.
//!
//! Contract (pinned by corruption-refusal fixtures in `lib.rs`, including the historical
//! vectors: missing comma, double comma from comment-blind heuristics, dangling feature refs):
//! 1. [`guarded_rewrite`] (a) REPARSES the candidate content with this kernel — a structurally
//!    corrupt rewrite is refused before any caller-visible success; (b) runs the CALLER-SUPPLIED
//!    semantic validation hook over the parsed document; (c) on ANY failure refuses and returns
//!    the pre-image, so the caller's only sound move is to keep/restore the original bytes.
//! 2. [`PreImageRegistry`] keeps the FIRST pre-image per path key — a file edited twice rolls
//!    back to its ORIGINAL content, never an intermediate state (the #693 LOW-X3 class).
//! 3. Rollback iteration is DETERMINISTIC (path-ordered), so failure handling is reproducible.
//!
//! The harness is pure over strings: the CALLER owns all filesystem I/O (R0 kernel purity).

use std::collections::BTreeMap;

use crate::parser::{BuckDoc, parse};

/// First-pre-image-wins registry of original file contents, keyed by a caller-chosen path key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PreImageRegistry {
    images: BTreeMap<String, String>,
}

impl PreImageRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `pre_image` for `path_key`. The FIRST recorded image wins; later records for the
    /// same key are ignored (a second edit to the same file must roll back to the ORIGINAL).
    pub fn record(&mut self, path_key: &str, pre_image: &str) {
        self.images
            .entry(path_key.to_owned())
            .or_insert_with(|| pre_image.to_owned());
    }

    /// The recorded pre-image for `path_key`, if any.
    pub fn get(&self, path_key: &str) -> Option<&str> {
        self.images.get(path_key).map(String::as_str)
    }

    /// Deterministic (path-ordered) iteration over every recorded pre-image, for rollback.
    pub fn images(&self) -> impl Iterator<Item = (&str, &str)> {
        self.images
            .iter()
            .map(|(path, content)| (path.as_str(), content.as_str()))
    }

    pub fn is_empty(&self) -> bool {
        self.images.is_empty()
    }

    pub fn len(&self) -> usize {
        self.images.len()
    }
}

/// A refused rewrite. Carries the pre-image so the caller restores the original bytes — the
/// ONLY sound outcome of a failed validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardRefusal {
    /// Why the rewrite was refused.
    pub reason: String,
    /// The original content to keep/restore.
    pub pre_image: String,
}

impl std::fmt::Display for GuardRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rewrite refused (pre-image preserved): {}", self.reason)
    }
}

impl std::error::Error for GuardRefusal {}

/// The write-through guard. Records the pre-image (first wins), reparses `candidate` with this
/// kernel, runs the caller's semantic hook over the parsed document, and either returns the
/// candidate (safe to persist) or refuses with the pre-image (caller persists NOTHING / restores).
///
/// The semantic hook receives the parsed candidate document plus the candidate text and returns
/// `Err(reason)` to refuse — e.g. "target X vanished", "injected value not visible after
/// reparse", "dangling feature reference".
pub fn guarded_rewrite<F>(
    path_key: &str,
    pre_image: &str,
    candidate: &str,
    registry: &mut PreImageRegistry,
    semantic: F,
) -> Result<String, GuardRefusal>
where
    F: FnOnce(&BuckDoc, &str) -> Result<(), String>,
{
    registry.record(path_key, pre_image);
    let doc = match parse(candidate) {
        Ok(doc) => doc,
        Err(parse_error) => {
            return Err(GuardRefusal {
                reason: format!(
                    "self-validation reparse failed — rewritten content is structurally corrupt; refusing write ({parse_error})"
                ),
                pre_image: pre_image.to_owned(),
            });
        }
    };
    if let Err(reason) = semantic(&doc, candidate) {
        return Err(GuardRefusal {
            reason: format!("semantic validation failed — refusing write ({reason})"),
            pre_image: pre_image.to_owned(),
        });
    }
    Ok(candidate.to_owned())
}
