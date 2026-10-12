mod render;
pub(super) fn script() -> String {
    format!("{}\n{}", render::SCRIPT, SCRIPT)
}
pub(super) const SCRIPT: &str = r#"
function createOntologyExplorer(lifecycle) {
  const find = id => document.getElementById(`live-ontology-${id}`);
  const root = find('explorer');
  let identity = null, generation = 0, epoch = 0, abort = null;
  let next = null, cursor = null, objectRef = null, historyShown = false, readPage = false;
  const queryControls = ['type', 'revision', 'limit', 'property', 'comparison', 'kind', 'from', 'to'];
  const note = text => { find('status').textContent = text; };
  const valid = token => token === epoch && identity && lifecycle.valid(identity, generation) && root.isConnected;
  function invalidate() { epoch += 1; abort?.abort(); abort = null; }
  function busy(value) { root.setAttribute('aria-busy', String(value)); }
  function emptyResults() {
    if ([find('next'), find('first')].includes(document.activeElement)) find('read').focus();
    if (root.contains(document.activeElement) && document.activeElement.closest('[data-live-results]')) find('type').focus();
    for (const id of ['page', 'properties', 'history', 'audit-rows']) find(id).replaceChildren();
    find('detail').hidden = true; find('audit').hidden = true;
    find('next').hidden = true; find('first').hidden = true;
    next = null; cursor = null; objectRef = null; historyShown = false; readPage = false;
  }
  function clear() {
    invalidate();
    if (root.contains(document.activeElement)) lifecycle.fallback();
    emptyResults();
    find('type').replaceChildren(); find('type').disabled = true; find('read').disabled = true;
    find('revision').value = '1'; find('limit').value = '25';
    for (const id of ['property', 'from', 'to']) find(id).value = '';
    find('comparison').value = 'none'; find('kind').value = 'text';
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
  async function fetchRead(path, token) {
    const response = await fetch(`/api/ontology/${encodeURIComponent(identity.tenantId)}/${path}`, {
      credentials: 'same-origin', cache: 'no-store', redirect: 'error', signal: abort.signal,
    });
    if (!valid(token)) return null;
    if (response.ok && (response.headers.get('x-oyatie-tenant') !== identity.tenantId
      || response.headers.get('x-oyatie-user') !== identity.userId
      || !Number.isSafeInteger(Number(response.headers.get('x-oyatie-session-expires')))
      || Number(response.headers.get('x-oyatie-session-expires')) * 1000 <= Date.now())) throw new Error('unauthenticated');
    const body = await response.json();
    if (!valid(token)) return null;
    if (!response.ok) throw new Error(typeof body.error === 'string' ? body.error : 'source_unavailable');
    return body;
  }
  function failure(error, token) {
    if (!valid(token) || error.name === 'AbortError') return;
    emptyResults();
    if (['forbidden', 'source_unavailable'].includes(error.message) || !['unauthenticated','invalid_query','absent_object','revision_conflict','filter_conflict'].includes(error.message)) { find('type').replaceChildren(); find('type').disabled = true; find('read').disabled = true; if (document.activeElement === find('type')) find('types-refresh').focus(); }
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
  function renderPage(page, params, token) {
    if (!valid(token)) return;
    ontologyDecode.page(page, Number(params.get('revision')), Number(params.get('limit')));
    const active = document.activeElement;
    const focusedRef = active?.dataset?.liveObject;
    const restoreFocus = focusedRef && find('page').contains(active);
    find('page').replaceChildren(ontologyTable('Pinned native objects', ['Object reference', 'Stored schema revision', 'Upcast state'], page.objects.map(object => {
      const button = document.createElement('button'); button.type = 'button'; button.textContent = object.object_ref;
      button.dataset.liveObject = object.object_ref;
      button.addEventListener('click', () => detail(object.object_ref, button));
      return [button, object.written_revision, object.upcast_state];
    })));
    if (restoreFocus) {
      const replacement = [...find('page').querySelectorAll('[data-live-object]')].find(b => b.dataset.liveObject === focusedRef);
      replacement?.focus();
    }
    next = page.next; cursor = params.get('cursor'); readPage = true;
    find('next').hidden = !next; find('first').hidden = !cursor;
    note(page.objects.length ? `${page.objects.length} native objects · requested schema revision ${page.requested_revision}. Stored revisions are shown per object.`
      : `No native objects match this query at requested schema revision ${page.requested_revision}.`);
  }
  function renderDetail(detail, reference, token, trigger = null) {
    if (!valid(token)) return;
    ontologyDecode.detail(detail, reference, Number(find('revision').value));
    objectRef = reference;
    find('detail-title').textContent = `${reference} · requested schema ${detail.requested_revision} · stored schema ${detail.object.written_revision} · ${detail.object.upcast_state}`;
    find('properties').replaceChildren(ontologyTable('Native object properties', ['Property', 'Value kind', 'Privacy class', 'Value'], Object.entries(detail.object.properties).map(([name, p]) => [name, p.value_type, p.data_class, JSON.stringify(p.value)])));
    find('detail').hidden = false;
    if (trigger && document.activeElement === trigger) find('detail-title').focus();
  }
  async function loadPage(token, after = null, selected = null) {
    const params = query(after), page = await fetchRead(`objects?${params}`, token);
    if (!page || !valid(token)) return;
    renderPage(page, params, token);
    if (selected) {
      const detail = await fetchRead(`objects/${encodeURIComponent(selected)}?revision=${params.get('revision')}`, token);
      if (detail) renderDetail(detail, selected, token);
      if (historyShown && valid(token)) await loadHistory(token);
    }
  }
  function detail(reference, trigger) {
    return run('Reading pinned native object…', async token => {
      find('properties').replaceChildren(); find('history').replaceChildren(); find('detail').hidden = true; historyShown = false;
      const result = await fetchRead(`objects/${encodeURIComponent(reference)}?revision=${find('revision').value}`, token);
      if (result) { renderDetail(result, reference, token, trigger); note('Pinned native object read. History is a separate native log read.'); }
    });
  }
  async function loadHistory(token) {
    const rows = await fetchRead(`objects/${encodeURIComponent(objectRef)}/history`, token);
    if (!rows || !valid(token)) return;
    ontologyDecode.history(rows); historyShown = true;
    find('history').replaceChildren(ontologyTable('Native object history', ['Ordinal', 'Object sequence', 'Principal', 'Decision', 'Action', 'Event', 'Schema revision'], rows.map(r => [r.ordinal, r.object_sequence, r.principal_id, r.decision_id, r.action_type, r.audit_event_type, r.schema_revision])));
    note(rows.length ? `${rows.length} native history entries. Each has its own schema revision.` : 'No native history entries for this object.');
  }
  async function open(session, version, retaining = false) {
    if (!session || !lifecycle.valid(session, version)) { clear(); return; }
    const same = identity?.tenantId === session.tenantId && identity?.userId === session.userId;
    const selection = retaining && same ? { type: find('type').value, revision: find('revision').value, cursor, objectRef, readPage, auditShown: !find('audit').hidden } : null;
    if (!selection) clear();
    identity = session; generation = version; root.hidden = false;
    find('identity').textContent = `Verified tenant ${session.tenantId} · account ${session.userId}`;
    await run('Reading native entity types…', async token => {
      const types = await fetchRead('types', token);
      if (!types || !valid(token)) return;
      ontologyDecode.types(types);
      find('type').replaceChildren(...types.map(t => { const option = document.createElement('option'); option.value = t.entity_type; option.textContent = `${t.entity_type} · declared revision ${t.revision}`; option.dataset.revision = t.revision; return option; }));
      find('type').disabled = types.length === 0; find('read').disabled = types.length === 0;
      if (selection && types.some(t => t.entity_type === selection.type)) { find('type').value = selection.type; find('revision').value = selection.revision; }
      else { emptyResults(); if (types.length) find('revision').value = types[0].revision; }
      note(types.length ? 'Native types loaded. Choose a type and requested schema revision, then Read objects.' : 'No entity types are declared for this verified tenant.');
      if (selection?.readPage && find('type').value === selection.type) await loadPage(token, selection.cursor, selection.objectRef);
      if (selection?.auditShown && valid(token)) await loadAudit(token);
    });
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
  for (const id of queryControls) find(id).addEventListener('input', () => {
    invalidate(); emptyResults(); busy(false);
    if (id === 'type') find('revision').value = find('type').selectedOptions[0]?.dataset.revision || '1';
    note('Read selection changed. Read objects to load this context.');
  });
  return { clear, open, hold: version => { invalidate(); generation = version; busy(true); note('Checking current session and Ontology access…'); } };
}
"#;
