pub(super) const SCRIPT: &str = r#"
function createOntologyDraftForms(context) {
  const {find}=context;let typeId=null,linkId=null,propertyIndex=0,enabled=false,held=null;
  const ids=['entity-name','entity-revision','property-choice','property-name','property-kind','property-required','new-property-name','new-property-kind','new-property-required','link-from','link-to','link-cardinality','link-revision','new-entity-id','new-entity-name','new-link-id','new-link-from','new-link-to','new-link-cardinality'];
  const scalarKinds=['string','integer','double','boolean','date','timestamp'],cardinalities=['one_to_one','one_to_many','many_to_many'];
  const formIds=['entity-edit','property-edit','property-add','link-edit','entity-add','link-add'];
  const dirtyFields=new Set(),appliedFields={
    'entity-edit':['entity-name','entity-revision'],
    'property-edit':['property-name','property-kind','property-required'],
    'property-add':['new-property-name','new-property-kind','new-property-required','entity-revision'],
    'link-edit':['link-from','link-to','link-cardinality','link-revision'],
    'entity-add':['new-entity-id','new-entity-name'],
    'link-add':['new-link-id','new-link-from','new-link-to','new-link-cardinality'],
  };
  const declaration=()=>context.document()?.entities.find(e=>e.id===typeId);
  const relationship=()=>context.document()?.links.find(l=>l.id===linkId);
  function enable(value){enabled=Boolean(value);for(const id of ids)find(id).disabled=!enabled;find('property-choice').disabled=!declaration();for(const id of formIds)for(const control of find(id).querySelectorAll('button'))control.disabled=!enabled;}
  function reset(){held=null;typeId=null;linkId=null;propertyIndex=0;dirtyFields.clear();for(const id of ids)find(id).value='';find('new-property-kind').value='string';find('new-property-required').value='optional';find('new-link-cardinality').value='one_to_many';}
  function clear(){reset();find('property-choice').replaceChildren();for(const id of ['link-from','link-to','new-link-from','new-link-to'])find(id).replaceChildren();for(const id of ['entity-edit','property-edit','property-add','link-edit'])find(id).hidden=true;find('property-compatibility').textContent='';enable(false);}
  function property(){
    const e=declaration(),p=e?.properties[propertyIndex];find('property-edit').hidden=!p;
    if(!p)return;
    if(!context.dirty()){find('property-name').value=p.name;find('property-kind').value=p.value_type===null?'legacy':p.value_type.shape==='scalar'?p.value_type.kind:'legacy';find('property-required').value=p.required?'required':'optional';}
    const supported=p.tier==='scalar' && (p.value_type===null || p.value_type.shape==='scalar');
    find('property-compatibility').textContent=`Native tier ${p.tier} · privacy ${p.data_class} · declaration ${JSON.stringify(p.value_type)}. ${supported?'Untouched display, key and privacy metadata is preserved.':'Structured declarations are read-only in this editor.'}`;
    for(const id of ['property-name','property-kind','property-required'])find(id).disabled=!enabled || !supported;
    for(const button of find('property-edit').querySelectorAll('button'))button.disabled=!enabled || !supported;
  }
  function render(modelDocument,draft){
    if(!modelDocument || !draft){clear();return;}
    if(held){typeId=held.typeId;linkId=held.linkId;propertyIndex=held.propertyIndex;dirtyFields.clear();for(const id of held.dirtyFields)dirtyFields.add(id);}
    else if(context.dirty())return;
    if(!modelDocument.entities.some(e=>e.id===typeId) && !modelDocument.links.some(l=>l.id===linkId)){typeId=modelDocument.entities[0]?.id || null;linkId=null;}
    const e=declaration(),l=relationship();
    find('draft-selection').textContent=e?`${e.display_name} · ${e.id} · pillar ${e.pillar ?? 'none'} · primary key ${e.primary_key_property ?? 'none'} · title ${e.title_property ?? 'none'}`:l?`${l.id} · cross-tenant intent ${l.allow_cross_tenant}`:'Select a declaration or add a type.';
    find('entity-edit').hidden=!e;find('property-add').hidden=!e;find('link-edit').hidden=!l;
    const option=(id,label)=>{const o=document.createElement('option');o.value=id;o.textContent=label;return o;};
    for(const id of ['link-from','link-to','new-link-from','new-link-to']){const prior=find(id).value;find(id).replaceChildren(...modelDocument.entities.map(e=>option(e.id,`${e.display_name} · ${e.id}`)));if(modelDocument.entities.some(e=>e.id===prior))find(id).value=prior;}
    find('property-choice').replaceChildren(...(e?.properties || []).map((p,i)=>option(String(i),p.name)));find('property-choice').disabled=!e;
    if(e){find('entity-name').value=e.display_name;find('entity-revision').value=e.revision;propertyIndex=Math.min(propertyIndex,Math.max(0,e.properties.length-1));find('property-choice').value=String(propertyIndex);}
    if(l){find('link-from').value=l.from_entity_type;find('link-to').value=l.to_entity_type;find('link-cardinality').value=l.cardinality;find('link-revision').value=l.revision;}
    property();
    if(held){for(const id of ids)find(id).value=held.values[id];held=null;}
  }
  const revision=value=>{const n=Number(value);if(!value || !Number.isInteger(n) || n<1 || n>4294967295)throw new Error('Enter a declaration revision from 1 to 4294967295.');return n;};
  const named=(value,prefix=null)=>{if(!value.trim() || new TextEncoder().encode(value).length>4096 || prefix && !new RegExp(`^${prefix}[A-Za-z0-9_-]+$`).test(value))throw new Error('Enter a valid bounded declaration name or ID.');return value;};
  function submit(id,work){find(id).addEventListener('submit',event=>{event.preventDefault();if(!context.editable())return;try{const selection={typeId,linkId,propertyIndex};work();for(const field of appliedFields[id])dirtyFields.delete(field);if(dirtyFields.size){typeId=selection.typeId;linkId=selection.linkId;propertyIndex=selection.propertyIndex;}context.changed(dirtyFields.size>0);}catch(error){context.note(error.message);}});}
  submit('entity-edit',()=>{const e=declaration();if(!e)return;const name=named(find('entity-name').value),rev=revision(find('entity-revision').value);e.display_name=name;e.revision=rev;});
  submit('property-edit',()=>{
    const e=declaration(),p=e?.properties[propertyIndex];if(!p || !(p.tier==='scalar' && (p.value_type===null || p.value_type.shape==='scalar')))return;
    const name=named(find('property-name').value),kind=find('property-kind').value;
    if(e.properties.some((other,i)=>i!==propertyIndex && other.name===name))throw new Error('A property with this name already exists.');
    if(kind==='legacy' && p.value_type!==null)throw new Error('Legacy String preserves an existing untyped declaration only.');
    const next=kind==='legacy'?null:{shape:'scalar',kind};
    if(kind!=='legacy' && !scalarKinds.includes(kind))throw new Error('Choose a supported native scalar declaration.');
    if(!ontologyModelEqual(p.value_type,next) || p.name!==name || p.required!==(find('property-required').value==='required')){
      const rev=revision(find('entity-revision').value);p.name=name;p.value_type=next;p.required=find('property-required').value==='required';e.revision=rev;dirtyFields.delete('entity-revision');
    }
  });
  submit('property-add',()=>{const e=declaration();if(!e)return;const name=named(find('new-property-name').value),kind=find('new-property-kind').value,presence=find('new-property-required').value;if(e.properties.length>=256 || e.properties.some(p=>p.name===name))throw new Error('Property limit reached or name already exists.');if(!scalarKinds.includes(kind))throw new Error('Choose a supported native scalar declaration.');if(!['required','optional'].includes(presence))throw new Error('Choose Required or Optional presence.');const rev=revision(find('entity-revision').value);e.properties.push({name,tier:'scalar',data_class:'InternalOnly',required:presence==='required',value_type:{shape:'scalar',kind},display:null});e.revision=rev;propertyIndex=e.properties.length-1;find('new-property-name').value='';});
  submit('entity-add',()=>{const d=context.document(),id=named(find('new-entity-id').value,'ety_'),name=named(find('new-entity-name').value);if(d.entities.length>=256 || d.entities.some(e=>e.id===id))throw new Error('Type limit reached or ID already exists.');d.entities.push({id,display_name:name,properties:[{name:'id',tier:'scalar',data_class:'InternalOnly',required:true,value_type:{shape:'scalar',kind:'string'},display:null}],revision:1,pillar:null,primary_key_property:'id',title_property:null,display:null});typeId=id;linkId=null;propertyIndex=0;find('new-entity-id').value='';find('new-entity-name').value='';});
  submit('link-edit',()=>{const l=relationship();if(!l)return;if(l.allow_cross_tenant)throw new Error('Cross-tenant relationship editing requires its native security workflow.');const from=find('link-from').value,to=find('link-to').value,cardinality=find('link-cardinality').value,rev=revision(find('link-revision').value);if(!context.document().entities.some(e=>e.id===from) || !context.document().entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose declared same-tenant endpoints and native cardinality.');l.from_entity_type=from;l.to_entity_type=to;l.cardinality=cardinality;l.revision=rev;});
  submit('link-add',()=>{const d=context.document(),id=named(find('new-link-id').value,'lty_'),from=find('new-link-from').value,to=find('new-link-to').value,cardinality=find('new-link-cardinality').value;if(d.links.length>=256 || d.links.some(l=>l.id===id) || !d.entities.some(e=>e.id===from) || !d.entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose unique relationship ID, declared same-tenant endpoints and native cardinality.');d.links.push({id,from_entity_type:from,to_entity_type:to,cardinality,allow_cross_tenant:false,revision:1,display:null});linkId=id;typeId=null;find('new-link-id').value='';});
  for(const id of ids.filter(id=>id!=='property-choice'))find(id).addEventListener('input',()=>{if(context.acceptInput()){dirtyFields.add(id);context.input();}});
  find('property-choice').addEventListener('change',()=>{
    if(context.dirty()){find('property-choice').value=String(propertyIndex);context.note('Apply form edits or discard and reload before changing property.');return;}
    propertyIndex=Number(find('property-choice').value);property();
  });
  const capture=()=>({typeId,linkId,propertyIndex,dirtyFields:[...dirtyFields],values:Object.fromEntries(ids.map(id=>[id,find(id).value]))});
  return {capture,restore:snapshot=>{held=snapshot;render(context.document(),true);},suspend:()=>{const snapshot=capture();clear();held=snapshot;},enable:value=>{enable(value);property();},clear,render,selection:()=>typeId,selectedLink:()=>linkId,reset,selectType:id=>{typeId=id;linkId=null;propertyIndex=0;render(context.document(),true);},selectLink:id=>{linkId=id;typeId=null;render(context.document(),true);}};
}
"#;
