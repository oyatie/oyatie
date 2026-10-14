use super::explorer_test::check_controller;

#[test]
fn shared_shell_tracks_verified_identity_and_destination_without_granting_access() {
    check_controller(
        r##"
(async()=>{
  await flush();
  for(const id of ['shell-account-title','shell-account-detail','shell-session-entry','shell-breadcrumb-scope','shell-breadcrumb-destination'])assert.ok(input.html.includes(`id="${id}"`));
  assert.equal(find('shell-session-entry').textContent,'Sign in');
  context.location.hash='#foundry-workspace';context.window.emit('hashchange');
  assert.equal(find('shell-breadcrumb-scope').textContent,'Foundry');assert.equal(find('shell-breadcrumb-destination').textContent,'Ontology');
  assert.equal(find('console-shell').attrs['aria-labelledby'],'foundry-workspace-title','main follows the actual workspace heading');
  find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';
  modelEnabled=true;nativeError='forbidden';statusError=403;session.roles=['foundry-model-author'];
  holdPath='/session';find('session-restore').emit('click');await flush();
  assert.equal(find('shell-session-entry').textContent,'Checking session');assert.doesNotMatch(find('shell-account-detail').textContent,/usr_alice/);
  held();await flush();
  assert.equal(find('shell-session-entry').textContent,'Account & session');
  assert.equal(find('shell-account-title').textContent,'ten_acme');assert.equal(find('shell-account-detail').textContent,'usr_alice');
  assert.match(find('shell-rail-status').textContent,/Session verified/);
  assert.equal(find('verified-ontology-card').hidden,true);assert.equal(find('live-ontology-type').children.length,0,'identity does not grant status/object access');
  assert.equal(find('live-ontology-draft-controls').hidden,false,'model-only access remains independent');
  assert.doesNotMatch(find('shell-rail-title').textContent,/Sample|Anonymous/);
  context.location.hash='#service-catalog';context.window.emit('hashchange');
  assert.equal(find('shell-breadcrumb-scope').textContent,'Sample views');assert.equal(find('shell-breadcrumb-destination').textContent,'Catalog samples');
  assert.equal(find('console-shell').attrs['aria-labelledby'],'console-title','sample navigation preserves its truthful main landmark');
  assert.equal(find('shell-account-detail').textContent,'usr_alice','sample navigation never changes the authenticated account');
  context.location.hash='#foundry-workspace';context.window.emit('hashchange');
  holdPath='/session';find('session-restore').emit('click');await flush();assert.match(find('shell-rail-status').textContent,/Checking session/);assert.match(find('live-ontology-identity').textContent,/Checking tenant ten_acme.*usr_alice/);
  held();await flush();assert.match(find('shell-rail-status').textContent,/Session verified/);
  find('session-logout').emit('click');assert.equal(find('shell-session-entry').textContent,'Sign in');assert.doesNotMatch(find('shell-account-detail').textContent,/usr_alice/);await flush();
  holdPath='/session';find('session-restore').emit('click');await flush();find('sign-in-account').value='usr_bob';find('sign-in-account').emit('input');held();await flush();
  assert.equal(find('shell-session-entry').textContent,'Sign in');assert.doesNotMatch(find('shell-account-detail').textContent,/usr_alice|usr_bob/,'stale identity is never presented as verified');
  find('sign-in-account').value='usr_alice';await restore();timers.at(-1)();assert.equal(find('shell-session-entry').textContent,'Sign in');assert.equal(root.hidden,true,'expiry clears both shared and product context');
  console.log('Shared verified identity, checking, destination, preview separation, logout, stale response and expiry PASS');
})().catch(e=>{console.error(e);process.exitCode=1;});
"##,
    );
}

#[test]
fn saved_model_selection_restores_only_the_current_keyboard_focus_owner() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';
  const first='9007199254740993',second='9007199254740994';
  for(const id of [first,second])modelDrafts.set(id,{id,tenant_id:session.tenantId,owner:session.userId,base_generation:modelWorkspace.generation,version:'1',document:structuredClone(modelDocument),publication:null});
  await restore();const choice=find('live-ontology-draft-choice');
  async function start(id){choice.focus();choice.value=id;holdPath=`/drafts/${id}`;choice.emit('change');await flush();assert.equal(choice.disabled,true);}
  await start(first);assert.ok(document.activeElement===document.body,'browser disable blur is modeled');held();await flush();
  assert.ok(document.activeElement===choice,'opening a saved model restores its initiating selector');assert.equal(choice.disabled,false);assert.equal(choice.focusOptions.preventScroll,true);
  await start(second);find('live-ontology-search').focus();held();await flush();assert.ok(document.activeElement===find('live-ontology-search'),'new focus owns navigation');
  await start(first);find('live-ontology-mode-objects').focus();find('live-ontology-mode-objects').emit('click');held();await flush();assert.ok(document.activeElement===find('live-ontology-mode-objects'),'mode change cancels restoration');
  find('live-ontology-mode-schema').emit('click');
  await start(second);context.location.hash='#service-catalog';context.window.emit('hashchange');held();await flush();assert.ok(document.activeElement===document.body,'route change cancels restoration even when it moves no focus');
  context.location.hash='#foundry-workspace';context.window.emit('hashchange');
  await start(first);document.emit('pointerdown',{target:document.body});held();await flush();assert.ok(document.activeElement===document.body,'clicking a nonfocusable surface cancels restoration');
  await start(second);document.emit('keydown',{key:'Tab'});held();await flush();assert.ok(document.activeElement===document.body,'Tab intent cancels restoration even before another target receives focus');
  await start(first);document.visibilityState='hidden';document.emit('visibilitychange');held();await flush();assert.ok(document.activeElement===document.body,'hidden document cancels restoration');document.visibilityState='visible';
  choice.value=first;holdPath=`/drafts/${first}`;find('sign-in-account').focus();choice.emit('change');await flush();held();await flush();assert.ok(document.activeElement===find('sign-in-account'),'selection without focus never creates a focus owner');
  await start(second);find('session-logout').emit('click');held();await flush();assert.ok(document.activeElement!==choice);assert.equal(root.hidden,true,'revocation invalidates pending restoration');
  await restore();await start(first);modelError='forbidden';held();await flush();
  // The held response is captured before denial; a new denied load still cannot restore focus.
  choice.focus();choice.value=second;choice.emit('change');await flush();assert.ok(document.activeElement!==choice);assert.equal(find('live-ontology-draft-controls').hidden,true);
  console.log('Saved model focus ownership, native blur, focus movement, mode/route change, logout and denied load PASS');
})().catch(e=>{console.error(e);process.exitCode=1;});
"##,
    );
}
