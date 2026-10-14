use super::check_controller;

const CHECK: &str = r#"
(async()=>{
  for(const id of ['relationship-type','relationship-direction']) {
    assert.ok(input.html.includes(`id="live-ontology-${id}"`));assert.ok(input.html.includes(`for="live-ontology-${id}"`));
  }
  assert.ok(input.html.includes('A bounded one-hop table of object instances.'));
  assert.ok(input.html.includes('an instance graph canvas, multi-hop exploration'));
  nativeLinks=[{link_type:'lty_profile',from_entity_type:'ety_record',to_entity_type:'ety_profile',cardinality:'one_to_many',allow_cross_tenant:false,revision:1},{link_type:'lty_self',from_entity_type:'ety_profile',to_entity_type:'ety_profile',cardinality:'many_to_many',allow_cross_tenant:false,revision:1}];
  const graph={nodes:[{entity_id:'ent_alpha',entity_type_id:'ety_record',written_revision:1},{entity_id:'ent_profile',entity_type_id:'ety_profile',written_revision:3}],edges:[{from_entity_id:'ent_alpha',to_entity_id:'ent_profile',edge_type_id:'lty_profile'}],truncated:false};
  nativeGraph=graph;nativeDetails.set('ent_profile',{object_ref:'ent_profile',written_revision:3,upcast_state:'current',properties:{name:{value_type:'string',data_class:'INTERNAL_ONLY',value:'<img onerror=bad>'}}});
  await flush();find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-mode-objects').emit('click');find('live-ontology-query').emit('submit');await flush();
  const openRoot=async()=>{const row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();row.emit('click');await flush();};
  const select=()=>{find('live-ontology-relationship-type').value='lty_profile';find('live-ontology-relationship-type').emit('input');};
  const read=async()=>{select();find('live-ontology-relationship-read').focus();find('live-ontology-relationship-read').emit('click');await flush();};
  await openRoot();assert.equal(find('live-ontology-relationship-read').disabled,true,'relationship must be explicitly selected');
  assert.match(text(find('live-ontology-relationship-type')),/lty_profile.*Record.*직원 <img/,'unsafe labels are literal');
  await read();
  let requests=calls.filter(([url])=>url.endsWith('/search-around'));assert.equal(requests.length,1);
  const first=JSON.parse(requests[0][1].body);
  assert.deepEqual(first.root,{entity_type:'ety_record',object_ref:'ent_alpha',revision:1});assert.equal(first.link_type,'lty_profile');assert.equal(first.direction,'outbound');
  assert.equal(Object.keys(first).length,5);assert.ok(first.idempotency_key);assert.equal(requests[0][1].method,'POST');assert.equal(requests[0][1].headers['content-type'],'application/json');
  assert.match(text(find('live-ontology-relationship-results')),/lty_profile.*ent_alpha.*ent_profile.*ety_profile.*3/);
  assert.match(find('live-ontology-relationship-status').textContent,/caller intent metadata/);
  assert.equal(document.activeElement,find('live-ontology-relationship-read'),'rendering results retains control focus');
  await read();requests=calls.filter(([url])=>url.endsWith('/search-around'));assert.notEqual(JSON.parse(requests[1][1].body).idempotency_key,first.idempotency_key,'explicit refresh gets a new key');
  let neighbor=find('live-ontology-relationship-results').querySelectorAll('[data-live-object]')[0];neighbor.focus();neighbor.emit('click');await flush();
  assert.equal(find('live-ontology-revision').value,'1','source listing pin remains unchanged');
  assert.ok(calls.some(([url])=>url.endsWith('/objects/ent_profile?revision=3')),'target uses its own written revision');
  assert.match(find('live-ontology-detail-title').textContent,/ent_profile.*ety_profile.*requested schema 3/);
  assert.equal(document.activeElement,find('live-ontology-detail-title'),'owned neighbor activation focuses target heading');
  assert.match(text(find('live-ontology-properties')),/<img onerror=bad>/);
  find('live-ontology-history-read').emit('click');await flush();assert.ok(calls.some(([url])=>url.endsWith('/objects/ent_profile/history')));
  intervals[0]();await flush();assert.ok(calls.at(-1)[0].endsWith('/objects/ent_profile/history'),'same-user refresh keeps cross-type context and history');
  assert.match(find('live-ontology-detail-title').textContent,/ent_profile.*requested schema 3/);
  find('live-ontology-relationship-direction').value='inbound';find('live-ontology-relationship-direction').emit('input');
  await read();assert.equal(JSON.parse(calls.filter(([url])=>url.endsWith('/search-around')).at(-1)[1].body).direction,'inbound');
  assert.match(text(find('live-ontology-relationship-results')),/ent_alpha/,'canonical edge direction stays source to target');
  find('live-ontology-relationship-direction').value='outbound';find('live-ontology-relationship-direction').emit('input');find('live-ontology-relationship-type').value='lty_self';find('live-ontology-relationship-type').emit('input');
  nativeGraph={nodes:[graph.nodes[1]],edges:[{from_entity_id:'ent_profile',to_entity_id:'ent_profile',edge_type_id:'lty_self'}],truncated:false};find('live-ontology-relationship-read').emit('click');await flush();
  assert.match(text(find('live-ontology-relationship-results')),/ent_profile \(self relationship\)/,'a valid self-edge remains a truthful related row');assert.doesNotMatch(find('live-ontology-relationship-status').textContent,/No related objects/);nativeGraph=graph;
  await openRoot();await read();neighbor=find('live-ontology-relationship-results').querySelectorAll('[data-live-object]')[0];neighbor.focus();holdPath='/objects/ent_profile?';neighbor.emit('click');await flush();find('sign-in-account').focus();held();await flush();
  assert.equal(document.activeElement,find('sign-in-account'),'delayed neighbor detail never steals newer outside focus');
  await openRoot();nativeGraph={...graph,edges:[],nodes:[graph.nodes[0]]};await read();assert.match(find('live-ontology-relationship-status').textContent,/No related objects.*root exists.*completed/);assert.equal(find('live-ontology-relationship-results').querySelectorAll('[data-live-object]').length,0);
  nativeGraph={...graph,truncated:true};await read();assert.match(find('live-ontology-relationship-status').textContent,/INCOMPLETE.*partial one-hop.*no next page/);
  nativeGraph=graph;await read();holdPath='/search-around';find('live-ontology-relationship-read').emit('click');await flush();
  find('live-ontology-revision').value='2';find('live-ontology-revision').emit('input');assert.equal(find('live-ontology-relationship-results').children.length,0,'pin changes clear results synchronously');held();await flush();assert.equal(find('live-ontology-relationship-results').children.length,0,'late previous-root graph stays cleared');
  find('live-ontology-query').emit('submit');await flush();await openRoot();await read();holdPath='/search-around';find('live-ontology-relationship-read').emit('click');await flush();await openRoot();held();await flush();assert.equal(find('live-ontology-relationship-results').children.length,0,'late graph after another root read stays cleared');
  await read();find('live-ontology-relationship-results').querySelectorAll('[data-live-object]')[0].focus();holdPath='/session';intervals[0]();assert.equal(document.activeElement,find('live-ontology-type'),'cleared relationship focus settles on an enabled stable control');assert.equal(find('live-ontology-relationship-results').children.length,0,'session revalidation clears relationship results synchronously');assert.equal(find('live-ontology-relationship-root').textContent,'','session hold clears traversal context');await flush();held();await flush();
  await openRoot();await read();holdPath='/search-around';find('live-ontology-relationship-read').emit('click');await flush();find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');assert.equal(find('live-ontology-relationship-results').children.length,0);held();await flush();assert.equal(root.hidden,true,'old-account graph cannot return');
  find('sign-in-account').value='usr_alice';await restore();find('live-ontology-query').emit('submit');await flush();await openRoot();await read();
  holdPath='/search-around';find('live-ontology-relationship-read').emit('click');await flush();find('sign-in-tenant').value='ten_other';find('sign-in-tenant').emit('input');held();await flush();assert.equal(root.hidden,true,'old-tenant graph cannot return');
  find('sign-in-tenant').value='ten_acme';await restore();find('live-ontology-query').emit('submit');await flush();await openRoot();await read();
  for(const error of ['forbidden','source_unavailable','invalid_query','revision_conflict']) {
    nativeError=error;find('live-ontology-relationship-read').emit('click');await flush();assert.equal(find('live-ontology-relationship-results').children.length,0,'refusal never retains graph or successful empty');
    assert.equal(find('live-ontology-detail').hidden,true);nativeError=null;await restore();find('live-ontology-query').emit('submit');await flush();await openRoot();await read();
  }
  holdPath='/search-around';find('live-ontology-relationship-read').emit('click');await flush();timers.at(-1)();held();await flush();assert.equal(root.hidden,true,'expiry clears and fences late traversal');
  const query={...first,observed_at_epoch_seconds:100};const decode=value=>vm.runInNewContext(`decodeOntologyRelationships(${JSON.stringify(value)},${JSON.stringify(query)},${JSON.stringify(nativeLinks[0])})`,context);
  const valid={...graph,observed_at_epoch_seconds:100};assert.doesNotThrow(()=>decode(valid));assert.doesNotThrow(()=>decode({...valid,truncated:true}));
  const malformed=[null,{...valid,extra:true},{...valid,nodes:[...valid.nodes,valid.nodes[0]]},{...valid,edges:[...valid.edges,valid.edges[0]]},{...valid,nodes:[...valid.nodes,{entity_id:'ent_orphan',entity_type_id:'ety_profile',written_revision:1}]},{...valid,nodes:[valid.nodes[0]]},{...valid,nodes:[{...valid.nodes[0],entity_type_id:'ety_profile'},valid.nodes[1]]},{...valid,nodes:[valid.nodes[0],{...valid.nodes[1],written_revision:4294967296}]},{...valid,edges:[{...valid.edges[0],edge_type_id:'lty_other'}]},{...valid,edges:[{...valid.edges[0],from_entity_id:'ent_profile',to_entity_id:'ent_alpha'}]},{...valid,observed_at_epoch_seconds:101},{...valid,truncated:'true'},{...valid,nodes:Array(1001).fill(valid.nodes[0])},{...valid,edges:Array(5001).fill(valid.edges[0])}];
  for(const value of malformed)assert.throws(()=>decode(value),/source_unavailable/,'malformed graph is refused before rendering');
  const target=id=>({...valid,nodes:[valid.nodes[0],{...valid.nodes[1],entity_id:id}],edges:[{...valid.edges[0],to_entity_id:id}]});
  for(const id of ['ent_profile\uFEFF','ent_'+'가'.repeat(84)])assert.doesNotThrow(()=>decode(target(id)),'native-legal FEFF and exact UTF8 byte bound remain readable');
  for(const id of ['ent_profile\u0085','ent_profile\u00A0','ent_profile\u2003','ent_pro\u0000file','ent_pro\u007Ffile','ent_pro\u009Ffile','ent_'+'가'.repeat(85)])assert.throws(()=>decode(target(id)),/source_unavailable/,'native whitespace, controls and UTF8 overflow remain refused');
  assert.ok(calls.filter(([url])=>url.endsWith('/search-around')).every(([,options])=>options.credentials==='same-origin'&&options.cache==='no-store'&&options.redirect==='error'));
  console.log('Native human relationship selection, own target revision, lifecycle, bounded decoder and focus contracts PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;

#[test]
fn human_relationship_explorer_preserves_context_and_refuses_stale_or_malformed_graphs() {
    check_controller(CHECK);
}
