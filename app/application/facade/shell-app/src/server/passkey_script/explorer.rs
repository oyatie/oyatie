mod editor;
mod model;
mod objects;
mod relationship_codec;
mod relationships;
mod render;
pub(super) fn script() -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
        render::SCRIPT,
        model::SCRIPT,
        editor::script(),
        relationship_codec::SCRIPT,
        SETUP,
        objects::SCRIPT,
        relationships::SCRIPT,
        SCRIPT
    )
}
const SETUP: &str = r#"
function createOntologyExplorer(lifecycle) {
  const find = id => document.getElementById(`live-ontology-${id}`);
  const root = find('explorer');
  let identity = null, generation = 0, epoch = 0, abort = null;
  let schemas = [], links = [], workspaceMode = 'schema';
  const paneBar=find('module-strip');
  const workspace=document.getElementById('foundry-workspace'),shell=document.getElementById('oyatie-shell');
  const header=shell.querySelector(':scope > .app-header');
  const contextHeader=workspace.querySelector(':scope > .foundry-workspace-header');
  const workpane=find('workpane'),toolbar=find('model').querySelector(':scope > .foundry-canvas-toolbar');
  const tray=find('draft-tray'),trayActions=tray.querySelector(':scope > .foundry-draft-actions');
  let paneGeneration=-1;
  function measureWorkArea(stripHeight) {
    if(!header?.isConnected || header.parentElement!==shell || !contextHeader?.isConnected || contextHeader.parentElement!==workspace || !toolbar?.isConnected || !workpane.contains(toolbar) || !trayActions?.isConnected || !tray.contains(trayActions)) {
      root.setAttribute('data-workspace-layout','flow');return;
    }
    const height=node=>{const style=getComputedStyle(node);return node.getClientRects().length?node.getBoundingClientRect().height+Number.parseFloat(style.marginTop)+Number.parseFloat(style.marginBottom):0;};
    const padding=node=>{const style=getComputedStyle(node);return Number.parseFloat(style.paddingTop)+Number.parseFloat(style.paddingBottom);};
    const minimum=Number.parseFloat(getComputedStyle(find('graph')).minHeight);
    const trayHeight=tray.getClientRects().length?height(trayActions)+height(find('draft-status'))+padding(tray)+Number.parseFloat(getComputedStyle(find('draft-review')).minHeight):0;
    const chrome=height(toolbar)+height(find('model-note'))+padding(workpane);
    const remaining=window.innerHeight-height(header)-Math.max(stripHeight,height(contextHeader))-trayHeight-chrome;
    const layout=Number.isFinite(remaining) && Number.isFinite(minimum) && minimum>0 && remaining>=minimum?'panes':'flow';
    if(root.getAttribute('data-workspace-layout')!==layout)root.setAttribute('data-workspace-layout',layout);
  }
  function measurePaneBar() {
    if(paneGeneration!==generation || !identity || !lifecycle.valid(identity,generation) || root.hidden || !root.isConnected || !paneBar.isConnected || !root.contains(paneBar) || !paneBar.getClientRects().length)return;
    let height=paneBar.getBoundingClientRect().height;
    if(!Number.isFinite(height) || height<=0)return;
    if(!workspace?.isConnected || !workspace.contains(root) || workspace.parentElement?.parentElement!==shell)return;
    measureWorkArea(height);
    height=paneBar.getBoundingClientRect().height;
    if(!Number.isFinite(height) || height<=0)return;
    let headerHeight=0;
    if(shell.getAttribute('data-shell-frame')==='foundry' && header?.parentElement===shell && header.isConnected && header.getClientRects().length && getComputedStyle(header).position==='sticky') {
      headerHeight=header.getBoundingClientRect().height;
      if(!Number.isFinite(headerHeight) || headerHeight<=0)return;
    }
    const overlayHeight=getComputedStyle(paneBar).position==='sticky'?height:0;
    for(const [owner,name,value]of [[root,'--foundry-pane-bar-height',overlayHeight],[workspace,'--foundry-header-height',headerHeight]]) {
      const measured=`${value}px`;
      if(owner.style.getPropertyValue(name)!==measured)owner.style.setProperty(name,measured);
    }
  }
  const paneObserver=typeof ResizeObserver==='function'?new ResizeObserver(measurePaneBar):null;
  window.addEventListener('resize',measurePaneBar);
  function revealPane(id, focus = false) {
    if(!identity || !lifecycle.valid(identity,generation) || root.hidden || !root.isConnected || !['library-title','workpane','inspector-title'].includes(id))return;
    const target=find(id);
    if(target?.disabled || !revealOntologyDetails(target,root))return;
    measurePaneBar();
    if(focus && target.getClientRects().length)target.focus();
  }
  for(const [pane,target] of [['library','library-title'],['workpane','workpane'],['inspector','inspector-title']])find(`pane-${pane}`).addEventListener('click',()=>revealPane(target,true));
  window.addEventListener('hashchange',()=>revealPane(location.hash.slice(1).replace(/^live-ontology-/,''),true));
  let editor;
  const model = createOntologyModelView(id => {
    if (workspaceMode==='schema' && editor?.active()) { editor.selectType(id); return; }
    find('type').value = id;
    changeType();
  }, focusSchemaControl, id=>workspaceMode!=='schema' || !editor?.active() ? true : editor.selectLink(id), value=>{
    workspaceMode=value;editor?.setMode(value);
    if(value==='objects' || !editor?.active()) {model.setModel(schemas,links,find('type').value || null);renderSchema();}
  });
  editor=createOntologyDraftEditor(lifecycle,{setModel:(rows,relations,selected,source)=>{model.setModel(rows,relations,selected,source);renderSchemaFor(rows.find(row=>row.entity_type===selected));},selectLink:id=>model.selectLink(id),fallback:()=>{model.setModel(schemas,links,find('type').value || null);renderSchema();}});
  let next = null, cursor = null, activeObject = null, historyShown = false, readPage = false;
  const queryControls = ['type', 'revision', 'limit', 'property', 'comparison', 'kind', 'from', 'to'];
  const note = text => { find('status').textContent = text; };
  const valid = token => token === epoch && identity && lifecycle.valid(identity, generation) && root.isConnected;
  function invalidate() { epoch += 1; abort?.abort(); abort = null; }
  function busy(value) { root.setAttribute('aria-busy', String(value)); }
  function ownsReadFocus(active) {
    return root.contains(active) && (active.closest('[data-live-results]') || ['type','read','next','first'].some(id => find(id) === active));
  }
  function focusSchemaControl() {
    [find('type'), find('types-refresh')].find(control => control && !control.disabled && !control.hidden && control.getClientRects().length)?.focus({ preventScroll: true });
  }
  function emptySchema() {
    const restoreFocus = find('schema').contains(document.activeElement);
    find('schema-summary').textContent = ''; find('schema-properties').replaceChildren(); find('schema').hidden = true;
    if (restoreFocus) focusSchemaControl();
  }
  function clearModel() {
    schemas = []; links = []; emptySchema(); model.clear();
    find('model-note').textContent='Current schema and relationship declarations are loaded separately. This view is not an atomic model snapshot.';
    for (const option of find('type').children) option.textContent = option.value;
  }
  function renderSchema() {
    renderSchemaFor(schemas.find(row => row.entity_type === find('type').value));
  }
  function renderSchemaFor(schema) {
    if (!schema) { emptySchema(); return; }
    find('schema-summary').textContent = `${schema.display_name} · ${schema.entity_type} · declaration revision ${schema.revision}. Primary-key property: ${schema.primary_key_property ?? 'none declared'}. Title property: ${schema.title_property ?? 'none declared'}.`;
    find('schema-properties').replaceChildren(ontologyTable('Native declared properties', ['Property', 'Tier', 'Value declaration', 'Presence', 'Privacy class'], schema.properties.map(property => [property.name, property.tier, ontologyDeclarationText(property.value_type), property.required ? 'Required' : 'Optional', property.data_class])));
    find('schema').hidden = false; model.select(schema.entity_type);
  }
  function emptyResults() {
    if ([find('next'), find('first')].includes(document.activeElement)) find('read').focus();
    if (['page','detail','audit'].some(id => find(id).contains(document.activeElement))) find('type').focus({ preventScroll: true });
    for (const id of ['page', 'properties', 'history', 'audit-rows']) find(id).replaceChildren();
    find('detail-title').textContent = 'Object detail';
    find('detail').hidden = true; find('audit').hidden = true;
    find('next').hidden = true; find('first').hidden = true;
    clearRelationships();
    next = null; cursor = null; activeObject = null; historyShown = false; readPage = false;
  }
  function clear() {
    const ownedFocus = root.contains(document.activeElement);
    paneObserver?.disconnect();paneGeneration=-1;root.style.removeProperty('--foundry-pane-bar-height');workspace?.style.removeProperty('--foundry-header-height');root.removeAttribute('data-workspace-layout');
    invalidate(); editor.clear();
    if (ownedFocus && (document.activeElement === document.body || root.contains(document.activeElement))) lifecycle.fallback();
    emptyResults();
    schemas = []; links = []; emptySchema(); model.reset();
    find('type').replaceChildren(); find('type').disabled = true; find('read').disabled = true;
    find('revision').value = '1'; find('limit').value = '25';
    for (const id of ['property', 'from', 'to']) find(id).value = '';
    find('comparison').value = 'none'; find('kind').value = 'text';
    find('access-badge').textContent='Sign-in required';
    find('access-notice').hidden = false;
    find('identity').textContent = ''; root.hidden = true; identity = null; busy(false);
  }
  function query(after = null) {
    const revision = Number(find('revision').value), limit = Number(find('limit').value);
    if (!find('revision').value || !Number.isInteger(revision) || revision < 0 || revision > 4294967295
      || !Number.isInteger(limit) || limit < 1 || limit > 1000 || !find('type').value) throw new Error('invalid_query');
    const params = new URLSearchParams({ type: find('type').value, revision, limit });
    if (after) params.set('cursor', after);
    const comparison = find('comparison').value;
    if (comparison !== 'none') {
      params.set('property', find('property').value);
      const kind = find('kind').value;
      params.set(comparison === 'equals' ? 'equals' : 'from', `${kind}:${find('from').value}`);
      if (comparison === 'range') params.set('to', `${kind}:${find('to').value}`);
    }
    return params;
  }
  async function fetchRead(path, token, body = undefined) {
    const response = await fetch(`/api/ontology/${encodeURIComponent(identity.tenantId)}/${path}`, {
      credentials: 'same-origin', cache: 'no-store', redirect: 'error', signal: abort.signal,
      method: body === undefined ? 'GET' : 'POST', headers: body === undefined ? {} : {'content-type':'application/json'}, body,
    });
    if (!valid(token)) return null;
    if (response.ok && (response.headers.get('x-oyatie-tenant') !== identity.tenantId
      || response.headers.get('x-oyatie-user') !== identity.userId
      || !Number.isSafeInteger(Number(response.headers.get('x-oyatie-session-expires')))
      || Number(response.headers.get('x-oyatie-session-expires')) * 1000 <= Date.now())) throw new Error('unauthenticated');
    const replyExpiry = Number(response.headers.get('x-oyatie-session-expires'));
    const payload = await response.json();
    if (!valid(token)) return null;
    if (!response.ok) throw new Error(typeof payload.error === 'string' ? payload.error : 'source_unavailable');
    if (replyExpiry * 1000 <= Date.now()) throw new Error('unauthenticated');
    return payload;
  }
  function failure(error, token) {
    if (!valid(token) || error.name === 'AbortError') return;
    const active = document.activeElement;
    const restoreFocus = ownsReadFocus(active);
    emptyResults();
    if (!editor.active()) clearModel(); else { schemas=[];links=[];emptySchema();editor.render(); }
    if (['forbidden', 'source_unavailable'].includes(error.message) || !['unauthenticated','invalid_query','absent_object','revision_conflict','filter_conflict'].includes(error.message)) { find('type').replaceChildren(); find('type').disabled = true; find('read').disabled = true; }
    if (restoreFocus) focusSchemaControl();
    const messages = {
      unauthenticated: 'Session revoked or expired. Sign in again.',
      forbidden: 'Ontology access refused. Ask your administrator to check access.',
      invalid_query: 'Read query refused. Check the type, revision, page limit and filter values.',
      absent_object: 'This object is absent from the durable projection.',
      revision_conflict: 'The requested schema revision was never accepted for this entity type.',
      filter_conflict: 'Stored values of this property have another kind. Choose a matching filter kind.',
      source_unavailable: 'Native Ontology reads are unavailable. Retry. No sample rows are shown here.',
    };
    note(messages[error.message] || messages.source_unavailable);
    if (error.message === 'unauthenticated') lifecycle.expired(messages.unauthenticated);
  }
  async function run(label, work) {
    if (!identity || !lifecycle.valid(identity, generation)) { clear(); return; }
    invalidate(); abort = new AbortController(); const token = epoch;
    busy(true); note(label);
    try { await work(token); } catch (error) { failure(error, token); }
    finally { if (valid(token)) busy(false); }
  }
"#;
pub(super) const SCRIPT: &str = r#"
  async function open(session, version, retaining = false) {
    if (!session || !lifecycle.valid(session, version)) { clear(); return; }
    const same = identity?.tenantId === session.tenantId && identity?.userId === session.userId;
    const selection = retaining && same ? { type: find('type').value, revision: find('revision').value, cursor, activeObject, readPage, auditShown: !find('audit').hidden } : null;
    if (!selection) clear();
    else clearRelationships();
    identity = session; generation = version; root.hidden = false;find('access-badge').textContent='Session verified'; find('access-notice').hidden = true;
    paneGeneration=version;
    for(const node of [paneBar,header,contextHeader,toolbar,find('model-note'),tray,trayActions,find('draft-status')])if(node?.isConnected)paneObserver?.observe(node);
    measurePaneBar();
    revealPane(location.hash.slice(1).replace(/^live-ontology-/,''));
    find('identity').textContent = `Verified tenant ${session.tenantId} · account ${session.userId}`;
    if (!editor.active()) clearModel();
    const modelAccess=editor.open(session,version,retaining);
    await run('Reading native entity schemas…', async token => {
      const rows = await fetchRead('schemas', token);
      if (!valid(token)) return;
      ontologyDecode.schemas(rows);
      const labelRows = await fetchRead('type-labels', token);
      if (!valid(token)) return;
      const labels = ontologyDecode.labels(labelRows, rows);
      const relations = await fetchRead('link-types', token);
      if (!valid(token)) return;
      ontologyDecode.links(relations, rows); schemas = rows.map(row => ({...row, display_name: labels.get(row.entity_type).display_name})); links = relations;
      const active = document.activeElement, restoreFocus = ownsReadFocus(active);
      find('type').replaceChildren(...schemas.map(t => { const option = document.createElement('option'); option.value = t.entity_type; option.textContent = `${t.display_name} · ${t.entity_type} · declared revision ${t.revision}`; option.dataset.revision = t.revision; return option; }));
      find('type').disabled = schemas.length === 0; find('read').disabled = schemas.length === 0;
      if (selection && schemas.some(t => t.entity_type === selection.type)) { find('type').value = selection.type; find('revision').value = selection.revision; }
      else { emptyResults(); if (schemas.length) find('revision').value = schemas[0].revision; }
      if (!editor.active()) {model.setModel(schemas, links, find('type').value || null); renderSchema();}
      if (restoreFocus && (schemas.length === 0 || active === find('type'))) focusSchemaControl();
      note(schemas.length ? 'Native schemas loaded. Choose a type and requested schema revision, then Read objects.' : 'No entity types are declared for this verified tenant.');
      if (selection?.readPage && find('type').value === selection.type) await loadPage(token, selection.cursor, selection.activeObject);
      if (selection?.auditShown && valid(token)) await loadAudit(token);
    });
    await modelAccess;
    if (editor.active()) {editor.render();note('Native model workspace loaded. Object access is checked independently.');}
  }
  find('query').addEventListener('submit', event => { event.preventDefault(); emptyResults(); run('Reading pinned native objects…', token => loadPage(token)); });
  find('next').addEventListener('click', () => { const after = next; emptyResults(); run('Reading next native page…', token => loadPage(token, after)); });
  find('first').addEventListener('click', () => { emptyResults(); run('Reading first native page…', token => loadPage(token)); });
  find('types-refresh').addEventListener('click', () => open(identity, generation, true));
  find('history-read').addEventListener('click', () => run('Reading native object history…', loadHistory));
  async function loadAudit(token) {
    find('audit-rows').replaceChildren(); find('audit').hidden = true;
    const rows = await fetchRead('audit', token); if (!rows || !valid(token)) return;
    ontologyDecode.audit(rows);
    find('audit-rows').replaceChildren(ontologyTable('Native tenant audit', ['Ordinal', 'Object', 'Action', 'Disposition', 'Poison reason', 'Principal', 'Decision'], rows.map(r => [r.ordinal, r.object_ref, r.action_type, r.disposition, r.poison_reason, r.principal_id, r.decision_id])));
    find('audit').hidden = false;
    note(rows.length ? `${rows.length} native audit entries. These rows do not authorize a current read.` : 'No native tenant audit entries.');
  }
  find('audit-read').addEventListener('click', () => run('Reading native tenant audit…', loadAudit));
  function changeType() {
    invalidate(); emptyResults(); busy(false);
    find('revision').value = find('type').selectedOptions[0]?.dataset.revision || '1'; renderSchema();
    note('Model selection changed. Choose Objects to read instances of this type.');
  }
  for (const id of queryControls) find(id).addEventListener('input', () => {
    if (id === 'type') { changeType(); return; }
    invalidate(); emptyResults(); busy(false);
    note('Read selection changed. Read objects to load this context.');
  });
  return { clear, open, measure:measurePaneBar, hold: version => {paneObserver?.disconnect();find('access-badge').textContent='Checking session';find('identity').textContent=identity?`Checking tenant ${identity.tenantId} · account ${identity.userId}`:''; invalidate(); clearRelationships(); editor.hold(); clearModel(); generation = version; busy(true); note('Checking current session and Ontology access…'); } };
}
"#;
