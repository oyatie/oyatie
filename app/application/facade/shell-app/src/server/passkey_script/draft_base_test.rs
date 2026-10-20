use super::check_controller;

#[test]
fn changes_use_exact_saved_base_and_preserve_local_work_when_comparison_fails() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const id='9007199254740993',base=modelWorkspace.generation;
  modelWorkspace.document=structuredClone(modelDocument);modelWorkspace.generation='9007199254740994';
  modelWorkspace.document.entities[0].display_name='HEAD ONLY';modelWorkspace.document.entities[0].primary_key_property='name';
  await restore();
  const changes=()=>text(find('live-ontology-draft-diff'));
  assert.match(changes(),new RegExp(`draft base.*generation ${base}`),'comparison names exact historical base');
  assert.doesNotMatch(changes(),/HEAD ONLY/,'current head is never substituted for the saved base');
  assert.equal(find('live-ontology-entity-primary').disabled,true,'current head still fixes the form primary key');
  assert.match(changes(),/current published generation 9007199254740994/i);
  find('live-ontology-entity-name').value='Local working';find('live-ontology-entity-revision').value='2';find('live-ontology-entity-edit').emit('submit');
  assert.match(changes(),/Local working/);assert.doesNotMatch(changes(),/HEAD ONLY/);
  modelBaseError='model_unavailable';holdPath='/base?';find('live-ontology-draft-save').emit('click');await flush();
  assert.equal(modelDrafts.get(id).version,'2','native save has already succeeded');
  assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 2/,'accepted save remains acknowledged while exact comparison loads');
  held();await flush();
  assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 2/);
  assert.match(find('live-ontology-draft-status').textContent,/Exact submitted document saved/,'failed subsequent GET preserves the accepted save acknowledgement');
  find('live-ontology-entity-name').value='Unapplied text';find('live-ontology-entity-name').emit('input');await restore();
  assert.match(find('live-ontology-draft-state').textContent,/Unapplied.*version 2/);
  assert.equal(find('live-ontology-entity-name').value,'Unapplied text');
  assert.doesNotMatch(find('live-ontology-draft-status').textContent,/Outcome unknown|Submission outcome unknown/,'failed subsequent GET cannot make an accepted save uncertain');
  assert.match(changes(),/unavailable/i);assert.doesNotMatch(changes(),/HEAD ONLY|Local working|Outcome unknown/);
  assert.equal(find('live-ontology-draft-choice').value,id);assert.equal(find('live-ontology-draft-controls').hidden,false);
  modelBaseError=null;await restore();assert.equal(find('live-ontology-entity-name').value,'Unapplied text');assert.match(changes(),/Local working/);
  assert.equal(calls.filter(([url,options])=>url.endsWith('/save') && options.method==='POST').length,1);
  for(const override of [{draft_id:'2'},{draft_version:'1'},{base_generation:'1'},{unexpected:true},{document:{entities:[],links:[],unexpected:true}}]){
    modelBaseTamper=override;await restore();assert.match(changes(),/unavailable/i);assert.doesNotMatch(changes(),/Local working|HEAD ONLY/);assert.equal(find('live-ontology-entity-name').value,'Unapplied text');
  }
  modelBaseTamper=null;holdPath='/base?';await restore();const release=held;
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');release();await flush();
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-draft-diff').children.length,0,'late prior-account base cannot restore comparison');
  find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  modelBaseError='forbidden';await restore();assert.equal(find('live-ontology-draft-controls').hidden,true);assert.equal(find('live-ontology-entity-name').value,'');assert.equal(find('live-ontology-draft-diff').children.length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn base_codec_is_closed_and_auth_status_clears_even_with_malformed_error_body() {
    check_controller(
        r#"
(async()=>{
  context.baseSaved={id:'9007199254740993',version:'1',base_generation:'9007199254740993'};
  const wire={draft_id:context.baseSaved.id,draft_version:'1',base_generation:context.baseSaved.base_generation,document:modelDocument};
  function decode(value){context.baseCandidate=value;return vm.runInNewContext('ontologyModelWire.base(baseCandidate,baseSaved)',context);}
  decode(wire);
  for(const override of [{draft_id:'1'},{draft_version:'2'},{base_generation:'1'},{draft_version:'01'},{draft_id:'9223372036854775808'},{draft_version:1},{document:{}},{extra:true},{tenant_id:'ten_acme'}])assert.throws(()=>decode({...wire,...override}),/model_unavailable/);
  const missing={...wire};delete missing.document;assert.throws(()=>decode(missing),/model_unavailable/);
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();const id='9007199254740993';
  for(const code of ['unauthenticated','forbidden']){
    modelBaseError=code;modelBaseMalformed=true;await restore();
    assert.equal(find('live-ontology-draft-controls').hidden,true,`${code} status clears protected state before parsing its body`);
    assert.equal(find('live-ontology-entity-name').value,'');assert.equal(find('live-ontology-draft-diff').children.length,0);
    modelBaseError=null;modelBaseMalformed=false;await restore();find('live-ontology-draft-choice').value=id;find('live-ontology-draft-choice').emit('change');await flush();
  }
  holdPath='/base?';await restore();const old=held;modelDrafts.get(id).version='2';await restore();
  assert.match(text(find('live-ontology-draft-diff')),/unavailable/i);old();await flush();
  assert.match(text(find('live-ontology-draft-diff')),/unavailable/i,'late exact old tuple cannot supersede a newer authority epoch');
  find('live-ontology-draft-reload').emit('click');await flush();assert.match(find('live-ontology-draft-state').textContent,/Saved.*version 2/);
  assert.match(text(find('live-ontology-draft-diff')),/draft base/,'explicit native reload acquires the new exact tuple');
  find('live-ontology-draft-choice').value='published';find('live-ontology-draft-choice').emit('change');await flush();assert.equal(find('live-ontology-draft-diff').children.length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
