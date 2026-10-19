use super::explorer_test::check_controller;

#[test]
fn default_constants_keep_native_carriers_and_refuse_unknown_or_invalid_shapes() {
    check_controller(
        r#"
(async()=>{
  await flush();
  const valid=[{kind:'string',value:''},{kind:'integer',value:'9223372036854775807'},{kind:'integer',value:'-9223372036854775808'},{kind:'double',value:1.25},{kind:'boolean',value:false},{kind:'date',year:0,month:2,day:29},{kind:'timestamp',epoch_millis:'-9223372036854775808'},{kind:'array',items:[]},{kind:'struct',fields:[]}];
  for(const value of valid){context.literal=value;assert.equal(vm.runInNewContext('ontologyDefaultConstant(literal)',context),true);}
  const invalid=[null,[],{kind:'string',value:'',future:true},{kind:'integer',value:1},{kind:'integer',value:'-0'},{kind:'integer',value:'01'},{kind:'integer',value:'+1'},{kind:'integer',value:'9223372036854775808'},{kind:'timestamp',epoch_millis:'-9223372036854775809'},{kind:'double',value:Infinity},{kind:'double',value:'1'},{kind:'boolean',value:'false'},{kind:'date',year:1900,month:2,day:29},{kind:'date',year:2147483648,month:1,day:1},{kind:'struct',fields:[{name:'x',value:valid[0]},{name:'x',value:valid[0]}]},{kind:'array',items:Array(257).fill(valid[0])},{kind:'string',value:'한'.repeat(1366)},{kind:'future',value:'x'}];
  let deep=valid[0];for(let i=0;i<8;i++)deep={kind:'array',items:[deep]};invalid.push(deep);
  for(const value of invalid){context.literal=value;assert.equal(vm.runInNewContext('ontologyDefaultConstant(literal)',context),false,JSON.stringify(value));}
  function accepts(value,present=true){context.candidate=structuredClone(modelDocument);if(present)context.candidate.entities[0].properties[0].default_value=value;return vm.runInNewContext('ontologyModelWire.document(candidate)',context);}
  assert.equal(accepts(undefined,false),true,'fieldless documents retain legacy shape');
  for(const value of valid)assert.equal(accepts(value),true,'metadata decoder does not impose semantic declaration conformance before saved validation');
  for(const value of invalid)assert.equal(accepts(value),false);
  context.candidate=structuredClone(modelDocument);context.candidate.entities[0].properties[0].future_default={};assert.equal(vm.runInNewContext('ontologyModelWire.document(candidate)',context),false);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn default_forms_add_edit_remove_and_preserve_newer_values_across_save_and_refresh() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993',host=find('live-ontology-property-default'),newHost=find('live-ontology-new-property-default');
  const press=(host,label)=>{const button=host.querySelectorAll('button').find(button=>button.textContent===label);assert.ok(button,label);button.emit('click');};
  const enter=(id,value)=>{const field=find('live-ontology-'+id);field.value=value;field.emit('input');};
  const value=()=>document.getElementById('live-ontology-property-default-value');
  assert.equal(host.attrs['aria-label'],'Selected property default');assert.equal(newHost.attrs['aria-label'],'New property default');assert.match(text(host),/new object omits.*Existing objects are unchanged/);
  const initialCalls=calls.length;press(host,'Add default');assert.equal(value().value,'');assert.match(find('live-ontology-draft-state').textContent,/Unapplied/);assert.equal(find('live-ontology-draft-save').disabled,true);assert.equal(calls.length,initialCalls,'editing metadata submits no object action');
  enter('entity-revision','2');find('live-ontology-property-edit').emit('submit');assert.match(text(find('live-ontology-draft-diff')),/default_value/);assert.match(text(find('live-ontology-schema-properties')),/Default for new objects/);find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.entities[0].properties[0].default_value,{kind:'string',value:''});
  value().value='saved';value().emit('input');find('live-ontology-property-edit').emit('submit');holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  value().value='newer pending';value().emit('input');held();await flush();assert.equal(value().value,'newer pending');assert.match(find('live-ontology-draft-state').textContent,/Unapplied/);
  intervals[0]();await flush();assert.equal(value().value,'newer pending','same-identity suspension restores default input');
  find('live-ontology-property-edit').emit('submit');modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/);find('live-ontology-draft-reload').emit('click');await flush();assert.equal(value().value,'newer pending');
  press(host,'Remove default');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();assert.equal(Object.hasOwn(modelDrafts.get(id).document.entities[0].properties[0],'default_value'),false);find('live-ontology-draft-reload').emit('click');await flush();assert.ok(host.querySelectorAll('button').some(button=>button.textContent==='Add default'));
  for(const [name,kind,raw,expected] of [['count','integer','9223372036854775807',{kind:'integer',value:'9223372036854775807'}],['flag','boolean','false',{kind:'boolean',value:false}],['stamp','timestamp','-9223372036854775808',{kind:'timestamp',epoch_millis:'-9223372036854775808'}]]){
    enter('new-property-name',name);enter('new-property-kind',kind);find('live-ontology-new-property-kind').emit('change');press(newHost,'Add default');const field=document.getElementById('live-ontology-new-property-default-value');field.value=raw;field.emit('input');find('live-ontology-property-add').emit('submit');find('live-ontology-draft-save').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document.entities[0].properties.at(-1).default_value,expected);
  }
  value().value='01';value().emit('input');find('live-ontology-property-edit').emit('submit');assert.equal(value().value,'01','failed Apply preserves invalid local text');assert.equal(value().attrs['aria-invalid'],'true','rollback retains the associated parse error');assert.equal(document.getElementById('live-ontology-property-default-error').hidden,false);assert.equal(modelDrafts.get(id).document.entities[0].properties.at(-1).default_value.epoch_millis,'-9223372036854775808');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(root.hidden,true);assert.equal(host.children.length,0,'identity loss clears classified default values');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn saved_named_default_validation_repairs_only_the_exact_live_field() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];modelDocument.named_string_types=[{id:'vty_status',revision:1,display_name:'EmploymentStatus',members:['active','onLeave','ended']}];modelDocument.entities[0].properties.push({name:'employment.status',tier:'scalar',data_class:'InternalOnly',required:false,value_type:{shape:'named_string',type_id:'vty_status',revision:1},display:null,default_value:{kind:'string',value:'leave'}});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();const id='9007199254740993';
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');const host=find('live-ontology-property-default'),field=()=>document.getElementById('live-ontology-property-default-value');
  assert.equal(find('live-ontology-property-kind').value,'named_string');assert.equal(find('live-ontology-property-kind').disabled,true,'published named declarations remain fixed while defaults can be repaired');assert.equal(find('live-ontology-property-value').querySelectorAll('select')[0].disabled,true);assert.equal(field().disabled,false);assert.equal(field().value,'leave','invalid saved member remains editable');assert.deepEqual(field().children.map(option=>option.value),['leave','active','onLeave','ended']);
  find('live-ontology-entity-name').value='Unrelated metadata';find('live-ontology-entity-name').emit('input');find('live-ontology-entity-revision').value='2';find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();find('live-ontology-draft-reload').emit('click');await flush();
  assert.equal(field().value,'leave');assert.deepEqual(modelDrafts.get(id).document.entities[0].properties[2].default_value,{kind:'string',value:'leave'});
  modelValidation={valid:false,diagnostics:[{code:'invalid_property_default',path:'entities.ety_record.properties.employment.status.default_value',message:'Choose a default admitted by the property declaration.'}]};find('live-ontology-draft-validate').emit('click');await flush();let inspect=find('live-ontology-draft-diagnostics').querySelectorAll('button')[0];assert.ok(inspect);assert.equal(field().attrs['aria-invalid'],'true');assert.match(field().attrs['aria-describedby'],/property-default-error/);const requests=calls.length;inspect.emit('click');assert.equal(document.activeElement,field());assert.equal(calls.length,requests);
  field().value='active';field().emit('input');assert.equal(field().attrs['aria-invalid'],'false');const focused=document.activeElement;inspect.emit('click');assert.equal(document.activeElement,focused,'stale diagnostic cannot act after form edit');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();modelValidation=null;find('live-ontology-draft-validate').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/pass schema validation/);
  context.ambiguous=structuredClone(modelDrafts.get(id).document);context.ambiguous.entities[0].properties.push(structuredClone(context.ambiguous.entities[0].properties[2]));context.problem={code:'invalid_property_default',path:'entities.ety_record.properties.employment.status.default_value'};assert.equal(vm.runInNewContext('ontologyModelDiagnosticTarget(ambiguous,problem)',context),null);
  const saved=modelDrafts.get(id);saved.document.entities[0].properties[2].value_type={shape:'named_string',type_id:'vty_missing',revision:1};find('live-ontology-draft-reload').emit('click');await flush();assert.equal(find('live-ontology-property-restore').hidden,false);find('live-ontology-property-restore').emit('click');find('live-ontology-draft-save').emit('click');await flush();assert.equal(modelDrafts.get(id).document.entities[0].properties[2].value_type.type_id,'vty_status','explicit repair restores the published exact named reference');assert.equal(modelDrafts.get(id).document.entities[0].properties[2].default_value.value,'active','reference repair preserves its configured default');
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher'];await restore();assert.ok(host.querySelectorAll('button').every(button=>button.disabled));assert.equal(field().disabled,true);
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');inspect.emit('click');assert.equal(root.hidden,true);assert.equal(host.children.length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn structured_defaults_keep_all_carriers_and_picker_refusals_without_coercion() {
    check_controller(
        r#"
(async()=>{
  await flush();
  const host=new Element('div');host.id='constant-test';panel.append(host);let changes=0;context.constantHost=host;context.changed=()=>changes++;
  const constant={kind:'struct',fields:[{name:'integer',value:{kind:'integer',value:'-9223372036854775808'}},{name:'timestamp',value:{kind:'timestamp',epoch_millis:'9223372036854775807'}},{name:'flags',value:{kind:'array',items:[{kind:'boolean',value:false},{kind:'double',value:1.25},{kind:'date',year:-44,month:3,day:15},{kind:'string',value:''}]}}]};
  const canonical=structuredClone(constant);canonical.fields=[canonical.fields[2],canonical.fields[0],canonical.fields[1]];
  context.literal=constant;const form=vm.runInNewContext("createOntologyDefaultForm(constantHost,changed,'Composite')",context);form.write({shape:'struct',fields:[]},constant);form.enable(true);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),canonical);
  const before=form.capture(),integer=host.querySelectorAll('input')[0];integer.value='pending invalid';integer.emit('input');const pending=form.capture();form.write(null,undefined);form.restore(pending);assert.equal(host.querySelectorAll('input')[0].value,'pending invalid');assert.throws(()=>form.read(),/valid default/);form.restore(before);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),canonical);
  host.querySelectorAll('input').find(field=>field.value==='9223372036854775807').value='-1';host.querySelectorAll('input').find(field=>field.value==='1.25').value='2.5';host.querySelectorAll('select')[0].value='true';const edited=JSON.parse(JSON.stringify(form.read()));assert.deepEqual(edited.fields[2].value,{kind:'timestamp',epoch_millis:'-1'});assert.deepEqual(edited.fields[0].value.items.slice(0,2),[{kind:'boolean',value:true},{kind:'double',value:2.5}]);
  form.write({shape:'struct',fields:[{name:'counts',required:true,value_type:{shape:'array',element:{shape:'scalar',kind:'integer'}}},{name:'ready',required:false,value_type:{shape:'scalar',kind:'boolean'}}]},undefined);form.enable(true);host.querySelectorAll('button').find(button=>button.textContent==='Add default').emit('click');host.querySelectorAll('button').find(button=>button.textContent.includes('Add Composite field counts item')).emit('click');host.querySelectorAll('button').find(button=>button.textContent==='Add Composite field ready').emit('click');assert.deepEqual(JSON.parse(JSON.stringify(form.read())),{kind:'struct',fields:[{name:'counts',value:{kind:'array',items:[{kind:'integer',value:'0'}]}},{name:'ready',value:{kind:'boolean',value:false}}]});
  const accessible=['input','textarea','select','button'].flatMap(tag=>host.querySelectorAll(tag).map(control=>control.attrs['aria-label']));assert.equal(new Set(accessible).size,accessible.length,'every structured default control has a distinct accessible name');
  context.unordered={kind:'struct',fields:[{name:'\u{10000}',value:{kind:'array',items:[{kind:'integer',value:'2'},{kind:'integer',value:'1'}]}},{name:'\uE000',value:{kind:'struct',fields:[{name:'z',value:{kind:'boolean',value:false}},{name:'a',value:{kind:'boolean',value:true}}]}}]};const sorted=JSON.parse(JSON.stringify(vm.runInNewContext('ontologyDefaultCanonical(unordered)',context)));assert.deepEqual(sorted.fields.map(field=>field.name),['\uE000','\u{10000}'],'native UTF8 ordering matches Unicode scalars, not UTF16 code units');assert.deepEqual(sorted.fields[0].value.fields.map(field=>field.name),['a','z']);assert.deepEqual(sorted.fields[1].value.items.map(item=>item.value),['2','1'],'canonical struct ordering never sorts arrays');assert.deepEqual(context.unordered.fields.map(field=>field.name),['\u{10000}','\uE000'],'canonicalization does not mutate received values');assert.equal(document.getElementById('constant-test-value').attrs.role,'group','composite error/focus target has a named accessible group');
  const typeDeclaration={shape:'struct',fields:[{name:'z',required:true,value_type:{shape:'scalar',kind:'integer'}},{name:'a',required:true,value_type:{shape:'scalar',kind:'boolean'}}]},typeBefore=JSON.stringify(typeDeclaration);form.write(typeDeclaration,undefined);form.enable(true);host.querySelectorAll('button').find(button=>button.textContent==='Add default').emit('click');assert.deepEqual(JSON.parse(JSON.stringify(form.read())).fields.map(field=>field.name),['a','z']);assert.equal(JSON.stringify(typeDeclaration),typeBefore,'canonical default values never reorder struct type declarations');
  const names=[{name:'Name',required:false,value_type:{shape:'scalar',kind:'string'}},{name:'name',required:false,value_type:{shape:'scalar',kind:'string'}}];form.write({shape:'struct',fields:[...names,{name:'Group',required:true,value_type:{shape:'struct',fields:names}}]},undefined);form.enable(true);host.querySelectorAll('button').find(button=>button.textContent==='Add default').emit('click');
  const addNames=['Add Composite field Name','Add Composite field name','Add Composite field Group field Name','Add Composite field Group field name'];
  const labels=texts=>texts.map(text=>{const control=host.querySelectorAll('button').find(button=>button.textContent===text);assert.ok(control,text);return control.attrs['aria-label'];});
  assert.equal(new Set(labels(addNames)).size,4,'case-sensitive direct and nested Add controls retain exact field identity');for(const text of addNames)host.querySelectorAll('button').find(button=>button.textContent===text).emit('click');
  const removeNames=addNames.map(text=>text.replace(/^Add /,'Remove '));assert.equal(new Set(labels(removeNames)).size,4,'case-sensitive direct and nested Remove controls retain exact field identity');
  host.querySelectorAll('button').find(button=>button.textContent==='Remove Composite field Name').emit('click');let named=JSON.parse(JSON.stringify(form.read()));assert.deepEqual(named.fields.map(field=>field.name),['Group','name']);assert.deepEqual(named.fields[0].value.fields.map(field=>field.name),['Name','name'],'direct removal preserves both nested fields');
  host.querySelectorAll('button').find(button=>button.textContent==='Remove Composite field Group field name').emit('click');named=JSON.parse(JSON.stringify(form.read()));assert.deepEqual(named.fields[0].value.fields.map(field=>field.name),['Name'],'nested removal changes only the exact case-sensitive field');assert.equal(named.fields[1].name,'name','nested removal preserves direct lower-case field');
  const control=new Element('select'),nested=new Element('div');for(const value of ['string','array','struct','named_string']){const option=new Element('option');option.value=value;control.append(option);}panel.append(control,nested);let rows=[{id:'vty_status',revision:1,display_name:'Status',members:['active']}];context.pickerControl=control;context.pickerHost=nested;context.catalog=()=>rows;
  const picker=vm.runInNewContext("createOntologyValueForm(pickerControl,pickerHost,changed,'Exact',catalog)",context);picker.write({shape:'named_string',type_id:'vty_missing',revision:2});picker.enable(true);assert.deepEqual(JSON.parse(JSON.stringify(picker.read())),{shape:'named_string',type_id:'vty_missing',revision:2},'untouched unresolved metadata remains lossless');
  const reference=nested.querySelectorAll('select')[0];reference.value=JSON.stringify({shape:'named_string',type_id:'vty_status',revision:1});reference.emit('input');assert.equal(picker.read().type_id,'vty_status');rows.push(structuredClone(rows[0]));picker.enable(true);assert.throws(()=>picker.read(),/unresolved or ambiguous/);
  const unsupported={shape:'array',element:{shape:'named_string',type_id:'vty_status',revision:1}};picker.write(unsupported);picker.enable(true);assert.deepEqual(JSON.parse(JSON.stringify(picker.read())),unsupported);assert.equal(nested.querySelectorAll('select').at(-1).disabled,true,'loaded nested named references are preserved but cannot be selected anew');assert.ok(changes>0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn string_defaults_render_edit_and_restore_with_readonly_textarea_type() {
    check_controller(
        r#"
(async()=>{
  await flush();
  const host=new Element('div');host.id='string-default-test';panel.append(host);let changes=0;context.stringHost=host;context.changed=()=>changes++;
  const form=vm.runInNewContext("createOntologyDefaultForm(stringHost,changed,'String')",context),type={shape:'scalar',kind:'string'};
  form.write(type,undefined);form.enable(true);host.querySelectorAll('button').find(button=>button.textContent==='Add default').emit('click');
  const textarea=host.querySelector('textarea');assert.ok(textarea,'Add default renders a string textarea');assert.equal(textarea.type,'textarea');assert.equal(Object.getOwnPropertyDescriptor(textarea,'type').set,undefined);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),{kind:'string',value:''});
  const literal='<script>alert("text")</script>\n한글 & <b>literal</b>';textarea.value=literal;textarea.emit('input');assert.deepEqual(JSON.parse(JSON.stringify(form.read())),{kind:'string',value:literal});assert.equal(host.querySelectorAll('script').length,0,'string defaults remain text');
  const pending=form.capture();form.write(type,undefined);form.restore(pending);assert.equal(host.querySelector('textarea').value,literal);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),{kind:'string',value:literal});
  const nested={kind:'struct',fields:[{name:'items',value:{kind:'array',items:[{kind:'string',value:''},{kind:'string',value:literal}]}},{name:'title',value:{kind:'string',value:'Saved title'}}]};
  form.write({shape:'struct',fields:[]},nested);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),nested);assert.deepEqual(host.querySelectorAll('textarea').map(field=>field.value),['',literal,'Saved title']);
  host.querySelectorAll('textarea')[0].value='Edited item';host.querySelectorAll('textarea')[0].emit('input');const edited=structuredClone(nested);edited.fields[0].value.items[0].value='Edited item';assert.deepEqual(JSON.parse(JSON.stringify(form.read())),edited);assert.ok(changes>=3);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
