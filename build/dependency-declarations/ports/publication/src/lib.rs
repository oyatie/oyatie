//! Dependency-free publication effect seam for declaration reconciliation.

#![forbid(unsafe_code)]

/// Publishes an admitted request without defining request or observation types.
pub trait PublicationPort<Request, Output, Error> {
    fn publish(&self, request: &Request) -> Result<Output, Error>;
}

/// Reports whether an effect adapter can honor a profile before any attempt.
pub trait PublicationCapabilityPort<Profile> {
    fn supports(&self, profile: &Profile) -> bool;
}
