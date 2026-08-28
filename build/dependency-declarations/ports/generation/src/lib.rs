//! Dependency-free generation effect seam for declaration reconciliation.

#![forbid(unsafe_code)]

/// Generates an output from an admitted request without defining either type.
pub trait GenerationPort<Request, Output, Error> {
    fn generate(&self, request: &Request) -> Result<Output, Error>;
}
