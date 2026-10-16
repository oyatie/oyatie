pub(super) const SCRIPT: &str = r#"
function ontologyModelSourceKey(context) {
  if(!context || !['tenantId','userId'].every(key=>typeof context[key]==='string' && context[key].length))return null;
  const decimal=value=>typeof value==='string' && /^(0|[1-9][0-9]*)$/.test(value);
  const owner=[context.tenantId,context.userId,context.kind];
  if(context.kind==='catalog')return JSON.stringify(owner);
  if(context.kind==='published' && decimal(context.generation))return JSON.stringify([...owner,context.generation]);
  if(context.kind==='draft' && decimal(context.id) && decimal(context.base))return JSON.stringify([...owner,context.id,context.base]);
  return null;
}
function ontologyModelProjection(schemas, links, selected, scope) {
  if(!['selected','related','all'].includes(scope))throw new Error('Unknown declaration scope');
  function unique(rows,key) {
    const counts=new Map();for(const row of rows)counts.set(row[key],(counts.get(row[key]) || 0)+1);
    return new Map(rows.filter(row=>typeof row[key]==='string' && row[key].length && counts.get(row[key])===1).map(row=>[row[key],row]));
  }
  const typeById=unique(schemas,'entity_type'),linkById=unique(links,'link_type');
  const types=schemas.filter(row=>typeById.has(row.entity_type));
  const relationships=links.filter(row=>linkById.has(row.link_type) && typeById.has(row.from_entity_type) && typeById.has(row.to_entity_type));
  const selectedRelationship=linkById.get(selected?.link),seeds=new Set();
  if(selectedRelationship){for(const id of [selectedRelationship.from_entity_type,selectedRelationship.to_entity_type])if(typeById.has(id))seeds.add(id);}
  else if(!selected?.link && typeById.has(selected?.type))seeds.add(selected.type);
  const shown=new Set(scope==='all'?typeById.keys():seeds);
  if(scope==='related')for(const link of relationships)if(seeds.has(link.from_entity_type) || seeds.has(link.to_entity_type)){shown.add(link.from_entity_type);shown.add(link.to_entity_type);}
  const visibleTypes=types.filter(row=>shown.has(row.entity_type));
  const visibleLinks=relationships.filter(row=>shown.has(row.from_entity_type) && shown.has(row.to_entity_type) && (scope!=='selected' || row===selectedRelationship));
  const count=(total,eligible,displayed)=>({total,displayed,remaining:eligible-displayed,undrawn:total-eligible});
  return {schemas:visibleTypes,links:visibleLinks,typeById,linkById,counts:{types:count(schemas.length,types.length,visibleTypes.length),relationships:count(links.length,relationships.length,visibleLinks.length)}};
}
"#;
