use super::check_bridge;

#[test]
fn settings_reveal_only_current_managed_frame_destinations() {
    check_bridge(
        r##"
const frameHost=find('#dashboard-island-root'),signin=find('#oyatie-sign-in'),drawer=find('#settings-center'),frameEvents=[];
document.addEventListener('oyatie-shell-destination',event=>{frameEvents.push(event.target.id);frameHost.removeAttribute('data-shell-inactive');signin.removeAttribute('data-shell-inactive');});
function prepare(){frameHost.setAttribute('data-shell-inactive','');drawer.open=true;}
function navigate(){click(find('[data-settings-action="open-identity"]'));}
prepare();const canceledURL=location.href;navigate();emit('keydown',{key:'Escape',isTrusted:true,target:tree});flush();
assert.equal(frameHost.hasAttribute('data-shell-inactive'),true,'canceled Settings route leaves composition unchanged');assert.equal(location.href,canceledURL);assert.equal(frameEvents.length,0);
prepare();navigate();flush();
assert.deepEqual(frameEvents,['identity-panel-auth']);assert.equal(frameHost.hasAttribute('data-shell-inactive'),false);assert.equal(document.activeElement,find('#identity-panel-auth'));assert.ok(document.activeElement.getClientRects().length);
for(const field of ['hidden','inert','aria-hidden']){
  prepare();const target=find('#identity-panel-auth'),url=location.href,before=frameEvents.length;
  target.setAttribute(field,field==='aria-hidden'?'true':'');navigate();flush();
  assert.equal(frameEvents.length,before,'native hidden/inert destination cannot reveal frame');assert.equal(frameHost.hasAttribute('data-shell-inactive'),true);assert.equal(location.href,url);target.removeAttribute(field);
}
prepare();navigate();const source=root().getAttribute('data-account-source'),before=frameEvents.length,url=location.href;root().setAttribute('data-account-source','[]');flush();
assert.equal(frameEvents.length,before,'source invalidated after admission cannot reveal frame');assert.equal(frameHost.hasAttribute('data-shell-inactive'),true);assert.equal(location.href,url);root().setAttribute('data-account-source',source);flush();
frameHost.removeAttribute('data-shell-inactive');click(find('#identity-tab-auth'));flush();
const tenant=find('#sign-in-tenant');tenant.value='real-tenant';signin.setAttribute('data-shell-inactive','');
click(find('[data-identity-action="add-passkey"]'));assert.equal(location.hash,'#sign-in-invitation-title');assert.equal(frameEvents.at(-1),'sign-in-invitation-title');assert.equal(signin.hasAttribute('data-shell-inactive'),false);assert.equal(document.activeElement,find('#sign-in-invitation-title'));assert.ok(document.activeElement.getClientRects().length);assert.equal(tenant.value,'real-tenant');assert.equal(find('#passkey-enroll').events.click,undefined,'invitation reveal does not start credential ceremony');
console.log('Settings managed frame reveal, cancellation, native hiding, source refusal and native invitation PASS');
"##,
    );
}
