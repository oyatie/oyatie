pub(super) const SCRIPT: &str = r#"
const assert = require('node:assert/strict'), vm = require('node:vm');
const input = JSON.parse(require('node:fs').readFileSync(0, 'utf8'));
class Element {
  constructor(tag='div') { this.tagName=tag; this.textContent=''; this.children=[]; this.dataset={}; this.events={}; this.attrs={}; this.hidden=false; this._value=''; this.isConnected=true; this.style={setProperty(name,value){this[name]=String(value);},removeProperty(name){delete this[name];},getPropertyValue(name){return this[name]||'';}};this.classList={add(){},contains(){return false;}};this.clientWidth=640;this.clientHeight=480;this.scrollLeft=0;this.scrollTop=0; if(['input','select','button'].includes(tag))this.disabled=false; if(tag==='textarea')Object.defineProperty(this,'type',{get:()=> 'textarea'}); }
  append(...children) { for (const child of children) { if(child.parentElement)child.parentElement.children=child.parentElement.children.filter(node=>node!==child); child.parentElement=this; const connect=node=>{node.isConnected=this.isConnected;for(const nested of node.children)connect(nested);};connect(child);this.children.push(child); } }
  removeChild(child) { if(!this.children.includes(child))throw new Error('Not a child');if(child.contains(document.activeElement))document.activeElement=document.body;this.children=this.children.filter(node=>node!==child);child.parentElement=null;const disconnect=node=>{node.isConnected=false;for(const nested of node.children)disconnect(nested);};disconnect(child);return child; }
  replaceChildren(...children) { for(const child of [...this.children])this.removeChild(child);this.append(...children); }
  setAttribute(name,value) { this.attrs[name]=String(value); }
  getAttribute(name) { return this.attrs[name] ?? null; }
  hasAttribute(name) { return Object.hasOwn(this.attrs,name); }
  removeAttribute(name) { delete this.attrs[name]; }
  dispatchEvent(event) { document.emit(event.type,{...event,target:this});return true; }
  get value() { return this.tagName==='select' ? (this.children.some(c=>c.value===this._value) ? this._value : this.children[0]?.value || '') : this._value; }
  set value(value) { this._value=String(value); }
  get selectedOptions() { return this.children.filter(c=>c.value===this.value); }
  get disabled() { return this._disabled === true; }
  set disabled(value) { this._disabled=Boolean(value);if(this._disabled&&document.activeElement===this)document.activeElement=document.body; }
  addEventListener(name,action) { (this.events[name] ||= []).push(action); }
  emit(name,values={}) { for (const action of this.events[name] || []) action({target:this,preventDefault(){},...values}); }
  scrollBy({left=0,top=0}) {this.scrollLeft+=left;this.scrollTop+=top;}
  get offsetLeft() {return Number.parseFloat(this.style.left)||0;}
  get offsetTop() {return Number.parseFloat(this.style.top)||0;}
  get offsetWidth() {return this.mockWidth || (this.className==='foundry-type-node'?182:this.className==='foundry-edge-label'?104:0);}
  get offsetHeight() {return this.mockHeight || (this.className==='foundry-type-node'?68:this.className==='foundry-edge-label'?44:0);}
  getBBox() {if(this.mockBounds)return this.mockBounds;const n=(this.attrs.d||'').match(/-?(?:\d*\.)?\d+/g)?.map(Number)||[],xs=n.filter((_,i)=>i%2===0),ys=n.filter((_,i)=>i%2===1);return n.length?{x:Math.min(...xs),y:Math.min(...ys),width:Math.max(...xs)-Math.min(...xs),height:Math.max(...ys)-Math.min(...ys)}:{x:0,y:0,width:0,height:0};}
  getBoundingClientRect() {return {height:this.mockHeight??0,width:this.clientWidth};}
  contains(target) { for(let node=target;node;node=node.parentElement) if(node===this)return true;return false; }
  closest(selector) { for(let node=this;node;node=node.parentElement)if(selector==='[data-live-results]'&&node.dataset.liveResults||selector==='[hidden]'&&node.hidden||selector==='[data-shell-inactive]'&&node.hasAttribute('data-shell-inactive')||selector==='details'&&node.tagName==='details'||selector==="a[href^='#']"&&node.tagName==='a'&&node.getAttribute('href')?.startsWith('#'))return node;return null; }
  getClientRects() { if(!this.isConnected || this.closest('[hidden]') || this.closest('[data-shell-inactive]'))return [];for(let child=this,parent=this.parentElement;parent;child=parent,parent=parent.parentElement)if(parent.tagName==='details'&&!parent.open&&child!==parent.children.find(node=>node.tagName==='summary'))return [];return [{}]; }
  querySelector(selector) { return this.querySelectorAll(selector)[0]||null; }
  querySelectorAll(selector) { const direct=selector.match(/^:scope > \.([\w-]+)$/);if(direct)return this.children.filter(child=>child.getAttribute('class')?.split(' ').includes(direct[1]));const result=[];for(const child of this.children){if(selector===child.tagName||selector==='[data-live-object]'&&child.dataset.liveObject||selector==='[data-live-fact]'&&child.dataset.liveFact||selector==='[data-model-type]'&&child.dataset.modelType!==undefined||selector==='[data-model-value]'&&child.dataset.modelValue!==undefined||selector==='[data-model-link]'&&child.dataset.modelLink!==undefined||selector==='[data-model-edge]'&&child.dataset.modelEdge)result.push(child);result.push(...child.querySelectorAll(selector));}return result; }
  focus(options) { if(this.disabled || !this.getClientRects().length)return;this.focusOptions=options;document.activeElement=this;document.emit('focusin',{target:this}); }
}
const elements={}, panel=new Element('section');
function add(id,tag='div',parent=panel) { const e=new Element(tag);e.id=id;elements[id]=e;parent.append(e);return e; }
const shell=add('oyatie-shell','div');
const shellHeader=add('test-shell-header','header',shell);shellHeader.setAttribute('class','app-header');shellHeader.mockPosition='sticky';shellHeader.mockHeight=44;
const main=add('console-shell','main',shell);
add('shell-sample-intro','section',main);
add('dashboard-island-root','div',main);
add('oyatie-sign-in','section',main);
add('foundry-workspace','section',main);
add('shell-skip-link','a',shell).setAttribute('href','#console-shell');
add('shell-navigation-menu','details',shell).open=true;
add('service-catalog','section',elements['dashboard-island-root']);
for(const [id,label] of Object.entries({'shell-rail-product':'Control Center','shell-rail-scope':'ANONYMOUS SAMPLE VIEWS','shell-rail-title':'Explore shell samples','shell-rail-description':'Catalog, cloud, evidence and Work Hub samples. Live operations are unavailable.','shell-rail-status':'Navigation only · no live operation','shell-account-title':'Sample workspace','shell-account-detail':'Anonymous preview','shell-breadcrumb-scope':'Sample views','shell-breadcrumb-destination':'Control Center','shell-session-entry':'Sign in','shell-header-identity':'Anonymous preview','sign-in-title':'Sign in with a passkey'}))add(id).textContent=label;
for(const id of ['tenant','account'])add('sign-in-'+id,'input',elements['oyatie-sign-in']);
for(const id of ['status'])add('sign-in-'+id);
for(const id of ['passkey-sign-in','session-restore','passkey-enroll','session-logout','passkey-cancel','ontology-refresh'])add(id,'button');
for(const id of ['passkey-form','passkey-invitation','verified-ontology-identity','verified-ontology-updated'])add(id,'div',elements['oyatie-sign-in']);
const card=add('verified-ontology-card');for(const key of ['policyVersion','servedTenants','projectionLag','poisonedEntries']){const e=new Element();e.dataset.liveFact=key;card.append(e);}
const root=add('live-ontology-explorer','section',elements['foundry-workspace']);root.hidden=true;
for(const id of ['identity','query','status','page','detail','detail-title','properties','history','audit','audit-rows'])add('live-ontology-'+id,'div',root);
for(const id of ['access-badge','access-notice','model-note','model','model-summary','library-summary','library-types','library-links','model-table','graph-help','graph-controls','zoom','inspector-empty'])add('live-ontology-'+id,'div',root);
elements['live-ontology-access-badge'].textContent='Sign-in required';
const graph=add('live-ontology-graph','div',root),stage=add('live-ontology-stage','div',graph);add('live-ontology-scene','div',stage);
const link=add('live-ontology-link','section',root);link.hidden=true;add('live-ontology-link-properties','div',link);
add('live-ontology-objects','section',root);add('live-ontology-search','input',root);
for(const id of ['mode-schema','mode-objects','view-graph','view-table','zoom-out','zoom-in','fit'])add('live-ontology-'+id,'button',root);
const schema=add('live-ontology-schema','section',root);schema.hidden=true;
for(const id of ['schema-title','schema-summary','schema-properties'])add('live-ontology-'+id,'div',schema);
for(const id of ['type','comparison','kind'])add('live-ontology-'+id,'select',root);
for(const id of ['revision','limit','property','from','to'])add('live-ontology-'+id,'input',root);
for(const id of ['read','types-refresh','audit-read','history-read','next','first'])add('live-ontology-'+id,'button',root);
elements['live-ontology-objects'].append(...['query','page','detail','audit','first','next'].map(id=>elements['live-ontology-'+id]));
elements['live-ontology-detail'].append(...['detail-title','properties','history','history-read'].map(id=>elements['live-ontology-'+id]));
for(const id of ['relationship-root','relationship-status','relationship-results'])add('live-ontology-'+id,'div',elements['live-ontology-detail']);
for(const id of ['relationship-direction','relationship-type'])add('live-ontology-'+id,'select',elements['live-ontology-detail']);
add('live-ontology-relationship-read','button',elements['live-ontology-detail'])._disabled=true;
for(const value of ['outbound','inbound']){const option=new Element('option');option.value=value;elements['live-ontology-relationship-direction'].append(option);}
elements['live-ontology-audit'].append(elements['live-ontology-audit-rows']);
elements['live-ontology-model'].append(...['graph','graph-help','graph-controls','model-table'].map(id=>elements['live-ontology-'+id]));
for(const id of ['page','detail','audit','schema'])elements['live-ontology-'+id].dataset.liveResults='true';
for(const [id,values] of [['comparison',['none','equals','range']],['kind',['text','int','bool']]])for(const value of values){const option=new Element('option');option.value=value;elements['live-ontology-'+id].append(option);}
add('live-ontology-object-type-controls','span',root).append(elements['live-ontology-type']);
elements['live-ontology-revision'].value='1';elements['live-ontology-limit'].value='25';
for(const id of ['draft-controls','draft-inspector','draft-tray','draft-state','draft-status','draft-selection','draft-diff','draft-diagnostics','property-compatibility','property-value','new-property-value','entity-key-help'])add('live-ontology-'+id,'div',root);
for(const id of ['draft-choice','entity-primary','entity-title','new-entity-key-kind','new-entity-title','property-choice','property-kind','property-required','new-property-kind','new-property-required','link-from','link-to','link-cardinality','new-link-from','new-link-to','new-link-cardinality'])add('live-ontology-'+id,'select',root);
for(const id of ['entity-name','entity-description','entity-plural','new-entity-key','property-display','entity-revision','property-name','new-property-name','link-revision','new-entity-id','new-entity-name','new-link-id'])add('live-ontology-'+id,'input',root);
for(const id of ['draft-more','draft-new','draft-save','draft-reload','draft-validate'])add('live-ontology-'+id,'button',root);
for(const id of ['entity-edit','property-edit','property-add','link-edit','entity-add','link-add']){const form=add('live-ontology-'+id,'form',findEditorParent());form.append(new Element('button'));}
function findEditorParent(){return elements['live-ontology-draft-inspector'];}
add('live-ontology-draft-inspector-title','h4',findEditorParent()).setAttribute('tabindex','-1');findEditorParent().append(elements['live-ontology-property-choice']);elements['live-ontology-property-choice'].hidden=true;
const propertyOverview=add('live-ontology-property-overview','section',findEditorParent());add('live-ontology-property-search','input',propertyOverview);add('live-ontology-property-count','p',propertyOverview);add('live-ontology-property-rows','div',propertyOverview);
add('live-ontology-property-restore','button',elements['live-ontology-property-edit']);
for(const id of ['property-default','new-property-default'])add('live-ontology-'+id,'div',findEditorParent());
for(const [form,fields] of [['entity-edit',['entity-name','entity-primary','entity-title','entity-description','entity-plural','entity-revision']],['property-edit',['property-name','property-kind','property-required','property-display','property-value','property-default']],['property-add',['new-property-name','new-property-kind','new-property-required','new-property-value','new-property-default']],['link-edit',['link-from','link-to','link-cardinality','link-revision']],['entity-add',['new-entity-id','new-entity-name','new-entity-key','new-entity-key-kind','new-entity-title']],['link-add',['new-link-id','new-link-from','new-link-to','new-link-cardinality']]])elements['live-ontology-'+form].append(...fields.map(id=>elements['live-ontology-'+id]));
const libraryPane=add('live-ontology-library','details',root);libraryPane.open=true;add('live-ontology-library-title','summary',libraryPane);libraryPane.append(elements['live-ontology-search'],elements['live-ontology-library-summary']);
for(const [kind,label,id]of [['type','Object types','library-types'],['link','Relationships','library-links'],['value','Reusable string types','library-values']]){const group=add(`live-ontology-library-${kind}-group`,'details',libraryPane);group.open=kind==='type';add(`live-ontology-library-${kind}-heading`,'summary',group).textContent=label;if(kind==='type')add('live-ontology-library-scope','p',group);if(kind==='value')add('live-ontology-values-note','p',group);if(elements['live-ontology-'+id])group.append(elements['live-ontology-'+id]);else add('live-ontology-'+id,'div',group);}
const libraryNew=add('live-ontology-library-new','details',libraryPane);add('live-ontology-library-new-summary','summary',libraryNew);for(const kind of ['type','link'])add('live-ontology-library-new-'+kind,'button',libraryNew);add('live-ontology-library-new-status','p',libraryNew);

const workpane=add('live-ontology-workpane','div',root);workpane.append(elements['live-ontology-model'],elements['live-ontology-objects']);
const inspectorPane=add('live-ontology-inspector','details',root);inspectorPane.open=true;const inspectorTitle=add('live-ontology-inspector-title','summary',inspectorPane),inspectorContext=add('live-ontology-inspector-context','span',inspectorTitle);for(const id of ['selected','identity','name'])add('live-ontology-context-'+id,'span',inspectorContext);const selectionDetails=add('live-ontology-selection-details','details',inspectorPane);add('live-ontology-selection-values','dl',selectionDetails);inspectorPane.append(elements['live-ontology-draft-inspector'],schema,link);
const paneNavigation=add('live-ontology-pane-navigation','nav',root);paneNavigation.mockHeight=58;
for(const id of ['library','workpane','inspector'])add('live-ontology-pane-'+id,'a',paneNavigation);
const moduleStrip=add('live-ontology-module-strip','div',root);moduleStrip.mockHeight=58;moduleStrip.mockPosition='sticky';moduleStrip.append(paneNavigation,elements['live-ontology-status'],elements['live-ontology-object-type-controls']);
const contextHeader=add('test-workspace-header','header',elements['foundry-workspace']);contextHeader.setAttribute('class','foundry-workspace-header');contextHeader.mockHeight=58;contextHeader.append(elements['live-ontology-identity'],elements['live-ontology-access-badge']);
const canvasToolbar=add('test-canvas-toolbar','div',elements['live-ontology-model']);canvasToolbar.setAttribute('class','foundry-canvas-toolbar');canvasToolbar.mockHeight=60;canvasToolbar.append(elements['live-ontology-graph-controls']);
const tray=elements['live-ontology-draft-tray'],actions=add('test-draft-actions','div',tray);actions.setAttribute('class','foundry-draft-actions');actions.mockHeight=70;tray.mockHeight=160;tray.mockMinHeight='201.5px';actions.append(elements['live-ontology-draft-controls']);
elements['live-ontology-draft-controls'].append(...['draft-choice','draft-more','draft-new','draft-save','draft-reload','draft-validate','draft-state'].map(id=>elements['live-ontology-'+id]));tray.append(...['draft-status','draft-diff','draft-diagnostics'].map(id=>elements['live-ontology-'+id]));
const review=add('live-ontology-draft-review','div',tray);review.mockMinHeight='52px';review.append(elements['live-ontology-draft-diff'],elements['live-ontology-draft-diagnostics']);
const body=add('test-workspace-body','div',root);body.append(libraryPane,workpane,inspectorPane,tray);elements['live-ontology-model'].append(elements['live-ontology-model-note']);elements['live-ontology-model-note'].mockHeight=30;
for(const [form,id] of [['entity-edit','type-settings'],['property-add','add-property']]){const parent=elements['live-ontology-'+form],details=add('live-ontology-'+id,'details',parent);details.open=false;details.append(...parent.children.filter(child=>child!==details));}
for(const [id,values] of [['new-entity-key-kind',['string','integer','double','boolean','date','timestamp']],['new-entity-title',['none','primary']],['property-kind',['legacy','string','integer','double','boolean','date','timestamp','array','struct','named_string']],['property-required',['required','optional']],['new-property-kind',['string','integer','double','boolean','date','timestamp','array','struct','named_string']],['new-property-required',['optional','required']],['link-cardinality',['one_to_one','one_to_many','many_to_many']],['new-link-cardinality',['one_to_many','one_to_one','many_to_many']]])for(const value of values){const option=new Element('option');option.value=value;elements['live-ontology-'+id].append(option);}
const document={activeElement:panel,body:panel,visibilityState:'visible',events:{},getElementById:id=>{const visit=node=>node.id===id?node:node.children.map(visit).find(Boolean);return visit(panel) || null;},createElement:tag=>new Element(tag),createElementNS:(namespace,tag)=>{const e=new Element(tag);e.namespaceURI=namespace;return e;},addEventListener(name,action){(this.events[name] ||= []).push(action);},emit(name,values={}){for(const action of this.events[name] || [])action(values);}};
let intervals=[], timers=[], calls=[], held=null, holdPath=null, nativeError=null, statusError=null, modelEnabled=false, modelError=null, modelBaseError=null, modelBaseTamper=null, modelBaseMalformed=false, modelLoseAck=false, modelTamper=false, modelValidation=null, headerUser='usr_alice', pageEmpty=false;
const session={tenantId:'ten_acme',userId:'usr_alice',expiresAt:Math.floor(Date.now()/1000)+3600,roles:['foundry-operator']};
const object={object_ref:'ent_alpha',written_revision:1,upcast_state:'current',properties:{name:{value_type:'string',data_class:'INTERNAL_ONLY',value:'<script>Ada</script>'}}};
const schemaFixture=[
  {entity_type:'ety_record',revision:1,properties:[{name:'name',tier:'scalar',data_class:'INTERNAL_ONLY',required:true,value_type:null},{name:'note',tier:'scalar',data_class:'INTERNAL_ONLY',required:false,value_type:null}],primary_key_property:null,title_property:'name'},
  {entity_type:'ety_profile',revision:3,properties:[{name:'id',tier:'scalar',data_class:'PUBLIC',required:true,value_type:{kind:'integer'}},{name:'<script>label</script>',tier:'scalar',data_class:'PII_IDENTIFYING',required:false,value_type:{kind:'string'}},{name:'profile',tier:'struct',data_class:'INTERNAL_ONLY',required:false,value_type:{kind:'struct',fields:[{name:'flags',required:true,value_type:{kind:'array',element:{kind:'boolean'}}}]}}],primary_key_property:'id',title_property:'<script>label</script>'},
];
const labelFixture=[{entity_type:'ety_record',revision:1,display_name:'Record'}, {entity_type:'ety_profile',revision:3,display_name:'직원 <img src=x onerror=alert(1)>'}];
let nativeSchemas=schemaFixture, nativeLinks=[], nativeLabels=null, nativeGraph=null, nativeDetails=new Map();
function payload(url,options={}) {
  if(url.endsWith('/search-around'))return nativeGraph && {...nativeGraph,observed_at_epoch_seconds:JSON.parse(options.body).observed_at_epoch_seconds};
  if(url.endsWith('identity-config'))return {enabled:true};
  if(url.endsWith('/session'))return session;
  if(url.includes('ontology-status'))return {policyVersion:'p',servedTenants:1,projectionLag:0,poisonedEntries:0};
  if(url.endsWith('/types'))return [{entity_type:'ety_record',revision:1,properties:['name']}];
  if(url.endsWith('/schemas'))return nativeSchemas;
  if(url.endsWith('/type-labels'))return nativeLabels ?? labelFixture.filter(label=>nativeSchemas.some(schema=>schema.entity_type===label.entity_type)).map(label=>({...label,revision:nativeSchemas.find(schema=>schema.entity_type===label.entity_type).revision}));
  if(url.endsWith('/link-types'))return nativeLinks;
  if(url.includes('/objects?'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),objects:pageEmpty?[]:[object],next:null};
  if(url.endsWith('/history'))return [{ordinal:1,object_sequence:1,principal_id:'usr_alice',decision_id:'dec_native',action_type:'aty_record_write',audit_event_type:'record.written',schema_revision:1}];
  if(url.endsWith('/audit'))return [{ordinal:1,object_ref:'ent_alpha',action_type:'aty_record_write',disposition:'applied',poison_reason:null,principal_id:'usr_alice',decision_id:'dec_native'}];
  if(url.includes('/objects/'))return {requested_revision:Number(new URL('https://shell.invalid'+url).searchParams.get('revision')),object:nativeDetails.get(decodeURIComponent(new URL('https://shell.invalid'+url).pathname.split('/').at(-1))) || object};
  return null;
}
const modelDocument={entities:[{id:'ety_record',display_name:'Record',properties:[{name:'name',tier:'scalar',data_class:'InternalOnly',required:true,value_type:null,display:null},{name:'structure',tier:'struct',data_class:'InternalOnly',required:false,value_type:{shape:'struct',fields:[{name:'date',required:true,value_type:{shape:'scalar',kind:'date'}}]},display:{display_name:'Untouched',description:null,icon:'native',color:null,plural_name:null}}],revision:1,pillar:null,primary_key_property:null,title_property:'name',display:{display_name:'Record',description:'Preserve this',icon:null,color:null,plural_name:null}}],links:[]};
const modelWorkspace={generation:'9007199254740993',document:modelDocument,capabilities:{author:true,publish:false}}, modelDrafts=new Map(), modelHistory=new Map();let nextDraft=9007199254740993n;
function modelReply(url,options){
  let body=null,code=modelEnabled?modelError:'forbidden';if(code)return {ok:false,status:code==='unauthenticated'?401:code==='model_unavailable'?503:403,json:async()=>({error:code})};
  const path=new URL('https://shell.invalid'+url).pathname, payload=options.body?JSON.parse(options.body):null;
  if(path.endsWith('/model')){if(!modelHistory.has(modelWorkspace.generation))modelHistory.set(modelWorkspace.generation,structuredClone(modelWorkspace.document));body=modelWorkspace;}
  else if(path.endsWith('/drafts') && options.method==='POST'){const id=String(nextDraft++);body={id,tenant_id:session.tenantId,owner:session.userId,base_generation:payload.expected_generation,version:'1',document:payload.document,publication:null};modelDrafts.set(id,structuredClone(body));}
  else if(path.endsWith('/drafts'))body={drafts:[...modelDrafts.values()].filter(d=>d.owner===session.userId).map(d=>({id:d.id,version:d.version,base_generation:d.base_generation,published_generation:null})),next:null};
  else {const parts=path.split('/'),id=['save','validate','base'].includes(parts.at(-1))?parts.at(-2):parts.at(-1),draft=modelDrafts.get(id);
    if(!draft || draft.owner!==session.userId)return {ok:false,status:404,json:async()=>({error:'draft_not_found'})};
    if(payload && (payload.expected_version!==draft.version || payload.expected_base!==draft.base_generation))return {ok:false,status:409,json:async()=>({error:'draft_conflict'})};
    if(path.endsWith('/base')){if(modelBaseError)return {ok:false,status:modelBaseError==='unauthenticated'?401:modelBaseError==='forbidden'?403:503,json:async()=>{if(modelBaseMalformed)throw new Error('invalid JSON');return {error:modelBaseError};}};const query=new URL('https://shell.invalid'+url).searchParams;if(query.get('expected_version')!==draft.version || query.get('expected_base')!==draft.base_generation)return {ok:false,status:409,json:async()=>({error:'draft_conflict'})};const base=modelHistory.get(draft.base_generation);if(!base)return {ok:false,status:503,json:async()=>({error:'model_unavailable'})};body={draft_id:id,draft_version:draft.version,base_generation:draft.base_generation,document:base,...(modelBaseTamper || {})};}
    else if(path.endsWith('/save')){draft.version=String(BigInt(draft.version)+1n);draft.document=structuredClone(payload.document);body=draft;}
    else if(path.endsWith('/validate'))body={draft_id:id,draft_version:draft.version,base_generation:draft.base_generation,valid:true,diagnostics:[],...(modelValidation || {})};
    else body=draft;
  }
  if(options.method==='POST' && modelLoseAck){modelLoseAck=false;throw new Error('network lost after native commit');}
  body=structuredClone(body);if(modelTamper && options.method==='POST' && body.id){modelTamper=false;body.version='9007199254740995';}
  return {ok:true,status:options.method==='POST' && path.endsWith('/drafts')?201:200,headers:{get:name=>({'x-oyatie-user':headerUser,'x-oyatie-tenant':session.tenantId,'x-oyatie-session-expires':String(session.expiresAt)})[name]??null},json:async()=>body};
}
function reply(url,error=nativeError,options={}) { const isRead=url.includes('/api/ontology/');if(url.includes('ontology-status') && statusError)return {ok:false,status:statusError,json:async()=>({error:'forbidden'})};return {ok:!isRead||!error,status:url.endsWith('session:logout')?204:isRead&&error?403:200,headers:{get:name=>({'x-oyatie-user':headerUser,'x-oyatie-tenant':session.tenantId,'x-oyatie-session-expires':String(session.expiresAt)})[name]??null},json:async()=>isRead&&error?{error}:payload(url,options)}; }
const resizeObservers=[];
class ResizeObserver { constructor(callback){this.callback=callback;resizeObservers.push(this);}observe(target){this.target??=target;(this.targets??=new Set()).add(target);this.observing=true;}disconnect(){this.observing=false;this.targets?.clear();}emit(){this.callback([{target:this.target}]);} }
const context={Event:class {constructor(type){this.type=type;}},ResizeObserver,crypto:require('node:crypto').webcrypto,document,Element,URLSearchParams,AbortController,TextEncoder,location:{hash:'',hostname:'shell.invalid'},getComputedStyle:element=>({strokeWidth:String(element.mockStrokeWidth||1.5),fontSize:'13px',position:element.mockPosition||'static',minHeight:element.mockMinHeight||'312px',paddingTop:'10px',paddingBottom:'10px',marginTop:'0px',marginBottom:'0px'}),window:{innerHeight:941,isSecureContext:true,PublicKeyCredential:function(){},events:{},addEventListener(name,action){(this.events[name] ||= []).push(action);},emit(name){for(const action of this.events[name] || [])action();}},navigator:{credentials:{},locks:{request:(_,action)=>action()}},fetch:(url,options)=>{calls.push([url,options]);if(url.includes('ontology-status') && statusError==='network')return Promise.reject(new Error('status transport unavailable'));if(url.includes('/model')){if(holdPath && url.includes(holdPath)){holdPath=null;return new Promise(resolve=>{const captured=modelReply(url,options);held=()=>resolve(captured);});}try{return Promise.resolve(modelReply(url,options));}catch(e){return Promise.reject(e);}}if(holdPath&&url.includes(holdPath)){holdPath=null;return new Promise(resolve=>{held=()=>resolve(reply(url,nativeError,options));});}return Promise.resolve(reply(url,nativeError,options));},setTimeout:callback=>{timers.push(callback);return timers.length;},clearTimeout(){},setInterval:callback=>intervals.push(callback)};
vm.runInNewContext("'use strict';\n"+input.script,context,{timeout:1000});
const find=id=>document.getElementById(id), text=node=>(node.textContent||'')+node.children.map(text).join(' ');
async function flush() { for(let i=0;i<80;i++)await Promise.resolve(); }
async function restore() { find('session-restore').emit('click');await flush(); }
"#;
