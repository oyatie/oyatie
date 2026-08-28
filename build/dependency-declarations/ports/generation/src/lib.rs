//! Dependency-free generation effect seam for declaration reconciliation.

#![forbid(unsafe_code)]

/// Generates an output from an admitted request without defining either type.
pub trait GenerationPort<Request, Output, Error> {
    fn generate(&self, request: &Request) -> Result<Output, Error>;
}

/// Independently parses rendered BUCK bytes into a bounded typed projection.
///
/// This is intentionally distinct from [`GenerationPort`]: a generator cannot
/// attest to its own rendering by reusing its producer graph as parser output.
pub trait GeneratedBuckParserPort<Output, Error> {
    fn parse(&self, rendered_buck: &[u8]) -> Result<Output, Error>;
}
