use std::collections::VecDeque;
use std::convert::Infallible;
use std::sync::Mutex;

use dependency_declarations_generation::{
    DeclarationProviderCapabilityPort, GenerationPort, RenderedDeclarationProjectionPort,
};
use dependency_declarations_publication::{PublicationCapabilityPort, PublicationPort};
use dependency_declarations_reconcile::*;

use super::{ProviderArtifactFaultV1, raw_provider_artifact_with_fault};

type ScriptedOutput = Result<(RuleGraphV1, Vec<u8>), GenerationPortErrorV1>;

pub struct ScriptedGenerator {
    outputs: Mutex<VecDeque<ScriptedOutput>>,
    invocations: Mutex<Vec<DigestV1>>,
    supported: bool,
    fault: Option<ProviderArtifactFaultV1>,
    stderr: Vec<u8>,
    observed_reads: Mutex<VecDeque<DigestV1>>,
}

impl ScriptedGenerator {
    pub fn new(outputs: Vec<ScriptedOutput>) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            supported: true,
            fault: None,
            stderr: Vec::new(),
            observed_reads: Mutex::new(VecDeque::new()),
        }
    }

    pub fn unsupported(outputs: Vec<ScriptedOutput>) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            supported: false,
            fault: None,
            stderr: Vec::new(),
            observed_reads: Mutex::new(VecDeque::new()),
        }
    }

    pub fn with_stderr(outputs: Vec<ScriptedOutput>, stderr: Vec<u8>) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            supported: true,
            fault: None,
            stderr,
            observed_reads: Mutex::new(VecDeque::new()),
        }
    }

    pub fn with_fault(outputs: Vec<ScriptedOutput>, fault: ProviderArtifactFaultV1) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            supported: true,
            fault: Some(fault),
            stderr: Vec::new(),
            observed_reads: Mutex::new(VecDeque::new()),
        }
    }

    pub fn with_observed_reads(
        outputs: Vec<ScriptedOutput>,
        observed_reads: Vec<DigestV1>,
    ) -> Self {
        Self {
            outputs: Mutex::new(outputs.into()),
            invocations: Mutex::new(Vec::new()),
            supported: true,
            fault: None,
            stderr: Vec::new(),
            observed_reads: Mutex::new(observed_reads.into()),
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
        let observed_reads = self
            .observed_reads
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| DigestV1::of(b"observed repository and Cargo reads"));
        Ok(raw_provider_artifact_with_fault(
            request,
            &graph,
            bytes,
            self.stderr.clone(),
            observed_reads,
            DigestV1::of(b"observed stage writes"),
            self.fault,
        ))
    }
}

impl DeclarationProviderCapabilityPort<GenerationRequestV1> for ScriptedGenerator {
    fn supports(&self, _profile: &GenerationRequestV1) -> bool {
        self.supported
    }
}

pub struct FixedProjection {
    graph: RenderedRuleGraphV1,
    profile: DigestV1,
    calls: Mutex<usize>,
}

impl FixedProjection {
    pub fn new(graph: RuleGraphV1, profile: DigestV1) -> Self {
        Self {
            graph: graph.rendered_projection().unwrap(),
            profile,
            calls: Mutex::new(0),
        }
    }

    pub fn calls(&self) -> usize {
        *self.calls.lock().unwrap()
    }
}

impl RenderedDeclarationProjectionPort for FixedProjection {
    type Profile = DigestV1;
    type Projection = ParsedBuckProjectionV1;
    type Error = ProjectionPortErrorV1;

    fn profile(&self) -> &Self::Profile {
        &self.profile
    }

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
