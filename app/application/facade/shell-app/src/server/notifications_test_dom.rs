pub(super) const SCRIPT: &str = r##"
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
  getClientRects() { const panel=this.closest('.identity-panel, .finance-panel, .cockpit-panel, .resource-panel'), dialog=this.closest('dialog');return this.isConnected&&!this.closest('[data-shell-inactive]')&&!this.closest('[hidden], [inert], [aria-hidden="true"]')&&!(dialog&&!dialog.open)&&!(panel&&!panel.classList.contains('active'))?[{}]:[]; }
  dispatchEvent(event) { emit(event.type,{...event,target:this});return true; }
  focus() { if(this.disabled||!this.getClientRects().length)return;document.activeElement=this;emit('focusin',{target:this}); }
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
const context={Event:class{constructor(type){this.type=type;}},document,location,Element,window,queueMicrotask:callback=>microtasks.push(callback),requestAnimationFrame:callback=>frames.push(callback),
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
"##;
