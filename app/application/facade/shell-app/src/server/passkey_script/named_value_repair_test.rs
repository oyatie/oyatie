use super::check_named;

#[test]
fn local_creation_refuses_an_existing_exact_definition_and_retains_user_input() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const canonical={id:'vty_status',revision:1,display_name:'Status',members:['A']};modelDocument.named_string_types=[canonical];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();const id=find('live-ontology-draft-choice').value;
  for(const [field,value] of [['new-value-id','vty_status'],['new-value-name','Different'],['new-value-members','["B"]']])enterAllowed(field,value);find('live-ontology-value-add').emit('submit');
  assert.match(find('live-ontology-draft-status').textContent,/exact.*already exists/);assert.equal(find('live-ontology-library-values').querySelectorAll('[data-model-value]').length,1);assert.equal(find('live-ontology-new-value-name').value,'Different');assert.equal(allowedRows('new-value-members')[0].value,'B');assert.deepEqual(modelDrafts.get(id).document.named_string_types,[canonical]);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn reopened_accepted_duplicates_restore_and_remove_only_with_a_canonical_survivor() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const canonical={id:'vty_status',revision:1,display_name:'Status',members:[' A ','A\nB','\uFEFF']},other={id:'vty_other',revision:1,display_name:'Other',members:['Keep\r\nthis']};modelDocument.named_string_types=[canonical];
  const doc=structuredClone(modelDocument);doc.named_string_types=[canonical,{...structuredClone(canonical),display_name:'Changed',members:['Wrong']},other,structuredClone(canonical)];doc.entities[0].revision=2;doc.entities[0].display.description='Retain this unrelated draft edit';
  for(const name of ['status','review_status'])doc.entities[0].properties.push({name,tier:'scalar',data_class:'InternalOnly',required:false,value_type:{shape:'named_string',type_id:'vty_status',revision:1},display:null});
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  const select=index=>find('live-ontology-library-values').querySelectorAll('[data-model-value]')[index].emit('click');select(1);assert.equal(find('live-ontology-value-id').disabled,true);assert.equal(find('live-ontology-value-restore').hidden,false);
  enterAllowed('new-value-name','Unapplied');const before=text(find('live-ontology-draft-diff'));find('live-ontology-value-restore').emit('click');assert.equal(text(find('live-ontology-draft-diff')),before);assert.equal(find('live-ontology-new-value-name').value,'Unapplied');
  find('live-ontology-draft-reload').emit('click');await flush();select(1);find('sign-in-account').focus();find('live-ontology-value-restore').emit('click');assert.equal(document.activeElement,find('sign-in-account'),'restore does not steal outside focus');find('live-ontology-draft-save').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document.named_string_types[1],canonical);
  select(3);const remove=find('live-ontology-value-remove');assert.equal(remove.hidden,false);remove.focus();remove.emit('click');assert.equal(document.activeElement,find('live-ontology-draft-inspector-title'));assert.match(find('live-ontology-draft-selection').textContent,/catalog row 1/);find('live-ontology-draft-save').emit('click');await flush();
  select(1);find('live-ontology-value-remove').emit('click');find('live-ontology-draft-save').emit('click');await flush();
  const saved=modelDrafts.get(id).document;assert.deepEqual(saved.named_string_types,[canonical,other]);assert.deepEqual(saved.entities,doc.entities);assert.deepEqual(saved.links,doc.links);assert.equal(find('live-ontology-value-remove').hidden,true,'last canonical definition cannot be removed');
  find('live-ontology-draft-reload').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document,saved);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn sole_mismatched_accepted_definition_has_explicit_restore_and_refuses_removal_or_stale_actions() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const canonical={id:'vty_status',revision:1,display_name:'Status',members:['Exact\r\nvalue','\uFEFF']};modelDocument.named_string_types=[canonical];const doc=structuredClone(modelDocument);doc.named_string_types[0].members=['Invalid change'];doc.entities[0].display.description='Unrelated';
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].emit('click');
  const restoreButton=find('live-ontology-value-restore'),remove=find('live-ontology-value-remove');assert.equal(remove.hidden,true);remove.emit('click');assert.deepEqual(modelDrafts.get(id).document,doc);
  restoreButton.focus();restoreButton.emit('click');assert.equal(document.activeElement,find('live-ontology-draft-inspector-title'));find('live-ontology-draft-save').emit('click');await flush();const expected=structuredClone(doc);expected.named_string_types[0]=canonical;assert.deepEqual(modelDrafts.get(id).document,expected);assert.equal(restoreButton.hidden,true);
  find('sign-in-account').focus();find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');restoreButton.emit('click');remove.emit('click');assert.equal(root.hidden,true);assert.equal(document.activeElement,find('sign-in-account'));assert.equal(find('live-ontology-library-values').children.length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn accepted_duplicates_require_an_exact_survivor_and_an_unambiguous_published_definition() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  const canonical={id:'vty_status',revision:1,display_name:'Status',members:['Exact\r\nvalue']};modelDocument.named_string_types=[canonical];const doc=structuredClone(modelDocument);doc.named_string_types=[{...canonical,members:['First mismatch']},{...canonical,members:['Second mismatch']}];
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  const select=index=>find('live-ontology-library-values').querySelectorAll('[data-model-value]')[index].emit('click');select(0);assert.equal(find('live-ontology-value-remove').hidden,true);find('live-ontology-value-remove').emit('click');find('live-ontology-value-restore').emit('click');select(1);assert.equal(find('live-ontology-value-remove').hidden,false);find('live-ontology-value-remove').emit('click');find('live-ontology-draft-save').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document.named_string_types,[canonical]);
  modelDocument.named_string_types.push({...canonical,members:['Ambiguous published definition']});await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();select(0);assert.equal(find('live-ontology-value-id').disabled,true);assert.equal(find('live-ontology-value-restore').hidden,true);assert.equal(find('live-ontology-value-remove').hidden,true);const before=text(find('live-ontology-draft-diff'));find('live-ontology-value-restore').emit('click');find('live-ontology-value-remove').emit('click');assert.equal(text(find('live-ontology-draft-diff')),before);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn pending_reusable_property_refreshes_after_catalog_creation_without_retargeting_or_losing_focus()
{
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();const id=find('live-ontology-draft-choice').value;find('live-ontology-add-property').open=true;
  for(const [field,value] of [['new-property-name','Pending status'],['new-property-kind','named_string'],['new-property-required','optional'],['entity-revision','2']])enterAllowed(field,value);find('live-ontology-new-property-kind').emit('change');
  const picker=find('live-ontology-new-property-value').querySelectorAll('select')[0];assert.equal(picker.children.length,0);picker.focus();assert.equal(document.activeElement===picker,true,'pending picker owns focus before catalog creation');
  const add=(id,name)=>{for(const [field,value] of [['new-value-id',id],['new-value-name',name],['new-value-members','[" Exact\\r\\nvalue "]']])enterAllowed(field,value);picker.focus();assert.equal(document.activeElement===picker,true,'picker owns focus at catalog application');find('live-ontology-value-add').emit('submit');};
  add('vty_status','Status');const key=JSON.stringify({shape:'named_string',type_id:'vty_status',revision:1});assert.ok(picker.children.some(option=>option.value===key),'catalog creation refreshes an already-open empty picker');assert.equal(picker.value,'','no pending exact reference is selected automatically');assert.equal(find('live-ontology-new-property-value').querySelectorAll('select')[0],picker);assert.equal(document.activeElement,picker);
  find('live-ontology-new-property-kind').emit('change');assert.ok(picker.children.some(option=>option.value===key));picker.value=key;picker.emit('input');
  add('vty_review','Status');const second=JSON.stringify({shape:'named_string',type_id:'vty_review',revision:1});picker.value=second;picker.emit('input');add('vty_other','Other');
  assert.equal(picker.value,second,'refresh retains a chosen exact second reference');assert.equal(document.activeElement,picker);assert.equal(find('live-ontology-new-property-name').value,'Pending status');assert.equal(find('live-ontology-new-property-kind').value,'named_string');assert.equal(find('live-ontology-new-property-required').value,'optional');assert.equal(find('live-ontology-entity-revision').value,'2');
  find('live-ontology-property-add').emit('submit');find('live-ontology-draft-save').emit('click');await flush();const property=modelDrafts.get(id).document.entities[0].properties.at(-1);assert.equal(property.name,'Pending status');assert.equal(property.tier,'scalar');assert.equal(property.required,false);assert.deepEqual(property.value_type,{shape:'named_string',type_id:'vty_review',revision:1});assert.equal(modelDrafts.get(id).document.named_string_types.length,3);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn value_form_refresh_preserves_stale_exact_references_and_nested_input_while_refusing_new_invalid_adoption()
 {
    check_named(
        r#"
(async()=>{
  await flush();const control=new Element('select'),host=new Element('div');panel.append(control,host);for(const value of ['string','named_string','struct','array']){const option=new Element('option');option.value=value;control.append(option);}
  const a={id:'vty_a',revision:1,display_name:'First',members:['A']},b={id:'vty_b',revision:1,display_name:'Second',members:['B']};let rows=[a,b];context.refreshControl=control;context.refreshHost=host;context.refreshCatalog=()=>rows;const form=vm.runInNewContext("createOntologyValueForm(refreshControl,refreshHost,()=>{},'Pending',refreshCatalog)",context);
  form.write({shape:'scalar',kind:'string'});form.enable(true);control.value='named_string';control.emit('change');const picker=host.querySelectorAll('select')[0],key=JSON.stringify({shape:'named_string',type_id:'vty_b',revision:1});picker.value=key;picker.emit('input');picker.focus();
  rows=[a];form.enable(true);assert.equal(host.querySelectorAll('select')[0],picker);assert.equal(picker.value,key);assert.match(picker.children.find(option=>option.value===key).textContent,/Unresolved or ambiguous/);assert.equal(document.activeElement,picker);assert.throws(()=>form.read(),/unresolved or ambiguous/);
  rows=[a,b,{...b,members:['Duplicate']}];form.enable(true);assert.equal(picker.value,key);assert.throws(()=>form.read(),/unresolved or ambiguous/);rows=[a,{...b,display_name:'Updated second'}];form.enable(true);assert.equal(picker.value,key);assert.match(picker.children.find(option=>option.value===key).textContent,/Updated second/);assert.deepEqual(JSON.parse(JSON.stringify(form.read())),{shape:'named_string',type_id:'vty_b',revision:1});
  const loaded={shape:'named_string',type_id:'vty_missing',revision:0};form.write(loaded);form.enable(true);assert.deepEqual(form.read(),loaded,'unchanged loaded invalid declarations retain structural draft compatibility');
  form.write({shape:'struct',fields:[{name:'Initial',required:false,value_type:{shape:'array',element:{shape:'scalar',kind:'integer'}}}]});form.enable(true);const name=host.querySelectorAll('input')[0];name.value='Pending literal \uFEFF text';name.emit('input');name.focus();const before=JSON.parse(JSON.stringify(form.capture().value));rows=[a];form.enable(true);assert.equal(host.querySelectorAll('input')[0],name);assert.equal(document.activeElement,name);assert.deepEqual(JSON.parse(JSON.stringify(form.capture().value)),before,'refresh preserves nested name, presence, kind and value inputs');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn accepted_catalog_restore_and_duplicate_removal_refresh_retained_exact_value_choices() {
    check_named(
        r#"
(async()=>{
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];const canonical={id:'vty_status',revision:1,display_name:'Status',members:['Exact']};modelDocument.named_string_types=[canonical];const doc=structuredClone(modelDocument),extra=structuredClone(doc.entities[0]);extra.id='ety_draft';extra.properties[0].value_type={shape:'named_string',type_id:'vty_status',revision:1};doc.entities.push(extra);doc.named_string_types.push({...canonical,members:['Changed']});
  const id='9007199254740993';modelDrafts.set(id,{id,tenant_id:'ten_acme',owner:'usr_alice',base_generation:modelWorkspace.generation,version:'1',document:doc,publication:null});find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();find('live-ontology-library-types').querySelectorAll('[data-model-type]').find(row=>row.dataset.modelType==='ety_draft').emit('click');
  const picker=find('live-ontology-property-value').querySelectorAll('select')[0],key=picker.value;assert.match(picker.children[0].textContent,/Unresolved or ambiguous/);find('live-ontology-library-values').querySelectorAll('[data-model-value]')[1].emit('click');find('sign-in-account').focus();find('live-ontology-value-restore').emit('click');assert.equal(picker.value,key);assert.match(picker.children[0].textContent,/Unresolved or ambiguous/);find('live-ontology-value-remove').emit('click');assert.equal(find('live-ontology-property-value').querySelectorAll('select')[0],picker);assert.equal(picker.value,key);assert.match(picker.children[0].textContent,/^Status · vty_status · revision 1$/);assert.equal(document.activeElement,find('sign-in-account'));find('live-ontology-draft-save').emit('click');await flush();assert.deepEqual(modelDrafts.get(id).document.entities,doc.entities);assert.deepEqual(modelDrafts.get(id).document.named_string_types,[canonical]);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
