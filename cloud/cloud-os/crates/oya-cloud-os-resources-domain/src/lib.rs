//! # talos-resources
//!
//! The COSI resource type catalog for the operating-system Talos port. This crate models
//! Talos's `pkg/machinery/resources/*` packages: every typed resource has a
//! canonical type name, lives in a namespace, carries command-line aliases and
//! `talosctl get` print columns, and declares whether its spec is sensitive.
//!
//! The pieces:
//!
//! - [`definition`][]: [`ResourceDefinition`](definition::ResourceDefinition) (RD)
//!   — the per-type descriptor mirroring COSI's `meta.ResourceDefinition`, built
//!   via a validating builder.
//! - [`printcolumns`][]: [`PrintColumn`](printcolumns::PrintColumn) plus a small
//!   self-contained `JSONPath` evaluator over a
//!   [`SpecValue`](printcolumns::SpecValue) tree, used to render `talosctl get`
//!   table cells.
//! - [`namespaces`][]: the well-known COSI [`Namespace`](namespaces::Namespace)s
//!   (`config`, `network`, `k8s`, `secrets`, ...).
//! - [`kinds`][]: the full type catalog across config, network, k8s, cluster,
//!   secrets, runtime, hardware, perf, time, etcd, block, siderolink and files.
//! - [`aliases`][]: the [`AliasTable`](aliases::AliasTable) that resolves a
//!   friendly name to a canonical type, reporting ambiguities.
//! - [`registry`][]: the [`Registry`](registry::Registry) that COSI and
//!   `talosctl get` consume — namespace + RD registration, alias resolution,
//!   default-namespace selection.
//!
//! [`Registry::with_defaults`](registry::Registry::with_defaults) yields a
//! registry pre-loaded with the entire built-in catalog, the analogue of
//! Talos's `RegisterDefaultResources` at boot.

#![warn(clippy::pedantic)]
// Type and module names intentionally echo the crate's domain vocabulary
// (`ResourceDefinition`, `definition`, etc.) to match Talos/COSI naming.
#![allow(clippy::module_name_repetitions)]
// Pedantic lints intentionally allowed crate-wide: these are documentation- and
// annotation-only nags that do not change behavior or improve idiom for this
// internal catalog crate. Suppressing them keeps the signal-to-noise ratio of
// `clippy::pedantic` useful for the lints that do matter.
#![allow(
    clippy::must_use_candidate,
    clippy::return_self_not_must_use,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::doc_markdown
)]

pub mod aliases;
pub mod definition;
pub mod kinds;
pub mod namespaces;
pub mod printcolumns;
pub mod registry;

pub use aliases::AliasTable;
pub use definition::{ResourceDefinition, ResourceDefinitionBuilder, Sensitivity};
pub use namespaces::Namespace;
pub use printcolumns::{PrintColumn, SpecValue};
pub use registry::Registry;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_to_end_get_machineconfig() {
        // Mirror `talosctl get mc`: resolve the alias, pick the namespace,
        // confirm redaction.
        let reg = Registry::with_defaults();
        let rd = reg.resolve("mc").expect("alias resolves");
        assert_eq!(rd.kind(), "MachineConfigs");
        assert_eq!(reg.effective_namespace(rd, None).unwrap(), "config");
        assert!(rd.sensitivity().is_sensitive());
    }

    #[test]
    fn end_to_end_render_route_columns() {
        let reg = Registry::with_defaults();
        let rd = reg.resolve("route").expect("alias resolves");
        let spec = SpecValue::map([
            ("destination", SpecValue::Str("10.0.0.0/24".into())),
            ("gateway", SpecValue::Str("10.0.0.1".into())),
        ]);
        let cells: Vec<String> = rd.print_columns().iter().map(|c| c.render(&spec)).collect();
        assert_eq!(
            cells,
            vec!["10.0.0.0/24".to_string(), "10.0.0.1".to_string()]
        );
    }

    #[test]
    fn registry_round_trips_with_alias_table() {
        let reg = Registry::with_defaults();
        let defs: Vec<ResourceDefinition> = reg.definitions().cloned().collect();
        let table = AliasTable::from_definitions(&defs);
        // Everything resolvable through the registry resolves through a freshly
        // built table identically. Exact canonical types must always round
        // trip; short kinds only round-trip when they are unambiguous (Talos has
        // both runtime and block `MountStatuses`).
        for d in &defs {
            assert_eq!(table.resolve(d.type_name()).unwrap(), d.type_name());
            if table.is_unambiguous(d.kind()) {
                assert_eq!(table.resolve(d.kind()).unwrap(), d.type_name());
            }
            for alias in d.aliases() {
                assert_eq!(table.resolve(alias).unwrap(), d.type_name());
            }
        }
        assert_eq!(
            table.resolve("MountStatuses").unwrap_err().kind(),
            "invalid"
        );
        assert!(table.len() >= reg.len());
    }
}
