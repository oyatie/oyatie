use std::sync::Mutex;

use dependency_declarations_generation::{
    DeclarationConsumerCapabilityPort, DeclarationConsumerQualificationPort,
};
use dependency_declarations_reconcile::*;

pub struct FixedBuckConsumer {
    supported: bool,
    error: Option<BuckConsumerPortErrorV1>,
    replay: bool,
    observation: Mutex<Option<BuckConsumerQualificationObservationV1>>,
    calls: Mutex<usize>,
}

impl FixedBuckConsumer {
    pub fn new() -> Self {
        Self {
            supported: true,
            error: None,
            replay: false,
            observation: Mutex::new(None),
            calls: Mutex::new(0),
        }
    }

    pub fn unsupported() -> Self {
        Self {
            supported: false,
            ..Self::new()
        }
    }

    pub fn failing(error: BuckConsumerPortErrorV1) -> Self {
        Self {
            error: Some(error),
            ..Self::new()
        }
    }

    pub fn replaying() -> Self {
        Self {
            replay: true,
            ..Self::new()
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl Default for FixedBuckConsumer {
    fn default() -> Self {
        Self::new()
    }
}

impl DeclarationConsumerCapabilityPort<BuckConsumerProfileV1> for FixedBuckConsumer {
    fn supports(&self, _profile: &BuckConsumerProfileV1) -> bool {
        self.supported
    }
}

impl<'a>
    DeclarationConsumerQualificationPort<
        BuckConsumerQualificationInvocationV1<'a>,
        BuckConsumerQualificationObservationV1,
        BuckConsumerPortErrorV1,
    > for FixedBuckConsumer
{
    fn qualify(
        &self,
        invocation: &BuckConsumerQualificationInvocationV1<'a>,
    ) -> Result<BuckConsumerQualificationObservationV1, BuckConsumerPortErrorV1> {
        let call = {
            let mut calls = self.calls.lock().unwrap();
            *calls += 1;
            *calls
        };
        if let Some(error) = self.error {
            return Err(error);
        }
        let mut observation = self.observation.lock().unwrap();
        if self.replay
            && let Some(existing) = observation.as_ref()
        {
            return Ok(existing.clone());
        }
        let value = BuckConsumerQualificationObservationV1::completed(
            invocation,
            DigestV1::of(b"configured non-building query result"),
            DigestV1::of(b"representative consumption result"),
            DigestV1::of(if call == 1 {
                b"first consumer execution receipt"
            } else {
                b"later consumer execution receipt"
            }),
        );
        *observation = Some(value.clone());
        Ok(value)
    }
}
