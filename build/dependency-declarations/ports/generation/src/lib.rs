//! Dependency-free generation effect seam for declaration reconciliation.

#![forbid(unsafe_code)]

/// Generates an output from an admitted request without defining either type.
pub trait GenerationPort<Request, Output, Error> {
    fn generate(&self, request: &Request) -> Result<Output, Error>;
}

/// Refuses an unsupported exact provider profile before any provider effect.
pub trait DeclarationProviderCapabilityPort<Profile> {
    fn supports(&self, profile: &Profile) -> bool;
}

/// Projects rendered declarations through an independent maintained parser.
pub trait RenderedDeclarationProjectionPort {
    type Profile;
    type Projection;
    type Error;

    fn profile(&self) -> &Self::Profile;
    fn project(&self, rendered: &[u8]) -> Result<Self::Projection, Self::Error>;
}

/// Qualifies one bounded declaration artifact against its configured consumer.
pub trait DeclarationConsumerQualificationPort<Request, Output, Error> {
    fn qualify(&self, request: &Request) -> Result<Output, Error>;
}

/// Refuses a configured declaration consumer profile before any qualification effect.
pub trait DeclarationConsumerCapabilityPort<Profile> {
    fn supports(&self, profile: &Profile) -> bool;
}
