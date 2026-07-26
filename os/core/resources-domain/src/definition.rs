//! Resource definitions.
//!
//! Mirrors COSI's `meta.ResourceDefinition` and Talos's
//! `pkg/machinery/resources/.../*.go` `ResourceDefinition()` methods. A
//! [`ResourceDefinition`] (RD) describes a registered resource type: its
//! canonical kind (`Type`), the default namespace it lives in, the short
//! aliases `talosctl get` accepts, the columns `talosctl get` prints, and
//! whether the spec carries sensitive data that must be redacted.
//!
//! In real Talos every typed resource has a `ResourceDefinition()` method that
//! returns one of these; the COSI runtime stores them in the `meta` namespace
//! as `ResourceDefinition` resources so that `talosctl get rd` can list them.

use crate::printcolumns::PrintColumn;
use os_kernel::error::{Error, Result};

/// Whether a resource's spec contains sensitive data.
///
/// Resources flagged [`Sensitivity::Sensitive`] (e.g. secrets) are redacted by
/// the API layer unless the caller holds the `os:admin` role; non-sensitive
/// resources are returned verbatim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Sensitivity {
    /// The spec may be returned to any authorized reader.
    #[default]
    NonSensitive,
    /// The spec must be redacted from low-privilege readers.
    Sensitive,
}

impl Sensitivity {
    /// Stable lowercase string used in the RD listing.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Sensitivity::NonSensitive => "",
            Sensitivity::Sensitive => "sensitive",
        }
    }

    /// Whether redaction is required.
    #[must_use]
    pub fn is_sensitive(self) -> bool {
        matches!(self, Sensitivity::Sensitive)
    }
}

/// A registered resource type definition.
///
/// The canonical [`type_name`](ResourceDefinition::type_name) follows Talos
/// convention `Kind<Domain>.<group>.talos.dev` (for example
/// `MachineConfigs.config.talos.dev`). [`aliases`](ResourceDefinition::aliases)
/// are the friendly names accepted on the command line
/// (`machineconfig`, `mc`, ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceDefinition {
    type_name: String,
    default_namespace: String,
    aliases: Vec<String>,
    print_columns: Vec<PrintColumn>,
    sensitivity: Sensitivity,
}

impl ResourceDefinition {
    /// Begin building a definition for `type_name` in `default_namespace`.
    #[must_use]
    pub fn builder(
        type_name: impl Into<String>,
        default_namespace: impl Into<String>,
    ) -> ResourceDefinitionBuilder {
        ResourceDefinitionBuilder {
            type_name: type_name.into(),
            default_namespace: default_namespace.into(),
            aliases: Vec::new(),
            print_columns: Vec::new(),
            sensitivity: Sensitivity::NonSensitive,
        }
    }

    /// The canonical resource type name (e.g. `MachineConfigs.config.talos.dev`).
    #[must_use]
    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    /// The short kind portion of the type name (the part before the first `.`).
    ///
    /// `MachineConfigs.config.talos.dev` -> `MachineConfigs`.
    #[must_use]
    pub fn kind(&self) -> &str {
        self.type_name.split('.').next().unwrap_or(&self.type_name)
    }

    /// The suffix/group of the type name after the kind.
    ///
    /// `MachineConfigs.config.talos.dev` -> `config.talos.dev`.
    #[must_use]
    pub fn group(&self) -> &str {
        self.type_name
            .split_once('.')
            .map_or("", |(_, group)| group)
    }

    /// The namespace resources of this type live in by default.
    #[must_use]
    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    /// The command-line aliases for this type (lowercased, deduplicated).
    #[must_use]
    pub fn aliases(&self) -> &[String] {
        &self.aliases
    }

    /// The columns `talosctl get` prints for this type.
    #[must_use]
    pub fn print_columns(&self) -> &[PrintColumn] {
        &self.print_columns
    }

    /// The spec sensitivity classification.
    #[must_use]
    pub fn sensitivity(&self) -> Sensitivity {
        self.sensitivity
    }

    /// Whether `name` matches this definition by canonical type, kind, or any
    /// alias. Matching is case-insensitive.
    #[must_use]
    pub fn matches(&self, name: &str) -> bool {
        let name = name.to_ascii_lowercase();
        self.type_name.eq_ignore_ascii_case(&name)
            || self.kind().eq_ignore_ascii_case(&name)
            || self.aliases.iter().any(|a| a == &name)
    }
}

/// Builder for [`ResourceDefinition`] with validation on [`build`].
#[derive(Debug, Clone)]
pub struct ResourceDefinitionBuilder {
    type_name: String,
    default_namespace: String,
    aliases: Vec<String>,
    print_columns: Vec<PrintColumn>,
    sensitivity: Sensitivity,
}

impl ResourceDefinitionBuilder {
    /// Add a command-line alias (stored lowercased; duplicates are ignored).
    #[must_use]
    pub fn alias(mut self, alias: impl Into<String>) -> Self {
        let alias = alias.into().to_ascii_lowercase();
        if !alias.is_empty() && !self.aliases.contains(&alias) {
            self.aliases.push(alias);
        }
        self
    }

    /// Add several aliases at once.
    #[must_use]
    pub fn aliases<I, S>(mut self, aliases: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for a in aliases {
            self = self.alias(a);
        }
        self
    }

    /// Add a print column.
    #[must_use]
    pub fn print_column(mut self, name: impl Into<String>, json_path: impl Into<String>) -> Self {
        self.print_columns.push(PrintColumn::new(name, json_path));
        self
    }

    /// Mark this resource's spec as sensitive (redacted for low-privilege reads).
    #[must_use]
    pub fn sensitive(mut self) -> Self {
        self.sensitivity = Sensitivity::Sensitive;
        self
    }

    /// Finalize, validating the canonical type name and namespace.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Invalid`] if the type name is not a valid
    /// `Kind.group` form (see [`validate_type_name`]) or the default namespace
    /// is empty.
    pub fn build(self) -> Result<ResourceDefinition> {
        validate_type_name(&self.type_name)?;
        if self.default_namespace.is_empty() {
            return Err(Error::invalid("resource definition has empty namespace"));
        }
        Ok(ResourceDefinition {
            type_name: self.type_name,
            default_namespace: self.default_namespace,
            aliases: self.aliases,
            print_columns: self.print_columns,
            sensitivity: self.sensitivity,
        })
    }
}

/// Validate a canonical resource type name.
///
/// Talos requires the suffixed form `Kind.group.talos.dev`: a capitalized kind
/// followed by at least one dotted group component. We enforce a non-empty kind
/// starting with an uppercase ASCII letter and that the type ends with a
/// `talos.dev` (or, for Kubernetes-mirrored types, any) domain group.
pub fn validate_type_name(type_name: &str) -> Result<()> {
    if type_name.is_empty() {
        return Err(Error::invalid("resource type name is empty"));
    }
    let (kind, group) = type_name
        .split_once('.')
        .ok_or_else(|| Error::invalid("resource type name must be of the form Kind.group"))?;
    if kind.is_empty() {
        return Err(Error::invalid("resource type name has empty kind"));
    }
    let first = kind.chars().next().unwrap();
    if !first.is_ascii_uppercase() {
        return Err(Error::invalid(
            "resource kind must start with an uppercase letter",
        ));
    }
    for c in kind.chars() {
        if !c.is_ascii_alphanumeric() {
            return Err(Error::invalid("resource kind must be alphanumeric"));
        }
    }
    if group.is_empty() {
        return Err(Error::invalid("resource type name has empty group"));
    }
    for label in group.split('.') {
        if label.is_empty() {
            return Err(Error::invalid(
                "resource type name group has an empty label",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_parses_kind_and_group() {
        let rd = ResourceDefinition::builder("MachineConfigs.config.talos.dev", "config")
            .alias("machineconfig")
            .alias("mc")
            .print_column("VERSION", "{.version}")
            .build()
            .unwrap();
        assert_eq!(rd.kind(), "MachineConfigs");
        assert_eq!(rd.group(), "config.talos.dev");
        assert_eq!(rd.default_namespace(), "config");
        assert_eq!(rd.aliases(), &["machineconfig", "mc"]);
        assert_eq!(rd.print_columns().len(), 1);
        assert!(!rd.sensitivity().is_sensitive());
    }

    #[test]
    fn aliases_are_lowercased_and_deduped() {
        let rd = ResourceDefinition::builder("Routes.net.talos.dev", "network")
            .aliases(["Route", "ROUTE", "routes"])
            .build()
            .unwrap();
        assert_eq!(rd.aliases(), &["route", "routes"]);
    }

    #[test]
    fn matches_type_kind_and_alias() {
        let rd = ResourceDefinition::builder("NodeAddresses.net.talos.dev", "network")
            .alias("nodeaddress")
            .build()
            .unwrap();
        assert!(rd.matches("NodeAddresses.net.talos.dev"));
        assert!(rd.matches("nodeaddresses"));
        assert!(rd.matches("NodeAddresses"));
        assert!(rd.matches("nodeaddress"));
        assert!(!rd.matches("routes"));
    }

    #[test]
    fn sensitive_flag_round_trips() {
        let rd = ResourceDefinition::builder("MachineConfigs.config.talos.dev", "config")
            .sensitive()
            .build()
            .unwrap();
        assert!(rd.sensitivity().is_sensitive());
        assert_eq!(rd.sensitivity().as_str(), "sensitive");
    }

    #[test]
    fn invalid_type_names_rejected() {
        assert!(validate_type_name("").is_err());
        assert!(validate_type_name("NoGroup").is_err());
        assert!(validate_type_name("lowercase.config.talos.dev").is_err());
        assert!(validate_type_name("Bad-Kind.config.talos.dev").is_err());
        assert!(validate_type_name("Kind..talos.dev").is_err());
        assert!(validate_type_name("Kind.config.talos.dev").is_ok());
    }
}
