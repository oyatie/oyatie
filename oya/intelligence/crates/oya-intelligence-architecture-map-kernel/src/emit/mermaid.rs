//! Mermaid `flowchart LR` emitter.

use super::sanitize_id;
use crate::{ArchitectureMap, EdgeKind};

pub struct Emitter;

impl Emitter {
    pub fn render(map: &ArchitectureMap) -> String {
        let mut out = String::from("flowchart LR\n");

        // Nodes (sorted by id for stable output).
        let mut nodes: Vec<_> = map.nodes().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        for n in &nodes {
            out.push_str("  ");
            out.push_str(&sanitize_id(&n.id.0));
            out.push_str("[\"");
            out.push_str(n.kind.name());
            out.push_str(": ");
            out.push_str(&escape_label(&n.label));
            out.push_str("\"]\n");
        }

        // Edges (insertion order).
        for e in map.edges() {
            out.push_str("  ");
            out.push_str(&sanitize_id(&e.source.0));
            out.push_str(" -->");
            out.push_str(arrow_suffix(e.kind));
            out.push(' ');
            out.push_str(&sanitize_id(&e.target.0));
            out.push('\n');
        }
        out
    }
}

fn escape_label(s: &str) -> String {
    s.replace('"', "&quot;")
}

fn arrow_suffix(kind: EdgeKind) -> &'static str {
    // Mermaid edge labels: `--label-->` would clutter for these short edges;
    // we use the trailing `|label|` form for clarity on hover.
    match kind {
        EdgeKind::Contains => "|contains|",
        EdgeKind::Exposes => "|exposes|",
        EdgeKind::Governs => "|governs|",
        EdgeKind::DependsOn => "|depends-on|",
        EdgeKind::Enforces => "|enforces|",
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
    fn header_is_flowchart_lr() {
        assert!(Emitter::render(&ArchitectureMap::new()).starts_with("flowchart LR\n"));
    }

    #[test]
    fn empty_map_renders_just_header() {
        assert_eq!(Emitter::render(&ArchitectureMap::new()), "flowchart LR\n");
    }

    #[test]
    fn node_line_contains_kind_and_label() {
        let out = Emitter::render(&populated());
        assert!(out.contains("ops[\"microservice: ops\"]"));
        assert!(out.contains("ops_docs_portal[\"bounded-context: ops/docs-portal\"]"));
    }

    #[test]
    fn edge_line_includes_label() {
        let out = Emitter::render(&populated());
        assert!(out.contains("ops -->|contains| ops_docs_portal"));
    }

    #[test]
    fn quotes_in_label_are_escaped() {
        let mut m = ArchitectureMap::new();
        m.add_node(Node {
            id: NodeId("a".into()),
            kind: NodeKind::Crate,
            label: "with \"quote\"".into(),
            owning_team: None,
        })
        .unwrap();
        let out = Emitter::render(&m);
        assert!(out.contains("&quot;quote&quot;"));
        assert!(!out.contains("with \"quote\""));
    }

    #[test]
    fn nodes_emit_in_sorted_order() {
        let mut m = ArchitectureMap::new();
        m.add_node(node("z", NodeKind::Crate)).unwrap();
        m.add_node(node("a", NodeKind::Crate)).unwrap();
        let out = Emitter::render(&m);
        let pos_a = out.find("a[\"crate: a\"]").unwrap();
        let pos_z = out.find("z[\"crate: z\"]").unwrap();
        assert!(pos_a < pos_z);
    }
}
