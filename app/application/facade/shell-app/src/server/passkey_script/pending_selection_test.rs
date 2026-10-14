use super::explorer_test::check_controller;

#[test]
fn pending_nonfirst_type_and_link_selection_survive_verified_refresh() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993';
  function enter(field,value){find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');}
  enter('new-entity-id','ety_team');enter('new-entity-name','Team');find('live-ontology-entity-add').emit('submit');
  find('live-ontology-draft-save').emit('click');await flush();
  const saved=structuredClone(modelDrafts.get(id)),publishedType=find('live-ontology-type').value,publishedRevision=find('live-ontology-revision').value;
  find('live-ontology-library-types').querySelectorAll('[data-model-type]').find(button=>button.dataset.modelType==='ety_team').emit('click');
  function selection(kind,id){
    for(const surface of ['scene',kind==='type'?'library-types':'library-links','model-table']){
      const buttons=find('live-ontology-'+surface).querySelectorAll(`[data-model-${kind}]`);
      assert.ok(buttons.some(button=>button.dataset[kind==='type'?'modelType':'modelLink']===id),'selected declaration exists in every representation');
      for(const button of buttons)assert.equal(button.attrs['aria-pressed'],String(button.dataset[kind==='type'?'modelType':'modelLink']===id),'canvas, library and table agree on the selected declaration');
    }
  }
  function pending(){assert.match(find('live-ontology-draft-state').textContent,/Unapplied form edits/);assert.equal(find('live-ontology-draft-save').disabled,true);assert.equal(find('live-ontology-draft-validate').disabled,true);}
  function team(){
    for(const surface of ['explorer','model','graph','scene','schema','library-types','draft-inspector','property-edit','property-required']){
      const element=find('live-ontology-'+surface);assert.equal(element.isConnected,true);assert.equal(element.closest('[hidden]'),null,'restored model and pending controls must be visible through their ancestors');assert.ok(element.getClientRects().length>0);
    }
    assert.equal(find('live-ontology-property-required').disabled,false,'verified model author can edit the restored pending field');
    assert.match(find('live-ontology-draft-selection').textContent,/Team.*ety_team/);
    assert.equal(find('live-ontology-entity-name').value,'Team');assert.equal(find('live-ontology-property-name').value,'id');
    assert.equal(find('live-ontology-property-required').value,'optional');
    assert.match(find('live-ontology-schema-summary').textContent,/Team.*ety_team.*declaration revision 1/,'pending inspector target and schema must agree after restore');
    selection('type','ety_team');pending();
  }
  enter('property-required','optional');team();
  for(const refresh of [restore,async()=>{intervals[0]();await flush();}]){
    holdPath='/schemas';const requests=calls.length;await refresh();team();
    assert.ok(calls.slice(requests).every(([,options])=>(options.method || 'GET')==='GET'),'refresh does not apply or submit pending edits');
    assert.deepEqual(modelDrafts.get(id),saved,'refresh preserves exact native owner, version, base and saved document');
    assert.equal(find('live-ontology-type').value,publishedType);assert.equal(find('live-ontology-revision').value,publishedRevision);
    assert.equal(find('live-ontology-type').children.some(option=>option.value==='ety_team'),false,'draft selection never grants a published Objects type');
    held();await flush();team();
  }
  find('live-ontology-draft-reload').emit('click');await flush();
  enter('new-link-id','lty_record_team');enter('new-link-from','ety_record');enter('new-link-to','ety_team');find('live-ontology-link-add').emit('submit');
  find('live-ontology-draft-save').emit('click');await flush();
  const linked=structuredClone(modelDrafts.get(id));
  find('live-ontology-library-links').querySelectorAll('[data-model-link]').find(button=>button.dataset.modelLink==='lty_record_team').emit('click');
  enter('link-cardinality','many_to_many');
  await restore();pending();selection('link','lty_record_team');
  assert.match(find('live-ontology-draft-selection').textContent,/lty_record_team/);assert.equal(find('live-ontology-link-cardinality').value,'many_to_many');
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-link').hidden,false);
  assert.match(text(find('live-ontology-link-properties')),/lty_record_team.*one → many/,'readonly inspector keeps the unapplied native relationship declaration');
  assert.deepEqual(modelDrafts.get(id),linked);
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-draft-selection').textContent,'');assert.equal(find('live-ontology-link-cardinality')._value,'');
  assert.equal(find('live-ontology-link-cardinality').disabled,true);
  for(const surface of ['scene','library-types','library-links','model-table']){
    const element=find('live-ontology-'+surface);for(const kind of ['type','link'])assert.equal(element.querySelectorAll(`[data-model-${kind}]`).length,0,'identity loss clears declaration selections');
    assert.doesNotMatch(text(element),/ety_team|lty_record_team/);
  }
  find('sign-in-account').value='usr_alice';await restore();
  assert.match(find('live-ontology-draft-state').textContent,/Published/);assert.doesNotMatch(find('live-ontology-draft-state').textContent,/Unapplied/);
  assert.match(find('live-ontology-schema-summary').textContent,/Record.*ety_record.*declaration revision 1/);selection('type','ety_record');
  assert.equal(find('live-ontology-draft-inspector').hidden,true);assert.doesNotMatch(text(find('live-ontology-scene')),/ety_team|lty_record_team/);
  assert.deepEqual(modelDrafts.get(id),linked,'identity clearing and published fallback do not mutate the saved draft');
  console.log('Pending non-first type/link selection refresh, read-only declaration, Objects isolation and identity cleanup PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

const PENDING_SAVE_CHECK: &str = r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const first='9007199254740993',second='9007199254740994';
  const other=structuredClone(modelDrafts.get(first));other.id=second;other.document.entities[0].display_name='Other saved draft';modelDrafts.set(second,other);
  await restore();
  function enter(field,value){find('live-ontology-'+field).value=value;find('live-ontology-'+field).emit('input');}
  enter('entity-name','Submitted change');enter('entity-revision','2');find('live-ontology-entity-edit').emit('submit');
  const realFetch=context.fetch;let releaseFailure;
  context.fetch=(url,options)=>{const result=realFetch(url,options);if(url.endsWith('/save'))return new Promise((_,reject)=>releaseFailure=()=>reject(new Error('network disconnected after commit')));return result;};
  find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(find('live-ontology-draft-choice').disabled,true,'in-flight selection is unavailable');
  enter('entity-name','Newer local entity');enter('property-name','newer_local_property');releaseFailure();await flush();context.fetch=realFetch;
  assert.equal(modelDrafts.get(first).version,'2','save committed before its acknowledgement was lost');
  const choice=find('live-ontology-draft-choice'),requests=calls.length;
  assert.equal(choice.disabled,false,'the outcome-unknown selector is reachable');assert.ok(choice.getClientRects().length);
  choice.focus();choice.value=destination==='same'?first:second;choice.emit('change');await flush();
  assert.equal(choice.value,first,'an uncertain save cannot change the selected draft');
  assert.equal(find('live-ontology-entity-name').value,'Newer local entity','reopening cannot discard newer local entity text');assert.equal(find('live-ontology-property-name').value,'newer_local_property');
  assert.equal(document.activeElement,choice,'refusal retains selector focus');assert.equal(calls.length,requests,'refusal neither reads another draft nor retries the save');
  assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown.*version 1/);
  assert.match(find('live-ontology-draft-status').textContent,/Reconcile.*before changing models/);
  assert.match(text(find('live-ontology-draft-diff')),/Submitted change/,'the submitted working document is retained too');
  const saves=calls.filter(([url,options])=>url.endsWith('/save') && options.method==='POST').length;
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.match(find('live-ontology-draft-status').textContent,/Saved submission recovered; newer local edits remain unsaved/);
  assert.match(find('live-ontology-draft-state').textContent,/Unsaved.*Unapplied.*version 2/);
  assert.equal(find('live-ontology-entity-name').value,'Newer local entity');assert.equal(find('live-ontology-property-name').value,'newer_local_property');
  assert.equal(calls.filter(([url,options])=>url.endsWith('/save') && options.method==='POST').length,saves,'reconciliation never retries the mutation');
  const reconciledRequests=calls.length;choice.value=second;choice.emit('change');await flush();
  assert.equal(choice.value,first);assert.equal(calls.length,reconciledRequests);assert.match(find('live-ontology-draft-status').textContent,/Save or explicitly discard/);
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.equal(find('live-ontology-entity-name').value,'Submitted change');assert.equal(find('live-ontology-property-name').value,'name');
  assert.match(find('live-ontology-draft-status').textContent,/local edits discarded/);
  choice.value=second;choice.emit('change');await flush();
  assert.equal(choice.value,second);assert.equal(find('live-ontology-entity-name').value,'Other saved draft');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;

#[test]
fn uncertain_save_refuses_switching_to_another_owned_draft() {
    check_controller(&format!("const destination='other';{PENDING_SAVE_CHECK}"));
}

#[test]
fn uncertain_save_refuses_reopening_the_same_owned_draft() {
    check_controller(&format!("const destination='same';{PENDING_SAVE_CHECK}"));
}

#[test]
fn failed_save_before_commit_retains_applied_edits_until_explicit_discard() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const first='9007199254740993',second='9007199254740994';
  const other=structuredClone(modelDrafts.get(first));other.id=second;other.document.entities[0].display_name='Other saved draft';modelDrafts.set(second,other);await restore();
  assert.equal(find('live-ontology-entity-name').disabled,false);
  find('live-ontology-entity-name').value='Applied local change';find('live-ontology-entity-revision').value='2';find('live-ontology-entity-edit').emit('submit');
  const realFetch=context.fetch;let saveAttempts=0;
  context.fetch=(url,options)=>{if(url.endsWith('/save')){saveAttempts++;return Promise.reject(new Error('network failed before native commit'));}return realFetch(url,options);};
  find('live-ontology-draft-save').emit('click');await flush();
  const choice=find('live-ontology-draft-choice'),requests=calls.length;assert.equal(choice.disabled,false);assert.ok(choice.getClientRects().length);
  assert.equal(modelDrafts.get(first).version,'1');choice.focus();choice.value=second;choice.emit('change');await flush();
  assert.equal(choice.value,first,'failed save cannot discard applied working edits through draft navigation');
  assert.equal(find('live-ontology-entity-name').value,'Applied local change');assert.match(text(find('live-ontology-draft-diff')),/Applied local change/);
  assert.match(find('live-ontology-draft-state').textContent,/Outcome unknown/);assert.equal(calls.length,requests);
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.match(find('live-ontology-draft-status').textContent,/does not match.*Local edits are retained/);assert.match(find('live-ontology-draft-state').textContent,/Conflict/);
  assert.equal(find('live-ontology-entity-name').value,'Applied local change');assert.match(text(find('live-ontology-draft-diff')),/Applied local change/);
  choice.value=second;choice.emit('change');await flush();assert.equal(choice.value,first);assert.match(find('live-ontology-draft-status').textContent,/Save or explicitly discard/);
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.equal(find('live-ontology-entity-name').value,'Record');assert.match(find('live-ontology-draft-status').textContent,/local edits discarded/);
  choice.value=second;choice.emit('change');await flush();assert.equal(choice.value,second);assert.equal(find('live-ontology-entity-name').value,'Other saved draft');
  assert.equal(saveAttempts,1,'reconciliation and navigation never retry the save');assert.equal(modelDrafts.get(first).version,'1');context.fetch=realFetch;
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn uncertain_creation_still_allows_explicit_saved_draft_inspection() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  modelLoseAck=true;find('live-ontology-draft-new').emit('click');await flush();
  assert.match(find('live-ontology-draft-status').textContent,/outcome unknown/);
  const creates=calls.filter(([url,options])=>url.endsWith('/drafts') && options.method==='POST').length;
  find('live-ontology-draft-reload').emit('click');await flush();
  assert.match(find('live-ontology-draft-status').textContent,/Choose an owned draft to inspect/);
  const choice=find('live-ontology-draft-choice');assert.equal(choice.disabled,false);
  choice.value='published';choice.emit('change');await flush();
  assert.match(find('live-ontology-draft-status').textContent,/Reconcile.*before changing models/);
  choice.value='9007199254740993';choice.emit('change');await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Draft 9007199254740993.*Saved.*version 1/);
  assert.match(find('live-ontology-draft-status').textContent,/Saved owned draft opened/);
  assert.equal(calls.filter(([url,options])=>url.endsWith('/drafts') && options.method==='POST').length,creates,'inspection never retries creation');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
