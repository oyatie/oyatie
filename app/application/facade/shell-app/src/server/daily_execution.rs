pub(super) const SCRIPT: &str = r#"
(() => {
  const host = document.getElementById('dashboard-island-root');
  let current = null, pending = null, settling = false;
  const owner = () => {
    const root = document.getElementById('daily-execution');
    return root?.dataset.dailySource && host?.contains(root) ? root : null;
  };
  const rows = root => [...root.querySelectorAll('[data-daily-row]')];
  const details = root => [...root.querySelectorAll('[data-daily-detail]')];
  const available = root => root.dataset.dailyAvailable === 'true';
  const normalize = value => value.normalize('NFC').trim().toLocaleLowerCase();
  function source(root) {
    return JSON.stringify([root.dataset.dailySource, rows(root).map(row => [row.dataset.dailyRecord,
      row.dataset.dailyKind, row.dataset.dailyState, row.dataset.dailySearchText,
      row.querySelector('[data-daily-title]')?.textContent, row.querySelector('[data-daily-body]')?.textContent,
      row.querySelector('dl')?.textContent]), [...root.querySelectorAll('.daily-timeline li')].map(item =>
        [...item.children].map(field => field.textContent))]);
  }
  function remember() {
    if (!current) return;
    current.query = current.root.querySelector('[data-daily-search]').value;
    current.open = details(current.root).filter(detail => detail.open).map(detail => detail.dataset.dailyDetail);
    current.packetOpen = current.root.querySelector('[data-daily-packet]').open;
  }
  function packet() {
    const output = current.root.querySelector('[data-daily-packet-rows]');
    output.replaceChildren();
    rows(current.root).filter(row => current.staged.has(row.dataset.dailyId) && !row.hidden).forEach(row => {
      const record = JSON.parse(row.dataset.dailyRecord), item = document.createElement('li');
      const title = document.createElement('strong'), body = document.createElement('p'), metadata = document.createElement('dl');
      title.textContent = `${record[0]} · ${record[3]}`; body.textContent = record[4];
      for (const [label, value] of [['ID', record[0]], ['Kind', record[1]], ['State', record[2]],
        ['Owner', record[5]], ['Due', record[6]], ['Route', record[7]]]) {
        const pair = document.createElement('div'), name = document.createElement('dt'), text = document.createElement('dd');
        name.textContent = label; text.textContent = value; pair.append(name, text); metadata.append(pair);
      }
      item.dataset.dailyPacketRow = record[0]; item.append(title, body, metadata); output.append(item);
    });
    current.root.querySelector('[data-daily-packet-empty]').hidden = output.children.length !== 0;
  }
  function apply() {
    const root = current.root, all = rows(root), query = normalize(current.query);
    let visible = 0;
    all.forEach(row => {
      row.hidden = !((current.filter === 'all' || row.dataset.dailyKind === current.filter
        || current.filter === 'blocking' && row.dataset.dailyState === 'blocking')
        && normalize(row.dataset.dailySearchText).includes(query));
      if (row.hidden) { row.querySelector('details').open = false; current.staged.delete(row.dataset.dailyId); }
      else visible++;
      const staged = current.staged.has(row.dataset.dailyId), button = row.querySelector('[data-daily-stage]');
      row.classList.toggle('daily-staged', staged); button.setAttribute('aria-pressed', String(staged));
      button.textContent = staged ? 'Remove sample' : 'Stage sample';
    });
    current.staged = new Set([...current.staged].filter(id => all.some(row => row.dataset.dailyId === id && !row.hidden)));
    root.querySelectorAll('[data-daily-filter]').forEach(button => {
      const selected = button.dataset.dailyFilter === current.filter;
      button.setAttribute('aria-pressed', String(selected)); button.classList.toggle('active', selected);
    });
    root.querySelector('[data-daily-status]').textContent = `${visible} of ${all.length} sample rows shown · ${current.staged.size} staged`;
    root.querySelector('[data-daily-empty]').hidden = visible !== 0;
    root.querySelector('[data-daily-availability]').textContent = `${current.available ? 'Observed anonymous sample' : 'Retained anonymous sample'}: ${root.dataset.dailyLabel}. ${current.available ? 'Local sample controls available.' : 'Requested sample unavailable; local controls paused.'} Live Calendar access and capacity are unavailable.`;
    root.querySelectorAll('button, input').forEach(control => { control.disabled = !current.available; });
    root.querySelectorAll('summary, [data-daily-route]').forEach(control => control.setAttribute('aria-disabled', String(!current.available)));
    packet(); remember();
  }
  function restoreFocus() {
    const saved = current.focus;
    if (!saved || saved.node.isConnected || !current.available) return;
    if (!document.hasFocus() || location.href !== saved.url) { current.focus = null; return; }
    const active = document.activeElement;
    if (active && active !== document.body && active.isConnected) { current.focus = null; return; }
    const target = [...current.root.querySelectorAll('[data-daily-focus]')]
      .find(control => control.dataset.dailyFocus === saved.token && !control.disabled && control.getClientRects().length);
    if (!target) { current.focus = null; return; }
    const start = saved.node.selectionStart, end = saved.node.selectionEnd;
    target.focus({ preventScroll: true });
    if (document.activeElement !== target) { current.focus = null; return; }
    if (start !== undefined && typeof target.setSelectionRange === 'function') target.setSelectionRange(start, end);
  }
  function synchronize() {
    const root = owner();
    if (!root) return;
    const key = source(root), active = available(root), mode = root.dataset.dailyMode;
    if (current?.root === root && current.key === key && current.available === active && current.mode === mode) return;
    remember();
    const previous = current, same = previous?.key === key;
    if (!same) pending = null;
    current = { root, key, available: active, mode, query: same ? previous.query : '',
      filter: same ? previous.filter : 'all', open: same ? previous.open : [],
      staged: same ? previous.staged : new Set(), focus: same ? previous.focus : null,
      packetOpen: same ? previous.packetOpen : false };
    root.querySelector('[data-daily-search]').value = current.query;
    details(root).forEach(detail => { detail.open = current.open.includes(detail.dataset.dailyDetail); });
    root.querySelector('[data-daily-packet]').open = current.packetOpen;
    root.querySelector('[data-daily-reset]').hidden = same || !previous;
    apply(); restoreFocus();
    if (pending && current.mode === 'island' && current.available) {
      const action = pending; pending = null;
      if (action.key === key && location.href === action.url) openChannel(action.channel);
    }
  }
  function target(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const root = event.target.closest('#daily-execution[data-daily-source]');
    if (!root || root !== owner()) return null;
    synchronize();
    return current?.root === root ? event.target : null;
  }
  function workspace() {
    const hub = document.getElementById('work-hub'), result = hub?.querySelector('.communications-hub[data-hub-source]');
    return result && host.contains(hub) && result.dataset.hubContext === current.root.dataset.dailyContext ? result : null;
  }
  function channelReady(root, channel) {
    return root?.querySelector(`#hub-tab-${channel.toLowerCase()}`)?.getAttribute('aria-selected') === 'true'
      && root.querySelector('[data-comms-product-board]')?.dataset.commsBoardSurface === channel
      && root.querySelector('.hub-list') && root.querySelector('.hub-detail') && root.querySelector('.hub-composer textarea');
  }
  function openChannel(channel) {
    if (!['Mail', 'Messenger', 'Community'].includes(channel) || !current.available) return false;
    let root = workspace();
    if (!root || root.dataset.hubMode !== current.mode) return false;
    if (channelReady(root, channel)) return true;
    if (current.mode === 'island') {
      const tab = root.querySelector(`#hub-tab-${channel.toLowerCase()}`);
      if (!tab || tab.disabled) return false;
      tab.click();
    } else {
      const template = [...current.root.querySelectorAll('[data-daily-hub-template]')]
        .find(item => item.dataset.dailyHubTemplate === channel);
      const replacement = template?.content.firstElementChild?.cloneNode(true);
      if (!replacement || replacement.dataset.hubContext !== current.root.dataset.dailyContext || !channelReady(replacement, channel)) return false;
      root.replaceWith(replacement); root = replacement;
    }
    const key = current.key;
    requestAnimationFrame(() => {
      if (current?.key !== key || !current.available) return;
      const ready = channelReady(workspace(), channel);
      current.root.querySelector('[data-daily-route-status]').textContent = ready
        ? `${channel} existing sample opened; no draft, message, publication, or receipt created.`
        : `${channel} sample unavailable; no output created.`;
    });
    return true;
  }
  function revealSSR(destination) {
    const panel = destination.closest('[role="tabpanel"]');
    if (panel) {
      const tab = document.getElementById(panel.getAttribute('aria-labelledby'));
      const group = tab?.closest('[role="tablist"]');
      if (!tab || tab.disabled || tab.getAttribute('aria-controls') !== panel.id || tab.parentElement !== group
        || group.hasAttribute('data-native-hub-tabs') || group.classList.contains('hub-tabs')) return false;
      tab.click();
      if (tab.getAttribute('aria-selected') !== 'true' || !panel.classList.contains('active')) return false;
    }
    if (document.hasFocus()) {
      if (!destination.hasAttribute('tabindex')) destination.setAttribute('tabindex', '-1');
      destination.focus({ preventScroll: true }); destination.scrollIntoView({ block: 'start' });
    }
    return true;
  }
  document.addEventListener('input', event => {
    const control = target(event);
    if (!control?.matches('[data-daily-search]')) return;
    if (!current.available) { control.value = current.query; return; }
    current.query = control.value; apply();
  });
  document.addEventListener('click', event => {
    const control = target(event);
    if (!control || event.defaultPrevented || event.button !== 0 || event.altKey || event.ctrlKey || event.metaKey || event.shiftKey) return;
    if (!current.available) { event.preventDefault(); return; }
    const filter = control.closest('[data-daily-filter]'), stage = control.closest('[data-daily-stage]');
    if (filter) { current.filter = filter.dataset.dailyFilter; apply(); }
    else if (stage) {
      const id = stage.dataset.dailyStage;
      if (!rows(current.root).some(row => row.dataset.dailyId === id && !row.hidden)) return;
      if (current.staged.has(id)) current.staged.delete(id); else current.staged.add(id);
      apply();
    } else if (control.closest('[data-daily-clear]')) {
      current.query = ''; current.filter = 'all'; current.root.querySelector('[data-daily-search]').value = ''; apply();
    } else if (control.closest('[data-daily-close]')) {
      const detail = control.closest('details'); detail.open = false;
      if (document.hasFocus()) detail.querySelector('summary').focus({ preventScroll: true });
      remember();
    } else {
      const link = control.closest('[data-daily-route]');
      if (!link) return;
      const destination = document.getElementById(link.getAttribute('href').slice(1));
      if (!destination || link.dataset.dailyChannel && !openChannel(link.dataset.dailyChannel)) {
        event.preventDefault(); current.root.querySelector('[data-daily-route-status]').textContent = 'Sample destination unavailable; no output created.'; return;
      }
      if (current.mode === 'ssr' && !revealSSR(destination)) { event.preventDefault(); return; }
      current.focus = null;
      current.root.querySelector('[data-daily-route-status]').textContent = 'Existing sample view opened; no task, event, draft, or receipt created.';
      if (current.mode === 'ssr' && link.dataset.dailyChannel && location.href === link.href) event.preventDefault();
      if (current.mode === 'ssr' && link.dataset.dailyChannel) pending = { key: current.key,
        channel: link.dataset.dailyChannel, oldURL: location.href, url: link.href, awaitingHash: location.href !== link.href, ownPop: location.href !== link.href };
    }
  });
  document.addEventListener('toggle', event => {
    const control = target(event);
    if (control?.matches('[data-daily-detail], [data-daily-packet]')) remember();
  }, true);
  document.addEventListener('focusin', event => {
    const control = target(event)?.closest('[data-daily-focus]');
    if (control && current.available) current.focus = { node: control, token: control.dataset.dailyFocus, url: location.href };
    else if (current && event.target !== document.body) current.focus = null;
  });
  for (const name of ['pointerdown', 'keydown']) document.addEventListener(name, event => {
    if (!event.isTrusted || !(event.target instanceof Element)) return;
    pending = null;
    if (current?.focus && event.target !== current.focus.node) current.focus = null;
  }, true);
  document.addEventListener('keydown', event => {
    const control = target(event);
    if (control && !current.available && ['Enter', ' ', 'Spacebar'].includes(event.key)) event.preventDefault();
  }, true);
  window.addEventListener('hashchange', event => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && event.oldURL === pending.oldURL && event.newURL === pending.url && location.href === pending.url) {
      pending.awaitingHash = false; pending.ownPop = false;
    } else pending = null;
  });
  window.addEventListener('popstate', () => {
    if (current) current.focus = null;
    if (pending?.awaitingHash && pending.ownPop && location.href === pending.url) pending.ownPop = false;
    else pending = null;
  });
  if (host) new MutationObserver(() => {
    if (settling) return;
    settling = true; queueMicrotask(() => { settling = false; synchronize(); });
  }).observe(host, { childList: true, subtree: true, characterData: true, attributes: true,
    attributeFilter: ['data-daily-source', 'data-daily-context', 'data-daily-mode', 'data-daily-available',
      'data-daily-record', 'data-daily-kind', 'data-daily-state', 'data-daily-search-text'] });
  synchronize();
})();
"#;
