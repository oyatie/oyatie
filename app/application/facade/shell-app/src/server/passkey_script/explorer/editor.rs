#[path = "editor_codec.rs"]
mod editor_codec;
#[path = "editor_defaults.rs"]
mod editor_defaults;
#[path = "editor_forms.rs"]
mod editor_forms;
#[path = "editor_values.rs"]
mod editor_values;
pub(super) fn script() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}",
        editor_defaults::SCRIPT,
        editor_codec::SCRIPT,
        editor_values::SCRIPT,
        editor_forms::SCRIPT,
        SCRIPT
    )
}
const SCRIPT: &str = r#"
function createOntologyDraftEditor(lifecycle, view) {
  const find=id=>document.getElementById(`live-ontology-${id}`), clone=v=>JSON.parse(JSON.stringify(v));
  let identity=null,generation=0,epoch=0,abort=null,workspace=null,draft=null,working=null,validation=null;
  let summaries=[],next=null,pending=null,blocked=false,loading=false,formDirty=false,mode='schema';
  let selectionFocus=null;
  const active=()=>Boolean(workspace && identity && lifecycle.valid(identity,generation));
  const valid=token=>token===epoch && identity && lifecycle.valid(identity,generation) && find('explorer').isConnected;
  const unsaved=()=>draft && working && !ontologyModelEqual(working,draft.document);
  const note=text=>{find('draft-status').textContent=text;};
  function cancelSelectionFocus() {selectionFocus=null;}
  document.addEventListener('focusin',event=>{if(event.target!==find('draft-choice'))cancelSelectionFocus();});
  document.addEventListener('pointerdown',cancelSelectionFocus);
  document.addEventListener('keydown',event=>{if(['Tab','Escape'].includes(event.key))cancelSelectionFocus();});
  document.addEventListener('visibilitychange',()=>{if(document.visibilityState!=='visible')cancelSelectionFocus();});
  window.addEventListener('hashchange',cancelSelectionFocus);
  window.addEventListener('blur',cancelSelectionFocus);
  function invalidate() {cancelSelectionFocus();epoch+=1;abort?.abort();abort=null;}
  function controls() {
    const ready=active() && mode==='schema' && !loading && !pending && !blocked;
    const author=ready && workspace.capabilities.author,validator=ready && (workspace.capabilities.author || workspace.capabilities.publish);
    find('object-type-controls').hidden=active() && mode==='schema';
    find('draft-controls').hidden=!active() || mode!=='schema';find('draft-tray').hidden=!active() || mode!=='schema';
    find('draft-choice').disabled=!active() || loading;find('draft-more').hidden=!next;find('draft-more').disabled=loading;
    find('draft-new').disabled=!author || Boolean(draft) || unsaved();find('draft-save').disabled=!author || !draft || draft.publication!==null || !unsaved() || formDirty;
    find('draft-validate').disabled=!validator || !draft || draft.publication!==null || unsaved() || formDirty;
    find('draft-reload').disabled=!active() || loading;
    find('draft-reload').textContent=pending ? 'Reconcile saved state' : unsaved() || formDirty ? 'Discard edits and reload saved' : 'Reload saved';
    find('draft-inspector').hidden=!active() || !draft || mode!=='schema';
    forms.enable(author && draft && draft.publication===null);
    find('draft-state').textContent=!active() ? '' : !draft ? `Published model · generation ${workspace.generation}` : `Draft ${draft.id} · ${pending ? 'Outcome unknown' : blocked ? 'Conflict — reload required' : formDirty ? 'Unsaved · Unapplied form edits' : unsaved() ? 'Unsaved' : 'Saved'} · saved version ${draft.version} · base generation ${draft.base_generation}${draft.publication ? ` · published as generation ${draft.publication.generation}` : ''}`;
  }
  function choices(selected=draft?.id || 'published') {
    const option=(value,label)=>{const o=document.createElement('option');o.value=value;o.textContent=label;return o;};
    const visible=draft && !summaries.some(s=>s.id===draft.id)?[...summaries,{id:draft.id,version:draft.version,base_generation:draft.base_generation,published_generation:draft.publication?.generation || null}]:summaries;
    find('draft-choice').replaceChildren(option('published',`Published · generation ${workspace.generation}`),...visible.map(s=>option(s.id,`Draft ${s.id} · saved v${s.version} · base ${s.base_generation}${s.published_generation ? ' · closed' : ''}`)));
    find('draft-choice').value=selected;
  }
  function diff() {
    find('draft-diff').replaceChildren();
    if (!working || !draft) return;
    const rows=[];
    function compare(a,b,path='document') {
      if (ontologyModelEqual(a,b)) return;
      if (a===null || b===null || typeof a!=='object' || typeof b!=='object' || Array.isArray(a)!==Array.isArray(b)) {rows.push([path,JSON.stringify(a),JSON.stringify(b)]);return;}
      for(const key of new Set([...Object.keys(a),...Object.keys(b)])) {
        if (!Object.hasOwn(a,key) || !Object.hasOwn(b,key)) rows.push([`${path}.${key}`,JSON.stringify(a[key]) ?? 'Absent',JSON.stringify(b[key]) ?? 'Absent']);
        else compare(a[key],b[key],`${path}.${key}`);
      }
    }
    compare(workspace.document,working);
    find('draft-diff').append(ontologyTable('Actual metadata changes from published model',['Field path','Published','Working draft'],rows));
  }
  function render(selected=null) {
    if (!active() || mode!=='schema') {controls();return;}
    forms.render(working,draft);const rows=ontologyModelRows(working || workspace.document);view.setModel(rows.schemas,rows.links,selected || forms.selection() || rows.schemas[0]?.entity_type || null,draft?'Draft':'Published model');
    find('model-note').textContent=draft ? `Working draft based on generation ${draft.base_generation}. Object reads use published declarations.` : `Coherent published metadata snapshot at generation ${workspace.generation}. Object reads are independent.`;
    if(forms.selectedLink())view.selectLink(forms.selectedLink());forms.describe();diff();controls();
  }
  const forms=createOntologyDraftForms({find,document:()=>working,baseline:()=>workspace?.document,editable:()=>mode==='schema' && active() && workspace.capabilities.author && draft && !draft.publication && !loading && !pending && !blocked,
    acceptInput:()=>mode==='schema' && active() && workspace.capabilities.author && draft && !draft.publication, dirty:()=>formDirty, input:()=>{forms.diagnostics([]);formDirty=true;validation=null;find('draft-diagnostics').replaceChildren();note('Unapplied form edits. Apply changes or discard and reload before saving or validating.');controls();},
    changed:unapplied=>{formDirty=unapplied;validation=null;find('draft-diagnostics').replaceChildren();note(unapplied?'Unapplied form edits remain. Apply changes or discard and reload before saving or validating.':'Unsaved local edits. Save the draft before schema validation.');render();},describe:value=>{if(active() && draft && mode==='schema')renderOntologySelectedContext(value);},note});
  function hide(preserve=false) {
    for(const id of ['draft-controls','draft-inspector','draft-tray'])find(id).hidden=true;
    find('model-note').textContent='';
    for(const id of ['draft-state','draft-status','draft-selection'])find(id).textContent='';
    for(const id of ['draft-diff','draft-diagnostics','draft-choice'])find(id).replaceChildren();
    if(preserve)forms.suspend();else forms.clear();
  }
  function clear() {invalidate();identity=null;workspace=null;draft=null;working=null;validation=null;summaries=[];next=null;pending=null;blocked=false;loading=false;formDirty=false;hide();}
  async function request(path, token, body=undefined) {
    const mutation=body!==undefined;
    let response;
    try {response=await fetch(`/api/ontology/${encodeURIComponent(identity.tenantId)}/model${path}`,{method:mutation?'POST':'GET',credentials:'same-origin',cache:'no-store',redirect:'error',signal:abort.signal,headers:mutation?{'content-type':'application/json'}:{},body:mutation?JSON.stringify(body):undefined});}
    catch(e) {if(mutation && valid(token))throw new Error('outcome_unknown');throw e;}
    if(!valid(token))return null;
    if(response.ok && (response.headers.get('x-oyatie-tenant')!==identity.tenantId || response.headers.get('x-oyatie-user')!==identity.userId || !Number.isSafeInteger(Number(response.headers.get('x-oyatie-session-expires'))) || Number(response.headers.get('x-oyatie-session-expires'))*1000<=Date.now()))throw new Error('unauthenticated');
    let bodyValue;try {bodyValue=await response.json();} catch {throw new Error(mutation?'outcome_unknown':'model_unavailable');}
    if(!valid(token))return null;
    if(!response.ok)throw new Error(typeof bodyValue?.error==='string'?bodyValue.error:mutation?'outcome_unknown':'model_unavailable');
    return bodyValue;
  }
  function failure(error,token,mutation=false) {
    if(!valid(token) || error.name==='AbortError')return;
    cancelSelectionFocus();
    if(error.message==='unauthenticated') {clear();lifecycle.expired('Session revoked or expired. Sign in again.');return;}
    if(error.message==='forbidden' || (!mutation || !pending) && error.message==='model_unavailable') {clear();view.fallback();return;}
    validation=null;find('draft-diagnostics').replaceChildren();
    if(mutation && ['outcome_unknown','model_unavailable'].includes(error.message)) {blocked=true;note('Submission outcome unknown. Reconcile saved state to reopen the native draft. No automatic retry was sent.');}
    else {pending=null;blocked=['draft_conflict','head_conflict','draft_closed'].includes(error.message);note(({draft_conflict:'Saved version changed. Local edits are retained; discard edits and reload to continue.',head_conflict:'Published generation changed. Local edits are retained; no rebase was performed.',draft_closed:'This draft is closed. Reopen saved state to inspect it.',draft_not_found:'The selected saved draft is absent.',malformed_request:'Native request refused. Check the declaration fields and document limits.',body_too_large:'The native document limit was exceeded.',model_unavailable:'Model source unavailable. Local edits are retained.',outcome_unknown:'Submission outcome unknown. Reopen saved state.'})[error.message] || 'Model operation refused. Local edits are retained.');}
  }
  async function run(label,work,mutation=false,restoreChoice=false) {
    if(!identity || !lifecycle.valid(identity,generation)) {clear();return;}
    const choice=find('draft-choice'),owned=restoreChoice && document.activeElement===choice;
    invalidate();abort=new AbortController();const token=epoch;selectionFocus=owned?token:null;loading=true;controls();note(label);
    try {await work(token);}catch(error){failure(error,token,mutation);}finally{if(valid(token)){
      loading=false;controls();
      if(selectionFocus===token && mode==='schema' && !pending && !blocked && choice.isConnected && !choice.disabled && choice.getClientRects().length && [choice,document.body].includes(document.activeElement))choice.focus({preventScroll:true});
      if(selectionFocus===token)cancelSelectionFocus();
    }}
  }
  async function discover(token,after=null) {
    const page=await request(`/drafts?limit=25${after?`&after=${encodeURIComponent(after)}`:''}`,token);if(!page || !valid(token))return;
    ontologyModelWire.page(page,after);summaries=after?[...new Map([...summaries,...page.drafts].map(s=>[s.id,s])).values()].sort((a,b)=>BigInt(a.id)<BigInt(b.id)?-1:1):page.drafts;next=page.next;choices();
  }
  function adopt(value) {
    const selected=draft?.id===value.id?{type:forms.selection(),link:forms.selectedLink(),property:forms.selectedProperty()}:null;
    formDirty=false;draft=value;working=clone(value.document);pending=null;blocked=false;validation=null;
    if(!summaries.some(s=>s.id===value.id))summaries.push({id:value.id,version:value.version,base_generation:value.base_generation,published_generation:value.publication?.generation || null});
    summaries=summaries.map(s=>s.id===value.id?{...s,version:value.version,base_generation:value.base_generation}:s);choices(value.id);find('draft-diagnostics').replaceChildren();forms.reset();if(selected?.link)forms.selectLink(selected.link);else if(selected?.type)forms.selectType(selected.type,selected.property);render();
  }
  async function open(session,version,retaining=false) {
    const same=identity?.tenantId===session.tenantId && identity?.userId===session.userId;
    if(!retaining || !same)clear();identity=session;generation=version;
    await run('Reading published model and owned drafts…',async token=>{
      const head=await request('',token);if(!head || !valid(token))return;workspace=ontologyModelWire.workspace(head);
      await discover(token);if(!valid(token))return;render();note(draft?'Owned draft retained for this verified identity.':'Published model loaded. Create a draft to edit declarations.');
    });
    return active();
  }
  async function freshCapability(token,action) {
    const head=await request('',token);if(!head || !valid(token))return false;workspace=ontologyModelWire.workspace(head);
    if(!(workspace.capabilities.author || action==='validate' && workspace.capabilities.publish))throw new Error('forbidden');return true;
  }
  find('draft-new').addEventListener('click',()=>{
    if(!active() || loading || draft || pending || blocked)return;
    run('Creating native owned draft…',async token=>{
      if(!await freshCapability(token,'author'))return;const submitted=clone(workspace.document),base=workspace.generation;pending={kind:'create',document:submitted,base};
      const value=await request('/drafts',token,{expected_generation:base,document:submitted});if(!value || !valid(token))return;
      try{ontologyModelWire.draft(value,identity);if(value.version!=='1' || value.base_generation!==base || value.publication || !ontologyModelEqual(value.document,submitted))throw new Error();}catch{throw new Error('outcome_unknown');}
      adopt(value);note('Draft created and saved.');
    },true);
  });
  find('draft-save').addEventListener('click',()=>{
    if(!active() || loading || !draft || !unsaved() || formDirty || pending || blocked)return;
    run('Saving exact native draft version…',async token=>{
      if(!await freshCapability(token,'author'))return;const submitted=clone(working),snapshot=clone(draft);pending={kind:'save',document:submitted,id:snapshot.id,version:snapshot.version,base:snapshot.base_generation};
      const value=await request(`/drafts/${snapshot.id}/save`,token,{expected_version:snapshot.version,expected_base:snapshot.base_generation,document:submitted});if(!value || !valid(token))return;
      try{ontologyModelWire.draft(value,identity,snapshot.id);if(BigInt(value.version)!==BigInt(snapshot.version)+1n || value.base_generation!==snapshot.base_generation || value.publication || !ontologyModelEqual(value.document,submitted))throw new Error();}catch{throw new Error('outcome_unknown');}
      const local=working,localForm=formDirty?forms.capture():null;adopt(value);if(!ontologyModelEqual(local,submitted))working=local;if(localForm){formDirty=true;forms.restore(localForm);}render();note(unsaved() || formDirty?'Submitted document saved; newer local edits remain unsaved.':'Exact submitted document saved.');
    },true);
  });
  find('draft-reload').addEventListener('click',()=>run('Reopening saved state…',async token=>{
    if(pending?.kind==='create'){await discover(token);if(valid(token))note('Creation outcome remains unknown. Choose an owned draft to inspect its saved document before creating again.');return;}
    if(!draft){const head=await request('',token);if(head){workspace=ontologyModelWire.workspace(head);render();}return;}
    const value=await request(`/drafts/${draft.id}`,token);if(!value || !valid(token))return;ontologyModelWire.draft(value,identity,draft.id);
    if(pending){const expected=pending;const local=working,localForm=formDirty?forms.capture():null;
      if(value.base_generation===expected.base && BigInt(value.version)===BigInt(expected.version)+1n && ontologyModelEqual(value.document,expected.document) && !value.publication){adopt(value);working=local;if(localForm){formDirty=true;forms.restore(localForm);}render();note(unsaved() || formDirty?'Saved submission recovered; newer local edits remain unsaved.':'Saved submission recovered from native state.');}
      else {pending=null;blocked=true;note('Reopened state does not match the submitted document/version. Local edits are retained. Discard edits and reload to continue.');}
    }else {adopt(value);note('Saved native draft reloaded; local edits discarded.');}
  }));
  find('draft-choice').addEventListener('change',()=>{
    if(pending?.kind==='save'){choices();note('Reconcile the uncertain submission before changing models.');return;}
    const choice=find('draft-choice').value;if((unsaved() || formDirty) && !pending){choices();note('Save or explicitly discard local edits and reload before changing the selected model.');return;}
    run('Opening selected native model…',async token=>{
      if(choice==='published'){if(pending){choices();note('Reconcile the uncertain submission before changing models.');return;}draft=null;working=null;validation=null;blocked=false;forms.reset();render();return;}
      if(!ontologyModelWire.decimal(choice))throw new Error('model_unavailable');const value=await request(`/drafts/${choice}`,token);if(value && valid(token)){ontologyModelWire.draft(value,identity,choice);adopt(value);note('Saved owned draft opened.');}
    },false,true);
  });
  find('draft-more').addEventListener('click',()=>{if(next)run('Discovering more owned drafts…',token=>discover(token,next));});
  function savedMatches(snapshot){return active() && mode==='schema' && draft && !draft.publication && !formDirty && draft.id===snapshot.id && draft.version===snapshot.version && draft.base_generation===snapshot.base_generation && ontologyModelEqual(draft.document,snapshot.document) && ontologyModelEqual(working,snapshot.document);}
  function inspectionControl(id){const element=find(id);return element?.isConnected && find('draft-inspector').contains(element) && !element.disabled && element.getClientRects().length ? element : null;}
  function inspectionAction(diagnostic,snapshot,accepted,owner){
    const target=ontologyModelDiagnosticTarget(snapshot.document,diagnostic);if(!target)return '';
    const row=(target.kind==='link'?snapshot.document.links:snapshot.document.entities).find(row=>row.id===target.id);
    const button=document.createElement('button');button.type='button';
    button.textContent=target.kind==='link'?`Inspect relationship ${row.id}`:target.kind==='property'?`Inspect property ${row.properties[target.propertyIndex].name} · ${row.display_name} · ${row.id}`:`Inspect type ${row.display_name} · ${row.id}`;
    button.addEventListener('click',()=>{
      if(!button.isConnected || !find('explorer').isConnected || !find('draft-diagnostics').contains(button) || validation!==accepted || identity?.tenantId!==owner.tenantId || identity?.userId!==owner.userId || generation!==owner.generation || loading || blocked || pending || !savedMatches(snapshot))return;
      const title=find('draft-inspector-title');
      if(!title?.isConnected || !find('draft-inspector').contains(title) || title.disabled || !revealOntologyDetails(title,find('explorer')) || !inspectionControl('draft-inspector-title'))return;
      if(!(target.kind==='link'?forms.selectLink(target.id):forms.selectType(target.id,target.propertyIndex ?? null)))return;
      render(target.kind==='link'?null:target.id);
      const destinations=workspace.capabilities.author?[target.focus,...(target.kind==='property'?['property-choice']:[]),'draft-inspector-title']:['draft-inspector-title'];
      for(const id of destinations){const candidate=find(id);if(!candidate?.disabled && find('draft-inspector').contains(candidate) && revealOntologyDetails(candidate,find('explorer'))){const control=inspectionControl(id);if(control){control.focus();break;}}}
    });
    return button;
  }
  find('draft-validate').addEventListener('click',()=>{
    if(!draft || unsaved() || formDirty || pending || blocked || loading)return;
    run('Validating saved changes…',async token=>{
      if(!await freshCapability(token,'validate'))return;const snapshot=clone(draft);const value=await request(`/drafts/${snapshot.id}/validate`,token,{expected_version:snapshot.version,expected_base:snapshot.base_generation});if(!value || !valid(token))return;
      ontologyModelWire.validation(value,snapshot);if(!savedMatches(snapshot))return;validation=value;forms.diagnostics(value.diagnostics);
      const owner={tenantId:identity.tenantId,userId:identity.userId,generation};
      find('draft-diagnostics').replaceChildren(ontologyTable('Schema validation',['Code','Field path','Message','Inspect declaration'],value.diagnostics.map(d=>[d.code,d.path,d.message,inspectionAction(d,snapshot,value,owner)])));
      note(value.valid?`Saved changes pass schema validation · version ${snapshot.version}. Publishing is not available yet.`:`Saved changes were refused by schema validation · version ${snapshot.version}.`);
    });
  });
  return {open,clear,active,render,selectType:id=>{if(active()){if(formDirty){note('Apply form edits or discard and reload before changing declaration.');return;}if(working && !forms.selectType(id))return;render(id);}},selectLink:id=>{if(!active())return false;if(formDirty){note('Apply form edits before changing declaration.');return false;}if(!working)return true;const prior=forms.selectedLink();if(!forms.selectLink(id))return false;if(prior!==id)forms.render(working,draft);controls();return forms.describe();},hold:()=>{invalidate();hide(true);},setMode:value=>{cancelSelectionFocus();mode=value;controls();if(value==='schema')render();}};
}
"#;
