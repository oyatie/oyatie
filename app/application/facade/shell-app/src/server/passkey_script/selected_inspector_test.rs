use super::explorer_test::check_controller;

#[test]
fn selected_context_keeps_long_literal_parts_and_exact_details_distinguishable() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  const typeName='Employee · Property → 人事と勤務先の管理 · Équipe internationale',propertyName='勤務先 · Draft relationship → work_address';
  const idA='ety_'+ 'x'.repeat(4079)+'A_shared_tail',idB='ety_'+ 'x'.repeat(4079)+'B_shared_tail';
  assert.equal(new TextEncoder().encode(idA).length,4096);assert.equal(idA.slice(0,64),idB.slice(0,64));assert.equal(idA.slice(-8),idB.slice(-8));
  modelDocument.entities[0].display_name=typeName;modelDocument.entities[0].properties[1].name=propertyName;
  const longName='名'.repeat(1365)+'x',longProperty='p'.repeat(4096);assert.equal(new TextEncoder().encode(longName).length,4096);
  for(const id of [idA,idB]){const entity={...JSON.parse(JSON.stringify(modelDocument.entities[0])),id,display_name:longName};entity.properties[1].name=longProperty;modelDocument.entities.push(entity);}
  const longLink='lty_'+ 'y'.repeat(4092);modelDocument.links.push({id:longLink,from_entity_type:idA,to_entity_type:idB,cardinality:'many_to_many',allow_cross_tenant:false,revision:1,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const parts=()=>['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent),details=()=>text(find('live-ontology-selection-values'));
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  assert.equal(parts()[0],'Property · '+propertyName,'selected property has its own literal bounded row');assert.equal(parts()[1],'Draft · ety_record');assert.equal(parts()[2],typeName);assert.ok(details().includes(propertyName)&&details().includes(typeName));
  const before=JSON.stringify({parts:parts(),details:details()});find('live-ontology-property-name').value='Pending · Property → rename';find('live-ontology-property-name').emit('input');find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.equal(JSON.stringify({parts:parts(),details:details()}),before,'pending name and refused relationship do not retarget any context part');
  find('live-ontology-draft-reload').emit('click');await flush();const buttons=()=>find('live-ontology-library-types').querySelectorAll('[data-model-type]');
  for(const id of [idA,idB]){buttons().find(b=>b.dataset.modelType===id).emit('click');find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');assert.equal(parts()[0],'Property · '+longProperty);assert.equal(parts()[1],'Draft · '+id);assert.equal(parts()[2],longName);assert.ok(details().includes(id)&&details().includes(longProperty)&&details().includes(longName),'exact full admitted values stay inspectable even for identical abbreviated prefixes/suffixes');}
  find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.equal(parts()[0],'Relationship · '+longLink);assert.equal(parts()[1],'Draft · '+idA+' → '+idB);assert.ok(details().includes(longLink)&&details().includes(idA)&&details().includes(idB));
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();assert.equal(find('live-ontology-link-cardinality').disabled,true);assert.ok(details().includes(longLink),'readonly publisher retains exact applied relationship context');
  find('live-ontology-draft-choice').value='published';find('live-ontology-draft-choice').emit('change');await flush();buttons()[0].emit('click');assert.equal(parts()[0],'Type · ety_record');assert.equal(parts()[1],'Published model');assert.equal(parts()[2],typeName);
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.deepEqual(parts(),['','','']);assert.equal(details(),'');assert.equal(find('live-ontology-selection-details').hidden,true,'identity clearing removes exact context too');
  const start=input.html.indexOf('id="live-ontology-inspector-title"'),end=input.html.indexOf('</summary>',start),full=input.html.indexOf('id="live-ontology-selection-details"');assert.ok(start>=0&&full>end,'ordinary exact-context disclosure is outside the sticky interactive summary');assert.ok(input.html.includes('Full selection details'));
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn refused_relationship_selection_preserves_the_current_applied_inspector() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.links.push({id:'lty_self',from_entity_type:'ety_record',to_entity_type:'ety_record',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null});
  nativeLinks=[{link_type:'lty_published',from_entity_type:'ety_record',to_entity_type:'ety_profile',cardinality:'one_to_many',allow_cross_tenant:false,revision:1}];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const heading=()=>['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent).filter(Boolean).join(' · ');
  const state=()=>JSON.stringify({heading:heading(),selection:find('live-ontology-draft-selection').textContent,property:find('live-ontology-property-choice').value,schemaHidden:find('live-ontology-schema').hidden,linkHidden:find('live-ontology-link').hidden,pins:[find('live-ontology-type').value,find('live-ontology-revision').value],pressed:['library-types','library-links','scene','model-table'].flatMap(id=>find('live-ontology-'+id).querySelectorAll('button').filter(button=>button.dataset.modelType||button.dataset.modelLink).map(button=>[button.dataset.modelType||button.dataset.modelLink,button.attrs['aria-pressed']]))});
  for(const field of ['entity-name','property-name'])for(const surface of ['library-links','scene','model-table']){
    find('live-ontology-draft-reload').emit('click');await flush();find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
    find('live-ontology-'+field).value='Pending retarget '+field;find('live-ontology-'+field).emit('input');const before=state(),requests=calls.length;
    const button=find('live-ontology-'+surface).querySelectorAll('[data-model-link]')[0];button.focus();button.emit('click');
    assert.equal(state(),before,'refused dirty relationship click keeps heading, fields, metadata visibility and pressed context');assert.equal(find('live-ontology-'+field).value,'Pending retarget '+field);assert.equal(calls.length,requests);
  }
  find('live-ontology-draft-reload').emit('click');await flush();modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();
  let requests=calls.length;find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.match(heading(),/^Relationship.*lty_self.*Draft/);assert.equal(find('live-ontology-link').hidden,false);assert.equal(find('live-ontology-link-cardinality').disabled,true);assert.equal(calls.length,requests,'owned readonly publisher inspection remains local');
  find('live-ontology-draft-choice').value='published';find('live-ontology-draft-choice').emit('change');await flush();requests=calls.length;find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.match(heading(),/^Relationship.*lty_self/);assert.equal(find('live-ontology-link').hidden,false);assert.equal(calls.length,requests,'active published no-draft inspection is admitted');
  find('live-ontology-mode-objects').emit('click');const pins=[find('live-ontology-type').value,find('live-ontology-revision').value];requests=calls.length;find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.match(heading(),/^Relationship.*lty_published/);assert.deepEqual([find('live-ontology-type').value,find('live-ontology-revision').value],pins);assert.equal(calls.length,requests,'Objects declaration inspection preserves independent pins and performs no read');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn selected_inspector_context_tracks_applied_source_and_property_choice() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.links.push({id:'lty_self',from_entity_type:'ety_record',to_entity_type:'ety_record',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const heading=()=>['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent).filter(Boolean).join(' · ');
  assert.match(heading(),/name.*ety_record.*Record/);
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  assert.match(heading(),/structure.*ety_record.*Record/,'property() updates the applied selected context');
  const before=heading();find('live-ontology-entity-name').value='Pending rename';find('live-ontology-entity-name').emit('input');
  assert.equal(heading(),before,'pending text is not applied identity');
  const rich=()=>find('live-ontology-property-value');rich().querySelectorAll('button').find(button=>button.textContent==='Add field').emit('click');
  const pendingName=rich().querySelectorAll('input')[1];pendingName.value='pending nested value';pendingName.emit('input');
  const remove=rich().querySelectorAll('button').find(button=>button.textContent==='Remove field');remove.focus();remove.emit('click');assert.equal(document.activeElement===pendingName,true);
  find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');
  assert.equal(heading(),before,'dirty property retarget is refused');
  const objectPin=[find('live-ontology-type').value,find('live-ontology-revision').value];
  find('live-ontology-mode-objects').emit('click');assert.match(heading(),/ety_record.*Record/);assert.ok(!heading().includes('structure'));
  await restore();assert.ok(!heading().includes('structure'),'late model controls cannot overwrite Objects context');
  find('live-ontology-mode-schema').emit('click');assert.equal(heading(),before);assert.equal(find('live-ontology-entity-name').value,'Pending rename');
  assert.equal(rich().querySelectorAll('input')[0].value,'pending nested value','removed-row pending form survives mode and verified recheck');assert.match(find('live-ontology-draft-state').textContent,/Unapplied form edits/);assert.equal(find('live-ontology-draft-save').disabled,true);
  assert.deepEqual([find('live-ontology-type').value,find('live-ontology-revision').value],objectPin);
  find('live-ontology-draft-reload').emit('click');await flush();
  find('live-ontology-library-links').querySelectorAll('[data-model-link]')[0].emit('click');assert.match(heading(),/lty_self/);
  find('live-ontology-library-types').querySelectorAll('[data-model-type]')[0].emit('click');assert.match(heading(),/ety_record.*Record/);
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();
  find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');assert.match(heading(),/name/);
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');assert.match(heading(),/structure/);
  assert.equal(find('live-ontology-property-kind').disabled,true,'readonly property choice grants no editing');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(heading(),'');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn recursive_removal_retains_survivors_and_owned_next_focus() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  const fields=['first','middle','last'].map((name,index)=>({name,required:index!==1,value_type:{shape:'scalar',kind:index===1?'integer':'string'}}));
  for(const prefix of ['property','new-property']){
    if(prefix==='new-property')find('live-ontology-add-property').open=true;
    const host=find('live-ontology-'+prefix+'-value'),control=find('live-ontology-'+prefix+'-kind');
    context.removalControl=control;context.removalHost=host;context.removalValue={shape:'struct',fields};
    const value=vm.runInNewContext("createOntologyValueForm(removalControl,removalHost,()=>{},'Removal test')",context);
    function reset(){value.write(context.removalValue);value.enable(true);}
    const names=()=>host.querySelectorAll('input'),buttons=()=>host.querySelectorAll('button'),remove=index=>buttons().filter(b=>b.textContent==='Remove field')[index],add=()=>buttons().find(b=>b.textContent==='Add field');
    reset();let next=names()[1];remove(0).focus();remove(0).emit('click');assert.equal(document.activeElement===next,true,'first removal focuses next sibling Name');assert.equal(JSON.stringify(value.read().fields.map(f=>f.name)),JSON.stringify(['middle','last']));
    reset();next=names()[2];remove(1).focus();remove(1).emit('click');assert.equal(document.activeElement===next,true,'middle removal focuses next sibling Name');
    reset();remove(2).focus();remove(2).emit('click');assert.equal(document.activeElement===add(),true,'last removal focuses same struct Add');
    reset();const survivor=names()[2];survivor.value='Pending last';survivor.focus();remove(0).emit('click');assert.equal(document.activeElement===survivor,true,'surviving focus owner remains the same DOM node');assert.equal(names()[1],survivor);assert.equal(survivor.value,'Pending last');
    reset();const external=find('sign-in-account');external.focus();remove(0).emit('click');assert.equal(document.activeElement===external,true,'external focus is not stolen');
    reset();value.enable(false);const original=value.capture();remove(0).focus();remove(0).emit('click');assert.deepEqual(value.capture(),original,'disabled removal is inert');
    value.write({shape:'struct',fields:[fields[0]]});value.enable(true);remove(0).focus();remove(0).emit('click');assert.equal(document.activeElement===add(),true);assert.throws(()=>value.read(),/struct needs 1 to 256 named fields/,'empty pending struct remains refused');add().emit('click');names()[0].value='restored';assert.equal(value.read().fields[0].name,'restored');
    reset();const name=names()[1],row=name.parentElement;assert.equal(row.tagName,'label','visible implicit label owns Name');assert.ok(text(row).includes('Name'));assert.ok(host.querySelectorAll('label').some(label=>text(label).includes('Type')));assert.ok(host.querySelectorAll('label').some(label=>text(label).includes('Required')));
  }
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn pane_links_and_saved_diagnostics_reveal_only_current_owners() {
    check_controller(
        r##"
(async()=>{
  for(const [pane,target] of [['library','library-title'],['workpane','workpane'],['inspector','inspector-title']])assert.ok(input.html.includes(`id="live-ontology-pane-${pane}" href="#live-ontology-${target}"`));
  assert.equal(input.html.match(/id="live-ontology-property-edit"/g).length,1);
  assert.ok(input.html.indexOf('id="live-ontology-property-edit"')<input.html.indexOf('id="live-ontology-entity-edit"'),'selected property precedes secondary type settings');
  assert.ok(input.html.indexOf('id="live-ontology-draft-inspector"')<input.html.indexOf('id="live-ontology-schema"'),'primary editor precedes full readonly metadata');
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const native=JSON.stringify([...modelDrafts]),objects=[find('live-ontology-type').value,find('live-ontology-revision').value],requests=calls.length;
  for(const pane of ['library','inspector']){const owner=find('live-ontology-'+pane);owner.open=false;find('live-ontology-pane-'+pane).emit('click');assert.equal(owner.open,true);owner.open=false;context.location.hash='#live-ontology-'+pane+'-title';context.window.emit('hashchange');assert.equal(owner.open,true);}
  find('live-ontology-mode-objects').emit('click');find('live-ontology-pane-workpane').emit('click');assert.equal(find('live-ontology-model').hidden,true);assert.equal(find('live-ontology-objects').hidden,false,'pane navigation does not switch mode');
  find('live-ontology-mode-schema').emit('click');find('live-ontology-view-table').emit('click');find('live-ontology-pane-workpane').emit('click');assert.equal(find('live-ontology-graph').hidden,true);assert.equal(find('live-ontology-model-table').hidden,false,'pane navigation does not switch representation');
  assert.equal(calls.length,requests);assert.equal(JSON.stringify([...modelDrafts]),native);assert.deepEqual([find('live-ontology-type').value,find('live-ontology-revision').value],objects);
  modelValidation={valid:false,diagnostics:[{code:'revision_not_increased',path:'entities.ety_record.revision',message:'Increase declaration revision'}]};find('live-ontology-draft-validate').emit('click');await flush();
  const action=find('live-ontology-draft-diagnostics').querySelectorAll('button')[0],inspector=find('live-ontology-inspector'),settings=find('live-ontology-type-settings');
  inspector.open=false;settings.open=false;find('live-ontology-draft-inspector').hidden=true;action.emit('click');assert.equal(inspector.open,false,'hidden owner cannot be revealed');assert.equal(settings.open,false);find('live-ontology-draft-inspector').hidden=false;
  find('live-ontology-mode-objects').emit('click');action.emit('click');assert.equal(inspector.open,false,'hidden mode cannot be revealed');find('live-ontology-mode-schema').emit('click');
  const before=calls.length;action.focus();action.emit('click');assert.equal(inspector.open,true);assert.equal(settings.open,true);assert.equal(document.activeElement===find('live-ontology-entity-revision'),true);assert.equal(calls.length,before,'inspection does not request or submit');
  inspector.open=false;find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');action.emit('click');find('live-ontology-pane-inspector').emit('click');assert.equal(inspector.open,false,'stale authority cannot reveal panes');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn pane_history_focuses_current_destination_without_overwriting_later_focus() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');
  const pending=find('live-ontology-property-name');pending.value='Pending history edit';pending.emit('input');
  const state=()=>JSON.stringify({drafts:[...modelDrafts],choice:find('live-ontology-property-choice').value,context:['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent),pins:[find('live-ontology-type').value,find('live-ontology-revision').value],version:find('live-ontology-draft-state').textContent,pending:pending.value});
  const before=state(),requests=calls.length,search=find('live-ontology-search');
  function navigate(id){context.location.hash='#live-ontology-'+id;context.window.emit('hashchange');}
  for(const [pane,id] of [['library','library-title'],['inspector','inspector-title'],['workpane','workpane']]){
    const target=find('live-ontology-'+id),owner=find('live-ontology-'+pane);
    search.focus();if(pane!=='workpane')owner.open=false;navigate(id);
    assert.equal(document.activeElement===target,true,'history must focus its current '+pane+' destination rather than keep earlier search focus');
    if(pane!=='workpane')assert.equal(owner.open,true);
    pending.focus();await flush();assert.equal(document.activeElement===pending,true,'later user focus survives synchronous history navigation');
    search.focus();find('live-ontology-pane-'+pane).emit('click');assert.equal(document.activeElement===target,true,'same-fragment pane activation also focuses its destination');
  }
  assert.equal(state(),before,'history retains pending edits, applied selection, version and object read pins');assert.equal(calls.length,requests,'pane navigation makes no requests');
  find('live-ontology-mode-objects').emit('click');navigate('workpane');assert.equal(find('live-ontology-model').hidden,true);assert.equal(find('live-ontology-objects').hidden,false);
  find('live-ontology-mode-schema').emit('click');find('live-ontology-view-table').emit('click');navigate('workpane');assert.equal(find('live-ontology-graph').hidden,true);assert.equal(find('live-ontology-model-table').hidden,false);
  search.focus();await restore();assert.equal(document.activeElement===search,true,'verified same-session refresh may reveal its hash but does not steal later user focus');assert.equal(pending.value,'Pending history edit');
  const target=find('live-ontology-inspector-title'),owner=find('live-ontology-inspector'),external=find('sign-in-account');
  function refused(message){owner.open=false;external.focus();navigate('inspector-title');assert.equal(owner.open,false,message);assert.equal(document.activeElement===external,true,message);}
  root.hidden=true;refused('hidden root refuses navigation');root.hidden=false;
  root.isConnected=false;refused('disconnected root refuses navigation');root.isConnected=true;
  owner.hidden=true;refused('hidden pane refuses navigation');owner.hidden=false;
  target.disabled=true;refused('disabled destination refuses navigation');target.disabled=false;
  const parent=target.parentElement;target.parentElement=panel;refused('foreign destination refuses navigation');target.parentElement=parent;
  target.isConnected=false;refused('disconnected destination refuses navigation');target.isConnected=true;
  const rects=target.getClientRects;target.getClientRects=()=>[];external.focus();navigate('inspector-title');assert.equal(document.activeElement===external,true,'nonphysical destination receives no focus');target.getClientRects=rects;
  external.focus();navigate('property-name');assert.equal(document.activeElement===external,true,'non-pane hash cannot steal focus');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');refused('stale account refuses navigation');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn pane_height_tracks_only_current_visible_native_geometry() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const nav=find('live-ontology-pane-library').parentElement,height=()=>root.style.getPropertyValue('--foundry-pane-bar-height');
  assert.equal(height(),'58px','verified open measures actual pane bar instead of assuming one row');
  const observer=resizeObservers.find(item=>item.target===nav);assert.ok(observer?.observing,'current pane bar is observed');
  find('live-ontology-property-choice').value='1';find('live-ontology-property-choice').emit('change');const field=find('live-ontology-property-name');field.value='Pending measured height';field.emit('input');field.focus();
  const state=()=>JSON.stringify({value:find('live-ontology-property-name').value,choice:find('live-ontology-property-choice').value,context:['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent),pins:[find('live-ontology-type').value,find('live-ontology-revision').value],draft:find('live-ontology-draft-state').textContent});const before=state(),requests=calls.length;
  nav.mockHeight=183;observer.emit();assert.equal(height(),'183px','native wrapping height reaches existing sticky/scroll-margin variable');assert.equal(document.activeElement===field,true,'resize never steals newer focus');assert.equal(state(),before);assert.equal(calls.length,requests);
  for(const invalid of [0,-1,NaN,Infinity]){nav.mockHeight=invalid;observer.emit();assert.equal(height(),'183px','zero or invalid geometry cannot replace a valid height');}nav.mockHeight=200;
  root.hidden=true;observer.emit();assert.equal(height(),'183px');root.hidden=false;root.isConnected=false;observer.emit();assert.equal(height(),'183px');root.isConnected=true;
  nav.hidden=true;observer.emit();assert.equal(height(),'183px');nav.hidden=false;nav.isConnected=false;observer.emit();assert.equal(height(),'183px');nav.isConnected=true;
  const parent=nav.parentElement;nav.parentElement=panel;observer.emit();assert.equal(height(),'183px','foreign pane owner refused');nav.parentElement=parent;
  find('live-ontology-pane-inspector').emit('click');assert.equal(height(),'200px','navigation synchronously measures before destination focus');assert.equal(document.activeElement===find('live-ontology-inspector-title'),true);assert.equal(state(),before);
  field.focus();nav.mockHeight=121.5;find('session-restore').emit('click');assert.equal(observer.observing,false,'session hold disconnects measurement');observer.emit();assert.equal(height(),'200px','invalid-generation callback cannot change current bar');await flush();assert.equal(observer.observing,true);assert.equal(height(),'121.5px','verified reopen reobserves and measures current owner');assert.equal(find('live-ontology-property-name').value,'Pending measured height');
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(height(),'');assert.equal(observer.observing,false,'clear releases observer and CSS override');nav.mockHeight=250;observer.emit();assert.equal(height(),'','queued stale callback cannot restore cleared geometry');
  find('sign-in-account').value='usr_alice';await restore();assert.equal(observer.observing,true);assert.equal(height(),'250px','later verified open resumes native measurement');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn recursive_nested_removal_preserves_pending_capture_and_new_focus_owner() {
    check_controller(
        r##"
(async()=>{
  await flush();const host=new Element(),control=new Element('select');panel.append(host,control);for(const kind of ['struct','array','string']){const option=new Element('option');option.value=kind;control.append(option);}
  context.nestedHost=host;context.nestedControl=control;context.changedOwner=null;
  const form=vm.runInNewContext("createOntologyValueForm(nestedControl,nestedHost,()=>{if(changedOwner)changedOwner.focus();},'Nested')",context);
  const field=name=>({name,required:false,value_type:{shape:'scalar',kind:'string'}}),inner={shape:'struct',fields:[field('first'),field('next')]};
  form.write({shape:'array',element:inner});form.enable(true);
  let names=host.querySelectorAll('input'),remove=host.querySelectorAll('button').find(b=>b.textContent==='Remove field');names[1].value='surviving pending';remove.focus();remove.emit('click');assert.equal(document.activeElement===names[1],true,'array-contained struct keeps local next Name focus');
  const snapshot=form.capture();form.clear();form.restore(snapshot);form.enable(true);assert.equal(host.querySelectorAll('input')[0].value,'surviving pending');assert.equal(document.activeElement===host.querySelectorAll('input')[0],true,'capture/restore retains surviving focused field');
  context.changedOwner=find('sign-in-account');remove=host.querySelectorAll('button').find(b=>b.textContent==='Remove field');remove.focus();remove.emit('click');assert.equal(document.activeElement===context.changedOwner,true,'a synchronous newer callback focus owner is retained');
  context.changedOwner=null;let deep={shape:'struct',fields:[field('deep')]};for(let i=0;i<6;i++)deep={shape:'array',element:deep};form.write(deep);form.enable(true);remove=host.querySelectorAll('button').find(b=>b.textContent==='Remove field');remove.focus();remove.emit('click');const add=host.querySelectorAll('button').find(b=>b.textContent==='Add field');assert.equal(document.activeElement===add,true);assert.equal(add.disabled,false,'deepest admitted struct still has useful Add fallback');
  form.write({shape:'struct',fields:Array.from({length:256},(_,i)=>field('f'+i))});form.enable(true);remove=host.querySelectorAll('button').filter(b=>b.textContent==='Remove field').at(-1);remove.focus();remove.emit('click');assert.equal(form.read().fields.length,255);assert.equal(document.activeElement.textContent,'Add field');assert.equal(document.activeElement.disabled,false);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}
