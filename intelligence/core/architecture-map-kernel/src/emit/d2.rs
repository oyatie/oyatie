//! [D2](https://d2lang.com) emitter.

use crate::{ArchitectureMap, EdgeKind};

pub struct Emitter;

impl Emitter {
    pub fn render(map: &ArchitectureMap) -> String {
        let mut out = String::new();

        let mut nodes: Vec<_> = map.nodes().collect();
        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        for n in &nodes {
            // D2 supports quoted identifiers; we use them so node ids
            // with slashes / dots stay readable.
            out.push('"');
            out.push_str(&n.id.0);
            out.push_str("\": {\n");
            out.push_str("  label: \"");
            out.push_str(&escape(&n.label));
            out.push_str("\"\n");
            out.push_str("  shape: rectangle\n");
            out.push_str("  class: ");
            out.push_str(n.kind.name());
            out.push('\n');
            out.push_str("}\n");
        }

        for e in map.edges() {
            out.push('"');
            out.push_str(&e.source.0);
            out.push_str("\" -> \"");
            out.push_str(&e.target.0);
            out.push_str("\": ");
            out.push_str(edge_label(e.kind));
            out.push('\n');
        }
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
    fn empty_map_renders_empty_string() {
        assert_eq!(Emitter::render(&ArchitectureMap::new()), "");
    }

    #[test]
    fn node_block_has_label_shape_class() {
        let out = Emitter::render(&populated());
        assert!(out.contains("\"ops\": {"));
        assert!(out.contains("label: \"ops\""));
        assert!(out.contains("shape: rectangle"));
        assert!(out.contains("class: microservice"));
    }

    #[test]
    fn edge_line_has_label() {
        let out = Emitter::render(&populated());
        assert!(out.contains("\"ops\" -> \"ops/docs-portal\": contains"));
    }

    #[test]
    fn backslash_in_label_is_escaped() {
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
    fn quotes_in_label_are_escaped() {
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
        let pos_a = out.find("\"a\": {").unwrap();
        let pos_z = out.find("\"z\": {").unwrap();
        assert!(pos_a < pos_z);
    }
}
