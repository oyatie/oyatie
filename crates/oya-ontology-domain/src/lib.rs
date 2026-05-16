//! Ontology domain surface.
//!
//! ADR-0122 moved the data-classed typed-entity kernel into
//! `oya-ontology-kernel`. This crate intentionally remains as the domain-layer
//! import surface for existing application/API code while the usecase/adapter
//! crates are sequenced separately.

pub use oya_ontology_kernel::*;
