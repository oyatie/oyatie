pub(super) const SCRIPT: &str = r#"
const ontologyModelWire = (() => {
  const object = v => v !== null && typeof v === 'object' && !Array.isArray(v);
  const shape = (v, keys) => object(v) && Object.keys(v).length === keys.length && keys.every(k => Object.hasOwn(v,k));
  const text = (v, max=4096) => typeof v === 'string' && new TextEncoder().encode(v).length <= max;
  const nullable = v => v === null || text(v);
  const decimal = v => typeof v === 'string' && /^[1-9][0-9]{0,18}$/.test(v) && (v.length < 19 || v <= '9223372036854775807');
  const revision = v => Number.isInteger(v) && v >= 0 && v <= 4294967295;
  const list = v => Array.isArray(v) && v.length <= 256;
  const display = v => v === null || shape(v,['display_name','description','icon','color','plural_name']) && Object.values(v).every(nullable);
  function value(v, depth=1) {
    if (!object(v) || depth > 8) return false;
    if (v.shape === 'scalar') return shape(v,['shape','kind']) && ['string','integer','double','boolean','date','timestamp'].includes(v.kind);
    if (v.shape === 'array') return shape(v,['shape','element']) && value(v.element,depth+1);
    return v.shape === 'struct' && shape(v,['shape','fields']) && list(v.fields) && v.fields.every(f => shape(f,['name','value_type','required']) && text(f.name) && typeof f.required === 'boolean' && value(f.value_type,depth+1));
  }
  const property = p => shape(p,['name','tier','data_class','required','value_type','display']) && text(p.name) && ['scalar','vector','timeseries','geo','ciphertext','struct'].includes(p.tier) && text(p.data_class) && typeof p.required === 'boolean' && (p.value_type === null || value(p.value_type)) && display(p.display);
  const entity = e => shape(e,['id','display_name','properties','revision','pillar','primary_key_property','title_property','display']) && text(e.id) && text(e.display_name) && list(e.properties) && e.properties.every(property) && revision(e.revision) && [null,'org','person'].includes(e.pillar) && nullable(e.primary_key_property) && nullable(e.title_property) && display(e.display);
  const link = l => shape(l,['id','from_entity_type','to_entity_type','cardinality','allow_cross_tenant','revision','display']) && [l.id,l.from_entity_type,l.to_entity_type].every(v=>text(v)) && ['one_to_one','one_to_many','many_to_many'].includes(l.cardinality) && typeof l.allow_cross_tenant === 'boolean' && revision(l.revision) && display(l.display);
  const document = d => shape(d,['entities','links']) && list(d.entities) && d.entities.every(entity) && list(d.links) && d.links.every(link) && new TextEncoder().encode(JSON.stringify(d)).length <= 1048576;
  const receipt = r => shape(r,['generation','prior_generation','draft_id','draft_version','owner']) && ['generation','prior_generation','draft_id','draft_version'].every(k=>decimal(r[k])) && BigInt(r.generation) === BigInt(r.prior_generation)+1n && text(r.owner,256);
  const assert = valid => { if (!valid) throw new Error('model_unavailable'); };
  function draft(d, identity, id=null) {
    assert(shape(d,['id','tenant_id','owner','base_generation','version','document','publication']) && [d.id,d.base_generation,d.version].every(decimal) && (id===null || d.id===id) && d.tenant_id===identity.tenantId && d.owner===identity.userId && document(d.document) && (d.publication===null || receipt(d.publication) && d.publication.draft_id===d.id && d.publication.draft_version===d.version && d.publication.prior_generation===d.base_generation && d.publication.owner===d.owner));
    return d;
  }
  return {
    decimal, document, draft,
    workspace: v => { assert(shape(v,['generation','document','capabilities']) && decimal(v.generation) && document(v.document) && shape(v.capabilities,['author','publish']) && Object.values(v.capabilities).every(b=>typeof b==='boolean')); return v; },
    page: (v,after=null) => { assert(shape(v,['drafts','next']) && list(v.drafts) && (v.next===null || decimal(v.next)) && v.drafts.every((d,i)=>shape(d,['id','version','base_generation','published_generation']) && [d.id,d.version,d.base_generation].every(decimal) && (d.published_generation===null || decimal(d.published_generation) && BigInt(d.published_generation)>BigInt(d.base_generation)) && (!after || BigInt(d.id)>BigInt(after)) && (!i || BigInt(d.id)>BigInt(v.drafts[i-1].id))) && (v.next===null || v.drafts.at(-1)?.id===v.next)); return v; },
    validation: (v,d) => { assert(shape(v,['draft_id','draft_version','base_generation','valid','diagnostics']) && v.draft_id===d.id && v.draft_version===d.version && v.base_generation===d.base_generation && typeof v.valid==='boolean' && Array.isArray(v.diagnostics) && v.diagnostics.length<=1 && v.valid===(v.diagnostics.length===0) && v.diagnostics.every(r=>shape(r,['code','path','message']) && text(r.code,64) && r.code.length && text(r.path,8256) && text(r.message,256) && r.message.length)); return v; },
  };
})();
function ontologyModelEqual(a,b) {
  if (a===b) return true;
  if (a===null || b===null || typeof a!=='object' || typeof b!=='object' || Array.isArray(a)!==Array.isArray(b) || Array.isArray(a) && a.length!==b.length) return false;
  const keys=Object.keys(a); return keys.length===Object.keys(b).length && keys.every(k=>Object.hasOwn(b,k) && ontologyModelEqual(a[k],b[k]));
}
function ontologyModelRows(document) {
  const declaration = v => v===null ? null : v.shape==='scalar' ? {kind:v.kind} : v.shape==='array' ? {kind:'array',element:declaration(v.element)} : {kind:'struct',fields:v.fields.map(f=>({...f,value_type:declaration(f.value_type)}))};
  return {schemas:document.entities.map(e=>({entity_type:e.id,display_name:e.display_name,revision:e.revision,properties:e.properties.map(p=>({...p,value_type:declaration(p.value_type)})),primary_key_property:e.primary_key_property,title_property:e.title_property})),links:document.links.map(l=>({...l,link_type:l.id}))};
}
"#;
