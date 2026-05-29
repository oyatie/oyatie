//! Workflow Studio editor action primitives — P07 merge-variant delta-1.
//!
//! `EditorActionKind` is the minimal closed-set of user-initiated actions the
//! visual canvas editor can dispatch to the kernel.  Per ADR-0023 (plugin
//! sandbox) this kernel takes no UI / WASM dependencies; the canvas SDK
//! (`oya-workflow-leptos-canvas`) depends inward on these types, not vice
//! versa.
//!
//! `NodeKind` mirrors `workflow.node_kind` from the P07 DDL so the kernel and
//! DB schema share one canonical closed-set without requiring a DB dep here.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Actions a Workflow Studio editor user can dispatch on the canvas.
///
/// Closed enum — exhaustive match required; add variants only when the P07
/// DDL or canvas SDK spec is updated in lockstep.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum EditorActionKind {
    /// Add a new node to the canvas at the given position.
    AddNode,
    /// Remove an existing node and all its incident edges.
    RemoveNode,
    /// two nodes with a directed edge.
    ConnectNodes,
    /// Disconnect (remove) an existing edge between two nodes.
    DisconnectNodes,
    /// Move a node to a new position without changing topology.
    MoveNode,
    /// Open the property panel for a node or edge.
    OpenProperties,
    /// Undo the most recent reversible action.
    Undo,
    /// Redo the most recently undone action.
    Redo,
    /// Persist the current canvas state as a draft definition.
    SaveDraft,
    /// Publish the current draft definition (makes it runnable).
    Publish,
}

impl EditorActionKind {
    /// Returns `true` when the action mutates canvas topology (nodes/edges).
    ///
    /// `Undo` and `Redo` are included because they can replay or revert
    /// `AddNode`/`RemoveNode`/`ConnectNodes`/`DisconnectNodes` actions,
    /// making them topology-mutating in effect.
    pub fn is_topology_mutation(self) -> bool {
        matches!(
            self,
            Self::AddNode
                | Self::RemoveNode
                | Self::ConnectNodes
                | Self::DisconnectNodes
                | Self::Undo
                | Self::Redo
        )
    }

    /// Returns `true` when the action is reversible via Undo/Redo.
    ///
    /// `Undo` and `Redo` are themselves reversible: `Undo` can be undone by
    /// `Redo` and vice-versa, so they belong in the reversible set.
    pub fn is_reversible(self) -> bool {
        matches!(
            self,
            Self::AddNode
                | Self::RemoveNode
                | Self::ConnectNodes
                | Self::DisconnectNodes
                | Self::MoveNode
                | Self::Undo
                | Self::Redo
        )
    }
}

/// Node kinds supported by the Workflow Studio canvas, mirroring
/// `workflow.node_kind` in the P07 DDL.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NodeKind {
    /// Entry-point node — event, schedule, webhook, or manual trigger.
    Trigger,
    /// Executes a plugin action step.
    Action,
    /// Conditional branch (if/else or switch).
    Condition,
    /// Human-in-the-loop approval gate.
    Approval,
    /// Delay / SLA timer.
    Timer,
    /// LLM / AI agent node dispatched through the agentic runtime.
    Agentic,
    /// Third-party integration node (OAuth, REST, GraphQL).
    Integration,
    /// Nested sub-workflow invocation.
    SubWorkflow,
}

impl NodeKind {
    /// Returns `true` when the node kind requires a human actor to proceed.
    pub fn requires_human(self) -> bool {
        matches!(self, Self::Approval)
    }

    /// Returns `true` when the node kind introduces a time delay.
    pub fn introduces_delay(self) -> bool {
        matches!(self, Self::Timer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn editor_action_topology_mutations_cover_add_remove_connect_disconnect() {
        assert!(EditorActionKind::AddNode.is_topology_mutation());
        assert!(EditorActionKind::RemoveNode.is_topology_mutation());
        assert!(EditorActionKind::ConnectNodes.is_topology_mutation());
        assert!(EditorActionKind::DisconnectNodes.is_topology_mutation());
        assert!(!EditorActionKind::MoveNode.is_topology_mutation());
        assert!(EditorActionKind::Undo.is_topology_mutation());
        assert!(EditorActionKind::Redo.is_topology_mutation());
        assert!(!EditorActionKind::SaveDraft.is_topology_mutation());
        assert!(!EditorActionKind::Publish.is_topology_mutation());
    }

    #[test]
    fn editor_action_reversible_set_includes_topology_and_move() {
        assert!(EditorActionKind::AddNode.is_reversible());
        assert!(EditorActionKind::RemoveNode.is_reversible());
        assert!(EditorActionKind::ConnectNodes.is_reversible());
        assert!(EditorActionKind::DisconnectNodes.is_reversible());
        assert!(EditorActionKind::MoveNode.is_reversible());
        assert!(!EditorActionKind::OpenProperties.is_reversible());
        assert!(EditorActionKind::Undo.is_reversible());
        assert!(EditorActionKind::Redo.is_reversible());
        assert!(!EditorActionKind::SaveDraft.is_reversible());
        assert!(!EditorActionKind::Publish.is_reversible());
    }

    #[test]
    fn node_kind_requires_human_only_for_approval() {
        assert!(NodeKind::Approval.requires_human());
        assert!(!NodeKind::Action.requires_human());
        assert!(!NodeKind::Agentic.requires_human());
        assert!(!NodeKind::Trigger.requires_human());
    }

    #[test]
    fn node_kind_introduces_delay_only_for_timer() {
        assert!(NodeKind::Timer.introduces_delay());
        assert!(!NodeKind::Action.introduces_delay());
        assert!(!NodeKind::Condition.introduces_delay());
        assert!(!NodeKind::SubWorkflow.introduces_delay());
        assert!(!NodeKind::Integration.introduces_delay());
    }

    #[test]
    fn all_editor_action_variants_are_distinct() {
        let all = [
            EditorActionKind::AddNode,
            EditorActionKind::RemoveNode,
            EditorActionKind::ConnectNodes,
            EditorActionKind::DisconnectNodes,
            EditorActionKind::MoveNode,
            EditorActionKind::OpenProperties,
            EditorActionKind::Undo,
            EditorActionKind::Redo,
            EditorActionKind::SaveDraft,
            EditorActionKind::Publish,
        ];
        // ensure Ord/PartialOrd are consistent — no two variants compare equal
        for i in 0..all.len() {
            for j in 0..all.len() {
                if i == j {
                    assert_eq!(all[i], all[j]);
                } else {
                    assert_ne!(all[i], all[j]);
                }
            }
        }
    }

    #[test]
    fn all_node_kind_variants_are_distinct() {
        let all = [
            NodeKind::Trigger,
            NodeKind::Action,
            NodeKind::Condition,
            NodeKind::Approval,
            NodeKind::Timer,
            NodeKind::Agentic,
            NodeKind::Integration,
            NodeKind::SubWorkflow,
        ];
        for i in 0..all.len() {
            for j in 0..all.len() {
                if i == j {
                    assert_eq!(all[i], all[j]);
                } else {
                    assert_ne!(all[i], all[j]);
                }
            }
        }
    }
}
