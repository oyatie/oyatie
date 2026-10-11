use super::*;

pub(super) fn workflow_display_nodes(
    nodes: &[WorkflowNode],
    draft_node_count: usize,
) -> Vec<WorkflowNode> {
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
            "Drafts the formal approval mail with policy, cost, and audit receipt context.",
        ),
        (
            "messenger-ops",
            "Messenger ops update",
            "Messenger",
            640,
            190,
            "Posts a local-only operational summary into the governed Ops room preview.",
        ),
        (
            "community-post",
            "Community notice",
            "Community",
            250,
            300,
            "Stages a community update for the governance council with reviewer trace links.",
        ),
        (
            "receipt",
            "Immutable receipt",
            "Audit",
            640,
            300,
            "Seals a visual receipt preview that can be inspected from the evidence spine.",
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
    for index in 0..draft_node_count {
        display_nodes.push(WorkflowNode {
            id: format!("draft-block-{}", index + 1),
            label: format!("Draft block {}", index + 1),
            kind: "Local".to_string(),
            x: 110 + ((index as i32 % 4) * 165),
            y: 164 + ((index as i32 / 4) * 74),
            explanation: "Local visual-only block added in the shell. It is not yet wired to a backend or workflow engine.".to_string(),
        });
    }
    display_nodes
}

pub(super) fn board_edges(nodes: &[WorkflowNode]) -> Vec<(String, String, String)> {
    nodes
        .windows(2)
        .enumerate()
        .map(|(index, pair)| {
            let from = &pair[0];
            let to = &pair[1];
            let start_x = workflow_board_x(index) + 156;
            let start_y = workflow_board_y(index, from) + 36;
            let end_x = workflow_board_x(index + 1);
            let end_y = workflow_board_y(index + 1, to) + 36;
            let control_delta = ((end_x - start_x).abs() / 2).max(64);
            let path = format!(
                "M {start_x} {start_y} C {} {start_y}, {} {end_y}, {end_x} {end_y}",
                start_x + control_delta,
                end_x - control_delta,
            );
            (from.id.clone(), to.id.clone(), path)
        })
        .collect()
}

pub(super) fn workflow_board_x(index: usize) -> i32 {
    55 + ((index as i32 % 4) * 156)
}

pub(super) fn workflow_board_y(index: usize, node: &WorkflowNode) -> i32 {
    if node.id.starts_with("draft-block-") {
        258 + ((index as i32 / 4) * 104)
    } else {
        72 + ((index as i32 / 4) * 112)
    }
}

pub(super) fn selected_workflow_node<'a>(
    nodes: &'a [WorkflowNode],
    selected_node_id: &str,
) -> Option<&'a WorkflowNode> {
    nodes.iter().find(|node| node.id == selected_node_id)
}
