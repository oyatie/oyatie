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
  assert.equal(find('shell-header-identity').textContent,'ten_acme · usr_alice','compact header uses verified IDs without inventing display names');
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
  choice.value=first;holdPath=`/drafts/${first}`;find('shell-session-entry').focus();choice.emit('change');await flush();held();await flush();assert.ok(document.activeElement===find('shell-session-entry'),'selection without focus never creates a focus owner');
  await start(second);find('session-logout').emit('click');held();await flush();assert.ok(document.activeElement!==choice);assert.equal(root.hidden,true,'revocation invalidates pending restoration');
  await restore();await start(first);modelError='forbidden';held();await flush();
  // The held response is captured before denial; a new denied load still cannot restore focus.
  choice.focus();choice.value=second;choice.emit('change');await flush();assert.ok(document.activeElement!==choice);assert.equal(find('live-ontology-draft-controls').hidden,true);
  console.log('Saved model focus ownership, native blur, focus movement, mode/route change, logout and denied load PASS');
})().catch(e=>{console.error(e);process.exitCode=1;});
"##,
    );
}

#[test]
fn shared_frame_keeps_native_destinations_connected_and_skip_in_current_workspace() {
    check_controller(
        r##"
(async()=>{
  await flush();
  const shell=find('oyatie-shell'), intro=find('shell-sample-intro'), dashboard=find('dashboard-island-root'), accountPanel=find('oyatie-sign-in'), workspace=find('foundry-workspace'),skip=find('shell-skip-link');
  const original=[intro,dashboard,accountPanel,workspace];
  function navigate(hash){context.location.hash=hash;context.window.emit('hashchange');}
  navigate('#foundry-workspace');
  assert.equal(skip.getAttribute('href'),'#foundry-workspace','skip stays in the active Foundry workspace');
  for(const node of [intro,dashboard,accountPanel])assert.equal(node.getClientRects().length,0,'inactive frame leaves layout and keyboard traversal');
  assert.equal(workspace.getClientRects().length,1);assert.equal(shell.getAttribute('data-shell-frame'),'foundry');
  find('shell-session-entry').focus();const priorFocus=document.activeElement;find('sign-in-account').focus();assert.ok(document.activeElement===priorFocus,'focus cannot enter the inactive Account form');
  assert.deepEqual([intro,dashboard,accountPanel,workspace],original);assert.ok(original.every(node=>node.isConnected));
  const requests=calls.length;
  navigate('#sign-in-account');assert.equal(find('shell-navigation-menu').open,true,'noncompact Account keeps the full native navigation available');assert.equal(accountPanel.getClientRects().length,1);assert.equal(workspace.getClientRects().length,0);assert.equal(skip.getAttribute('href'),'#oyatie-sign-in');
  navigate('#service-catalog');assert.equal(dashboard.getClientRects().length,1);assert.equal(find('shell-breadcrumb-destination').textContent,'Catalog samples');
  navigate('#live-ontology-inspector-title');assert.equal(workspace.getClientRects().length,1);assert.equal(skip.getAttribute('href'),'#foundry-workspace');
  const before=shell.getAttribute('data-shell-frame'),foreign=new Element();foreign.id='not-owned';foreign.dispatchEvent(new context.Event('oyatie-shell-destination'));assert.equal(shell.getAttribute('data-shell-frame'),before,'unowned destination cannot change frame');
  const native=find('service-catalog');native.isConnected=false;native.dispatchEvent(new context.Event('oyatie-shell-destination'));assert.equal(shell.getAttribute('data-shell-frame'),before,'disconnected destination cannot change frame');native.isConnected=true;
  native.dispatchEvent(new context.Event('oyatie-shell-destination'));assert.equal(shell.getAttribute('data-shell-frame'),'samples','existing focus owner reveals its connected destination synchronously');assert.equal(context.location.hash,'#live-ontology-inspector-title','display handoff does not navigate');
  const menu=find('shell-navigation-menu'), anchor=new Element('a');menu.append(anchor);anchor.setAttribute('href','#service-catalog');
  document.emit('click',{target:anchor,button:0});assert.equal(menu.open,true,'sample route does not close the noncompact rail without an opener');
  anchor.setAttribute('href','#foundry-workspace');document.emit('click',{target:anchor,button:0});assert.equal(shell.getAttribute('data-shell-frame'),'foundry');assert.equal(menu.open,false,'Foundry keeps its native compact All views opener');
  navigate('#%E0%A4%A');assert.equal(shell.getAttribute('data-shell-frame'),'samples','malformed fragment uses truthful sample fallback');
  assert.equal(calls.length,requests,'frame and skip navigation make no service requests');
  console.log('Shared frame native destination,connected nodes,skip and synchronous display handoff PASS');
})().catch(e=>{console.error(e);process.exitCode=1;});
"##,
    );
}

#[test]
fn shared_frame_retains_pending_form_and_uses_visible_owned_session_fallback() {
    check_controller(
        r##"
(async()=>{
  await flush();modelEnabled=true;find('sign-in-tenant').value='ten_acme';find('sign-in-account').value='usr_alice';await restore();
  find('live-ontology-draft-new').emit('click');await flush();
  function navigate(hash){context.location.hash=hash;context.window.emit('hashchange');}
  navigate('#foundry-workspace');
  const pending=find('live-ontology-property-name'), form=find('live-ontology-property-edit'),draft=find('live-ontology-draft-state').textContent;
  pending.value='Pending frame edit';pending.emit('input');pending.focus();
  const requests=calls.length;
  navigate('#sign-in-account');assert.equal(find('oyatie-sign-in').getClientRects().length,1);
  navigate('#live-ontology-inspector-title');
  assert.equal(find('live-ontology-property-name'),pending);assert.equal(pending.parentElement,form);assert.equal(pending.value,'Pending frame edit');assert.ok(pending.isConnected&&form.isConnected);assert.equal(calls.length,requests,'frame navigation does not reload or save a draft');
  assert.match(find('live-ontology-draft-state').textContent,/Unapplied form edits/);assert.ok(draft.includes('saved version 1'));
  pending.focus();timers.at(-1)();assert.equal(root.hidden,true);assert.equal(document.activeElement,find('shell-session-entry'),'owned cleared focus lands on visible Account entry');assert.equal(document.activeElement.getClientRects().length,1);assert.equal(context.location.hash,'#live-ontology-inspector-title','expiry adds no navigation');
  await restore();find('shell-session-entry').focus();timers.at(-1)();assert.equal(document.activeElement,find('shell-session-entry'),'expiry does not steal newer outside focus');
  console.log('Frame pending node/value retention and visible owned expiry fallback PASS');
})().catch(e=>{console.error(e);process.exitCode=1;});
"##,
    );
}
