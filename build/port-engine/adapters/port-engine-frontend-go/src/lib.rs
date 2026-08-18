//! # port-engine-frontend-go — Go SourceModel snapshot consumer (W0-B Slice 4).
//!
//! ADR-0638 D3 snapshot firewall: this adapter consumes **SourceModel snapshot bytes only** and
//! must never invoke a Go toolchain in-process or from the `verify()` path. Slice 4 lands decode
//! of the canonical snapshot envelope plus an architecture test that refuses spawning the `go`
//! binary from library sources used by verify.
#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fmt;

use port_engine_api::{Declaration, Digest, UnitId};
use serde::Deserialize;

/// Canonical bootstrap extractor identity (ADR-0638 D3).
pub const PRODUCER_BOOTSTRAP_GO: &str = "bootstrap-go-packages-go-types";

/// Owned Rust front-end producer identity (authorized only after W2 equivalence).
pub const PRODUCER_OWNED_RUST: &str = "owned-rust-go-front-end";

/// Envelope version carrying unit identity only.
pub const SCHEMA_VERSION_IDENTITY_ONLY: u32 = 0;

/// Envelope version carrying the declaration tree.
pub const SCHEMA_VERSION_DECLARATIONS: u32 = 1;

/// Declaration kinds this Go adapter admits, at package scope.
///
/// CLOSED, and the closure lives here rather than in `port-engine-api` on purpose. The neutral
/// seam treats `kind` as an opaque slug because a second language pair must not need a second
/// seam. This adapter is the Go half, so this is exactly where Go's declaration taxonomy is
/// allowed to be named — and where an extractor that emits a kind the engine has never heard of
/// gets refused instead of translated into silence.
pub const KNOWN_DECLARATION_KINDS: &[&str] = &[
    "alias",
    "const",
    "func",
    "interface",
    "named",
    "struct",
    "var",
];

/// Declaration kinds admitted below package scope, as children of a declaration.
pub const KNOWN_MEMBER_KINDS: &[&str] = &["field", "method", "param", "result"];

/// The closed flag vocabulary. Same argument as [`KNOWN_DECLARATION_KINDS`]: a flag the engine
/// does not know is a flag nothing will ever select on, and accepting it would let a misspelled
/// `exported` silently unexport a declaration.
pub const KNOWN_FLAGS: &[&str] = &["embedded", "exported", "variadic"];

/// Fail-closed readiness gate. `true` once Slice 4 snapshot decode is present.
pub const fn w0_ready() -> bool {
    true
}

/// Typed refusal from snapshot decode / producer validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SnapshotError {
    /// JSON could not be parsed.
    Parse {
        /// Parser detail (no path — adapter receives bytes only).
        detail: String,
    },
    /// Required field missing or wrong type / empty.
    Schema {
        /// Which field failed.
        field: &'static str,
    },
    /// Package producer is not one of the ADR-0638 canonical identities.
    UnknownProducer {
        /// Producer string found on a package.
        actual: String,
    },
    /// Duplicate `unit_id` — non-deterministic model shape.
    DuplicateUnit {
        /// The repeated unit id.
        unit_id: String,
    },
    /// Envelope claims a schema version this decoder does not implement.
    UnknownSchemaVersion {
        /// Version claimed by the artifact.
        actual: u32,
    },
    /// Declaration kind is outside the closed Go vocabulary.
    UnknownDeclarationKind {
        /// Unit the declaration belongs to.
        unit_id: String,
        /// Kind string found.
        actual: String,
    },
    /// Flag is outside the closed flag vocabulary.
    UnknownFlag {
        /// Unit the declaration belongs to.
        unit_id: String,
        /// Flag string found.
        actual: String,
    },
    /// Two declarations share one name in a scope that has a single namespace.
    DuplicateDeclaration {
        /// Unit the declarations belong to.
        unit_id: String,
        /// The repeated name.
        name: String,
    },
    /// The envelope version and its payload disagree.
    VersionPayloadMismatch {
        /// What the version claims.
        detail: &'static str,
    },
}

impl fmt::Display for SnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse { detail } => {
                write!(f, "source-model snapshot JSON parse failed: {detail}")
            }
            Self::Schema { field } => {
                write!(
                    f,
                    "source-model snapshot schema missing or invalid: {field}"
                )
            }
            Self::UnknownProducer { actual } => write!(
                f,
                "source-model snapshot package producer must be `{PRODUCER_BOOTSTRAP_GO}` or `{PRODUCER_OWNED_RUST}`, got `{actual}`"
            ),
            Self::DuplicateUnit { unit_id } => {
                write!(f, "source-model snapshot has duplicate unit_id `{unit_id}`")
            }
            Self::UnknownSchemaVersion { actual } => write!(
                f,
                "source-model snapshot schema_version must be {SCHEMA_VERSION_IDENTITY_ONLY} or \
                 {SCHEMA_VERSION_DECLARATIONS}, got {actual}"
            ),
            Self::UnknownDeclarationKind { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` declares unknown kind `{actual}`"
            ),
            Self::UnknownFlag { unit_id, actual } => write!(
                f,
                "source-model snapshot unit `{unit_id}` carries unknown flag `{actual}`"
            ),
            Self::DuplicateDeclaration { unit_id, name } => write!(
                f,
                "source-model snapshot unit `{unit_id}` declares `{name}` more than once in one \
                 namespace"
            ),
            Self::VersionPayloadMismatch { detail } => {
                write!(
                    f,
                    "source-model snapshot version/payload mismatch: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for SnapshotError {}

#[derive(Deserialize)]
struct SnapshotDocument {
    /// Absent in v0 artifacts, which predate the field.
    #[serde(default)]
    schema_version: u32,
    language: String,
    snapshot_digest: String,
    packages: Vec<PackageEntry>,
}

#[derive(Deserialize)]
struct PackageEntry {
    unit_id: String,
    producer: String,
    #[serde(default)]
    declarations: Vec<DeclarationEntry>,
}

/// Wire shape of one declaration node. Recursive and uniform, matching the extractor.
#[derive(Deserialize)]
struct DeclarationEntry {
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default, rename = "type")]
    type_ref: String,
    #[serde(default)]
    flags: Vec<String>,
    #[serde(default)]
    children: Vec<DeclarationEntry>,
}

/// Decoded Go SourceModel snapshot (no Go toolchain; artifact bytes only).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoSourceModel {
    schema_version: u32,
    language: String,
    snapshot_digest: Digest,
    units: Vec<UnitId>,
    /// Parallel to [`Self::units`] — ADR-0638 D3 package→producer map (one producer per package).
    producers: Vec<String>,
    /// Parallel to [`Self::units`] — declaration tree per package. Empty for a v0 artifact.
    declarations: Vec<Vec<Declaration>>,
}

impl GoSourceModel {
    /// Decode snapshot JSON bytes into an unadmitted Go model.
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode(bytes: &[u8]) -> Result<Self, SnapshotError> {
        let text = std::str::from_utf8(bytes).map_err(|err| SnapshotError::Parse {
            detail: format!("utf-8: {err}"),
        })?;
        Self::decode_str(text)
    }

    /// Decode snapshot JSON from an in-memory string (test hook and future adapter input).
    ///
    /// # Errors
    /// [`SnapshotError`] on parse failure, schema violation, unknown producer, or duplicate unit.
    pub fn decode_str(json: &str) -> Result<Self, SnapshotError> {
        let doc: SnapshotDocument =
            serde_json::from_str(json).map_err(|err| SnapshotError::Parse {
                detail: err.to_string(),
            })?;
        if doc.language.is_empty() {
            return Err(SnapshotError::Schema { field: "language" });
        }
        if doc.snapshot_digest.is_empty() {
            return Err(SnapshotError::Schema {
                field: "snapshot_digest",
            });
        }
        if doc.schema_version != SCHEMA_VERSION_IDENTITY_ONLY
            && doc.schema_version != SCHEMA_VERSION_DECLARATIONS
        {
            return Err(SnapshotError::UnknownSchemaVersion {
                actual: doc.schema_version,
            });
        }

        let mut units = Vec::with_capacity(doc.packages.len());
        let mut producers = Vec::with_capacity(doc.packages.len());
        let mut declarations = Vec::with_capacity(doc.packages.len());
        let mut seen = BTreeSet::new();
        for pkg in doc.packages {
            if pkg.unit_id.is_empty() || pkg.unit_id.contains('\0') {
                return Err(SnapshotError::Schema {
                    field: "packages.unit_id",
                });
            }
            if pkg.producer != PRODUCER_BOOTSTRAP_GO && pkg.producer != PRODUCER_OWNED_RUST {
                return Err(SnapshotError::UnknownProducer {
                    actual: pkg.producer,
                });
            }
            if !seen.insert(pkg.unit_id.clone()) {
                return Err(SnapshotError::DuplicateUnit {
                    unit_id: pkg.unit_id,
                });
            }
            // A v0 artifact carrying declarations is a version lie, not a bonus. Accepting it
            // would mean the version field says one thing about the payload while the payload
            // says another, and every later reader has to guess which one to believe.
            if doc.schema_version == SCHEMA_VERSION_IDENTITY_ONLY && !pkg.declarations.is_empty() {
                return Err(SnapshotError::VersionPayloadMismatch {
                    detail: "schema_version 0 carries declarations",
                });
            }
            declarations.push(convert_declarations(&pkg.unit_id, &pkg.declarations)?);
            units.push(UnitId(pkg.unit_id));
            producers.push(pkg.producer);
        }
        Ok(Self {
            schema_version: doc.schema_version,
            language: doc.language,
            snapshot_digest: Digest(doc.snapshot_digest),
            units,
            producers,
            declarations,
        })
    }

    /// Source-language slug claimed by the decoded snapshot.
    #[must_use]
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Semantic digest claimed by the decoded snapshot.
    #[must_use]
    pub fn snapshot_digest(&self) -> Digest {
        self.snapshot_digest.clone()
    }

    /// Units in decoded snapshot order.
    #[must_use]
    pub fn units(&self) -> Vec<UnitId> {
        self.units.clone()
    }

    /// Envelope version this model was decoded from.
    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Producer identity for `unit`, if present in the snapshot map.
    #[must_use]
    pub fn producer_for(&self, unit: &UnitId) -> Option<&str> {
        self.units
            .iter()
            .position(|u| u == unit)
            .map(|idx| producers_at(self, idx))
    }

    /// Declaration tree for `unit`, or `None` when the model does not carry that unit.
    #[must_use]
    pub fn declarations_for(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.units
            .iter()
            .position(|u| u == unit)
            .map(|idx| self.declarations[idx].clone())
    }
}

fn producers_at(model: &GoSourceModel, idx: usize) -> &str {
    &model.producers[idx]
}

/// Validate and convert one package's declaration nodes.
///
/// Go gives every package-scope identifier a single shared namespace — a `const` and a `func` in
/// one package cannot both be called `Add` — so a repeated name at this level is not a stylistic
/// smell, it is proof the extractor lost information. Below package scope the same rule holds per
/// parent, with one exception: an unnamed or blank identifier repeats legitimately, because
/// `func(int, int) int` really does declare two nameless parameters.
fn convert_declarations(
    unit_id: &str,
    entries: &[DeclarationEntry],
) -> Result<Vec<Declaration>, SnapshotError> {
    convert_level(unit_id, entries, KNOWN_DECLARATION_KINDS)
}

fn convert_level(
    unit_id: &str,
    entries: &[DeclarationEntry],
    allowed_kinds: &[&str],
) -> Result<Vec<Declaration>, SnapshotError> {
    let mut named = BTreeSet::new();
    let mut out = Vec::with_capacity(entries.len());

    for entry in entries {
        if !allowed_kinds.contains(&entry.kind.as_str()) {
            return Err(SnapshotError::UnknownDeclarationKind {
                unit_id: unit_id.to_owned(),
                actual: entry.kind.clone(),
            });
        }
        if entry.name.contains('\0') || entry.type_ref.contains('\0') {
            return Err(SnapshotError::Schema {
                field: "packages.declarations",
            });
        }
        if !entry.name.is_empty() && entry.name != "_" && !named.insert(entry.name.clone()) {
            return Err(SnapshotError::DuplicateDeclaration {
                unit_id: unit_id.to_owned(),
                name: entry.name.clone(),
            });
        }

        let mut flags = BTreeSet::new();
        for flag in &entry.flags {
            if !KNOWN_FLAGS.contains(&flag.as_str()) {
                return Err(SnapshotError::UnknownFlag {
                    unit_id: unit_id.to_owned(),
                    actual: flag.clone(),
                });
            }
            flags.insert(flag.clone());
        }

        out.push(Declaration {
            kind: entry.kind.clone(),
            name: entry.name.clone(),
            type_ref: entry.type_ref.clone(),
            flags,
            children: convert_level(unit_id, &entry.children, KNOWN_MEMBER_KINDS)?,
        });
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"{
  "language": "go",
  "snapshot_digest": "sha256:fixture-slice4",
  "packages": [
    {"unit_id": "example.com/a", "producer": "bootstrap-go-packages-go-types"},
    {"unit_id": "example.com/b", "producer": "bootstrap-go-packages-go-types"}
  ]
}"#;

    #[test]
    fn slice4_claims_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn decodes_ordered_units_and_producers() {
        let model = GoSourceModel::decode_str(FIXTURE).expect("fixture must decode");
        assert_eq!(model.language(), "go");
        assert_eq!(
            model.snapshot_digest(),
            Digest("sha256:fixture-slice4".into())
        );
        assert_eq!(
            model.units(),
            vec![
                UnitId("example.com/a".into()),
                UnitId("example.com/b".into())
            ]
        );
        assert_eq!(
            model.producer_for(&UnitId("example.com/a".into())),
            Some(PRODUCER_BOOTSTRAP_GO)
        );
    }

    #[test]
    fn refuses_unknown_producer() {
        let json = r#"{"language":"go","snapshot_digest":"d","packages":[{"unit_id":"x","producer":"gccgo"}]}"#;
        let err = GoSourceModel::decode_str(json).expect_err("unknown producer must refuse");
        assert!(matches!(err, SnapshotError::UnknownProducer { .. }));
    }

    #[test]
    fn refuses_duplicate_unit() {
        let json = r#"{"language":"go","snapshot_digest":"d","packages":[{"unit_id":"x","producer":"bootstrap-go-packages-go-types"},{"unit_id":"x","producer":"bootstrap-go-packages-go-types"}]}"#;
        let err = GoSourceModel::decode_str(json).expect_err("duplicate unit must refuse");
        assert!(matches!(err, SnapshotError::DuplicateUnit { .. }));
    }

    #[test]
    fn refuses_nul_in_unit_identity() {
        let json = r#"{"language":"go","snapshot_digest":"d","packages":[{"unit_id":"x\u0000y","producer":"bootstrap-go-packages-go-types"}]}"#;
        let err = GoSourceModel::decode_str(json)
            .expect_err("NUL would make the semantic snapshot preimage ambiguous");
        assert_eq!(
            err,
            SnapshotError::Schema {
                field: "packages.unit_id",
            }
        );
    }

    const V1: &str = r#"{
  "schema_version": 1,
  "language": "go",
  "snapshot_digest": "sha256:fixture-v1",
  "packages": [
    {
      "unit_id": "example.com/a",
      "producer": "bootstrap-go-packages-go-types",
      "declarations": [
        {"kind": "const", "name": "Max", "type": "int", "flags": ["exported"]},
        {"kind": "func", "name": "Add", "flags": ["exported"], "children": [
          {"kind": "param", "name": "a", "type": "int"},
          {"kind": "param", "name": "b", "type": "int"},
          {"kind": "result", "name": "", "type": "int"}
        ]}
      ]
    }
  ]
}"#;

    fn with_declarations(body: &str) -> String {
        format!(
            r#"{{"schema_version":1,"language":"go","snapshot_digest":"d","packages":[{{"unit_id":"x","producer":"{PRODUCER_BOOTSTRAP_GO}","declarations":[{body}]}}]}}"#
        )
    }

    #[test]
    fn decodes_v1_declaration_tree() {
        let model = GoSourceModel::decode_str(V1).expect("v1 fixture must decode");
        assert_eq!(model.schema_version(), SCHEMA_VERSION_DECLARATIONS);

        let declarations = model
            .declarations_for(&UnitId("example.com/a".into()))
            .expect("unit is present");
        assert_eq!(declarations.len(), 2);

        let add = &declarations[1];
        assert_eq!(add.kind, "func");
        assert!(add.has_flag("exported"));
        assert_eq!(add.children.len(), 3);
        assert_eq!(add.children_of_kind("param").len(), 2);
        assert_eq!(add.children_of_kind("result")[0].type_ref, "int");
    }

    #[test]
    fn v0_artifact_still_decodes_and_declares_nothing() {
        let model = GoSourceModel::decode_str(FIXTURE).expect("v0 fixture must still decode");
        assert_eq!(model.schema_version(), SCHEMA_VERSION_IDENTITY_ONLY);
        assert_eq!(
            model.declarations_for(&UnitId("example.com/a".into())),
            Some(Vec::new())
        );
    }

    #[test]
    fn refuses_unknown_schema_version() {
        let json = r#"{"schema_version":2,"language":"go","snapshot_digest":"d","packages":[]}"#;
        let err = GoSourceModel::decode_str(json).expect_err("a future version must refuse");
        assert_eq!(err, SnapshotError::UnknownSchemaVersion { actual: 2 });
    }

    /// A v0 envelope carrying declarations is a version lie: the field says the payload has no
    /// declarations while the payload has them. Accepting it would leave every later reader
    /// guessing which of the two to believe, and the digest rule is selected by version.
    #[test]
    fn refuses_v0_envelope_carrying_declarations() {
        let json = format!(
            r#"{{"language":"go","snapshot_digest":"d","packages":[{{"unit_id":"x","producer":"{PRODUCER_BOOTSTRAP_GO}","declarations":[{{"kind":"const","name":"K"}}]}}]}}"#
        );
        let err = GoSourceModel::decode_str(&json).expect_err("version/payload lie must refuse");
        assert!(matches!(err, SnapshotError::VersionPayloadMismatch { .. }));
    }

    #[test]
    fn refuses_declaration_kind_outside_the_closed_vocabulary() {
        let json = with_declarations(r#"{"kind":"goroutine","name":"g"}"#);
        let err = GoSourceModel::decode_str(&json).expect_err("unknown kind must refuse");
        assert!(matches!(err, SnapshotError::UnknownDeclarationKind { .. }));
    }

    /// A member kind at package scope, or a package-scope kind nested inside a declaration, is a
    /// structural error and not merely an unusual shape — `param` is not a thing a package
    /// declares, and `struct` is not a thing a parameter list contains.
    #[test]
    fn refuses_member_kind_at_package_scope() {
        let json = with_declarations(r#"{"kind":"param","name":"a","type":"int"}"#);
        let err = GoSourceModel::decode_str(&json).expect_err("member kind at top level refuses");
        assert!(matches!(err, SnapshotError::UnknownDeclarationKind { .. }));
    }

    #[test]
    fn refuses_package_scope_kind_nested_as_a_member() {
        let json = with_declarations(
            r#"{"kind":"func","name":"f","children":[{"kind":"const","name":"K"}]}"#,
        );
        let err = GoSourceModel::decode_str(&json).expect_err("nested package kind refuses");
        assert!(matches!(err, SnapshotError::UnknownDeclarationKind { .. }));
    }

    #[test]
    fn refuses_flag_outside_the_closed_vocabulary() {
        let json = with_declarations(r#"{"kind":"const","name":"K","flags":["exportd"]}"#);
        let err = GoSourceModel::decode_str(&json).expect_err("misspelled flag must refuse");
        assert_eq!(
            err,
            SnapshotError::UnknownFlag {
                unit_id: "x".into(),
                actual: "exportd".into(),
            },
            "a silently dropped `exported` would unexport a declaration with no diagnostic"
        );
    }

    /// Go gives every package-scope identifier one namespace, so a repeat is proof the extractor
    /// lost information rather than a naming choice.
    #[test]
    fn refuses_duplicate_declaration_name_in_one_namespace() {
        let json = with_declarations(
            r#"{"kind":"const","name":"K","type":"int"},{"kind":"func","name":"K"}"#,
        );
        let err = GoSourceModel::decode_str(&json).expect_err("duplicate name must refuse");
        assert!(matches!(err, SnapshotError::DuplicateDeclaration { .. }));
    }

    /// The exception that keeps the rule usable: `func(int, int) int` really does declare two
    /// nameless parameters, so blank and empty names may repeat.
    #[test]
    fn admits_repeated_blank_member_names() {
        let json = with_declarations(
            r#"{"kind":"func","name":"f","children":[{"kind":"param","name":"","type":"int"},{"kind":"param","name":"","type":"int"},{"kind":"param","name":"_","type":"int"},{"kind":"param","name":"_","type":"int"}]}"#,
        );
        let model = GoSourceModel::decode_str(&json).expect("unnamed parameters are legal Go");
        let decls = model
            .declarations_for(&UnitId("x".into()))
            .expect("unit present");
        assert_eq!(decls[0].children.len(), 4);
    }

    /// ADR-0638 D3 architecture fence: library sources used by verify must not spawn `go`.
    #[test]
    fn library_source_never_spawns_go_command() {
        let src = include_str!("lib.rs");
        // Build needles without embedding the forbidden call site as a contiguous literal in
        // production code paths (this test body is the only place that may mention the pattern).
        let cmd_new = ["Command", "::", "new"].concat();
        let go_lit = ["\"", "go", "\""].concat();
        let forbidden_call = format!("{cmd_new}({go_lit})");
        let process_cmd = ["std", "::", "process", "::", "Command"].concat();
        // Strip this #[cfg(test)] module so the assertion text does not self-fail.
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("lib.rs must have a production section");
        assert!(
            !production.contains(&forbidden_call),
            "port-engine-frontend-go production sources must not invoke Go via {forbidden_call}"
        );
        assert!(
            !production.contains(&process_cmd),
            "port-engine-frontend-go production sources must not import {process_cmd} (Go firewall)"
        );
    }

    /// ADR-0638 D3, second half: the firewall is not only "do not spawn `go`", it is "do not
    /// READ Go". The bootstrap corpus and extractor live under `gosrc/` beside this crate, and
    /// a library source that named that tree — to `include_str!` a `.go` file, to walk the
    /// corpus, to re-derive anything the snapshot already carries — would make the engine's
    /// answer depend on Go source at verify time even though no toolchain ever ran. That is the
    /// same defect the process fence exists to prevent, arriving through the filesystem instead
    /// of through a subprocess.
    #[test]
    fn library_source_never_reads_the_go_corpus() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("lib.rs must have a production section");

        let corpus_tree = ["go", "src/"].concat();
        let go_extension = [".", "go", "\""].concat();
        assert!(
            !production.contains(&corpus_tree),
            "port-engine-frontend-go production sources must not name the `{corpus_tree}` \
             out-of-band tree — the engine consumes snapshot artifacts, never Go source"
        );
        assert!(
            !production.contains(&go_extension),
            "port-engine-frontend-go production sources must not reference a `{go_extension}` path"
        );
    }
}
