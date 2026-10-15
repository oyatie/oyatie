pub(super) const SCRIPT: &str = r#"
function renderOntologySelectedContext(value) {
  const find=id=>document.getElementById(`live-ontology-${id}`),relationship=value?.kind==='relationship',named=value?.kind==='value';
  const selected=!value?'':relationship?`Relationship · ${value.linkId}`:named?`Reusable value · ${value.value.id}`:value.kind==='property'?`Property · ${value.propertyName}`:`Type · ${value.type.id}`;
  const identity=!value?'':relationship?`${value.source} · ${value.from.id} → ${value.to.id}`:named?`${value.source} · revision ${value.value.revision}`:value.kind==='property'?`${value.source} · ${value.type.id}`:value.source;
  const name=!value?'':relationship?`${value.from.name} → ${value.to.name}`:named?value.value.display_name:value.type.name;
  for(const [id,text] of [['selected',selected],['identity',identity],['name',name]])find(`context-${id}`).textContent=text;
  const details=find('selection-details');details.hidden=!value;if(!value)details.open=false;
  const rows=!value?[]:named?[['Source',value.source],['Value type ID',value.value.id],['Revision',value.value.revision],['Display name',value.value.display_name]]:relationship?[['Source',value.source],['Relationship ID',value.linkId],['From type ID',value.from.id],['From type name',value.from.name],['To type ID',value.to.id],['To type name',value.to.name]]:[['Source',value.source],['Type ID',value.type.id],['Type name',value.type.name],...(value.kind==='property'?[['Property name',value.propertyName]]:[])];
  find('selection-values').replaceChildren(...rows.flatMap(([label,text])=>{const term=document.createElement('dt'),definition=document.createElement('dd');term.textContent=label;definition.textContent=text;return [term,definition];}));
}
function ontologyTable(caption, columns, rows) {
  const table = document.createElement('table');
  const title = document.createElement('caption'); title.textContent = caption; table.append(title);
  const head = document.createElement('thead'), headers = document.createElement('tr');
  for (const name of columns) { const th = document.createElement('th'); th.scope = 'col'; th.textContent = name; headers.append(th); }
  head.append(headers); table.append(head);
  const body = document.createElement('tbody');
  for (const row of rows) {
    const tr = document.createElement('tr');
    for (const value of row) { const td = document.createElement('td'); if (value instanceof Element) td.append(value); else td.textContent = value === null ? '—' : String(value); tr.append(td); }
    body.append(tr);
  }
  if (!rows.length) { const tr = document.createElement('tr'), td = document.createElement('td'); td.colSpan = columns.length; td.textContent = 'No native rows.'; tr.append(td); body.append(tr); }
  table.append(body); return table;
}
function ontologyDeclarationText(value) {
  if (value === null) return 'Legacy string contract';
  if (value.kind === 'named_string') return `${value.display_name ?? 'Unresolved reusable string'} · ${value.type_id} · revision ${value.revision}`;
  if (value.kind === 'array') return `array of ${ontologyDeclarationText(value.element)}`;
  if (value.kind === 'struct') return `struct { ${value.fields.map(field => `${field.name} (${field.required ? 'required' : 'optional'}): ${ontologyDeclarationText(field.value_type)}`).join('; ')} }`;
  return value.kind;
}
function renderOntologyValueDefinition(row,source) {
  const find=id=>document.getElementById(`live-ontology-${id}`);
  find('value').hidden=!row;find('value-properties').replaceChildren();
  if(!row)return;
  find('value-properties').append(ontologyTable('Reusable string definition',['Declaration','Value'],[['ID',row.id],['Revision',row.revision],['Display name',row.display_name],['Allowed values',row.members.length]]));
  find('value-properties').append(ontologyTable('Allowed values',['Exact value'],row.members.map(member=>{const literal=document.createElement('span');literal.className='foundry-value-member-text';literal.textContent=member;return [literal];})));
  renderOntologySelectedContext({kind:'value',source,value:row});
}
const ontologyDecode = (() => {
  const object = v => v !== null && typeof v === 'object' && !Array.isArray(v);
  const integer = v => Number.isSafeInteger(v) && v >= 0;
  const string = v => typeof v === 'string';
  const nullable = v => v === null || string(v);
  const bounded = v => string(v) && new TextEncoder().encode(v).length<=4096;
  const named = v => bounded(v) && !/^\p{White_Space}*$/u.test(v);
  const shape = (v, keys) => object(v) && Object.keys(v).length === keys.length && keys.every(key => Object.hasOwn(v, key));
  const uniqueNames = v => new Set(v.map(row => row.name)).size === v.length;
  const privacyClasses = ['INTERNAL_ONLY','PHI','PII_IDENTIFYING','PII_QUASI_IDENTIFIER','PCI','FINANCIAL','FINANCIAL_REGULATED_CREDIT','BEHAVIORAL_TENANT_PRODUCT','BEHAVIORAL_ADS','DECLARED_PREFERENCE','SEARCH_QUERY','PUBLIC','SENSITIVE_PIPA_ART23'];
  const tiers = ['scalar','vector','timeseries','geo','ciphertext','struct'];
  function declaration(v, depth = 1) {
    if (!object(v) || depth > 8) return false;
    if (v.kind==='named_string')return depth===1 && shape(v,['kind','type_id','revision','display_name','members']) && bounded(v.type_id) && v.type_id.startsWith('vty_') && v.type_id.length>4 && integer(v.revision) && v.revision>0 && v.revision<=4294967295 && named(v.display_name) && Array.isArray(v.members) && v.members.length>0 && v.members.length<=256 && v.members.every(named) && new Set(v.members).size===v.members.length;
    if (['string','integer','double','boolean','date','timestamp'].includes(v.kind)) return shape(v, ['kind']);
    if (v.kind === 'array') return shape(v, ['kind','element']) && declaration(v.element, depth + 1);
    return v.kind === 'struct' && shape(v, ['kind','fields']) && Array.isArray(v.fields) && v.fields.length > 0
      && v.fields.every(field => shape(field, ['name','required','value_type']) && named(field.name) && typeof field.required === 'boolean' && declaration(field.value_type, depth + 1)) && uniqueNames(v.fields);
  }
  const schemaProperty = p => shape(p, ['name','tier','data_class','required','value_type']) && named(p.name)
    && tiers.includes(p.tier) && privacyClasses.includes(p.data_class) && typeof p.required === 'boolean'
    && (p.value_type === null || declaration(p.value_type) && p.tier === (p.value_type.kind === 'array' ? 'vector' : p.value_type.kind === 'struct' ? 'struct' : 'scalar'));
  const schema = s => shape(s, ['entity_type','revision','properties','primary_key_property','title_property'])
    && string(s.entity_type) && s.entity_type.startsWith('ety_') && s.entity_type.length > 4 && integer(s.revision) && s.revision <= 4294967295
    && Array.isArray(s.properties) && s.properties.length > 0 && s.properties.every(schemaProperty) && uniqueNames(s.properties)
    && nullable(s.primary_key_property) && nullable(s.title_property)
    && (s.primary_key_property === null || s.properties.some(p => p.name === s.primary_key_property && p.required && (p.value_type === null || p.tier === 'scalar')))
    && (s.title_property === null || s.properties.some(p => p.name === s.title_property));
  const rows = (v, accepts) => { if (!Array.isArray(v) || !v.every(accepts)) throw new Error('source_unavailable'); };
  const pinned = v => object(v) && string(v.object_ref) && integer(v.written_revision)
    && ['current', 'upcast_pending'].includes(v.upcast_state) && object(v.properties)
    && Object.values(v.properties).every(p => object(p) && string(p.value_type) && string(p.data_class) && Object.hasOwn(p, 'value'));
  return {
    schemas: v => { if(new TextEncoder().encode(JSON.stringify(v)).length>1048576)throw new Error('source_unavailable');rows(v, schema); if (new Set(v.map(s => s.entity_type)).size !== v.length) throw new Error('source_unavailable'); },
    labels: (v, schemas) => {
      rows(v, row => shape(row, ['entity_type','revision','display_name']) && string(row.entity_type)
        && row.entity_type.startsWith('ety_') && row.entity_type.length > 4 && integer(row.revision)
        && row.revision <= 4294967295 && named(row.display_name));
      const labels = new Map(v.map(row => [row.entity_type, row]));
      if (labels.size !== v.length || v.length !== schemas.length || schemas.some(row => labels.get(row.entity_type)?.revision !== row.revision)) throw new Error('source_unavailable');
      return labels;
    },
    links: (v, schemas) => {
      const id = (value, prefix) => string(value) && value.startsWith(prefix) && value.length > prefix.length;
      rows(v, row => shape(row, ['link_type','from_entity_type','to_entity_type','cardinality','allow_cross_tenant','revision'])
        && id(row.link_type, 'lty_') && id(row.from_entity_type, 'ety_') && id(row.to_entity_type, 'ety_')
        && ['one_to_one','one_to_many','many_to_many'].includes(row.cardinality) && typeof row.allow_cross_tenant === 'boolean'
        && integer(row.revision) && row.revision <= 4294967295);
      const types = new Set(schemas.map(row => row.entity_type));
      if (new Set(v.map(row => row.link_type)).size !== v.length || v.some(row => !types.has(row.from_entity_type) || !types.has(row.to_entity_type))) throw new Error('source_unavailable');
    },
    page: (v, pin, limit) => { if (!object(v) || v.requested_revision !== pin || !nullable(v.next)) throw new Error('source_unavailable'); rows(v.objects, pinned); if (v.objects.length > limit) throw new Error('source_unavailable'); },
    detail: (v, ref, pin) => { if (!object(v) || v.requested_revision !== pin || !pinned(v.object) || v.object.object_ref !== ref) throw new Error('source_unavailable'); },
    history: v => rows(v, r => object(r) && ['ordinal', 'object_sequence', 'schema_revision'].every(k => integer(r[k])) && ['principal_id', 'decision_id', 'action_type', 'audit_event_type'].every(k => string(r[k]))),
    audit: v => rows(v, r => object(r) && integer(r.ordinal) && string(r.object_ref) && string(r.action_type) && ['applied', 'poisoned'].includes(r.disposition) && ['poison_reason', 'principal_id', 'decision_id'].every(k => nullable(r[k]))),
  };
})();
"#;
