use super::explorer_test::check_controller;

#[test]
fn structured_property_forms_save_native_shapes_and_preserve_untouched_metadata() {
    check_controller(
        r#"
(async()=>{
  await flush();assert.match(input.html,/id="live-ontology-access-badge"[^>]*>Sign-in required<\/span>/);assert.equal(find('live-ontology-access-badge').textContent,'Sign-in required');modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.equal(find('live-ontology-access-badge').textContent,'Session verified');find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993',original=structuredClone(modelDrafts.get(id).document.entities[0]);
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-name').disabled,false,'native struct editing is available');
  assert.equal(find('live-ontology-property-kind').value,'struct');
  const value=find('live-ontology-property-value'),name=value.querySelectorAll('input')[0];
  name.value='anniversary';name.emit('input');
  const fieldKind=value.querySelectorAll('select')[1];fieldKind.value='array';fieldKind.emit('change');
  const elementKind=value.querySelectorAll('select')[2];elementKind.value='timestamp';elementKind.emit('change');
  find('live-ontology-property-name').value='contact';find('live-ontology-property-name').emit('input');
  find('live-ontology-property-display').value='Contact';find('live-ontology-property-display').emit('input');
  find('live-ontology-entity-revision').value='2';
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  let saved=modelDrafts.get(id).document.entities[0];
  assert.deepEqual(saved.properties[1],{...original.properties[1],name:'contact',value_type:{shape:'struct',fields:[{name:'anniversary',required:true,value_type:{shape:'array',element:{shape:'scalar',kind:'timestamp'}}}]},display:{...original.properties[1].display,display_name:'Contact'}});
  assert.deepEqual(saved.properties[0],original.properties[0]);assert.deepEqual(saved.display,original.display);assert.equal(saved.revision,2);
  find('live-ontology-draft-reload').emit('click');await flush();
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-value').querySelectorAll('input')[0].value,'anniversary','saved nested field reopens');
  find('live-ontology-entity-name').value='Submitted';find('live-ontology-entity-edit').emit('submit');
  holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  const pending=find('live-ontology-property-value').querySelectorAll('input')[0];pending.value='pending_field';pending.emit('input');held();await flush();
  assert.equal(find('live-ontology-property-value').querySelectorAll('input')[0].value,'pending_field','newer nested form edits survive a prior save acknowledgement');
  assert.match(find('live-ontology-draft-state').textContent,/Unapplied/);
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].properties[1].value_type.fields[0].name,'pending_field');
  const native=structuredClone(modelDrafts.get(id).document);
  find('live-ontology-property-value').querySelectorAll('input')[0].value='';find('live-ontology-property-edit').emit('submit');
  assert.match(find('live-ontology-draft-status').textContent,/Struct fields need unique names/);assert.deepEqual(modelDrafts.get(id).document,native);
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-access-badge').textContent,'Sign-in required');assert.equal(find('live-ontology-property-value').children.length,0,'identity loss clears nested classified declaration state');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn custom_keys_titles_display_and_nested_creation_use_existing_native_draft_lifecycle() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993';assert.equal(find('live-ontology-object-type-controls').hidden,true,'model selection does not show an unrelated Objects selection');find('live-ontology-mode-objects').emit('click');assert.equal(find('live-ontology-object-type-controls').hidden,false);find('live-ontology-mode-schema').emit('click');assert.equal(find('live-ontology-entity-primary').disabled,true,'published key designation is read-only');
  assert.match(find('live-ontology-entity-key-help').textContent,/Published identity is fixed/);
  const enter=(field,value)=>{find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');};
  for(const [field,value] of [['new-entity-id','ety_booking'],['new-entity-name','Booking'],['new-entity-key','booking_no'],['new-entity-key-kind','integer'],['new-entity-title','primary']])enter(field,value);
  find('live-ontology-entity-add').emit('submit');
  assert.equal(find('live-ontology-entity-primary').value,'booking_no');assert.equal(find('live-ontology-entity-primary').disabled,false);
  enter('new-property-name','events');enter('new-property-kind','array');find('live-ontology-new-property-kind').emit('change');
  const host=find('live-ontology-new-property-value'),child=host.querySelectorAll('select')[0];child.value='struct';child.emit('change');
  host.querySelectorAll('button').find(b=>b.textContent==='Add field').emit('click');
  const field=host.querySelectorAll('input')[0];field.value='when';field.emit('input');
  const kind=host.querySelectorAll('select').at(-1);kind.value='date';kind.emit('change');
  find('live-ontology-property-add').emit('submit');
  enter('entity-description','Bookings and dates');enter('entity-plural','Bookings');find('live-ontology-entity-edit').emit('submit');
  find('live-ontology-draft-save').emit('click');await flush();
  const saved=modelDrafts.get(id).document.entities.find(e=>e.id==='ety_booking');
  assert.equal(saved.primary_key_property,'booking_no');assert.equal(saved.title_property,'booking_no');
  assert.deepEqual(saved.properties[0],{name:'booking_no',tier:'scalar',data_class:'InternalOnly',required:true,value_type:{shape:'scalar',kind:'integer'},display:null});
  assert.deepEqual(saved.properties[1],{name:'events',tier:'vector',data_class:'InternalOnly',required:false,value_type:{shape:'array',element:{shape:'struct',fields:[{name:'when',required:true,value_type:{shape:'scalar',kind:'date'}}]}},display:null});
  assert.deepEqual(saved.display,{display_name:null,description:'Bookings and dates',icon:null,color:null,plural_name:'Bookings'});
  find('live-ontology-draft-validate').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/pass schema validation/);
  find('live-ontology-draft-reload').emit('click');await flush();
  find('live-ontology-entity-primary').value='';find('live-ontology-entity-title').value='events';find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  const reopened=modelDrafts.get(id).document.entities.find(e=>e.id==='ety_booking');assert.equal(reopened.primary_key_property,null);assert.equal(reopened.title_property,'events');
  assert.equal(find('live-ontology-type').children.length,0,'draft selections never grant published Objects reads');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
