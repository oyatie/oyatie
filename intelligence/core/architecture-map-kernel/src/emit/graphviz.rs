//! Graphviz DOT emitter.

use super::sanitize_id;
use crate::{ArchitectureMap, EdgeKind};

pub struct Emitter;

impl Emitter {
    pub fn render(map: &ArchitectureMap) -> String {
        let mut out = String::from("digraph ArchitectureMap {\n");
        out.push_str("  rankdir=LR;\n");
        out.push_str("  node [shape=box];\n");

        let mut nodes: Vec<_> = map.nodes().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        for n in &nodes {
            out.push_str("  ");
            out.push_str(&sanitize_id(&n.id.0));
            out.push_str(" [label=\"");
            out.push_str(n.kind.name());
            out.push_str(": ");
            out.push_str(&escape(&n.label));
            out.push_str("\"];\n");
        }

        for e in map.edges() {
            out.push_str("  ");
            out.push_str(&sanitize_id(&e.source.0));
            out.push_str(" -> ");
            out.push_str(&sanitize_id(&e.target.0));
            out.push_str(" [label=\"");
            out.push_str(edge_label(e.kind));
            out.push_str("\"];\n");
        }

        out.push_str("}\n");
        out
    }
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn edge_label(kind: EdgeKind) -> &'static str {
    match kind {
        EdgeKind::Contains => "contains",
        EdgeKind::Exposes => "exposes",
        EdgeKind::Governs => "governs",
        EdgeKind::DependsOn => "depends-on",
        EdgeKind::Enforces => "enforces",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Edge, Node, NodeId, NodeKind};

    fn node(id: &str, kind: NodeKind) -> Node {
        Node {
            id: NodeId(id.into()),
            kind,
            label: id.into(),
            owning_team: None,
        }
    }

    fn populated() -> ArchitectureMap {
        let mut m = ArchitectureMap::new();
        m.add_node(node("ops", NodeKind::Microservice)).unwrap();
        m.add_node(node("ops/docs-portal", NodeKind::BoundedContext))
            .unwrap();
        m.add_edge(Edge {
            source: NodeId("ops".into()),
            target: NodeId("ops/docs-portal".into()),
            kind: EdgeKind::Contains,
        })
        .unwrap();
        m
    }

    #[test]
    fn opens_digraph_and_closes_brace() {
        let out = Emitter::render(&ArchitectureMap::new());
        assert!(out.starts_with("digraph ArchitectureMap {\n"));
        assert!(out.trim_end().ends_with('}'));
    }

    #[test]
    fn rankdir_lr_is_set() {
        let out = Emitter::render(&ArchitectureMap::new());
        assert!(out.contains("rankdir=LR;"));
    }

    #[test]
    fn nodes_use_sanitized_ids() {
        let out = Emitter::render(&populated());
        assert!(out.contains("ops [label=\"microservice: ops\"];"));
        assert!(out.contains("ops_docs_portal [label=\"bounded-context: ops/docs-portal\"];"));
    }

    #[test]
    fn edges_use_sanitized_ids_and_label() {
        let out = Emitter::render(&populated());
        assert!(out.contains("ops -> ops_docs_portal [label=\"contains\"];"));
    }

    #[test]
    fn backslash_in_label_escaped() {
        let mut m = ArchitectureMap::new();
        m.add_node(Node {
            id: NodeId("a".into()),
            kind: NodeKind::Crate,
            label: "back\\slash".into(),
            owning_team: None,
        })
        .unwrap();
        let out = Emitter::render(&m);
        assert!(out.contains("back\\\\slash"));
    }

    #[test]
    fn quotes_in_label_escaped() {
        let mut m = ArchitectureMap::new();
        m.add_node(Node {
            id: NodeId("a".into()),
            kind: NodeKind::Crate,
            label: "\"q\"".into(),
            owning_team: None,
        })
        .unwrap();
        let out = Emitter::render(&m);
        assert!(out.contains("\\\"q\\\""));
    }

    #[test]
    fn nodes_sorted_in_output() {
        let mut m = ArchitectureMap::new();
        m.add_node(node("z", NodeKind::Crate)).unwrap();
        m.add_node(node("a", NodeKind::Crate)).unwrap();
        let out = Emitter::render(&m);
        let pos_a = out.find("a [label").unwrap();
        let pos_z = out.find("z [label").unwrap();
        assert!(pos_a < pos_z);
    }
}
