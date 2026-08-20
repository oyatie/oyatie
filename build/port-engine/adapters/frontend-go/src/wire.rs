//! Serde wire shapes for the snapshot envelope.
//!
//! Separate from the decoded model so the shape the artifact is WRITTEN in and the shape the
//! engine REASONS over can diverge without either pretending to be the other.

use std::collections::BTreeMap;

use serde::Deserialize;

/// Wire shape of one type-tree node.
#[derive(Deserialize)]
pub(crate) struct TypeEntry {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) package: String,
    #[serde(default)]
    pub(crate) args: Vec<TypeEntry>,
}

#[derive(Deserialize)]
pub(crate) struct SnapshotDocument {
    /// Absent in v0 artifacts, which predate the field.
    #[serde(default)]
    pub(crate) schema_version: u32,
    pub(crate) language: String,
    /// The configuration the corpus was type-checked FOR, canonicalised.
    ///
    /// An input that changes what is extracted, and was the one input that changed nothing
    /// observable: two extractions at Go 1.21 and Go 1.24 produced identical digests, while Go 1.22
    /// rescoped the loop variable. Defaulted so a snapshot written before this field still admits.
    #[serde(default)]
    pub(crate) build_config: String,
    pub(crate) snapshot_digest: String,
    pub(crate) packages: Vec<PackageEntry>,
}

#[derive(Deserialize)]
pub(crate) struct PackageEntry {
    pub(crate) unit_id: String,
    pub(crate) producer: String,
    #[serde(default)]
    pub(crate) declarations: Vec<DeclarationEntry>,
}

/// Wire shape of one declaration node. Recursive and uniform, matching the extractor.
#[derive(Deserialize)]
pub(crate) struct DeclarationEntry {
    pub(crate) kind: String,
    #[serde(default)]
    pub(crate) name: String,
    #[serde(default, rename = "type")]
    pub(crate) type_ref: Option<TypeEntry>,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
    #[serde(default)]
    pub(crate) attrs: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) children: Vec<DeclarationEntry>,
}
