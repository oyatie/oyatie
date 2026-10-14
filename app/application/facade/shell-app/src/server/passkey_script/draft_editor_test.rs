use super::explorer_test::check_controller;
const CHECK: &str = r#"
(async()=>{
  assert.equal(vm.runInNewContext("ontologyModelEqual({a:null,b:[1,2]},{b:[1,2],a:null})",context),true,'ack equality uses semantic map fields');
  assert.equal(vm.runInNewContext("ontologyModelEqual([1,2],[2,1])",context),false,'array order is native meaning');
  assert.equal(vm.runInNewContext("ontologyModelEqual({a:null},{})",context),false,'legacy null remains distinct from omitted field');
  await flush();modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.equal(root.hidden,false,'model-only verified member enters the workspace');
  assert.equal(find('verified-ontology-card').hidden,true,'denied status stays independent');
  assert.equal(find('live-ontology-draft-controls').hidden,false);assert.match(find('live-ontology-draft-state').textContent,/Published.*9007199254740993/);
  assert.equal(find('live-ontology-type').children.length,0,'object schemas remain refused');
  assert.equal(find('live-ontology-draft-new').disabled,false);
  find('live-ontology-draft-new').emit('click');await flush();
  const firstId='9007199254740993';assert.match(find('live-ontology-draft-state').textContent,/Draft 9007199254740993.*Saved.*version 1.*9007199254740993/);
  const original=structuredClone(modelDrafts.get(firstId).document);
  find('live-ontology-entity-name').value='<script>Edited name</script>';find('live-ontology-entity-name').emit('input');
  assert.match(find('live-ontology-draft-state').textContent,/Unapplied form edits/);assert.equal(find('live-ontology-draft-validate').disabled,true);
  intervals[0]();await flush();assert.equal(find('live-ontology-entity-name').value,'<script>Edited name</script>','same-identity fresh access preserves pending form edits');
  find('live-ontology-entity-revision').value='2';find('live-ontology-entity-edit').emit('submit');
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved/);assert.match(text(find('live-ontology-draft-diff')),/display_name.*Edited name/);
  const declarationCopy=()=>['schema-summary','model-summary','model-table','library-types','library-links','scene','link-properties'].map(id=>text(find('live-ontology-'+id))).join(' ');
  assert.match(find('live-ontology-schema-summary').textContent,/Edited name.*ety_record.*declaration revision 2/);
  assert.doesNotMatch(declarationCopy(),/\b(accepted|registered)\b/i,'edited unpublished declarations cannot claim acceptance or registration');
  assert.equal(find('live-ontology-type').children.length,0,'draft graph selection does not grant object types');
  find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 2/);
  assert.deepEqual(modelDrafts.get(firstId).document.entities[0].properties,original.entities[0].properties,'full structured/legacy fields remain byte-equivalent native values');
  assert.deepEqual(modelDrafts.get(firstId).document.entities[0].display,original.entities[0].display,'hidden native display metadata remains untouched');
  find('live-ontology-draft-validate').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/Saved changes pass schema validation.*version 2.*Publishing is not available yet/);
  find('live-ontology-new-entity-id').value='ety_unpublished';find('live-ontology-new-entity-name').value='Unpublished';find('live-ontology-entity-add').emit('submit');
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved/);assert.match(text(find('live-ontology-scene')),/Unpublished/);assert.equal(find('live-ontology-draft-diagnostics').children.length,0,'local edits invalidate exact saved validation');
  assert.match(find('live-ontology-schema-summary').textContent,/Unpublished.*ety_unpublished.*declaration revision 1/);
  assert.doesNotMatch(declarationCopy(),/\b(accepted|registered)\b/i,'new working type has declaration provenance');
  assert.match(find('live-ontology-model-note').textContent,/Working draft.*Object reads use published declarations/);
  for(const [name,kind,presence] of [['birth_date','date','required'],['recorded_at','timestamp','optional']]){
    for(const [field,value] of [['new-property-name',name],['new-property-kind',kind],['new-property-required',presence]]){find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');}
    find('live-ontology-property-add').emit('submit');
  }
  assert.equal(find('live-ontology-type').children.length,0,'unpublished type never enters Objects selection');
  find('live-ontology-new-link-cardinality').value='many_to_many';find('live-ontology-new-link-cardinality').emit('input');
  find('live-ontology-new-link-id').value='lty_record_child';find('live-ontology-new-link-from').value='ety_record';find('live-ontology-new-link-to').value='ety_unpublished';find('live-ontology-link-add').emit('submit');
  assert.match(text(find('live-ontology-scene')),/lty_record_child/,'real same-tenant native declaration joins working graph');
  assert.match(text(find('live-ontology-link-properties')),/many → many \(many_to_many\)/,'creation uses the selected cardinality immediately');
  find('live-ontology-link-cardinality').value='many_to_many';find('live-ontology-link-edit').emit('submit');
  assert.match(text(find('live-ontology-link-properties')),/many.*many/,'relationship editing reflects actual native cardinality');
  assert.doesNotMatch(declarationCopy(),/\b(accepted|registered)\b/i,'working relationships use the same declaration provenance');
  find('live-ontology-mode-objects').emit('click');assert.equal(find('live-ontology-draft-inspector').hidden,true);assert.equal(find('live-ontology-read').disabled,true);
  find('live-ontology-mode-schema').emit('click');assert.match(text(find('live-ontology-scene')),/Unpublished/,'working model restores independently');
  modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/);assert.equal(modelDrafts.get(firstId).version,'3');
  const created=modelDrafts.get(firstId).document.entities.find(e=>e.id==='ety_unpublished');
  assert.deepEqual(created.properties.slice(1).map(p=>[p.name,p.value_type,p.required,p.data_class]),[['birth_date',{shape:'scalar',kind:'date'},true,'InternalOnly'],['recorded_at',{shape:'scalar',kind:'timestamp'},false,'InternalOnly']],'creation saves the chosen native shapes without intermediate default declarations');
  assert.equal(created.properties.length,3);assert.equal(created.revision,1,'creation keeps the explicitly selected revision');
  assert.equal(modelDrafts.get(firstId).document.links[0].cardinality,'many_to_many');assert.equal(modelDrafts.get(firstId).document.links[0].allow_cross_tenant,false);
  for(const [field,values] of [['new-property-kind',['string','integer','double','boolean','date','timestamp','array','struct']],['new-property-required',['optional','required']],['new-link-cardinality',['one_to_many','one_to_one','many_to_many']]]){
    assert.ok(input.html.includes(`for="live-ontology-${field}"`),'each creation choice has an associated label');
    const select=input.html.match(new RegExp(`<select\\b[^>]*id="live-ontology-${field}"[^>]*>(.*?)</select>`));assert.ok(select,'creation select is in the rendered shell');
    assert.deepEqual([...select[1].matchAll(/<option\b[^>]*value="([^"]+)"/g)].map(match=>match[1]),values,'only native creation choices are exposed');
  }
  const submittedSaves=calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length;
  assert.equal(find('live-ontology-draft-save').disabled,true,'uncertain save cannot be silently retried');
  find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 3/);assert.match(find('live-ontology-draft-status').textContent,/submission recovered/);
  assert.equal(calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length,submittedSaves,'reconciliation reopens without resubmission');
  [...find('live-ontology-library-types').querySelectorAll('[data-model-type]')].find(e=>e.dataset.modelType==='ety_unpublished').emit('click');find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');find('live-ontology-property-kind').value='timestamp';find('live-ontology-entity-revision').value='2';find('live-ontology-property-edit').emit('submit');
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved/);
  modelTamper=true;find('live-ontology-draft-save').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/,'wrong-version acknowledgement never claims Saved');
  find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 4/);
  find('live-ontology-entity-name').value='Submitted';find('live-ontology-entity-revision').value='3';find('live-ontology-entity-edit').emit('submit');
  holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  find('live-ontology-entity-name').value='Newer pending form text';find('live-ontology-entity-name').emit('input');held();await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved.*Unapplied/,'late save ack cannot mark newer pending form state saved');assert.equal(find('live-ontology-entity-name').value,'Newer pending form text');
  assert.equal(find('live-ontology-draft-save').disabled,true);find('live-ontology-entity-edit').emit('submit');
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved/);find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 5/);
  nativeError=null;statusError=null;await restore();assert.equal(find('live-ontology-type').value,'ety_record');
  assert.equal(find('verified-ontology-card').hidden,false);statusError='network';await restore();assert.equal(find('verified-ontology-card').hidden,true,'unavailable status clears only that independently authorized surface');assert.equal(find('live-ontology-draft-controls').hidden,false,'status outage does not remove freshly authorized model access');statusError=null;
  const publishedPin=find('live-ontology-revision').value;const newType=[...find('live-ontology-library-types').querySelectorAll('[data-model-type]')].find(e=>e.dataset.modelType==='ety_unpublished');newType.emit('click');
  assert.equal(find('live-ontology-type').value,'ety_record','working draft selection preserves published object selection');assert.equal(find('live-ontology-revision').value,publishedPin);
  find('live-ontology-draft-choice').value='published';find('live-ontology-draft-choice').emit('change');await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Published.*9007199254740993/);assert.match(find('live-ontology-model-note').textContent,/Coherent published metadata snapshot/);
  assert.match(find('live-ontology-schema-summary').textContent,/Record.*ety_record.*declaration revision 1/);assert.doesNotMatch(text(find('live-ontology-model-table')),/ety_unpublished|Edited name/);
  find('live-ontology-mode-objects').emit('click');assert.equal(find('live-ontology-type').children.some(e=>e.value==='ety_unpublished'),false);
  assert.equal(find('live-ontology-type').value,'ety_record');assert.equal(find('live-ontology-revision').value,publishedPin);
  find('live-ontology-query').emit('submit');await flush();const readUrl=calls.filter(([u])=>u.includes('/objects?')).at(-1)[0];
  assert.equal(new URL('https://shell.invalid'+readUrl).searchParams.get('type'),'ety_record');assert.equal(new URL('https://shell.invalid'+readUrl).searchParams.get('revision'),publishedPin);
  assert.match(text(find('live-ontology-page')),/ent_alpha/,'Objects still renders published native data');
  find('live-ontology-mode-schema').emit('click');find('live-ontology-draft-choice').value=firstId;find('live-ontology-draft-choice').emit('change');await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Draft.*Saved.*version 5/);assert.match(text(find('live-ontology-scene')),/ety_unpublished/);
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher'];await restore();
  assert.equal(find('live-ontology-draft-validate').disabled,false,'publisher-only owner can validate an exact saved native draft');assert.equal(find('live-ontology-draft-save').disabled,true);
  find('live-ontology-draft-validate').emit('click');await flush();assert.match(find('live-ontology-draft-status').textContent,/Saved changes pass schema validation/);
  const beforePublisherEdit=modelDrafts.get(firstId).document.entities[0].display_name;find('live-ontology-entity-name').value='Publisher cannot edit';find('live-ontology-entity-edit').emit('submit');assert.equal(modelDrafts.get(firstId).document.entities[0].display_name,beforePublisherEdit,'publisher validation never grants author editing');
  find('live-ontology-draft-reload').emit('click');await flush();
  modelWorkspace.capabilities.author=true;modelWorkspace.capabilities.publish=false;session.roles=['foundry-model-author'];await restore();
  holdPath='/model/drafts/';find('live-ontology-draft-reload').emit('click');await flush();
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(root.hidden,true);assert.equal(find('live-ontology-draft-choice').children.length,0);assert.equal(find('live-ontology-entity-name').value,'');assert.equal(find('live-ontology-draft-diff').children.length,0);
  held();await flush();assert.equal(root.hidden,true,'late prior-account document cannot restore the editor');assert.equal(find('live-ontology-draft-state').textContent,'');
  find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=firstId;find('live-ontology-draft-choice').emit('change');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 5/,'owned native draft survives controller reopen');
  modelError='forbidden';await restore();assert.equal(find('live-ontology-draft-controls').hidden,true);assert.equal(find('live-ontology-draft-choice').children.length,0);assert.equal(find('live-ontology-entity-name').value,'');assert.equal(find('live-ontology-draft-diff').children.length,0);assert.equal(find('live-ontology-draft-diagnostics').children.length,0);assert.equal(find('live-ontology-model-note').textContent.includes('9007199254740993'),false,'revoked model generation label clears');assert.match(find('live-ontology-schema-summary').textContent,/Record/,'operator object authority retains readonly model after independent model denial');
  modelError=null;await restore();find('session-logout').emit('click');assert.equal(root.hidden,true);assert.equal(find('live-ontology-draft-state').textContent,'');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;
#[test]
fn persistent_draft_editor_keeps_role_identity_versions_and_published_objects_separate() {
    check_controller(CHECK);
}

#[test]
fn pending_property_form_keeps_its_target_until_applied_or_discarded() {
    check_controller(
        r#"
(async()=>{
  await flush();modelDocument.entities[0].properties[0].value_type={shape:'scalar',kind:'string'};
  modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993', structured=structuredClone(modelDrafts.get(id).document.entities[0].properties[1]);
  find('live-ontology-new-property-name').value='unrelated';find('live-ontology-property-add').emit('submit');
  find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');
  find('live-ontology-property-name').value='intended_first_property';find('live-ontology-property-name').emit('input');
  find('live-ontology-property-kind').value='timestamp';find('live-ontology-property-required').value='optional';find('live-ontology-entity-revision').value='2';
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-choice').value,'0','dirty form must retain its property target');
  assert.equal(find('live-ontology-property-name').value,'intended_first_property');
  assert.equal(find('live-ontology-property-kind').value,'timestamp');assert.equal(find('live-ontology-property-required').value,'optional');
  assert.match(find('live-ontology-draft-status').textContent,/Apply.*discard.*reload.*before changing property/);
  assert.match(find('live-ontology-draft-state').textContent,/Unapplied form edits/);
  assert.equal(modelDrafts.get(id).document.entities[0].properties.length,2,'rejected selection does not submit or apply');
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  let properties=modelDrafts.get(id).document.entities[0].properties;
  assert.equal(properties[0].name,'intended_first_property');assert.equal(properties[0].value_type.kind,'timestamp');assert.equal(properties[0].required,false);
  assert.equal(properties[2].name,'unrelated','saving property 0 never overwrites property 2');assert.deepEqual(properties[1],structured);
  const unrelated=structuredClone(properties[2]);
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-name').value,'unrelated','applied forms allow explicit target changes');
  find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');
  find('live-ontology-entity-name').value='Submitted';find('live-ontology-entity-edit').emit('submit');
  holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  find('live-ontology-property-name').value='newer_pending_first_property';find('live-ontology-property-name').emit('input');
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-choice').value,'0','pending save cannot retarget newer form text');
  held();await flush();
  assert.equal(find('live-ontology-property-choice').value,'0');assert.equal(find('live-ontology-property-name').value,'newer_pending_first_property');
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved.*Unapplied/);
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  properties=modelDrafts.get(id).document.entities[0].properties;
  assert.equal(properties[0].name,'newer_pending_first_property');assert.deepEqual(properties[2],unrelated);assert.deepEqual(properties[1],structured);
  find('live-ontology-property-name').value='explicitly_discarded';find('live-ontology-property-name').emit('input');
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');assert.equal(find('live-ontology-property-choice').value,'0');
  find('live-ontology-draft-reload').emit('click');await flush();
  find('live-ontology-property-choice').value='2';find('live-ontology-property-choice').emit('change');
  assert.equal(find('live-ontology-property-name').value,'unrelated','explicit discard releases the target fence');
  assert.deepEqual(modelDrafts.get(id).document.entities[0].properties[2],unrelated);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn independently_pending_forms_survive_partial_apply_and_lifecycle() {
    check_controller(
        r#"
(async()=>{
  await flush();modelDocument.entities[0].properties[0].value_type={shape:'scalar',kind:'string'};
  modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993', original=structuredClone(modelDrafts.get(id).document.entities[0]);
  function enter(field,value){find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');}
  function pending(){assert.match(find('live-ontology-draft-state').textContent,/Unsaved.*Unapplied form edits/);assert.equal(find('live-ontology-draft-save').disabled,true);assert.equal(find('live-ontology-draft-validate').disabled,true);}
  const defaults=[['new-property-kind','string'],['new-property-required','optional'],['new-link-cardinality','one_to_many']];
  function defaultChoices(){for(const [field,value] of defaults){assert.equal(find('live-ontology-'+field).value,value,'creation defaults are explicit');assert.equal(find('live-ontology-'+field)._value,value,'reset assigns defaults instead of relying on the mock first-option fallback');}}defaultChoices();
  enter('entity-revision','2');find('live-ontology-property-edit').emit('submit');
  assert.equal(find('live-ontology-entity-revision').value,'2','unchanged property Apply preserves a revision it did not apply');pending();
  assert.deepEqual(modelDrafts.get(id).document.entities[0],original);
  find('live-ontology-entity-edit').emit('submit');assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);
  find('live-ontology-draft-save').emit('click');await flush();assert.equal(modelDrafts.get(id).document.entities[0].revision,2);
  enter('entity-name','Pending entity name');enter('property-name','Applied property name');enter('entity-revision','2');
  find('live-ontology-property-edit').emit('submit');
  assert.equal(find('live-ontology-entity-name').value,'Pending entity name');pending();
  const saves=calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length;
  find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length,saves,'unapplied forms cannot submit a partial save');
  intervals[0]();await flush();
  assert.equal(find('live-ontology-entity-name').value,'Pending entity name');assert.equal(find('live-ontology-property-name').value,'Applied property name');pending();
  find('live-ontology-entity-edit').emit('submit');assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);
  find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].display_name,'Pending entity name');assert.equal(modelDrafts.get(id).document.entities[0].properties[0].name,'Applied property name');
  enter('property-name','Pending property name');enter('entity-name','Applied entity name');
  find('live-ontology-entity-edit').emit('submit');assert.equal(find('live-ontology-property-name').value,'Pending property name');pending();
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');assert.equal(find('live-ontology-property-choice').value,'0');
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].display_name,'Applied entity name');assert.equal(modelDrafts.get(id).document.entities[0].properties[0].name,'Pending property name');
  enter('entity-name','Submitted entity');find('live-ontology-entity-edit').emit('submit');
  holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  enter('entity-name','Ack pending entity');enter('property-name','Ack pending property');held();await flush();
  assert.equal(find('live-ontology-entity-name').value,'Ack pending entity');assert.equal(find('live-ontology-property-name').value,'Ack pending property');pending();
  find('live-ontology-property-edit').emit('submit');assert.equal(find('live-ontology-entity-name').value,'Ack pending entity');pending();
  find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].display_name,'Ack pending entity');assert.equal(modelDrafts.get(id).document.entities[0].properties[0].name,'Ack pending property');
  enter('entity-name','Unknown submitted entity');find('live-ontology-entity-edit').emit('submit');
  holdPath='/model';modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();
  enter('entity-name','Recovered pending entity');enter('property-name','Recovered pending property');held();await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/);
  const unknownSaves=calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length;
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.equal(calls.filter(([u,o])=>u.endsWith('/save') && o.method==='POST').length,unknownSaves,'recovery never resubmits');
  assert.equal(find('live-ontology-entity-name').value,'Recovered pending entity');assert.equal(find('live-ontology-property-name').value,'Recovered pending property');pending();
  find('live-ontology-property-edit').emit('submit');assert.equal(find('live-ontology-entity-name').value,'Recovered pending entity');pending();
  find('live-ontology-entity-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).document.entities[0].display_name,'Recovered pending entity');assert.equal(modelDrafts.get(id).document.entities[0].properties[0].name,'Recovered pending property');
  enter('entity-name','Original target remains pending');enter('new-entity-id','ety_added');enter('new-entity-name','Added type');
  find('live-ontology-entity-add').emit('submit');
  assert.match(find('live-ontology-draft-selection').textContent,/ety_record/);assert.equal(find('live-ontology-entity-name').value,'Original target remains pending');pending();
  find('live-ontology-entity-edit').emit('submit');
  enter('property-name','Original property remains pending');enter('new-property-name','added_property');
  find('live-ontology-property-add').emit('submit');
  assert.equal(find('live-ontology-property-choice').value,'0','adding a property cannot retarget another pending property form');assert.equal(find('live-ontology-property-name').value,'Original property remains pending');pending();
  find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();
  let saved=structuredClone(modelDrafts.get(id).document);
  assert.equal(saved.entities[0].display_name,'Original target remains pending');assert.equal(saved.entities[0].properties[0].name,'Original property remains pending');assert.equal(saved.entities[0].properties[2].name,'added_property');
  assert.equal(saved.entities.find(e=>e.id==='ety_added').display_name,'Added type');assert.deepEqual(saved.entities[0].properties[1],original.properties[1]);assert.deepEqual(saved.entities[0].display,original.display);
  assert.equal(saved.entities[0].properties[2].value_type.kind,'string');assert.equal(saved.entities[0].properties[2].required,false,'untouched creation settings preserve optional String');
  enter('entity-revision','3');find('live-ontology-entity-edit').emit('submit');holdPath='/save';find('live-ontology-draft-save').emit('click');await flush();
  const choices=[['new-property-name','birth_choice'],['new-property-kind','date'],['new-property-required','required'],['new-link-id','lty_choice'],['new-link-cardinality','many_to_many']];
  for(const [field,value] of choices)enter(field,value);held();await flush();pending();
  function heldChoices(){for(const [field,value] of choices)assert.equal(find('live-ontology-'+field).value,value,'pending creation settings survive unrelated lifecycle work');}heldChoices();
  find('live-ontology-entity-edit').emit('submit');pending();heldChoices();intervals[0]();await flush();pending();heldChoices();
  find('live-ontology-property-add').emit('submit');pending();assert.equal(find('live-ontology-new-link-cardinality').value,'many_to_many','property Apply cannot clear relationship choices');
  enter('new-link-from','ety_record');enter('new-link-to','ety_added');find('live-ontology-link-add').emit('submit');assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);
  find('live-ontology-draft-save').emit('click');await flush();saved=structuredClone(modelDrafts.get(id).document);
  assert.deepEqual(saved.entities[0].properties[3],{name:'birth_choice',tier:'scalar',data_class:'InternalOnly',required:true,value_type:{shape:'scalar',kind:'date'},display:null});
  assert.equal(saved.entities[0].revision,3);assert.equal(saved.links[0].cardinality,'many_to_many');assert.equal(saved.links[0].allow_cross_tenant,false);defaultChoices();
  find('live-ontology-library-types').querySelectorAll('[data-model-type]').find(e=>e.dataset.modelType==='ety_record').emit('click');
  for(const [field,bad,message] of [['new-property-kind','enum',/supported native value declaration/],['new-property-required','sometimes',/Required or Optional/],['new-link-cardinality','many_to_one',/native cardinality/]]){
    const option=new Element('option');option.value=bad;find('live-ontology-'+field).append(option);enter(field,bad);
    const relationship=field==='new-link-cardinality';enter(relationship?'new-link-id':'new-property-name',relationship?'lty_refused':'refused_property');
    const before=text(find('live-ontology-draft-diff')),native=structuredClone(modelDrafts.get(id).document);
    find('live-ontology-'+(relationship?'link-add':'property-add')).emit('submit');assert.match(find('live-ontology-draft-status').textContent,message);
    find('live-ontology-entity-edit').emit('submit');assert.match(find('live-ontology-draft-status').textContent,/Unapplied form edits remain/);
    assert.equal(text(find('live-ontology-draft-diff')),before,'invalid choice never mutates the working document');assert.deepEqual(modelDrafts.get(id).document,native);
    find('live-ontology-draft-reload').emit('click');await flush();defaultChoices();assert.deepEqual(modelDrafts.get(id).document,saved);
  }
  enter('entity-name','Discard entity');enter('property-name','Discard property');enter('new-entity-id','ety_discard');enter('new-entity-name','Discard addition');enter('new-property-name','discard_property');
  enter('new-property-kind','timestamp');enter('new-property-required','required');enter('new-link-cardinality','one_to_one');
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.equal(find('live-ontology-entity-name').value,saved.entities[0].display_name);assert.equal(find('live-ontology-property-name').value,saved.entities[0].properties[0].name);
  for(const field of ['new-entity-id','new-entity-name','new-property-name'])assert.equal(find('live-ontology-'+field).value,'','explicit discard clears every pending addition field');
  assert.match(find('live-ontology-draft-state').textContent,/Saved/);assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);assert.deepEqual(modelDrafts.get(id).document,saved);
  defaultChoices();
  enter('entity-name','Prior identity entity');enter('property-name','Prior identity property');enter('new-link-id','lty_prior');
  enter('new-property-kind','double');enter('new-property-required','required');enter('new-link-cardinality','many_to_many');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');
  assert.equal(root.hidden,true);for(const field of ['entity-name','property-name','new-link-id'])assert.equal(find('live-ontology-'+field).value,'');assert.equal(find('live-ontology-draft-state').textContent,'');
  defaultChoices();for(const [field] of defaults)assert.equal(find('live-ontology-'+field).disabled,true,'identity loss disables every creation choice');
  find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Saved/);assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);assert.equal(find('live-ontology-entity-name').value,saved.entities[0].display_name);assert.equal(find('live-ontology-property-name').value,saved.entities[0].properties[0].name);
  defaultChoices();
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
