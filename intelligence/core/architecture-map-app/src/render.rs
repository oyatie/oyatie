//! The map's JSON face: rendering it, and the field reader the freshness gate
//! uses to compare a committed map against a fresh one.

use intelligence_architecture_map_kernel::{
    ArchitectureMap, Edge, EdgeKind, Node, NodeId, NodeKind,
};

pub(crate) fn render_json(map: &ArchitectureMap) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    out.push_str("  \"$schema_ref\": \"specs/knowledge-graph-schema.json\",\n");
    out.push_str("  \"_artifact_id\": \"architecture-map\",\n");
    out.push_str(
        "  \"_meta\": { \"emitter\": \"intelligence-architecture-map-app::build_map\", \"purpose\": \"Generated architecture graph of crates, contracts, registries, and ownership edges for repository navigation and drift checks.\" },\n",
    );
    out.push_str("  \"nodes\": [\n");
    let nodes: Vec<&Node> = map.nodes().collect();
    for (i, node) in nodes.iter().enumerate() {
        let trailing = if i + 1 == nodes.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"id\": \"{}\", \"kind\": \"{}\", \"label\": \"{}\" }}{}\n",
            escape_json(&node.id.0),
            node.kind.name(),
            escape_json(&node.label),
            trailing
        ));
    }
    out.push_str("  ],\n");
    out.push_str("  \"edges\": [\n");
    let edges = map.edges();
    for (i, edge) in edges.iter().enumerate() {
        let trailing = if i + 1 == edges.len() { "" } else { "," };
        out.push_str(&format!(
            "    {{ \"source\": \"{}\", \"target\": \"{}\", \"kind\": \"{}\" }}{}\n",
            escape_json(&edge.source.0),
            escape_json(&edge.target.0),
            edge.kind.name(),
            trailing
        ));
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Return the string value of `"key": "value"` on this trimmed line, if it
/// matches.
pub(crate) fn json_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\"");
    let after_key = line.strip_prefix(&needle)?.trim_start().strip_prefix(':')?;
    read_json_string(after_key.trim_start())
}

/// Read a JSON string starting at the leading `"`. Strips trailing `,` if any.
pub(crate) fn read_json_string(input: &str) -> Option<String> {
    let after_quote = input.strip_prefix('"')?;
    let close = after_quote.find('"')?;
    Some(after_quote[..close].to_string())
}

pub(crate) fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_json_round_trip_shape() {
        let mut map = ArchitectureMap::new();
        map.add_node(Node {
            id: NodeId("a".into()),
            kind: NodeKind::Microservice,
            label: "a".into(),
            owning_team: None,
        })
        .unwrap();
        map.add_node(Node {
            id: NodeId("a/b".into()),
            kind: NodeKind::BoundedContext,
            label: "a/b".into(),
            owning_team: None,
        })
        .unwrap();
        map.add_edge(Edge {
            source: NodeId("a".into()),
            target: NodeId("a/b".into()),
            kind: EdgeKind::Contains,
        })
        .unwrap();
        let body = render_json(&map);
        assert!(body.contains("\"_artifact_id\": \"architecture-map\""));
        assert!(body.contains("\"id\": \"a\""));
        assert!(body.contains("\"kind\": \"microservice\""));
        assert!(body.contains("\"kind\": \"bounded-context\""));
        assert!(body.contains("\"source\": \"a\""));
        assert!(body.contains("\"target\": \"a/b\""));
        assert!(body.contains("\"kind\": \"contains\""));
    }

    #[test]
    fn json_field_extracts_string_value() {
        assert_eq!(
            json_field(r#""foo": "bar","#, "foo"),
            Some("bar".to_string())
        );
        assert_eq!(
            json_field(r#""foo": "bar""#, "foo"),
            Some("bar".to_string())
        );
        assert!(json_field(r#""other": "bar""#, "foo").is_none());
    }
}
