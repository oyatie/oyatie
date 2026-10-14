pub(super) const SCRIPT: &str = r#"
function decodeOntologyRelationships(graph, request, link) {
    const shape = (value, keys) => value && typeof value === 'object' && !Array.isArray(value) && Object.keys(value).length === keys.length && keys.every(key => Object.hasOwn(value,key));
    const id = (value, prefix) => typeof value === 'string' && value.startsWith(prefix) && value.length > prefix.length && new TextEncoder().encode(value).length <= 256 && value.trim() === value && !/[\u0000-\u001f\u007f-\u009f]/.test(value);
    const reject = () => { throw new Error('source_unavailable'); };
    if (!shape(graph,['nodes','edges','observed_at_epoch_seconds','truncated']) || !Array.isArray(graph.nodes) || !Array.isArray(graph.edges)
      || graph.nodes.length < 1 || graph.nodes.length > 1000 || graph.edges.length > 5000 || typeof graph.truncated !== 'boolean'
      || graph.observed_at_epoch_seconds !== request.observed_at_epoch_seconds) reject();
    const nodes = new Map();
    for (const node of graph.nodes) {
      if (!shape(node,['entity_id','entity_type_id','written_revision']) || !id(node.entity_id,'ent_') || !id(node.entity_type_id,'ety_')
        || !Number.isInteger(node.written_revision) || node.written_revision < 0 || node.written_revision > 4294967295 || nodes.has(node.entity_id)) reject();
      nodes.set(node.entity_id,node);
    }
    if (nodes.get(request.root.object_ref)?.entity_type_id !== request.root.entity_type) reject();
    const reached = new Set([request.root.object_ref]), edges = new Set();
    for (const edge of graph.edges) {
      const from = nodes.get(edge?.from_entity_id), to = nodes.get(edge?.to_entity_id);
      if (!shape(edge,['from_entity_id','to_entity_id','edge_type_id']) || edge.edge_type_id !== request.link_type || !from || !to
        || from.entity_type_id !== link.from_entity_type || to.entity_type_id !== link.to_entity_type
        || (request.direction === 'outbound' ? edge.from_entity_id : edge.to_entity_id) !== request.root.object_ref) reject();
      const key = JSON.stringify([edge.from_entity_id,edge.to_entity_id,edge.edge_type_id]); if (edges.has(key)) reject(); edges.add(key);
      reached.add(edge.from_entity_id); reached.add(edge.to_entity_id);
    }
    if (reached.size !== nodes.size) reject();
    return nodes;
  }
"#;
