//! Invocation settlement status and the trace-observer seam.

use crate::*;

use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvocationSettlementStatus {
    NotApplicable,
    Completed,
    Failed,
}

impl InvocationSettlementStatus {
    pub(crate) fn as_completion_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }

    pub(crate) fn as_release_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Completed => "released",
            Self::Failed => "failed",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvocationDataUseDenial {
    pub(crate) effective_purpose: Purpose,
    pub(crate) denied_data_class: Option<DataClass>,
    pub(crate) reason: &'static str,
}

#[derive(Clone)]
pub struct FoundationObservability {
    pub(crate) invocation_trace_observer: Arc<dyn CapabilityInvocationTraceObserver>,
}

impl FoundationObservability {
    pub(crate) fn new(observer: impl CapabilityInvocationTraceObserver + 'static) -> Self {
        Self {
            invocation_trace_observer: Arc::new(observer),
        }
    }

    pub(crate) fn start_capability_invocation(
        &self,
        context: &CapabilityInvocationTraceContext,
    ) -> Box<dyn CapabilityInvocationTraceSpan> {
        self.invocation_trace_observer
            .start_capability_invocation(context)
    }
}

impl Default for FoundationObservability {
    fn default() -> Self {
        Self::new(NoopCapabilityInvocationTraceObserver)
    }
}

impl fmt::Debug for FoundationObservability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FoundationObservability")
            .field("invocation_trace_observer", &self.invocation_trace_observer)
            .finish()
    }
}
