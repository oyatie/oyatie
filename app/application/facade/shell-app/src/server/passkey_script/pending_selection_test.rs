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
