//! The six receipt axes ADR-0637 fixes, and the receipt that carries them.
//!
//! Every emitted-byte change must be attributable to at least one axis; an unattributable change
//! is RED. See `port-engine-kernel::verify`.

use std::collections::BTreeSet;

use crate::identity::Digest;

/// The six receipt axes ADR-0637 fixes. Every emitted-byte change must be attributable to at least
/// one of them; see `port-engine-kernel::verify`.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ReceiptAxis {
    /// The upstream pin the source snapshot was taken at.
    Pin,
    /// The digest of the source snapshot.
    Snapshot,
    /// The digest of the engine binary.
    Engine,
    /// The digest of the rule pack.
    RulePack,
    /// The digest of the toolchain.
    Toolchain,
    /// The digest of the formatter.
    Formatter,
}

/// Every axis, in declaration order. Registered as a constant so a seventh axis cannot be added
/// without the comparison in [`Receipt::differing_axes`] being updated alongside it.
pub const RECEIPT_AXES: [ReceiptAxis; 6] = [
    ReceiptAxis::Pin,
    ReceiptAxis::Snapshot,
    ReceiptAxis::Engine,
    ReceiptAxis::RulePack,
    ReceiptAxis::Toolchain,
    ReceiptAxis::Formatter,
];

/// The six-axis provenance of one emission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    /// The upstream pin (an opaque revision identifier).
    pub pin: String, // data_class: INTERNAL_ONLY
    /// Digest of the source snapshot.
    pub snapshot_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the engine that emitted.
    pub engine_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the rule pack in force.
    pub rulepack_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the toolchain in force.
    pub toolchain_digest: Digest, // data_class: INTERNAL_ONLY
    /// Digest of the formatter in force.
    pub formatter_digest: Digest, // data_class: INTERNAL_ONLY
}

impl Receipt {
    /// The axes on which `self` and `other` disagree.
    #[must_use]
    pub fn differing_axes(&self, other: &Self) -> BTreeSet<ReceiptAxis> {
        let mut differing = BTreeSet::new();
        for axis in RECEIPT_AXES {
            let differs = match axis {
                ReceiptAxis::Pin => self.pin != other.pin,
                ReceiptAxis::Snapshot => self.snapshot_digest != other.snapshot_digest,
                ReceiptAxis::Engine => self.engine_digest != other.engine_digest,
                ReceiptAxis::RulePack => self.rulepack_digest != other.rulepack_digest,
                ReceiptAxis::Toolchain => self.toolchain_digest != other.toolchain_digest,
                ReceiptAxis::Formatter => self.formatter_digest != other.formatter_digest,
            };
            if differs {
                differing.insert(axis);
            }
        }
        differing
    }

    /// The axes that say NOTHING — an empty pin or an empty digest.
    ///
    /// [`Receipt::differing_axes`] answers "did this axis move", which is only a usable answer
    /// when the axis carries a value on both sides. An unfilled axis makes an apparent difference
    /// absence of information rather than evidence of a cause, and `port-engine-kernel::verify`
    /// must not spend it as an explanation. Walks [`RECEIPT_AXES`] for the same reason
    /// `differing_axes` does: a seventh axis cannot be added without this answer being updated
    /// alongside it.
    #[must_use]
    pub fn incomplete_axes(&self) -> BTreeSet<ReceiptAxis> {
        let mut incomplete = BTreeSet::new();
        for axis in RECEIPT_AXES {
            let empty = match axis {
                ReceiptAxis::Pin => self.pin.is_empty(),
                ReceiptAxis::Snapshot => self.snapshot_digest.0.is_empty(),
                ReceiptAxis::Engine => self.engine_digest.0.is_empty(),
                ReceiptAxis::RulePack => self.rulepack_digest.0.is_empty(),
                ReceiptAxis::Toolchain => self.toolchain_digest.0.is_empty(),
                ReceiptAxis::Formatter => self.formatter_digest.0.is_empty(),
            };
            if empty {
                incomplete.insert(axis);
            }
        }
        incomplete
    }
}
