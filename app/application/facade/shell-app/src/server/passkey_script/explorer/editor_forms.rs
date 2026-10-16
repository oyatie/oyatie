pub(super) const SCRIPT: &str = r#"
function createOntologyDraftForms(context) {
  const {find}=context;let typeId=null,linkId=null,propertyIndex=0,enabled=false,held=null;
  const ids=['entity-name','entity-primary','entity-title','entity-description','entity-plural','entity-revision','property-choice','property-name','property-kind','property-required','property-display','new-property-name','new-property-kind','new-property-required','link-from','link-to','link-cardinality','link-revision','new-entity-id','new-entity-name','new-entity-key','new-entity-key-kind','new-entity-title','new-link-id','new-link-from','new-link-to','new-link-cardinality'];
  const scalarKinds=['string','integer','double','boolean','date','timestamp'],cardinalities=['one_to_one','one_to_many','many_to_many'];
  const formIds=['entity-edit','property-edit','property-add','link-edit','entity-add','link-add'];
  const dirtyFields=new Set(),appliedFields={
    'entity-edit':['entity-name','entity-primary','entity-title','entity-description','entity-plural','entity-revision'],
    'property-edit':['property-name','property-kind','property-required','property-display','property-value','property-default'],
    'property-add':['new-property-name','new-property-kind','new-property-required','new-property-value','new-property-default','entity-revision'],
    'link-edit':['link-from','link-to','link-cardinality','link-revision'],
    'entity-add':['new-entity-id','new-entity-name','new-entity-key','new-entity-key-kind','new-entity-title'],
    'link-add':['new-link-id','new-link-from','new-link-to','new-link-cardinality'],
  };
  function input(id){if(context.acceptInput()){dirtyFields.add(id);context.input();}}
  const propertyValue=createOntologyValueForm(find('property-kind'),find('property-value'),()=>input('property-value'),'Selected property',()=>context.document()?.named_string_types || []);
  const newPropertyValue=createOntologyValueForm(find('new-property-kind'),find('new-property-value'),()=>input('new-property-value'),'New property',()=>context.document()?.named_string_types || []);
  const propertyDefault=createOntologyDefaultForm(find('property-default'),()=>input('property-default'),'Selected property',()=>context.document()?.named_string_types || [],()=>propertyValue.read());
  const newPropertyDefault=createOntologyDefaultForm(find('new-property-default'),()=>input('new-property-default'),'New property',()=>context.document()?.named_string_types || [],()=>newPropertyValue.read());
  let issues=[];
  function diagnostics(rows){issues=rows;const selected=declaration()?.properties[propertyIndex],match=selected && rows.find(row=>{const target=ontologyModelDiagnosticTarget(context.document(),row);return target?.kind==='property' && target.id===typeId && target.propertyIndex===propertyIndex && target.focus==='property-default-value';});propertyDefault.error(match?.message || '');newPropertyDefault.error('');}
  const baseline=()=>context.baseline()?.entities.find(e=>e.id===typeId);
  const supported=p=>['scalar','vector','struct'].includes(p.tier) && (p.value_type!==null || p.tier==='scalar');
  const namedReference=value=>value?.shape==='named_string';
  const tier=value=>value?.shape==='array'?'vector':value?.shape==='struct'?'struct':'scalar';
  function display(prior,changes){
    if(Object.entries(changes).every(([key,value])=>(prior?.[key] ?? '')===value))return prior;
    const result={display_name:null,description:null,icon:null,color:null,plural_name:null,...prior};
    for(const [key,value] of Object.entries(changes)){if(new TextEncoder().encode(value).length>4096)throw new Error('Display fields allow at most 4096 bytes.');if(value && /^\p{White_Space}*$/u.test(value))throw new Error('Display fields must contain text or be empty.');result[key]=value || null;}
    return Object.values(result).every(value=>value===null)?null:result;
  }
  const declaration=()=>context.document()?.entities.find(e=>e.id===typeId);
  const relationship=()=>context.document()?.links.find(l=>l.id===linkId);
  function describe(){
    const e=declaration(),l=relationship(),p=e?.properties[propertyIndex],type=id=>({id,name:context.document()?.entities.find(e=>e.id===id)?.display_name || id});
    const value=e?{source:'Draft',kind:p?'property':'type',type:{id:e.id,name:e.display_name},...(p?{propertyName:p.name}:{})}:l?{source:'Draft',kind:'relationship',linkId:l.id,from:type(l.from_entity_type),to:type(l.to_entity_type)}:null;
    context.describe?.(value);return value;
  }
  function enable(value){enabled=Boolean(value);for(const id of ids)find(id).disabled=!enabled;find('property-choice').disabled=!declaration();for(const id of formIds)for(const control of find(id).querySelectorAll('button'))control.disabled=!enabled;find('entity-primary').disabled=!enabled || Boolean(baseline());propertyValue.enable(enabled);newPropertyValue.enable(enabled);propertyDefault.enable(enabled);newPropertyDefault.enable(enabled);}
  function reset(){held=null;typeId=null;linkId=null;propertyIndex=0;dirtyFields.clear();for(const id of ids)find(id).value='';find('new-property-kind').value='string';find('new-property-required').value='optional';find('new-link-cardinality').value='one_to_many';find('new-entity-key').value='id';find('new-entity-key-kind').value='string';find('new-entity-title').value='none';propertyValue.clear();newPropertyValue.write({shape:'scalar',kind:'string'});propertyDefault.clear();newPropertyDefault.write({shape:'scalar',kind:'string'},undefined);issues=[];}
  function clear(){reset();find('property-choice').replaceChildren();for(const id of ['link-from','link-to','new-link-from','new-link-to'])find(id).replaceChildren();for(const id of ['entity-edit','property-edit','property-add','link-edit'])find(id).hidden=true;find('property-compatibility').textContent='';find('entity-key-help').textContent='';enable(false);}
  function property(){
    const e=declaration(),p=e?.properties[propertyIndex];find('property-edit').hidden=!p;describe();
    if(!p)return;
    if(!context.dirty()){find('property-name').value=p.name;propertyValue.write(p.value_type);find('property-required').value=p.required?'required':'optional';find('property-display').value=p.display?.display_name ?? '';propertyDefault.write(p.value_type,p.default_value);}
    const editable=supported(p),prior=baseline()?.properties.find(row=>row.name===p.name),fixed=Boolean(prior && (namedReference(p.value_type) || namedReference(prior.value_type)));
    find('property-restore').hidden=!fixed || ontologyModelEqual(prior.value_type,p.value_type) && prior.tier===p.tier;
    find('property-compatibility').textContent=`${p.tier} · ${p.data_class}. ${editable?'Scalar, array and struct declarations use native validation. Untouched privacy and appearance metadata is preserved.':'This reserved native declaration is read-only; its complete metadata is preserved.'}`;
    for(const id of ['property-name','property-kind','property-required','property-display'])find(id).disabled=!enabled || !editable;
    propertyValue.enable(enabled && editable && !fixed);find('property-kind').disabled=!enabled || !editable || fixed;diagnostics(issues);
    for(const button of find('property-edit').querySelectorAll('button'))button.disabled=!enabled || !editable;find('property-restore').disabled=!enabled || context.dirty();propertyDefault.enable(enabled && editable);
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
    if(e){
      find('entity-primary').replaceChildren(option('','No primary key'),...e.properties.filter(p=>p.required && p.tier==='scalar' && (p.value_type===null || ['scalar','named_string'].includes(p.value_type.shape))).map(p=>option(p.name,p.name)));
      if(e.primary_key_property && !find('entity-primary').children.length)throw new Error('model_unavailable');
      if(e.primary_key_property && !Array.from(find('entity-primary').children).some(o=>o.value===e.primary_key_property))find('entity-primary').append(option(e.primary_key_property,`${e.primary_key_property} · current declaration`));
      find('entity-primary').value=e.primary_key_property ?? '';find('entity-primary').disabled=!enabled || Boolean(baseline());
      find('entity-title').replaceChildren(option('','No title property'),...e.properties.map(p=>option(p.name,p.name)));if(e.title_property && !e.properties.some(p=>p.name===e.title_property))find('entity-title').append(option(e.title_property,`${e.title_property} · unresolved`));find('entity-title').value=e.title_property ?? '';
      find('entity-key-help').textContent=baseline()?'Published identity is fixed. This type’s primary-key designation is read-only; native validation checks property compatibility.':'Choose a required scalar property as this new type’s identity before publication.';
      find('entity-description').value=e.display?.description ?? '';find('entity-plural').value=e.display?.plural_name ?? '';
      find('entity-name').value=e.display_name;find('entity-revision').value=e.revision;propertyIndex=Math.min(propertyIndex,Math.max(0,e.properties.length-1));find('property-choice').value=String(propertyIndex);}
    if(l){find('link-from').value=l.from_entity_type;find('link-to').value=l.to_entity_type;find('link-cardinality').value=l.cardinality;find('link-revision').value=l.revision;}
    property();
    if(held){for(const id of ids)find(id).value=held.values[id];propertyValue.restore(held.propertyValue);newPropertyValue.restore(held.newPropertyValue);propertyDefault.restore(held.propertyDefault);newPropertyDefault.restore(held.newPropertyDefault);held=null;}
  }
  const revision=value=>{const n=Number(value);if(!value || !Number.isInteger(n) || n<1 || n>4294967295)throw new Error('Enter a declaration revision from 1 to 4294967295.');return n;};
  const named=(value,prefix=null)=>{if(/^\p{White_Space}*$/u.test(value) || new TextEncoder().encode(value).length>4096 || prefix && !new RegExp(`^${prefix}[A-Za-z0-9_-]+$`).test(value))throw new Error('Enter a valid bounded declaration name or ID.');return value;};
  function submit(id,work){find(id).addEventListener('submit',event=>{event.preventDefault();if(!context.editable())return;try{const selection={typeId,linkId,propertyIndex},doc=context.document(),before=JSON.parse(JSON.stringify(doc)),form=capture();try{work();if(!ontologyModelWire.document(doc))throw new Error('The native document limit was exceeded; local declarations were retained.');}catch(error){form.propertyDefault.problem=propertyDefault.capture().problem;form.newPropertyDefault.problem=newPropertyDefault.capture().problem;for(const key of Object.keys(doc))delete doc[key];Object.assign(doc,before);held=form;render(doc,true);throw error;}for(const field of appliedFields[id])dirtyFields.delete(field);if(dirtyFields.size){typeId=selection.typeId;linkId=selection.linkId;propertyIndex=selection.propertyIndex;}context.changed(dirtyFields.size>0);}catch(error){context.note(error.message);}});}
  submit('entity-edit',()=>{
    const e=declaration();if(!e)return;const name=named(find('entity-name').value),rev=revision(find('entity-revision').value),primary=find('entity-primary').value || null,title=find('entity-title').value || null;
    if(baseline() && primary!==e.primary_key_property)throw new Error('Published primary-key designation is fixed.');
    if(primary!==e.primary_key_property && primary && !e.properties.some(p=>p.name===primary && p.required && p.tier==='scalar' && (p.value_type===null || ['scalar','named_string'].includes(p.value_type.shape))))throw new Error('Choose a required scalar primary-key property.');
    if(title && !e.properties.some(p=>p.name===title))throw new Error('Choose a declared title property.');
    const appearance=display(e.display,{description:find('entity-description').value,plural_name:find('entity-plural').value});
    e.display_name=name;e.primary_key_property=primary;e.title_property=title;e.display=appearance;e.revision=rev;
  });
  submit('property-edit',()=>{
    const e=declaration(),p=e?.properties[propertyIndex];if(!p || !supported(p))return;
    const name=named(find('property-name').value),next=propertyValue.read(),presence=find('property-required').value;
    if(e.properties.some((other,i)=>i!==propertyIndex && other.name===name))throw new Error('A property with this name already exists.');
    if(baseline()?.properties.some(prior=>prior.name===p.name) && (namedReference(p.value_type) || namedReference(next)) && !ontologyModelEqual(p.value_type,next))throw new Error('Published property value types are immutable. Add a new optional property to adopt a reusable type.');
    if(next===null && p.value_type!==null)throw new Error('Legacy String preserves an existing untyped declaration only.');
    if(!['required','optional'].includes(presence))throw new Error('Choose Required or Optional presence.');
    if(baseline()?.primary_key_property===p.name && name!==p.name)throw new Error('A published primary-key property cannot be renamed.');
    const appearance=display(p.display,{display_name:find('property-display').value}),defaultValue=propertyDefault.read(),valueChanged=!ontologyModelEqual(p.value_type,next);
    if(valueChanged || !ontologyModelEqual(p.default_value,defaultValue) || p.name!==name || p.required!==(presence==='required') || !ontologyModelEqual(p.display,appearance)){
      const rev=revision(find('entity-revision').value),priorName=p.name;
      p.name=name;p.value_type=next;if(valueChanged)p.tier=tier(next);if(defaultValue===undefined)delete p.default_value;else p.default_value=defaultValue;p.required=presence==='required';p.display=appearance;e.revision=rev;
      if(e.primary_key_property===priorName)e.primary_key_property=name;if(e.title_property===priorName)e.title_property=name;dirtyFields.delete('entity-revision');
    }
  });
  submit('property-add',()=>{
    const e=declaration();if(!e)return;const name=named(find('new-property-name').value),value_type=newPropertyValue.read(),presence=find('new-property-required').value;
    if(e.properties.length>=256 || e.properties.some(p=>p.name===name))throw new Error('Property limit reached or name already exists.');
    if(value_type===null)throw new Error('Choose a supported native value declaration.');
    if(!['required','optional'].includes(presence))throw new Error('Choose Required or Optional presence.');
    const defaultValue=newPropertyDefault.read(),rev=revision(find('entity-revision').value);e.properties.push({name,tier:tier(value_type),data_class:'InternalOnly',required:presence==='required',value_type,display:null,...(defaultValue===undefined?{}:{default_value:defaultValue})});newPropertyDefault.write(value_type,undefined);e.revision=rev;propertyIndex=e.properties.length-1;find('new-property-name').value='';
  });
  submit('entity-add',()=>{
    const d=context.document(),id=named(find('new-entity-id').value,'ety_'),name=named(find('new-entity-name').value),key=named(find('new-entity-key').value),kind=find('new-entity-key-kind').value,title=find('new-entity-title').value;
    if(d.entities.length>=256 || d.entities.some(e=>e.id===id))throw new Error('Type limit reached or ID already exists.');
    if(!scalarKinds.includes(kind) || !['none','primary'].includes(title))throw new Error('Choose a native scalar key and initial title.');
    d.entities.push({id,display_name:name,properties:[{name:key,tier:'scalar',data_class:'InternalOnly',required:true,value_type:{shape:'scalar',kind},display:null}],revision:1,pillar:null,primary_key_property:key,title_property:title==='primary'?key:null,display:null});
    typeId=id;linkId=null;propertyIndex=0;find('new-entity-id').value='';find('new-entity-name').value='';
  });
  submit('link-edit',()=>{const l=relationship();if(!l)return;if(l.allow_cross_tenant)throw new Error('Cross-tenant relationship editing requires its native security workflow.');const from=find('link-from').value,to=find('link-to').value,cardinality=find('link-cardinality').value,rev=revision(find('link-revision').value);if(!context.document().entities.some(e=>e.id===from) || !context.document().entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose declared same-tenant endpoints and native cardinality.');l.from_entity_type=from;l.to_entity_type=to;l.cardinality=cardinality;l.revision=rev;});
  submit('link-add',()=>{const d=context.document(),id=named(find('new-link-id').value,'lty_'),from=find('new-link-from').value,to=find('new-link-to').value,cardinality=find('new-link-cardinality').value;if(d.links.length>=256 || d.links.some(l=>l.id===id) || !d.entities.some(e=>e.id===from) || !d.entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose unique relationship ID, declared same-tenant endpoints and native cardinality.');d.links.push({id,from_entity_type:from,to_entity_type:to,cardinality,allow_cross_tenant:false,revision:1,display:null});linkId=id;typeId=null;find('new-link-id').value='';});
  find('property-restore').addEventListener('click',()=>{
    if(!context.editable() || context.dirty())return;
    const e=declaration(),p=e?.properties[propertyIndex],prior=baseline()?.properties.find(row=>row.name===p?.name);if(!p || !prior || !(namedReference(p.value_type) || namedReference(prior.value_type)) || ontologyModelEqual(p.value_type,prior.value_type) && p.tier===prior.tier)return;
    const owned=document.activeElement===find('property-restore');p.value_type=JSON.parse(JSON.stringify(prior.value_type));p.tier=prior.tier;context.changed(false);
    const choice=find('property-choice');if(owned && [document.body,find('property-restore')].includes(document.activeElement) && !choice.disabled && choice.getClientRects().length)choice.focus({preventScroll:true});
  });
  for(const id of ids.filter(id=>id!=='property-choice'))find(id).addEventListener('input',()=>input(id));
  find('property-choice').addEventListener('change',()=>{
    if(context.dirty()){find('property-choice').value=String(propertyIndex);context.note('Apply form edits or discard and reload before changing property.');return;}
    propertyIndex=Number(find('property-choice').value);property();
  });
  const capture=()=>({typeId,linkId,propertyIndex,dirtyFields:[...dirtyFields],values:Object.fromEntries(ids.map(id=>[id,find(id).value])),propertyValue:propertyValue.capture(),newPropertyValue:newPropertyValue.capture(),propertyDefault:propertyDefault.capture(),newPropertyDefault:newPropertyDefault.capture()});
  function selectType(id,index=null){
    const rows=context.document()?.entities.filter(e=>e.id===id) || [];
    if(context.dirty() || rows.length!==1 || index!==null && (!Number.isInteger(index) || index<0 || !rows[0].properties[index] || rows[0].properties.filter(p=>p.name===rows[0].properties[index].name).length!==1))return false;
    typeId=id;linkId=null;propertyIndex=index ?? 0;return true;
  }
  function selectLink(id){if(context.dirty() || context.document()?.links.filter(l=>l.id===id).length!==1)return false;linkId=id;typeId=null;return true;}
  return {diagnostics,capture,restore:snapshot=>{held=snapshot;render(context.document(),true);},suspend:()=>{const snapshot=capture();snapshot.propertyValue.focus=null;snapshot.newPropertyValue.focus=null;snapshot.propertyDefault.focus=-1;snapshot.newPropertyDefault.focus=-1;clear();held=snapshot;},enable:value=>{enable(value);property();},clear,render,describe,selection:()=>typeId,selectedLink:()=>linkId,selectedProperty:()=>propertyIndex,reset,selectType,selectLink};
}
"#;
