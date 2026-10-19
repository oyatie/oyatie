pub(super) const SCRIPT: &str = r#"
function renderOntologyResourceLibrary(find,node,typeButton,linkButton,schemas,links,values,projected,scope,catalogAvailable) {
  const search=find('search').value.toLocaleLowerCase(),matches=parts=>parts.some(value=>String(value).toLocaleLowerCase().includes(search));
  const typeMatch=row=>matches([row.display_name,row.entity_type,...row.properties.flatMap(property=>[property.name,ontologyDeclarationText(property.value_type)])]);
  const typeText=id=>`${projected.typeById.get(id)?.display_name || id} · ${id}`;
  const types=schemas.filter(typeMatch),relations=links.filter(row=>matches([row.link_type,row.display?.display_name || '',typeText(row.from_entity_type),typeText(row.to_entity_type)]));
  const filtered=values.map((row,index)=>({row,index})).filter(({row})=>matches([row.id,row.display_name,row.revision]));
  const eligible=new Set(projected.schemas.map(row=>row.entity_type));
  const category=row=>!projected.typeById.has(row.entity_type)?'invalid':eligible.has(row.entity_type)?'eligible':'outside';
  const make=row=>{const button=typeButton(row,'foundry-library-item');button.dataset.libraryScope=category(row);return button;};
  const disclose=(id,label,rows,help)=>{const prior=find(id)?.open || false,details=node('details','','foundry-library-more');details.id=`live-ontology-${id}`;details.open=prior;details.append(node('summary',label),node('p',help),...rows.map(make));return details;};
  const shown=projected.schemas.filter(typeMatch),outside=schemas.filter(row=>category(row)==='outside'),invalid=schemas.filter(row=>category(row)==='invalid');
  const children=search?types.map(make):[...shown.map(make),...(outside.length?[disclose('library-outside',`More object types (${outside.length})`,outside,'Outside the current graph scope. Opening this list does not change graph scope.')]:[]),...(invalid.length?[disclose('library-invalid',`Invalid or ambiguous identities (${invalid.length})`,invalid,'These native rows cannot be drawn. Inspect the complete Table and validate the draft to reconcile them.')]:[])];
  find('library-types').replaceChildren(...(children.length?children:[node('p',schemas.length?'No matching types.':'No type declarations.')]));
  const relationshipCounts=projected.counts.relationships;
  find('library-links').replaceChildren(...(relations.length?[node('p',`${projected.links.length} eligible in graph scope · ${relationshipCounts.remaining} outside scope · ${relationshipCounts.undrawn} invalid or ambiguous.`),...relations.map(row=>linkButton(row,'foundry-library-item'))]:[node('p',links.length?'No matching relationships.':'No relationship declarations.')]));
  find('library-values').replaceChildren(...filtered.map(({row,index})=>{const detail=node('details','','foundry-library-more');detail.dataset.modelValue=String(index);const duplicate=values.filter(other=>other.id===row.id && other.revision===row.revision).length>1;detail.append(node('summary',`${row.display_name} · ${row.id} · revision ${row.revision}${duplicate?' · duplicate row '+(index+1):''}`));const members=node('ul');for(const member of row.members)members.append(node('li',member));detail.append(members);return detail;}));
  for(const [kind,label,total,matching] of [['type','Object types',schemas.length,types.length],['link','Relationships',links.length,relations.length],['value','Reusable string types',catalogAvailable?values.length:'unavailable',catalogAvailable?filtered.length:'unavailable']]){find(`library-${kind}-heading`).textContent=`${label} (${total}${search?'; '+matching+' matches':''})`;if(search && typeof matching==='number' && matching)find(`library-${kind}-group`).open=true;}
  const counts=projected.counts.types;find('library-scope').textContent=`${scope[0].toUpperCase()+scope.slice(1)} scope · ${projected.schemas.length} of ${counts.total} types · ${counts.remaining} outside${counts.undrawn?' · '+counts.undrawn+' invalid or ambiguous':''}`;
  find('library-summary').textContent=search?`Search matches: ${types.length} of ${schemas.length} types · ${relations.length} of ${links.length} relationships · ${catalogAvailable?filtered.length+' of '+values.length:'unavailable'} reusable string types`:'';
}
"#;
