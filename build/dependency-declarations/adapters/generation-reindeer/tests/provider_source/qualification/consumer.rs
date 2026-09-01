use dependency_declarations_generation::{
    DeclarationConsumerCapabilityPort, DeclarationConsumerQualificationPort,
};
use dependency_declarations_reconcile::{
    BuckConsumerPortErrorV1, BuckConsumerProfileV1, BuckConsumerQualificationInvocationV1,
    BuckConsumerQualificationObservationV1, DigestV1,
};

pub(super) struct FixtureBuckConsumer;

pub(super) fn profile() -> BuckConsumerProfileV1 {
    BuckConsumerProfileV1::try_new(
        super::artifact("buck2", "qualification"),
        super::artifact("buck2-prelude", "qualification"),
        DigestV1::of(b"rules"),
        DigestV1::of(b"toolchain"),
        DigestV1::of(b"cell config"),
        DigestV1::of(b"buck config"),
        DigestV1::of(b"fixture query and consumption plan"),
    )
    .unwrap()
}

impl DeclarationConsumerCapabilityPort<BuckConsumerProfileV1> for FixtureBuckConsumer {
    fn supports(&self, _profile: &BuckConsumerProfileV1) -> bool {
        true
    }
}

impl<'a>
    DeclarationConsumerQualificationPort<
        BuckConsumerQualificationInvocationV1<'a>,
        BuckConsumerQualificationObservationV1,
        BuckConsumerPortErrorV1,
    > for FixtureBuckConsumer
{
    fn qualify(
        &self,
        invocation: &BuckConsumerQualificationInvocationV1<'a>,
    ) -> Result<BuckConsumerQualificationObservationV1, BuckConsumerPortErrorV1> {
        Ok(BuckConsumerQualificationObservationV1::completed(
            invocation,
            DigestV1::of(b"fixture query receipt"),
            DigestV1::of(b"fixture consumption receipt"),
            DigestV1::of(b"fixture execution receipt"),
        ))
    }
}
