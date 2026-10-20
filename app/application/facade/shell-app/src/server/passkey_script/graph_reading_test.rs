use super::check_controller;

#[test]
fn graph_reading_cue_uses_native_identity_priority_and_render_lifetime() {
    check_controller(
        r#"
(async()=>{
  await flush();root.hidden=false;
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene');
  const rows=['ety_record','ety_profile','ety_third'].map(entity_type=>({...schemaFixture[0],entity_type,display_name:'Same type name'}));
  const link=(link_type,from_entity_type,to_entity_type)=>({link_type,from_entity_type,to_entity_type,cardinality:'one_to_many',revision:1,allow_cross_tenant:false,display:{display_name:'Same relationship name'}});
  const links=[link('lty_out','ety_record','ety_profile'),link('lty_parallel','ety_record','ety_profile'),link('lty_self','ety_record','ety_record'),link('lty_reverse','ety_profile','ety_record'),link('lty_induced','ety_profile','ety_third')];
  const source={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'71',base:'2'},selections=[];
  const view=context.createOntologyModelView(id=>{selections.push(['type',id]);return true;},()=>selections.push(['fallback']),id=>{selections.push(['link',id]);return true;});
  view.setModel(rows,links,{type:null,link:'lty_out'},'Draft',source);view.setScope('all');
  const labels=new Map(scene.querySelectorAll('[data-model-link]').map(node=>[node.dataset.modelLink,node]));
  const paths=new Map(scene.querySelectorAll('[data-model-edge]').map(node=>[node.dataset.modelEdge,node]));
  const markers=scene.querySelectorAll('marker'),arrows=new Map(markers.map(marker=>[marker.children[0].dataset.modelArrow,marker.children[0]]));
  assert.equal(markers.length,links.length,'every drawn native route owns its effective arrow paint');
  assert.deepEqual([...labels.keys()],links.map(row=>row.link_type));assert.deepEqual([...arrows.keys()],links.map(row=>row.link_type));
  const visibleText=node=>node.textContent+node.children.map(visibleText).join('');
  for(const row of links){
    const label=labels.get(row.link_type),arrow=arrows.get(row.link_type),path=paths.get(row.link_type);
    assert.equal(visibleText(label),'Same relationship nameone → many');
    assert.equal(label.children[0].className,'foundry-edge-text','the native name and cardinality share a stroke-protecting backplate');
    for(const literal of [row.link_type,row.from_entity_type,row.to_entity_type])assert.ok(label.getAttribute('aria-label').includes(literal));
    assert.equal(path.getAttribute('marker-end'),'url(#'+arrow.parentElement.getAttribute('id')+')','path references its own native-ID arrow');
  }
  assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-link]').length,5);
  const subscriptions=()=>JSON.stringify([document,context.window,graph,scene].map(owner=>Object.entries(owner.events).map(([name,actions])=>[name,actions.length])));
  const registered=subscriptions(),external=find('shell-session-entry');external.focus();graph.scrollLeft=83;graph.scrollTop=71;
  const state=()=>JSON.stringify({rows,links,scope:view.scope(),selection:selections,requests:calls.length,table:visibleText(find('live-ontology-model-table')),camera:[graph.scrollLeft,graph.scrollTop,scene.style.transform],active:document.activeElement.id,pressed:[...labels.values()].map(node=>node.getAttribute('aria-pressed')),subscriptions:subscriptions()});
  const geometry=()=>JSON.stringify([...paths].map(([id,path])=>[id,path.getAttribute('d'),path.getAttribute('marker-end'),arrows.get(id).getAttribute('d'),['markerWidth','markerHeight','refX','refY','orient'].map(key=>arrows.get(id).parentElement.getAttribute(key))]));
  const before=state(),shape=geometry(),writes=[];
  for(const [id,parts]of [...paths].map(([id,path])=>[id,[path,arrows.get(id)]]))for(const part of parts){const set=part.setAttribute.bind(part);part.setAttribute=(name,value)=>{if(name==='data-model-reading')writes.push(id);set(name,value);};}
  const cue=id=>{for(const parts of [paths,arrows])assert.deepEqual([...parts].filter(([,node])=>node.getAttribute('data-model-reading')==='true').map(([key])=>key),id?[id]:[]);if(id)assert.equal(paths.get(id).parentElement.children.at(-1),paths.get(id),'current native route paints above shared segments');};
  const fire=(id,event,expected,touched)=>{writes.length=0;labels.get(id).emit(event);cue(expected);assert.equal(state(),before,'reading events change no selection, requests, model, focus, camera, scope or Table');assert.equal(geometry(),shape);assert.deepEqual([...new Set(writes)].sort(),touched.sort());assert.equal(writes.length,touched.length*2,'only previous/current path and own arrow are painted');};
  // Synthetic label events prove handler logic; real Tab/focus is qualified separately.
  cue('lty_out');fire('lty_parallel','mouseenter','lty_parallel',['lty_out','lty_parallel']);
  fire('lty_self','focus','lty_self',['lty_parallel','lty_self']);
  fire('lty_induced','mouseenter','lty_self',[]);fire('lty_parallel','mouseleave','lty_self',[]);
  fire('lty_self','blur','lty_induced',['lty_self','lty_induced']);
  fire('lty_induced','mouseleave','lty_out',['lty_induced','lty_out']);
  fire('lty_out','focus','lty_out',[]);fire('lty_reverse','mouseenter','lty_out',[]);
  fire('lty_out','blur','lty_reverse',['lty_out','lty_reverse']);fire('lty_reverse','mouseleave','lty_out',['lty_reverse','lty_out']);
  for(const row of links){const changed=row.link_type==='lty_out'?[]:['lty_out',row.link_type];fire(row.link_type,'mouseenter',row.link_type,[...changed]);fire(row.link_type,'mouseleave','lty_out',[...changed]);}
  const retired=[...labels.values()],noCue=()=>assert.equal(scene.querySelectorAll('path').filter(node=>node.getAttribute('data-model-reading')==='true').length,0);
  for(let index=0;index<3;index++){
    retired.push(...scene.querySelectorAll('[data-model-link]'));
    view.setModel(rows,links,{type:'ety_record',link:null},'Draft',{...source,id:String(72+index)});view.setScope('all');noCue();
    for(const old of retired){assert.equal(old.isConnected,false);old.emit('focus');old.emit('mouseenter');}
    noCue();assert.equal(subscriptions(),registered,'rerender adds no document/window/graph/scene subscriptions');assert.equal(document.activeElement,external);
  }
  view.setModel(rows,[...links,{...links[0]}],{type:null,link:'lty_out'},'Draft',source);view.setScope('all');noCue();
  assert.ok(!scene.querySelectorAll('[data-model-link]').some(node=>node.dataset.modelLink==='lty_out'),'ambiguous selected native IDs do not acquire a cue');
  assert.equal(find('live-ontology-model-table').querySelectorAll('[data-model-link]').length,6,'ambiguous raw declarations remain in the complete Table');
  view.setModel(rows,links,{type:null,link:'unmatched'},'Draft',source);view.setScope('all');noCue();
  retired.push(...scene.querySelectorAll('[data-model-link]'));view.clear();
  for(const old of retired){old.emit('focus');old.emit('mouseenter');}
  assert.equal(scene.children.length,0);assert.equal(subscriptions(),registered);assert.deepEqual(selections,[]);assert.equal(document.activeElement,external);
  console.log('Synthetic native-ID path/own-arrow priority, duplicate names, parallel/self/reverse/induced edges, passive paint and disposed render lifetime PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
#[test]
fn draft_graph_reading_cue_preserves_pending_edits_and_manual_camera() {
    check_controller(
        r#"
(async()=>{
  await flush();modelEnabled=true;session.roles=['foundry-model-author','foundry-operator'];
  modelDocument.entities.push({...structuredClone(modelDocument.entities[0]),id:'ety_team',display_name:'Team'});
  modelDocument.links.push({id:'lty_team',from_entity_type:'ety_record',to_entity_type:'ety_team',cardinality:'one_to_many',allow_cross_tenant:false,revision:1,display:null});
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene');
  find('live-ontology-zoom-out').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  find('live-ontology-entity-name').value='Unapplied Record';find('live-ontology-entity-name').emit('input');
  const label=scene.querySelectorAll('[data-model-link]').find(node=>node.dataset.modelLink==='lty_team');assert.ok(label);
  const external=find('shell-session-entry');external.focus();
  const state=()=>JSON.stringify({value:find('live-ontology-entity-name').value,selected:find('live-ontology-context-selected').textContent,summary:find('live-ontology-model-summary').textContent,counts:find('live-ontology-graph-counts').textContent,transform:scene.style.transform,scroll:[graph.scrollLeft,graph.scrollTop],pressed:scene.querySelectorAll('[data-model-link]').map(node=>node.getAttribute('aria-pressed')),session,modelDocument,requests:calls.length,active:document.activeElement.id});
  const before=state(),painted=()=>scene.querySelectorAll('path').filter(node=>node.getAttribute('data-model-reading')==='true');
  for(const [event,count]of [['mouseenter',2],['focus',2],['mouseleave',2],['blur',0]]){
    label.emit(event);assert.equal(painted().length,count);assert.ok(painted().every(node=>(node.dataset.modelEdge || node.dataset.modelArrow)==='lty_team'));
    assert.equal(state(),before,'cue handlers preserve pending input, selection/session, native model, camera/counts, requests and outside focus');
  }
  label.emit('click');assert.equal(state(),before,'a reading cue cannot bypass pending-edit selection refusal');
  console.log('Pending draft cue is passive and preserves manual camera, native records, session, requests and selection refusal PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#,
    );
}
