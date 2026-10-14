pub(super) const SCRIPT: &str = r#"
  function renderPage(page, params, token) {
    if (!valid(token)) return;
    ontologyDecode.page(page, Number(params.get('revision')), Number(params.get('limit')));
    const active = document.activeElement;
    const focusedRef = active?.dataset?.liveObject;
    const restoreFocus = find('page').contains(active);
    find('page').replaceChildren(ontologyTable('Pinned native objects', ['Object reference', 'Stored schema revision', 'Upcast state'], page.objects.map(object => {
      const button = document.createElement('button'); button.type = 'button'; button.textContent = object.object_ref;
      button.dataset.liveObject = object.object_ref;
      button.addEventListener('click', () => detail({entity_type: params.get('type'), object_ref: object.object_ref, revision: Number(params.get('revision'))}, button));
      return [button, object.written_revision, object.upcast_state];
    })));
    if (restoreFocus) {
      const replacement = [...find('page').querySelectorAll('[data-live-object]')].find(b => b.dataset.liveObject === focusedRef);
      const fallback = [find('type'), find('read')].find(control => control && !control.disabled && !control.hidden);
      (replacement || fallback)?.focus({ preventScroll: true });
    }
    next = page.next; cursor = params.get('cursor'); readPage = true;
    find('next').hidden = !next; find('first').hidden = !cursor;
    note(page.objects.length ? `${page.objects.length} native objects · requested schema revision ${page.requested_revision}. Stored revisions are shown per object.`
      : `No native objects match this query at requested schema revision ${page.requested_revision}.`);
  }
  function renderDetail(detail, context, token, trigger = null) {
    if (!valid(token)) return;
    ontologyDecode.detail(detail, context.object_ref, context.revision);
    activeObject = {...context};
    find('detail-title').textContent = `${context.object_ref} · selected type ${context.entity_type} · requested schema ${detail.requested_revision} · stored schema ${detail.object.written_revision} · ${detail.object.upcast_state}`;
    find('properties').replaceChildren(ontologyTable('Native object properties', ['Property', 'Value kind', 'Privacy class', 'Value'], Object.entries(detail.object.properties).map(([name, p]) => [name, p.value_type, p.data_class, JSON.stringify(p.value)])));
    find('detail').hidden = false; renderRelationshipChoices(true);
    if (trigger && document.activeElement === trigger) find('detail-title').focus();
  }
  async function loadPage(token, after = null, selected = null) {
    const params = query(after), page = await fetchRead(`objects?${params}`, token);
    if (!page || !valid(token)) return;
    renderPage(page, params, token);
    if (selected) {
      const detail = await fetchRead(pointPath(selected), token);
      if (detail) renderDetail(detail, selected, token);
      if (historyShown && valid(token)) await loadHistory(token);
    }
  }
  function pointPath(context) {
    return `objects/${encodeURIComponent(context.object_ref)}?${new URLSearchParams({type:context.entity_type,revision:context.revision})}`;
  }
  function detail(context, trigger) {
    const ownedFocus = trigger && document.activeElement === trigger;
    const removedTrigger = trigger && find('relationship-results').contains(trigger);
    return run('Reading pinned native object…', async token => {
      clearRelationships(); activeObject = null;
      const focusOwner = ownedFocus && removedTrigger ? find('read') : trigger;
      if (ownedFocus && removedTrigger) focusOwner.focus({preventScroll:true});
      find('detail-title').textContent = 'Object detail';
      find('properties').replaceChildren(); find('history').replaceChildren(); find('detail').hidden = true; historyShown = false;
      const result = await fetchRead(pointPath(context), token);
      if (result) { renderDetail(result, context, token, ownedFocus ? focusOwner : null); note('Pinned native object read. History is a separate native log read.'); }
    });
  }
  async function loadHistory(token) {
    if (!activeObject) throw new Error('invalid_query');
    const rows = await fetchRead(`objects/${encodeURIComponent(activeObject.object_ref)}/history`, token);
    if (!rows || !valid(token)) return;
    ontologyDecode.history(rows); historyShown = true;
    find('history').replaceChildren(ontologyTable('Native object history', ['Ordinal', 'Object sequence', 'Principal', 'Decision', 'Action', 'Event', 'Schema revision'], rows.map(r => [r.ordinal, r.object_sequence, r.principal_id, r.decision_id, r.action_type, r.audit_event_type, r.schema_revision])));
    note(rows.length ? `${rows.length} native history entries. Each has its own schema revision.` : 'No native history entries for this object.');
  }
"#;
