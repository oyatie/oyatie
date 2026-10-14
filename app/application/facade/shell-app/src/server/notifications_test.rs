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
  querySelector(selector) {
    const visit=node=>{for(const child of node.children){if(child.matches(selector))return child;const found=visit(child);if(found)return found;}return null;};
    return visit(this);
  }
  cloneNode() { const copy=new Element(this.tagName,{...this.attrs});for(const part of this.parts)typeof part==='string'?copy.parts.push(part):copy.append(part.cloneNode(true));if(this.content)copy.content=this.content.cloneNode(true);return copy; }
  getClientRects() { const panel=this.closest('.identity-panel, .finance-panel, .cockpit-panel, .resource-panel'), dialog=this.closest('dialog');return this.isConnected&&!this.closest('[hidden], [inert], [aria-hidden="true"]')&&!(dialog&&!dialog.open)&&!(panel&&!panel.classList.contains('active'))?[{}]:[]; }
  focus() { document.activeElement=this;emit('focusin',{target:this}); }
  scrollIntoView() { this.scrolled=true; }
  close() { this.open=false; }
  click() { click(this); }
  addEventListener(name,callback) { (this.events[name] ||= []).push(callback); }
  removeEventListener(name,callback) { this.events[name]=(this.events[name]||[]).filter(item=>item!==callback); }
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
function firstLiteralId(node,id) {
  for(const child of node.parts) {
    if(!(child instanceof Element))continue;
    if(child.hasAttribute('id')&&child.id===id)return child;
    const found=firstLiteralId(child,id);if(found)return found;
  }
  return null;
}
const document={body:tree,activeElement:tree,hasFocus:()=>focused,getElementById:id=>id===''?null:firstLiteralId(tree,id),querySelectorAll:selector=>tree.querySelectorAll(selector),querySelector:selector=>tree.querySelector(selector),createElement:tag=>new Element(tag),createComment:data=>new Comment(data),createTreeWalker:(root,whatToShow)=>{
  assert.equal(whatToShow,NodeFilter.SHOW_COMMENT);
  const comments=[],visit=node=>{for(const child of node.parts||[]){if(child instanceof Comment)comments.push(child);else if(child instanceof Element)visit(child);}};visit(root);
  let index=0;return {nextNode:()=>comments[index++]||null};
},addEventListener:(name,callback)=>(events[name] ||= []).push(callback),removeEventListener:(name,callback)=>{events[name]=(events[name]||[]).filter(item=>item!==callback);}};
(() => {
  const root=tree.append(new Element('div',{class:'fixture-first-match'})), branch=root.append(new Element('section'));
  const first=branch.append(new Element('span',{class:'fixture-first-match',id:'fixture.[literal]#id'}));
  const later=first.append(new Element('span')), second=root.append(new Element('span',{class:'fixture-first-match fixture-last-match',id:first.id}));
  const selector='.fixture-last-match, .fixture-first-match';
  assert.equal(root.querySelector(selector),first,'first descendant follows tree order, not selector order');
  assert.deepEqual(root.querySelectorAll(selector),[first,second],'all-match lookup retains duplicate descendants and excludes receiver');
  later.matches=second.matches=()=>assert.fail('first-match lookup must stop before later descendants');
  assert.equal(root.querySelector(selector),first);delete later.matches;delete second.matches;
  assert.equal(first.querySelector(selector),null,'receiver does not match itself');
  assert.equal(root.querySelector('.fixture-missing'),null,'missing descendant returns null');
  later.hasAttribute=second.hasAttribute=()=>assert.fail('literal ID lookup must stop before later descendants');
  assert.equal(document.getElementById(first.id),first,'literal duplicate ID follows tree order');delete later.hasAttribute;delete second.hasAttribute;
  assert.equal(document.getElementById(''),null);assert.equal(document.getElementById('fixture-missing'),null);
  first.remove();assert.equal(root.querySelector(selector),second);assert.equal(document.getElementById(second.id),second,'removal exposes next ID');
  root.remove();assert.equal(document.getElementById(second.id),null,'detached IDs are absent');
})();
const location=new URL('https://preview.invalid/');
const emit=(name,event)=>(events[name]||[]).forEach(callback=>callback(event));
function click(target,options={}) { const event={target,button:0,isTrusted:false,defaultPrevented:false,...options,preventDefault(){this.defaultPrevented=true;}};
  for(let node=target;node;node=node.parentElement)for(const callback of [...(node.events.click||[])])callback(event);
  if(target.isConnected)emit('click',event);return event; }
const window={addEventListener:(name,callback)=>(windowEvents[name] ||= []).push(callback),removeEventListener:(name,callback)=>{windowEvents[name]=(windowEvents[name]||[]).filter(item=>item!==callback);}};
const context={document,location,Element,window,queueMicrotask:callback=>microtasks.push(callback),requestAnimationFrame:callback=>frames.push(callback),
  MutationObserver:class{constructor(callback){this.callback=callback;observers.push(this);}observe(){}disconnect(){this.disconnected=true;}}};
const find=selector=>document.querySelector(selector), panel=()=>find('#notification-center');
const counts=()=>Object.fromEntries(panel().querySelectorAll('[data-activity-count]').map(node=>[node.dataset.activityCount,node.textContent]));
const flush=()=>{observers.filter(observer=>!observer.disconnected).forEach(observer=>observer.callback());
  for(let count=0;microtasks.length||frames.length;count++){assert.ok(count<20,'bounded synchronization');for(const callback of microtasks.splice(0))callback();for(const callback of frames.splice(0))callback();}};
const open=()=>{panel().open=true;};
const action=name=>find(`[data-activity-action="${name}"]`), filter=name=>find(`[data-activity-filter="${name}"]`), route=name=>find(`[data-notification-route="${name}"]`);
const read=id=>panel().querySelector(`[data-activity-item="${id}"] [data-activity-action="mark-read"]`);
const shown=()=>panel().querySelectorAll('[data-activity-item]').filter(node=>!node.hidden).map(node=>node.dataset.activityItem);
const picker=()=>find('[data-sample-context-requested]');
const initialDashboard=find('[data-island="render-envelope-dashboard"]').cloneNode(true);
assert.equal(panel().tagName,'DIALOG');assert.equal(panel().open,false);assert.equal(panel().getAttribute('closedby'),'any');
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
    let owner = Owner::new();
    let html = owner.with(|| super::super::document::ServerShell().into_view().to_html());
    owner.cleanup();
    let input = serde_json::json!({"html": html, "tabs": crate::app::ssr_tab_navigation_script(), "bridge": super::script()});
    let mut child = Command::new("node")
        .args(["-e", CHECK])
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
