pub(super) const SCRIPT: &str = r#"
function createOntologyPropertyOverview(context) {
  const {find}=context,search=find('property-search'),host=find('property-overview');let rows=[],preview=null,more=null;
  function filter(revealMatches=false) {
    const query=search.value.toLocaleLowerCase();let shown=0,extra=0,selectedHidden=false;
    for(const row of rows){row.element.hidden=!row.text.toLocaleLowerCase().includes(query);if(!row.element.hidden){shown++;if(row.index>=4)extra++;}else if(row.index===context.selected())selectedHidden=true;}
    if(preview)preview.hidden=shown===extra;
    if(more){more.hidden=!extra;more.querySelector('summary').textContent=`More properties (${extra})${context.selected()>=4?' · Selected property':''}`;if(revealMatches && query && extra)more.open=true;}
    const count=find('property-count'),message=`${shown} of ${rows.length} properties${selectedHidden?' · Selected property is filtered out.':context.selected()>=4 && !more?.open?' · Selected property is under More properties.':''}`;if(count.textContent!==message)count.textContent=message;
    const empty=find('property-empty');if(empty)empty.hidden=shown>0;
  }
  function selected() {
    for(const row of rows){const active=row.index===context.selected();row.element.setAttribute('data-selected',String(active));row.button.setAttribute('aria-pressed',String(active));}
    if(rows.length)filter();
  }
  search.addEventListener('input',()=>filter(true));
  return {
    clear(){rows=[];preview=null;more=null;search.value='';search.disabled=true;host.hidden=true;find('property-rows').replaceChildren();find('property-count').textContent='';},
    render(){
      const entity=context.declaration(),expanded=more?.open || false;host.hidden=!entity;search.disabled=!entity;rows=[];
      if(!entity){preview=null;more=null;find('property-rows').replaceChildren();return;}
      const properties=ontologyModelRows({...context.document(),entities:[entity]}).schemas[0].properties;
      const cells=entity.properties.map((property,index)=>{
        const button=document.createElement('button');button.type='button';button.textContent=property.display?.display_name || property.name;button.setAttribute('title',`${button.textContent} · ${property.name}`);button.setAttribute('aria-controls','live-ontology-property-edit');button.setAttribute('aria-label',`Inspect property ${property.name}`);
        const value=properties[index].value_type,type=property.value_type===null && property.tier!=='scalar'?`${property.tier} · no value declaration`:value?.kind==='named_string'?value.display_name || 'Unresolved reusable string':value?.kind==='array'?'Array':value?.kind==='struct'?`Struct · ${value.fields.length} ${value.fields.length===1?'field':'fields'}`:ontologyDeclarationText(value),typeLabel=document.createElement('span'),exact=value?.kind==='named_string'?`${type} · ${value.type_id} · revision ${value.revision}`:type;
        typeLabel.textContent=type;typeLabel.setAttribute('title',exact);typeLabel.setAttribute('aria-label',exact);
        const tags=[...(entity.primary_key_property===property.name?['Primary']:[]),...(entity.title_property===property.name?['Title']:[]),...(property.required?['Required']:[])],labels=document.createElement('span');labels.className='foundry-property-tags';
        for(const tag of tags){const label=document.createElement('span');label.textContent=tag;labels.append(label);}
        button.addEventListener('click',()=>{if(button.isConnected && host.contains(button) && button.getClientRects().length && !search.disabled)context.select(index);});
        rows.push({index,button,text:[property.name,property.display?.display_name || '',type,exact,...tags].join(' ')});return [button,typeLabel,labels];
      });
      const table=(label,values)=>{const result=ontologyTable(label,['Name','Type','Tags'],values);result.querySelector('caption').className='sr-only';return result;};
      preview=table(`Properties of ${entity.display_name}`,cells.slice(0,4));const remainder=table(`More properties of ${entity.display_name}`,cells.slice(4)),bodies=[preview.querySelector('tbody'),remainder.querySelector('tbody')];rows.forEach((row,index)=>{row.element=bodies[index<4?0:1].children[index<4?index:index-4];});
      more=document.createElement('details');more.id='live-ontology-property-more';more.open=expanded;const summary=document.createElement('summary');more.append(summary);if(cells.length>4)more.append(remainder);more.addEventListener('toggle',()=>filter());
      const empty=document.createElement('p');empty.id='live-ontology-property-empty';empty.textContent=rows.length?'No matching properties. Clear the search to show all properties.':'No properties declared.';
      find('property-rows').replaceChildren(preview,more,empty);selected();filter(true);
    },
    selected,
    enable(value){search.disabled=!value;for(const row of rows)row.button.disabled=!value;},
  };
}
"#;
