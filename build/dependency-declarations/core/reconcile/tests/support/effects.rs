use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Mutex;

use dependency_declarations_generation::{GenerationPort, RenderedDeclarationProjectionPort};
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::*;

use super::{ProviderArtifactFaultV1, raw_provider_artifact_with_fault};

type ScriptedOutput = Result<(RuleGraphV1, Vec<u8>), GenerationPortErrorV1>;

pub struct ScriptedGenerator {
    outputs: Mutex<VecDeque<ScriptedOutput>>,
    invocations: Mutex<Vec<DigestV1>>,
    fault: Option<ProviderArtifactFaultV1>,
    stderr: Vec<u8>,
}

impl ScriptedGenerator {
    pub fn new(outputs: Vec<ScriptedOutput>) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            fault: None,
            stderr: Vec::new(),
        }
    }

    pub fn with_stderr(outputs: Vec<ScriptedOutput>, stderr: Vec<u8>) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            fault: None,
            stderr,
        }
    }

    pub fn with_fault(outputs: Vec<ScriptedOutput>, fault: ProviderArtifactFaultV1) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            fault: Some(fault),
            stderr: Vec::new(),
        }
    }

    pub fn invocations(&self) -> Vec<DigestV1> {
        self.invocations.lock().unwrap().clone()
    }
}

impl<'a> GenerationPort<GenerationInvocationV1<'a>, RawGenerationV1, GenerationPortErrorV1>
    for ScriptedGenerator
{
    fn generate(
        &self,
        request: &GenerationInvocationV1<'a>,
    ) -> Result<RawGenerationV1, GenerationPortErrorV1> {
        self.invocations
            .lock()
            .unwrap()
            .push(request.invocation_id());
        let (graph, bytes) = self.outputs.lock().unwrap().pop_front().unwrap()?;
        Ok(raw_provider_artifact_with_fault(
            request,
            &graph,
            bytes,
            self.stderr.clone(),
            self.fault,
        ))
    }
}

pub struct FixedProjection {
    graph: RuleGraphV1,
    profile: DigestV1,
    calls: Mutex<usize>,
}

impl FixedProjection {
    pub fn new(graph: RuleGraphV1, profile: DigestV1) -> Self {
        Self {
            graph,
            profile,
            calls: Mutex::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl RenderedDeclarationProjectionPort for FixedProjection {
    type Projection = ParsedBuckProjectionV1;
    type Error = ProjectionPortErrorV1;

    fn project(&self, source: &[u8]) -> Result<ParsedBuckProjectionV1, ProjectionPortErrorV1> {
        *self.calls.lock().unwrap() += 1;
        Ok(ParsedBuckProjectionV1::for_projection(
            self.profile,
            self.graph.clone(),
            source,
        ))
    }
}

pub struct RecordingPublisher {
    outcome: PublicationOutcomeV1,
    supported: bool,
    calls: Mutex<usize>,
}

impl RecordingPublisher {
    pub fn new(outcome: PublicationOutcomeV1) -> Self {
        Self {
            outcome,
            supported: true,
            calls: Mutex::new(0),
        }
    }

    pub fn unsupported(outcome: PublicationOutcomeV1) -> Self {
        Self {
            outcome,
            supported: false,
            calls: Mutex::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl PublicationCapabilityPort<PublisherProfileV1> for RecordingPublisher {
    fn supports(&self, _profile: &PublisherProfileV1) -> bool {
        self.supported
    }
}

impl PublicationPort<PublicationRequestV1, PublicationObservationV1, Infallible>
    for RecordingPublisher
{
    fn publish(
        &self,
        _request: &PublicationRequestV1,
    ) -> Result<PublicationObservationV1, Infallible> {
        *self.calls.lock().unwrap() += 1;
        Ok(PublicationObservationV1::new(self.outcome.clone()))
    }
}
