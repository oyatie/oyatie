use super::super::explorer_test::check_controller;

#[test]
fn library_disclosures_restore_owned_focus_or_local_fallback_without_external_theft() {
    check_controller(
        r##"
(async()=>{
  await flush();root.hidden=false;const rows=[...schemaFixture.map((row,index)=>({...structuredClone(row),display_name:'Type '+index})),{...structuredClone(schemaFixture[0]),entity_type:'',display_name:'Missing identity'}],owner={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'};
  const view=context.createOntologyModelView(()=>true,()=>find('live-ontology-search').focus()),render=next=>view.setModel(next,[],{type:'ety_record',link:null},'Draft',owner,[]);render(rows);
  for(const id of ['library-outside','library-invalid']){let summary=find('live-ontology-'+id).querySelector('summary');summary.focus();render(rows);assert.equal(document.activeElement,find('live-ontology-'+id).querySelector('summary'),'same-context update restores '+id+' summary focus');const external=find('shell-session-entry');external.focus();render(rows);assert.equal(document.activeElement,external,'newer external focus is retained');}
  find('live-ontology-library-invalid').querySelector('summary').focus();render(rows.slice(0,2));assert.equal(document.activeElement,find('live-ontology-search'),'removed disclosure falls back to local search');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn library_shortlist_uses_scope_eligibility_and_keeps_all_remaining_native_rows() {
    check_controller(
        r##"
(async()=>{
  await flush();root.hidden=false;
  const rows=Array.from({length:52},(_,index)=>({...structuredClone(schemaFixture[0]),entity_type:'ety_'+index,display_name:'Type '+index}));
  const link=(id,from,to)=>({link_type:id,from_entity_type:from,to_entity_type:to,cardinality:'one_to_many',revision:1,allow_cross_tenant:false});
  const links=[...Array.from({length:7},(_,index)=>link('lty_neighbor_'+index,'ety_0','ety_'+(index+1))),link('lty_self','ety_0','ety_0'),link('lty_parallel','ety_0','ety_1'),link('lty_induced_a','ety_1','ety_2'),link('lty_induced_b','ety_2','ety_3'),...Array.from({length:137},(_,index)=>link('lty_outside_'+index,'ety_8','ety_'+(9+index%43)))];
  const owner={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'7',base:'2'};
  let view;view=context.createOntologyModelView(id=>{view.setModel(rows,links,{type:id,link:null},'Draft',owner,[]);return true;},()=>find('live-ontology-search').focus());
  find('live-ontology-graph').clientWidth=320;find('live-ontology-graph').clientHeight=280;view.setModel(rows,links,{type:'ety_0',link:null},'Draft',owner,[]);
  const library=find('live-ontology-library-types'),direct=()=>library.children.filter(row=>row.dataset.modelType),scene=find('live-ontology-scene'),graph=find('live-ontology-graph');
  assert.equal(direct().length,8,'shortlist uses all eligible Related types, even when viewport cannot draw all eight');assert.ok(scene.querySelectorAll('[data-model-type]').length<8);assert.match(find('live-ontology-library-scope').textContent,/8 of 52 types.*44 outside/);assert.match(find('live-ontology-library-type-heading').textContent,/52/);assert.equal(find('live-ontology-library-outside').querySelectorAll('[data-model-type]').length,44);assert.equal(library.querySelectorAll('[data-model-type]').length,52);assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-type]').length,52);
  graph.scrollLeft=73;graph.scrollTop=41;const state=()=>JSON.stringify({scope:view.scope(),nodes:scene.querySelectorAll('[data-model-type]').map(row=>row.dataset.modelType),transform:scene.style.transform,scroll:[graph.scrollLeft,graph.scrollTop]}),before=state(),requests=calls.length;
  find('live-ontology-library-outside').open=true;assert.equal(state(),before,'revealing outside types does not expand or change graph scope/camera');
  const outside=library.querySelectorAll('[data-model-type]').find(row=>row.dataset.modelType==='ety_37');outside.focus();outside.emit('click');assert.equal(direct().some(row=>row.dataset.modelType==='ety_37'),true);assert.equal(document.activeElement,direct().find(row=>row.dataset.modelType==='ety_37'),'accepted outside selection restores the same semantic resource in its new eligible location');
  view.setScope('selected');assert.equal(direct().length,1);view.setScope('all');assert.equal(direct().length,52);assert.equal(find('live-ontology-library-outside'),null);assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-type]').length,52);assert.equal(calls.length,requests);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn library_global_search_reaches_outside_and_invalid_matches_without_retargeting() {
    check_controller(
        r##"
(async()=>{
  await flush();root.hidden=false;
  const rows=Array.from({length:9},(_,index)=>({...structuredClone(schemaFixture[0]),entity_type:'ety_'+index,display_name:index===7||index===8?'Shared label':'Type '+index}));rows[7].properties[0].name='employmentStatus';rows[7].title_property='employmentStatus';
  rows.push({...structuredClone(rows[0]),entity_type:'ety_duplicate',display_name:'Duplicate resource'}, {...structuredClone(rows[0]),entity_type:'ety_duplicate',display_name:'Duplicate resource'}, {...structuredClone(rows[0]),entity_type:'',display_name:'Missing identity'});
  const relation={link_type:'lty_named',from_entity_type:'ety_0',to_entity_type:'ety_1',cardinality:'one_to_many',revision:1,allow_cross_tenant:false,display:{display_name:'Manages',description:null,icon:null,color:null,plural_name:null}};
  const view=context.createOntologyModelView(()=>true,()=>find('live-ontology-search').focus());view.setModel(rows,[relation],{type:'ety_0',link:null},'Draft',{tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'},[]);
  const library=find('live-ontology-library-types'),search=find('live-ontology-search'),graph=find('live-ontology-graph');assert.equal(find('live-ontology-library-invalid').querySelectorAll('[data-model-type]').length,3);assert.equal(find('live-ontology-library-invalid').querySelectorAll('button').every(button=>button.disabled),true);assert.match(find('live-ontology-library-scope').textContent,/2 of 12 types.*7 outside.*3 invalid/);
  const state=()=>JSON.stringify({scope:view.scope(),heading:find('live-ontology-context-name').textContent,transform:find('live-ontology-scene').style.transform,scroll:[graph.scrollLeft,graph.scrollTop]}),before=state(),requests=calls.length;
  search.focus();search.value='employmentstatus';search.emit('input');let buttons=library.querySelectorAll('[data-model-type]');assert.equal(buttons.length,1);assert.equal(buttons[0].dataset.modelType,'ety_7');assert.equal(buttons[0].dataset.libraryScope,'outside');assert.equal(find('live-ontology-library-outside'),null,'global matches do not require opening the outside list');assert.equal(state(),before);assert.equal(document.activeElement,search);
  search.value='Duplicate resource';search.emit('input');buttons=library.querySelectorAll('[data-model-type]');assert.equal(buttons.length,2);assert.equal(buttons.every(button=>button.disabled),true);assert.match(find('live-ontology-library-summary').textContent,/2 of 12 types/);
  search.value='manages';search.emit('input');assert.equal(find('live-ontology-library-links').querySelectorAll('[data-model-link]').length,1);assert.equal(find('live-ontology-library-link-group').open,true);assert.match(find('live-ontology-library-link-heading').textContent,/1 matches/);
  search.value='';search.emit('input');buttons=library.querySelectorAll('[data-model-type]');for(const id of ['ety_7','ety_8']){const button=buttons.find(row=>row.dataset.modelType===id);assert.match(button.getAttribute('aria-label'),new RegExp(id));assert.equal(button.querySelectorAll('small')[0].className,'','equal display names expose exact IDs for disambiguation');}
  assert.equal(state(),before);assert.equal(calls.length,requests);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn library_named_rows_keep_readonly_exact_revisions_and_unavailable_catalog_distinct() {
    check_controller(
        r##"
(async()=>{
  await flush();root.hidden=false;
  const view=context.createOntologyModelView(()=>true,()=>{}),owner={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'},catalog=[{id:'vty_a',revision:1,display_name:'Status',members:['A']},{id:'vty_a',revision:1,display_name:'Status',members:['B']},{id:'vty_b',revision:2,display_name:'Status',members:['C']}];
  view.setModel([{...schemaFixture[0],display_name:'Record'}],[],{type:'ety_record',link:null},'Draft',owner,catalog);
  assert.equal(find('live-ontology-library-value-heading').textContent,'Reusable string types (3)');find('live-ontology-library-value-group').open=true;
  const rows=find('live-ontology-library-values').querySelectorAll('[data-model-value]'),before=find('live-ontology-context-identity').textContent,requests=calls.length;assert.equal(rows.length,3);assert.equal(find('live-ontology-library-values').querySelectorAll('button').length,0,'readonly catalog exposes no unsupported selection action');
  for(let index=0;index<3;index++){assert.equal(rows[index].tagName,'details');assert.match(rows[index].querySelector('summary').textContent,new RegExp(catalog[index].id));rows[index].open=true;assert.equal(rows[index].querySelector('li').textContent,catalog[index].members[0]);}
  rows[2].querySelector('summary').focus();view.setModel([{...schemaFixture[0],display_name:'Record'}],[],{type:'ety_record',link:null},'Draft',owner,catalog);assert.equal(document.activeElement,find('live-ontology-library-values').querySelectorAll('[data-model-value]')[2].querySelector('summary'),'same-context refresh restores the exact readonly catalog disclosure focus');
  assert.match(rows[0].querySelector('summary').textContent,/duplicate row 1/);assert.match(rows[1].querySelector('summary').textContent,/duplicate row 2/);assert.equal(find('live-ontology-context-identity').textContent,before);assert.equal(calls.length,requests);
  const search=find('live-ontology-search');search.value='vty_b';search.emit('input');assert.equal(find('live-ontology-library-values').querySelectorAll('[data-model-value]')[0].dataset.modelValue,'2');
  view.setModel([{...schemaFixture[0],display_name:'Record'}],[],{type:'ety_record',link:null},'Published declarations',{tenantId:'ten_acme',userId:'usr_alice',kind:'catalog'});assert.match(find('live-ontology-library-value-heading').textContent,/unavailable.*unavailable matches/);assert.equal(find('live-ontology-library-values').children.length,0);assert.match(find('live-ontology-values-note').textContent,/requires native model access/);assert.equal(find('live-ontology-library-value-group').open,false);
  search.value='';search.emit('input');view.setModel([{...schemaFixture[0],display_name:'Record'}],[],{type:'ety_record',link:null},'Published model',{tenantId:'ten_acme',userId:'usr_alice',kind:'published',generation:'2'},[]);assert.equal(find('live-ontology-library-value-heading').textContent,'Reusable string types (0)','authorized empty catalog differs from unavailable');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn library_new_reveals_existing_forms_without_selection_changes_or_writes() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  const menu=find('live-ontology-library-new'),status=find('live-ontology-library-new-status');assert.equal(find('live-ontology-library-new-type').disabled,true);assert.match(status.textContent,/Create or open an owned draft/);
  find('live-ontology-draft-new').emit('click');await flush();
  const state=()=>JSON.stringify({native:[...modelDrafts],selection:find('live-ontology-draft-selection').textContent,scope:find('live-ontology-library-scope').textContent,camera:find('live-ontology-scene').style.transform,scroll:[find('live-ontology-graph').scrollLeft,find('live-ontology-graph').scrollTop]}),before=state(),requests=calls.length;
  for(const [kind,id] of [['type','new-entity-id'],['link','new-link-id']]){menu.open=true;const button=find('live-ontology-library-new-'+kind),target=find('live-ontology-'+id);find('live-ontology-inspector').open=false;button.focus();button.emit('click');assert.equal(find('live-ontology-inspector').open,true);assert.equal(document.activeElement,target);assert.equal(target.getClientRects().length,1);assert.equal(menu.open,false);assert.equal(state(),before);}
  menu.open=true;const external=find('shell-session-entry');external.focus();find('live-ontology-library-new-type').emit('click');assert.equal(document.activeElement,external,'programmatic reveal does not steal newer external focus');assert.equal(calls.length,requests);assert.equal(state(),before);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn library_new_refuses_pending_loading_readonly_objects_and_uncertain_save_states() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const menu=find('live-ontology-library-new'),button=find('live-ontology-library-new-type'),status=find('live-ontology-library-new-status');
  const pending=find('live-ontology-property-name');pending.value='Unapplied name';pending.emit('input');pending.focus();menu.open=true;const callsBefore=calls.length;button.emit('click');assert.equal(button.disabled,true);assert.match(status.textContent,/Apply form edits/);assert.equal(document.activeElement,pending);assert.equal(pending.value,'Unapplied name');assert.equal(calls.length,callsBefore);
  const beforeSelection=find('live-ontology-draft-selection').textContent;const other=find('live-ontology-library-types').querySelectorAll('[data-model-type]')[0];other.emit('click');assert.equal(find('live-ontology-draft-selection').textContent,beforeSelection);assert.equal(pending.value,'Unapplied name');
  find('live-ontology-draft-reload').emit('click');await flush();find('live-ontology-mode-objects').emit('click');assert.equal(button.disabled,true);assert.match(status.textContent,/Choose Model/);find('live-ontology-mode-schema').emit('click');
  holdPath='/model';find('live-ontology-draft-reload').emit('click');await flush();assert.equal(button.disabled,true);assert.match(status.textContent,/Wait for/);held();await flush();
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();assert.equal(button.disabled,true);assert.match(status.textContent,/author access/);const requests=calls.length;button.emit('click');assert.equal(calls.length,requests);
  modelWorkspace.capabilities.author=true;modelWorkspace.capabilities.publish=false;session.roles=['foundry-model-author','foundry-operator'];await restore();find('live-ontology-property-name').value='Applied name';find('live-ontology-property-name').emit('input');find('live-ontology-property-edit').emit('submit');modelLoseAck=true;find('live-ontology-draft-save').emit('click');await flush();assert.equal(button.disabled,true);assert.match(status.textContent,/Reconcile saved state/);const afterSave=calls.length;menu.open=true;button.emit('click');assert.equal(calls.length,afterSave,'uncertain save is not retried by New');
  find('live-ontology-draft-reload').emit('click');await flush();const id=find('live-ontology-draft-choice').value,draft=modelDrafts.get(id);draft.publication={generation:String(BigInt(draft.base_generation)+1n),prior_generation:draft.base_generation,draft_id:id,draft_version:draft.version,owner:draft.owner};find('live-ontology-draft-reload').emit('click');await flush();assert.equal(button.disabled,true);assert.match(status.textContent,/published and closed/);
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');button.emit('click');assert.equal(root.hidden,true);assert.equal(button.disabled,true);assert.equal(menu.open,false);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}
