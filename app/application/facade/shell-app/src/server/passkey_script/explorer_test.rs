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
  assert.ok(input.html.includes('Native Ontology explorer'));
  assert.ok(input.html.includes('Sample dashboard'));
  await flush();find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  assert.equal(root.hidden,false);assert.match(find('live-ontology-identity').textContent,/ten_acme.*usr_alice/);
  find('live-ontology-query').emit('submit');await flush();
  assert.match(text(find('live-ontology-page')),/ent_alpha/);
  let row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();row.emit('click');await flush();
  assert.equal(document.activeElement,find('live-ontology-detail-title'),'native object activation focuses its heading');
  assert.match(text(find('live-ontology-properties')),/<script>Ada<\/script>/,'property content is literal text');
  find('live-ontology-history-read').emit('click');await flush();assert.match(text(find('live-ontology-history')),/record.written/);
  find('live-ontology-audit-read').emit('click');await flush();assert.match(text(find('live-ontology-audit-rows')),/applied/);
  find('live-ontology-revision').value='2';find('live-ontology-revision').emit('input');
  assert.equal(find('live-ontology-page').children.length,0,'changed pin clears prior rows synchronously');
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
  assert.equal(root.hidden,true,'late previous-account rows stay absent');assert.equal(document.activeElement,find('sign-in-account'));
  find('sign-in-account').value='usr_alice';await restore();
  holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('live-ontology-revision').value='3';find('live-ontology-revision').emit('input');held();await flush();
  assert.equal(find('live-ontology-page').children.length,0,'late previous-pin rows stay absent');
  nativeError='forbidden';find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/access refused/);assert.equal(find('live-ontology-page').children.length,0);
  nativeError='source_unavailable';find('live-ontology-types-refresh').emit('click');await flush();
  assert.match(find('live-ontology-status').textContent,/unavailable/);assert.equal(find('live-ontology-type').children.length,0,'unavailable read does not retain native type rows');
  nativeError=null;await restore();holdPath='/objects?';find('live-ontology-query').emit('submit');await flush();
  find('session-logout').emit('click');assert.equal(root.hidden,true,'logout clears synchronously');held();await flush();
  assert.equal(root.hidden,true);assert.equal(find('live-ontology-page').children.length,0,'late logout reply cannot restore data');
  await restore();headerUser='usr_other';find('live-ontology-query').emit('submit');await flush();
  assert.equal(root.hidden,true,'cookie identity from another tab cannot retain old labels/data');
  headerUser='usr_alice';await restore();find('live-ontology-query').emit('submit');await flush();
  row=find('live-ontology-page').querySelectorAll('[data-live-object]')[0];row.focus();timers.at(-1)();
  assert.equal(root.hidden,true);assert.equal(document.activeElement,find('sign-in-account'),'expiry moves focus to the stable account input');
  for(const expression of [
    'ontologyDecode.page({requested_revision:2,objects:[],next:null},1,25)',
    'ontologyDecode.detail({requested_revision:1,object:{object_ref:"ent_other",written_revision:1,upcast_state:"current",properties:{}}},"ent_alpha",1)',
    'ontologyDecode.history([{ordinal:1,object_sequence:1,principal_id:"p",decision_id:"d",action_type:"a",audit_event_type:"e"}])',
    'ontologyDecode.audit([{ordinal:1,object_ref:"ent_alpha",action_type:"a",disposition:"invented",poison_reason:null,principal_id:null,decision_id:null}])',
  ]) assert.throws(()=>vm.runInNewContext(expression,context),/source_unavailable/);
  assert.ok(calls.filter(([url])=>url.includes('/api/ontology/')).every(([,options])=>options.credentials==='same-origin'&&options.cache==='no-store'&&options.redirect==='error'));
  console.log('Native explorer identity, pin, refresh, late-response, focus, logout and decoder contracts PASS');
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
