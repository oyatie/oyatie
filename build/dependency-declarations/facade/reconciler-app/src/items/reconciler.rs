use dependency_declarations_generation::{
    DeclarationConsumerCapabilityPort, DeclarationConsumerQualificationPort,
    DeclarationProviderCapabilityPort, GenerationPort, RenderedDeclarationProjectionPort,
};
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::{
    BuckConsumerPortErrorV1, BuckConsumerProfileV1, BuckConsumerQualificationInvocationV1,
    BuckConsumerQualificationObservationV1, GenerationInvocationV1, GenerationPortErrorV1,
    GenerationRequestV1, ParsedBuckProjectionV1, ProjectionPortErrorV1,
    PublicationObservationV1, PublicationPortErrorV1, PublicationRequestV1, PublisherProfileV1,
    RawGenerationV1, reconcile,
};

/// Versioned reconciler over independent declaration effects.
pub struct ReindeerDeclarationReconcilerV1<G, V, Q, P> {
    generator: G,
    projector: V,
    consumer: Q,
    publisher: P,
}

impl<G, V, Q, P> ReindeerDeclarationReconcilerV1<G, V, Q, P> {
    /// Binds generator, maintained parser, configured consumer, and publisher.
    #[must_use]
    pub const fn new(
        generator: G,
        projector: V,
        consumer: Q,
        publisher: P,
    ) -> Self {
        Self {
            generator,
            projector,
            consumer,
            publisher,
        }
    }
}

impl<G, V, Q, P> ReindeerDeclarationReconcilerV1<G, V, Q, P>
where
    G: for<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>,
    G: DeclarationProviderCapabilityPort<GenerationRequestV1>,
    V: RenderedDeclarationProjectionPort<
            Profile = DigestV1,
            Projection = ParsedBuckProjectionV1,
            Error = ProjectionPortErrorV1,
        >,
    Q: for<'a> DeclarationConsumerQualificationPort<
            BuckConsumerQualificationInvocationV1<'a>,
            BuckConsumerQualificationObservationV1,
            BuckConsumerPortErrorV1,
        >,
    Q: DeclarationConsumerCapabilityPort<BuckConsumerProfileV1>,
    P: PublicationPort<PublicationRequestV1, PublicationObservationV1, PublicationPortErrorV1>,
    P: PublicationCapabilityPort<PublisherProfileV1>,
{
    /// Reconciles one declarative desired state into a complete typed status.
    pub fn reconcile(&self, desired: &ReconciliationRequestV1) -> ReconciliationResultV1 {
        reconcile(
            desired,
            &self.generator,
            &self.projector,
            &self.consumer,
            &self.publisher,
        )
    }
}
