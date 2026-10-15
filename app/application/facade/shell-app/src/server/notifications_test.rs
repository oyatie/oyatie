use leptos::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

#[path = "notifications_test_dom.rs"]
mod dom;

const CHECK: &str = r##"assert.equal(panel().tagName,'DIALOG');assert.equal(panel().open,false);assert.equal(panel().getAttribute('closedby'),'any');
assert.equal(panel().getAttribute('aria-labelledby'),find('#notification-title').id);
assert.equal(panel().getAttribute('aria-describedby'),'notification-notice notification-status');
const invoker=find('[data-header-action="notifications"]'), close=panel().querySelector('[data-utility-close]');
for(const node of [invoker,close])assert.equal(node.getAttribute('commandfor'),panel().id);
assert.equal(invoker.getAttribute('command'),'show-modal');assert.equal(close.getAttribute('command'),'close');
assert.equal(close.hasAttribute('autofocus'),true);assert.equal(find('[data-utility-backdrop]'),null);
assert.deepEqual(counts(),{unread:'3',total:'4',blocking:'2'});
assert.match(panel().textContent,/example receipt, not a sealed or persisted/);
assert.ok(panel().querySelectorAll('[data-activity-action], [data-activity-filter], [data-notification-route]').every(node=>node.disabled));
assert.ok(panel().querySelectorAll('button').every(node=>!node.getClientRects().length),'closed native dialog has no visible controls in fixture');
vm.runInNewContext(input.tabs,context,{timeout:1000});vm.runInNewContext(input.bridge,context,{timeout:1000});
assert.deepEqual(counts(),{unread:'3',total:'4',blocking:'2'});
click(read('insurance-delta'));assert.equal(counts().unread,'3','closed programmatic action refused');
open();click(filter('unread'));read('insurance-delta').focus();click(read('insurance-delta'));
assert.deepEqual(shown(),['withholding-brief','vendor-renewal']);assert.equal(document.activeElement,filter('unread'),'hidden or disabled local control yields to current filter');
assert.equal(find('[data-activity-badge]').textContent,'2');assert.equal(counts().blocking,'2','read status does not change severity');
click(action('clear-read'));assert.equal(counts().total,'2');assert.equal(counts().blocking,'1');assert.equal(action('clear-read').disabled,true);
// Same valid dashboard remount retains the newest filter/read/clear choices, without focus restoration.
const signIn=find('#oyatie-sign-in'), fresh=initialDashboard.cloneNode(true);document.activeElement=signIn;
find('[data-island="render-envelope-dashboard"]').replaceWith(fresh);flush();
assert.equal(document.activeElement,signIn);assert.equal(filter('unread').getAttribute('aria-pressed'),'true');assert.equal(counts().total,'2');
// A busy, missing, mismatched, failed or duplicate context cannot authorize sample actions.
function unavailable(change,restore) {const before=location.href;change();flush();assert.equal(counts().unread,'—');
  assert.ok(panel().querySelectorAll('[data-activity-action], [data-activity-filter], [data-notification-route]').every(node=>node.disabled));
  click(read('withholding-brief'));click(route('evidence'));assert.equal(location.href,before);restore();flush();assert.equal(counts().total,'2');}
const dashboard=()=>find('[data-island="render-envelope-dashboard"]');
unavailable(()=>dashboard().setAttribute('aria-busy','true'),()=>dashboard().setAttribute('aria-busy','false'));
unavailable(()=>picker().removeAttribute('data-sample-context-displayed'),()=>picker().setAttribute('data-sample-context-displayed','tenant-admin'));
unavailable(()=>picker().setAttribute('data-sample-context-requested','corporate-office'),()=>picker().setAttribute('data-sample-context-requested','tenant-admin'));
unavailable(()=>dashboard().removeAttribute('data-fragment-navigation'),()=>dashboard().setAttribute('data-fragment-navigation','ssr'));
const duplicatePicker=picker().cloneNode(true);unavailable(()=>dashboard().append(duplicatePicker),()=>duplicatePicker.remove());
const failure=new Element('p',{class:'fetch-error'});unavailable(()=>dashboard().append(failure),()=>failure.remove());
// Changed displayed sample resets the bounded record choices instead of borrowing another context's state.
picker().setAttribute('data-sample-context-requested','corporate-office');picker().setAttribute('data-sample-context-displayed','corporate-office');flush();
assert.deepEqual(counts(),{unread:'3',total:'4',blocking:'2'});assert.equal(filter('all').getAttribute('aria-pressed'),'true');
click(filter('blocking'));click(read('vendor-renewal'));assert.equal(counts().unread,'2');
const stalePanel=panel(), replacement=panel().cloneNode(true), staleControl=stalePanel.querySelector('[data-activity-action="mark-read"]');
stalePanel.replaceWith(replacement);flush();assert.equal(stalePanel.events.click.length,0,'prior root listener detached');assert.equal(staleControl.disabled,true);
click(staleControl);assert.equal(counts().unread,'2');assert.equal(filter('blocking').getAttribute('aria-pressed'),'true');
const previous=window.oyatieSampleNotifications;vm.runInNewContext(input.bridge,context,{timeout:1000});flush();
assert.notEqual(window.oyatieSampleNotifications,previous);assert.equal(observers.filter(observer=>!observer.disconnected).length,1);
assert.equal(panel().events.click.length,1,'takeover has one active root listener');assert.equal(windowEvents.blur.length,1);
assert.equal(events.pointerdown.length,1);assert.equal(filter('blocking').getAttribute('aria-pressed'),'true');assert.equal(counts().unread,'2');
// Missing native source, changed record identity and mutated displayed facts fail closed.
const source=panel().getAttribute('data-notification-source');
function invalid(change,restore) {change();flush();assert.equal(counts().unread,'—');assert.equal(action('clear-read').disabled,true);restore();flush();assert.equal(counts().unread,'2');}
invalid(()=>panel().removeAttribute('data-notification-source'),()=>panel().setAttribute('data-notification-source',source));
invalid(()=>panel().setAttribute('data-notification-source','[]'),()=>panel().setAttribute('data-notification-source',source));
const row=panel().querySelector('[data-activity-item]'), rowID=row.dataset.activityItem;
invalid(()=>row.setAttribute('data-activity-item','changed-record'),()=>row.setAttribute('data-activity-item',rowID));
const detail=row.querySelector('[data-notification-value="detail"]'), detailText=detail.textContent;
invalid(()=>detail.textContent='changed native sample',()=>detail.textContent=detailText);
const duplicate=panel().cloneNode(true);invalid(()=>panel().parentElement.append(duplicate),()=>duplicate.remove());
// Filter-empty and fully-cleared states are distinct and counted from retained record IDs.
open();click(filter('all'));for(const id of ['insurance-delta','withholding-brief'])click(read(id));click(filter('unread'));
assert.deepEqual(shown(),[]);assert.equal(find('[data-activity-empty]').hidden,false);assert.match(find('[data-activity-empty]').textContent,/match this filter/);
click(action('clear-read'));assert.deepEqual(counts(),{unread:'0',total:'0',blocking:'0'});assert.match(find('[data-activity-empty]').textContent,/No sample notifications remain/);
// Restore an actual SSR sample for native tab routes and their late refusal checks.
dashboard().replaceWith(initialDashboard.cloneNode(true));flush();open();assert.equal(counts().unread,'3');
for(const name of ['work-hub','evidence','cloud','audit']) {open();click(route(name));flush();
  const target=find('#'+({'work-hub':'work-hub',evidence:'evidence-spine',cloud:'cockpit-panel-topology',audit:'resource-panel-audit'})[name]);
  assert.equal(location.hash,'#'+target.id,name+' opens existing fragment');assert.equal(document.activeElement,target);assert.ok(target.getClientRects().length);assert.equal(panel().open,false);}
const audit=find('#resource-audit-console'), evidence=find('#evidence-spine');
for(const root of [evidence,audit])for(const field of ['data-ae-source-available','data-ae-available','data-ae-source','data-ae-context','data-ae-label','data-ae-mode','data-ae-hub-source'])for(const name of ['evidence','audit']) {
  const before=root.getAttribute(field), url=location.href;root.removeAttribute(field);open();click(route(name));flush();assert.equal(location.href,url);assert.equal(panel().open,true);root.setAttribute(field,before);flush();
}
for(const target of [evidence,find('#resource-panel-audit')])for(const field of ['hidden','inert','aria-hidden']) {
  const url=location.href;target.setAttribute(field,field==='aria-hidden'?'true':'');open();click(route(target===evidence?'evidence':'audit'));flush();assert.equal(location.href,url);assert.equal(panel().open,true);target.removeAttribute(field);flush();
}
for(const name of ['evidence','audit']) {
  const url=location.href;open();click(route(name));for(const callback of frames.splice(0))callback();
  audit.setAttribute('data-ae-available','false');flush();assert.equal(location.href,url,'late linked source refusal');audit.setAttribute('data-ae-available','true');flush();
}
for(const newer of ['pointerdown','keydown','focusin']) {
  const url=location.href;open();click(route('evidence'));document.activeElement=signIn;emit(newer,{target:signIn,isTrusted:true});flush();assert.equal(location.href,url);assert.equal(document.activeElement,signIn,'newer intent owns focus');
}
for(const newer of ['blur','popstate','hashchange']) {const url=location.href;open();click(route('evidence'));windowEvents[newer].forEach(callback=>callback({isTrusted:true}));flush();assert.equal(location.href,url);}
const url=location.href;open();click(route('evidence'));dashboard().setAttribute('aria-busy','true');flush();assert.equal(location.href,url);dashboard().setAttribute('aria-busy','false');flush();
// Replacing or taking over the notification consumer cancels a retained route callback.
open();click(route('evidence'));const old=panel();old.replaceWith(old.cloneNode(true));flush();assert.equal(location.href,url);assert.equal(old.events.click.length,0);
open();click(route('evidence'));vm.runInNewContext(input.bridge,context,{timeout:1000});flush();assert.equal(location.href,url);
open();click(read('insurance-delta'));assert.equal(counts().unread,'2');dashboard().setAttribute('aria-busy','true');
vm.runInNewContext(input.bridge,context,{timeout:1000});flush();assert.equal(counts().unread,'—');
dashboard().setAttribute('aria-busy','false');flush();assert.equal(counts().unread,'3','unavailable takeover refuses inherited choices');
console.log('Notification native source, counts, context/refusal, local lifecycle, listener takeover and route focus cancellation PASS');
"##;

#[test]
fn notifications_native_source_lifecycle_and_refusal() {
    check_bridge(CHECK);
}

fn check_bridge(check: &str) {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let input = serde_json::json!({"html": html, "tabs": crate::app::ssr_tab_navigation_script(), "bridge": super::script()});
    let check = format!("{}{check}", dom::SCRIPT);
    let mut child = Command::new("node")
        .args(["-e", check.as_str()])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs notification source proof");
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

#[test]
fn notifications_shared_fallback_uses_native_dialog_and_record_counts() {
    let html = crate::app::static_dashboard_html();
    assert!(html.contains("<dialog id=\"notification-center\""));
    assert!(!html.contains("data-utility-backdrop"));
    assert!(html.contains("data-activity-count=\"total\">4"));
    assert!(html.contains("data-activity-count=\"blocking\">2"));
    assert!(html.contains("commandfor=\"notification-center\" command=\"show-modal\""));
    assert!(html.contains("commandfor=\"notification-center\" command=\"close\""));
}

#[test]
fn notifications_reveal_only_current_managed_frame_destinations() {
    check_bridge(
        r##"
vm.runInNewContext(input.tabs,context,{timeout:1000});vm.runInNewContext(input.bridge,context,{timeout:1000});
const frameHost=find('#dashboard-island-root'),frameEvents=[];
document.addEventListener('oyatie-shell-destination',event=>{frameEvents.push(event.target.id);frameHost.removeAttribute('data-shell-inactive');});
frameHost.setAttribute('data-shell-inactive','');const canceledFrameURL=location.href;open();click(route('evidence'));emit('keydown',{key:'Escape',isTrusted:true,target:tree});flush();assert.equal(frameHost.hasAttribute('data-shell-inactive'),true,'canceled preview route does not reveal inactive frame');assert.equal(location.href,canceledFrameURL);assert.equal(frameEvents.length,0);
frameHost.setAttribute('data-shell-inactive','');open();click(route('evidence'));flush();
assert.deepEqual(frameEvents,['evidence-spine'],'validated managed destination requests one synchronous display handoff');assert.equal(frameHost.hasAttribute('data-shell-inactive'),false);assert.equal(document.activeElement,find('#evidence-spine'));assert.ok(document.activeElement.getClientRects().length);
frameHost.setAttribute('data-shell-inactive','');const frameURL=location.href;find('#evidence-spine').setAttribute('hidden','');open();click(route('evidence'));flush();assert.equal(frameEvents.length,1,'native hidden destination refuses before frame reveal');assert.equal(location.href,frameURL);assert.equal(frameHost.hasAttribute('data-shell-inactive'),true);find('#evidence-spine').removeAttribute('hidden');
const frameSource=find('#evidence-spine').getAttribute('data-ae-source');find('#evidence-spine').setAttribute('data-ae-source','[]');open();click(route('evidence'));flush();assert.equal(frameEvents.length,1,'forged source cannot reveal an inactive frame');assert.equal(frameHost.hasAttribute('data-shell-inactive'),true);find('#evidence-spine').setAttribute('data-ae-source',frameSource);frameHost.removeAttribute('data-shell-inactive');
console.log('Notifications managed frame reveal, cancellation, native hiding and source refusal PASS');
"##,
    );
}
