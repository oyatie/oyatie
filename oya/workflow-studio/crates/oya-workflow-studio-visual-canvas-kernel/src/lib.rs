//! Workflow Studio visual canvas kernel.
//!
//! Owns pure graph validation before Workflow Studio emits the canonical
//! `workflow_spec.v1` contract. This crate deliberately has no persistence,
//! network, clock, or provider dependencies.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

const CANVAS_SCHEMA_VERSION: &str = "workflow_studio.canvas.v1";

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CanvasNodeKind {
    Http,
    Transform,
    Branch,
    Join,
    CapabilityCall,
    HumanReview,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanvasNode {
    pub id: String,           // data_class: INTERNAL_ONLY
    pub label: String,        // data_class: INTERNAL_ONLY
    pub kind: CanvasNodeKind, // data_class: PUBLIC
    pub position_x: i32,      // data_class: INTERNAL_ONLY
    pub position_y: i32,      // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CanvasEdge {
    pub id: String,           // data_class: INTERNAL_ONLY
    pub from_node_id: String, // data_class: INTERNAL_ONLY
    pub to_node_id: String,   // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VisualCanvas {
    pub schema_version: String, // data_class: INTERNAL_ONLY
    pub nodes: Vec<CanvasNode>, // data_class: INTERNAL_ONLY
    pub edges: Vec<CanvasEdge>, // data_class: INTERNAL_ONLY
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanvasValidationError {
    InvalidSchemaVersion,
    EmptyNodeSet,
    EmptyNodeId,
    EmptyNodeLabel,
    DuplicateNodeId(String),
    EmptyEdgeId,
    DuplicateEdgeId(String),
    SelfLoop(String),
    DanglingEdgeSource(String),
    DanglingEdgeTarget(String),
}

impl CanvasNode {
    pub fn new(
        id: impl Into<String>,
        label: impl Into<String>,
        kind: CanvasNodeKind,
        position_x: i32,
        position_y: i32,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            kind,
            position_x,
            position_y,
        }
    }
}

impl CanvasEdge {
    pub fn new(
        id: impl Into<String>,
        from_node_id: impl Into<String>,
        to_node_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            from_node_id: from_node_id.into(),
            to_node_id: to_node_id.into(),
        }
    }
}

impl VisualCanvas {
    pub fn new(nodes: Vec<CanvasNode>, edges: Vec<CanvasEdge>) -> Self {
        Self {
            schema_version: CANVAS_SCHEMA_VERSION.to_string(),
            nodes,
            edges,
        }
    }

    pub fn schema_version() -> &'static str {
        CANVAS_SCHEMA_VERSION
    }

    pub fn validate(&self) -> Result<(), CanvasValidationError> {
        if self.schema_version != CANVAS_SCHEMA_VERSION {
            return Err(CanvasValidationError::InvalidSchemaVersion);
        }
        if self.nodes.is_empty() {
            return Err(CanvasValidationError::EmptyNodeSet);
        }

        let mut node_ids = BTreeSet::new();
        for node in &self.nodes {
            if node.id.trim().is_empty() {
                return Err(CanvasValidationError::EmptyNodeId);
            }
            if node.label.trim().is_empty() {
                return Err(CanvasValidationError::EmptyNodeLabel);
            }
            if !node_ids.insert(node.id.clone()) {
                return Err(CanvasValidationError::DuplicateNodeId(node.id.clone()));
            }
        }

        let mut edge_ids = BTreeSet::new();
        for edge in &self.edges {
            if edge.id.trim().is_empty() {
                return Err(CanvasValidationError::EmptyEdgeId);
            }
            if !edge_ids.insert(edge.id.clone()) {
                return Err(CanvasValidationError::DuplicateEdgeId(edge.id.clone()));
            }
            if edge.from_node_id == edge.to_node_id {
                return Err(CanvasValidationError::SelfLoop(edge.id.clone()));
            }
            if !node_ids.contains(&edge.from_node_id) {
                return Err(CanvasValidationError::DanglingEdgeSource(edge.id.clone()));
            }
            if !node_ids.contains(&edge.to_node_id) {
                return Err(CanvasValidationError::DanglingEdgeTarget(edge.id.clone()));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_canvas() -> VisualCanvas {
        VisualCanvas::new(
            vec![
                CanvasNode::new("node_start", "Start", CanvasNodeKind::Http, 0, 0),
                CanvasNode::new("node_prepare", "Prepare", CanvasNodeKind::Transform, 160, 0),
                CanvasNode::new(
                    "node_review",
                    "Approve",
                    CanvasNodeKind::HumanReview,
                    320,
                    0,
                ),
            ],
            vec![
                CanvasEdge::new("edge_start_prepare", "node_start", "node_prepare"),
                CanvasEdge::new("edge_prepare_review", "node_prepare", "node_review"),
            ],
        )
    }

    #[test]
    fn validates_well_formed_canvas() {
        assert_eq!(valid_canvas().validate(), Ok(()));
    }

    #[test]
    fn rejects_duplicate_nodes() {
        let canvas = VisualCanvas::new(
            vec![
                CanvasNode::new("node_start", "Start", CanvasNodeKind::Http, 0, 0),
                CanvasNode::new("node_start", "Again", CanvasNodeKind::Transform, 160, 0),
            ],
            Vec::new(),
        );

        assert_eq!(
            canvas.validate(),
            Err(CanvasValidationError::DuplicateNodeId(
                "node_start".to_string()
            ))
        );
    }

    #[test]
    fn rejects_dangling_edge_target() {
        let canvas = VisualCanvas::new(
            vec![CanvasNode::new(
                "node_start",
                "Start",
                CanvasNodeKind::Http,
                0,
                0,
            )],
            vec![CanvasEdge::new(
                "edge_missing",
                "node_start",
                "node_missing",
            )],
        );

        assert_eq!(
            canvas.validate(),
            Err(CanvasValidationError::DanglingEdgeTarget(
                "edge_missing".to_string()
            ))
        );
    }

    #[test]
    fn rejects_self_loop_edges() {
        let canvas = VisualCanvas::new(
            vec![CanvasNode::new(
                "node_start",
                "Start",
                CanvasNodeKind::Http,
                0,
                0,
            )],
            vec![CanvasEdge::new("edge_self", "node_start", "node_start")],
        );

        assert_eq!(
            canvas.validate(),
            Err(CanvasValidationError::SelfLoop("edge_self".to_string()))
        );
    }

    #[test]
    fn serde_roundtrip_preserves_canvas_contract() {
        let json = serde_json::to_string(&valid_canvas()).unwrap();
        let decoded: VisualCanvas = serde_json::from_str(&json).unwrap();

        assert_eq!(decoded, valid_canvas());
        assert_eq!(decoded.validate(), Ok(()));
    }
}
