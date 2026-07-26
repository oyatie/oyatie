//! Timeserver configuration layering, mirroring
//! `siderolabs/talos`'s `TimeServerConfigController`.
//!
//! Talos derives the effective NTP server list from several config *layers*,
//! each contributed by a different source and merged by priority:
//!
//! 1. **Machine config** (`machine.time.servers`) — highest priority.
//! 2. **Operator** — dynamic network operators such as DHCP can publish NTP
//!    servers discovered from leases.
//! 3. **Platform** — cloud platforms (AWS, GCP, ...) advertise a local NTP
//!    server (e.g. `169.254.169.123`) which is used when the machine config does
//!    not specify one.
//! 4. **Cmdline** — a `talos.experimental.timeserver=` kernel arg.
//! 5. **Default** — the built-in fallback (`time.cloudflare.com`).
//!
//! The controller picks the highest-priority *non-empty* layer (it does not
//! concatenate layers; a present machine-config list fully replaces the rest),
//! deduplicates, and emits a single `TimeServerSpec`. This module models that
//! resolution faithfully, plus the disable flag that short-circuits sync.

use crate::sync::SyncSpec;
use crate::{Result, TimeError};

/// Where a timeserver layer came from. Higher discriminant = higher priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConfigLayer {
    /// Built-in fallback list.
    Default,
    /// `talos.experimental.timeserver=` kernel cmdline argument.
    Cmdline,
    /// Cloud platform metadata (e.g. the link-local NTP server).
    Platform,
    /// Dynamic network operator output (e.g. DHCP NTP servers).
    Operator,
    /// Operator-supplied `machine.time.servers` in the machine config.
    MachineConfig,
}

impl ConfigLayer {
    /// Short, stable name for the layer (matches the Talos controller's source).
    pub fn name(self) -> &'static str {
        match self {
            ConfigLayer::Default => "default",
            ConfigLayer::Cmdline => "cmdline",
            ConfigLayer::Platform => "platform",
            ConfigLayer::Operator => "operator",
            ConfigLayer::MachineConfig => "machine-config",
        }
    }
}

/// One contributed layer: a source and the servers it provides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeServerLayer {
    /// The originating layer.
    pub layer: ConfigLayer,
    /// The servers this layer contributes (may be empty, in which case the layer
    /// does not participate).
    pub servers: Vec<String>,
}

impl TimeServerLayer {
    /// Build a layer from a source and a server iterator.
    pub fn new<I, S>(layer: ConfigLayer, servers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        TimeServerLayer {
            layer,
            servers: servers.into_iter().map(Into::into).collect(),
        }
    }

    /// Whether this layer contributes any servers.
    pub fn is_empty(&self) -> bool {
        self.servers.iter().all(|s| s.trim().is_empty())
    }
}

/// The merged, resolved timeserver configuration the controller publishes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeServerSpec {
    /// The effective, deduplicated server list.
    pub servers: Vec<String>,
    /// The layer that won the resolution.
    pub source: ConfigLayer,
    /// Whether NTP sync is disabled outright.
    pub disabled: bool,
}

impl TimeServerSpec {
    /// Resolve the effective spec from a set of layers.
    ///
    /// The highest-priority non-empty layer wins; its servers are trimmed and
    /// deduplicated (preserving first-seen order). When `disabled` is set, the
    /// resulting spec carries an empty list and the disabled flag.
    pub fn resolve(layers: &[TimeServerLayer], disabled: bool) -> Self {
        if disabled {
            return TimeServerSpec {
                servers: Vec::new(),
                source: ConfigLayer::Default,
                disabled: true,
            };
        }
        // Choose the highest-priority non-empty layer.
        let chosen = layers
            .iter()
            .filter(|l| !l.is_empty())
            .max_by_key(|l| l.layer);

        match chosen {
            Some(layer) => {
                let mut seen: Vec<String> = Vec::new();
                for s in &layer.servers {
                    let t = s.trim();
                    if t.is_empty() {
                        continue;
                    }
                    let lower = t.to_ascii_lowercase();
                    if !seen.iter().any(|e| e.eq_ignore_ascii_case(&lower)) {
                        seen.push(t.to_string());
                    }
                }
                TimeServerSpec {
                    servers: seen,
                    source: layer.layer,
                    disabled: false,
                }
            }
            None => TimeServerSpec {
                servers: vec![String::from("time.cloudflare.com")],
                source: ConfigLayer::Default,
                disabled: false,
            },
        }
    }

    /// Convenience: resolve from the conventional four ordered inputs, where each
    /// is an optional server list (`None`/empty meaning "layer absent").
    pub fn from_sources(
        machine_config: Option<Vec<String>>,
        platform: Option<Vec<String>>,
        cmdline: Option<Vec<String>>,
        disabled: bool,
    ) -> Self {
        TimeServerSpec::from_sources_with_operator(
            machine_config,
            None,
            platform,
            cmdline,
            disabled,
        )
    }

    /// Resolve from conventional inputs plus dynamic network operator servers.
    ///
    /// This mirrors Talos' network `ConfigOperator` layer for DHCP-provided NTP
    /// servers: operator output wins over platform/cmdline/default, while
    /// machine configuration remains the highest-priority explicit user input.
    pub fn from_sources_with_operator(
        machine_config: Option<Vec<String>>,
        operator: Option<Vec<String>>,
        platform: Option<Vec<String>>,
        cmdline: Option<Vec<String>>,
        disabled: bool,
    ) -> Self {
        let mut layers = Vec::new();
        if let Some(s) = machine_config {
            layers.push(TimeServerLayer::new(ConfigLayer::MachineConfig, s));
        }
        if let Some(s) = operator {
            layers.push(TimeServerLayer::new(ConfigLayer::Operator, s));
        }
        if let Some(s) = platform {
            layers.push(TimeServerLayer::new(ConfigLayer::Platform, s));
        }
        if let Some(s) = cmdline {
            layers.push(TimeServerLayer::new(ConfigLayer::Cmdline, s));
        }
        TimeServerSpec::resolve(&layers, disabled)
    }

    /// Project the resolved spec into a runnable [`SyncSpec`], validating it.
    pub fn to_sync_spec(&self) -> Result<SyncSpec> {
        if self.disabled {
            return Ok(SyncSpec::disabled());
        }
        if self.servers.is_empty() {
            return Err(TimeError::invalid_config(
                "resolved timeserver spec has no servers",
            ));
        }
        let spec = SyncSpec::with_servers(self.servers.clone());
        spec.validate()?;
        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn machine_config_wins_over_platform_and_default() {
        let spec = TimeServerSpec::from_sources(
            Some(vec!["operator.ntp".into()]),
            Some(vec!["169.254.169.123".into()]),
            None,
            false,
        );
        assert_eq!(spec.source, ConfigLayer::MachineConfig);
        assert_eq!(spec.servers, vec!["operator.ntp".to_string()]);
    }

    #[test]
    fn platform_used_when_machine_config_absent() {
        let spec =
            TimeServerSpec::from_sources(None, Some(vec!["169.254.169.123".into()]), None, false);
        assert_eq!(spec.source, ConfigLayer::Platform);
        assert_eq!(spec.servers, vec!["169.254.169.123".to_string()]);
    }

    #[test]
    fn operator_wins_over_platform_and_cmdline() {
        let spec = TimeServerSpec::from_sources_with_operator(
            None,
            Some(vec!["fd00:ec2::123".into(), " 169.254.169.123 ".into()]),
            Some(vec!["platform.ntp".into()]),
            Some(vec!["cmdline.ntp".into()]),
            false,
        );
        assert_eq!(spec.source, ConfigLayer::Operator);
        assert_eq!(
            spec.servers,
            vec!["fd00:ec2::123".to_string(), "169.254.169.123".to_string()]
        );
    }

    #[test]
    fn machine_config_still_wins_over_operator() {
        let spec = TimeServerSpec::from_sources_with_operator(
            Some(vec!["machine.ntp".into()]),
            Some(vec!["dhcp.ntp".into()]),
            Some(vec!["platform.ntp".into()]),
            None,
            false,
        );
        assert_eq!(spec.source, ConfigLayer::MachineConfig);
        assert_eq!(spec.servers, vec!["machine.ntp".to_string()]);
    }

    #[test]
    fn cmdline_beats_default_but_loses_to_platform() {
        let spec = TimeServerSpec::from_sources(
            None,
            Some(vec!["plat.ntp".into()]),
            Some(vec!["cmdline.ntp".into()]),
            false,
        );
        assert_eq!(spec.source, ConfigLayer::Platform);

        let spec2 =
            TimeServerSpec::from_sources(None, None, Some(vec!["cmdline.ntp".into()]), false);
        assert_eq!(spec2.source, ConfigLayer::Cmdline);
        assert_eq!(spec2.servers, vec!["cmdline.ntp".to_string()]);
    }

    #[test]
    fn falls_back_to_default_when_no_layers() {
        let spec = TimeServerSpec::resolve(&[], false);
        assert_eq!(spec.source, ConfigLayer::Default);
        assert_eq!(spec.servers, vec!["time.cloudflare.com".to_string()]);
    }

    #[test]
    fn empty_layer_does_not_participate() {
        let spec = TimeServerSpec::from_sources(
            Some(vec!["  ".into(), String::new()]),
            Some(vec!["platform.ntp".into()]),
            None,
            false,
        );
        // machine-config layer is all-whitespace => platform wins.
        assert_eq!(spec.source, ConfigLayer::Platform);
    }

    #[test]
    fn dedupes_and_trims_preserving_order() {
        let spec = TimeServerSpec::resolve(
            &[TimeServerLayer::new(
                ConfigLayer::MachineConfig,
                ["  a.ntp ", "B.NTP", "a.ntp", "c.ntp"],
            )],
            false,
        );
        assert_eq!(
            spec.servers,
            vec![
                "a.ntp".to_string(),
                "B.NTP".to_string(),
                "c.ntp".to_string()
            ]
        );
    }

    #[test]
    fn disabled_short_circuits() {
        let spec = TimeServerSpec::from_sources(Some(vec!["x".into()]), None, None, true);
        assert!(spec.disabled);
        assert!(spec.servers.is_empty());
        let sync = spec.to_sync_spec().unwrap();
        assert!(!sync.enabled);
    }

    #[test]
    fn to_sync_spec_validates() {
        let spec = TimeServerSpec::from_sources(Some(vec!["good.ntp".into()]), None, None, false);
        let sync = spec.to_sync_spec().unwrap();
        assert!(sync.enabled);
        assert_eq!(sync.servers, vec!["good.ntp".to_string()]);

        // A spec whose server contains whitespace would already be filtered out
        // at resolve time, leaving an empty list -> error.
        let empty = TimeServerSpec {
            servers: Vec::new(),
            source: ConfigLayer::Default,
            disabled: false,
        };
        assert!(empty.to_sync_spec().is_err());
    }

    #[test]
    fn layer_ordering_is_total() {
        assert!(ConfigLayer::MachineConfig > ConfigLayer::Platform);
        assert!(ConfigLayer::MachineConfig > ConfigLayer::Operator);
        assert!(ConfigLayer::Operator > ConfigLayer::Platform);
        assert!(ConfigLayer::Platform > ConfigLayer::Cmdline);
        assert!(ConfigLayer::Cmdline > ConfigLayer::Default);
        assert_eq!(ConfigLayer::Operator.name(), "operator");
        assert_eq!(ConfigLayer::Platform.name(), "platform");
    }
}
