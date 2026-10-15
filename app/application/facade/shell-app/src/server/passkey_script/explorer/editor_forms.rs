pub(super) const SCRIPT: &str = r#"
function createOntologyDraftForms(context) {
  const {find}=context;let typeId=null,linkId=null,valueIndex=null,propertyIndex=0,enabled=false,held=null;
  const valueIds=['value-id','value-revision','value-name','new-value-id','new-value-name'];
  const ids=[...valueIds,'entity-name','entity-primary','entity-title','entity-description','entity-plural','entity-revision','property-choice','property-name','property-kind','property-required','property-display','new-property-name','new-property-kind','new-property-required','link-from','link-to','link-cardinality','link-revision','new-entity-id','new-entity-name','new-entity-key','new-entity-key-kind','new-entity-title','new-link-id','new-link-from','new-link-to','new-link-cardinality'];
  const scalarKinds=['string','integer','double','boolean','date','timestamp'],cardinalities=['one_to_one','one_to_many','many_to_many'];
  const formIds=['value-edit','value-add','entity-edit','property-edit','property-add','link-edit','entity-add','link-add'];
  const dirtyFields=new Set(),appliedFields={
    'value-edit':['value-id','value-revision','value-name','value-members'],
    'value-add':['new-value-id','new-value-name','new-value-members'],
    'entity-edit':['entity-name','entity-primary','entity-title','entity-description','entity-plural','entity-revision'],
    'property-edit':['property-name','property-kind','property-required','property-display','property-value'],
    'property-add':['new-property-name','new-property-kind','new-property-required','new-property-value','entity-revision'],
    'link-edit':['link-from','link-to','link-cardinality','link-revision'],
    'entity-add':['new-entity-id','new-entity-name','new-entity-key','new-entity-key-kind','new-entity-title'],
    'link-add':['new-link-id','new-link-from','new-link-to','new-link-cardinality'],
  };
  function input(id){if(context.acceptInput()){dirtyFields.add(id);context.input();}}
  const propertyValue=createOntologyValueForm(find('property-kind'),find('property-value'),()=>input('property-value'),'Selected property',()=>context.document()?.named_string_types || []);
  const newPropertyValue=createOntologyValueForm(find('new-property-kind'),find('new-property-value'),()=>input('new-property-value'),'New property',()=>context.document()?.named_string_types || []);
  const valueMembers=createOntologyMemberForm(find('value-members'),()=>input('value-members'),'Selected value type');
  const newValueMembers=createOntologyMemberForm(find('new-value-members'),()=>input('new-value-members'),'New value type');
  const baseline=()=>context.baseline()?.entities.find(e=>e.id===typeId);
  const supported=p=>['scalar','vector','struct'].includes(p.tier) && (p.value_type!==null || p.tier==='scalar');
  const tier=value=>value?.shape==='array'?'vector':value?.shape==='struct'?'struct':'scalar';
  function display(prior,changes){
    if(Object.entries(changes).every(([key,value])=>(prior?.[key] ?? '')===value))return prior;
    const result={display_name:null,description:null,icon:null,color:null,plural_name:null,...prior};
    for(const [key,value] of Object.entries(changes)){if(new TextEncoder().encode(value).length>4096)throw new Error('Display fields allow at most 4096 bytes.');if(value && /^\p{White_Space}*$/u.test(value))throw new Error('Display fields must contain text or be empty.');result[key]=value || null;}
    return Object.values(result).every(value=>value===null)?null:result;
  }
  const declaration=()=>context.document()?.entities.find(e=>e.id===typeId);
  const value=()=>valueIndex===null?null:context.document()?.named_string_types?.[valueIndex];
  const publishedValues=()=>value()?(context.baseline()?.named_string_types || []).filter(row=>row.id===value().id && row.revision===value().revision):[];
  const acceptedValue=()=>publishedValues().length>0;
  const canonicalValue=()=>{const rows=publishedValues();return rows.length===1?rows[0]:null;};
  const canonicalSurvivor=prior=>(context.document()?.named_string_types || []).findIndex((row,index)=>index!==valueIndex && ontologyModelEqual(row,prior));
  const relationship=()=>context.document()?.links.find(l=>l.id===linkId);
  function describe(){
    const e=declaration(),l=relationship(),p=e?.properties[propertyIndex],type=id=>({id,name:context.document()?.entities.find(e=>e.id===id)?.display_name || id});
    const selected=value()?{source:'Draft',kind:'value',value:value()}:e?{source:'Draft',kind:p?'property':'type',type:{id:e.id,name:e.display_name},...(p?{propertyName:p.name}:{})}:l?{source:'Draft',kind:'relationship',linkId:l.id,from:type(l.from_entity_type),to:type(l.to_entity_type)}:null;
    context.describe?.(selected);return selected;
  }
  function enable(value){enabled=Boolean(value);for(const id of ids)find(id).disabled=!enabled;find('property-choice').disabled=!declaration();for(const id of formIds)for(const control of find(id).querySelectorAll('button'))control.disabled=!enabled;find('entity-primary').disabled=!enabled || Boolean(baseline());propertyValue.enable(enabled);newPropertyValue.enable(enabled);newValueMembers.enable(enabled);catalog();}
  function reset(){held=null;typeId=null;linkId=null;valueIndex=null;propertyIndex=0;dirtyFields.clear();for(const id of ids)find(id).value='';valueMembers.clear();newValueMembers.clear();find('new-property-kind').value='string';find('new-property-required').value='optional';find('new-link-cardinality').value='one_to_many';find('new-entity-key').value='id';find('new-entity-key-kind').value='string';find('new-entity-title').value='none';propertyValue.clear();newPropertyValue.write({shape:'scalar',kind:'string'});}
  function clear(){reset();find('property-choice').replaceChildren();for(const id of ['link-from','link-to','new-link-from','new-link-to'])find(id).replaceChildren();for(const id of ['value-edit','entity-edit','property-edit','property-add','link-edit'])find(id).hidden=true;find('property-compatibility').textContent='';find('entity-key-help').textContent='';enable(false);}
  function property(){
    const e=declaration(),p=e?.properties[propertyIndex];find('property-edit').hidden=!p;describe();
    if(!p)return;
    if(!context.dirty()){find('property-name').value=p.name;propertyValue.write(p.value_type);find('property-required').value=p.required?'required':'optional';find('property-display').value=p.display?.display_name ?? '';}
    const editable=supported(p),prior=baseline()?.properties.find(prior=>prior.name===p.name),fixed=Boolean(prior);
    find('property-restore').hidden=!prior || ontologyModelEqual(prior.value_type,p.value_type) && prior.tier===p.tier;
    find('property-kind').disabled=!enabled || !editable || fixed;
    find('property-compatibility').textContent=`${p.tier} · ${p.data_class}. ${editable?'Scalar, array and struct declarations use native validation. Published property value types are immutable; adopt a reusable type on a new optional property. Untouched privacy and appearance metadata is preserved.':'This reserved native declaration is read-only; its complete metadata is preserved.'}`;
    for(const id of ['property-name','property-kind','property-required','property-display'])find(id).disabled=!enabled || !editable;
    propertyValue.enable(enabled && editable && !fixed);find('property-kind').disabled=!enabled || !editable || fixed;
    for(const button of find('property-edit').querySelectorAll('button'))button.disabled=!enabled || !editable;find('property-restore').disabled=!enabled || context.dirty();
  }

  function catalog(){
    const row=value();find('value-edit').hidden=!row;
    const prior=canonicalValue();find('value-restore').hidden=!prior || ontologyModelEqual(row,prior);find('value-remove').hidden=!prior || canonicalSurvivor(prior)<0;
    if(!row){valueMembers.clear();valueMembers.enable(false);return;}
    if(!context.dirty()){for(const [id,content] of [['value-id',row.id],['value-revision',row.revision],['value-name',row.display_name]])find(id).value=content;valueMembers.write(row.members);}
    const fixed=acceptedValue();find('value-compatibility').textContent=fixed?'Accepted definitions are immutable. Add a new value type and adopt it on a new optional property.':'Draft-only definition. Repair its exact fields, save, then validate. References are not retargeted automatically.';
    for(const id of ['value-id','value-revision','value-name'])find(id).disabled=!enabled || fixed;valueMembers.enable(enabled && !fixed);
    for(const button of find('value-edit').querySelectorAll('button'))button.disabled=!enabled || fixed;valueMembers.enable(enabled && !fixed);
    for(const id of ['value-restore','value-remove'])find(id).disabled=!enabled || context.dirty();
  }
  function render(modelDocument,draft){
    if(!modelDocument || !draft){clear();return;}
    if(held){typeId=held.typeId;linkId=held.linkId;valueIndex=held.valueIndex;propertyIndex=held.propertyIndex;dirtyFields.clear();for(const id of held.dirtyFields)dirtyFields.add(id);}
    else if(context.dirty())return;
    if(valueIndex!==null && !modelDocument.named_string_types?.[valueIndex])valueIndex=null;
    if(valueIndex===null && !modelDocument.entities.some(e=>e.id===typeId) && !modelDocument.links.some(l=>l.id===linkId)){typeId=modelDocument.entities[0]?.id || null;linkId=null;}
    const e=declaration(),l=relationship();
    find('draft-selection').textContent=value()?`Reusable value ${value().display_name} · ${value().id} · revision ${value().revision} · catalog row ${valueIndex+1}`:e?`${e.display_name} · ${e.id} · pillar ${e.pillar ?? 'none'} · primary key ${e.primary_key_property ?? 'none'} · title ${e.title_property ?? 'none'}`:l?`${l.id} · cross-tenant intent ${l.allow_cross_tenant}`:'Select a declaration or add a type.';
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
    property();catalog();
    if(held){for(const id of ids)find(id).value=held.values[id];propertyValue.restore(held.propertyValue);newPropertyValue.restore(held.newPropertyValue);valueMembers.restore(held.valueMembers);newValueMembers.restore(held.newValueMembers);held=null;}
  }
  const revision=value=>{const n=Number(value);if(!value || !Number.isInteger(n) || n<1 || n>4294967295)throw new Error('Enter a declaration revision from 1 to 4294967295.');return n;};
  const named=(value,prefix=null)=>{if(/^\p{White_Space}*$/u.test(value) || new TextEncoder().encode(value).length>4096 || prefix && !new RegExp(`^${prefix}[A-Za-z0-9_-]+$`).test(value))throw new Error('Enter a valid bounded declaration name or ID.');return value;};
  function submit(id,work){find(id).addEventListener('submit',event=>{event.preventDefault();if(!context.editable())return;try{const selection={typeId,linkId,valueIndex,propertyIndex},doc=context.document(),before=JSON.parse(JSON.stringify(doc)),form=capture();try{work();if(!ontologyModelWire.document(doc))throw new Error('The native document limit was exceeded; local declarations were retained.');}catch(error){for(const key of Object.keys(doc))delete doc[key];Object.assign(doc,before);held=form;render(doc,true);throw error;}for(const field of appliedFields[id])dirtyFields.delete(field);if(dirtyFields.size){typeId=selection.typeId;linkId=selection.linkId;valueIndex=selection.valueIndex;propertyIndex=selection.propertyIndex;}context.changed(dirtyFields.size>0);}catch(error){context.note(error.message);}});}
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
    if(baseline()?.properties.some(prior=>prior.name===p.name) && !ontologyModelEqual(p.value_type,next))throw new Error('Published property value types are immutable. Add a new optional property to adopt a reusable type.');
    if(next===null && p.value_type!==null)throw new Error('Legacy String preserves an existing untyped declaration only.');
    if(!['required','optional'].includes(presence))throw new Error('Choose Required or Optional presence.');
    if(baseline()?.primary_key_property===p.name && name!==p.name)throw new Error('A published primary-key property cannot be renamed.');
    const appearance=display(p.display,{display_name:find('property-display').value}),valueChanged=!ontologyModelEqual(p.value_type,next);
    if(valueChanged || p.name!==name || p.required!==(presence==='required') || !ontologyModelEqual(p.display,appearance)){
      const rev=revision(find('entity-revision').value),priorName=p.name;
      p.name=name;p.value_type=next;if(valueChanged)p.tier=tier(next);p.required=presence==='required';p.display=appearance;e.revision=rev;
      if(e.primary_key_property===priorName)e.primary_key_property=name;if(e.title_property===priorName)e.title_property=name;dirtyFields.delete('entity-revision');
    }
  });
  submit('property-add',()=>{
    const e=declaration();if(!e)return;const name=named(find('new-property-name').value),value_type=newPropertyValue.read(),presence=find('new-property-required').value;
    if(e.properties.length>=256 || e.properties.some(p=>p.name===name))throw new Error('Property limit reached or name already exists.');
    if(value_type===null)throw new Error('Choose a supported native value declaration.');
    if(!['required','optional'].includes(presence))throw new Error('Choose Required or Optional presence.');
    if(baseline() && presence==='required')throw new Error('New properties on a published type must be optional.');
    const rev=revision(find('entity-revision').value);e.properties.push({name,tier:tier(value_type),data_class:'InternalOnly',required:presence==='required',value_type,display:null});e.revision=rev;propertyIndex=e.properties.length-1;find('new-property-name').value='';
  });
  submit('entity-add',()=>{
    const d=context.document(),id=named(find('new-entity-id').value,'ety_'),name=named(find('new-entity-name').value),key=named(find('new-entity-key').value),kind=find('new-entity-key-kind').value,title=find('new-entity-title').value;
    if(d.entities.length>=256 || d.entities.some(e=>e.id===id))throw new Error('Type limit reached or ID already exists.');
    if(!scalarKinds.includes(kind) || !['none','primary'].includes(title))throw new Error('Choose a native scalar key and initial title.');
    d.entities.push({id,display_name:name,properties:[{name:key,tier:'scalar',data_class:'InternalOnly',required:true,value_type:{shape:'scalar',kind},display:null}],revision:1,pillar:null,primary_key_property:key,title_property:title==='primary'?key:null,display:null});
    typeId=id;linkId=null;valueIndex=null;propertyIndex=0;find('new-entity-id').value='';find('new-entity-name').value='';
  });
  submit('link-edit',()=>{const l=relationship();if(!l)return;if(l.allow_cross_tenant)throw new Error('Cross-tenant relationship editing requires its native security workflow.');const from=find('link-from').value,to=find('link-to').value,cardinality=find('link-cardinality').value,rev=revision(find('link-revision').value);if(!context.document().entities.some(e=>e.id===from) || !context.document().entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose declared same-tenant endpoints and native cardinality.');l.from_entity_type=from;l.to_entity_type=to;l.cardinality=cardinality;l.revision=rev;});
  submit('link-add',()=>{const d=context.document(),id=named(find('new-link-id').value,'lty_'),from=find('new-link-from').value,to=find('new-link-to').value,cardinality=find('new-link-cardinality').value;if(d.links.length>=256 || d.links.some(l=>l.id===id) || !d.entities.some(e=>e.id===from) || !d.entities.some(e=>e.id===to) || !cardinalities.includes(cardinality))throw new Error('Choose unique relationship ID, declared same-tenant endpoints and native cardinality.');d.links.push({id,from_entity_type:from,to_entity_type:to,cardinality,allow_cross_tenant:false,revision:1,display:null});linkId=id;typeId=null;valueIndex=null;find('new-link-id').value='';});
  find('property-restore').addEventListener('click',()=>{
    if(!context.editable() || context.dirty())return;
    const e=declaration(),p=e?.properties[propertyIndex],prior=baseline()?.properties.find(row=>row.name===p?.name);if(!p || !prior || ontologyModelEqual(p.value_type,prior.value_type) && p.tier===prior.tier)return;
    const owned=document.activeElement===find('property-restore');p.value_type=JSON.parse(JSON.stringify(prior.value_type));p.tier=prior.tier;context.changed(false);
    const choice=find('property-choice');if(owned && [document.body,find('property-restore')].includes(document.activeElement) && !choice.disabled && choice.getClientRects().length)choice.focus({preventScroll:true});
  });
  function definition(prefix,rev){
    const members=(prefix===''?valueMembers:newValueMembers).read();
    const prior=prefix===''?value():null,take=(field,next)=>prior && !dirtyFields.has(`value-${field}`)?prior[field==='name'?'display_name':field]:next;
    return {id:take('id',find(`${prefix}value-id`).value),revision:take('revision',rev),display_name:take('name',find(`${prefix}value-name`).value),members:take('members',members)};
  }
  submit('value-add',()=>{const doc=context.document(),rows=doc.named_string_types || [],row=definition('new-',1);if(rows.some(prior=>prior.id===row.id && prior.revision===row.revision))throw new Error('This exact value type ID and revision already exists.');if(rows.length>=256)throw new Error('Reusable value type limit reached.');doc.named_string_types=[...rows,row];valueIndex=rows.length;typeId=null;linkId=null;for(const id of ['new-value-id','new-value-name'])find(id).value='';newValueMembers.clear();});
  submit('value-edit',()=>{if(!value() || acceptedValue())return;const raw=find('value-revision').value,rev=Number(raw);if(!raw || !Number.isInteger(rev) || rev<0 || rev>4294967295)throw new Error('Enter a bounded native revision; new definitions must use 1.');context.document().named_string_types[valueIndex]=definition('',rev);});
  for(const action of ['restore','remove'])find(`value-${action}`).addEventListener('click',()=>{
    if(!context.editable() || context.dirty())return;
    const row=value(),prior=canonicalValue();if(!row || !prior)return;
    const candidate=JSON.parse(JSON.stringify(context.document()));let selected=valueIndex;
    if(action==='restore'){if(ontologyModelEqual(row,prior))return;candidate.named_string_types[valueIndex]=JSON.parse(JSON.stringify(prior));}
    else{const survivor=canonicalSurvivor(prior);if(survivor<0)return;candidate.named_string_types.splice(valueIndex,1);selected=survivor-(survivor>valueIndex?1:0);}
    if(!ontologyModelWire.document(candidate)){context.note('The native document limit was exceeded; local declarations were retained.');return;}
    const button=find(`value-${action}`),owned=document.activeElement===button;context.document().named_string_types=candidate.named_string_types;valueIndex=selected;context.changed(false);
    const title=find('draft-inspector-title');if(owned && [document.body,button].includes(document.activeElement) && title.isConnected && !title.disabled && title.getClientRects().length)title.focus({preventScroll:true});
  });
  for(const id of ids.filter(id=>id!=='property-choice'))find(id).addEventListener('input',()=>input(id));
  find('property-choice').addEventListener('change',()=>{
    if(context.dirty()){find('property-choice').value=String(propertyIndex);context.note('Apply form edits or discard and reload before changing property.');return;}
    propertyIndex=Number(find('property-choice').value);property();
  });
  const capture=()=>({typeId,linkId,valueIndex,propertyIndex,dirtyFields:[...dirtyFields],values:Object.fromEntries(ids.map(id=>[id,find(id).value])),propertyValue:propertyValue.capture(),newPropertyValue:newPropertyValue.capture(),valueMembers:valueMembers.capture(),newValueMembers:newValueMembers.capture()});
  function selectType(id,index=null){
    const rows=context.document()?.entities.filter(e=>e.id===id) || [];
    if(context.dirty() || rows.length!==1 || index!==null && (!Number.isInteger(index) || index<0 || !rows[0].properties[index] || rows[0].properties.filter(p=>p.name===rows[0].properties[index].name).length!==1))return false;
    typeId=id;linkId=null;valueIndex=null;propertyIndex=index ?? 0;return true;
  }
  function selectLink(id){if(context.dirty() || context.document()?.links.filter(l=>l.id===id).length!==1)return false;linkId=id;typeId=null;valueIndex=null;return true;}
  function selectValue(index){if(context.dirty() || !Number.isInteger(index) || index<0 || !context.document()?.named_string_types?.[index])return false;valueIndex=index;typeId=null;linkId=null;return true;}
  return {selectedValue:()=>valueIndex,selectValue,capture,restore:snapshot=>{held=snapshot;render(context.document(),true);},suspend:()=>{const snapshot=capture();snapshot.propertyValue.focus=null;snapshot.newPropertyValue.focus=null;snapshot.valueMembers.focus=-1;snapshot.newValueMembers.focus=-1;clear();held=snapshot;},enable:value=>{enable(value);property();},clear,render,describe,selection:()=>typeId,selectedLink:()=>linkId,selectedProperty:()=>propertyIndex,reset,selectType,selectLink};
}
"#;
