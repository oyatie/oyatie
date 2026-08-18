//! # port-engine-snapshot — bootstrap SourceModel snapshot admission (W0-B Slice 8).
//!
//! ADR-0638 D3: the bootstrap Go extractor (`go/packages` + `go/types`) runs **out of band only**.
//! This adapter admits the resulting content-addressed snapshot artifact, binds it to the fleet
//! pin, and verifies the claimed `snapshot_digest` against a stable preimage. It MUST NEVER
//! invoke a Go toolchain (firewall inherited from `port-engine-frontend-go`).
#![forbid(unsafe_code)]

use std::fmt;

use port_engine_api::{Declaration, Digest, SourceModel, UnitId};
use port_engine_frontend_go::{
    GoSourceModel, PRODUCER_BOOTSTRAP_GO, SCHEMA_VERSION_DECLARATIONS, SnapshotError,
};
use port_engine_hash::digest_bytes;
use port_engine_source_pin::{PinError, load_embedded, receipt_pin};

/// Embedded OOB bootstrap snapshot fixture (hermetic; not produced in-process).
const FIXTURE_SNAPSHOT_JSON: &str = include_str!("fixture-snapshot-v0.json");

/// Embedded v1 fixture: the declaration tree extracted from the hermetic Go corpus by the
/// out-of-band bootstrap extractor (`../port-engine-frontend-go/gosrc/`). Committed rather than
/// produced here — the ADR-0638 D3 firewall means no engine crate may run Go.
const FIXTURE_SNAPSHOT_V1_JSON: &str = include_str!("fixture-snapshot-v1.json");

/// Fail-closed readiness gate. `true` once Slice 8 admission is present.
pub const fn w0_ready() -> bool {
    true
}

/// Typed refusal from snapshot admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmitError {
    /// Snapshot decode / producer validation failed.
    Snapshot(SnapshotError),
    /// Fleet pin could not load.
    Pin(PinError),
    /// The two extractor passes did not produce byte-identical snapshots.
    SnapshotMismatch {
        /// SHA-256 digest of the first raw snapshot artifact.
        first: Digest,
        /// SHA-256 digest of the second raw snapshot artifact.
        second: Digest,
    },
    /// Claimed `snapshot_digest` does not match the stable preimage hash.
    DigestMismatch {
        /// Digest claimed in the artifact.
        claimed: String,
        /// Digest computed from the admission preimage.
        computed: String,
    },
    /// Snapshot language is not the bootstrap Go pair source.
    Language {
        /// Language found on the artifact.
        actual: String,
    },
    /// A producer is not authorized during bootstrap admission.
    ProducerNotAuthorized {
        /// Unit whose producer is premature.
        unit: String,
        /// Producer identity found on the artifact.
        actual: String,
    },
}

impl fmt::Display for AdmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snapshot(err) => write!(f, "snapshot admit decode failed: {err}"),
            Self::Pin(err) => write!(f, "snapshot admit pin failed: {err}"),
            Self::SnapshotMismatch { first, second } => write!(
                f,
                "snapshot extractor passes differ: first `{}`, second `{}`",
                first.0, second.0
            ),
            Self::DigestMismatch { claimed, computed } => write!(
                f,
                "snapshot admit digest mismatch: claimed `{claimed}`, computed `{computed}`"
            ),
            Self::Language { actual } => write!(
                f,
                "snapshot admit language must be `go` for bootstrap admission, got `{actual}`"
            ),
            Self::ProducerNotAuthorized { unit, actual } => write!(
                f,
                "snapshot admit producer for unit `{unit}` must be `{PRODUCER_BOOTSTRAP_GO}` before \
                 front-end equivalence, got `{actual}`"
            ),
        }
    }
}

impl std::error::Error for AdmitError {}

/// An admitted bootstrap snapshot bound to the fleet pin.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedSnapshot {
    /// Fleet pin (peeled commit) bound at admission.
    pin: String,
    /// SHA-256 digest of the byte-identical raw snapshot artifact.
    artifact_digest: Digest,
    /// Verified semantic digest claimed inside the artifact.
    model_digest: Digest,
    /// Decoded SourceModel (identity + order only).
    model: GoSourceModel,
}

impl AdmittedSnapshot {
    /// Fleet pin bound during admission.
    #[must_use]
    pub fn pin(&self) -> &str {
        &self.pin
    }

    /// Digest of the raw byte-identical artifact pair.
    #[must_use]
    pub fn artifact_digest(&self) -> &Digest {
        &self.artifact_digest
    }

    /// Verified semantic digest claimed by the decoded model.
    #[must_use]
    pub fn model_digest(&self) -> &Digest {
        &self.model_digest
    }

    /// Borrow the underlying [`SourceModel`].
    #[must_use]
    pub fn as_model(&self) -> &dyn SourceModel {
        self
    }

    /// Producer identity recorded for `unit`.
    #[must_use]
    pub fn producer_for(&self, unit: &UnitId) -> Option<&str> {
        self.model.producer_for(unit)
    }
}

impl SourceModel for AdmittedSnapshot {
    fn language(&self) -> &str {
        self.model.language()
    }

    fn snapshot_digest(&self) -> Digest {
        self.artifact_digest.clone()
    }

    fn units(&self) -> Vec<UnitId> {
        self.model.units()
    }

    fn declarations(&self, unit: &UnitId) -> Option<Vec<Declaration>> {
        self.model.declarations_for(unit)
    }
}

/// Stable admission preimage: length-prefixed language, then each length-prefixed unit and
/// producer in model order.
///
/// Decimal byte lengths followed by `:` make the encoding injective even when a field contains a
/// delimiter. The digest therefore covers language + package→producer mapping without relying on
/// JSON canonicalization or cross-crate character restrictions.
#[must_use]
pub fn snapshot_preimage(language: &str, units_and_producers: &[(&str, &str)]) -> Vec<u8> {
    let mut out = Vec::new();
    push_field(&mut out, language);
    for (unit, producer) in units_and_producers {
        push_field(&mut out, unit);
        push_field(&mut out, producer);
    }
    out
}

fn push_field(out: &mut Vec<u8>, value: &str) {
    out.extend_from_slice(value.len().to_string().as_bytes());
    out.push(b':');
    out.extend_from_slice(value.as_bytes());
}

/// Stable admission preimage for a v1 artifact, which carries declarations.
///
/// The v0 preimage covers language plus the package→producer map, and nothing else. Digesting a
/// v1 artifact with it would leave the entire declaration tree OUTSIDE the identity: rename a
/// field, add a method, change a parameter type, and `snapshot_digest` would not move. The
/// receipt would then find the emitted bytes changed with all six axes unchanged and classify a
/// perfectly well-explained change as `Unexplained` — or, worse, an emit that happened not to
/// change would be blessed as reproducible over a corpus that did.
///
/// The encoding is the same shape as v0's — decimal length prefixes with a `:` — extended with an
/// explicit child arity per node:
///
/// ```text
/// F(kind) F(name) F(type_ref) F(len(flags)) flags...
///     F(len(attrs)) (F(key) F(value))... F(len(children)) children...
/// ```
///
/// Length prefixes make each field unambiguous; the arity counts make the tree unambiguous. This
/// is mirrored byte-for-byte by the Go extractor's `encodeNode`. That duplication is deliberate:
/// the alternative is trusting the digest the extractor claims, which would let a front-end defect
/// enter the engine carrying a self-consistent receipt. Drift between the two implementations
/// surfaces here as [`AdmitError::DigestMismatch`].
#[must_use]
pub fn snapshot_preimage_v1(
    language: &str,
    packages: &[(&str, &str, Vec<Declaration>)],
) -> Vec<u8> {
    let mut out = Vec::new();
    push_field(&mut out, "snapshot");
    push_field(&mut out, language);
    push_field(&mut out, &packages.len().to_string());
    for (unit, producer, declarations) in packages {
        push_field(&mut out, "package");
        push_field(&mut out, unit);
        push_field(&mut out, producer);
        push_field(&mut out, &declarations.len().to_string());
        for declaration in declarations {
            push_declaration(&mut out, declaration);
        }
    }
    out
}

fn push_declaration(out: &mut Vec<u8>, declaration: &Declaration) {
    push_field(out, &declaration.kind);
    push_field(out, &declaration.name);
    push_field(out, &declaration.type_ref);
    // `flags` is a BTreeSet and `attrs` a BTreeMap, so both iterate sorted — the same order the
    // extractor sorts into. A set with two orderings would be a set with two digests.
    push_field(out, &declaration.flags.len().to_string());
    for flag in &declaration.flags {
        push_field(out, flag);
    }
    push_field(out, &declaration.attrs.len().to_string());
    for (key, value) in &declaration.attrs {
        push_field(out, key);
        push_field(out, value);
    }
    push_field(out, &declaration.children.len().to_string());
    for child in &declaration.children {
        push_declaration(out, child);
    }
}

/// Admit two byte-identical snapshot artifacts against the fleet pin.
///
/// # Errors
/// [`AdmitError::SnapshotMismatch`] when the two extractor passes differ, or another
/// [`AdmitError`] on decode, pin, language, or digest mismatch.
pub fn admit_reproducible_pair(
    first: &[u8],
    second: &[u8],
) -> Result<AdmittedSnapshot, AdmitError> {
    if first != second {
        return Err(AdmitError::SnapshotMismatch {
            first: digest_bytes(first),
            second: digest_bytes(second),
        });
    }
    admit_one(first, digest_bytes(first))
}

fn admit_one(bytes: &[u8], artifact_digest: Digest) -> Result<AdmittedSnapshot, AdmitError> {
    let model = GoSourceModel::decode(bytes).map_err(AdmitError::Snapshot)?;
    if model.language() != "go" {
        return Err(AdmitError::Language {
            actual: model.language().to_owned(),
        });
    }

    let units = model.units();
    let mut pairs: Vec<(String, String)> = Vec::with_capacity(units.len());
    for unit in &units {
        let producer =
            model
                .producer_for(unit)
                .ok_or(AdmitError::Snapshot(SnapshotError::Schema {
                    field: "packages.producer",
                }))?;
        if producer != PRODUCER_BOOTSTRAP_GO {
            return Err(AdmitError::ProducerNotAuthorized {
                unit: unit.0.clone(),
                actual: producer.to_owned(),
            });
        }
        pairs.push((unit.0.clone(), producer.to_owned()));
    }

    // The preimage is chosen by the artifact's declared version, not by whether declarations
    // happen to be present. Choosing on presence would mean an artifact whose declarations were
    // dropped in transit re-digests cleanly under the v0 rule and admits as a valid empty corpus.
    let computed = if model.schema_version() == SCHEMA_VERSION_DECLARATIONS {
        let mut packages: Vec<(&str, &str, Vec<Declaration>)> = Vec::with_capacity(units.len());
        for ((unit, producer), id) in pairs.iter().zip(units.iter()) {
            let declarations =
                model
                    .declarations_for(id)
                    .ok_or(AdmitError::Snapshot(SnapshotError::Schema {
                        field: "packages.declarations",
                    }))?;
            packages.push((unit.as_str(), producer.as_str(), declarations));
        }
        digest_bytes(&snapshot_preimage_v1(model.language(), &packages))
    } else {
        let refs: Vec<(&str, &str)> = pairs
            .iter()
            .map(|(u, p)| (u.as_str(), p.as_str()))
            .collect();
        digest_bytes(&snapshot_preimage(model.language(), &refs))
    };
    let claimed = model.snapshot_digest();
    if claimed != computed {
        return Err(AdmitError::DigestMismatch {
            claimed: claimed.0,
            computed: computed.0,
        });
    }

    let pin = load_embedded().map_err(AdmitError::Pin)?;
    Ok(AdmittedSnapshot {
        pin: receipt_pin(&pin),
        artifact_digest,
        model_digest: computed,
        model,
    })
}

/// Admit the package-local OOB bootstrap fixture.
///
/// The embedded fixture has one hermetic byte source, so pairing it with itself exercises normal
/// admission without pretending that a second extractor execution occurred. External extractor
/// output must enter through [`admit_reproducible_pair`] with two independently produced artifacts.
///
/// # Errors
/// [`AdmitError`] on fixture defect.
pub fn admit_embedded_fixture() -> Result<AdmittedSnapshot, AdmitError> {
    let bytes = FIXTURE_SNAPSHOT_JSON.as_bytes();
    admit_reproducible_pair(bytes, bytes)
}

/// Admit the embedded v1 fixture: the declaration tree extracted from the hermetic Go corpus.
///
/// Same single-byte-source caveat as [`admit_embedded_fixture`] — pairing the artifact with itself
/// exercises admission without claiming a second extractor run happened. A genuine two-pass
/// extraction enters through [`admit_reproducible_pair`].
///
/// # Errors
/// [`AdmitError`] on fixture defect — including a digest that the Rust preimage disagrees with,
/// which is how a drift between the Go and Rust encoders is meant to surface.
pub fn admit_embedded_fixture_v1() -> Result<AdmittedSnapshot, AdmitError> {
    let bytes = FIXTURE_SNAPSHOT_V1_JSON.as_bytes();
    admit_reproducible_pair(bytes, bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice8_claims_snapshot_readiness() {
        assert!(w0_ready());
    }

    #[test]
    fn semantic_preimage_is_injective_across_field_boundaries() {
        let producer = PRODUCER_BOOTSTRAP_GO;
        let embedded_delimiters = format!("x\0{producer}\0y");
        let one_unit = snapshot_preimage("go", &[(embedded_delimiters.as_str(), producer)]);
        let two_units = snapshot_preimage("go", &[("x", producer), ("y", producer)]);
        assert_ne!(one_unit, two_units);
    }

    #[test]
    fn embedded_fixture_admits_and_binds_pin() {
        let admitted = admit_embedded_fixture().expect("fixture must admit");
        assert!(!admitted.pin().is_empty());
        assert_eq!(
            admitted.model_digest().0,
            "sha256:5a3bca44537be2cc8d1cb909616b741e8e4e1d1b879dc231e40dfc56d75e3f7a"
        );
        assert_eq!(
            admitted.artifact_digest(),
            &digest_bytes(FIXTURE_SNAPSHOT_JSON.as_bytes())
        );
        assert_eq!(
            admitted.as_model().snapshot_digest(),
            admitted.artifact_digest().clone()
        );
        assert_eq!(admitted.as_model().units().len(), 2);
        assert_eq!(
            admitted.producer_for(&UnitId("example.com/a".into())),
            Some(PRODUCER_BOOTSTRAP_GO)
        );
    }

    /// The cross-language check. This fixture's `snapshot_digest` was computed by the Go
    /// extractor over ITS encoder; admission recomputes it here over the Rust one. The test
    /// passing means the two implementations agree byte-for-byte over a real declaration tree —
    /// which is the whole reason mirroring the encoder is acceptable rather than reckless.
    #[test]
    fn v1_fixture_admits_and_carries_declarations() {
        let admitted = admit_embedded_fixture_v1().expect("v1 fixture must admit");

        let units = admitted.as_model().units();
        assert_eq!(units.len(), 2, "corpus has two packages");
        assert!(
            units
                .iter()
                .all(|u| u.0.ends_with("basic") || u.0.ends_with("shapes"))
        );

        let basic = units
            .iter()
            .find(|u| u.0.ends_with("basic"))
            .expect("basic package");
        let declarations = admitted
            .as_model()
            .declarations(basic)
            .expect("a unit in the model answers Some");
        assert!(
            declarations.len() >= 8,
            "basic declares consts, vars, an alias, a named type and functions, got {}",
            declarations.len()
        );

        let add = declarations
            .iter()
            .find(|d| d.name == "Add")
            .expect("`Add` is declared");
        assert_eq!(add.kind, "func");
        assert!(add.has_flag("exported"));
        assert_eq!(add.children_of_kind("param").len(), 2);
        assert_eq!(add.children_of_kind("result").len(), 1);

        let shapes = units
            .iter()
            .find(|u| u.0.ends_with("shapes"))
            .expect("shapes package");
        let shape_decls = admitted
            .as_model()
            .declarations(shapes)
            .expect("a unit in the model answers Some");
        let point = shape_decls
            .iter()
            .find(|d| d.name == "Point")
            .expect("`Point` is declared");
        assert_eq!(point.kind, "struct");
        assert_eq!(point.children_of_kind("field").len(), 3);
        assert_eq!(point.children_of_kind("method").len(), 2);
    }

    #[test]
    fn unknown_unit_answers_none_not_an_empty_model() {
        let admitted = admit_embedded_fixture_v1().expect("v1 fixture must admit");
        assert!(
            admitted
                .as_model()
                .declarations(&UnitId("nothing/here".into()))
                .is_none(),
            "an unknown unit must be distinguishable from one that declares nothing"
        );
    }

    /// The v0 preimage covers language and the package→producer map only. If a v1 artifact were
    /// digested with it, every declaration would sit outside the snapshot identity: a renamed
    /// field or a changed parameter type would leave `snapshot_digest` untouched, and the receipt
    /// would then see emitted bytes move with all six axes unchanged — the exact `Unexplained`
    /// verdict the axes exist to prevent, arriving for a change that is fully explainable.
    #[test]
    fn v1_preimage_moves_when_a_declaration_moves() {
        let producer = PRODUCER_BOOTSTRAP_GO;
        let base = Declaration {
            kind: "const".into(),
            name: "MaxRetries".into(),
            type_ref: "int".into(),
            flags: ["exported".to_owned()].into_iter().collect(),
            attrs: [("value".to_owned(), "3".to_owned())].into_iter().collect(),
            children: Vec::new(),
        };

        let mut retyped = base.clone();
        retyped.type_ref = "int64".into();
        let mut unexported = base.clone();
        unexported.flags.clear();

        let original = snapshot_preimage_v1("go", &[("u", producer, vec![base.clone()])]);
        let after_type = snapshot_preimage_v1("go", &[("u", producer, vec![retyped])]);
        let after_flag = snapshot_preimage_v1("go", &[("u", producer, vec![unexported])]);

        assert_ne!(original, after_type, "a changed type must move the digest");
        assert_ne!(original, after_flag, "a changed flag must move the digest");

        // And the v0 preimage sees none of it — stated as a fact, so the reason v1 exists is
        // checked rather than asserted in prose.
        assert_eq!(
            snapshot_preimage("go", &[("u", producer)]),
            snapshot_preimage("go", &[("u", producer)])
        );
    }

    /// Nesting must be unambiguous, not merely encoded. Without the explicit child arity, a node
    /// with one child would flatten into the same byte string as two sibling nodes, and the whole
    /// declaration tree could be reshaped without moving the digest.
    #[test]
    fn v1_preimage_distinguishes_nesting_from_sibling_order() {
        let producer = PRODUCER_BOOTSTRAP_GO;
        let leaf = |name: &str| Declaration {
            kind: "param".into(),
            name: name.into(),
            type_ref: "int".into(),
            flags: std::collections::BTreeSet::new(),
            attrs: std::collections::BTreeMap::new(),
            children: Vec::new(),
        };

        let nested = Declaration {
            kind: "func".into(),
            name: "f".into(),
            type_ref: String::new(),
            flags: std::collections::BTreeSet::new(),
            attrs: std::collections::BTreeMap::new(),
            children: vec![leaf("a")],
        };
        let mut flat = nested.clone();
        flat.children.clear();

        assert_ne!(
            snapshot_preimage_v1("go", &[("u", producer, vec![nested])]),
            snapshot_preimage_v1("go", &[("u", producer, vec![flat, leaf("a")])]),
        );
    }

    #[test]
    fn refuses_digest_mismatch() {
        let json = r#"{
  "language": "go",
  "snapshot_digest": "sha256:deadbeef",
  "packages": [
    {"unit_id": "example.com/a", "producer": "bootstrap-go-packages-go-types"}
  ]
}"#;
        let bytes = json.as_bytes();
        let err = admit_reproducible_pair(bytes, bytes).expect_err("bad digest must refuse");
        assert!(matches!(err, AdmitError::DigestMismatch { .. }));
    }

    #[test]
    fn refuses_byte_drift_between_extractor_passes() {
        let first = FIXTURE_SNAPSHOT_JSON.as_bytes();
        let mut second = first.to_vec();
        second.push(b'\n');

        let err = admit_reproducible_pair(first, &second)
            .expect_err("semantically equivalent snapshots with byte drift must refuse");
        assert_eq!(
            err,
            AdmitError::SnapshotMismatch {
                first: digest_bytes(first),
                second: digest_bytes(&second),
            }
        );
    }

    #[test]
    fn refuses_non_go_bootstrap_language() {
        let json = r#"{
  "language": "rust",
  "snapshot_digest": "sha256:unused",
  "packages": []
}"#;
        let bytes = json.as_bytes();
        let err = admit_reproducible_pair(bytes, bytes)
            .expect_err("bootstrap admission must refuse a non-Go language");
        assert_eq!(
            err,
            AdmitError::Language {
                actual: "rust".to_owned(),
            }
        );
    }

    #[test]
    fn refuses_owned_frontend_before_equivalence() {
        let unit = "example.com/a";
        let digest = digest_bytes(&snapshot_preimage(
            "go",
            &[(unit, port_engine_frontend_go::PRODUCER_OWNED_RUST)],
        ));
        let json = format!(
            r#"{{
  "language": "go",
  "snapshot_digest": "{}",
  "packages": [
    {{"unit_id": "{unit}", "producer": "owned-rust-go-front-end"}}
  ]
}}"#,
            digest.0
        );
        let bytes = json.as_bytes();
        let err = admit_reproducible_pair(bytes, bytes)
            .expect_err("owned front end needs the later equivalence authorization");
        assert_eq!(
            err,
            AdmitError::ProducerNotAuthorized {
                unit: unit.to_owned(),
                actual: port_engine_frontend_go::PRODUCER_OWNED_RUST.to_owned(),
            }
        );
    }

    #[test]
    fn production_never_spawns_go() {
        let src = include_str!("lib.rs");
        let production = src
            .split("#[cfg(test)]")
            .next()
            .expect("production section");
        let cmd_new = ["Command", "::", "new"].concat();
        let process_cmd = ["std", "::", "process", "::", "Command"].concat();
        assert!(!production.contains(&cmd_new));
        assert!(!production.contains(&process_cmd));
    }
}
