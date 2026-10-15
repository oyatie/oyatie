use super::explorer_test::check_controller;

fn check_named(check: &str) {
    let helpers = r#"
function allowedRows(field){return find('live-ontology-'+field).querySelectorAll('textarea');}
function replaceAllowed(field,values){
  const host=find('live-ontology-'+field);
  for(const button of host.querySelectorAll('button').filter(button=>button.textContent==='Remove value'))button.emit('click');
  for(const value of values){host.querySelectorAll('button').find(button=>button.textContent==='Add allowed value').emit('click');const input=allowedRows(field).at(-1);input.value=value;input.emit('input');}
}
function enterAllowed(field,value){if(field.endsWith('value-members'))replaceAllowed(field,JSON.parse(value));else{find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');}}
"#;
    check_controller(&format!("{helpers}{check}"));
}

#[test]
fn reusable_catalog_creates_exact_definitions_and_reuses_root_references_through_saved_drafts() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.match(find('live-ontology-values-note').textContent,/Complete reusable catalog from this authorized model/);
  find('live-ontology-draft-new').emit('click');await flush();const id=find('live-ontology-draft-choice').value;
  const enter=enterAllowed;
  const members=[' Active ','Line\nBreak\t\0','\uFEFF','é','<script>literal</script>'];
  for(const [field,value] of [['new-value-id','vty_状态'],['new-value-name','Status'],['new-value-members',JSON.stringify(members)]])enter(field,value);
  find('live-ontology-value-add').emit('submit');
  assert.equal(find('live-ontology-value').hidden,false);assert.match(text(find('live-ontology-value-properties')),/vty_状态/);
  assert.deepEqual(allowedRows('value-members').map(row=>row.value),members);
  for(const [field,value] of [['new-value-id','vty_other'],['new-value-name','Status'],['new-value-members','["Other"]']])enter(field,value);
  find('live-ontology-value-add').emit('submit');
  assert.equal(find('live-ontology-library-values').querySelectorAll('[data-model-value]').length,2,'equal labels retain distinct exact identities');
  find('live-ontology-library-types').querySelectorAll('[data-model-type]')[0].emit('click');
  for(const name of ['status','review_status']){
    enter('new-property-name',name);enter('new-property-kind','named_string');find('live-ontology-new-property-kind').emit('change');
    const picker=find('live-ontology-new-property-value').querySelectorAll('select')[0];
    assert.equal(picker.children.length,2);picker.value=picker.children.find(o=>JSON.parse(o.value).type_id==='vty_状态').value;picker.emit('input');
    enter('entity-revision','2');find('live-ontology-property-add').emit('submit');
  }
  find('live-ontology-draft-save').emit('click');await flush();
  const saved=structuredClone(modelDrafts.get(id).document),refs=saved.entities[0].properties.slice(-2).map(p=>p.value_type);
  assert.deepEqual(refs,[{shape:'named_string',type_id:'vty_状态',revision:1},{shape:'named_string',type_id:'vty_状态',revision:1}]);
  assert.deepEqual(saved.named_string_types,[{id:'vty_状态',revision:1,display_name:'Status',members},{id:'vty_other',revision:1,display_name:'Status',members:['Other']}]);
  assert.equal(refs.some(ref=>Object.hasOwn(ref,'members')),false,'property references stay compact');
  find('live-ontology-draft-reload').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document,saved);
  enter('property-display','A new display');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.named_string_types,saved.named_string_types);assert.deepEqual(modelDrafts.get(id).document.entities[0].properties.slice(-2).map(p=>p.value_type),refs);
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');
  enter('value-name','Edited Status');enter('value-members',JSON.stringify([...members,'Later']));find('live-ontology-value-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.named_string_types[0].members,[...members,'Later']);
  find('live-ontology-draft-validate').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/pass schema validation/);
  assert.equal(find('live-ontology-type').children.length,0,'model-only catalog does not confer object access');
  assert.match(find('live-ontology-context-selected').textContent,/Reusable value/);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn structural_reader_preserves_invalid_named_drafts_and_enforces_whole_document_budget() {
    check_named(
        r#"
(async()=>{
  await flush();
  const accepts=doc=>vm.runInNewContext(`ontologyModelWire.document(${JSON.stringify(doc)})`,context);
  const legacy=structuredClone(modelDocument);assert.equal(accepts(legacy),true);assert.equal(Object.hasOwn(legacy,'named_string_types'),false);
  const invalid=structuredClone(legacy);invalid.named_string_types=[{id:'',revision:0,display_name:'',members:['','same','same','\u0085']},{id:'',revision:0,display_name:'',members:[]}];
  invalid.entities[0].properties[1].value_type={shape:'struct',fields:[{name:'',required:true,value_type:{shape:'named_string',type_id:'missing',revision:0}}]};
  assert.equal(accepts(invalid),true,'structural reader preserves bounded semantic failures');
  for(const edit of [d=>d.named_string_types[0].extra=true,d=>d.named_string_types[0].revision=4294967296,d=>d.named_string_types[0].members=[1],d=>d.named_string_types[0].members=Array(257).fill('a'),d=>d.named_string_types[0].id='x'.repeat(4097),d=>d.entities[0].properties[1].value_type.extra=true]){const bad=structuredClone(invalid);edit(bad);assert.equal(accepts(bad),false);}
  const budget={entities:[],links:[],named_string_types:Array.from({length:16},(_,i)=>({id:'vty_'+i,revision:1,display_name:'A',members:Array(255).fill('a')}))};
  assert.equal(accepts(budget),true,'exact 4096 global nodes');budget.named_string_types[0].members.push('a');assert.equal(accepts(budget),false,'definition/member budget is global');
  const oversized={entities:[],links:[],named_string_types:Array.from({length:256},(_,i)=>({id:'vty_'+i,revision:1,display_name:'x'.repeat(4096),members:[]}))};assert.equal(accepts(oversized),false,'whole document byte cap remains 1MiB');
  modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:invalid,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  find('live-ontology-entity-name').value='Metadata repair';find('live-ontology-entity-name').emit('input');find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  const saved=modelDrafts.get(id).document;assert.deepEqual(saved.named_string_types,invalid.named_string_types);assert.deepEqual(saved.entities[0].properties,invalid.entities[0].properties);
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');find('live-ontology-property-display').value='Metadata only';find('live-ontology-property-display').emit('input');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.entities[0].properties[1].value_type,invalid.entities[0].properties[1].value_type,'loaded invalid nested reference is retained without fallthrough or normalization');
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[1].emit('click');assert.equal(find('live-ontology-value-revision').value,'0');
  find('live-ontology-value-id').value='vty_repaired';find('live-ontology-value-id').emit('input');find('live-ontology-value-revision').value='1';find('live-ontology-value-revision').emit('input');find('live-ontology-value-name').value='Repaired';find('live-ontology-value-name').emit('input');replaceAllowed('value-members',['One']);find('live-ontology-value-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.named_string_types[1],{id:'vty_repaired',revision:1,display_name:'Repaired',members:['One']});
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn published_named_definitions_and_property_types_are_immutable_and_scalar_keys_stay_truthful() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  modelDocument.named_string_types=[{id:'vty_key',revision:1,display_name:'Key',members:['A']}];
  modelDocument.entities[0].properties[0].value_type={shape:'named_string',type_id:'vty_key',revision:1};modelDocument.entities[0].primary_key_property='name';
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');assert.equal(find('live-ontology-value').hidden,false);
  find('live-ontology-draft-new').emit('click');await flush();const id=find('live-ontology-draft-choice').value;
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');
  assert.equal(find('live-ontology-value-id').disabled,true);assert.equal(allowedRows('value-members')[0].disabled,true);
  const original=structuredClone(modelDrafts.get(id).document);find('live-ontology-value-name').value='Tampered';find('live-ontology-value-edit').emit('submit');
  find('live-ontology-library-types').querySelectorAll('[data-model-type]')[0].emit('click');
  assert.equal(find('live-ontology-property-kind').disabled,true);assert.equal(find('live-ontology-entity-primary').value,'name');
  find('live-ontology-entity-name').value='Changed metadata';find('live-ontology-entity-name').emit('input');find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].primary_key_property,'name');assert.deepEqual(modelDrafts.get(id).document.named_string_types,original.named_string_types);assert.deepEqual(modelDrafts.get(id).document.entities[0].properties,original.entities[0].properties);
  find('live-ontology-property-kind').value='string';find('live-ontology-property-edit').emit('submit');assert.match(find('live-ontology-draft-status').textContent,/Published property value types are immutable/);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn objects_named_reader_is_strict_and_does_not_invent_model_catalog_authority() {
    check_named(
        r#"
(async()=>{
  await flush();const named={kind:'named_string',type_id:'vty_status',revision:1,display_name:'Status',members:[' Active ','\uFEFF','A\nB']};
  nativeSchemas=structuredClone(schemaFixture);nativeSchemas[0].properties[0].value_type=named;nativeSchemas[0].primary_key_property='name';
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-mode-objects').emit('click');
  assert.equal(find('live-ontology-type').disabled,false);assert.match(text(find('live-ontology-schema-properties')),/Status · vty_status · revision 1/);
  assert.equal(find('live-ontology-library-values').children.length,0);assert.match(find('live-ontology-values-note').textContent,/Complete reusable catalog requires native model access/);
  find('live-ontology-query').emit('submit');await flush();assert.match(text(find('live-ontology-page')),/ent_alpha/);
  const decode=value=>vm.runInNewContext(`ontologyDecode.schemas(${JSON.stringify(value)})`,context);
  const changed=edit=>{const rows=structuredClone(nativeSchemas);edit(rows[0].properties[0]);return rows;};
  for(const edit of [p=>p.value_type.extra=true,p=>delete p.value_type.members,p=>p.value_type.members=[],p=>p.value_type.members=['a','a'],p=>p.value_type.members=['\u0085'],p=>p.value_type.members=[1],p=>p.value_type.revision=0,p=>p.value_type.revision=4294967296,p=>p.value_type.type_id='vty_',p=>{p.tier='vector';p.value_type={kind:'array',element:named};},p=>{p.tier='struct';p.value_type={kind:'struct',fields:[{name:'a',required:false,value_type:named}]};}])assert.throws(()=>decode(changed(edit)),/source_unavailable/);
  for(const tier of ['scalar','vector','timeseries','geo','ciphertext','struct'])assert.doesNotThrow(()=>decode(changed(p=>{p.value_type=null;p.tier=tier;})));
  for(const kind of ['string','integer','double','boolean','date','timestamp'])assert.doesNotThrow(()=>decode(changed(p=>p.value_type={kind})));
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn named_diagnostics_select_exact_owned_rows_and_reject_ambiguous_or_late_targets() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const id='9007199254740993',doc=structuredClone(modelDocument);doc.named_string_types=[{id:'vty_a',revision:1,display_name:'',members:['a']},{id:'vty_a',revision:1,display_name:'same',members:[]}];
  doc.entities[0].properties[1].value_type={shape:'array',element:{shape:'named_string',type_id:'vty_a',revision:1}};doc.entities[0].properties[1].tier='vector';
  modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  for(const [code,path,focus,index] of [['empty_display_name','named_string_types[0]','value-name',0],['invalid_named_string_members','named_string_types[1].members','value-members',1],['invalid_named_string_revision','named_string_types[1].revision','value-revision',1],['duplicate_named_string_type','named_string_types[1]','value-id',1]]){
    modelValidation={valid:false,diagnostics:[{code,path,message:'Native semantic refusal'}]};find('live-ontology-draft-validate').emit('click');await flush();
    const button=find('live-ontology-draft-diagnostics').querySelectorAll('button')[0];assert.ok(button,`${code} has an indexed target`);button.emit('click');
    assert.equal(document.activeElement,focus==='value-members'?find('live-ontology-value-members').querySelectorAll('button').find(button=>button.textContent==='Add allowed value'):find('live-ontology-'+focus));assert.equal(find('live-ontology-value-name').value,doc.named_string_types[index].display_name);
  }
  modelValidation={valid:false,diagnostics:[{code:'unsupported_named_string_position',path:'entities.ety_record.properties.structure.value_type.element',message:'Root references only'}]};find('live-ontology-draft-validate').emit('click');await flush();
  find('live-ontology-draft-diagnostics').querySelectorAll('button')[0].emit('click');assert.equal(find('live-ontology-property-choice').value,'1');
  assert.equal(document.activeElement,find('live-ontology-property-choice'),'frozen value type falls back to owned enabled property choice');
  const duplicate=structuredClone(doc);duplicate.entities[0].properties.push(structuredClone(duplicate.entities[0].properties[1]));
  const target=vm.runInNewContext(`ontologyModelDiagnosticTarget(${JSON.stringify(duplicate)},${JSON.stringify(modelValidation.diagnostics[0])})`,context);assert.equal(target,null,'duplicate full paths cannot guess a target');
  modelValidation={valid:false,diagnostics:[{code:'schema_response_too_large',path:'schemas',message:'Materialized schemas exceed the response limit'}]};find('live-ontology-draft-validate').emit('click');await flush();assert.equal(find('live-ontology-draft-diagnostics').querySelectorAll('button').length,0,'aggregate diagnostic has no guessed individual target');
  modelValidation={valid:false,diagnostics:[{code:'invalid_named_string_members',path:'named_string_types[1].members',message:'Member set refused'}]};find('live-ontology-draft-validate').emit('click');await flush();const old=find('live-ontology-draft-diagnostics').querySelectorAll('button')[0];
  find('sign-in-account').focus();find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');old.emit('click');assert.equal(document.activeElement,find('sign-in-account'));assert.equal(find('live-ontology-value-properties').children.length,0);assert.equal(allowedRows('value-members').length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn catalog_forms_retain_newer_pending_edits_and_refuse_budget_or_identity_loss_without_retry() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();const id=find('live-ontology-draft-choice').value;
  const enter=enterAllowed;
  for(const [field,value] of [['new-value-id','vty_status'],['new-value-name','Status'],['new-value-members','["One"]']])enter(field,value);find('live-ontology-value-add').emit('submit');
  holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();assert.equal(allowedRows('value-members')[0].disabled,true);
  const pending=allowedRows('value-members')[0];pending.value='Newer\n\uFEFF';pending.emit('input');find('live-ontology-library-types').querySelectorAll('[data-model-type]')[0].emit('click');assert.match(find('live-ontology-draft-status').textContent,/Apply form edits/);
  held();await flush();assert.deepEqual(allowedRows('value-members').map(row=>row.value),['Newer\n\uFEFF']);assert.match(find('live-ontology-draft-state').textContent,/Unapplied/);
  find('live-ontology-value-edit').emit('submit');modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/outcome unknown/);
  const saves=()=>calls.filter(([url])=>url.endsWith('/save')).length,count=saves();find('live-ontology-draft-save').emit('click');find('live-ontology-value-edit').emit('submit');assert.equal(saves(),count,'uncertain write never retries automatically');
  find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/Saved submission recovered/);
  const native=structuredClone(modelDrafts.get(id).document);enter('new-value-id','vty_budget');enter('new-value-name','Too large');const raw='x'.repeat(4097);enter('new-value-members',JSON.stringify([raw]));find('live-ontology-value-add').emit('submit');
  assert.match(find('live-ontology-draft-status').textContent,/document limit/);assert.equal(allowedRows('new-value-members')[0].value,raw,'refused local mutation preserves literal user input');assert.equal(find('live-ontology-new-value-id').value,'vty_budget');assert.deepEqual(modelDrafts.get(id).document,native);
  find('live-ontology-draft-reload').emit('click');await flush();holdPath='/drafts/'+id;find('live-ontology-draft-reload').emit('click');await flush();find('sign-in-tenant').value='ten_other';find('sign-in-tenant').emit('input');held();await flush();
  assert.equal(root.hidden,true);for(const suffix of ['library-values','value-properties'])assert.equal(find('live-ontology-'+suffix).children.length,0);assert.equal(allowedRows('value-members').length,0);
  find('sign-in-tenant').value='ten_acme';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();holdPath='/drafts/'+id;find('live-ontology-draft-reload').emit('click');await flush();find('session-logout').emit('click');held();await flush();assert.equal(root.hidden,true);assert.equal(find('live-ontology-library-values').children.length,0);assert.equal(allowedRows('value-members').length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn human_member_rows_preserve_untouched_literals_and_restore_only_owned_focus() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const doc=structuredClone(modelDocument),definition={id:'vty_\r\nstatus',revision:1,display_name:'Status\r\n',members:['One\r\nTwo','\u0000','\uFEFF']};doc.named_string_types=[definition];
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');
  const host=find('live-ontology-value-members');let rows=allowedRows('value-members');assert.equal(rows[0].attrs['aria-label'],'Selected value type allowed value 1');assert.equal(rows[0].parentElement.tagName,'label');
  rows[0].value='One\nTwo';find('live-ontology-value-id').value='vty_status';find('live-ontology-value-name').value='Status';
  rows[1].value='Edited\tvalue';rows[1].emit('input');find('live-ontology-value-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.named_string_types[0],{...definition,members:['One\r\nTwo','Edited\tvalue','\uFEFF']},'control normalization without input never rewrites untouched native text');
  rows=allowedRows('value-members');const next=rows[1],remove=host.querySelectorAll('button').filter(b=>b.textContent==='Remove value')[0];remove.focus();remove.emit('click');assert.equal(document.activeElement,next);assert.equal(allowedRows('value-members')[0],next);assert.equal(next.value,'Edited\tvalue');
  find('sign-in-account').focus();host.querySelectorAll('button').find(b=>b.textContent==='Remove value').emit('click');assert.equal(document.activeElement,find('sign-in-account'),'member removal does not steal newer external focus');
  const add=host.querySelectorAll('button').find(b=>b.textContent==='Add allowed value');add.emit('click');assert.equal(document.activeElement,allowedRows('value-members').at(-1));
  const labelValues=['Selected value type','New value type'].flatMap(prefix=>['Add '+prefix+' allowed value']);assert.equal(new Set(labelValues).size,2);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn invalid_saved_property_can_restore_the_exact_published_declaration_without_retargeting_or_losing_other_fields()
 {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const id='9007199254740993',doc=structuredClone(modelDocument);doc.entities[0].properties[0].value_type={shape:'named_string',type_id:'vty_missing',revision:0};doc.entities[0].properties[0].display={display_name:'Retain me',description:null,icon:null,color:null,plural_name:null};
  modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  const restoreButton=find('live-ontology-property-restore');assert.equal(restoreButton.hidden,false);assert.equal(find('live-ontology-property-kind').disabled,true);
  enterAllowed('property-display','Pending display');restoreButton.emit('click');assert.equal(find('live-ontology-property-display').value,'Pending display');assert.equal(find('live-ontology-property-kind').value,'named_string','unapplied edits refuse restore');
  find('live-ontology-draft-reload').emit('click');await flush();const count=calls.length;restoreButton.focus();restoreButton.emit('click');assert.equal(calls.length,count);assert.equal(document.activeElement,find('live-ontology-property-choice'));assert.equal(restoreButton.hidden,true);
  find('live-ontology-draft-save').emit('click');await flush();const expected=structuredClone(doc);expected.entities[0].properties[0].value_type=modelDocument.entities[0].properties[0].value_type;expected.entities[0].properties[0].tier=modelDocument.entities[0].properties[0].tier;assert.deepEqual(modelDrafts.get(id).document,expected,'restore changes only exact type and carrier from published baseline');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn duplicate_catalog_rows_remain_inspectable_and_loaded_nested_references_are_not_authoring_choices()
 {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const doc=structuredClone(modelDocument),extra=structuredClone(doc.entities[0]);extra.id='ety_draft';extra.properties[1].tier='vector';extra.properties[1].value_type={shape:'array',element:{shape:'named_string',type_id:'vty_same',revision:1}};doc.entities.push(extra);doc.named_string_types=[{id:'vty_same',revision:1,display_name:'Same',members:['A']},{id:'vty_same',revision:1,display_name:'Same',members:['B']}];
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  for(let index=0;index<2;index++){const button=find('live-ontology-library-values').querySelectorAll('[data-model-value]')[index];assert.equal(button.disabled,false);assert.ok(button.textContent.endsWith('duplicate row '+(index+1)));button.emit('click');assert.match(find('live-ontology-draft-selection').textContent,new RegExp('catalog row '+(index+1)));assert.equal(allowedRows('value-members')[0].value,index?'B':'A');}
  find('live-ontology-library-types').querySelectorAll('[data-model-type]').find(row=>row.dataset.modelType==='ety_draft').emit('click');find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  const child=find('live-ontology-property-value').querySelectorAll('select')[0],retained=child.children.find(option=>option.value==='named_string');assert.ok(retained);assert.equal(child.disabled,false,'draft-only nested type remains repairable');assert.equal(retained.disabled,true,'loaded invalid reference cannot be chosen as new nested authoring');
  enterAllowed('new-property-kind','array');find('live-ontology-new-property-kind').emit('change');const choices=find('live-ontology-new-property-value').querySelectorAll('select')[0].children.map(option=>option.value);assert.equal(choices.includes('named_string'),false,'fresh nested menus contain supported native kinds only');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn accepted_definition_identity_uses_exact_revision_and_draft_only_invalid_revision_remains_repairable()
 {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const accepted={id:'vty_status',revision:1,display_name:'Status',members:['A']};modelDocument.named_string_types=[accepted];const doc=structuredClone(modelDocument);doc.named_string_types.push({id:'vty_status',revision:0,display_name:'',members:[]});
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[1].emit('click');assert.equal(find('live-ontology-value-id').disabled,false,'different revision is a draft-only exact identity');assert.equal(find('live-ontology-value-revision').value,'0');
  for(const [field,value] of [['value-id','vty_repaired'],['value-revision','1'],['value-name','Repaired'],['value-members','["B"]']])enterAllowed(field,value);find('live-ontology-value-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.deepEqual(modelDrafts.get(id).document.named_string_types,[accepted,{id:'vty_repaired',revision:1,display_name:'Repaired',members:['B']}]);
  find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');assert.equal(find('live-ontology-value-id').disabled,true);assert.equal(allowedRows('value-members')[0].disabled,true,'accepted exact pair remains immutable');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn metadata_only_property_edit_preserves_invalid_named_tier_for_native_validation() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const id='9007199254740993',doc=structuredClone(modelDocument);doc.entities[0].properties[0].value_type={shape:'named_string',type_id:'vty_missing',revision:0};doc.entities[0].properties[0].tier='vector';
  modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  enterAllowed('property-display','Edited display only');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  const expected=structuredClone(doc);expected.entities[0].properties[0].display={display_name:'Edited display only',description:null,icon:null,color:null,plural_name:null};assert.deepEqual(modelDrafts.get(id).document,expected,'display editing must preserve the complete invalid tier/reference for native semantic admission');
  modelValidation={valid:false,diagnostics:[{code:'value_type_tier_mismatch',path:'entities.ety_record.properties.name',message:'Native tier mismatch'}]};find('live-ontology-draft-validate').emit('click');await flush();assert.match(text(find('live-ontology-draft-diagnostics')),/value_type_tier_mismatch/);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[path = "named_value_repair_test.rs"]
mod repair;
