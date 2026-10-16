use super::explorer_test::check_controller;

#[test]
fn declaration_scope_keeps_exact_membership_and_invalid_draft_counts() {
    check_controller(
        r#"
(async()=>{
  await flush();
  const project=context.ontologyModelProjection;
  assert.equal(typeof project,'function','production owns an explicit declaration projection');
  const rows=Array.from({length:52},(_,i)=>({...schemaFixture[0],entity_type:`ety_${i}`,display_name:`Type ${i}`}));
  const link=(id,from,to)=>({link_type:id,from_entity_type:from,to_entity_type:to,cardinality:'one_to_many',revision:1,allow_cross_tenant:false});
  const links=[link('self','ety_0','ety_0'),link('out','ety_0','ety_1'),link('parallel','ety_0','ety_1'),link('in','ety_2','ety_0'),link('induced','ety_1','ety_2'),link('other','ety_3','ety_4')];
  const ids=p=>[p.schemas.map(s=>s.entity_type),p.links.map(l=>l.link_type)];
  const json=v=>JSON.parse(JSON.stringify(v));
  let p=project(rows,links,{type:'ety_0',link:null},'selected');
  assert.deepEqual(json(ids(p)),[['ety_0'],[]]);
  assert.deepEqual(json(p.counts),{types:{total:52,displayed:1,remaining:51,undrawn:0},relationships:{total:6,displayed:0,remaining:6,undrawn:0}});
  p=project(rows,links,{type:'ety_0',link:null},'related');
  assert.deepEqual(json(ids(p)),[['ety_0','ety_1','ety_2'],['self','out','parallel','in','induced']]);
  p=project(rows,links,{type:null,link:'out'},'selected');
  assert.deepEqual(json(ids(p)),[['ety_0','ety_1'],['out']],'relationship selection includes exactly its directed declaration');
  p=project(rows,links,{type:null,link:'self'},'selected');
  assert.deepEqual(json(ids(p)),[['ety_0'],['self']]);
  p=project(rows,links,{type:null,link:'out'},'related');
  assert.deepEqual(json(ids(p)),[['ety_0','ety_1','ety_2'],['self','out','parallel','in','induced']]);
  const hub=Array.from({length:148},(_,i)=>link(`hub_${i}`,'ety_0',`ety_${i%51+1}`));
  p=project(rows,hub,{type:'ety_0',link:null},'related');
  assert.equal(p.schemas.length,52);assert.equal(p.links.length,148,'no invented viewport budget truncates controller truth');
  assert.equal(project([],[],{type:null,link:null},'all').schemas.length,0);
  assert.equal(project([{...rows[0],entity_type:''}],[],{type:'',link:null},'all').counts.types.undrawn,1,'a missing declaration identity is disclosed, not invented');
  assert.equal(project(rows,links,{type:'absent',link:null},'related').schemas.length,0);
  const invalidRows=[rows[0],rows[0],rows[1],rows[2]],invalidLinks=[link('duplicate','ety_1','ety_2'),link('duplicate','ety_1','ety_2'),link('missing','ety_1','ety_missing'),link('ambiguous','ety_0','ety_1'),link('valid','ety_1','ety_2')];
  p=project(invalidRows,invalidLinks,{type:'ety_0',link:null},'all');
  assert.deepEqual(json(ids(p)),[['ety_1','ety_2'],['valid']]);
  assert.deepEqual(json(p.counts),{types:{total:4,displayed:2,remaining:0,undrawn:2},relationships:{total:5,displayed:1,remaining:0,undrawn:4}});
  p=project(invalidRows,invalidLinks,{type:null,link:'missing'},'selected');
  assert.deepEqual(json(ids(p)),[['ety_1'],[]]);
  for(const mode of ['selected','related','all'])for(const counts of Object.values(project(invalidRows,invalidLinks,{type:'ety_1',link:null},mode).counts))assert.equal(counts.total,counts.displayed+counts.remaining+counts.undrawn);
  assert.throws(()=>project(rows,links,{type:'ety_0',link:null},'unrecognized'),/scope/);
  console.log('Exact Selected/Related/All identity, self/parallel/induced membership,52/148hub and invalid draft counts PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn draft_graph_and_table_activation_preserve_semantic_focus_and_refuse_pending_edits() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const enter=(id,value)=>{find('live-ontology-'+id).value=value;find('live-ontology-'+id).emit('input');};
  enter('new-entity-id','ety_team');enter('new-entity-name','Team');find('live-ontology-entity-add').emit('submit');
  find('live-ontology-scope-all').emit('click');
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene');
  const button=(surface,id)=>find('live-ontology-'+surface).querySelectorAll('[data-model-type]').find(b=>b.dataset.modelType===id);
  for(const [surface,representation]of [['scene','graph'],['model-table','table'],['library-types','table']]){
    find('live-ontology-view-'+representation).emit('click');
    let invoke=button(surface,'ety_record');invoke.focus();invoke.emit('click');
    assert.ok(document.activeElement===button(surface,'ety_record'),'successful draft activation keeps its semantic focus on '+surface);
    assert.equal(document.activeElement.disabled,false);
    assert.equal(find('live-ontology-context-name').textContent,'Record');
  }
  find('live-ontology-view-graph').emit('click');find('live-ontology-zoom-in').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  const transform=scene.style.transform,requests=calls.length;
  enter('entity-name','Unapplied Record');
  const state=()=>JSON.stringify({heading:find('live-ontology-context-name').textContent,summary:find('live-ontology-model-summary').textContent,value:find('live-ontology-entity-name').value,pressed:scene.querySelectorAll('[data-model-type]').map(b=>[b.dataset.modelType,b.attrs['aria-pressed']]),transform:scene.style.transform,scroll:[graph.scrollLeft,graph.scrollTop]});
  const before=state();let invoke=button('scene','ety_team');invoke.focus();invoke.emit('click');
  assert.ok(document.activeElement===invoke,'refused selection retains the invoking control');assert.equal(state(),before);assert.equal(calls.length,requests);
  find('live-ontology-entity-edit').emit('submit');
  assert.equal(scene.style.transform,transform);assert.deepEqual([graph.scrollLeft,graph.scrollTop],[83,71],'same-source apply preserves deliberate viewport');
  const external=find('shell-session-entry');external.focus();find('live-ontology-entity-edit').emit('submit');assert.ok(document.activeElement===external,'outside focus is not stolen by synchronous rerender');
  assert.ok(calls.slice(requests).every(([,options])=>(options.method||'GET')==='GET'),'scope/selection never submits ontology writes');
  console.log('Active draft graph/Table/library accepted focus, pending refusal and same-context viewport PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn model_scope_binds_source_and_preserves_only_owned_viewport_and_focus() {
    check_controller(
        r#"
(async()=>{
  await flush();root.hidden=false;
  const rows=schemaFixture.map((row,i)=>({...row,display_name:`Type ${i}`}));
  const source={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'};
  let current=rows,refused=false,view;
  view=context.createOntologyModelView(id=>{if(refused)return false;view.setModel(current,[],{type:id,link:null},'Draft',source);return true;},()=>find('live-ontology-types-refresh').focus());
  view.setModel(rows,[],{type:'ety_record',link:null},'Draft',source);
  assert.equal(view.setScope('selected'),true);assert.equal(view.scope().mode,'selected');
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene'),cards=()=>scene.querySelectorAll('[data-model-type]');
  assert.deepEqual(cards().map(b=>b.dataset.modelType),['ety_record']);
  assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-type]').length,2,'Table is complete independently of graph scope');
  graph.scrollLeft=64;graph.scrollTop=48;view.setModel(rows,[],{type:'ety_record',link:null},'New caption',source);
  assert.equal(view.scope().mode,'selected');assert.deepEqual([graph.scrollLeft,graph.scrollTop],[64,48],'caption changes cannot reset same native source');
  const original=cards()[0];original.focus();view.setModel(rows,[],{type:'ety_record',link:null},'Draft',source);assert.ok(document.activeElement===cards()[0]);
  find('live-ontology-search').value='absent';find('live-ontology-search').emit('input');assert.equal(cards().length,1,'library filtering does not change scope');
  assert.equal(view.setScope('unrecognized'),false);assert.equal(view.scope().mode,'selected');
  view.setScope('all');const invoke=cards()[1];invoke.focus();refused=true;invoke.emit('click');assert.ok(document.activeElement===invoke);assert.equal(cards()[0].attrs['aria-pressed'],'true');
  refused=false;invoke.emit('click');assert.ok(document.activeElement===cards()[1]);assert.equal(cards()[1].attrs['aria-pressed'],'true');
  view.setScope('related');graph.scrollLeft=90;graph.scrollTop=70;view.clear(true);assert.equal(cards().length,0);assert.equal(find('live-ontology-model-summary').textContent,'');
  view.setModel(rows,[],{type:'ety_profile',link:null},'Draft',source);assert.equal(view.scope().mode,'related');assert.deepEqual([graph.scrollLeft,graph.scrollTop],[90,70],'verified same-source reopen restores geometry without retaining DOM rows');
  view.setModel(rows,[],{type:'ety_record',link:null},'Draft',{...source,userId:'usr_other'});assert.equal(view.scope().mode,'related');assert.deepEqual([graph.scrollLeft,graph.scrollTop],[0,0]);
  view.setScope('selected');view.setModel(rows,[],{type:'ety_record',link:null},'Draft',{...source,id:'2'});assert.equal(view.scope().mode,'related','draft IDs bind view context');
  view.setScope('selected');view.setModel(rows,[],null,'Draft',{...source,id:'2',base:'3'});assert.equal(view.scope().mode,'related','draft base changes bind view context');
  const published={tenantId:source.tenantId,userId:source.userId,kind:'published',generation:'3'};
  view.setModel(rows,[],{type:'ety_record',link:null},'Published',published);view.setScope('selected');
  view.setModel(rows,[],null,'Published',{...published,generation:'4'});assert.equal(view.scope().mode,'related','published generation changes bind view context');
  view.setScope('selected');view.setModel(rows,[],{type:'ety_record',link:null},'Draft',null);assert.equal(view.scope().mode,'related','unknown source cannot inherit context');
  const invalid=[rows[0],rows[0],rows[1]];view.setModel(invalid,[],{type:'ety_record',link:null},'Draft',source);view.setScope('all');
  assert.equal(cards().length,1);const raw=find('live-ontology-model-table').querySelectorAll('[data-model-type]');assert.equal(raw.length,3);assert.ok(raw.slice(0,2).every(b=>b.disabled),'ambiguous raw declarations remain visible but cannot select the first row');
  assert.match(find('live-ontology-graph-counts').textContent,/2 undrawn/);
  cards()[0].focus();view.setModel([],[],null,'Draft',source);assert.ok(document.activeElement===graph,'a disappearing semantic target uses a visible local fallback');
  console.log('Source-bound scope, complete Table, ambiguity refusal, owned focus and viewport retention PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn coherent_published_selection_survives_recheck_without_stealing_outside_focus() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.entities.push({...structuredClone(modelDocument.entities[0]),id:'ety_team',display_name:'Team'});
  modelDocument.links.push({id:'lty_team',from_entity_type:'ety_record',to_entity_type:'ety_team',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene'),link=scene.querySelectorAll('[data-model-link]')[0];link.focus();link.emit('click');
  assert.equal(find('live-ontology-context-selected').textContent,'Relationship · lty_team');
  find('live-ontology-zoom-in').emit('click');graph.scrollLeft=80;graph.scrollTop=64;const transform=scene.style.transform;
  const external=find('shell-session-entry');external.focus();await restore();
  assert.ok(document.activeElement===external,'verified recheck does not restore an older declaration focus owner');
  assert.equal(find('live-ontology-context-selected').textContent,'Relationship · lty_team','same coherent source retains accepted relationship selection');
  assert.equal(scene.style.transform,transform);assert.deepEqual([graph.scrollLeft,graph.scrollTop],[80,64]);
  console.log('Coherent published selection, user viewport and newer outside focus survive verified recheck PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

fn check_all_camera(variant: &str, action: &str) {
    let check = r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  const base=structuredClone(modelDocument.entities[0]);
  modelDocument.entities.push({...base,id:'ety_team',display_name:'Team'});
  modelDocument.links.push({id:'lty_forward',from_entity_type:'ety_record',to_entity_type:'ety_team',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null},{id:'lty_reverse',from_entity_type:'ety_team',to_entity_type:'ety_record',cardinality:'many_to_many',allow_cross_tenant:false,revision:1,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  if('__VARIANT__'==='draft'){find('live-ontology-draft-new').emit('click');await flush();}
  find('live-ontology-scope-all').emit('click');
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene');
  const invoke=()=>find('live-ontology-library-links').querySelectorAll('[data-model-link]').find(row=>row.dataset.modelLink==='lty_forward');
  const snapshot=()=>({scale:scene.style.transform,text:find('live-ontology-zoom').textContent,scroll:[graph.scrollLeft,graph.scrollTop]});
  const resize=()=>{const observer=resizeObservers.find(row=>row.observing && row.targets.has(graph));assert.ok(observer,'production graph ResizeObserver remains connected');observer.emit();};
  find('live-ontology-zoom-out').emit('click');find('live-ontology-zoom-out').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  assert.equal(scene.style.transform,'scale(0.5)');const requests=calls.length;
  invoke().focus();invoke().emit('click');const selected=snapshot();
  assert.equal(invoke().getAttribute('aria-pressed'),'true');assert.equal(find('live-ontology-context-selected').textContent,'Relationship · lty_forward');assert.ok(document.activeElement===invoke(),'accepted selection retains its invoking semantic focus');
  if('__ACTION__'==='zoom'){
    find('live-ontology-zoom-out').emit('click');const after=snapshot();console.log(JSON.stringify({variant:'__VARIANT__',action:'__ACTION__',selected,after}));
    assert.ok(Number(after.scale.match(/scale\(([^)]+)\)/)[1])<Number(selected.scale.match(/scale\(([^)]+)\)/)[1]),'Zoom out decreases the displayed accepted-selection scale');assert.equal(after.scale,'scale(0.75)');assert.equal(after.text,'75%');
  }else{resize();const after=snapshot();console.log(JSON.stringify({variant:'__VARIANT__',action:'__ACTION__',selected,after}));assert.deepEqual(after,selected,'resize cannot defer a previously unrendered camera transition');}
  assert.equal(selected.scale,'scale(1)');assert.equal(selected.text,'100%','new accepted selection frames immediately at natural scale');
  find('live-ontology-fit').emit('click');find('live-ontology-zoom-out').emit('click');find('live-ontology-zoom-out').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  const same=snapshot();invoke().focus();invoke().emit('click');assert.deepEqual(snapshot(),same,'same selection preserves the manual camera');
  find('live-ontology-zoom-out').emit('click');assert.equal(scene.style.transform,'scale(0.25)');const manual=snapshot();resize();assert.deepEqual(snapshot(),manual,'resize retains the same-selection camera and pan');
  find('live-ontology-scope-related').emit('click');assert.equal(scene.style.transform,'scale(1)','changed scope still frames intentionally');assert.equal(find('live-ontology-zoom').textContent,'100%');
  find('live-ontology-zoom-out').emit('click');graph.scrollLeft=91;graph.scrollTop=67;const changedScope=snapshot();resize();assert.deepEqual(snapshot(),changedScope,'scope framing is consumed immediately');
  assert.ok(calls.slice(requests).every(([,options])=>(options.method||'GET')==='GET'),'selection and camera controls never write model state');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;
    check_controller(
        &check
            .replace("__VARIANT__", variant)
            .replace("__ACTION__", action),
    );
}

#[test]
fn published_all_relationship_selection_keeps_zoom_control_coherent() {
    check_all_camera("published", "zoom");
}

#[test]
fn published_all_relationship_selection_consumes_camera_before_resize() {
    check_all_camera("published", "resize");
}

#[test]
fn draft_all_relationship_selection_keeps_zoom_control_coherent() {
    check_all_camera("draft", "zoom");
}

#[test]
fn draft_all_relationship_selection_consumes_camera_before_resize() {
    check_all_camera("draft", "resize");
}

#[test]
fn catalog_all_type_selection_keeps_zoom_and_resize_coherent() {
    check_controller(
        r#"
(async()=>{
  await flush();find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene'),invoke=()=>find('live-ontology-library-types').querySelectorAll('[data-model-type]').find(row=>row.dataset.modelType==='ety_profile');
  find('live-ontology-scope-all').emit('click');find('live-ontology-zoom-out').emit('click');find('live-ontology-zoom-out').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  invoke().focus();invoke().emit('click');assert.equal(invoke().getAttribute('aria-pressed'),'true');assert.ok(document.activeElement===invoke());assert.equal(scene.style.transform,'scale(1)');assert.equal(find('live-ontology-zoom').textContent,'100%');
  find('live-ontology-zoom-out').emit('click');assert.equal(scene.style.transform,'scale(0.75)');graph.scrollLeft=83;graph.scrollTop=71;
  const observer=resizeObservers.find(row=>row.observing && row.targets.has(graph));assert.ok(observer);observer.emit();assert.equal(scene.style.transform,'scale(0.75)');assert.deepEqual([graph.scrollLeft,graph.scrollTop],[83,71]);
  invoke().emit('click');assert.equal(scene.style.transform,'scale(0.75)','same catalog selection retains manual camera');find('live-ontology-zoom-out').emit('click');assert.equal(scene.style.transform,'scale(0.5)');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}

#[test]
fn selected_camera_reflows_only_when_automatic_and_preserves_focused_declarations() {
    check_camera_reflow(false);
}

#[test]
fn focused_scene_camera_reflow_keeps_the_native_control_physically_visible() {
    check_camera_reflow(true);
}

fn check_camera_reflow(focus_only: bool) {
    let check = r#"
(async()=>{
  await flush();root.hidden=false;
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene');
  const source={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'};
  const cards=()=>scene.querySelectorAll('[data-model-type]'),card=id=>cards().find(row=>row.dataset.modelType===id);
  const camera=()=>[scene.style.transform,graph.scrollLeft,graph.scrollTop];
  const inside=node=>node.offsetLeft>=graph.scrollLeft-.5 && node.offsetLeft+node.offsetWidth<=graph.scrollLeft+graph.clientWidth+.5 && node.offsetTop>=graph.scrollTop-.5 && node.offsetTop+node.offsetHeight<=graph.scrollTop+graph.clientHeight+.5;
  let observer;
  const resize=(width,height)=>{graph.clientWidth=width;graph.clientHeight=height;observer.emit();};
  const view=context.createOntologyModelView(()=>true,()=>find('live-ontology-types-refresh').focus());
  const ownControl=id=>find('live-ontology-graph-controls').children.filter(row=>row.id==='live-ontology-'+id).at(-1);
  observer=resizeObservers.filter(row=>row.observing && row.targets.has(graph)).at(-1);
  assert.ok(observer,'regression drives the native graph ResizeObserver');
  for(const [index,primary] of ['ety_camera_anchor','ety_other_anchor'].entries()){
    const rows=Array.from({length:8},(_,i)=>({...schemaFixture[0],entity_type:i?`ety_neighbor_${i}`:primary,display_name:i?'Neighbor '+i:primary}));
    const links=rows.slice(1).map((row,i)=>({link_type:`lty_camera_${i}`,from_entity_type:primary,to_entity_type:row.entity_type,cardinality:'one_to_many',revision:1,allow_cross_tenant:false}));
    resize(950,600);view.setModel(rows,links,{type:primary,link:null},'Draft',{...source,id:String(index+1)});
    assert.equal(view.setScope('related'),true,'the test admits a current native model source identity');
    for(let attempt=0;attempt<rows.length && cards().length<rows.length && !ownControl('expand').disabled;attempt++)ownControl('expand').emit('click');
    assert.equal(cards().length,8);assert.ok(inside(card(primary)));
    if(!__FOCUS_ONLY__){
    resize(607,468);const middle=camera();assert.ok(inside(card(primary)),'selected card remains visible before height shrink');
    observer.emit();assert.deepEqual(camera(),middle,'unchanged geometry does not move a visible automatic selection');
    resize(731,323);console.log(JSON.stringify({primary,camera:camera(),selected:{x:card(primary).offsetLeft,y:card(primary).offsetTop,w:card(primary).offsetWidth,h:card(primary).offsetHeight},viewport:[graph.clientWidth,graph.clientHeight]}));assert.ok(inside(card(primary)),'responsive reflow keeps the automatically selected card completely inside the graph');
    assert.equal(cards().length,8,'automatic framing never drops retained neighbors');assert.equal(scene.querySelectorAll('[data-model-edge]').length,7);
    for(const action of ['wheel','touchmove','scrollbar','arrow','zoom','overview']){
      resize(950,600);find('live-ontology-fit').emit('click');
      if(action==='wheel')graph.emit('wheel',{deltaY:60});
      if(action==='touchmove')graph.emit('touchmove');
      if(action==='scrollbar')graph.emit('pointerdown');
      if(action==='arrow')graph.emit('keydown',{key:'ArrowDown'});
      if(action==='zoom')find('live-ontology-zoom-in').emit('click');
      if(action==='overview')ownControl('overview').emit('click');
      graph.scrollLeft=13;graph.scrollTop=17;graph.emit('scroll',{isTrusted:true});
      if(action==='scrollbar')context.window.emit('pointerup');
      const deliberate=camera();resize(731,323);assert.deepEqual(camera(),deliberate,action+' retains deliberate camera through reflow');
    }
    for(const reset of ['fit','Home']){
      resize(950,600);
      if(reset==='fit')find('live-ontology-fit').emit('click');else graph.emit('keydown',{key:'Home'});
      graph.scrollTop=17;graph.emit('scroll',{isTrusted:true});
      resize(731,323);assert.ok(inside(card(primary)),reset+' restores automatic selected framing; scroll alone does not manufacture manual intent');
    }
    }
    resize(950,600);find('live-ontology-fit').focus();find('live-ontology-fit').emit('click');
    const other=cards().filter(row=>row.dataset.modelType!==primary).sort((a,b)=>b.offsetTop-a.offsetTop)[0];
    assert.ok(other && inside(other));const focusedId=other.dataset.modelType;other.focus();
    resize(607,468);assert.equal(document.activeElement,card(focusedId));assert.ok(inside(card(focusedId)),'three-to-two columns keep the different focused declaration physically visible');
    resize(731,323);
    assert.equal(document.activeElement,card(focusedId));assert.ok(inside(card(focusedId)),'height shrink keeps the focused control physically visible, not merely named');
    assert.equal(document.activeElement.focusOptions?.preventScroll,true);
    resize(950,600);find('live-ontology-fit').focus();find('live-ontology-fit').emit('click');const external=find('shell-session-entry');external.focus();
    resize(731,323);assert.ok(inside(card(primary)));assert.equal(document.activeElement,external,'external focus is not stolen by automatic framing');
  }
  console.log('Automatic selected reflow, distinct identities, retained topology, explicit manual camera, Fit/Home and physically visible native focus PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;
    check_controller(&check.replace("__FOCUS_ONLY__", if focus_only { "true" } else { "false" }));
}
