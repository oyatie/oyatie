//! Ontology domain surface.
//!
//! ADR-0122 moved the data-classed typed-entity kernel into
//! `ontology-kernel`. This crate intentionally remains as the domain-layer
//! import surface for existing application/API code while the usecase/adapter
//! crates are sequenced separately.

pub use data_ontology_kernel::*;
