use super::check_controller;

fn check_navigation(check: &str) {
    let setup = r#"
(async()=>{
  await flush();root.hidden=false;
  const graph=find('live-ontology-graph'),scene=find('live-ontology-scene'),frames=[];
  context.requestAnimationFrame=callback=>frames.push(callback);
  const drain=()=>{while(frames.length)frames.shift()();};
  graph.clientWidth=848;graph.clientHeight=509;graph.clientLeft=1;graph.clientTop=1;
  graph.getBoundingClientRect=()=>({left:328.75,top:198.25,right:1178.75,bottom:709.25,width:850,height:511});
  const rows=[{...schemaFixture[0],entity_type:'ety_keyboard_main',display_name:'Keyboard main'},{...schemaFixture[0],entity_type:'ety_keyboard_other',display_name:'Keyboard other'}];
  const links=[{link_type:'lty_keyboard',from_entity_type:rows[0].entity_type,to_entity_type:rows[1].entity_type,cardinality:'one_to_many',revision:1,allow_cross_tenant:false}];
  const source={tenantId:'ten_acme',userId:'usr_alice',kind:'draft',id:'1',base:'2'};
  const view=context.createOntologyModelView(()=>true,()=>{},()=>true);
  const load=(id='1')=>view.setModel(rows,links,{type:rows[0].entity_type,link:null},'Draft',{...source,id});
  load();
  const target=kind=>scene.querySelectorAll(kind==='type'?'[data-model-type]':'[data-model-link]')[0];
  const position=(node,left,top,width,height)=>{const x=left+graph.scrollLeft,y=top+graph.scrollTop;node.getBoundingClientRect=()=>({left:x-graph.scrollLeft,top:y-graph.scrollTop,right:x-graph.scrollLeft+width,bottom:y-graph.scrollTop+height,width,height});};
  const navigate=(from,to,options={})=>{from.focus({preventScroll:true});const event={key:'Tab',target:from,isTrusted:true,defaultPrevented:false,...options};document.emit('keydown',event);to.focus({preventScroll:true});return event;};
  const external=find('sign-in-account'),camera=()=>[graph.scrollLeft,graph.scrollTop];
  const state=()=>JSON.stringify({zoom:scene.style.transform,selected:scene.querySelectorAll('[data-model-type]').map(e=>[e.dataset.modelType,e.getAttribute('aria-pressed')]),scope:view.scope(),native:[...modelDrafts],requests:calls.length});
"#;
    let finish = r#"
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;
    check_controller(&format!("{setup}{check}{finish}"));
}

#[test]
fn native_tab_reveals_transformed_type_and_reverse_entry_relationship() {
    check_navigation(
        r#"
  find('live-ontology-zoom-in').emit('click');
  graph.scrollLeft=83;graph.scrollTop=71;
  const card=target('type');position(card,889.90625,494.40625,364,136.5);
  const before=state();navigate(graph,card);drain();
  assert.deepEqual(camera(),[160,71],'native keyboard focus reveals the clipped visual right edge with the smallest whole-pixel pan');
  assert.equal(state(),before,'reveal preserves zoom, selection, scope, native records and requests');
  assert.equal(document.activeElement,card);
  const label=target('link');position(label,318.75,188.25,104,44);
  const reverse=state();navigate(external,label,{shiftKey:true});drain();
  assert.deepEqual(camera(),[149,60],'reverse Tab from outside the graph reveals an owned relationship on both clipped axes');
  assert.equal(state(),reverse);assert.equal(document.activeElement,label);
  console.log('Native Tab transformed type and reverse external-entry relationship reveal PASS');
"#,
    );
}

#[test]
fn visible_passive_and_internal_focus_preserve_manual_camera() {
    check_navigation(
        r#"
  find('live-ontology-zoom-in').emit('click');graph.scrollLeft=83;graph.scrollTop=71;
  const card=target('type'),label=target('link');position(card,400,300,364,136.5);position(label,500,500,104,44);
  const before=state();navigate(external,card);drain();assert.deepEqual(camera(),[83,71],'already visible target does not pan');
  label.emit('mouseenter');label.emit('focus');label.emit('blur');label.emit('mouseleave');drain();
  assert.deepEqual(camera(),[83,71],'passive relationship reading cues do not pan');
  assert.equal(state(),before);
  position(card,889.90625,494.40625,364,136.5);card.focus({preventScroll:true});drain();
  assert.deepEqual(camera(),[83,71],'internal preventScroll focus restoration is not native navigation');
  const requests=calls.length,transform=scene.style.transform;load();drain();
  assert.deepEqual(camera(),[83,71],'same-source model focus restoration preserves deliberate camera');
  assert.equal(scene.style.transform,transform);assert.equal(calls.length,requests);
  const large=target('type');position(large,250,150,1000,800);navigate(external,large);drain();
  assert.deepEqual(camera(),[83,71],'oversized targets do not oscillate between unreachable edges');
  console.log('Visible, passive, internal restore and oversized camera stability PASS');
"#,
    );
}

#[test]
fn pending_keyboard_reveal_refuses_stale_foreign_and_newer_intent() {
    check_navigation(
        r#"
  graph.scrollLeft=83;graph.scrollTop=71;
  const clipped=()=>{const card=target('type');position(card,889.90625,494.40625,364,136.5);return card;};
  let card=clipped();navigate(external,card,{isTrusted:false});drain();assert.deepEqual(camera(),[83,71]);
  navigate(external,card,{defaultPrevented:true});drain();assert.deepEqual(camera(),[83,71]);
  const event=navigate(external,card);event.defaultPrevented=true;drain();assert.deepEqual(camera(),[83,71],'late cancellation is respected');
  navigate(external,card);external.focus();drain();assert.deepEqual(camera(),[83,71],'newer outside focus is not moved or stolen');assert.equal(document.activeElement,external);
  navigate(external,card);graph.emit('pointerdown',{target:card});drain();assert.deepEqual(camera(),[83,71],'newer pointer intent cancels pending keyboard pan');
  for(const intent of ['wheel','touchmove']){
    navigate(external,card);graph.emit(intent,{deltaX:37,deltaY:29});graph.scrollLeft=120;graph.scrollTop=100;drain();
    assert.deepEqual(camera(),[120,100],intent+' manual pan cancels the earlier keyboard reveal');
    graph.scrollLeft=83;graph.scrollTop=71;
  }
  navigate(external,card);card.isConnected=false;drain();assert.deepEqual(camera(),[83,71],'disconnected target is refused');card.isConnected=true;
  const foreign=new Element('button');foreign.dataset.modelType='ety_foreign';scene.append(foreign);position(foreign,889.90625,494.40625,364,136.5);navigate(external,foreign);drain();assert.deepEqual(camera(),[83,71],'unowned scene node is refused');scene.removeChild(foreign);
  navigate(external,card);load();drain();assert.deepEqual(camera(),[83,71],'intervening render cancels the old navigation ticket');
  card=clipped();navigate(external,card);load('2');const changed=camera();drain();assert.deepEqual(camera(),changed,'new model source owns its own camera');
  card=clipped();navigate(external,card);view.clear();const cleared=camera();drain();assert.deepEqual(camera(),cleared,'cleared model cancels the old target');
  console.log('Stale, foreign, canceled, disconnected and newer-intent refusals PASS');
"#,
    );
}
