//! Configuration layering.
//!
//! Talos network specs can come from multiple sources, and a deterministic
//! priority order decides which source wins when several provide the same
//! resource. Mirrors `internal/app/machined/pkg/controllers/network` where each
//! controller writes specs tagged with a [`ConfigLayer`] and the merge
//! controllers (e.g. `AddressMergeController`) fold them by priority.

use core::cmp::Ordering;
use core::fmt;

/// The provenance of a network spec. Higher-priority layers override
/// lower-priority ones during merging.
///
/// The ordering, from lowest to highest precedence, mirrors Talos:
/// `Default < Cmdline < Platform < Operator < Configuration`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigLayer {
    /// Built-in defaults (e.g. loopback, default hostname).
    Default,
    /// Values parsed from the kernel command line (`ip=`, `talos.hostname=`).
    Cmdline,
    /// Values discovered from the cloud/platform metadata service.
    Platform,
    /// Values produced by a dynamic operator (DHCP, virtual IP, ...).
    Operator,
    /// Values from the machine configuration document (highest priority).
    Configuration,
}

impl ConfigLayer {
    /// Numeric precedence; larger wins.
    pub fn precedence(self) -> u8 {
        match self {
            ConfigLayer::Default => 0,
            ConfigLayer::Cmdline => 1,
            ConfigLayer::Platform => 2,
            ConfigLayer::Operator => 3,
            ConfigLayer::Configuration => 4,
        }
    }

    /// Stable lowercase identifier, as used in COSI resource ids and labels.
    pub fn as_str(self) -> &'static str {
        match self {
            ConfigLayer::Default => "default",
            ConfigLayer::Cmdline => "cmdline",
            ConfigLayer::Platform => "platform",
            ConfigLayer::Operator => "operator",
            ConfigLayer::Configuration => "configuration",
        }
    }

    /// All layers in ascending precedence order.
    pub fn all() -> [ConfigLayer; 5] {
        [
            ConfigLayer::Default,
            ConfigLayer::Cmdline,
            ConfigLayer::Platform,
            ConfigLayer::Operator,
            ConfigLayer::Configuration,
        ]
    }

    /// Returns whichever layer wins between `self` and `other`.
    pub fn winner(self, other: ConfigLayer) -> ConfigLayer {
        if self.precedence() >= other.precedence() {
            self
        } else {
            other
        }
    }
}

impl PartialOrd for ConfigLayer {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ConfigLayer {
    fn cmp(&self, other: &Self) -> Ordering {
        self.precedence().cmp(&other.precedence())
    }
}

impl fmt::Display for ConfigLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence_order() {
        assert!(ConfigLayer::Configuration > ConfigLayer::Operator);
        assert!(ConfigLayer::Operator > ConfigLayer::Platform);
        assert!(ConfigLayer::Platform > ConfigLayer::Cmdline);
        assert!(ConfigLayer::Cmdline > ConfigLayer::Default);
    }

    #[test]
    fn winner_picks_higher_precedence() {
        assert_eq!(
            ConfigLayer::Default.winner(ConfigLayer::Configuration),
            ConfigLayer::Configuration
        );
        assert_eq!(
            ConfigLayer::Operator.winner(ConfigLayer::Platform),
            ConfigLayer::Operator
        );
        // tie resolves to the receiver
        assert_eq!(
            ConfigLayer::Platform.winner(ConfigLayer::Platform),
            ConfigLayer::Platform
        );
    }

    #[test]
    fn all_is_sorted_ascending() {
        let layers = ConfigLayer::all();
        for w in layers.windows(2) {
            assert!(w[0] < w[1]);
        }
    }

    #[test]
    fn string_round_trip_labels() {
        assert_eq!(ConfigLayer::Cmdline.as_str(), "cmdline");
        assert_eq!(ConfigLayer::Configuration.to_string(), "configuration");
    }
}
