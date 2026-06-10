#[cfg(any(feature = "ssr", test))]
use crate::render_envelope::server_derived_envelope;
use crate::render_envelope::{OperatorContext, TenantRenderEnvelope};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClientSessionState {
    pub active_context: OperatorContext,
    pub active_surface: Surface,
    pub selected_workflow_node_id: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Surface {
    Home,
    Modules,
    WorkflowStudio,
}

impl Surface {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Modules => "Modules",
            Self::WorkflowStudio => "Workflow Studio",
        }
    }
}

impl ClientSessionState {
    pub fn hydrated_from_server_envelope(envelope: &TenantRenderEnvelope) -> Self {
        let selected_workflow_node_id = envelope
            .workflow
            .nodes
            .first()
            .map(|node| node.id.clone())
            .unwrap_or_default();

        Self {
            active_context: envelope.context,
            active_surface: Surface::Home,
            selected_workflow_node_id,
        }
    }

    #[cfg(any(feature = "ssr", test))]
    pub fn fresh_envelope(&self) -> TenantRenderEnvelope {
        server_derived_envelope(self.active_context)
    }
}

#[cfg(test)]
mod tests {
    use super::{ClientSessionState, Surface};
    use crate::render_envelope::OperatorContext;

    #[test]
    fn client_state_stores_active_context_not_catalog() {
        let corporate_envelope =
            crate::render_envelope::server_derived_envelope(OperatorContext::CorporateOffice);
        let state = ClientSessionState::hydrated_from_server_envelope(&corporate_envelope);
        let envelope = state.fresh_envelope();

        assert_eq!(state.active_context, OperatorContext::CorporateOffice);
        assert_eq!(state.active_surface, Surface::Home);
        assert!(
            envelope
                .modules
                .iter()
                .all(|module| module.name != "Clinical Home")
        );
    }
}
