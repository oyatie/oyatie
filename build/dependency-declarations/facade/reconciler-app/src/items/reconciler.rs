use dependency_declarations_generation::{
    DeclarationProviderCapabilityPort, GenerationPort,
};
use dependency_declarations_generation_reindeer::StarlarkSyntaxProjectionV1;
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::{
    GenerationInvocationV1, GenerationPortErrorV1, GenerationRequestV1,
    PublicationObservationV1, PublicationPortErrorV1, PublicationRequestV1, PublisherProfileV1,
    RawGenerationV1, reconcile,
};

/// Versioned reconciler over caller-supplied generation and publication effects.
pub struct ReindeerDeclarationReconcilerV1<G, P> {
    generator: G,
    projector: StarlarkSyntaxProjectionV1,
    publisher: P,
}

impl<G, P> ReindeerDeclarationReconcilerV1<G, P> {
    /// Binds effect ports to one qualified maintained-parser profile.
    #[must_use]
    pub const fn new(generator: G, publisher: P, projection_profile_sha256: DigestV1) -> Self {
        Self {
            generator,
            projector: StarlarkSyntaxProjectionV1::new(projection_profile_sha256),
            publisher,
        }
    }
}

impl<G, P> ReindeerDeclarationReconcilerV1<G, P>
where
    G: for<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>,
    G: DeclarationProviderCapabilityPort<GenerationRequestV1>,
    P: PublicationPort<PublicationRequestV1, PublicationObservationV1, PublicationPortErrorV1>,
    P: PublicationCapabilityPort<PublisherProfileV1>,
{
    /// Reconciles one declarative desired state into a complete typed status.
    pub fn reconcile(&self, desired: &ReconciliationRequestV1) -> ReconciliationResultV1 {
        reconcile(
            desired,
            &self.generator,
            &self.projector,
            &self.publisher,
        )
    }
}
