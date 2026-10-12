use super::*;

pub(super) fn workflow_display_nodes(nodes: &[WorkflowNode]) -> Vec<WorkflowNode> {
    let mut display_nodes = nodes.to_vec();
    for (id, label, kind, x, y, explanation) in [
        (
            "finance-review",
            "Finance sign-off",
            "Human",
            250,
            190,
            "Adds a finance owner checkpoint before any costly tenant operation proceeds.",
        ),
        (
            "mail-notice",
            "Mail approval brief",
            "Mail",
            445,
            190,
            "Sample approval mail step; no draft or message is created.",
        ),
        (
            "messenger-ops",
            "Messenger ops update",
            "Messenger",
            640,
            190,
            "Sample operations message step; no post or send occurs.",
        ),
        (
            "community-post",
            "Community notice",
            "Community",
            250,
            300,
            "Sample community update step; no publication occurs.",
        ),
        (
            "receipt",
            "Sample evidence reference",
            "Audit",
            640,
            300,
            "Sample evidence link; no sealed receipt or evidence is issued.",
        ),
    ] {
        if display_nodes.iter().all(|node| node.id != id) {
            display_nodes.push(WorkflowNode {
                id: id.to_string(),
                label: label.to_string(),
                kind: kind.to_string(),
                x,
                y,
                explanation: explanation.to_string(),
            });
        }
    }
    display_nodes
}

pub(super) fn selected_workflow_node<'a>(
    nodes: &'a [WorkflowNode],
    selected_node_id: &str,
) -> Option<&'a WorkflowNode> {
    nodes.iter().find(|node| node.id == selected_node_id)
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(super) struct WorkflowSample {
    pub source: String,
    pub context: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<crate::render_envelope::WorkflowEdge>,
    pub selected_id: String,
    pub base_count: usize,
    pub query: String,
    pub zoom: i32,
    pub outcome: String,
    pub suggestion_dismissed: bool,
    pub suggestion_open: bool,
}

impl WorkflowSample {
    pub(super) fn from_envelope(envelope: &TenantRenderEnvelope) -> Self {
        let nodes = workflow_display_nodes(&envelope.workflow.nodes);
        let mut edges = envelope.workflow.edges.clone();
        for pair in nodes
            .windows(2)
            .skip(envelope.workflow.nodes.len().saturating_sub(1))
        {
            edges.push(crate::render_envelope::WorkflowEdge {
                from: pair[0].id.clone(),
                to: pair[1].id.clone(),
                label: "sample then".into(),
            });
        }
        let source = serde_json::to_string(&(
            envelope.context.id(),
            &envelope.tenant_name,
            &envelope.role_name,
            &envelope.tenant_class,
            &envelope.accreditation,
            &envelope.workflow,
        ))
        .expect("native Workflow sample source is serializable");
        Self {
            source,
            context: envelope.context.id().into(),
            selected_id: nodes
                .first()
                .map(|node| node.id.clone())
                .unwrap_or_default(),
            base_count: nodes.len(),
            nodes,
            edges,
            zoom: 100,
            outcome: "No local validation or path preview performed.".into(),
            ..Self::default()
        }
    }

    pub(super) fn selected(&self) -> Option<&WorkflowNode> {
        selected_workflow_node(&self.nodes, &self.selected_id)
    }

    pub(super) fn add(&mut self, label: &str, kind: &str, detail: &str) {
        let mut ordinal = self.nodes.len().saturating_sub(self.base_count) + 1;
        while self
            .nodes
            .iter()
            .any(|node| node.id == format!("draft-block-{ordinal}"))
        {
            ordinal += 1;
        }
        let id = format!("draft-block-{ordinal}");
        if let Some(previous) = self.nodes.last() {
            self.edges.push(crate::render_envelope::WorkflowEdge {
                from: previous.id.clone(),
                to: id.clone(),
                label: "sample then".into(),
            });
        }
        self.nodes.push(WorkflowNode {
            id: id.clone(),
            label: label.into(),
            kind: kind.into(),
            x: 0,
            y: 0,
            explanation: format!("{detail} Sample description only; no execution or output."),
        });
        self.selected_id = id;
        self.outcome = format!(
            "Added {label} sample block locally; validation is pending. Nothing saved or executed."
        );
    }

    pub(super) fn rename_selected(&mut self, label: String) {
        if let Some(node) = self
            .nodes
            .iter_mut()
            .find(|node| node.id == self.selected_id)
        {
            node.label = label;
            self.outcome =
                "Sample label changed locally; validation is pending. Nothing saved or executed."
                    .into();
        }
    }

    pub(super) fn validate(&mut self) {
        let errors = validate_workflow_sample(&self.nodes, &self.edges);
        self.outcome = if errors.is_empty() {
            format!(
                "Sample structure valid: {} nodes, {} links. No policy, permission, connector, or execution readiness verified.",
                self.nodes.len(),
                self.edges.len()
            )
        } else {
            format!(
                "Sample structure invalid: {}. No live action performed.",
                errors.join("; ")
            )
        };
    }

    pub(super) fn preview_path(&mut self) {
        let errors = validate_workflow_sample(&self.nodes, &self.edges);
        self.outcome = if errors.is_empty() {
            format!(
                "Sample list order ({} nodes): {}. No run, approval, draft, or receipt created.",
                self.nodes.len(),
                self.nodes
                    .iter()
                    .map(|node| node.label.as_str())
                    .collect::<Vec<_>>()
                    .join(" → ")
            )
        } else {
            format!(
                "Sample path unavailable: {}. No live action performed.",
                errors.join("; ")
            )
        };
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    // A handoff snapshot of the one editor model, cleared by a changed native source or page reload.
    static WORKFLOW_HANDOFF: std::cell::RefCell<Option<WorkflowSample>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn workflow_initial_context() -> OperatorContext {
    #[cfg(target_arch = "wasm32")]
    if let Some(context) = WORKFLOW_HANDOFF.with(|saved| {
        saved
            .borrow()
            .as_ref()
            .and_then(|sample| OperatorContext::from_id(&sample.context))
    }) {
        return context;
    }
    OperatorContext::TenantAdmin
}

pub(super) fn workflow_sample_for_envelope(
    envelope: &TenantRenderEnvelope,
    current: &WorkflowSample,
) -> WorkflowSample {
    let fresh = WorkflowSample::from_envelope(envelope);
    if current.source == fresh.source {
        return current.clone();
    }
    #[cfg(target_arch = "wasm32")]
    return WORKFLOW_HANDOFF.with(|saved| {
        let matching = saved
            .borrow()
            .as_ref()
            .filter(|sample| sample.source == fresh.source)
            .cloned();
        let next = matching.unwrap_or(fresh);
        *saved.borrow_mut() = Some(next.clone());
        next
    });
    #[cfg(not(target_arch = "wasm32"))]
    fresh
}

pub(super) fn edit_workflow_sample(
    sample: RwSignal<WorkflowSample>,
    edit: impl FnOnce(&mut WorkflowSample),
) {
    sample.update(edit);
    #[cfg(target_arch = "wasm32")]
    WORKFLOW_HANDOFF.with(|saved| *saved.borrow_mut() = Some(sample.get_untracked()));
}
