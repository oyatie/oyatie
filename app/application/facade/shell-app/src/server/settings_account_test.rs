use leptos::prelude::*;
use std::io::Write;
use std::process::{Command, Stdio};

const CHECK: &str = r##"
const assert = require('node:assert/strict'), vm = require('node:vm');
const input = JSON.parse(require('node:fs').readFileSync(0,'utf8'));
const events = {}, windowEvents = {}, microtasks = [], frames = [];
const observers = []; let focused = true;
const decode = value => value.replace(/&(quot|apos|amp|lt|gt|#x[0-9a-f]+|#[0-9]+);/gi, (_, name) => name[0] === '#'
  ? String.fromCodePoint(name[1].toLowerCase() === 'x' ? parseInt(name.slice(2),16) : Number(name.slice(1)))
  : ({quot:'"',apos:"'",amp:'&',lt:'<',gt:'>'})[name.toLowerCase()]);
const escape = value => String(value).replaceAll('&','&amp;').replaceAll('"','&quot;').replaceAll('<','&lt;').replaceAll('>','&gt;');
const voids = new Set(['input','meta','link','br','hr','img']);
class Comment {
  constructor(data) { this.data=data; this.nodeType=8; }
  get textContent() { return ''; }
  get outerHTML() { return `<!--${this.data}-->`; }
  cloneNode() { return new Comment(this.data); }
  remove() { Element.prototype.remove.call(this); }
}
class Element {
  constructor(tag,attrs={}) { this.tagName=tag.toUpperCase(); this.attrs=attrs; this.parts=[]; this.events={}; }
  get children() { return this.parts.filter(part => part instanceof Element); }
  get firstElementChild() { return this.children[0] || null; }
  get cells() { return this.children.filter(node => ['TD','TH'].includes(node.tagName)); }
  get nextElementSibling() { const nodes=this.parentElement?.children || [];return nodes[nodes.indexOf(this)+1] || null; }
  get id() { return this.attrs.id || ''; }
  get dataset() { return Object.fromEntries(Object.entries(this.attrs).filter(([name])=>name.startsWith('data-')).map(([name,value])=>[name.slice(5).replace(/-([a-z])/g,(_,letter)=>letter.toUpperCase()),value])); }
  get isConnected() { return this===tree || !!this.parentElement?.isConnected; }
  get textContent() { return this.parts.map(part=>typeof part==='string'?part:part.textContent).join(''); }
  set textContent(value) { this.children.forEach(node=>node.parentElement=null); this.parts=[String(value)]; }
  get innerHTML() { return this.parts.map(part=>typeof part==='string'?escape(part):part.outerHTML).join(''); }
  set innerHTML(value) { this.children.forEach(node=>node.parentElement=null); this.parts=[]; parse(value,this); }
  get outerHTML() { return `<${this.tagName.toLowerCase()}${Object.entries(this.attrs).map(([key,value])=>` ${key}="${escape(value)}"`).join('')}>${voids.has(this.tagName.toLowerCase())?'':this.innerHTML+`</${this.tagName.toLowerCase()}>`}`; }
  get disabled() { return this.hasAttribute('disabled'); }
  set disabled(value) { value?this.setAttribute('disabled',''):this.removeAttribute('disabled'); }
  get hidden() { return this.hasAttribute('hidden'); }
  set hidden(value) { value?this.setAttribute('hidden',''):this.removeAttribute('hidden'); }
  get open() { return this.hasAttribute('open'); }
  set open(value) { value?this.setAttribute('open',''):this.removeAttribute('open'); }
  get style() { return {getPropertyValue:name=>this.styles?.[name] || '',setProperty:(name,value)=>{(this.styles ||= {})[name]=value;this.attrs.style=Object.entries(this.styles).map(([key,value])=>`${key}: ${value}`).join(';');}}; }
  get classList() { const list=()=>new Set((this.attrs.class||'').split(/\s+/).filter(Boolean));return {contains:name=>list().has(name),toggle:(name,value)=>{const names=list();if(value)names.add(name);else names.delete(name);this.attrs.class=[...names].join(' ');}}; }
  getAttribute(name) { return this.attrs[name]??null; }
  hasAttribute(name) { return Object.hasOwn(this.attrs,name); }
  setAttribute(name,value) { this.attrs[name]=String(value); }
  removeAttribute(name) { delete this.attrs[name]; }
  append(node) { node.remove();node.parentElement=this;this.parts.push(node);return node; }
  remove() { if(this.parentElement){this.parentElement.parts=this.parentElement.parts.filter(part=>part!==this);this.parentElement=null;} }
  replaceWith(node) { const parent=this.parentElement,index=parent.parts.indexOf(this);this.parentElement=null;node.parentElement=parent;parent.parts[index]=node; }
  before(node) { node.remove();node.parentElement=this.parentElement;this.parentElement.parts.splice(this.parentElement.parts.indexOf(this),0,node); }
  contains(node) { for(;node;node=node.parentElement)if(node===this)return true;return false; }
  matches(selector) { return selector.split(',').some(part=>{const terms=part.trim().split(/\s+(?![^[]*\])/);let node=this;const match=(node,term)=>{
    const attrs=[...term.matchAll(/\[([\w-]+)(?:="([^"]*)")?\]/g)], plain=term.replace(/\[[^\]]*\]/g,'');
    const tag=plain.match(/^[a-z][\w-]*/i)?.[0].toUpperCase();
    return (!tag||node.tagName===tag)&&[...plain.matchAll(/#([\w-]+)/g)].every(([,id])=>node.id===id)
      &&[...plain.matchAll(/\.([\w-]+)/g)].every(([,name])=>node.classList.contains(name))
      &&attrs.every(([,name,value])=>value===undefined?node.hasAttribute(name):node.getAttribute(name)===value);
  };if(!match(node,terms.pop()))return false;while(terms.length){const term=terms.pop();do{node=node.parentElement;}while(node&&!match(node,term));if(!node)return false;}return true;}); }
  closest(selector) { for(let node=this;node;node=node.parentElement)if(node.matches(selector))return node;return null; }
  querySelectorAll(selector) { const nodes=[];const visit=node=>{for(const child of node.children){if(child.matches(selector))nodes.push(child);visit(child);}};visit(this);return nodes; }
  querySelector(selector) { return this.querySelectorAll(selector)[0]||null; }
  cloneNode() { const copy=new Element(this.tagName,{...this.attrs});for(const part of this.parts)typeof part==='string'?copy.parts.push(part):copy.append(part.cloneNode(true));if(this.content)copy.content=this.content.cloneNode(true);return copy; }
  getClientRects() { const panel=this.closest('.identity-panel, .finance-panel, .cockpit-panel, .resource-panel');return this.isConnected&&!this.closest('[hidden]')&&!(panel&&!panel.classList.contains('active'))?[{}]:[]; }
  focus() { document.activeElement=this;emit('focusin',{target:this}); }
  scrollIntoView() { this.scrolled=true; }
  close() { this.open=false; }
  click() { click(this); }
  addEventListener(name,callback) { (this.events[name] ||= []).push(callback); }
}
function parse(html,root) {
  const stack=[root];
  html=html.replace(/<(script|style)\b[^>]*>[\s\S]*?<\/\1>/gi,'');
  for(const [token] of html.matchAll(/<!--[\s\S]*?-->|<![^>]*>|<\/[a-z][^>]*>|<[a-z][^>]*>|[^<]+/gi)) {
    if(token.startsWith('<!--')) { stack.at(-1).append(new Comment(token.slice(4,-3)));continue; }
    if(token.startsWith('<!'))continue;
    if(token.startsWith('</')) { const tag=token.match(/^<\/([\w-]+)/)[1].toUpperCase(); if(tag==='TEMPLATE')stack.pop();else assert.equal(stack.pop().tagName,tag);continue; }
    if(!token.startsWith('<')) { stack.at(-1).parts.push(decode(token));continue; }
    const tag=token.match(/^<([\w-]+)/)[1],attrs={};
    for(const [,name,double,single,bare] of token.slice(tag.length+1,-1).matchAll(/([^\s=/>]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+)))?/g))attrs[name]=decode(double??single??bare??'');
    const node=stack.at(-1).append(new Element(tag,attrs));
    if(tag==='template'){node.content=new Element('fragment');stack.push(node.content);}
    else if(!voids.has(tag)&&!token.endsWith('/>'))stack.push(node);
  }
  assert.equal(stack.length,1);
}
const tree=new Element('body');parse(input.html,tree);
const NodeFilter={SHOW_COMMENT:128};
const document={body:tree,activeElement:tree,hasFocus:()=>focused,getElementById:id=>tree.querySelector('#'+id),querySelectorAll:selector=>tree.querySelectorAll(selector),querySelector:selector=>tree.querySelector(selector),createElement:tag=>new Element(tag),createComment:data=>new Comment(data),createTreeWalker:(root,whatToShow)=>{
  assert.equal(whatToShow,NodeFilter.SHOW_COMMENT);
  const comments=[],visit=node=>{for(const child of node.parts||[]){if(child instanceof Comment)comments.push(child);else if(child instanceof Element)visit(child);}};visit(root);
  let index=0;return {nextNode:()=>comments[index++]||null};
},addEventListener:(name,callback)=>(events[name] ||= []).push(callback)};
const required=selector=>{const nodes=document.querySelectorAll(selector);assert.equal(nodes.length,1,`one required native control: ${selector}`);return nodes[0];};
const commentOwners=['[data-account-person]','[data-account-context-facts]'].map(required);
for(const node of commentOwners)for(const parent of [node,node,node.firstElementChild])parent.append(document.createComment(''));
const rendererHTML=commentOwners.map(node=>node.innerHTML);
function assertHRUnavailable(stage) {
  const add=required('#identity-panel-employees [data-identity-action="add-employee"]');
  const explanation=required('#employee-add-unavailable');
  assert.equal(add.getAttribute('aria-describedby'),explanation.id,stage+' retains HR explanation');
  assert.equal(add.disabled,true,stage+' retains HR authored disabled Add employee');
}
assertHRUnavailable('before document bridge initialization');
const location=new URL('https://preview.invalid/');
const emit=(name,event)=>(events[name]||[]).forEach(callback=>callback(event));
function click(target,options={}){const event={target,button:0,isTrusted:false,defaultPrevented:false,...options,preventDefault(){this.defaultPrevented=true;}};for(let node=target;node;node=node.parentElement)for(const callback of node.events.click||[])callback(event);if(target.isConnected)emit('click',event);if(target.tagName==='A'&&!event.defaultPrevented)location.hash=target.getAttribute('href');return event;}
function enter(target){emit('keydown',{target,key:'Enter',isTrusted:true});return click(target,{detail:0});}
const context={document,location,Element,NodeFilter,URL,window:{addEventListener:(name,callback)=>(windowEvents[name] ||= []).push(callback)},queueMicrotask:callback=>microtasks.push(callback),requestAnimationFrame:callback=>frames.push(callback),MutationObserver:class{constructor(callback){observers.push(callback);}observe(){}}};
vm.runInNewContext(input.tabs,context,{timeout:1000});
vm.runInNewContext(input.hr,context,{timeout:1000});
vm.runInNewContext(input.bridge,context,{timeout:1000});
assertHRUnavailable('after document bridge initialization');
const flush=(accountRoots=1)=>{observers.forEach(callback=>callback());for(let count=0;microtasks.length||frames.length;count++){assert.ok(count<30,'bounded synchronization');for(const callback of microtasks.splice(0))callback();for(const callback of frames.splice(0))callback();}assert.equal(document.querySelectorAll('#identity-workforce-service').length,accountRoots,'explicit Account root phase');if(accountRoots===1)assertHRUnavailable('source synchronization');};
const find=selector=>document.querySelector(selector), root=()=>find('#identity-workforce-service');
function assertOwnedAvailability(available) {
  for(const selector of ['.identity-service-head [data-identity-action="open-audit"]','[data-account-facts="auth"] [data-identity-action="add-passkey"]','[data-account-facts="auth"] [data-identity-route-action="evidence"][data-account-focus="evidence"]','[data-account-facts="roles"] [data-identity-anchor-action="review-roles"]',...['advance-onboarding','back-onboarding','reset-onboarding'].map(action=>`[data-account-facts="onboarding"] [data-identity-action="${action}"]`),'[data-account-facts="onboarding"] [data-onboarding-anchor-action="advance-setup"]']) {
    assert.equal(required(selector).disabled,!available,selector+' follows Settings availability');
  }
  for(const kind of ['auth','sessions','roles','org','onboarding'])assert.equal(required(`.identity-settings-rail [data-identity-tab="${kind}"]`).disabled,!available,kind+' rail follows Settings availability');
  for(const route of ['tasks','schedule'])assert.equal(required(`[data-account-facts="onboarding"] [data-account-route="${route}"]`).getAttribute('aria-disabled'),String(!available),route+' anchor follows Settings availability');
  for(const tab of document.querySelectorAll('.identity-tabs [role="tab"]'))assert.equal(tab.disabled,false,'canonical tab state remains owned by tab navigation');
  assert.equal(required('.identity-settings-rail [data-identity-tab="employees"]').disabled,false,'HR rail state remains authored');
  assertHRUnavailable('owned control availability');
}
assertOwnedAvailability(true);
for(const [index,node] of commentOwners.entries()) {
  assert.equal(node.innerHTML,rendererHTML[index],'signature leaves renderer comments in the original DOM');
  const comment=node.append(document.createComment('source marker'));flush();assertOwnedAvailability(false);
  assert.match(required('[data-identity-status]').textContent,/invalid/,'nonempty comments remain source facts');
  assert.equal(comment.parentElement,node,'refusal leaves nonempty comments in the original DOM');comment.remove();flush();assertOwnedAvailability(true);
  const fact=node.querySelector('strong, dt'), original=fact.textContent;fact.textContent='Changed sample fact';flush();assertOwnedAvailability(false);
  fact.textContent=original;flush();assertOwnedAvailability(true);
  fact.setAttribute('data-literal','<!---->');flush();assertOwnedAvailability(false);
  assert.equal(fact.getAttribute('data-literal'),'<!---->','literal comment syntax remains an attribute value');
  fact.removeAttribute('data-literal');flush();assertOwnedAvailability(true);
}
const selectRail=kind=>{click(find(`.identity-settings-rail [data-identity-tab="${kind}"]`));flush();assert.equal(find(`#identity-tab-${kind}`).getAttribute('aria-selected'),'true');assert.equal(find(`.identity-settings-rail [data-identity-tab="${kind}"]`).getAttribute('aria-current'),'page');};
selectRail('onboarding');
const count=()=>{assertHRUnavailable('setup action');return find('[data-account-local="count"]').textContent;};
assert.match(count(),/3 of 9.*33%/);
assert.doesNotMatch(find('[data-identity-status]').textContent,/invalid|loading/);
const advance=()=>click(find('[data-identity-action="advance-onboarding"]'));
advance();assert.match(count(),/4 of 9.*44%/);assertHRUnavailable('setup advance');
click(find('[data-identity-action="back-onboarding"]'));assert.match(count(),/4 of 9/);
assert.match(find('[data-account-local="current-step"]').textContent,/4\. Payroll/);
advance();assert.match(count(),/4 of 9/,'reinspection does not double count');
for(let index=0;index<6;index++)advance();
assert.match(count(),/9 of 9.*100%/);assert.equal(find('[data-account-local="progress"]').getAttribute('aria-valuenow'),'100');
assert.match(find('[data-account-local="setup-status"]').textContent,/No tenant admission, go-live, readiness/);
click(find('[data-identity-action="reset-onboarding"]'));assert.match(count(),/3 of 9.*33%/);
const drawer=find('#settings-center');drawer.open=true;click(find('[data-settings-action="density-compact"]'));assert.equal(drawer.dataset.settingsDensity,'compact');
click(find('[data-settings-action="locale-ko"]'));assert.match(find('[data-settings-label="profile"]').textContent,/샘플/);
assert.equal(tree.getAttribute('lang'),null,'locale remains inside Settings drawer');

// Unhandled and foreign controls retain their own availability and click dispatch.
const foreign=root().append(new Element('button',{'data-identity-action':'owner-specific-action'}));
const foreignDisabled=root().append(new Element('button',{'data-identity-route-action':'owner-specific-route',disabled:''}));
const employees=required('#identity-panel-employees');
const foreignPasskey=employees.append(new Element('button',{'data-identity-action':'add-passkey'}));
const foreignSetup=employees.append(new Element('button',{'data-identity-action':'advance-setup'}));
const foreignAnchor=employees.append(new Element('a',{'data-account-route':'tasks',href:'#hr-owned-route'}));
flush();assert.equal(foreign.disabled,false);assert.equal(foreignDisabled.disabled,true);assert.equal(foreignPasskey.disabled,false);assert.equal(foreignSetup.disabled,false);assert.equal(foreignAnchor.getAttribute('aria-disabled'),null);
selectRail('employees');const beforeForeign=count(), invitationBefore=find('.sign-in-invitation').open;
for(const control of [foreign,foreignPasskey,foreignSetup])assert.equal(click(control).defaultPrevented,false,'Settings does not consume foreign or unhandled actions');
assert.equal(count(),beforeForeign);assert.equal(find('.sign-in-invitation').open,invitationBefore);
const foreignURL=location.href;assert.equal(click(foreignAnchor).defaultPrevented,false,'foreign anchor keeps native default');assert.equal(location.hash,'#hr-owned-route');location.href=foreignURL;
// A foreign or unknown click cannot cancel an already admitted Settings route.
location.hash='#settings-pending-owned-route';click(required('.identity-service-head [data-identity-action="open-audit"]'));
for(const control of [foreign,foreignPasskey,foreignSetup])assert.equal(click(control).defaultPrevented,false);
flush();assert.equal(location.hash,'#resource-panel-audit');assert.match(find('[data-identity-status]').textContent,/existing sample opened/);assert.equal(count(),beforeForeign);
// Invalid facts disable owned actions without claiming a newly introduced unknown action.
const unknownFacts=required('[data-account-facts="onboarding"]').append(new Element('button',{'data-identity-action':'unhandled-fact-action'}));
flush();assertOwnedAvailability(false);assert.equal(unknownFacts.disabled,false);assert.equal(foreign.disabled,false);assert.equal(foreignDisabled.disabled,true);assert.equal(foreignPasskey.disabled,false);assert.equal(foreignSetup.disabled,false);assert.equal(foreignAnchor.getAttribute('aria-disabled'),null);
click(required('#identity-tab-onboarding'));flush();assert.equal(click(unknownFacts).defaultPrevented,false);selectRail('employees');assert.equal(click(foreignPasskey).defaultPrevented,false);
unknownFacts.remove();flush();assertOwnedAvailability(true);
// A replacement owner must not bind foreign anchors to Settings refusal.
const ownershipOriginal=root(), ownershipReplacement=ownershipOriginal.cloneNode(true);ownershipOriginal.replaceWith(ownershipReplacement);flush();assertOwnedAvailability(true);
ownershipReplacement.setAttribute('data-account-available','false');flush();assertOwnedAvailability(false);selectRail('employees');
const remountedForeignAnchor=required('#identity-panel-employees [data-account-route="tasks"]');assert.equal(remountedForeignAnchor.getAttribute('aria-disabled'),null);assert.equal(click(remountedForeignAnchor).defaultPrevented,false,'unavailable Settings does not bind HR anchor refusal');assert.equal(location.hash,'#hr-owned-route');
ownershipReplacement.setAttribute('data-account-available','true');flush();assertOwnedAvailability(true);ownershipReplacement.replaceWith(ownershipOriginal);flush();assertOwnedAvailability(true);
for(const control of [foreign,foreignDisabled,foreignPasskey,foreignSetup,foreignAnchor])control.remove();flush();
for(const kind of ['sessions','roles','org','onboarding','auth'])selectRail(kind);
selectRail('roles');click(find('[data-identity-anchor-action="review-roles"]'));assert.match(find('[data-identity-status]').textContent,/sample grant rows/);
const signin=find('#oyatie-sign-in'), invitation=find('.sign-in-invitation'), tenant=find('#sign-in-tenant');tenant.value='real-tenant';
selectRail('auth');click(find('[data-identity-action="add-passkey"]'));assert.equal(invitation.open,true);assert.equal(tenant.value,'real-tenant');assert.equal(find('#passkey-enroll').events.click,undefined);
signin.setAttribute('aria-busy','true');invitation.open=false;document.activeElement=tenant;click(find('[data-identity-action="add-passkey"]'));assert.equal(invitation.open,false);assert.equal(document.activeElement,tenant);signin.setAttribute('aria-busy','false');document.activeElement=tree;
for(const route of ['workflow','finance','evidence']){click(find(`[data-identity-route-action="${route}"]`));flush();assert.equal(location.hash,{workflow:'#workflow-studio',finance:'#payroll-cockpit',evidence:'#evidence-spine'}[route]);}
selectRail('sessions');for(const route of ['policy','audit','cloud']){click(find(`[data-identity-anchor-action="route-${route}"]`));flush();assert.equal(location.hash,{policy:'#cockpit-panel-policy',audit:'#resource-panel-audit',cloud:'#cockpit-panel-topology'}[route]);}
selectRail('onboarding');for(const route of ['tasks','schedule']){click(find(`[data-account-route="${route}"]`));flush();assert.equal(location.hash,'#'+route+'-title');}
click(find('[data-onboarding-anchor-action="import-employees"]'));flush();assert.equal(find('#identity-tab-employees').getAttribute('aria-selected'),'true');assert.match(find('[data-identity-status]').textContent,/Import is unavailable/);assertHRUnavailable('import inspection');
selectRail('onboarding');const payrollMail=find('#payroll-mail-preview');
for(const route of ['mail','community']){click(find(`[data-onboarding-anchor-action="route-${route}"]`));flush();const channel=route==='mail'?'Mail':'Community';assert.equal(find('.communications-hub').querySelector('[data-comms-product-board]').dataset.commsBoardSurface,channel);assert.equal(find('#payroll-mail-preview'),payrollMail);assert.match(find('[data-identity-status]').textContent,/no draft, message, publication, receipt/);}
const hub=find('.communications-hub');hub.querySelector('.hub-composer').remove();const before=location.href;click(find('[data-onboarding-anchor-action="route-mail"]'));flush();assert.equal(location.href,before);assert.match(find('[data-identity-status]').textContent,/unavailable/);
// Restore a complete linked Hub before source/lifecycle checks.
const template=find('[data-account-hub-template="Mail"]');hub.replaceWith(template.content.firstElementChild.cloneNode(true));flush();
selectRail('onboarding');advance();find('[data-account-focus="setup-advance"]').focus();const old=root(), fresh=old.cloneNode(true);old.replaceWith(fresh);document.activeElement=tree;flush();assert.match(count(),/4 of 9/,'same exact semantic source retains valid sample completion');assert.equal(drawer.dataset.settingsDensity,'compact');assert.equal(document.activeElement,find('[data-account-focus="setup-advance"]'),'same-source visible local focus restored');
// Native mounting empties the stable host while its next envelope loads.
for(const loading of [false,true]) {
  selectRail('onboarding');find('[data-account-focus="setup-advance"]').focus();
  const replacement=fresh.cloneNode(true), parent=fresh.parentElement, frame=fresh.closest('[data-island="render-envelope-dashboard"]');
  const before=count(), url=location.href;click(find('.identity-service-head [data-identity-action="open-audit"]'));
  fresh.remove();document.activeElement=tree;flush(0);assert.equal(location.href,url,'missing source cancels pending navigation');
  assert.equal(fresh.querySelector('[data-account-local="count"]').textContent,before,'loading preserves only local sample progress');
  if(loading)frame.setAttribute('aria-busy','true');parent.append(replacement);flush();assert.equal(count(),before);
  if(loading){assertOwnedAvailability(false);assert.equal(document.activeElement,tree,'unavailable replacement never restores focus');frame.setAttribute('aria-busy','false');flush();}
  assert.equal(document.activeElement.dataset.accountFocus,'setup-advance',loading?'validated availability settlement restores local focus':'same-source remount restores focus after missing-root interval');
  assert.equal(location.href,url,'returning source does not replay cancelled navigation');
  replacement.replaceWith(fresh);document.activeElement=tree;flush();
}
// A duplicate host introduced during absence must stay refused through return.
selectRail('onboarding');find('[data-account-focus="setup-advance"]').focus();
const ambiguousReplacement=fresh.cloneNode(true), accountParent=fresh.parentElement, duplicateHostID=required('#dashboard-island-root').id;
const beforeAmbiguity=count(), ambiguityURL=location.href;
fresh.remove();document.activeElement=tree;flush(0);
const persistentDuplicateHost=tree.append(new Element('div',{id:duplicateHostID}));accountParent.append(ambiguousReplacement);flush();
assert.equal(document.activeElement.dataset.accountFocus,undefined,'duplicate host persisting through source return cancels local focus');
advance();click(find('.identity-service-head [data-identity-action="open-audit"]'));flush();
assert.equal(count(),beforeAmbiguity,'ambiguous host refuses owned setup actions');assert.equal(location.href,ambiguityURL,'ambiguous host refuses owned navigation');
persistentDuplicateHost.remove();flush();assertOwnedAvailability(true);assert.equal(document.activeElement.dataset.accountFocus,undefined,'resolved host ambiguity does not revive stale focus');
ambiguousReplacement.replaceWith(fresh);document.activeElement=tree;flush();
// Unique ownership cannot admit a never-bound invalid or unavailable source.
for(const refusal of ['malformed-source','changed-facts','unavailable-source']) {
  selectRail('onboarding');const replacement=fresh.cloneNode(true), parent=fresh.parentElement;
  if(refusal==='malformed-source')replacement.setAttribute('data-account-source','invalid');
  else if(refusal==='changed-facts')replacement.querySelector('[data-account-facts="org"] strong').textContent='changed';
  else replacement.setAttribute('data-account-available','false');
  const before=count(), url=location.href;fresh.remove();document.activeElement=tree;flush(0);parent.append(replacement);
  const anchors=['tasks','schedule'].map(route=>replacement.querySelector(`[data-account-route="${route}"]`));
  for(const anchor of anchors)assert.equal(anchor.events.click,undefined,'unique replacement anchor has never been bound');
  assert.equal(document.querySelectorAll('#identity-workforce-service').length,1,'invalid replacement has unique ownership');
  const modified=target=>click(target,{ctrlKey:true});
  const activateFirst=refusal==='unavailable-source'?modified:click;
  assert.equal(activateFirst(anchors[0]).defaultPrevented,true,refusal+' refuses the first unbound default');
  assert.equal(location.href,url);flush();assertOwnedAvailability(false);
  for(const anchor of anchors)for(const activate of [click,enter,modified,target=>click(target,{metaKey:true})]) {
    assert.equal(activate(anchor).defaultPrevented,true,refusal+' refuses primary, Enter and modified defaults');
    assert.equal(location.href,url);flush();
  }
  const unknown=replacement.append(new Element('button',{'data-identity-action':'owner-specific-action'}));
  assert.equal(click(unknown).defaultPrevented,false);assert.equal(unknown.disabled,false);
  const hrPanel=replacement.querySelector('#identity-panel-employees');hrPanel.classList.toggle('active',true);
  const hrAnchor=hrPanel.append(new Element('a',{'data-account-route':'tasks',href:'#hr-owned-route'}));
  assert.equal(click(hrAnchor).defaultPrevented,false,refusal+' preserves HR native default');assert.equal(hrAnchor.getAttribute('aria-disabled'),null);hrAnchor.remove();hrPanel.classList.toggle('active',false);location.href=url;
  const repaired=fresh.cloneNode(true);replacement.replaceWith(repaired);document.activeElement=tree;flush();assertOwnedAvailability(true);
  assert.equal(count(),before);assert.equal(location.href,url,refusal+' repair does not replay a refused route');
  for(const route of ['tasks','schedule'])assert.equal(repaired.querySelector(`[data-account-route="${route}"]`).getClientRects().length,1,refusal+' repairs a visible admitted route');
  for(const modifier of ['ctrlKey','metaKey']) {
    const anchor=repaired.querySelector('[data-account-route="tasks"]');
    const event=click(anchor,{[modifier]:true});
    assert.equal(event.defaultPrevented,false,`${refusal} repaired ${modifier} retains native default`);
    assert.equal(location.hash,'#tasks-title',refusal+' admitted modified anchor retains fragment default');
  }
  location.href=url;click(repaired.querySelector('[data-account-route="schedule"]'));flush();assert.equal(location.hash,'#schedule-title','fresh admitted route works after invalid-source repair');
  repaired.replaceWith(fresh);document.activeElement=tree;flush();
}
// Newly authored roots have no refusal listeners from an earlier validation.
for(const refusal of ['duplicate-roots','duplicate-host','misplaced-root']) {
  selectRail('onboarding');const replacement=fresh.cloneNode(true), parent=fresh.parentElement;
  const anchors=['tasks','schedule'].map(route=>replacement.querySelector(`[data-account-route="${route}"]`));
  for(const anchor of anchors)assert.equal(anchor.events.click,undefined,'replacement anchor has never been bound');
  const before=count(), url=location.href;fresh.remove();document.activeElement=tree;flush(0);
  (refusal==='misplaced-root'?tree:parent).append(replacement);
  const duplicate=refusal==='duplicate-roots'?parent.append(replacement.cloneNode(true))
    :refusal==='duplicate-host'?tree.append(new Element('div',{id:'dashboard-island-root'})):null;
  assert.equal(click(anchors[0]).defaultPrevented,true,refusal+' refuses unbound anchor before synchronization');
  assert.equal(location.href,url,refusal+' cannot follow the native fragment default');
  flush(refusal==='duplicate-roots'?2:1);
  for(const candidate of duplicate&&refusal==='duplicate-roots'?[replacement,duplicate]:[replacement]) {
    assert.equal(candidate.querySelector('[data-identity-action="advance-onboarding"]').disabled,true,refusal+' disables owned setup');
    assert.equal(candidate.querySelector('#identity-panel-employees [data-identity-action="add-employee"]').disabled,true,refusal+' preserves HR authored refusal');
    assert.match(candidate.querySelector('[data-identity-status]').textContent,/invalid|unavailable/,refusal+' explains ownership refusal');
    for(const route of ['tasks','schedule'])assert.equal(candidate.querySelector(`[data-account-route="${route}"]`).getAttribute('aria-disabled'),'true',refusal+' exposes unavailable link semantics');
  }
  for(const anchor of anchors)for(const activate of [click,enter,target=>click(target,{ctrlKey:true})]) {
    assert.equal(activate(anchor).defaultPrevented,true,refusal+' refuses primary, Enter and modified anchor defaults');
    flush(refusal==='duplicate-roots'?2:1);assert.equal(location.href,url);assert.equal(document.activeElement,tree);
  }
  const hrPanel=replacement.querySelector('#identity-panel-employees');hrPanel.classList.toggle('active',true);
  const hrAnchor=hrPanel.append(new Element('a',{'data-account-route':'tasks',href:'#hr-owned-route'}));
  assert.equal(click(hrAnchor).defaultPrevented,false,refusal+' preserves HR native anchor default');assert.equal(location.hash,'#hr-owned-route');
  assert.equal(hrAnchor.getAttribute('aria-disabled'),null);hrAnchor.remove();hrPanel.classList.toggle('active',false);location.href=url;
  const unrelated=tree.append(new Element('a',{href:'#tasks-title'}));
  assert.equal(click(unrelated).defaultPrevented,false,refusal+' preserves unrelated native fragment');assert.equal(location.hash,'#tasks-title');unrelated.remove();location.href=url;
  if(duplicate)duplicate.remove();if(refusal==='misplaced-root')parent.append(replacement);flush();assertOwnedAvailability(true);
  assert.equal(location.href,url,refusal+' repair does not replay refused navigation');assert.equal(count(),before,refusal+' repair preserves only sample progress');
  assert.equal(document.activeElement,tree,refusal+' repair does not restore obsolete focus');
  assert.equal(click(anchors[0],{ctrlKey:true}).defaultPrevented,false,'admitted modified anchor retains native default');assert.equal(location.hash,'#tasks-title');location.href=url;
  click(anchors[1]);flush();assert.equal(location.hash,'#schedule-title','fresh admitted Account action works after repair');
  replacement.replaceWith(fresh);document.activeElement=tree;flush();
}
// Absence cannot mask invalid ownership, source changes, or newer user intent.
for(const refusal of ['malformed-source','changed-facts','duplicate-roots','misplaced-root','detached-host','duplicate-host','connected-origin','pointerdown','keydown','blur','hashchange','popstate','iam-focus','changed-key']) {
  selectRail('onboarding');find('[data-account-focus="setup-advance"]').focus();
  let replacement=fresh.cloneNode(true);const parent=fresh.parentElement, mount=required('#dashboard-island-root'), mountParent=mount.parentElement, url=location.href;
  if(refusal==='connected-origin'){fresh.removeAttribute('id');flush(0);fresh.setAttribute('id','identity-workforce-service');fresh.replaceWith(replacement);document.activeElement=tree;}
  else {
    fresh.remove();document.activeElement=tree;flush(0);
    if(refusal==='malformed-source'||refusal==='changed-facts') {
      if(refusal==='malformed-source')replacement.setAttribute('data-account-source','invalid');else replacement.querySelector('[data-account-facts="org"] strong').textContent='changed';
      parent.append(replacement);flush();assertOwnedAvailability(false);const repaired=fresh.cloneNode(true);replacement.replaceWith(repaired);replacement=repaired;
    } else if(refusal==='duplicate-roots') {
      parent.append(replacement);const duplicate=replacement.cloneNode(true);parent.append(duplicate);flush(2);duplicate.remove();
    } else if(refusal==='misplaced-root') {tree.append(replacement);flush();parent.append(replacement);}
    else if(refusal==='detached-host'){mount.remove();flush(0);mountParent.append(mount);parent.append(replacement);}
    else if(refusal==='duplicate-host'){const duplicate=tree.append(new Element('div',{id:mount.id}));flush(0);duplicate.remove();parent.append(replacement);}
    else {
      if(['pointerdown','keydown'].includes(refusal))emit(refusal,{target:tenant,isTrusted:true});
      if(['blur','popstate'].includes(refusal))for(const callback of windowEvents[refusal]||[])callback({isTrusted:true});
      if(refusal==='hashchange'){location.hash='#newer-settings-intent';for(const callback of windowEvents.hashchange||[])callback({oldURL:url,newURL:location.href});location.href=url;}
      if(refusal==='iam-focus')tenant.focus();
      if(refusal==='changed-key'){const source=JSON.parse(replacement.dataset.accountSource);source[3][0]+='<p>Changed sample person illustration</p>';replacement.querySelector('[data-account-person]').innerHTML=source[3][0];replacement.setAttribute('data-account-source',JSON.stringify(source));}
      parent.append(replacement);
    }
  }
  flush();assertOwnedAvailability(true);assert.equal(document.activeElement=== (refusal==='iam-focus'?tenant:tree),true,refusal+' cancels obsolete local focus');
  assert.equal(location.href,url,refusal+' does not create navigation');if(refusal==='changed-key')assert.match(count(),/3 of 9.*33%/,'changed source resets local sample progress');
  replacement.replaceWith(fresh);document.activeElement=tree;flush();assertOwnedAvailability(true);assert.equal(tenant.value,'real-tenant','IAM input remains unchanged');
}
const status=find('[data-identity-status]');find('[data-account-facts="org"] strong').textContent='tampered';flush();const prior=count();advance();assert.equal(count(),prior);assert.match(status.textContent,/invalid/);assertOwnedAvailability(false);
// A valid changed semantic source resets completion, including when its context remains the same.
const changed=old.cloneNode(true), source=JSON.parse(changed.dataset.accountSource);source[3][0]+='<p>Changed sample person illustration</p>';changed.querySelector('[data-account-person]').innerHTML=source[3][0];changed.setAttribute('data-account-source',JSON.stringify(source));fresh.replaceWith(changed);flush();assert.match(count(),/3 of 9.*33%/);assert.equal(find('[data-account-reset]').hidden,false);assertOwnedAvailability(true);
const frame=changed.closest('[data-island="render-envelope-dashboard"]');frame.setAttribute('aria-busy','true');flush();assertOwnedAvailability(false);advance();assert.match(count(),/3 of 9/);frame.setAttribute('aria-busy','false');flush();assertOwnedAvailability(true);advance();assert.match(count(),/4 of 9/);
const picker=find('[data-sample-context-requested]');picker.setAttribute('data-sample-context-requested','corporate-office');flush();advance();assert.match(count(),/4 of 9/);picker.setAttribute('data-sample-context-requested','tenant-admin');flush();
for(const availability of ['false',null,'invalid']) { if(availability===null)changed.removeAttribute('data-account-available');else changed.setAttribute('data-account-available',availability);flush();assertOwnedAvailability(false);const prior=count();advance();assert.equal(count(),prior); }
changed.setAttribute('data-account-available','true');const error=frame.append(new Element('p',{class:'fetch-error'}));flush();const beforeError=count();advance();assert.equal(count(),beforeError);error.remove();flush();assertOwnedAvailability(true);
// Independently authored target availability and source are never normalized away.
selectRail('sessions');const evidence=find('#evidence-spine'), targetSource=evidence.dataset.aeSource;
for(const availability of ['false',null,'invalid']) { if(availability===null)evidence.removeAttribute('data-ae-available');else evidence.setAttribute('data-ae-available',availability);
  const before=location.href;click(find('[data-identity-anchor-action="route-evidence"]'));flush();assert.equal(location.href,before);assert.match(find('[data-identity-status]').textContent,/unavailable/); }
evidence.setAttribute('data-ae-available','true');evidence.setAttribute('data-ae-source','invalid');click(find('[data-identity-anchor-action="route-evidence"]'));flush();assert.match(find('[data-identity-status]').textContent,/unavailable/);evidence.setAttribute('data-ae-source',targetSource);
// Native Audit/Evidence is one linked consumer; both independent inputs on both roots must admit it.
function refusedRoute(route, change, restore, late) {
  location.hash='#account-route-refusal';const before=location.href;
  if(!late)change();click(find(`[data-identity-anchor-action="route-${route}"]`));if(late)change();flush();
  assert.equal(location.href,before,`${route} unavailable before ${late?'confirmation':'admission'} must not change URL`);
  assert.match(find('[data-identity-status]').textContent,/unavailable/);
  assert.doesNotMatch(find('[data-identity-status]').textContent,/existing sample opened/);
  restore();flush();
}
const audit=find('#resource-audit-console');
for(const member of [evidence,audit])for(const field of ['data-ae-source-available','data-ae-available'])for(const value of ['false',null,'invalid'])for(const route of ['evidence','audit'])for(const late of [false,true]) {
  const before=member.getAttribute(field);refusedRoute(route,()=>value===null?member.removeAttribute(field):member.setAttribute(field,value),()=>member.setAttribute(field,before),late);
}
for(const member of [evidence,audit]) {
  const parent=member.parentElement,next=member.nextElementSibling;
  for(const route of ['evidence','audit'])for(const late of [false,true])refusedRoute(route,()=>member.remove(),()=>next?next.before(member):parent.append(member),late);
  for(const field of ['data-ae-source','data-ae-context','data-ae-label','data-ae-hub-source','data-ae-mode']) {
    const before=member.getAttribute(field);
    const values=field==='data-ae-source'?[null,'invalid',JSON.stringify([...JSON.parse(before).slice(0,3),'changed linked source',...JSON.parse(before).slice(4)])]:['mismatched'];
    for(const value of values)for(const route of ['evidence','audit'])for(const late of [false,true])refusedRoute(route,()=>value===null?member.removeAttribute(field):member.setAttribute(field,value),()=>member.setAttribute(field,before),late);
  }
}
// Independent hiding/inertness cannot become an opened outcome, including a change after admission.
for(const target of [evidence,find('#resource-panel-audit')])for(const field of ['hidden','inert','aria-hidden'])for(const late of [false,true]) {
  refusedRoute(target===evidence?'evidence':'audit',()=>target.setAttribute(field,field==='aria-hidden'?'true':''),()=>target.removeAttribute(field),late);
}
const evidenceWrapper=new Element('div');evidence.before(evidenceWrapper);evidenceWrapper.append(evidence);
for(const field of ['hidden','inert'])for(const late of [false,true])refusedRoute('evidence',()=>evidenceWrapper.setAttribute(field,''),()=>evidenceWrapper.removeAttribute(field),late);
evidenceWrapper.before(evidence);evidenceWrapper.remove();
for(const route of ['evidence','audit']) {click(find(`[data-identity-anchor-action="route-${route}"]`));flush();assert.match(find('[data-identity-status]').textContent,/existing sample opened/);assert.ok(find(route==='evidence'?'#evidence-spine':'#resource-panel-audit').getClientRects().length);}
const duplicate=evidence.cloneNode(true);evidence.parentElement.append(duplicate);const duplicateURL=location.href;click(find('[data-identity-anchor-action="route-evidence"]'));flush();assert.equal(location.href,duplicateURL);duplicate.remove();
// A missing native target and newer trusted intent both cancel an in-flight route.
selectRail('auth');const target=find('#workflow-studio');target.remove();click(find('[data-identity-route-action="workflow"]'));flush();assert.match(find('[data-identity-status]').textContent,/unavailable/);
const url=location.href;click(find('[data-identity-route-action="evidence"]'));emit('pointerdown',{target:tenant,isTrusted:true});flush();assert.equal(location.href,url);
for(const name of ['popstate','blur']) {click(find('[data-identity-route-action="evidence"]'));for(const callback of windowEvents[name]||[])callback({isTrusted:true});flush();assert.equal(location.href,url,'newer history/background intent cancels route');}
const saved=find('[data-account-focus="setup-advance"]');selectRail('onboarding');saved.focus();const replacement=changed.cloneNode(true);changed.replaceWith(replacement);document.activeElement=tenant;flush();assert.equal(document.activeElement,tenant,'persistent IAM input focus wins remount');assert.equal(tenant.value,'real-tenant');
const stale=changed.querySelector('[data-account-route="tasks"]');const staleURL=location.href;assert.equal(click(stale).defaultPrevented,true);assert.equal(location.href,staleURL,'detached authored anchor refuses default navigation');
console.log('Settings/account native source controls, progress, tabs, complete channels, refusal and lifetime PASS');
"##;

#[test]
fn settings_account_native_source_controls_refusal_and_lifetime() {
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let input = serde_json::json!({"html": html, "tabs": crate::app::ssr_tab_navigation_script(), "bridge": super::SCRIPT, "hr": super::super::hr_directory::SCRIPT});
    let mut child = Command::new("node")
        .args(["-e", CHECK])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Node runs source bridge proof");
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
