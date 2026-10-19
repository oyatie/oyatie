use super::explorer_test::check_controller;

#[test]
fn property_overview_refreshes_applied_metadata_while_other_fields_remain_pending() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const pending=find('live-ontology-entity-name');pending.value='Unapplied type name';pending.emit('input');
  find('live-ontology-add-property').open=true;
  for(const [id,value] of [['new-property-name','new_date'],['new-property-kind','date'],['entity-revision','2']]){find('live-ontology-'+id).value=value;find('live-ontology-'+id).emit('input');}
  find('live-ontology-property-add').emit('submit');
  const rows=find('live-ontology-property-rows').querySelectorAll('tbody')[0].children,buttons=find('live-ontology-property-rows').querySelectorAll('button');
  assert.equal(find('live-ontology-property-count').textContent,'3 of 3 properties');assert.equal(buttons[2].textContent,'new_date');assert.equal(text(rows[2].children[1]),'date');assert.equal(find('live-ontology-property-choice').value,'0','partial apply retains original selected property');assert.equal(pending.value,'Unapplied type name');
  find('live-ontology-type-settings').open=true;pending.focus();buttons[2].emit('click');assert.equal(document.activeElement,pending);assert.equal(find('live-ontology-property-choice').value,'0');assert.equal(buttons[0].getAttribute('aria-pressed'),'true');assert.equal(find('live-ontology-draft-save').disabled,true,'remaining unapplied fields still refuse save');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn property_overview_filters_native_metadata_without_switching_the_editor() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.entities[0].primary_key_property='name';
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.equal(find('live-ontology-draft-inspector').hidden,true,'published model keeps its separate readonly metadata surface');
  find('live-ontology-draft-new').emit('click');await flush();
  const host=find('live-ontology-property-rows'),rows=host.querySelectorAll('tbody')[0].children,buttons=host.querySelectorAll('button'),search=find('live-ontology-property-search'),count=find('live-ontology-property-count');
  assert.equal(buttons.length,2);assert.equal(count.textContent,'2 of 2 properties');assert.deepEqual(host.querySelectorAll('th').map(node=>node.textContent),['Name','Type','Tags']);
  assert.equal(text(rows[0].children[1]),'Legacy string contract');assert.equal(text(rows[0].children[2]),'Primary Title Required');assert.equal(text(rows[1].children[2]),'');
  assert.equal(buttons[1].textContent,'Untouched');assert.match(buttons[1].getAttribute('title'),/structure/,'exact API name remains available');assert.equal(text(rows[1].children[1]),'Struct · 1 field');assert.ok(!text(rows[1].children[1]).includes('date'),'compact overview does not expand nested declarations');
  assert.equal(buttons[1].tagName,'button');assert.equal(buttons[1].type,'button');assert.equal(buttons[1].getAttribute('aria-label'),'Inspect property structure');assert.equal(buttons[1].getAttribute('aria-controls'),'live-ontology-property-edit');
  const native=JSON.stringify([...modelDrafts]),requests=calls.length;
  buttons[1].focus();buttons[1].emit('click');assert.equal(document.activeElement,buttons[1],'native button selection keeps its keyboard focus owner');assert.equal(find('live-ontology-property-choice').value,'1');assert.equal(find('live-ontology-property-name').value,'structure');assert.equal(buttons[1].getAttribute('aria-pressed'),'true');assert.equal(rows[1].getAttribute('data-selected'),'true');assert.equal(buttons[0].getAttribute('aria-pressed'),'false');
  search.focus();search.value='PRIMARY';search.emit('input');assert.equal(rows[0].hidden,false);assert.equal(rows[1].hidden,true);assert.equal(count.textContent,'1 of 2 properties · Selected property is filtered out.');assert.equal(find('live-ontology-property-choice').value,'1');assert.equal(find('live-ontology-property-name').value,'structure');assert.equal(document.activeElement,search);
  search.value='no matching property';search.emit('input');assert.equal(find('live-ontology-property-empty').hidden,false);assert.equal(count.textContent,'0 of 2 properties · Selected property is filtered out.');
  search.value='';search.emit('input');assert.equal(count.textContent,'2 of 2 properties');assert.equal(rows[1].hidden,false);assert.equal(buttons[1].getAttribute('aria-pressed'),'true');assert.equal(find('live-ontology-property-name').value,'structure');
  find('live-ontology-property-choice').value='0';find('live-ontology-property-choice').emit('change');assert.equal(buttons[0].getAttribute('aria-pressed'),'true','legacy selector and table share one applied selection');assert.equal(find('live-ontology-property-name').value,'name');
  assert.equal(calls.length,requests);assert.equal(JSON.stringify([...modelDrafts]),native,'overview inspection/search never saves or mutates the native draft');
  assert.ok(input.html.includes('type="search"'));assert.ok(input.html.includes('for="live-ontology-property-search"'));assert.ok(input.html.indexOf('id="live-ontology-property-overview"')<input.html.indexOf('id="live-ontology-property-edit"'));
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn property_overview_refuses_pending_switches_and_keeps_exact_forms_and_focus() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.named_string_types=[{id:'vty_status',revision:1,display_name:'Employment status',members:['active','onLeave']}];
  modelDocument.entities[0].properties.push({name:'status',tier:'scalar',data_class:'InternalOnly',required:false,value_type:{shape:'named_string',type_id:'vty_status',revision:1},display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const buttons=()=>find('live-ontology-property-rows').querySelectorAll('button'),choice=find('live-ontology-property-choice'),search=find('live-ontology-property-search');
  buttons()[2].emit('click');const picker=find('live-ontology-property-value').querySelectorAll('select')[0],exact=picker.value;
  const name=find('live-ontology-property-name');name.value='Pending status name';name.emit('input');name.focus();
  const state=()=>JSON.stringify({name:name.value,choice:choice.value,picker:picker.value,selected:['selected','identity','name'].map(id=>find('live-ontology-context-'+id).textContent)}),before=state(),requests=calls.length;
  buttons()[0].emit('click');assert.equal(state(),before);assert.equal(document.activeElement,name,'refused activation does not steal a surviving form focus');assert.equal(find('live-ontology-property-value').querySelectorAll('select')[0],picker);assert.match(find('live-ontology-draft-status').textContent,/Apply form edits/);assert.equal(buttons()[2].getAttribute('aria-pressed'),'true');
  choice.value='1';choice.emit('change');assert.equal(state(),before,'alternate selector uses the same pending refusal');
  search.value='no-match';search.emit('input');assert.equal(state(),before);search.value='status';search.emit('input');assert.equal(state(),before);assert.equal(picker.value,exact);
  find('live-ontology-mode-objects').emit('click');find('live-ontology-mode-schema').emit('click');assert.equal(state(),before);assert.equal(search.value,'status');
  await restore();assert.equal(state(),before);assert.equal(search.value,'status','verified recheck preserves purely local search and pending exact reference');
  assert.equal(calls.filter(([url,options])=>url.includes('/model')&&options.method==='POST').length,1,'only explicit create reached native state');
  find('live-ontology-property-edit').emit('submit');assert.equal(find('live-ontology-property-name').value,'Pending status name');assert.equal(choice.value,'2');assert.equal(buttons()[2].getAttribute('aria-pressed'),'true');
  find('live-ontology-draft-save').emit('click');await flush();const saved=[...modelDrafts.values()][0].document.entities[0].properties[2];assert.equal(saved.name,'Pending status name');assert.deepEqual(saved.value_type,{shape:'named_string',type_id:'vty_status',revision:1});
  assert.ok(calls.length>=requests);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn property_overview_inspects_readonly_drafts_and_refuses_stale_or_invalid_choices() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const original=find('live-ontology-property-rows').querySelectorAll('button')[1];
  modelWorkspace.capabilities.author=false;modelWorkspace.capabilities.publish=true;session.roles=['foundry-model-publisher','foundry-operator'];await restore();
  const buttons=find('live-ontology-property-rows').querySelectorAll('button'),choice=find('live-ontology-property-choice'),search=find('live-ontology-property-search');
  assert.equal(buttons[1].disabled,false);assert.equal(search.disabled,false);const requests=calls.length,native=JSON.stringify([...modelDrafts]);
  buttons[1].emit('click');assert.equal(choice.value,'1');assert.equal(find('live-ontology-property-name').value,'structure');assert.equal(find('live-ontology-property-name').disabled,true);assert.equal(find('live-ontology-property-edit').querySelectorAll('button').every(button=>button.disabled),true);
  for(const invalid of ['','-1','2','NaN','0.5']){const option=new Element('option');option.value=invalid;choice.append(option);choice.value=invalid;choice.emit('change');assert.equal(choice.value,'1');assert.equal(buttons[1].getAttribute('aria-pressed'),'true');choice.removeChild(option);}
  original.emit('click');assert.equal(choice.value,'1','disconnected stale overview action cannot retarget');
  buttons[0].focus();buttons[0].emit('click');assert.equal(choice.value,'0');assert.equal(document.activeElement,buttons[0]);
  find('live-ontology-mode-objects').emit('click');buttons[1].emit('click');find('live-ontology-mode-schema').emit('click');assert.equal(choice.value,'0','hidden schema interaction refuses before changing state');
  assert.equal(calls.length,requests);assert.equal(JSON.stringify([...modelDrafts]),native);
  const stale=find('live-ontology-property-rows').querySelectorAll('button')[1];find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');stale.emit('click');assert.equal(find('live-ontology-property-rows').children.length,0);assert.equal(find('live-ontology-property-count').textContent,'');assert.equal(search.value,'');assert.equal(search.disabled,true);
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn property_overview_resolves_only_exact_unambiguous_named_types_and_keeps_legacy_tiers() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.named_string_types=[{id:'vty_status',revision:1,display_name:'Exact status',members:['active']},{id:'vty_status',revision:2,display_name:'Other revision',members:['other']},{id:'vty_ambiguous',revision:1,display_name:'Ambiguous A',members:['a']},{id:'vty_ambiguous',revision:1,display_name:'Ambiguous B',members:['b']}];
  const property=(name,type_id,revision)=>({name,tier:'scalar',data_class:'InternalOnly',required:false,value_type:{shape:'named_string',type_id,revision},display:null});
  modelDocument.entities[0].properties.push(property('status','vty_status',1),property('absent','vty_status',3),property('ambiguous','vty_ambiguous',1),{name:'reserved',tier:'geo',data_class:'InternalOnly',required:false,value_type:null,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const rows=find('live-ontology-property-rows').querySelectorAll('tbody').flatMap(body=>body.children),types=rows.map(row=>text(row.children[1]));
  assert.equal(types[2],'Exact status');assert.equal(types[3],'Unresolved reusable string');assert.equal(types[4],'Unresolved reusable string');assert.match(rows[2].children[1].children[0].getAttribute('aria-label'),/vty_status.*revision 1/);assert.match(rows[3].children[1].children[0].getAttribute('aria-label'),/vty_status.*revision 3/);assert.match(rows[4].children[1].children[0].getAttribute('aria-label'),/vty_ambiguous.*revision 1/);assert.ok(!types[4].includes('Ambiguous A')&&!types[4].includes('Ambiguous B'));
  const buttons=find('live-ontology-property-rows').querySelectorAll('button');buttons[3].emit('click');const reference=find('live-ontology-property-value').querySelectorAll('select')[0].value;
  find('live-ontology-property-search').value='Exact status';find('live-ontology-property-search').emit('input');assert.equal(rows[2].hidden,false);assert.equal(rows[3].hidden,true);assert.equal(find('live-ontology-property-choice').value,'3');assert.equal(find('live-ontology-property-value').querySelectorAll('select')[0].value,reference,'search does not resolve a missing revision to the visible definition');
  find('live-ontology-property-search').value='';find('live-ontology-property-search').emit('input');find('live-ontology-property-more').open=true;buttons[5].emit('click');assert.equal(find('live-ontology-property-name').value,'reserved');assert.equal(find('live-ontology-property-name').disabled,true);assert.match(find('live-ontology-property-compatibility').textContent,/geo.*read-only/);assert.equal(types[5],'geo · no value declaration','reserved null declaration keeps its exact tier without inferring String');
})().catch(error=>{console.error(error);process.exitCode=1;});
"##,
    );
}

#[test]
fn property_overview_and_library_preserve_pending_defaults_through_verified_refresh() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const host=find('live-ontology-property-default');host.querySelectorAll('button').find(button=>button.textContent==='Add default').emit('click');
  const field=()=>document.getElementById('live-ontology-property-default-value'),literal='<script>literal</script>\n한글 pending';field().value=literal;field().emit('input');field().focus();
  const native=JSON.stringify([...modelDrafts]),posts=()=>calls.filter(([url,options])=>url.includes('/model')&&options.method==='POST').length;
  const rows=()=>find('live-ontology-property-rows').querySelectorAll('button');rows()[1].emit('click');assert.equal(field().value,literal);assert.equal(document.activeElement,field());assert.equal(find('live-ontology-property-choice').value,'0');
  const search=find('live-ontology-property-search');search.value='no-match';search.emit('input');find('live-ontology-search').value='Record';find('live-ontology-search').emit('input');assert.equal(field().value,literal);assert.equal(JSON.stringify([...modelDrafts]),native);assert.equal(posts(),1);
  await restore();assert.equal(field().value,literal,'verified same-identity refresh preserves exact textarea text');assert.equal(field().type,'textarea');assert.equal(find('live-ontology-property-search').value,'no-match');assert.equal(find('live-ontology-property-choice').value,'0');assert.equal(posts(),1);assert.equal(find('live-ontology-draft-save').disabled,true);
  find('live-ontology-entity-revision').value='2';find('live-ontology-entity-revision').emit('input');find('live-ontology-property-edit').emit('submit');find('live-ontology-draft-save').emit('click');await flush();assert.deepEqual([...modelDrafts.values()][0].document.entities[0].properties[0].default_value,{kind:'string',value:literal});
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(host.children.length,0);assert.equal(find('live-ontology-property-rows').children.length,0);assert.equal(find('live-ontology-library-values').children.length,0);
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn property_overview_keeps_four_primary_rows_and_reaches_selected_search_matches_beyond_them() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author'];modelDocument.named_string_types=[{id:'vty_employment_status',revision:1,display_name:'EmploymentStatus',members:['active','ended']}];
  modelDocument.entities[0].title_property='field_0';modelDocument.entities[0].properties=Array.from({length:28},(_,index)=>({name:'field_'+index,tier:index===1?'struct':'scalar',data_class:'InternalOnly',required:false,value_type:index===0?{shape:'named_string',type_id:'vty_employment_status',revision:1}:index===1?{shape:'struct',fields:Array.from({length:24},(_,field)=>({name:'nested_'+field,required:true,value_type:{shape:'array',element:{shape:'scalar',kind:'string'}}}))}:{shape:'scalar',kind:'string'},display:null}));
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();find('live-ontology-draft-new').emit('click');await flush();
  const host=find('live-ontology-property-rows'),bodies=host.querySelectorAll('tbody'),more=find('live-ontology-property-more');assert.equal(bodies[0].children.length,4);assert.equal(bodies[1].children.length,24);assert.equal(more.open,false);assert.equal(text(bodies[0].children[0].children[1]),'EmploymentStatus');assert.equal(text(bodies[0].children[1].children[1]),'Struct · 24 fields');assert.ok(!text(bodies[0]).includes('nested_'),'overview never expands the recursive declaration');assert.match(bodies[0].children[0].children[1].children[0].getAttribute('aria-label'),/vty_employment_status.*revision 1/);
  const requests=calls.length,choice=find('live-ontology-property-choice');choice.value='27';choice.emit('change');assert.equal(find('live-ontology-property-name').value,'field_27');assert.match(find('live-ontology-property-count').textContent,/Selected property is under More properties/);assert.match(more.querySelector('summary').textContent,/24.*Selected property/);assert.equal(bodies[1].children[23].querySelector('button').getAttribute('aria-pressed'),'true');
  const search=find('live-ontology-property-search');search.value='field_27';search.emit('input');assert.equal(more.open,true,'an outside-preview search match is reachable without a separate reveal');assert.equal(bodies[0].parentElement.hidden,true);assert.equal(bodies[1].children.filter(row=>!row.hidden).length,1);assert.equal(find('live-ontology-property-name').value,'field_27');assert.equal(calls.length,requests,'preview,selection and search send no native writes');more.open=false;more.emit('toggle');assert.equal(more.open,false,'a current search does not trap the native disclosure open');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
