#[path = "explorer_test_dom.rs"]
mod dom;
use leptos::prelude::*;
use std::{
    io::Write,
    process::{Command, Stdio},
};
const CHECK: &str = r#"
(async () => {
  for(const id of ['type','revision','limit','property','comparison','kind','from','to']) {
    assert.ok(input.html.includes(`id="live-ontology-${id}"`));
    assert.ok(input.html.includes(`for="live-ontology-${id}"`), 'native input has a label');
  }
  assert.ok(input.html.includes('id="live-ontology-detail-title" tabindex="-1"'));
  assert.ok(input.html.includes('id="live-ontology-query"'));
  assert.ok(input.html.includes('id="live-ontology-schema" aria-labelledby="live-ontology-schema-title"'));
  assert.ok(input.html.includes('Requested object revisions are set separately.'));
  assert.ok(input.html.includes('Native Ontology explorer'));
  assert.ok(input.html.includes('Sample dashboard'));
  await flush();find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.equal(root.hidden,false);assert.match(find('live-ontology-identity').textContent,/ten_acme.*usr_alice/);
  assert.equal(find('live-ontology-schema').hidden,false);
  assert.match(find('live-ontology-schema-summary').textContent,/ety_record.*declaration revision 1.*Primary-key property: none declared.*Title property: name/);
  assert.match(text(find('live-ontology-schema-properties')),/name scalar Legacy string contract Required INTERNAL_ONLY/);
  assert.match(text(find('live-ontology-schema-properties')),/note scalar Legacy string contract Optional INTERNAL_ONLY/);
  find('live-ontology-type').focus();find('live-ontology-type').value='ety_profile';find('live-ontology-type').emit('input');
  assert.equal(document.activeElement,find('live-ontology-type'),'schema selection does not move keyboard focus');
  assert.match(find('live-ontology-schema-summary').textContent,/ety_profile.*revision 3.*Primary-key property: id.*Title property: <script>label<\/script>/);
  assert.match(text(find('live-ontology-schema-properties')),/<script>label<\/script> scalar string Optional PII_IDENTIFYING/,'declaration names are literal text');
  assert.match(text(find('live-ontology-schema-properties')),/struct \{ flags \(required\): array of boolean \}/,'nested declared shape and field requirement remain visible');
  find('live-ontology-type').value='ety_record';find('live-ontology-type').emit('input');
  find('live-ontology-query').emit('submit');await flush();
  assert.match(text(find('live-ontology-page')),/ent_alpha/);
  let row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();row.emit('click');await flush();
  assert.equal(document.activeElement,find('live-ontology-detail-title'),'native object activation focuses its heading');
  assert.match(text(find('live-ontology-properties')),/<script>Ada<\/script>/,'property content is literal text');
  find('live-ontology-history-read').emit('click');await flush();assert.match(text(find('live-ontology-history')),/record.written/);
  find('live-ontology-audit-read').emit('click');await flush();assert.match(text(find('live-ontology-audit-rows')),/applied/);
  find('live-ontology-revision').value='2';find('live-ontology-revision').emit('input');
  assert.equal(find('live-ontology-page').children.length,0,'changed pin clears prior rows synchronously');
  assert.match(find('live-ontology-schema-summary').textContent,/declaration revision 1/,'current declaration is independent of requested object revision');
  find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();row.emit('click');await flush();
  assert.match(find('live-ontology-detail-title').textContent,/requested schema 2.*stored schema 1/);
  row.focus();intervals[0]();await flush();
  assert.equal(find('live-ontology-revision').value,'2','same-identity periodic refresh preserves requested pin');
  assert.equal(find('live-ontology-type').value,'ety_record');
  assert.equal(document.activeElement.dataset.liveObject,'ent_alpha','same-identity refresh preserves object-button focus');
  assert.equal(document.activeElement.focusOptions?.preventScroll,true,'matching-row restoration preserves scroll position');
  assert.match(find('live-ontology-detail-title').textContent,/ent_alpha.*requested schema 2/,'selected object stays usable');
  row=document.activeElement;
  const removalCheck=new Element(), removedButton=new Element('button');removalCheck.append(removedButton);removedButton.focus();removalCheck.replaceChildren();
  assert.equal(document.activeElement,document.body,'removed focused subtree resets browser focus to body');
  row.focus();pageEmpty=true;intervals[0]();await flush();
  assert.equal(find('live-ontology-page').querySelectorAll('[data-live-object]').length,0,'native disappearance is an empty page');
  assert.equal(document.activeElement,find('live-ontology-type'),'disappearing focused row returns to the enabled type selector');
  assert.equal(document.activeElement.disabled,false);
  assert.equal(document.activeElement.focusOptions?.preventScroll,true,'fallback preserves scroll position');
  pageEmpty=false;find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();pageEmpty=true;holdPath='/objects?';intervals[0]();await flush();
  find('live-ontology-type').disabled=true;held();await flush();
  assert.equal(document.activeElement,find('live-ontology-read'),'disabled type selector falls back to the enabled read control');
  assert.equal(document.activeElement.focusOptions?.preventScroll,true);find('live-ontology-type').disabled=false;
  pageEmpty=false;find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();pageEmpty=true;holdPath='/objects?';intervals[0]();await flush();
  find('sign-in-account').focus();held();await flush();
  assert.equal(document.activeElement,find('sign-in-account'),'delayed disappearance does not steal newer outside focus');
  pageEmpty=false;find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();row.emit('click');await flush();
  row.focus();holdPath='/objects/ent_alpha?';row.emit('click');await flush();
  find('sign-in-account').focus();held();await flush();assert.equal(document.activeElement,find('sign-in-account'),'late detail never steals newer focus');
  holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('sign-in-account').value='usr_other';find('sign-in-account').emit('input');
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-page').children.length,0);held();await flush();
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-schema-properties').children.length,0,'account change clears declarations');
  assert.equal(root.hidden,true,'late previous-account rows stay absent');assert.equal(document.activeElement,find('sign-in-account'));
  find('sign-in-account').value='usr_alice';await restore();
  holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('live-ontology-revision').value='3';find('live-ontology-revision').emit('input');held();await flush();
  assert.equal(find('live-ontology-page').children.length,0,'late previous-pin rows stay absent');
  nativeError='forbidden';find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/access refused/);assert.equal(find('live-ontology-page').children.length,0);
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-schema-summary').textContent,'');
  nativeError=null;await restore();assert.equal(find('live-ontology-schema').hidden,false);
  nativeError='source_unavailable';find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/unavailable/);assert.equal(find('live-ontology-type').children.length,0,'unavailable read does not retain native type rows');
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-schema-properties').children.length,0);
  nativeError=null;await restore();nativeError='source_unavailable';holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('live-ontology-schema').focus();held();await flush();
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(document.activeElement,find('live-ontology-types-refresh'),'failed object read retains schema focus ownership through clearing and native disable blur');assert.equal(document.activeElement.disabled,false);
  nativeError=null;await restore();holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('session-logout').emit('click');assert.equal(root.hidden,true,'logout clears synchronously');held();await flush();
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-page').children.length,0,'late logout reply cannot restore data');
  assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-schema-properties').children.length,0,'logout clears declarations');
  await restore();headerUser='usr_other';find('live-ontology-query').emit('submit');await flush();
  assert.equal(root.hidden,true,'cookie identity from another tab cannot retain old labels/data');
  assert.equal(find('live-ontology-schema').hidden,true);
  headerUser='usr_alice';await restore();find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();timers.at(-1)();
  assert.equal(root.hidden,true);assert.equal(document.activeElement,find('sign-in-account'),'expiry moves focus to the stable account input');
  assert.equal(find('live-ontology-schema').hidden,true);
  await restore();find('live-ontology-schema').focus();holdPath='/schemas';find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(document.activeElement,find('live-ontology-type'),'clearing a focused schema moves to the enabled type control');assert.equal(document.activeElement.disabled,false);
  find('sign-in-account').focus();held();await flush();assert.equal(document.activeElement,find('sign-in-account'),'late schema refresh does not steal newer outside focus');
  find('live-ontology-schema').focus();holdPath='/schemas';intervals[0]();
  assert.equal(find('live-ontology-schema').hidden,true,'session revalidation clears declarations synchronously');
  assert.equal(document.activeElement,find('live-ontology-type'),'session hold removes focus from the hidden schema region');assert.equal(document.activeElement.disabled,false);
  await flush();assert.equal(find('live-ontology-schema').hidden,true,'schema refresh never retains stale declarations while pending');
  find('sign-in-tenant').value='ten_other';find('sign-in-tenant').emit('input');held();await flush();
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-schema-properties').children.length,0,'late previous-tenant schema cannot repopulate');
  find('sign-in-tenant').value='ten_acme';await restore();holdPath='/schemas';find('live-ontology-types-refresh').emit('click');await flush();
  find('session-logout').emit('click');held();await flush();assert.equal(root.hidden,true);assert.equal(find('live-ontology-schema-summary').textContent,'','late schemas after logout stay cleared');
  await restore();headerUser='usr_other';find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(root.hidden,true,'schema response bound to another account expires current view');headerUser='usr_alice';
  await restore();nativeError='unauthenticated';find('live-ontology-types-refresh').emit('click');await flush();
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-schema-properties').children.length,0,'revoked schema read clears native declarations');nativeError=null;
  for (const malformed of [null, [{entity_type:'ety_record',revision:1,properties:['name']}], [{...schemaFixture[0],primary_key_property:'absent'}]]) {
    nativeSchemas=schemaFixture;await restore();assert.equal(find('live-ontology-schema').hidden,false);
    nativeSchemas=malformed;find('live-ontology-types-refresh').emit('click');await flush();
    assert.match(find('live-ontology-status').textContent,/unavailable/);assert.equal(find('live-ontology-type').children.length,0);
    assert.equal(find('live-ontology-schema').hidden,true);assert.equal(find('live-ontology-schema-summary').textContent,'');assert.equal(find('live-ontology-schema-properties').children.length,0,'malformed schemas clear all declarations');
  }
  nativeSchemas=schemaFixture;find('live-ontology-types-refresh').emit('click');await flush();find('live-ontology-schema').focus();
  nativeSchemas=[];find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/No entity types/);assert.equal(find('live-ontology-read').disabled,true);assert.equal(find('live-ontology-schema').hidden,true);
  assert.equal(document.activeElement,find('live-ontology-types-refresh'),'empty schemas settle focus on the surviving refresh control');assert.equal(document.activeElement.disabled,false);assert.equal(document.activeElement.focusOptions?.preventScroll,true);
  nativeSchemas=schemaFixture;find('live-ontology-types-refresh').emit('click');await flush();find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();holdPath='/schemas';find('live-ontology-types-refresh').emit('click');await flush();nativeSchemas=[];held();await flush();
  assert.equal(find('live-ontology-page').children.length,0);assert.equal(document.activeElement,find('live-ontology-types-refresh'),'empty schema registry clears focused object rows and settles on enabled refresh');assert.equal(document.activeElement.disabled,false);
  nativeSchemas=schemaFixture;find('live-ontology-types-refresh').emit('click');await flush();find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();holdPath='/schemas';find('live-ontology-types-refresh').emit('click');await flush();find('sign-in-account').focus();nativeSchemas=[];held();await flush();
  assert.equal(find('live-ontology-page').children.length,0);assert.equal(document.activeElement,find('sign-in-account'),'late empty schemas do not steal newer outside focus');
  nativeSchemas=schemaFixture;find('live-ontology-types-refresh').emit('click');await flush();assert.equal(find('live-ontology-schema').hidden,false);
  const decodeSchemas = rows => vm.runInNewContext(`ontologyDecode.schemas(${JSON.stringify(rows)})`,context);
  const changed = change => { const schema=JSON.parse(JSON.stringify(schemaFixture[0]));change(schema);return [schema]; };
  assert.doesNotThrow(()=>decodeSchemas(changed(s=>s.revision=0)),'native declaration revision zero remains inspectable');
  assert.doesNotThrow(()=>decodeSchemas(changed(s=>{s.properties[0].name='\uFEFF';s.title_property='\uFEFF';})),'native Unicode non-whitespace name is not trimmed away');
  assert.doesNotThrow(()=>decodeSchemas(changed(s=>{s.properties[0].tier='struct';s.properties[0].value_type={kind:'struct',fields:[{name:'\uFEFF',required:true,value_type:{kind:'string'}}]};})),'nested native Unicode non-whitespace name remains admitted');
  for(const tier of ['scalar','vector','timeseries','geo','ciphertext','struct'])assert.doesNotThrow(()=>decodeSchemas(changed(s=>s.properties[0].tier=tier)),'legacy null declaration retains every native tier');
  for(const kind of ['string','integer','double','boolean','date','timestamp'])assert.doesNotThrow(()=>decodeSchemas(changed(s=>s.properties[0].value_type={kind})));
  for(const label of ['INTERNAL_ONLY','PHI','PII_IDENTIFYING','PII_QUASI_IDENTIFIER','PCI','FINANCIAL','FINANCIAL_REGULATED_CREDIT','BEHAVIORAL_TENANT_PRODUCT','BEHAVIORAL_ADS','DECLARED_PREFERENCE','SEARCH_QUERY','PUBLIC','SENSITIVE_PIPA_ART23'])assert.doesNotThrow(()=>decodeSchemas(changed(s=>s.properties[0].data_class=label)));
  let depth8={kind:'string'};for(let depth=1;depth<8;depth++)depth8={kind:'array',element:depth8};
  assert.doesNotThrow(()=>decodeSchemas(changed(s=>{s.properties[0].tier='vector';s.properties[0].value_type=depth8;})),'native root-depth-one boundary admits depth eight');
  const invalidSchemas=[null,false,[schemaFixture[0],schemaFixture[0]],...[
    s=>delete s.primary_key_property,s=>delete s.title_property,s=>delete s.properties[0].value_type,
    s=>s.extra='unknown',s=>s.properties[0].extra='unknown',s=>s.properties[0].required='true',
    s=>s.properties.push(s.properties[0]),s=>s.properties[0].name=' ',s=>s.properties=[],
    s=>{s.properties[0].name='\u0085';s.title_property='\u0085';},
    s=>s.primary_key_property='note',s=>s.title_property='absent',s=>s.revision=-1,s=>s.revision=4294967296,
    ...['AUDIT','SECRET','CHILDREN','invented'].map(label=>s=>s.properties[0].data_class=label),
    s=>s.properties[0].value_type={kind:'string',extra:'unknown'},
    s=>{s.properties[0].tier='vector';s.properties[0].value_type={kind:'boolean'};},
    s=>{s.properties[0].tier='vector';s.properties[0].value_type={kind:'array',element:depth8};},
    ...[[],[{name:'x',required:true,value_type:{kind:'string'}},{name:'x',required:false,value_type:{kind:'integer'}}],[{name:' ',required:true,value_type:{kind:'boolean'}}],[{name:'\u0085',required:true,value_type:{kind:'boolean'}}],[{name:'x',required:true}]].map(fields=>s=>{s.properties[0].tier='struct';s.properties[0].value_type={kind:'struct',fields};}),
  ].map(changed)];
  for(const rows of invalidSchemas)assert.throws(()=>decodeSchemas(rows),/source_unavailable/,'malformed declaration metadata is never rendered');
  for(const expression of [
    'ontologyDecode.page({requested_revision:2,objects:[],next:null},1,25)',
    'ontologyDecode.detail({requested_revision:1,object:{object_ref:"ent_other",written_revision:1,upcast_state:"current",properties:{}}},"ent_alpha",1)',
    'ontologyDecode.history([{ordinal:1,object_sequence:1,principal_id:"p",decision_id:"d",action_type:"a",audit_event_type:"e"}])',
    'ontologyDecode.audit([{ordinal:1,object_ref:"ent_alpha",action_type:"a",disposition:"invented",poison_reason:null,principal_id:null,decision_id:null}])',
  ]) assert.throws(()=>vm.runInNewContext(expression,context),/source_unavailable/);
  assert.ok(calls.filter(([url])=>url.includes('/api/ontology/')).every(([,options])=>options.credentials==='same-origin'&&options.cache==='no-store'&&options.redirect==='error'));
  assert.ok(calls.some(([url])=>url.endsWith('/schemas')));assert.ok(!calls.some(([url])=>url.endsWith('/types')),'schema inspection uses the separate compatible reader');
  console.log('Native explorer schema, identity, pin, refresh, late-response, focus, logout and decoder contracts PASS');
})().catch(error=>{console.error(error);process.exitCode=1;});
"#;
#[test]
fn native_explorer_controller_preserves_identity_and_refuses_stale_rows_or_focus() {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let input = serde_json::json!({"html":html,"script":super::script()});
    let check = format!("{}{CHECK}", dom::SCRIPT);
    let mut child = Command::new("node")
        .args(["-e", &check])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs the native controller contract");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
