export function mountShellChrome() {
  const backdrop = document.querySelector('[data-command-backdrop]');
  const input = backdrop?.querySelector('input');

  function openPalette() {
    if (!backdrop) return;
    backdrop.hidden = false;
    requestAnimationFrame(() => input?.focus());
  }

  function closePalette() {
    if (!backdrop) return;
    backdrop.hidden = true;
  }

  document.querySelectorAll('[data-command-trigger]').forEach((trigger) => {
    if (trigger.dataset.shellChromeBound === 'true') return;
    trigger.dataset.shellChromeBound = 'true';
    trigger.addEventListener('click', openPalette);
  });

  backdrop?.addEventListener('click', (event) => {
    if (event.target === backdrop) closePalette();
  });

  document.addEventListener('keydown', (event) => {
    const key = event.key.toLowerCase();
    if ((event.metaKey || event.ctrlKey) && key === 'k') {
      event.preventDefault();
      openPalette();
    }
    if (event.key === 'Escape') closePalette();
  });

  document.querySelectorAll('.rail-nav').forEach((item) => {
    if (item.dataset.shellChromeBound === 'true') return;
    item.dataset.shellChromeBound = 'true';
    item.addEventListener('click', () => {
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.remove('active'));
      item.classList.add('active');
    });
  });

  backdrop?.querySelectorAll('.command-results button').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', closePalette);
  });
}

export function mountDashboardFallback() {
  mountShellChrome();

  const root = document.querySelector('#oya-dashboard-island-root');
  if (!root || root.dataset.visualFallbackMounted === 'true') return;
  root.dataset.visualFallbackMounted = 'true';

  const setActiveSurface = (surface) => {
    document.querySelectorAll('.surface-command').forEach((item) => {
      item.classList.toggle('active', item.textContent.toLowerCase().includes(surface));
    });
  };

  document.querySelectorAll('.surface-command, .surface-card').forEach((item) => {
    item.addEventListener('click', () => {
      const text = item.textContent.toLowerCase();
      if (text.includes('messenger')) activateHub('Messenger');
      if (text.includes('mail')) activateHub('Mail');
      if (text.includes('community')) activateHub('Community');
      if (text.includes('workflow')) setActiveSurface('workflow');
    });
  });

  const hub = root.querySelector('.interactive-hub');
  const hubList = hub?.querySelector('.hub-list');
  const hubDetail = hub?.querySelector('.hub-detail article');
  const textarea = hub?.querySelector('textarea');
  const queueButton = hub?.querySelector('.composer-actions button');
  let activeHub = 'Messenger';
  const channels = {
    Messenger: Array.from(hubList?.querySelectorAll('.hub-item') ?? []).map(buttonToItem),
    Mail: templateItems('template[data-mail-preview]'),
    Community: templateItems('template[data-community-preview]'),
  };

  function buttonToItem(button) {
    return {
      source: button.querySelector('span')?.textContent ?? 'Local',
      title: button.querySelector('strong')?.textContent ?? 'Item',
      body: button.querySelector('p')?.textContent ?? '',
    };
  }

  function templateItems(selector) {
    const template = root.querySelector(selector);
    if (!template) return [];
    const holder = document.createElement('div');
    holder.innerHTML = template.getAttribute(selector.includes('mail') ? 'data-mail-preview' : 'data-community-preview') ?? '';
    return Array.from(holder.querySelectorAll('.hub-item')).map(buttonToItem);
  }

  function renderHubList() {
    if (!hubList) return;
    hubList.innerHTML = '';
    (channels[activeHub] ?? []).forEach((item, index) => {
      const button = document.createElement('button');
      button.type = 'button';
      button.className = `hub-item${index === 0 ? ' active' : ''}`;
      button.innerHTML = `<span>${escapeHtml(item.source)}</span><strong>${escapeHtml(item.title)}</strong><p>${escapeHtml(item.body)}</p>`;
      button.addEventListener('click', () => {
        hubList.querySelectorAll('.hub-item').forEach((node) => node.classList.remove('active'));
        button.classList.add('active');
        renderHubDetail(item);
      });
      hubList.appendChild(button);
    });
    renderHubDetail((channels[activeHub] ?? [])[0]);
  }

  function renderHubDetail(item) {
    if (!hubDetail) return;
    hubDetail.innerHTML = item
      ? `<p class="eyebrow">${escapeHtml(activeHub)}</p><h4>${escapeHtml(item.title)}</h4><p>${escapeHtml(item.body)}</p><span class="hub-meta">Visual-only; no backend send</span>`
      : `<p class="eyebrow">${escapeHtml(activeHub)}</p><h4>No visible items</h4><p>Queue a local draft to preview this channel.</p>`;
  }

  function activateHub(label) {
    activeHub = label;
    setActiveSurface(label.toLowerCase());
    hub?.querySelectorAll('.hub-tab').forEach((tab) => {
      tab.classList.toggle('active', tab.textContent.trim() === label);
      tab.setAttribute('aria-selected', String(tab.textContent.trim() === label));
    });
    renderHubList();
  }

  hub?.querySelectorAll('.hub-tab').forEach((tab) => {
    tab.addEventListener('click', () => activateHub(tab.textContent.trim()));
  });

  queueButton?.addEventListener('click', () => {
    const body = textarea?.value.trim();
    if (!body) return;
    channels[activeHub] = [{ source: 'Local draft', title: 'Local draft queued', body }, ...(channels[activeHub] ?? [])];
    textarea.value = '';
    renderHubList();
  });

  const workflow = root.querySelector('#workflow-studio');
  const runChip = workflow?.querySelector('.workflow-run-chip');
  const status = workflow?.querySelector('.workflow-statusbar');
  const nodeToolbar = workflow?.querySelector('.node-toolbar');
  const inspector = workflow?.querySelector('.node-inspector');
  let localBlocks = 0;

  function setWorkflowMode(mode) {
    workflow?.querySelectorAll('.workflow-modebar button, .workflow-toolbar button').forEach((button) => {
      button.classList.toggle('active', button.textContent.trim() === mode);
    });
    workflow?.querySelectorAll('.workflow-node-group').forEach((node) => {
      node.classList.toggle('connectable', mode === 'Connect');
      node.classList.toggle('simulating', mode === 'Simulate');
    });
    if (runChip) runChip.lastChild.textContent = mode === 'Simulate' ? 'simulation preview' : `draft · ${mode.toLowerCase()} mode`;
    if (status?.lastElementChild) status.lastElementChild.textContent = mode === 'Connect' ? 'Click nodes to visualize links' : mode === 'Simulate' ? 'Previewing run path only' : 'Ready · mock';
  }

  workflow?.querySelectorAll('.workflow-modebar button, .workflow-toolbar button').forEach((button) => {
    button.addEventListener('click', () => setWorkflowMode(button.textContent.trim()));
  });

  workflow?.querySelectorAll('.workflow-actions button, .workflow-palette button').forEach((button) => {
    button.addEventListener('click', () => {
      const label = button.textContent.trim();
      if (label === 'Preview run') setWorkflowMode('Simulate');
      if (label === 'Add block' || ['Trigger', 'Policy check', 'Approval', 'Evidence note'].includes(label)) {
        localBlocks += 1;
        if (status?.children[1]) status.children[1].textContent = `Local blocks: ${localBlocks}`;
        const node = document.createElement('button');
        node.type = 'button';
        node.textContent = `Draft block ${localBlocks}`;
        node.addEventListener('click', () => renderInspector(node.textContent, 'Local', 'Local visual-only block added in the prototype.'));
        nodeToolbar?.appendChild(node);
        renderInspector(node.textContent, 'Local', 'Local visual-only block added in the prototype.');
      }
      if (label.includes('Messenger')) activateHub('Messenger');
      if (label.includes('Mail')) activateHub('Mail');
      if (label.includes('Community')) activateHub('Community');
    });
  });

  nodeToolbar?.querySelectorAll('button').forEach((button) => {
    button.addEventListener('click', () => renderInspector(button.textContent.trim(), 'Selected', 'Selected from the visual workflow canvas.'));
  });

  function renderInspector(title, kind, body) {
    if (!inspector) return;
    inspector.innerHTML = `<p class="eyebrow">Selected node</p><h4>${escapeHtml(title)}</h4><p><strong>${escapeHtml(kind)}</strong> · ${escapeHtml(body)}</p>`;
  }

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[char]));
  }
}
