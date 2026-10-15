use super::explorer_test::check_controller;

#[test]
fn saved_diagnostic_targets_match_native_complete_paths_uniquely() {
    check_controller(
        r##"
(async()=>{
  await flush();
  const saved=structuredClone(modelDocument);saved.entities[0].id='ety_a';saved.entities[0].properties[0].name='key';
  saved.links=[{id:'lty_a',from_entity_type:'ety_a',to_entity_type:'ety_a',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null}];
  const type=focus=>({kind:'type',id:'ety_a',focus}),link=focus=>({kind:'link',id:'lty_a',focus}),property=focus=>({kind:'property',id:'ety_a',propertyIndex:0,focus});
  function resolve(document,code,path){context.targetDocument=document;context.targetDiagnostic={code,path,message:'Native diagnostic'};return JSON.parse(JSON.stringify(vm.runInNewContext('ontologyModelDiagnosticTarget(targetDocument,targetDiagnostic)',context)));}
  const families=[
    ['no_changes','document',null],['malformed_model','document',null],
    ['new_definition_revision','entities.ety_a',type('entity-revision')],['unsupported_removal','entities.ety_removed',null],
    ['unsupported_key_adoption','entities.ety_a.primary_key_property',type('draft-inspector-title')],
    ['unsupported_data_class','entities.ety_a.properties.key',property('property-choice')],
    ['cross_tenant_link','links.lty_a',link('draft-inspector-title')],['invalid_entity_type_id','entities.ety_a',type('draft-inspector-title')],
    ['invalid_link_type_id','links.lty_a',link('draft-inspector-title')],['empty_display_name','entities.ety_a.display_name',type('entity-name')],
    ['empty_properties','entities.ety_a',type('draft-inspector-title')],['empty_property_name','entities.ety_a',type('draft-inspector-title')],
    ['duplicate_property_name','entities.ety_a.properties.key',null],['unknown_endpoint','links.lty_a',link('draft-inspector-title')],
    ['cross_pillar_link','links.lty_a',link('draft-inspector-title')],['revision_not_increased','entities.ety_a.revision',type('entity-revision')],
    ['incompatible_evolution','entities.ety_a',type('draft-inspector-title')],['frozen_pillar','entities.ety_a.pillar',type('draft-inspector-title')],
    ['blank_display_field','entities.ety_a.display.description',type('draft-inspector-title')],
    ['frozen_field_changed','links.lty_a.cardinality',link('link-cardinality')],
    ['invalid_value_type','entities.ety_a.properties.key',property('property-kind')],['value_type_tier_mismatch','entities.ety_a.properties.key',property('property-kind')],
    ['designated_property_not_declared','entities.ety_a',type('draft-inspector-title')],['primary_key_not_required','entities.ety_a.properties.key',property('property-required')],
    ['primary_key_not_scalar','entities.ety_a.properties.key',property('property-kind')],['frozen_primary_key','entities.ety_a.primary_key_property',type('draft-inspector-title')],
    ['native_invalid','entities.ety_a',null],['metadata_incompatible','document',null],
  ];
  assert.equal(new Set(families.map(row=>row[0])).size,28);
  for(const [code,path,target] of families){assert.deepEqual(resolve(saved,code,path),target,code);assert.equal(resolve(saved,code,'baseline.entity_revisions.revision'),null,'baseline is never a draft target');}
  for(const code of ['new_definition_revision','invalid_entity_type_id'])assert.deepEqual(resolve(saved,code,'links.lty_a'),link(code==='new_definition_revision'?'link-revision':'draft-inspector-title'));
  assert.deepEqual(resolve(saved,'revision_not_increased','links.lty_a.revision'),link('link-revision'));
  for(const field of ['display_name','description','icon','color','plural_name'])assert.deepEqual(resolve(saved,'blank_display_field','links.lty_a.display.'+field),link('draft-inspector-title'));
  for(const [field,focus] of [['from_entity_type','link-from'],['to_entity_type','link-to'],['allow_cross_tenant','draft-inspector-title']])assert.deepEqual(resolve(saved,'frozen_field_changed','links.lty_a.'+field),link(focus));
  for(const [code,path] of [['unknown_code','entities.ety_a'],['empty_display_name','entities.ety_a'],['invalid_value_type','entities.ety_a.properties.key.kind'],['frozen_field_changed','links.lty_a.batch_limit'],['blank_display_field','entities.ety_a.display.unknown'],['unsupported_removal','entities.ety_a']])assert.equal(resolve(saved,code,path),null);
  const dotted=structuredClone(saved);dotted.entities[0].id='ety_a.#[]<img>';dotted.entities[0].properties[0].name='key.part.<script>';
  assert.deepEqual(resolve(dotted,'invalid_value_type','entities.ety_a.#[]<img>.properties.key.part.<script>'),{kind:'property',id:'ety_a.#[]<img>',propertyIndex:0,focus:'property-kind'});
  const collision=structuredClone(saved);collision.entities[0].properties[0].name='x.properties.y';collision.entities.push({...structuredClone(collision.entities[0]),id:'ety_a.properties.x',properties:[{...structuredClone(collision.entities[0].properties[0]),name:'y'}]});
  assert.equal(resolve(collision,'invalid_value_type','entities.ety_a.properties.x.properties.y'),null,'full strings can collide across valid dotted IDs/names');
  const duplicate=structuredClone(saved);duplicate.entities[0].properties.push(structuredClone(duplicate.entities[0].properties[0]));
  for(const code of ['duplicate_property_name','invalid_value_type','primary_key_not_required'])assert.equal(resolve(duplicate,code,'entities.ety_a.properties.key'),null);
  const duplicateId=structuredClone(saved);duplicateId.entities.push({...structuredClone(saved.entities[0]),properties:[{...structuredClone(saved.entities[0].properties[0]),name:'other'}]});
  assert.equal(resolve(duplicateId,'invalid_value_type','entities.ety_a.properties.key'),null,'unique path still requires a unique owning ID');
  const duplicateLink=structuredClone(saved);duplicateLink.links.push(structuredClone(duplicateLink.links[0]));assert.equal(resolve(duplicateLink,'unknown_endpoint','links.lty_a'),null);
  let dirty=false;context.formContext={find:id=>find('live-ontology-'+id),document:()=>saved,dirty:()=>dirty,editable:()=>false,acceptInput:()=>false,input(){},changed(){},note(){}};
  const forms=vm.runInNewContext('createOntologyDraftForms(formContext)',context);assert.equal(forms.selectType('ety_a',1),true);
  const before=JSON.stringify(forms.capture());
  for(const index of [-1,2,'1',1.5]){assert.equal(forms.selectType('ety_a',index),false);assert.equal(JSON.stringify(forms.capture()),before,'invalid index refuses before retargeting');}
  dirty=true;assert.equal(forms.selectLink('lty_a'),false);assert.equal(forms.selectType('ety_a',0),false);assert.equal(JSON.stringify(forms.capture()),before);dirty=false;
  saved.entities[0].properties.push(structuredClone(saved.entities[0].properties[1]));assert.equal(forms.selectType('ety_a',1),false);assert.equal(JSON.stringify(forms.capture()),before);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn saved_validation_inspection_preserves_state_and_focus() {
    check_controller(
        r##"
(async()=>{
  await flush();
  modelDocument.entities[0].properties.push({...structuredClone(modelDocument.entities[0].properties[0]),name:'age.part.<img>',value_type:{shape:'scalar',kind:'integer'},required:false});
  modelDocument.entities.push({...structuredClone(modelDocument.entities[0]),id:'ety_other.part',display_name:'Other <script>'});
  modelDocument.links.push({id:'lty_reports.part',from_entity_type:'ety_record',to_entity_type:'ety_other.part',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null});
  modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();const id='9007199254740993';
  const native=structuredClone(modelDrafts.get(id).document),publishedType=find('live-ontology-type').value,publishedRevision=find('live-ontology-revision').value;
  find('live-ontology-mode-objects').emit('click');find('live-ontology-query').emit('submit');await flush();find('live-ontology-audit-read').emit('click');await flush();find('live-ontology-mode-schema').emit('click');
  const objects=()=>JSON.stringify([find('live-ontology-type').value,find('live-ontology-revision').value,text(find('live-ontology-page')),text(find('live-ontology-properties')),text(find('live-ontology-audit-rows'))]);const objectState=objects();
  function diagnostic(code,path){return {code,path,message:'Native <img src=x> constraint'};}
  async function validate(code,path){modelValidation={valid:false,diagnostics:[diagnostic(code,path)]};find('live-ontology-draft-validate').emit('click');await flush();return find('live-ontology-draft-diagnostics').querySelectorAll('button')[0];}
  function state(){return [find('live-ontology-draft-selection').textContent,...['entity-name','entity-revision','property-choice','property-name','property-kind','property-required','link-from','link-to','link-cardinality'].map(field=>find('live-ontology-'+field).value),document.activeElement,objects(),calls.length];}
  function refuse(button){const before=state();button.emit('click');assert.deepEqual(state(),before,'refusal leaves selection, values, focus, object authority and requests unchanged');}
  function inspect(button,focus){assert.ok(button);assert.equal(button.type,'button');const requests=calls.length;button.focus();button.emit('click');assert.equal(document.activeElement,find('live-ontology-'+focus));assert.equal(calls.length,requests,'Inspect has no network effect');assert.deepEqual(modelDrafts.get(id).document,native,'Inspect never edits the saved document');assert.equal(objects(),objectState);assert.equal(find('live-ontology-draft-save').disabled,true,'working document remains saved');}
  assert.match(input.html,/<h4[^>]*id="live-ontology-draft-inspector-title"[^>]*tabindex="-1"/);
  let button=await validate('revision_not_increased','entities.ety_other.part.revision');
  assert.match(button.textContent,/Inspect type Other <script>.*ety_other.part/);inspect(button,'entity-revision');assert.match(find('live-ontology-draft-selection').textContent,/ety_other.part/);
  const raw=text(find('live-ontology-draft-diagnostics'));assert.ok(raw.includes('revision_not_increased') && raw.includes('entities.ety_other.part.revision') && raw.includes('Native <img src=x> constraint'));assert.equal(find('live-ontology-draft-diagnostics').querySelectorAll('img').length,0);
  button=await validate('invalid_value_type','entities.ety_record.properties.age.part.<img>');inspect(button,'property-choice');assert.equal(find('live-ontology-property-choice').value,'2');assert.equal(find('live-ontology-property-name').value,'age.part.<img>');assert.equal(find('live-ontology-property-kind').value,'integer');
  button=await validate('primary_key_not_required','entities.ety_record.properties.name');inspect(button,'property-required');assert.equal(find('live-ontology-property-choice').value,'0');
  button=await validate('frozen_field_changed','links.lty_reports.part.cardinality');inspect(button,'link-cardinality');assert.match(find('live-ontology-draft-selection').textContent,/lty_reports.part/);assert.equal(find('live-ontology-link-cardinality').value,'one_to_many');
  button=await validate('empty_property_name','entities.ety_other.part');assert.match(button.textContent,/^Inspect type /);inspect(button,'draft-inspector-title');
  button=await validate('invalid_value_type','entities.ety_record.properties.structure');inspect(button,'property-choice');assert.equal(find('live-ontology-property-choice').value,'1');assert.equal(find('live-ontology-property-kind').disabled,true,'accepted value type diagnostics retain immutable declaration inspection');
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();
  button=await validate('invalid_value_type','entities.ety_record.properties.age.part.<img>');inspect(button,'draft-inspector-title');assert.equal(find('live-ontology-property-kind').disabled,true);assert.equal(find('live-ontology-entity-name').disabled,true,'readonly publisher inspection grants no editing');
  modelWorkspace.capabilities.author=true;modelWorkspace.capabilities.publish=false;session.roles=['foundry-model-author','foundry-operator'];await restore();
  button=await validate('unknown_code','entities.ety_record');assert.equal(button,undefined);assert.ok(text(find('live-ontology-draft-diagnostics')).includes('unknown_code'));
  button=await validate('invalid_value_type','entities.ety_record.properties.age.part.<img>');
  find('live-ontology-mode-objects').emit('click');find('live-ontology-query').focus();refuse(button);assert.equal(find('live-ontology-type').value,publishedType);assert.equal(find('live-ontology-revision').value,publishedRevision);
  find('live-ontology-mode-schema').emit('click');assert.ok(text(find('live-ontology-draft-diagnostics')).includes('invalid_value_type'));inspect(button,'property-choice');
  button=await validate('revision_not_increased','entities.ety_other.part.revision');
  find('live-ontology-draft-inspector').hidden=true;refuse(button);find('live-ontology-draft-inspector').hidden=false;button.isConnected=false;refuse(button);button.isConnected=true;
  holdPath='/validate';find('live-ontology-draft-validate').emit('click');await flush();
  assert.equal(find('live-ontology-entity-name').disabled,true);assert.equal(find('live-ontology-new-property-kind').disabled,true,'ordinary fields are disabled while validating');refuse(button);
  find('live-ontology-property-choice').focus();find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');const focused=document.activeElement;held();await flush();assert.equal(document.activeElement,focused,'validation completion never steals focus');
  button=find('live-ontology-draft-diagnostics').querySelectorAll('button')[0];inspect(button,'entity-revision');
  find('live-ontology-entity-name').focus();find('live-ontology-entity-name').value='Pending name';find('live-ontology-entity-name').emit('input');assert.equal(find('live-ontology-draft-diagnostics').children.length,0);refuse(button);
  find('live-ontology-entity-edit').emit('submit');assert.match(find('live-ontology-draft-state').textContent,/Unsaved/);refuse(button);
  modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/);refuse(button);
  find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 2/);refuse(button);
  async function reopen(){modelValidation=null;await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();}
  for(const override of [{draft_id:'9007199254740994'},{draft_version:'1'},{base_generation:'1'}]){
    await reopen();button=await validate('revision_not_increased','entities.ety_record.revision');modelValidation={...modelValidation,...override};find('live-ontology-draft-validate').emit('click');await flush();assert.equal(find('live-ontology-draft-diagnostics').children.length,0,'wrong saved validation identity/version/base is refused');refuse(button);
  }
  await reopen();button=await validate('revision_not_increased','entities.ety_record.revision');holdPath='/validate';find('live-ontology-draft-validate').emit('click');await flush();
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');refuse(button);held();await flush();assert.equal(root.hidden,true);assert.equal(find('live-ontology-draft-diagnostics').children.length,0,'late prior-account diagnostics cannot restore the editor');
  find('sign-in-account').value='usr_alice';await reopen();button=await validate('revision_not_increased','entities.ety_record.revision');
  const draft=modelDrafts.get(id);draft.publication={generation:String(BigInt(draft.base_generation)+1n),prior_generation:draft.base_generation,draft_id:id,draft_version:draft.version,owner:draft.owner};find('live-ontology-draft-reload').emit('click');await flush();assert.equal(find('live-ontology-draft-validate').disabled,true);refuse(button);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}
