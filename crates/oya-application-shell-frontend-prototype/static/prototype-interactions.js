export function mountShellChrome() {
  const backdrop = document.querySelector('[data-command-backdrop]');
  const input = backdrop?.querySelector('input');
  const sidePeek = document.querySelector('[data-side-peek]');
  const utilityPanels = document.querySelectorAll('[data-utility-panel]');
  const utilityBackdrop = document.querySelector('[data-utility-backdrop]');

  function openPalette() {
    if (!backdrop) return;
    backdrop.hidden = false;
    setCommandStatus('15 commands · REC-WF-7741 · cell-us-east-2 · local visual routes only');
    backdrop.querySelectorAll('[data-command-proof-action]').forEach((item) => item.classList.remove('is-selected'));
    requestAnimationFrame(() => input?.focus());
  }

  function closePalette() {
    if (!backdrop) return;
    backdrop.hidden = true;
  }

  function setCommandStatus(message) {
    const status = backdrop?.querySelector('[data-command-status]');
    if (status && message) status.textContent = message;
  }

  function openSidePeek(trigger) {
    if (!sidePeek) return;
    if (trigger?.dataset) {
      setText('[data-sidepeek-title-target]', trigger.dataset.sidepeekTitle);
      setText('[data-sidepeek-id-target]', trigger.dataset.sidepeekId);
      setText('[data-sidepeek-desc-target]', trigger.dataset.sidepeekDesc);
      setText('[data-sidepeek-owner-target]', trigger.dataset.sidepeekOwner);
      setText('[data-sidepeek-risk-target]', trigger.dataset.sidepeekRisk);
      setText('[data-sidepeek-sla-target]', trigger.dataset.sidepeekSla);
    }
    const status = sidePeek.querySelector('[data-sidepeek-status]');
    if (status) status.textContent = 'Inspector ready · REC-WF-7741 · cell-us-east-2 · local visual state only.';
    sidePeek.querySelectorAll('[data-sidepeek-route], [data-sidepeek-action]').forEach((item) => item.classList.remove('is-selected'));
    sidePeek.classList.add('open');
    sidePeek.setAttribute('aria-hidden', 'false');
    sidePeek.querySelector('[data-sidepeek-close]')?.focus();
  }

  function closeSidePeek() {
    if (!sidePeek) return;
    sidePeek.classList.remove('open');
    sidePeek.setAttribute('aria-hidden', 'true');
  }

  function openUtilityPanel(name) {
    if (utilityBackdrop) {
      utilityBackdrop.hidden = false;
      utilityBackdrop.classList.add('open');
    }
    utilityPanels.forEach((panel) => {
      const active = panel.dataset.utilityPanel === name;
      panel.classList.toggle('open', active);
      panel.setAttribute('aria-hidden', String(!active));
      if (active) requestAnimationFrame(() => panel.querySelector('[data-utility-close]')?.focus());
    });
  }

  function closeUtilityPanels() {
    if (utilityBackdrop) {
      utilityBackdrop.classList.remove('open');
      utilityBackdrop.hidden = true;
    }
    utilityPanels.forEach((panel) => {
      panel.classList.remove('open');
      panel.setAttribute('aria-hidden', 'true');
    });
  }

  function activityItems() {
    return Array.from(document.querySelectorAll('[data-activity-item]'));
  }

  function updateActivityCount() {
    const unread = activityItems().filter((item) => item.dataset.activityState === 'unread').length;
    document.querySelectorAll('[data-activity-count], [data-activity-badge]').forEach((node) => {
      node.textContent = String(unread);
      node.hidden = unread === 0 && node.dataset.activityBadge === 'true';
    });
  }

  function applyActivityFilter(filter = 'all') {
    activityItems().forEach((item) => {
      const unread = item.dataset.activityState === 'unread';
      const blocking = item.dataset.activitySeverity === 'blocking';
      item.hidden = (filter === 'unread' && !unread) || (filter === 'blocking' && !blocking);
    });
  }

  function pushActivity({ title = 'Local activity staged', body = 'Visual-only browser event.', severity = 'info' } = {}) {
    const list = document.querySelector('[data-activity-list]');
    if (!list) return;
    const item = document.createElement('li');
    item.dataset.activityItem = 'true';
    item.dataset.activityState = 'unread';
    item.dataset.activitySeverity = severity;
    const chip = severity === 'blocking' ? 'status-chip danger' : severity === 'review' ? 'status-chip warning' : 'status-chip';
    item.innerHTML = `<time>now</time><span class="${chip}">${escapeHtml(severity)}</span><strong>${escapeHtml(title)}</strong><p>${escapeHtml(body)}</p><button type="button" data-activity-action="mark-read">Mark read</button>`;
    list.prepend(item);
    bindActivityItem(item);
    updateActivityCount();
  }

  window.oyaPushActivity = pushActivity;

  const productRouteCopy = {
    fd001: {
      route: 'FD-001 graph',
      title: 'FD-001 graph · product substrate',
      body: 'Service catalog, workflow, Messenger, Mail, Community, cloud posture, and evidence receipts are visible as one operating graph.',
      status: 'FD-001 service graph active · Oyatie Cloud dogfood cell-us-east-2 · local visual route',
      target: '#service-catalog',
      severity: 'info',
    },
    workflow: {
      route: 'Workflow',
      title: 'Workflow · governed runbook',
      body: 'Payroll close DAG, visual rules, simulation overlays, and the right-side inspector are active without workflow execution.',
      status: 'Workflow Studio active · selective WASM island · no workflow execution',
      target: '#workflow-studio',
      severity: 'info',
    },
    daily: {
      route: 'Action Inbox',
      title: 'Action Inbox · priority work queue',
      body: 'Daily work, approvals, schedules, and review packets stay tied to the same FD-001 workload graph.',
      status: 'Action Inbox active · priority work inherits the command shell route context',
      target: '#command-center-workbench',
      severity: 'review',
    },
    messenger: {
      route: 'Messenger',
      title: 'Messenger · ops room thread',
      body: 'Operational chat extracts actions, links rollback evidence, and stays local to the browser session.',
      status: 'Messenger Work Hub active · no external post · FD-001 tenant workload preview',
      target: '#work-hub',
      severity: 'info',
    },
    mail: {
      route: 'Mail',
      title: 'Mail · formal approval brief',
      body: 'Structured mail preview carries recipients, evidence attachments, signoff checks, and disabled delivery state.',
      status: 'Mail Work Hub active · send preview only · no external delivery',
      target: '#work-hub',
      severity: 'review',
    },
    community: {
      route: 'Community',
      title: 'Community · governance council post',
      body: 'Role-aware community post, audience checks, moderation, and council digest are ready as visual local state.',
      status: 'Community Work Hub active · role-aware publication preview · local only',
      target: '#work-hub',
      severity: 'info',
    },
    cloud: {
      route: 'Oyatie Cloud',
      title: 'Oyatie Cloud · tenant cell',
      body: 'Cell topology, deployment gates, FinOps, residency, and rollback posture prove the FD-001 substrate claim.',
      status: 'Oyatie Cloud substrate active · topology/gates visible · no cloud mutation',
      target: '#cloud-ops-cockpit',
      severity: 'review',
    },
    finance: {
      route: 'Finance',
      title: 'Finance · close and ledger command',
      body: 'Payroll, ledger, vendors, billing, tax, and leave controls share the April close proof workload.',
      status: 'Finance command active · close and ledger panels inherit FD-001 context',
      target: '#finance-commercial-service',
      severity: 'review',
    },
    identity: {
      route: 'Identity',
      title: 'Identity · org and access envelope',
      body: 'Auth, org profile, sessions, workforce, and onboarding panels stay inside the same tenant role boundary.',
      status: 'Identity command active · org/access panels inherit tenant context',
      target: '#identity-workforce-service',
      severity: 'info',
    },
    evidence: {
      route: 'Evidence',
      title: 'Evidence · audit receipt',
      body: 'Immutable receipt and object graph show what would be proved without persisting or sending anything.',
      status: 'Evidence spine active · REC-FD001-CLOUD-009 · review only',
      target: '#audit-ledger',
      severity: 'review',
    },
    boundary: {
      route: 'Local boundary',
      title: 'Local boundary · unwired mock',
      body: 'Prototype remains visually interactive but performs no backend, workflow, mail, IAM, billing, deploy, or cloud mutation.',
      status: 'Local boundary highlighted · every action remains browser visual state',
      target: '#audit-ledger',
      severity: 'review',
    },
  };

  function normalizeProductRoute(route) {
    const value = String(route || '').toLowerCase();
    if (value === 'work-hub') return 'messenger';
    if (value === 'comms' || value === 'communications') return 'messenger';
    if (value === 'receipt' || value === 'audit') return 'evidence';
    if (value === 'action-inbox' || value === 'workbench') return 'daily';
    if (value === 'service-graph' || value === 'catalog' || value === 'modules') return 'fd001';
    return productRouteCopy[value] ? value : 'fd001';
  }

  function setProductActivity(route, message, options = {}) {
    const key = normalizeProductRoute(route);
    const copy = productRouteCopy[key] || productRouteCopy.fd001;
    const statusText = message || copy.status;

    document.querySelectorAll('[data-global-activity-status]').forEach((node) => { node.textContent = statusText; });
    document.querySelectorAll('[data-spine-active-route]').forEach((node) => { node.textContent = copy.route; });
    document.querySelectorAll('[data-spine-inspector-title]').forEach((node) => { node.textContent = copy.title; });
    document.querySelectorAll('[data-spine-inspector-body]').forEach((node) => { node.textContent = copy.body; });
    document.querySelectorAll('[data-spine-last-action]').forEach((node) => { node.textContent = `${copy.route} · ${options.source || 'shell'} · ${new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`; });
    document.querySelectorAll('[data-command-shell-route]').forEach((node) => { node.textContent = copy.route; });
    document.querySelectorAll('[data-command-shell-target]').forEach((node) => { node.textContent = copy.target || 'local route'; });
    document.querySelectorAll('[data-command-shell-status]').forEach((node) => { node.textContent = statusText; });
    document.querySelectorAll('[data-command-shell-updated]').forEach((node) => { node.textContent = options.source || 'shell'; });
    document.querySelectorAll('[data-activity-route]').forEach((node) => {
      const selected = normalizeProductRoute(node.dataset.activityRoute) === key;
      node.classList.toggle('selected', selected);
      node.classList.toggle('is-selected', selected);
      if (node.tagName === 'BUTTON') node.setAttribute('aria-pressed', String(selected));
    });
    document.querySelectorAll('[data-spine-step]').forEach((node) => {
      node.classList.toggle('selected', normalizeProductRoute(node.dataset.spineStep) === key);
    });
    document.querySelectorAll('[data-shell-context-route]').forEach((node) => {
      const selected = normalizeProductRoute(node.dataset.shellContextRoute) === key;
      node.classList.toggle('selected', selected);
      node.classList.toggle('is-selected', selected);
      node.setAttribute('aria-pressed', String(selected));
    });
    document.body.dataset.productActivityRoute = key;
  }

  function routeProductActivity(key, source = 'product activity spine') {
    const route = normalizeProductRoute(key);
    const copy = productRouteCopy[route] || productRouteCopy.fd001;
    if (route === 'fd001') activateServiceCatalog();
    else if (route === 'daily') activateDailyExecution();
    else if (route === 'workflow') routeToLocalTarget('#workflow-studio', 'Product activity Workflow');
    else if (route === 'messenger' || route === 'mail' || route === 'community') {
      const label = copy.route;
      activateSurfaceFromShell(label);
      markCommsSurface(label);
      setCommsRouteStatus(label, source);
      window.history.replaceState(null, '', '#work-hub');
    } else if (route === 'cloud') activateCockpitPanel('topology');
    else if (route === 'finance') activateFinancePanelFromShell('ledger');
    else if (route === 'identity') activateIdentityPanelFromShell('auth');
    else if (route === 'evidence' || route === 'boundary') activateResourcePanel('audit');
    if (copy.target) window.history.replaceState(null, '', copy.target);
    setProductActivity(route, copy.status, { source });
    pushActivity?.({
      title: `${copy.route} selected from ${source}`,
      body: copy.body,
      severity: copy.severity,
    });
  }

  window.oyaSetProductActivity = setProductActivity;
  window.oyaRouteProductActivity = routeProductActivity;

  setProductActivity('fd001', undefined, { source: 'initial SSR shell' });

  document.querySelectorAll('[data-activity-route]').forEach((button) => {
    if (button.dataset.productActivityBound === 'true') return;
    button.dataset.productActivityBound = 'true';
    button.addEventListener('click', (event) => {
      event.preventDefault();
      routeProductActivity(button.dataset.activityRoute, 'product activity spine');
    });
  });

  document.querySelectorAll('[data-shell-context-route]').forEach((button) => {
    if (button.dataset.commandShellBound === 'true') return;
    button.dataset.commandShellBound = 'true';
    button.addEventListener('click', (event) => {
      event.preventDefault();
      routeProductActivity(button.dataset.shellContextRoute, 'command shell substrate');
    });
  });

  const lowerPanelRouteMap = [
    ['#command-center-workbench', 'daily'],
    ['#daily-execution', 'daily'],
    ['#business-logics', 'fd001'],
    ['#service-catalog', 'fd001'],
    ['#workflow-studio', 'workflow'],
    ['#work-hub', 'messenger'],
    ['#finance-commercial-service', 'finance'],
    ['#identity-workforce-service', 'identity'],
    ['#cloud-ops-cockpit', 'cloud'],
    ['#resource-audit-console', 'evidence'],
    ['#ontology-command-console', 'fd001'],
    ['#intelligence-command-console', 'fd001'],
  ];

  function mountLowerPanelContextObserver() {
    if (!('IntersectionObserver' in window)) return;
    const observer = new IntersectionObserver((entries) => {
      const visible = entries
        .filter((entry) => entry.isIntersecting)
        .sort((a, b) => b.intersectionRatio - a.intersectionRatio)[0];
      if (!visible) return;
      const route = visible.target.dataset.commandShellPanelRoute;
      const key = normalizeProductRoute(route);
      const copy = productRouteCopy[key] || productRouteCopy.fd001;
      setProductActivity(key, `${copy.route} panel in view · lower command shell synchronized`, { source: 'scroll context' });
    }, { rootMargin: '-18% 0px -58% 0px', threshold: [0.2, 0.35, 0.5, 0.65] });

    lowerPanelRouteMap.forEach(([selector, route]) => {
      const panel = document.querySelector(selector);
      if (!panel) return;
      panel.dataset.commandShellPanelRoute = route;
      observer.observe(panel);
    });
  }

  mountLowerPanelContextObserver();

  document.querySelectorAll('.surface-command').forEach((command) => {
    if (command.dataset.productActivityBound === 'true') return;
    command.dataset.productActivityBound = 'true';
    command.addEventListener('click', (event) => {
      const label = command.querySelector('span')?.textContent?.trim() || command.textContent.trim();
      const route = label.includes('Workflow') ? 'workflow' : label.toLowerCase();
      event.preventDefault();
      routeProductActivity(route, 'surface command bar');
      document.querySelectorAll('.surface-command').forEach((item) => item.classList.toggle('active', item === command));
    });
  });

  document.querySelectorAll('[data-command-trigger]').forEach((trigger) => {
    if (trigger.dataset.shellChromeBound === 'true') return;
    trigger.dataset.shellChromeBound = 'true';
    trigger.addEventListener('click', openPalette);
  });

  document.querySelectorAll('[data-hero-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.heroAction;
      const status = document.querySelector('[data-hero-status]');
      const labels = {
        'close-april': 'April close package staged locally · FD-001 workload proof ready',
        'route-ledger': 'Ledger close cockpit opened from hero · local visual route',
        'route-cloud': 'Oyatie Cloud substrate proof opened from hero · no cloud mutation',
        'route-evidence': 'Close evidence receipt opened from hero · review only',
      };
      if (action === 'close-april') routeToLocalTarget('#ledger-preview', 'April close package');
      if (action === 'route-ledger') routeToLocalTarget('#ledger-preview', 'Hero ledger route');
      if (action === 'route-cloud') routeToLocalTarget('#cloud-ops-cockpit', 'Hero cloud proof route');
      if (action === 'route-evidence') routeToLocalTarget('#audit-ledger', 'Hero close evidence route');
      document.querySelectorAll('[data-hero-action]').forEach((item) => {
        item.classList.toggle('is-selected', item.dataset.heroAction === action);
      });
      const statusText = labels[action] || 'Hero command staged locally';
      if (status) status.textContent = statusText;
      pushActivity?.({
        title: statusText,
        body: 'Top-fold close command stayed inside FD-001 / Oyatie Cloud local visual boundaries.',
        severity: action === 'route-evidence' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-render-arch-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.renderArchAction;
      const status = document.querySelector('[data-render-arch-status]');
      const labels = {
        ssr: 'SSR shell highlighted · navigation and tenant proof render before hydration',
        islands: 'Selective WASM islands highlighted · Workflow and Work Hub hydrate local interaction only',
        boundary: 'Prototype boundary highlighted · no workflow, mail, IAM, billing, deploy, or cloud mutation',
      };
      if (action === 'ssr') {
        document.querySelector('#prototype-shell')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#prototype-shell');
      }
      if (action === 'islands') routeToLocalTarget('#workflow-studio', 'Selective WASM island');
      if (action === 'boundary') routeToLocalTarget('#audit-ledger', 'Prototype local boundary');
      document.querySelectorAll('[data-render-arch-action]').forEach((item) => {
        item.classList.toggle('is-selected', item.dataset.renderArchAction === action);
      });
      document.querySelectorAll('[data-render-arch-card]').forEach((card) => {
        card.classList.toggle('selected', card.dataset.renderArchCard === action);
      });
      if (status) status.textContent = labels[action] || 'Render architecture route staged locally';
      pushActivity?.({
        title: labels[action] || 'Render architecture route staged',
        body: 'SSR shell, selective WASM islands, and local-only boundaries stayed explicit in the top-fold prototype.',
        severity: action === 'boundary' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-sidepeek-trigger]').forEach((trigger) => {
    if (trigger.dataset.shellChromeBound === 'true') return;
    trigger.dataset.shellChromeBound = 'true';
    trigger.addEventListener('click', () => openSidePeek(trigger));
  });

  sidePeek?.querySelectorAll('[data-sidepeek-close]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', closeSidePeek);
  });

  sidePeek?.querySelectorAll('[data-sidepeek-route]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const route = button.dataset.sidepeekRoute;
      const status = sidePeek.querySelector('[data-sidepeek-status]');
      const labels = {
        workload: 'FD-001 workload graph opened from object inspector · local only',
        cloud: 'Oyatie Cloud cell posture opened from object inspector · no cloud mutation',
        evidence: 'REC-WF-7741 evidence spine opened from object inspector · review only',
      };
      const targets = {
        workload: '#object-graph',
        cloud: '#cloud-ops-cockpit',
        evidence: '#audit-ledger',
      };
      if (route === 'workload') routeToLocalTarget('#object-graph', 'Object inspector workload graph');
      if (route === 'cloud') routeToLocalTarget('#cloud-ops-cockpit', 'Object inspector cloud cell');
      if (route === 'evidence') routeToLocalTarget('#audit-ledger', 'Object inspector evidence receipt');
      if (targets[route]) window.history.replaceState(null, '', targets[route]);
      sidePeek.querySelectorAll('[data-sidepeek-route]').forEach((item) => item.classList.toggle('is-selected', item === button));
      if (status) status.textContent = labels[route] || 'Object route opened locally';
      pushActivity?.({
        title: labels[route] || 'Object inspector route opened',
        body: 'FD-001 / Oyatie Cloud object context changed locally with no backend mutation.',
        severity: route === 'evidence' ? 'review' : 'info',
      });
    });
  });

  sidePeek?.querySelectorAll('[data-sidepeek-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.sidepeekAction;
      const status = sidePeek.querySelector('[data-sidepeek-status]');
      const labels = {
        'assign-owner': 'Owner assignment staged locally for FD-001 object context',
        'draft-note': 'Messenger note drafted locally from object inspector',
        'review-evidence': 'Evidence review opened for REC-WF-7741 from object inspector',
      };
      if (action === 'draft-note') activateSurfaceFromShell('Messenger');
      if (action === 'review-evidence') routeToLocalTarget('#audit-ledger', 'Object inspector evidence review');
      sidePeek.querySelectorAll('[data-sidepeek-action]').forEach((item) => item.classList.toggle('is-selected', item === button));
      if (status) status.textContent = labels[action] || 'Object action staged locally';
      pushActivity?.({
        title: labels[action] || 'Object action staged',
        body: 'Object inspector action stayed visual-only in this browser session.',
        severity: action === 'review-evidence' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-header-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      if (button.dataset.headerAction === 'notifications') openUtilityPanel('notifications');
      if (button.dataset.headerAction === 'settings') openUtilityPanel('settings');
    });
  });

  document.querySelectorAll('[data-header-route]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const route = button.dataset.headerRoute;
      const status = document.querySelector('[data-header-route-status]');
      const labels = {
        fd001: 'FD-001 service graph opened from header quick route · local visual route',
        cloud: 'Oyatie Cloud substrate opened from header quick route · no cloud mutation',
        'work-hub': 'Work Hub opened at Messenger · Mail and Community quick routes remain visible · local only',
        evidence: 'REC-WF-7741 evidence ledger opened from header quick route · review only',
      };
      const targets = {
        fd001: '#service-graph',
        cloud: '#cloud-ops-cockpit',
        'work-hub': '#work-hub',
        evidence: '#audit-ledger',
      };
      const activeNavTargets = {
        fd001: '#modules-title',
        cloud: '#cloud-ops-cockpit',
        'work-hub': '#work-hub',
        evidence: '#audit-ledger',
      };

      if (route === 'fd001') {
        activateServiceCatalog();
        document.querySelector('#service-graph')?.scrollIntoView({ block: 'center' });
      }
      if (route === 'cloud') activateCockpitPanel('topology');
      if (route === 'work-hub') {
        activateSurfaceFromShell('Messenger');
        markCommsSurface('Messenger');
        setCommsRouteStatus('Messenger', 'header Work Hub');
      }
      if (route === 'evidence') activateResourcePanel('audit');
      if (targets[route]) window.history.replaceState(null, '', targets[route]);

      document.querySelectorAll('[data-header-route]').forEach((item) => item.classList.toggle('is-selected', item === button));
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.toggle('active', nav.getAttribute('href') === activeNavTargets[route]));
      if (status) status.textContent = labels[route] || 'Header route staged locally';
      pushActivity?.({
        title: labels[route] || 'Header quick route staged',
        body: 'The responsive header kept FD-001 tenant workload delivery and Oyatie Cloud substrate reachable when the left rail is hidden.',
        severity: route === 'evidence' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-header-comms-surface]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const label = button.dataset.headerCommsSurface;
      if (!label) return;
      activateSurfaceFromShell(label);
      markCommsSurface(label);
      window.history.replaceState(null, '', '#work-hub');
      document.querySelectorAll('[data-header-route]').forEach((item) => item.classList.toggle('is-selected', item.dataset.headerRoute === 'work-hub'));
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.toggle('active', nav.getAttribute('href') === '#work-hub'));
      const status = document.querySelector('[data-header-route-status]');
      if (status) status.textContent = `${label} selected · built-in Work Hub · FD-001 tenant workload · local only`;
      setCommsRouteStatus(label, 'header quick route');
      pushActivity?.({
        title: `${label} Work Hub surface selected`,
        body: 'Messenger, Mail, and Community stay explicit FD-001 tenant workload surfaces on Oyatie Cloud.',
        severity: 'info',
      });
    });
  });

  document.querySelectorAll('[data-utility-close]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', closeUtilityPanels);
  });

  utilityBackdrop?.addEventListener('click', closeUtilityPanels);

  backdrop?.addEventListener('click', (event) => {
    if (event.target === backdrop) closePalette();
  });

  document.addEventListener('keydown', (event) => {
    const key = event.key.toLowerCase();
    if ((event.metaKey || event.ctrlKey) && key === 'k') {
      event.preventDefault();
      openPalette();
    }
    if (event.key === 'Escape') {
      closePalette();
      closeSidePeek();
      closeUtilityPanels();
    }
  });

  input?.addEventListener('input', () => {
    const term = input.value.trim().toLowerCase();
    backdrop?.querySelectorAll('.command-results button').forEach((button) => {
      button.hidden = Boolean(term) && !button.textContent.toLowerCase().includes(term);
    });
    const visible = Array.from(backdrop?.querySelectorAll('.command-results button') ?? []).filter((button) => !button.hidden).length;
    setCommandStatus(`${visible} command${visible === 1 ? '' : 's'} visible · ${term || 'all'} · local command graph only`);
  });

  backdrop?.querySelectorAll('[data-command-proof-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.commandProofAction;
      const labels = {
        fd001: 'FD-001 service graph opened from command palette · local visual route',
        cloud: 'Oyatie Cloud substrate cells opened from command palette · no cloud mutation',
        receipt: 'REC-WF-7741 evidence receipt opened from command palette · review only',
        'local-boundary': 'Command boundary shown · no backend, workflow, mail, IAM, billing, deploy, or cloud mutation',
      };
      const targets = {
        fd001: '#service-graph',
        cloud: '#cloud-ops-cockpit',
        receipt: '#audit-ledger',
      };
      if (action === 'fd001') {
        activateServiceCatalog();
        window.history.replaceState(null, '', targets.fd001);
      }
      if (action === 'cloud') {
        activateCockpitPanel('topology');
        window.history.replaceState(null, '', targets.cloud);
      }
      if (action === 'receipt') {
        activateResourcePanel('audit');
        window.history.replaceState(null, '', targets.receipt);
      }
      backdrop.querySelectorAll('[data-command-proof-action]').forEach((item) => item.classList.toggle('is-selected', item === button));
      setCommandStatus(labels[action] || 'Command proof route staged locally');
      pushActivity?.({
        title: labels[action] || 'Command proof route staged',
        body: 'The command palette stayed inside FD-001 / Oyatie Cloud visual-only boundaries.',
        severity: action === 'receipt' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('.rail-nav').forEach((item) => {
    if (item.dataset.shellChromeBound === 'true') return;
    item.dataset.shellChromeBound = 'true';
    item.addEventListener('click', (event) => {
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.remove('active'));
      item.classList.add('active');
      const href = item.getAttribute('href');
      const deepRoutes = {
        '#identity-employees': () => activateIdentityPanelFromShell('employees'),
        '#identity-workforce-service': () => activateIdentityPanelFromShell('auth'),
        '#business-logics': () => activateBusinessLogics(),
        '#payroll-cockpit': () => activateBusinessLogics(),
        '#filing-readiness': () => activateBusinessLogics(),
        '#governance-analytics': () => activateBusinessLogics(),
        '#ledger-preview': () => activateFinancePanelFromShell('ledger'),
        '#vendors-spend': () => activateFinancePanelFromShell('vendors'),
        '#billing-tax': () => activateFinancePanelFromShell('billing'),
        '#leave-time': () => activateFinancePanelFromShell('leave'),
        '#cloud-ops-cockpit': () => activateCockpitPanel('topology'),
        '#policy-access': () => activateCockpitPanel('policy'),
        '#finops-pane': () => activateCockpitPanel('finops'),
        '#resource-inventory': () => activateResourcePanel('inventory'),
        '#audit-ledger': () => activateResourcePanel('audit'),
        '#deployment-gates': () => activateResourcePanel('gates'),
        '#modules-title': () => activateServiceCatalog(),
        '#service-catalog': () => activateServiceCatalog(),
        '#service-graph': () => activateServiceCatalog(),
        '#daily-execution': () => activateDailyExecution(),
        '#tasks-title': () => activateDailyExecution(),
        '#schedule-title': () => activateDailyExecution(),
        '#evidence-spine': () => activateEvidenceConsole(),
        '#evidence-ledger': () => activateEvidenceConsole('all'),
        '#object-graph': () => activateEvidenceConsole('graph'),
      };
      if (href && deepRoutes[href]) {
        event.preventDefault();
        window.history.replaceState(null, '', href);
        deepRoutes[href]();
      }
    });
  });

  document.querySelectorAll('[data-rail-proof-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.railProofAction;
      const status = document.querySelector('[data-rail-status]');
      const labels = {
        'service-graph': 'FD-001 service graph opened from persistent rail · local visual route',
        cloud: 'Oyatie Cloud substrate opened from persistent rail · no cloud mutation',
        evidence: 'REC-WF-7741 evidence ledger opened from persistent rail · review only',
        'work-hub': 'Work Hub opened at Messenger from persistent rail · Mail and Community remain one click away · local only',
      };
      const targets = {
        'service-graph': '#service-graph',
        cloud: '#cloud-ops-cockpit',
        evidence: '#audit-ledger',
        'work-hub': '#work-hub',
      };
      const activeNavTargets = {
        'service-graph': '#modules-title',
        cloud: '#cloud-ops-cockpit',
        evidence: '#audit-ledger',
        'work-hub': '#work-hub',
      };

      if (action === 'service-graph') {
        activateServiceCatalog();
        document.querySelector('#service-graph')?.scrollIntoView({ block: 'center' });
      }
      if (action === 'cloud') activateCockpitPanel('topology');
      if (action === 'evidence') activateResourcePanel('audit');
      if (action === 'work-hub') {
        activateSurfaceFromShell('Messenger');
        markCommsSurface('Messenger');
        setCommsRouteStatus('Messenger', 'persistent rail');
      }
      if (targets[action]) window.history.replaceState(null, '', targets[action]);

      document.querySelectorAll('[data-rail-proof-action]').forEach((item) => item.classList.toggle('is-selected', item === button));
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.toggle('active', nav.getAttribute('href') === activeNavTargets[action]));
      if (status) status.textContent = labels[action] || 'Persistent rail proof route staged locally';
      pushActivity?.({
        title: labels[action] || 'Persistent rail proof route staged',
        body: 'FD-001 tenant workload delivery and Oyatie Cloud substrate stayed connected in one visual-only shell graph.',
        severity: action === 'evidence' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-rail-comms-surface]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const label = button.dataset.railCommsSurface;
      if (!label) return;
      activateSurfaceFromShell(label);
      markCommsSurface(label);
      window.history.replaceState(null, '', '#work-hub');
      document.querySelectorAll('[data-rail-proof-action]').forEach((item) => item.classList.toggle('is-selected', item.dataset.railProofAction === 'work-hub'));
      document.querySelectorAll('.rail-nav').forEach((nav) => nav.classList.toggle('active', nav.getAttribute('href') === '#work-hub'));
      const status = document.querySelector('[data-rail-status]');
      if (status) status.textContent = `${label} selected · Work Hub dogfood route · cell-us-east-2 · local only`;
      setCommsRouteStatus(label, 'persistent rail');
      pushActivity?.({
        title: `${label} selected from persistent rail`,
        body: 'Built-in Messenger, Mail, and Community routing stayed visual-only inside the FD-001 tenant workload graph.',
        severity: 'info',
      });
    });
  });

  backdrop?.querySelectorAll('.command-results button').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.commandAction;
      if (action === 'workflow') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'community') activateSurfaceFromShell('Community');
      if (action === 'peek') {
        activateEvidenceConsole();
        openSidePeek();
      }
      if (action === 'topology') activateCockpitPanel('topology');
      if (action === 'policy') activateCockpitPanel('policy');
      if (action === 'finops') activateCockpitPanel('finops');
      if (action === 'inventory') activateResourcePanel('inventory');
      if (action === 'audit') activateResourcePanel('audit');
      if (action === 'gates') activateResourcePanel('gates');
      if (action === 'catalog') activateServiceCatalog();
      if (action === 'business-logics') activateBusinessLogics();
      if (action === 'identity') activateIdentityPanelFromShell('auth');
      if (action === 'finance') activateFinancePanelFromShell('ledger');
      if (action === 'notifications') openUtilityPanel('notifications');
      if (action === 'settings') openUtilityPanel('settings');
      closePalette();
    });
  });

  document.querySelectorAll('[data-activity-filter]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-activity-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyActivityFilter(button.dataset.activityFilter || 'all');
    });
  });

  function bindActivityItem(item) {
    item.querySelectorAll('[data-activity-action]').forEach((button) => {
      if (button.dataset.activityBound === 'true') return;
      button.dataset.activityBound = 'true';
      button.addEventListener('click', () => {
        const action = button.dataset.activityAction;
        if (action === 'mark-read') {
          item.dataset.activityState = 'read';
          item.classList.add('read');
          button.textContent = 'Read';
          button.disabled = true;
          updateActivityCount();
        }
        if (action === 'open-audit') {
          activateResourcePanel('audit');
          closeUtilityPanels();
        }
      });
    });
  }

  activityItems().forEach(bindActivityItem);
  document.querySelectorAll('[data-activity-action="clear-read"]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      activityItems().filter((item) => item.dataset.activityState === 'read').forEach((item) => item.remove());
      updateActivityCount();
    });
  });

  document.querySelectorAll('[data-utility-route]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const route = button.dataset.utilityRoute;
      const panel = button.closest('[data-utility-panel]');
      const activityStatus = panel?.querySelector('[data-activity-status]');
      const settingsStatus = panel?.querySelector('[data-settings-status]');
      const labels = {
        'work-hub': 'Work Hub opened from Activity Center · FD-001 signals stay local',
        evidence: 'Evidence spine opened from Activity Center · receipt review only',
        cloud: 'Cloud cell posture opened from utility drawer · no cloud mutation',
        identity: 'Identity role envelope opened from Settings · no auth mutation',
        policy: 'Policy access matrix opened from Settings · local review only',
        catalog: 'Tenant module catalog opened from Settings · no service admission',
      };
      const routeTargets = {
        'work-hub': '#work-hub',
        evidence: '#audit-ledger',
        cloud: '#cloud-ops-cockpit',
        identity: '#identity-workforce-service',
        policy: '#policy-access',
        catalog: '#service-catalog',
      };
      if (route === 'work-hub') activateSurfaceFromShell('Messenger');
      if (route === 'evidence') activateResourcePanel('audit');
      if (route === 'cloud') activateCockpitPanel('topology');
      if (route === 'identity') activateIdentityPanelFromShell('auth');
      if (route === 'policy') activateCockpitPanel('policy');
      if (route === 'catalog') activateServiceCatalog();
      if (routeTargets[route]) window.history.replaceState(null, '', routeTargets[route]);
      panel?.querySelectorAll('[data-utility-route]').forEach((item) => {
        item.classList.toggle('is-selected', item === button);
      });
      const statusText = labels[route] || 'Utility route opened locally';
      if (activityStatus) activityStatus.textContent = statusText;
      if (settingsStatus) settingsStatus.textContent = statusText;
      pushActivity({
        title: statusText,
        body: 'FD-001 / Oyatie Cloud utility control stayed visual-only in this browser session.',
        severity: route === 'evidence' || route === 'policy' ? 'review' : 'info',
      });
    });
  });

  function inboxInputs() {
    return Array.from(document.querySelectorAll('[data-inbox-select]'));
  }

  function visibleInboxInputs() {
    return inboxInputs().filter((input) => !input.closest('[data-workbench-row]')?.hidden && !input.disabled);
  }

  function updateInboxSelection() {
    const selected = inboxInputs().filter((input) => input.checked);
    document.querySelectorAll('[data-inbox-selected-count]').forEach((node) => {
      node.textContent = String(selected.length);
    });
    document.querySelectorAll('[data-inbox-bulk]').forEach((button) => {
      button.disabled = selected.length === 0;
    });
    const selectAll = document.querySelector('[data-inbox-select-all]');
    const visible = visibleInboxInputs();
    if (selectAll) {
      selectAll.checked = visible.length > 0 && visible.every((input) => input.checked);
      selectAll.indeterminate = visible.some((input) => input.checked) && !selectAll.checked;
    }
    inboxInputs().forEach((input) => {
      input.closest('[data-inbox-row]')?.classList.toggle('selected', input.checked);
    });
    const status = document.querySelector('[data-inbox-status]');
    if (status) {
      status.textContent = selected.length === 0
        ? 'No items selected · local inbox only'
        : `${selected.length} selected · bulk actions staged locally`;
    }
  }

  function applyWorkbenchFilter(filter = 'all') {
    document.querySelectorAll('[data-workbench-filter]').forEach((item) => {
      item.classList.toggle('active', item.dataset.workbenchFilter === filter);
    });
    document.querySelectorAll('[data-workbench-row]').forEach((row) => {
      const key = row.dataset.workbenchRow || 'all';
      row.hidden = filter !== 'all' && key !== filter;
    });
    updateInboxSelection();
  }

  document.querySelectorAll('[data-workbench-filter]').forEach((button) => {
    if (button.dataset.inboxBound === 'true') return;
    button.dataset.inboxBound = 'true';
    button.addEventListener('click', () => applyWorkbenchFilter(button.dataset.workbenchFilter || 'all'));
  });

  inboxInputs().forEach((input) => {
    if (input.dataset.inboxBound === 'true') return;
    input.dataset.inboxBound = 'true';
    input.addEventListener('change', updateInboxSelection);
  });

  document.querySelectorAll('[data-inbox-select-all]').forEach((input) => {
    if (input.dataset.inboxBound === 'true') return;
    input.dataset.inboxBound = 'true';
    input.addEventListener('change', () => {
      visibleInboxInputs().forEach((item) => {
        item.checked = input.checked;
      });
      updateInboxSelection();
    });
  });

  document.querySelectorAll('[data-inbox-bulk]').forEach((button) => {
    if (button.dataset.inboxBound === 'true') return;
    button.dataset.inboxBound = 'true';
    button.addEventListener('click', () => {
      const selected = inboxInputs().filter((input) => input.checked);
      if (selected.length === 0) return;
      const action = button.dataset.inboxBulk;
      const labels = {
        approve: 'Selected inbox items approved locally',
        defer: 'Selected inbox items deferred locally',
        mail: 'Mail brief drafted for selected inbox items',
        evidence: 'Evidence bundle attached to selected inbox items',
      };
      selected.forEach((input) => input.closest('[data-inbox-row]')?.classList.add('bulk-staged'));
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'evidence') activateEvidenceConsole();
      const status = document.querySelector('[data-inbox-status]');
      if (status) status.textContent = `${labels[action] || 'Bulk action staged'} · ${selected.length} items`;
      window.oyaPushActivity?.({
        title: labels[action] || 'Action Inbox bulk operation',
        body: `${selected.length} visible inbox item(s) changed local visual state only.`,
        severity: action === 'evidence' || action === 'approve' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-inbox-row-action]').forEach((button) => {
    if (button.dataset.inboxBound === 'true') return;
    button.dataset.inboxBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.inboxRowAction;
      const row = button.closest('[data-inbox-row]');
      const title = row?.querySelector('.inbox-row-main strong')?.textContent?.trim() || 'Inbox item';
      if (action === 'workflow') routeToLocalTarget('#workflow-studio', title);
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'audit') activateResourcePanel('audit');
      const status = document.querySelector('[data-inbox-status]');
      if (status) status.textContent = `${title} ${action} route opened locally`;
    });
  });

  document.querySelectorAll('[data-settings-tab]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const panel = button.dataset.settingsTab;
      document.querySelectorAll('[data-settings-tab]').forEach((item) => {
        const active = item === button;
        item.classList.toggle('active', active);
        item.setAttribute('aria-selected', String(active));
      });
      document.querySelectorAll('[data-settings-panel]').forEach((item) => item.classList.toggle('active', item.dataset.settingsPanel === panel));
    });
  });

  document.querySelectorAll('[data-settings-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.settingsAction;
      const status = document.querySelector('[data-settings-status]');
      if (action === 'density-compact') {
        document.documentElement.dataset.density = 'compact';
        if (status) status.textContent = 'Compact density applied locally';
        pushActivity({ title: 'Compact density applied', body: 'Stored in this browser session only.', severity: 'info' });
      }
      if (action === 'density-comfortable') {
        delete document.documentElement.dataset.density;
        if (status) status.textContent = 'Comfortable density applied locally';
      }
      if (action === 'locale-ko' || action === 'locale-en') {
        if (status) status.textContent = action === 'locale-ko' ? 'Korean-first labels staged locally' : 'English labels staged locally';
      }
      if (action === 'open-identity') {
        activateIdentityPanelFromShell('auth');
        closeUtilityPanels();
      }
    });
  });

  document.querySelectorAll('[data-cockpit-tab]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => activateCockpitPanel(button.dataset.cockpitTab));
  });

  document.querySelectorAll('[data-resource-tab]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => activateResourcePanel(button.dataset.resourceTab));
  });

  function cloudWorkloadCopy(button) {
    return {
      key: button?.dataset.cockpitWorkload || 'workflow',
      title: button?.dataset.workloadTitle || button?.querySelector('strong')?.textContent?.trim() || 'FD-001 workload',
      service: button?.dataset.workloadService || 'fd001-service',
      cell: button?.dataset.workloadCell || 'us-east-2',
      state: button?.dataset.workloadState || 'review',
      route: button?.dataset.workloadRoute || 'Workflow → Work Hub → Evidence',
      receipt: button?.dataset.workloadReceipt || 'REC-FD001-CLOUD-009',
    };
  }

  function statusChipClassForWorkload(state) {
    if (state === 'sealed' || state === 'active') return 'status-chip success';
    if (state === 'policy' || state === 'drafts' || state === 'review') return 'status-chip warning';
    return 'status-chip';
  }

  function setCloudWorkload(button, source = 'cloud workload plane') {
    if (!button) return;
    const plane = button.closest('[data-cloud-workload-plane]');
    const copy = cloudWorkloadCopy(button);
    plane?.querySelectorAll('[data-cockpit-workload]').forEach((item) => {
      const selected = item === button;
      item.classList.toggle('selected', selected);
      item.classList.toggle('is-selected', selected);
      item.setAttribute('aria-pressed', String(selected));
    });
    const status = plane?.querySelector('[data-cockpit-workload-status]');
    if (status) {
      status.className = statusChipClassForWorkload(copy.state);
      status.dataset.cockpitWorkloadStatus = 'true';
      status.textContent = `${copy.title} selected · ${copy.service} · ${copy.cell} · ${copy.state} · local-only substrate proof`;
    }
    const detailMap = {
      '[data-workload-detail-service]': copy.service,
      '[data-workload-detail-cell]': copy.cell,
      '[data-workload-detail-route]': copy.route,
      '[data-workload-detail-receipt]': copy.receipt,
    };
    Object.entries(detailMap).forEach(([selector, value]) => {
      plane?.querySelectorAll(selector).forEach((node) => { node.textContent = value; });
    });
    document.querySelectorAll('[data-cockpit-status]').forEach((node) => {
      node.textContent = `${copy.title} pinned as FD-001 tenant workload on Oyatie Cloud · no runtime mutation`;
    });
    setProductActivity('cloud', `${copy.title} selected · FD-001 microservice dogfoods Oyatie Cloud tenant substrate`, { source });
    window.oyaPushActivity?.({
      title: `${copy.title} workload selected`,
      body: `${copy.service} in ${copy.cell} remains a local visual proof; no deploy, DNS, IAM, billing, or cloud mutation executed.`,
      severity: copy.state === 'sealed' || copy.state === 'active' ? 'info' : 'review',
    });
  }

  function activeCloudWorkload(plane) {
    return plane?.querySelector('[data-cockpit-workload].selected, [data-cockpit-workload].is-selected')
      || plane?.querySelector('[data-cockpit-workload]');
  }

  function routeCloudWorkload(button) {
    const route = button.dataset.cockpitWorkloadRoute || 'workflow';
    const plane = button.closest('[data-cloud-workload-plane]');
    const workload = activeCloudWorkload(plane);
    const copy = cloudWorkloadCopy(workload);
    button.closest('.ops-workload-routes')?.querySelectorAll('[data-cockpit-workload-route]').forEach((item) => {
      item.classList.toggle('active', item === button);
      item.classList.toggle('is-selected', item === button);
    });
    const statusText = `${copy.title} → ${route} route opened locally · ${copy.receipt}`;
    const status = plane?.querySelector('[data-cockpit-workload-status]');
    if (status) status.textContent = statusText;
    document.querySelectorAll('[data-cockpit-status]').forEach((node) => { node.textContent = statusText; });

    if (route === 'workflow') {
      routeToLocalTarget('#workflow-studio', `${copy.title} substrate runbook`);
    } else if (route === 'mail') {
      if (window.oyaCommsHandoff) {
        window.oyaCommsHandoff({
          destination: 'Mail',
          source: 'Oyatie Cloud',
          title: `Substrate brief · ${copy.title}`,
          body: `${copy.service} is running as an FD-001 tenant workload in ${copy.cell}. ${copy.route}. Receipt ${copy.receipt} is attached locally; no cloud or mail mutation executed.`,
          audience: 'CFO · SRE reviewer · Governance council',
          kind: 'draft',
          meta: `${copy.receipt} · cloud workload plane local draft`,
        });
      } else {
        activateSurfaceFromShell('Mail');
      }
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      window.history.replaceState(null, '', '#work-hub');
    } else if (route === 'community') {
      if (window.oyaCommsHandoff) {
        window.oyaCommsHandoff({
          destination: 'Community',
          source: 'Oyatie Cloud',
          title: `Council note · ${copy.title}`,
          body: `${copy.title} proves FD-001 tenant workload delivery can run as a tenant workload on Oyatie Cloud. ${copy.receipt} remains local evidence.`,
          audience: 'Infrastructure · Finance · People Ops · Governance',
          kind: 'draft',
          meta: `${copy.receipt} · community route staged locally`,
        });
      } else {
        activateSurfaceFromShell('Community');
      }
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      window.history.replaceState(null, '', '#work-hub');
    } else if (route === 'evidence') {
      routeToLocalTarget('#audit-ledger', `${copy.title} workload receipt`);
    } else if (route === 'gates') {
      activateResourcePanel('gates');
      window.history.replaceState(null, '', '#deployment-gates');
    }
    setProductActivity(route === 'mail' || route === 'community' ? route : route === 'gates' ? 'cloud' : route, statusText, { source: 'cloud workload plane' });
    window.oyaPushActivity?.({
      title: statusText,
      body: `${copy.service} route stayed browser-local; no production substrate mutation occurred.`,
      severity: route === 'evidence' || route === 'gates' ? 'review' : 'info',
    });
  }

  document.querySelectorAll('[data-cockpit-workload]').forEach((button) => {
    if (button.dataset.cloudWorkloadBound === 'true') return;
    button.dataset.cloudWorkloadBound = 'true';
    button.setAttribute('aria-pressed', String(button.classList.contains('selected')));
    button.addEventListener('click', () => setCloudWorkload(button));
  });

  document.querySelectorAll('[data-cockpit-workload-route]').forEach((button) => {
    if (button.dataset.cloudWorkloadRouteBound === 'true') return;
    button.dataset.cloudWorkloadRouteBound = 'true';
    button.addEventListener('click', () => routeCloudWorkload(button));
  });

  document.querySelectorAll('[data-cockpit-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.cockpitAction;
      const labels = {
        'reconcile-cell': 'Cell evidence reconciled locally · 3 receipts checked',
        'simulate-failover': 'Failover simulation queued locally · no runtime mutation',
        'queue-runbook': 'Rollback runbook staged for reviewer approval',
        'open-commit': 'Commit plan opened locally · $48.2k run-rate reviewed',
        'tag-anomaly': 'Network anomaly tagged in local FinOps ledger',
        'draft-budget-note': 'Budget note drafted for CFO review',
        'select-us-east': 'us-east-2 primary cell selected locally',
        'select-eu-west': 'eu-west-1 standby cell selected locally',
        'select-kr-seoul': 'kr-seoul residency-gated cell selected locally',
        'open-resource-inventory': 'Resource inventory opened from Cloud Ops locally',
        'open-deployment-gates': 'Deployment gates opened from Cloud Ops locally',
        'open-workflow': 'Cloud rollback workflow route opened locally',
        'open-mail': 'Cloud incident mail brief opened locally',
        'open-evidence': 'Cloud evidence spine opened locally',
        'open-finops': 'FinOps cockpit opened locally',
      };
      button.closest('.ops-cell-grid, .ops-route-grid')?.querySelectorAll('[data-cockpit-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });
      if (action === 'open-resource-inventory') activateResourcePanel('inventory');
      if (action === 'open-deployment-gates') activateResourcePanel('gates');
      if (action === 'open-workflow') routeToLocalTarget('#workflow-studio', 'Cloud rollback workflow');
      if (action === 'open-mail') activateSurfaceFromShell('Mail');
      if (action === 'open-evidence') activateEvidenceConsole();
      if (action === 'open-finops') activateCockpitPanel('finops');
      document.querySelectorAll('[data-cockpit-status]').forEach((status) => {
        status.textContent = labels[action] || 'Local ops action staged';
      });
      window.oyaPushActivity?.({
        title: labels[action] || 'Ops cockpit action staged',
        body: 'Cloud, policy, or FinOps cockpit changed local visual state only.',
        severity: action === 'queue-runbook' || action === 'tag-anomaly' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-policy-anchor-action]').forEach((button) => {
    if (button.dataset.policyAnchorBound === 'true') return;
    button.dataset.policyAnchorBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.policyAnchorAction;
      const panel = document.querySelector('#policy-access');
      const status = document.querySelector('[data-policy-anchor-status]');
      const card = button.closest('[data-policy-card]');
      const labels = {
        'role-review': 'Role grant review staged · FD-001 tenant workload policy stays local',
        'open-identity': 'Identity role envelope opened locally',
        'route-evidence': 'Policy evidence spine opened locally',
        'route-cloud': 'Oyatie Cloud topology opened as dogfood substrate proof',
        'pipa-boundary': 'PIPA residency boundary preview staged locally',
        'open-audit': 'Audit ledger opened with policy receipts',
        'autonomy-ceiling': 'Autonomy ceiling traced · no live workflow execution',
        residency: 'Residency pack gate highlighted for tenant workload placement',
        'route-mail': 'Policy mail brief opened locally',
        'route-community': 'Community policy review opened locally',
      };

      if (card && panel) {
        panel.querySelectorAll('[data-policy-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      button.closest('.policy-command-actions, .policy-anchor-routes')?.querySelectorAll('[data-policy-anchor-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });

      if (action === 'role-review' || action === 'open-identity') activateIdentityPanelFromShell('roles');
      if (action === 'route-evidence') activateEvidenceConsole(action === 'route-evidence' ? 'graph' : 'all');
      if (action === 'route-cloud' || action === 'residency') routeToLocalTarget('#cloud-ops-cockpit', 'Oyatie Cloud policy substrate');
      if (action === 'pipa-boundary') routeToLocalTarget('#governance-analytics', 'PIPA boundary');
      if (action === 'open-audit') routeToLocalTarget('#audit-ledger', 'Policy receipt ledger');
      if (action === 'autonomy-ceiling') routeToLocalTarget('#workflow-studio', 'Policy autonomy ceiling');
      if (action === 'route-mail') {
        activateSurfaceFromShell('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        activateSurfaceFromShell('Community');
        window.history.replaceState(null, '', '#work-hub');
      }

      if (status) status.textContent = labels[action] || 'Policy board action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Policy board action staged',
        body: 'Policy Access changed visual state only; FD-001 tenant workload controls remain unwired.',
        severity: ['route-evidence', 'open-audit', 'pipa-boundary', 'residency'].includes(action) ? 'review' : 'info',
      });
    });
  });

  function applyResourceSearchFilter() {
    const search = document.querySelector('[data-resource-search]');
    const status = document.querySelector('[data-resource-status]');
    const term = search?.value.trim().toLowerCase() || '';
    const activeFilter = document.querySelector('[data-resource-filter].active')?.dataset.resourceFilter || 'all';
    const rows = Array.from(document.querySelectorAll('[data-resource-row]'));
    let visible = 0;
    rows.forEach((row) => {
      const state = row.dataset.resourceState || '';
      const matchesFilter = activeFilter === 'all' || state === activeFilter;
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      row.hidden = !(matchesFilter && matchesText);
      if (!row.hidden) visible += 1;
    });
    if (status) status.textContent = `${visible} visible · ${activeFilter} filter · local inventory only`;
  }

  document.querySelector('[data-resource-search]')?.addEventListener('input', applyResourceSearchFilter);

  document.querySelectorAll('[data-resource-filter]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-resource-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyResourceSearchFilter();
    });
  });

  document.querySelectorAll('[data-resource-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const status = document.querySelector('[data-resource-status]');
      const action = button.dataset.resourceAction;
      if (status) status.textContent = action === 'export' ? 'CSV export preview staged locally' : 'Inventory mock refreshed locally';
      window.oyaPushActivity?.({
        title: action === 'export' ? 'Resource CSV export previewed' : 'Resource inventory refreshed',
        body: 'No cloud provider or audit system was contacted.',
        severity: 'info',
      });
    });
  });

  function receiptCopy(button) {
    return {
      source: button?.dataset.receiptSource || 'workflow',
      title: button?.dataset.receiptTitle || 'FD-001 receipt packet',
      id: button?.dataset.receiptId || 'REC-FD001-WF-018',
      route: button?.dataset.receiptRoute || 'Workflow → Work Hub → Evidence',
      owner: button?.dataset.receiptOwner || 'Evidence spine',
      state: button?.dataset.receiptState || 'review',
    };
  }

  function activeReceiptSource(consoleNode) {
    return consoleNode?.querySelector('[data-receipt-source].selected, [data-receipt-source].is-selected')
      || consoleNode?.querySelector('[data-receipt-source]');
  }

  function setReceiptSource(button, source = 'receipt stitching console') {
    if (!button) return;
    const consoleNode = button.closest('[data-receipt-stitching-console]');
    const copy = receiptCopy(button);
    consoleNode?.querySelectorAll('[data-receipt-source]').forEach((item) => {
      const selected = item === button;
      item.classList.toggle('selected', selected);
      item.classList.toggle('is-selected', selected);
      item.setAttribute('aria-pressed', String(selected));
    });
    const detail = {
      '[data-receipt-detail-title]': copy.title,
      '[data-receipt-detail-id]': copy.id,
      '[data-receipt-detail-route]': copy.route,
      '[data-receipt-detail-owner]': copy.owner,
    };
    Object.entries(detail).forEach(([selector, value]) => {
      consoleNode?.querySelectorAll(selector).forEach((node) => { node.textContent = value; });
    });
    const status = consoleNode?.querySelector('[data-receipt-stitching-status]');
    if (status) status.textContent = `${copy.title} selected · ${copy.id} · ${copy.route} · local proof only`;
    const auditStatus = document.querySelector('[data-audit-anchor-status]');
    if (auditStatus) auditStatus.textContent = `${copy.title} pinned in receipt stitching console · no vault mutation`;
    setProductActivity('evidence', `${copy.title} selected in universal receipt stream`, { source });
    window.oyaPushActivity?.({
      title: `${copy.title} receipt selected`,
      body: `${copy.id} joins FD-001 tenant workload delivery and Oyatie Cloud substrate proof without persistence.`,
      severity: copy.state === 'sealed' ? 'info' : 'review',
    });
  }

  function routeReceiptAction(button) {
    const action = button.dataset.receiptStitchingAction || 'graph';
    const consoleNode = button.closest('[data-receipt-stitching-console]');
    const copy = receiptCopy(activeReceiptSource(consoleNode));
    button.closest('.receipt-stitching-actions, .receipt-stitching-head')?.querySelectorAll('[data-receipt-stitching-action]').forEach((item) => {
      item.classList.toggle('active', item === button);
      item.classList.toggle('is-selected', item === button);
    });
    const statusText = `${copy.title} → ${action} routed locally · ${copy.id}`;
    const status = consoleNode?.querySelector('[data-receipt-stitching-status]');
    if (status) status.textContent = statusText;
    const auditStatus = document.querySelector('[data-audit-anchor-status]');
    if (auditStatus) auditStatus.textContent = statusText;

    if (action === 'workflow') routeToLocalTarget('#workflow-studio', `${copy.title} receipt workflow`);
    if (action === 'cloud') routeToLocalTarget('#cloud-ops-cockpit', `${copy.title} cloud proof`);
    if (action === 'graph') {
      activateEvidenceConsole('graph');
      window.history.replaceState(null, '', '#object-graph');
    }
    if (action === 'gates') {
      activateResourcePanel('gates');
      window.history.replaceState(null, '', '#deployment-gates');
    }
    if (action === 'mail') {
      if (window.oyaCommsHandoff) {
        window.oyaCommsHandoff({
          destination: 'Mail',
          source: 'Evidence spine',
          title: `Receipt brief · ${copy.title}`,
          body: `${copy.id} proves ${copy.route}. This is a local-only reviewer mail draft; no evidence vault or mail send occurred.`,
          audience: 'CFO · SRE reviewer · Governance council',
          kind: 'draft',
          meta: `${copy.id} · receipt stitching console`,
        });
      } else {
        activateSurfaceFromShell('Mail');
      }
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      window.history.replaceState(null, '', '#work-hub');
    }
    if (action === 'community') {
      if (window.oyaCommsHandoff) {
        window.oyaCommsHandoff({
          destination: 'Community',
          source: 'Evidence spine',
          title: `Council receipt note · ${copy.title}`,
          body: `${copy.id} connects FD-001 product work to Oyatie Cloud substrate proof. The note remains role-gated and local-only.`,
          audience: 'Governance · Infrastructure · Finance · People Ops',
          kind: 'draft',
          meta: `${copy.id} · community proof note`,
        });
      } else {
        activateSurfaceFromShell('Community');
      }
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      window.history.replaceState(null, '', '#work-hub');
    }
    if (action === 'seal') {
      button.textContent = 'Packet sealed';
      button.classList.add('active');
    }
    setProductActivity(action === 'mail' || action === 'community' ? action : action === 'cloud' ? 'cloud' : 'evidence', statusText, { source: 'receipt stitching console' });
    window.oyaPushActivity?.({
      title: statusText,
      body: 'Receipt stitching stayed in browser-local visual state; no audit vault, deploy, mail, billing, or cloud mutation occurred.',
      severity: action === 'seal' || action === 'gates' || action === 'graph' ? 'review' : 'info',
    });
  }

  document.querySelectorAll('[data-receipt-source]').forEach((button) => {
    if (button.dataset.receiptSourceBound === 'true') return;
    button.dataset.receiptSourceBound = 'true';
    button.setAttribute('aria-pressed', String(button.classList.contains('selected')));
    button.addEventListener('click', () => setReceiptSource(button));
  });

  document.querySelectorAll('[data-receipt-stitching-action]').forEach((button) => {
    if (button.dataset.receiptStitchingBound === 'true') return;
    button.dataset.receiptStitchingBound = 'true';
    button.addEventListener('click', () => routeReceiptAction(button));
  });

  document.querySelectorAll('[data-audit-anchor-action]').forEach((button) => {
    if (button.dataset.auditAnchorBound === 'true') return;
    button.dataset.auditAnchorBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.auditAnchorAction;
      const panel = document.querySelector('#audit-ledger');
      const status = document.querySelector('[data-audit-anchor-status]');
      const card = button.closest('[data-audit-card]');
      const receipt = button.closest('.audit-ledger-list li');
      const receiptId = receipt?.querySelector('code')?.textContent?.trim();
      const labels = {
        'open-evidence': 'Audit evidence graph opened locally',
        'route-mail': 'Audit mail brief opened locally',
        'route-cloud': 'Oyatie Cloud topology opened from audit proof stream',
        'route-gates': 'Deployment gates opened from audit proof stream',
        'seal-packet': 'Receipt packet sealed visually · no vault mutation',
        'route-policy': 'Policy board opened from audit ledger',
        'inspect-receipt': `${receiptId || 'Receipt'} inspected locally`,
        'route-workflow': 'Workflow proof route opened locally',
        'route-community': 'Community review opened locally',
      };

      if (card && panel) {
        panel.querySelectorAll('[data-audit-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      if (receipt) {
        receipt.closest('.audit-ledger-list')?.querySelectorAll('li').forEach((item) => item.classList.toggle('selected', item === receipt));
      }
      button.closest('.audit-command-actions')?.querySelectorAll('[data-audit-anchor-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });

      if (action === 'open-evidence') activateEvidenceConsole('graph');
      if (action === 'route-mail') {
        activateSurfaceFromShell('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        activateSurfaceFromShell('Community');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-cloud') routeToLocalTarget('#cloud-ops-cockpit', 'Audit cloud topology');
      if (action === 'route-gates') activateResourcePanel('gates');
      if (action === 'route-policy') routeToLocalTarget('#policy-access', 'Audit policy board');
      if (action === 'route-workflow') routeToLocalTarget('#workflow-studio', 'Audit workflow proof');
      if (action === 'seal-packet') {
        button.textContent = 'Packet sealed';
        button.classList.add('active');
      }

      if (status) status.textContent = labels[action] || 'Audit ledger action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Audit ledger action staged',
        body: 'Audit Ledger changed local visual state only; no receipt vault, deploy, or cloud mutation occurred.',
        severity: ['open-evidence', 'route-gates', 'seal-packet', 'inspect-receipt'].includes(action) ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-trust-proof-action]').forEach((button) => {
    if (button.dataset.trustProofBound === 'true') return;
    button.dataset.trustProofBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.trustProofAction;
      const board = button.closest('.trust-anchor-board');
      const localStatus = board?.querySelector('[data-trust-proof-status]');
      const labels = {
        'stage-budget': 'Budget guardrail staged locally · no billing, procurement, deploy, DNS, or cloud mutation',
        'stage-close': 'Commercial close packet staged locally · no bank, payroll, tax, invoice, database, or cloud mutation',
        'stage-filing': 'Filing packet staged locally · no HomeTax, bank, payroll, billing, mail, or cloud mutation',
        'stage-catalog': 'Service catalog manifest staged locally · no service admission, IAM, deploy, billing, or cloud mutation',
        'seal-receipt': 'Receipt packet sealed visually · no vault, mail, deploy, or cloud mutation',
        'link-resource': 'Resource lineage linked locally · no provider, inventory, deploy, or database mutation',
        'trace-lineage': 'Object lineage traced locally across FD-001 and Oyatie Cloud proof surfaces',
        'route-finance': 'Finance close opened as FD-001 workload proof',
        'route-ledger': 'Ledger close opened as FD-001 commercial proof',
        'route-vendors': 'Vendor spend opened as FD-001 procurement proof',
        'route-billing': 'Billing and filing opened as FD-001 revenue proof',
        'route-filing': 'Filing readiness opened as FD-001 localization proof',
        'route-identity': 'Identity service opened as FD-001 access proof',
        'route-daily': 'Daily Work opened as FD-001 execution proof',
        'route-finops': 'FinOps opened as Oyatie Cloud tenant workload cost proof',
        'route-cloud': 'Oyatie Cloud topology opened as the dogfood substrate',
        'route-policy': 'Policy envelope opened for tenant workload admission proof',
        'route-inventory': 'Resource inventory opened for FD-001 workload fleet proof',
        'route-audit': 'Audit ledger opened for local receipt proof',
        'route-gates': 'Deployment gates opened for FD-001 tenant workload admission',
        'route-evidence': 'Evidence spine opened for receipt graph proof',
        'route-graph': 'Object graph opened for tenant operation lineage',
        'route-workflow': 'Workflow Studio opened for governed execution proof',
        'route-mail': 'Reviewer Mail opened locally',
        'route-messenger': 'Messenger room opened locally',
        'route-community': 'Community review opened locally',
        'route-catalog': 'Service catalog opened for service graph proof',
      };

      board?.querySelectorAll('[data-trust-proof-card]').forEach((card) => {
        card.classList.toggle('selected', card === button.closest('[data-trust-proof-card]'));
      });
      board?.querySelectorAll('[data-trust-proof-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });

      if (action === 'stage-budget') button.textContent = 'Budget staged';
      if (action === 'stage-close') button.textContent = 'Close staged';
      if (action === 'stage-filing') button.textContent = 'Filing staged';
      if (action === 'stage-catalog') button.textContent = 'Manifest staged';
      if (action === 'seal-receipt') button.textContent = 'Packet sealed';
      if (action === 'link-resource') button.textContent = 'Resource linked';
      if (action === 'trace-lineage') button.textContent = 'Lineage traced';
      if (['stage-budget', 'stage-close', 'stage-filing', 'stage-catalog', 'seal-receipt', 'link-resource', 'trace-lineage'].includes(action)) {
        button.classList.add('active');
      }

      if (action === 'route-finance') {
        activateFinancePanelFromShell('ledger');
        window.history.replaceState(null, '', '#ledger-preview');
      }
      if (action === 'route-ledger') {
        activateFinancePanelFromShell('ledger');
        window.history.replaceState(null, '', '#ledger-preview');
      }
      if (action === 'route-vendors') {
        activateFinancePanelFromShell('vendors');
        window.history.replaceState(null, '', '#vendors-spend');
      }
      if (action === 'route-billing') {
        activateFinancePanelFromShell('billing');
        window.history.replaceState(null, '', '#billing-tax');
      }
      if (action === 'route-filing') {
        document.querySelector('#filing-readiness')?.scrollIntoView({ block: 'center' });
        window.history.replaceState(null, '', '#filing-readiness');
      }
      if (action === 'route-identity') {
        activateIdentityPanelFromShell('auth');
        window.history.replaceState(null, '', '#identity-workforce-service');
      }
      if (action === 'route-daily') {
        activateDailyExecution();
        window.history.replaceState(null, '', '#daily-execution');
      }
      if (action === 'route-finops') {
        activateCockpitPanel('finops');
        window.history.replaceState(null, '', '#finops-pane');
      }
      if (action === 'route-cloud') {
        activateCockpitPanel('topology');
        window.history.replaceState(null, '', '#cloud-ops-cockpit');
      }
      if (action === 'route-policy') routeToLocalTarget('#policy-access', 'Trust proof policy envelope');
      if (action === 'route-inventory') {
        activateResourcePanel('inventory');
        window.history.replaceState(null, '', '#resource-inventory');
      }
      if (action === 'route-audit') {
        activateResourcePanel('audit');
        window.history.replaceState(null, '', '#audit-ledger');
      }
      if (action === 'route-gates') {
        activateResourcePanel('gates');
        window.history.replaceState(null, '', '#deployment-gates');
      }
      if (action === 'route-evidence') {
        activateEvidenceConsole();
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-graph') {
        activateEvidenceConsole('graph');
        document.querySelector('#object-graph')?.scrollIntoView({ block: 'center' });
        window.history.replaceState(null, '', '#object-graph');
      }
      if (action === 'route-workflow') routeToLocalTarget('#workflow-studio', 'Trust proof workflow');
      if (action === 'route-catalog') routeToLocalTarget('#service-catalog', 'Trust proof service catalog');
      if (action === 'route-mail' || action === 'route-messenger' || action === 'route-community') {
        activateSurfaceFromShell(action === 'route-mail' ? 'Mail' : action === 'route-community' ? 'Community' : 'Messenger');
        window.history.replaceState(null, '', '#work-hub');
      }

      document.querySelectorAll('[data-cockpit-status]').forEach((status) => {
        if (board?.closest('#finops-pane')) status.textContent = labels[action] || 'Trust proof action staged locally';
      });
      const resourceStatus = document.querySelector('[data-resource-status]');
      if (resourceStatus && board?.closest('#resource-inventory')) {
        resourceStatus.textContent = labels[action] || 'Resource trust proof action staged locally';
      }
      const evidenceStatus = document.querySelector('[data-evidence-status]');
      if (evidenceStatus && board?.closest('#evidence-ledger, #object-graph')) {
        evidenceStatus.textContent = labels[action] || 'Evidence trust proof action staged locally';
      }
      const financeStatus = document.querySelector('#finance-commercial-service [data-finance-status]');
      if (financeStatus && board?.closest('#ledger-preview')) {
        financeStatus.textContent = labels[action] || 'Ledger trust proof action staged locally';
      }
      const catalogStatus = document.querySelector('[data-catalog-status]');
      if (catalogStatus && board?.closest('#service-catalog')) {
        catalogStatus.textContent = labels[action] || 'Catalog trust proof action staged locally';
      }
      if (localStatus) localStatus.textContent = labels[action] || 'Trust proof action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Trust proof action staged locally',
        body: 'Trust proof changed visual state only; FD-001 tenant workloads and Oyatie Cloud substrate remain unwired with no backend, deploy, billing, mail, or cloud mutation.',
        severity: action?.startsWith('route-') || ['seal-receipt', 'trace-lineage'].includes(action) ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-gate-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.gateAction;
      const card = button.closest('.gate-card');
      const title = card?.querySelector('h5')?.textContent?.trim() || 'Deployment gate';
      const status = document.querySelector('[data-deployment-gate-status]');
      const labels = {
        'attach-evidence': `${title} evidence attached locally`,
        'open-evidence': `${title} evidence graph opened locally`,
        'route-owner': `${title} owner route staged locally`,
      };
      card?.classList.add('selected');
      if (action === 'attach-evidence') {
        button.textContent = 'Evidence attached';
        button.disabled = true;
      }
      if (action === 'open-evidence') activateEvidenceConsole('graph');
      if (action === 'route-owner') activateSurfaceFromShell(title.toLowerCase().includes('argocd') ? 'Messenger' : 'Mail');
      if (status) status.textContent = labels[action] || `${title} gate action staged locally`;
      window.oyaPushActivity?.({
        title: labels[action] || `${title} gate action staged`,
        body: 'Deployment evidence was staged locally for the FD-001/Oyatie Cloud mock release gate.',
        severity: 'review',
      });
    });
  });

  document.querySelectorAll('[data-deployment-gate-action]').forEach((button) => {
    if (button.dataset.deploymentGateBound === 'true') return;
    button.dataset.deploymentGateBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.deploymentGateAction;
      const panel = document.querySelector('#deployment-gates');
      const card = button.closest('[data-deployment-card]');
      const status = document.querySelector('[data-deployment-gate-status]');
      const labels = {
        'admit-fd001': 'FD-001 tenant workload admission simulated locally',
        'route-workflow': 'Deployment runbook opened in Workflow Studio locally',
        'route-cloud': 'Oyatie Cloud cell posture opened locally',
        'route-finops': 'FinOps guardrail opened locally',
        'seal-release': 'Release packet sealed visually · no deploy mutation',
        'route-mail': 'Reviewer mail brief opened locally',
        'ci-lane': 'CI mirror lane selected · Jenkins parity evidence highlighted',
        'attest-lane': 'Attestation lane selected · cosign/SBOM evidence highlighted',
        'admit-lane': 'Tenant admission lane selected · policy and PIPA gate highlighted',
        'observe-lane': 'Observe lane selected · SLO and audit emit highlighted',
        'route-policy': 'Policy envelope opened from deployment gate locally',
        'route-audit': 'Audit packet opened from deployment gate locally',
        'route-community': 'Community release note opened locally',
      };

      if (card && panel) {
        panel.querySelectorAll('[data-deployment-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      button.closest('.deployment-promotion-lane, .deployment-card-actions, .deployment-gate-routes')?.querySelectorAll('[data-deployment-gate-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });

      if (action === 'route-workflow') routeToLocalTarget('#workflow-studio', 'Deployment runbook');
      if (action === 'route-cloud') routeToLocalTarget('#cloud-ops-cockpit', 'Deployment cloud cell');
      if (action === 'route-finops') activateCockpitPanel('finops');
      if (action === 'route-mail') {
        activateSurfaceFromShell('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-policy') routeToLocalTarget('#policy-access', 'Deployment policy envelope');
      if (action === 'route-audit') routeToLocalTarget('#audit-ledger', 'Deployment audit packet');
      if (action === 'route-community') {
        activateSurfaceFromShell('Community');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'seal-release') {
        button.textContent = 'Packet sealed';
        button.classList.add('active');
      }

      if (status) status.textContent = labels[action] || 'Deployment gate action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Deployment gate action staged',
        body: 'Deployment Gates changed local visual state only; no registry, DNS, deploy, billing, or cloud mutation occurred.',
        severity: ['admit-fd001', 'seal-release', 'route-audit', 'admit-lane', 'attest-lane'].includes(action) ? 'review' : 'info',
      });
    });
  });

  function applyCatalogFilter() {
    const search = document.querySelector('[data-catalog-search]');
    const status = document.querySelector('[data-catalog-status]');
    const visibleCount = document.querySelector('[data-catalog-visible-count]');
    const term = search?.value.trim().toLowerCase() || '';
    const activeFilter = document.querySelector('[data-catalog-filter].active')?.dataset.catalogFilter || 'all';
    const rows = Array.from(document.querySelectorAll('[data-catalog-module]'));
    let visible = 0;
    rows.forEach((row) => {
      const group = row.dataset.catalogGroup || '';
      const state = row.dataset.catalogState || '';
      const matchesFilter = activeFilter === 'all' || group === activeFilter || state === activeFilter;
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      row.hidden = !(matchesFilter && matchesText);
      if (!row.hidden) visible += 1;
    });
    if (visibleCount) visibleCount.textContent = String(visible);
    if (status) status.textContent = `${visible} visible · ${activeFilter} filter · local catalog only`;
  }

  const catalogSearch = document.querySelector('[data-catalog-search]');
  if (catalogSearch && catalogSearch.dataset.shellChromeBound !== 'true') {
    catalogSearch.dataset.shellChromeBound = 'true';
    catalogSearch.addEventListener('input', applyCatalogFilter);
  }

  document.querySelectorAll('[data-catalog-filter]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-catalog-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyCatalogFilter();
    });
  });

  document.querySelectorAll('[data-catalog-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const row = button.closest('[data-catalog-module]');
      const title = row?.querySelector('.catalog-module-title')?.textContent?.trim() || 'Catalog module';
      const action = button.dataset.catalogAction;
      const status = document.querySelector('[data-catalog-status]');
      if (action === 'pin') {
        row?.classList.toggle('pinned');
        button.textContent = row?.classList.contains('pinned') ? 'Pinned' : 'Pin';
        if (status) status.textContent = `${title} ${row?.classList.contains('pinned') ? 'pinned' : 'unpinned'} locally`;
      }
      if (action === 'request') {
        button.textContent = 'Access staged';
        button.disabled = true;
        row?.classList.add('review-requested');
        if (status) status.textContent = `${title} access request staged for review`;
      }
      if (action === 'open') {
        const target = button.dataset.catalogTarget || '#service-catalog';
        if (target === '#workflow-studio') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
        else if (target === '#cloud-ops-cockpit') activateCockpitPanel(title.toLowerCase().includes('finops') ? 'finops' : 'topology');
        else if (target === '#audit-ledger') activateResourcePanel('audit');
        else if (target === '#identity-employees') activateIdentityPanelFromShell('employees');
        else if (target === '#work-hub') activateSurfaceFromShell('Messenger');
        else document.querySelector(target)?.scrollIntoView({ block: 'start' });
        if (status) status.textContent = `${title} opened in the local service graph`;
      }
      window.oyaPushActivity?.({
        title: `${title} catalog ${action || 'action'}`,
        body: 'Service catalog state changed locally; no entitlements or backend routes were modified.',
        severity: action === 'request' ? 'review' : 'info',
      });
      applyCatalogFilter();
    });
  });

  document.querySelectorAll('[data-catalog-graph-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.catalogGraphAction;
      if (action === 'workflow') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'community') activateSurfaceFromShell('Community');
      if (action === 'evidence') activateEvidenceConsole();
      const status = document.querySelector('[data-catalog-status]');
      if (status) status.textContent = `Service graph route opened: ${action}`;
    });
  });

  function applyBusinessLogicFilter() {
    const search = document.querySelector('[data-logic-search]');
    const status = document.querySelector('[data-logic-status]');
    const visibleCount = document.querySelector('[data-logic-visible-count]');
    const term = search?.value.trim().toLowerCase() || '';
    const activeFilter = document.querySelector('[data-logic-filter].active')?.dataset.logicFilter || 'all';
    const rows = Array.from(document.querySelectorAll('[data-logic-row]'));
    let visible = 0;
    rows.forEach((row) => {
      const category = row.dataset.logicCategory || '';
      const state = row.dataset.logicState || '';
      const needsAttention = ['attention', 'at-risk', 'blocked', 'review'].includes(state);
      const matchesFilter = activeFilter === 'all' || category === activeFilter || (activeFilter === 'attention' && needsAttention);
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      row.hidden = !(matchesFilter && matchesText);
      if (!row.hidden) visible += 1;
    });
    if (visibleCount) visibleCount.textContent = String(visible);
    if (status) status.textContent = `${visible} visible · ${activeFilter} logic filter · local only`;
  }

  const logicSearch = document.querySelector('[data-logic-search]');
  if (logicSearch && logicSearch.dataset.shellChromeBound !== 'true') {
    logicSearch.dataset.shellChromeBound = 'true';
    logicSearch.addEventListener('input', applyBusinessLogicFilter);
  }

  document.querySelectorAll('[data-logic-filter]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-logic-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyBusinessLogicFilter();
    });
  });

  document.querySelectorAll('[data-logic-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const row = button.closest('[data-logic-row]');
      const title = row?.querySelector('.logic-name-button')?.textContent?.trim() || 'Business logic';
      const action = button.dataset.logicAction;
      const status = document.querySelector('[data-logic-status]');
      if (action === 'open') {
        routeToLocalTarget(button.dataset.logicTarget || '#business-logics', title);
        if (status) status.textContent = `${title} opened from Business Logic OS`;
      }
      if (action === 'run') {
        button.textContent = 'Previewed';
        button.disabled = true;
        row?.classList.add('logic-previewed');
        if (status) status.textContent = `${title} run preview staged locally`;
      }
      window.oyaPushActivity?.({
        title: `${title} logic ${action || 'action'}`,
        body: 'Business Logic OS changed local visual state only; no workflow execution occurred.',
        severity: action === 'run' ? 'review' : 'info',
      });
      applyBusinessLogicFilter();
    });
  });

  document.querySelectorAll('[data-logic-graph-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.logicGraphAction;
      const routes = {
        workflow: '#workflow-studio',
        mail: '#work-hub',
        catalog: '#service-catalog',
        audit: '#audit-ledger',
      };
      if (action === 'mail') activateSurfaceFromShell('Mail');
      else routeToLocalTarget(routes[action] || '#business-logics', `Business logic ${action}`);
      const status = document.querySelector('[data-logic-status]');
      if (status) status.textContent = `Dependency map opened ${action} route`;
    });
  });

  function updateGovernanceStatus(message, severity = 'info') {
    const status = document.querySelector('[data-governance-status]');
    if (status) {
      status.textContent = message;
      status.classList.toggle('danger', severity === 'danger');
      status.classList.toggle('warning', severity === 'warning' || severity === 'review');
      status.classList.toggle('success', severity === 'success');
    }
  }

  function routeGovernanceSurface(route, actionLabel = 'Governance route') {
    if (route === 'workflow') routeToLocalTarget('#workflow-studio', actionLabel);
    else if (route === 'mail') {
      activateSurfaceFromShell('Mail');
      window.history.replaceState(null, '', '#work-hub');
    } else if (route === 'community') {
      activateSurfaceFromShell('Community');
      window.history.replaceState(null, '', '#work-hub');
    } else if (route === 'finance') {
      activateFinancePanelFromShell('ledger');
      window.history.replaceState(null, '', '#finance-commercial-service');
    } else if (route === 'cloud') routeToLocalTarget('#cloud-ops-cockpit', actionLabel);
    else if (route === 'identity') {
      activateIdentityPanelFromShell('roles');
      window.history.replaceState(null, '', '#identity-workforce-service');
    } else if (route === 'evidence') routeToLocalTarget('#evidence-spine', actionLabel);
    else if (route === 'catalog') routeToLocalTarget('#service-catalog', actionLabel);
    else if (route === 'inbox') routeToLocalTarget('#command-center-workbench', actionLabel);
    else routeToLocalTarget('#governance-analytics', actionLabel);
  }

  const governanceActionLabels = {
    'run-review': 'Governance review simulated locally · 5 risks ranked · 3 controls due',
    'seal-brief': 'Board brief staged on the Evidence spine · no packet exported',
    'route-inbox': 'Executive queue opened from governance command',
    'open-inbox': 'Decision queue opened from governance command',
    'select-payroll': 'Payroll close policy gate selected · CFO signoff required',
    'select-hometax': 'HomeTax filing policy gate selected · 사업자등록번호 confirmation required',
    'select-cloud': 'Cloud network split gate selected · rollback evidence required',
    'select-pipa': 'PIPA boundary gate selected · role matrix opened',
    'pillar-compliance': 'Compliance pillar selected · filing evidence route ready',
    'pillar-financial': 'Financial controls pillar selected · ledger route ready',
    'pillar-workforce': 'Workforce risk pillar selected · identity route ready',
    'pillar-disclosure': 'Board + disclosure pillar selected · community route ready',
    'calendar-review': 'Compliance calendar review opened in finance',
    'attest-payroll': 'Payroll 4-eyes attestation selected · workflow route ready',
    'attest-pipa': 'PIPA retention attestation selected · identity route ready',
    'attest-cloud': 'Cloud rollback attestation selected · runbook route ready',
    'route-workflow': 'Workflow Studio route opened from governance',
    'route-mail': 'Mail route opened from governance',
    'route-community': 'Community route opened from governance',
    'route-finance': 'Finance route opened from governance',
    'route-cloud': 'Cloud Ops route opened from governance',
    'route-identity': 'Identity route opened from governance',
    'route-evidence': 'Evidence spine route opened from governance',
    'route-catalog': 'Service catalog route opened from governance',
  };

  document.querySelectorAll('[data-governance-action]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.governanceAction;
      const route = button.dataset.governanceRoute;
      const label = governanceActionLabels[action] || 'Governance command staged locally';
      document.querySelectorAll('[data-governance-action]').forEach((item) => {
        item.classList.toggle('is-selected', item === button);
      });
      updateGovernanceStatus(label, /block|required|risk|review|due/.test(label.toLowerCase()) ? 'review' : 'info');
      if (route) routeGovernanceSurface(route, label);
      window.oyaPushActivity?.({
        title: label,
        body: 'Governance analytics changed local visual state only; no approval, filing, policy, or evidence write occurred.',
        severity: /block|required|risk|review|due/.test(label.toLowerCase()) ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-governance-risk]').forEach((button) => {
    if (button.dataset.riskBound === 'true') return;
    button.dataset.riskBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-governance-risk]').forEach((item) => item.classList.toggle('selected', item === button));
      const id = button.dataset.governanceRisk || 'RISK';
      const title = button.dataset.riskTitle || 'Risk selected';
      const detail = button.dataset.riskDetail || 'Risk details staged locally.';
      const owner = button.dataset.riskOwner || 'Owner';
      const score = button.dataset.riskScore || 'score';
      const idTarget = document.querySelector('[data-risk-peek-id]');
      const titleTarget = document.querySelector('[data-risk-peek-title]');
      const detailTarget = document.querySelector('[data-risk-peek-detail]');
      const ownerTarget = document.querySelector('[data-risk-peek-owner]');
      const scoreTarget = document.querySelector('[data-risk-peek-score]');
      if (idTarget) idTarget.textContent = id;
      if (titleTarget) titleTarget.textContent = title;
      if (detailTarget) detailTarget.textContent = detail;
      if (ownerTarget) ownerTarget.textContent = owner;
      if (scoreTarget) scoreTarget.textContent = score;
      updateGovernanceStatus(`${id} selected · ${score} · ${owner}`, 'review');
      window.oyaPushActivity?.({
        title: `${id} · ${title}`,
        body: detail,
        severity: /High|4×/.test(score) ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-gov-calendar-cell]').forEach((button) => {
    if (button.dataset.calendarBound === 'true') return;
    button.dataset.calendarBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-gov-calendar-cell]').forEach((item) => item.classList.toggle('is-selected', item === button));
      const label = button.dataset.govCalendarCell || button.textContent.trim();
      updateGovernanceStatus(`${label} selected · filing calendar remains visual-only`, /review|pending/.test(label) ? 'review' : 'success');
      window.oyaPushActivity?.({
        title: `Compliance calendar · ${label}`,
        body: 'Calendar selection changed local visual state only; no filing was submitted.',
        severity: /review|pending/.test(label) ? 'review' : 'info',
      });
    });
  });

  function applyEvidenceFilter(filterOverride) {
    const search = document.querySelector('[data-evidence-search]');
    const status = document.querySelector('[data-evidence-status]');
    const filter = filterOverride || document.querySelector('[data-evidence-filter].active')?.dataset.evidenceFilter || 'all';
    const term = search?.value.trim().toLowerCase() || '';
    const rows = Array.from(document.querySelectorAll('[data-evidence-event]'));
    let visible = 0;
    rows.forEach((row) => {
      const state = row.dataset.evidenceState || '';
      const matchesFilter = filter === 'all' || state === filter;
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      row.hidden = !(matchesFilter && matchesText);
      if (!row.hidden) visible += 1;
    });
    if (status) status.textContent = `${visible} visible · ${filter} state · local evidence only`;
  }

  const evidenceSearch = document.querySelector('[data-evidence-search]');
  if (evidenceSearch && evidenceSearch.dataset.shellChromeBound !== 'true') {
    evidenceSearch.dataset.shellChromeBound = 'true';
    evidenceSearch.addEventListener('input', () => applyEvidenceFilter());
  }

  document.querySelectorAll('[data-evidence-filter]').forEach((button) => {
    if (button.dataset.shellChromeBound === 'true') return;
    button.dataset.shellChromeBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-evidence-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyEvidenceFilter(button.dataset.evidenceFilter || 'all');
    });
  });

  document.querySelectorAll('[data-evidence-action]').forEach((button) => {
    if (button.dataset.evidenceBound === 'true') return;
    button.dataset.evidenceBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.evidenceAction;
      const status = document.querySelector('[data-evidence-status]');
      const row = button.closest('[data-evidence-event]');
      const title = row?.querySelector('strong')?.textContent?.trim() || 'Evidence packet';
      const labels = {
        open: `${title} opened in side peek`,
        attach: 'Evidence packet attached to selected Action Inbox context',
        export: 'Reviewer export packet staged locally',
        'run-review': 'Evidence review checklist simulated locally',
      };
      if (row) {
        document.querySelectorAll('[data-evidence-event]').forEach((item) => item.classList.remove('selected'));
        row.classList.add('selected');
      }
      applyEvidenceFilter();
      if (action === 'attach') document.querySelector('#command-center-workbench')?.scrollIntoView({ block: 'start' });
      if (action === 'export') activateResourcePanel('audit');
      if (status) status.textContent = labels[action] || 'Evidence action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Evidence action staged',
        body: 'Evidence Spine changed local visual state only; no ledger write occurred.',
        severity: action === 'open' || action === 'run-review' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-object-node]').forEach((button) => {
    if (button.dataset.objectBound === 'true') return;
    button.dataset.objectBound = 'true';
    button.addEventListener('click', () => {
      const label = button.dataset.objectLabel || button.textContent.trim() || 'Object';
      document.querySelectorAll('[data-object-node]').forEach((node) => node.classList.toggle('active', node === button));
      const status = document.querySelector('[data-object-status]');
      if (status) status.textContent = `${label} selected · side peek and lineage ready`;
    });
  });

  document.querySelectorAll('[data-intel-action]').forEach((button) => {
    if (button.dataset.intelBound === 'true') return;
    button.dataset.intelBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.intelAction;
      const status = document.querySelector('[data-copilot-rail-status]');
      const labels = {
        audit: 'Audit brief drafted from evidence spine locally',
        workflow: 'Workflow critical path simulation opened locally',
        mail: 'Mail approval draft opened locally',
        messenger: 'Messenger ops route opened locally',
        community: 'Community council note opened locally',
      };
      button.classList.add('staged');
      if (action === 'workflow') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
      if (action === 'audit') activateResourcePanel('audit');
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'messenger') activateSurfaceFromShell('Messenger');
      if (action === 'community') activateSurfaceFromShell('Community');
      if (status) status.textContent = labels[action] || 'Copilot rail action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Copilot rail action staged',
        body: 'Governed recommendation opened a local mock route; no backend execution occurred.',
        severity: action === 'audit' || action === 'workflow' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-substrate-action]').forEach((button) => {
    if (button.dataset.substrateBound === 'true') return;
    button.dataset.substrateBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.substrateAction;
      const status = document.querySelector('[data-substrate-status]');
      const labels = {
        cloud: 'Oyatie Cloud cells opened · FD-001 tenant workload substrate proof',
        workflow: 'Workflow proof route opened · FD-001 delivery remains product goal',
        evidence: 'Evidence spine opened · tenant workload proof packet staged',
        messenger: 'Messenger dogfood workload route opened locally',
        mail: 'Mail approval workload route opened locally',
        community: 'Community governance workload route opened locally',
        finops: 'FinOps localization cell watch opened locally',
        deployment: 'Deployment gates opened · Jenkins/ArgoCD/cosign proof path',
        finance: 'Finance close workload opened locally',
        identity: 'Identity policy envelope opened locally',
        catalog: 'Service catalog workload graph opened locally',
      };
      document.querySelectorAll('[data-substrate-action]').forEach((item) => item.classList.toggle('selected', item === button));
      if (action === 'cloud') activateCockpitPanel('topology');
      if (action === 'finops') activateCockpitPanel('finops');
      if (action === 'deployment') activateResourcePanel('gates');
      if (action === 'workflow') routeToLocalTarget('#workflow-studio', 'FD-001 workload proof');
      if (action === 'evidence') activateEvidenceConsole();
      if (action === 'messenger') activateSurfaceFromShell('Messenger');
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'community') activateSurfaceFromShell('Community');
      if (action === 'finance') activateFinancePanelFromShell('ledger');
      if (action === 'identity') activateIdentityPanelFromShell('roles');
      if (action === 'catalog') routeToLocalTarget('#service-catalog', 'Service catalog');
      if (status) status.textContent = labels[action] || 'Substrate proof action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Substrate proof action staged',
        body: 'FD-001 tenant workload dogfood state changed locally; no backend write or external send occurred.',
        severity: action === 'deployment' || action === 'evidence' || action === 'cloud' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-service-action]').forEach((button) => {
    if (button.dataset.platformBound === 'true') return;
    button.dataset.platformBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.platformAction;
      const card = button.closest('.service-card');
      const cardTitle = card?.querySelector('h4')?.textContent?.trim() || 'Service card';
      const labels = {
        'payroll-finance': 'Payroll finance cockpit opened locally',
        'payroll-workflow': 'Payroll workflow gate opened locally',
        'payroll-mail': 'Payroll Mail approval brief opened locally',
        'payroll-evidence': 'Payroll evidence receipt opened locally',
        'filing-billing': 'Billing and tax filing panel opened locally',
        'filing-community': 'Filing council note opened locally',
        'filing-evidence': 'Filing receipt opened locally',
        'employee-identity': 'Identity workforce service opened locally',
        'employee-onboarding': 'Onboarding setup checklist opened locally',
        'employee-policy': 'Policy access envelope opened locally',
        'employee-mail': 'Employee reviewer Mail draft opened locally',
        'governance-command': 'Governance command board opened locally',
        'governance-risk': 'Governance risk heatmap selected locally',
        'governance-community': 'Governance community route opened locally',
        'governance-evidence': 'Governance evidence spine opened locally',
      };
      card?.classList.add('service-card-selected');
      document.querySelectorAll('.service-card').forEach((item) => {
        if (item !== card) item.classList.remove('service-card-selected');
      });
      if (action === 'payroll-finance') activateFinancePanelFromShell('ledger');
      if (action === 'payroll-workflow') routeToLocalTarget('#workflow-studio', cardTitle);
      if (action === 'payroll-mail' || action === 'employee-mail') activateSurfaceFromShell('Mail');
      if (action === 'payroll-evidence' || action === 'filing-evidence' || action === 'governance-evidence') activateEvidenceConsole();
      if (action === 'filing-billing') activateFinancePanelFromShell('billing');
      if (action === 'filing-community' || action === 'governance-community') activateSurfaceFromShell('Community');
      if (action === 'employee-identity') activateIdentityPanelFromShell('employees');
      if (action === 'employee-onboarding') activateIdentityPanelFromShell('onboarding');
      if (action === 'employee-policy') activateCockpitPanel('policy');
      if (action === 'governance-command' || action === 'governance-risk') routeToLocalTarget('#governance-analytics', cardTitle);
      if (action === 'governance-risk') {
        document.querySelector('[data-governance-risk]')?.click();
      }
      window.oyaPushActivity?.({
        title: labels[action] || `${cardTitle} service route staged`,
        body: 'Operations service overview card routed to a richer local command surface; no backend mutation occurred.',
        severity: action?.includes('evidence') || action?.includes('workflow') || action?.includes('governance') ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-ontology-node]').forEach((button) => {
    if (button.dataset.ontologyBound === 'true') return;
    button.dataset.ontologyBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-ontology-node]').forEach((item) => item.classList.toggle('selected', item === button));
      const label = button.dataset.ontologyNode || button.textContent.trim();
      const route = button.dataset.nodeRoute;
      const status = document.querySelector('[data-ontology-status]');
      const detail = document.querySelector('[data-ontology-detail]');
      if (status) status.textContent = `${label} selected · FD-001 workload graph · local only`;
      if (detail) detail.textContent = `${label} lineage selected · no backend graph mutation`;
      if (route === 'cloud') activateCockpitPanel('topology');
      if (route === 'workflow') routeToLocalTarget('#workflow-studio', label);
      if (route === 'mail') activateSurfaceFromShell('Mail');
      if (route === 'evidence') activateEvidenceConsole('graph');
      if (route === 'identity') activateIdentityPanelFromShell('roles');
      window.oyaPushActivity?.({
        title: `${label} ontology node selected`,
        body: 'Ontology graph selection updated local visual state for FD-001 tenant workload dogfooding.',
        severity: route === 'evidence' || route === 'cloud' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-ontology-action]').forEach((button) => {
    if (button.dataset.ontologyActionBound === 'true') return;
    button.dataset.ontologyActionBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.ontologyAction;
      const status = document.querySelector('[data-ontology-status]');
      const labels = {
        lineage: 'FD-001 tenant workload lineage traced locally',
        policy: 'Policy envelope view staged locally',
        evidence: 'Ontology evidence route opened locally',
        'inspect-fact': 'Ontology fact inspected locally',
        'route-workflow': 'Ontology fact routed to Workflow Studio locally',
      };
      if (action === 'evidence') activateEvidenceConsole('graph');
      if (action === 'route-workflow') routeToLocalTarget('#workflow-studio', 'Ontology fact');
      if (action === 'policy') activateIdentityPanelFromShell('roles');
      if (status) status.textContent = labels[action] || 'Ontology action staged locally';
      button.closest('[data-ontology-fact]')?.classList.add('selected');
      window.oyaPushActivity?.({
        title: labels[action] || 'Ontology action staged',
        body: 'Typed graph action changed local visual state only; no graph write occurred.',
        severity: action === 'evidence' || action === 'route-workflow' ? 'review' : 'info',
      });
    });
  });

  document.querySelectorAll('[data-intelligence-action]').forEach((button) => {
    if (button.dataset.intelligenceBound === 'true') return;
    button.dataset.intelligenceBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.intelligenceAction;
      const route = button.dataset.intelligenceRoute;
      const card = button.closest('[data-intelligence-card]');
      const title = card?.querySelector('strong')?.textContent?.trim() || 'Governed AI suggestion';
      const status = document.querySelector('[data-intelligence-status]');
      const labels = {
        evaluate: 'AI evaluation harness ran locally · 14 guardrails checked',
        explain: 'AI rationale panel staged locally',
        preview: `${title} preview opened locally`,
        route: `${title} routed locally`,
        dismiss: `${title} dismissed locally`,
        'route-workflow': 'AI recommendation routed to Workflow Studio locally',
        'route-mail': 'AI recommendation routed to Mail locally',
        'route-community': 'AI recommendation routed to Community locally',
        'route-evidence': 'AI guardrail evidence opened locally',
      };
      card?.classList.add('selected');
      if (action === 'dismiss' && card) card.hidden = true;
      if ((action === 'route' && route === 'workflow') || action === 'route-workflow') routeToLocalTarget('#workflow-studio', title);
      if ((action === 'route' && route === 'mail') || action === 'route-mail') activateSurfaceFromShell('Mail');
      if ((action === 'route' && route === 'community') || action === 'route-community') activateSurfaceFromShell('Community');
      if (action === 'route-evidence') activateEvidenceConsole();
      if (status) status.textContent = labels[action] || 'Governed AI action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Governed AI action staged',
        body: 'AI command console remained human-gated and local-only; no production action executed.',
        severity: action?.includes('route') || action === 'evaluate' ? 'review' : 'info',
      });
    });
  });

  function activateEvidenceConsole(mode = 'all') {
    document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
    if (mode !== 'graph') applyEvidenceFilter();
    const status = document.querySelector('[data-evidence-status]');
    if (status) status.textContent = mode === 'graph'
      ? 'Object graph opened from shell navigation'
      : status.textContent || 'Evidence console opened from shell navigation';
  }

  function applyDailyFilter(filterOverride) {
    const search = document.querySelector('[data-daily-search]');
    const status = document.querySelector('[data-daily-status]');
    const filter = filterOverride || document.querySelector('[data-daily-filter].active')?.dataset.dailyFilter || 'all';
    const term = search?.value.trim().toLowerCase() || '';
    const rows = Array.from(document.querySelectorAll('[data-daily-row]'));
    let visible = 0;
    rows.forEach((row) => {
      const kind = row.dataset.dailyKind || '';
      const state = row.dataset.dailyState || '';
      const matchesFilter = filter === 'all' || kind === filter || state === filter;
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      row.hidden = !(matchesFilter && matchesText);
      if (!row.hidden) visible += 1;
    });
    if (status) status.textContent = `${visible} visible · ${filter} work · local only`;
  }

  const dailySearch = document.querySelector('[data-daily-search]');
  if (dailySearch && dailySearch.dataset.dailyBound !== 'true') {
    dailySearch.dataset.dailyBound = 'true';
    dailySearch.addEventListener('input', () => applyDailyFilter());
  }

  document.querySelectorAll('[data-daily-filter]').forEach((button) => {
    if (button.dataset.dailyBound === 'true') return;
    button.dataset.dailyBound = 'true';
    button.addEventListener('click', () => {
      document.querySelectorAll('[data-daily-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyDailyFilter(button.dataset.dailyFilter || 'all');
    });
  });

  document.querySelectorAll('[data-daily-action]').forEach((button) => {
    if (button.dataset.dailyBound === 'true') return;
    button.dataset.dailyBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.dailyAction;
      const row = button.closest('[data-daily-row]');
      const title = row?.querySelector('strong')?.textContent?.trim() || 'Daily work';
      const status = document.querySelector('[data-daily-status]');
      const labels = {
        workflow: `${title} opened in workflow route`,
        mail: `${title} mail brief staged locally`,
        messenger: `${title} messenger route opened locally`,
        evidence: `${title} evidence route opened locally`,
        stage: `${title} staged in local execution queue`,
      };
      if (row) {
        row.classList.add('daily-staged');
        document.querySelectorAll('[data-daily-row]').forEach((item) => item.classList.toggle('selected', item === row));
      }
      if (action === 'workflow') routeToLocalTarget(button.dataset.dailyTarget || '#workflow-studio', title);
      if (action === 'mail') activateSurfaceFromShell('Mail');
      if (action === 'messenger') activateSurfaceFromShell('Messenger');
      if (action === 'evidence') activateEvidenceConsole();
      if (status) status.textContent = labels[action] || 'Daily work action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Daily work action staged',
        body: 'Personal execution workbench changed local visual state only.',
        severity: action === 'evidence' || action === 'workflow' ? 'review' : 'info',
      });
      applyDailyFilter();
      if (status) status.textContent = labels[action] || 'Daily work action staged locally';
    });
  });

  document.querySelectorAll('[data-daily-proof-action]').forEach((button) => {
    if (button.dataset.dailyProofBound === 'true') return;
    button.dataset.dailyProofBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.dailyProofAction;
      const board = button.closest('.daily-proof-board');
      const localStatus = board?.querySelector('[data-daily-proof-status]');
      const dailyStatus = document.querySelector('[data-daily-status]');
      const inboxStatus = document.querySelector('[data-inbox-status]');
      const labels = {
        'stage-packet': 'FD-001 execution packet staged locally · no workflow, approval, mail, payroll, billing, or cloud mutation',
        'route-daily': 'Daily execution queue opened as FD-001 tenant workload proof',
        'route-inbox': 'Action Inbox opened as FD-001 priority work proof',
        'route-schedule': 'Schedule pressure route opened locally',
        'route-workflow': 'Workflow route opened for governed execution preview',
        'route-cloud': 'Oyatie Cloud substrate route opened',
        'route-policy': 'Policy envelope route opened',
        'route-audit': 'Audit ledger route opened',
        'route-evidence': 'Evidence spine route opened',
        'route-mail': 'Reviewer Mail route opened',
        'route-messenger': 'Messenger route opened',
        'route-community': 'Community route opened',
      };

      board?.querySelectorAll('[data-daily-proof-card]').forEach((card) => {
        card.classList.toggle('selected', card === button.closest('[data-daily-proof-card]'));
      });
      board?.querySelectorAll('[data-daily-proof-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });

      if (action === 'route-daily') routeToLocalTarget('#daily-execution', 'Daily FD-001 work queue');
      if (action === 'route-inbox') routeToLocalTarget('#command-center-workbench', 'Action Inbox');
      if (action === 'route-schedule') {
        routeToLocalTarget('#daily-execution', 'Schedule pressure');
        document.querySelector('#schedule-title')?.scrollIntoView({ block: 'center' });
        window.history.replaceState(null, '', '#schedule-title');
      }
      if (action === 'route-workflow') routeToLocalTarget('#workflow-studio', 'Daily proof workflow');
      if (action === 'route-cloud') routeToLocalTarget('#cloud-ops-cockpit', 'Oyatie Cloud substrate');
      if (action === 'route-policy') routeToLocalTarget('#policy-access', 'Daily proof policy envelope');
      if (action === 'route-audit') routeToLocalTarget('#audit-ledger', 'Daily proof audit ledger');
      if (action === 'route-evidence') {
        activateEvidenceConsole();
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-mail') {
        activateSurfaceFromShell('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-messenger') {
        activateSurfaceFromShell('Messenger');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        activateSurfaceFromShell('Community');
        window.history.replaceState(null, '', '#work-hub');
      }

      if (localStatus) localStatus.textContent = labels[action] || 'Daily proof action staged locally';
      if (dailyStatus) dailyStatus.textContent = labels[action] || 'Daily proof action staged locally';
      if (inboxStatus && board?.classList.contains('inbox-proof-board')) {
        inboxStatus.textContent = labels[action] || 'Action Inbox proof action staged locally';
      }
      window.oyaPushActivity?.({
        title: labels[action] || 'Daily proof action staged locally',
        body: 'Daily execution proof changed local visual state only; no backend write, approval, workflow execution, mail send, payroll, billing, or cloud mutation occurred.',
        severity: action === 'stage-packet' || action === 'route-policy' || action === 'route-audit' ? 'review' : 'info',
      });
    });
  });

  function activateDailyExecution() {
    document.querySelector('#daily-execution')?.scrollIntoView({ block: 'start' });
    const status = document.querySelector('[data-daily-status]');
    if (status) status.textContent = 'Daily execution opened from shell navigation';
    applyDailyFilter();
    setProductActivity('fd001', 'Daily execution opened · FD-001 action queue remains tied to the service graph', { source: 'shell navigation' });
  }

  function setText(selector, value) {
    if (!value) return;
    const target = sidePeek?.querySelector(selector);
    if (target) target.textContent = value;
  }

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[char]));
  }

  function activateCockpitPanel(panel) {
    document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
    document.querySelectorAll('[data-cockpit-tab]').forEach((button) => {
      const active = button.dataset.cockpitTab === panel;
      button.classList.toggle('active', active);
      button.setAttribute('aria-selected', String(active));
    });
    document.querySelectorAll('[data-cockpit-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.cockpitPanel === panel);
    });
    setProductActivity('cloud', `Oyatie Cloud ${panel} panel active · FD-001 substrate proof remains visual-only`, { source: 'cloud cockpit' });
  }

  function activateResourcePanel(panel) {
    document.querySelector('#resource-audit-console')?.scrollIntoView({ block: 'start' });
    document.querySelectorAll('[data-resource-tab]').forEach((button) => {
      const active = button.dataset.resourceTab === panel;
      button.classList.toggle('active', active);
      button.setAttribute('aria-selected', String(active));
    });
    document.querySelectorAll('[data-resource-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.resourcePanel === panel);
    });
    setProductActivity(panel === 'inventory' ? 'cloud' : 'evidence', `${panel} panel active · audit/substrate evidence route updated locally`, { source: 'resource console' });
  }

  function activateBusinessLogics() {
    document.querySelector('#business-logics')?.scrollIntoView({ block: 'start' });
    const status = document.querySelector('[data-logic-status]');
    if (status) status.textContent = 'Business Logic OS opened from shell navigation';
    applyBusinessLogicFilter();
    setProductActivity('fd001', 'Business Logic OS opened · FD-001 operating graph stays selected', { source: 'business logic shell' });
  }

  function routeToLocalTarget(target, title = 'Local route') {
    if (target === '#workflow-studio') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
    else if (target === '#cloud-ops-cockpit') activateCockpitPanel('topology');
    else if (target === '#policy-access') activateCockpitPanel('policy');
    else if (target === '#vendors-spend') activateFinancePanelFromShell('vendors');
    else if (target === '#filing-readiness' || target === '#payroll-cockpit' || target === '#governance-analytics' || target === '#business-logics') {
      document.querySelector(target)?.scrollIntoView({ block: 'start' });
    } else if (target === '#identity-onboarding') activateIdentityPanelFromShell('onboarding');
    else if (target === '#work-hub') activateSurfaceFromShell('Community');
    else if (target === '#service-catalog') activateServiceCatalog();
    else if (target === '#daily-execution' || target === '#tasks-title' || target === '#schedule-title') activateDailyExecution();
    else if (target === '#evidence-spine' || target === '#evidence-ledger' || target === '#object-graph') activateEvidenceConsole(target === '#object-graph' ? 'graph' : 'all');
    else if (target === '#audit-ledger') activateResourcePanel('audit');
    else document.querySelector(target)?.scrollIntoView({ block: 'start' });
    window.history.replaceState(null, '', target);
    const status = document.querySelector('[data-logic-status]');
    if (status) status.textContent = `${title} route opened locally`;
    const routeByTarget = {
      '#workflow-studio': 'workflow',
      '#cloud-ops-cockpit': 'cloud',
      '#policy-access': 'cloud',
      '#work-hub': 'messenger',
      '#audit-ledger': 'evidence',
      '#evidence-spine': 'evidence',
      '#evidence-ledger': 'evidence',
      '#object-graph': 'evidence',
      '#service-catalog': 'fd001',
      '#service-graph': 'fd001',
      '#modules-title': 'fd001',
      '#daily-execution': 'fd001',
      '#tasks-title': 'fd001',
      '#schedule-title': 'fd001',
    };
    setProductActivity(routeByTarget[target] || 'fd001', `${title} route opened locally`, { source: 'local route' });
  }

  function activateServiceCatalog() {
    document.querySelector('#service-catalog')?.scrollIntoView({ block: 'start' });
    const status = document.querySelector('[data-catalog-status]');
    if (status) status.textContent = 'Service catalog opened from shell navigation';
    applyCatalogFilter();
    setProductActivity('fd001', 'Service catalog opened · FD-001 service graph active on Oyatie Cloud', { source: 'service catalog' });
  }

  function activateIdentityPanelFromShell(panel = 'auth') {
    document.querySelector('#identity-workforce-service')?.scrollIntoView({ block: 'start' });
    document.querySelectorAll('[data-identity-tab]').forEach((button) => {
      const active = button.dataset.identityTab === panel;
      button.classList.toggle('active', active);
      if (button.getAttribute('role') === 'tab') button.setAttribute('aria-selected', String(active));
    });
    document.querySelectorAll('[data-identity-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.identityPanel === panel);
    });
    const status = document.querySelector('#identity-workforce-service [data-identity-status]');
    if (status) status.textContent = 'Identity workspace opened from command palette';
    setProductActivity('fd001', 'Identity workspace opened · role envelope remains part of FD-001 service graph', { source: 'identity service' });
  }

  function activateFinancePanelFromShell(panel = 'ledger') {
    document.querySelector('#finance-commercial-service')?.scrollIntoView({ block: 'start' });
    document.querySelectorAll('[data-finance-tab]').forEach((button) => {
      const active = button.dataset.financeTab === panel;
      button.classList.toggle('active', active);
      if (button.getAttribute('role') === 'tab') button.setAttribute('aria-selected', String(active));
    });
    document.querySelectorAll('[data-finance-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.financePanel === panel);
    });
    const status = document.querySelector('#finance-commercial-service [data-finance-status]');
    if (status) status.textContent = 'Finance control opened from command palette';
    setProductActivity('fd001', 'Finance control opened · payroll close remains the FD-001 proof workload', { source: 'finance service' });
  }

  function markCommsSurface(label) {
    document.querySelectorAll('[data-header-comms-surface]').forEach((button) => {
      button.classList.toggle('is-selected', button.dataset.headerCommsSurface === label);
    });
    document.querySelectorAll('[data-rail-comms-surface]').forEach((button) => {
      button.classList.toggle('is-selected', button.dataset.railCommsSurface === label);
    });
  }

  function setCommsRouteStatus(label, source) {
    const status = document.querySelector('[data-comms-route-status]');
    if (status) {
      status.textContent = `${label} route pinned from ${source} · Messenger/Mail/Community are FD-001 tenant workloads · no external send`;
    }
  }

  function activateSurfaceFromShell(label) {
    document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
    document.querySelectorAll('.surface-command').forEach((item) => {
      item.classList.toggle('active', item.textContent.toLowerCase().includes(label.toLowerCase()));
    });
    document.querySelectorAll('.hub-tab').forEach((tab) => {
      if (tab.textContent.trim() === label) tab.click();
    });
    setProductActivity(label, `${label} Work Hub surface active · FD-001 communications workload · local only`, { source: 'Work Hub' });
  }

  function commsBridgeCopy(button) {
    return {
      route: button?.dataset.commsBridgeRoute || 'messenger',
      title: button?.dataset.commsBridgeTitle || 'Messenger ops room',
      receipt: button?.dataset.commsBridgeReceipt || 'REC-COMMS-MSG-021',
      target: button?.dataset.commsBridgeTarget || 'Ops room → Mail brief → Community note',
    };
  }

  function activeCommsBridgeRoute(bridge) {
    return bridge?.querySelector('[data-comms-bridge-route].selected, [data-comms-bridge-route].is-selected')
      || bridge?.querySelector('[data-comms-bridge-route]');
  }

  function setCommsBridgeRoute(button, source = 'Work Hub comms receipt bridge') {
    if (!button) return;
    const bridge = button.closest('[data-comms-receipt-bridge]');
    const copy = commsBridgeCopy(button);
    bridge?.querySelectorAll('[data-comms-bridge-route]').forEach((item) => {
      const selected = item === button;
      item.classList.toggle('selected', selected);
      item.classList.toggle('is-selected', selected);
      item.setAttribute('aria-pressed', String(selected));
    });
    const details = {
      '[data-comms-bridge-detail-title]': copy.title,
      '[data-comms-bridge-detail-receipt]': copy.receipt,
      '[data-comms-bridge-detail-target]': copy.target,
    };
    Object.entries(details).forEach(([selector, value]) => {
      bridge?.querySelectorAll(selector).forEach((node) => { node.textContent = value; });
    });
    const statusText = `${copy.title} selected · ${copy.receipt} · ${copy.target} · local proof only`;
    const bridgeStatus = bridge?.querySelector('[data-comms-bridge-status]');
    if (bridgeStatus) bridgeStatus.textContent = statusText;
    setCommsRouteStatus(copy.route === 'receipt' ? 'Audit receipt' : copy.title, source);
    const substrateStatus = document.querySelector('[data-comms-substrate-status]');
    if (substrateStatus) substrateStatus.textContent = `${copy.title} mapped to FD-001 communications tenant workload · Oyatie Cloud visual proof`;
    if (copy.route === 'messenger') activateSurfaceFromShell('Messenger');
    if (copy.route === 'mail') activateSurfaceFromShell('Mail');
    if (copy.route === 'community') activateSurfaceFromShell('Community');
    if (copy.route === 'receipt') {
      activateResourcePanel('audit');
      const commsReceipt = document.querySelector('[data-receipt-source="comms"]');
      if (commsReceipt) setReceiptSource(commsReceipt, source);
      window.history.replaceState(null, '', '#audit-ledger');
    }
    setProductActivity(copy.route === 'receipt' ? 'evidence' : copy.route, statusText, { source });
    window.oyaPushActivity?.({
      title: `${copy.title} proof route selected`,
      body: `${copy.receipt} keeps Messenger, Mail, Community, and Audit stitched as one FD-001 tenant workload on Oyatie Cloud.`,
      severity: copy.route === 'receipt' ? 'review' : 'info',
    });
  }

  function routeCommsBridgeAction(button) {
    const bridge = button.closest('[data-comms-receipt-bridge]');
    const copy = commsBridgeCopy(activeCommsBridgeRoute(bridge));
    const action = button.dataset.commsBridgeAction || 'audit';
    bridge?.querySelectorAll('[data-comms-bridge-action]').forEach((item) => {
      item.classList.toggle('active', item === button);
      item.classList.toggle('is-selected', item === button);
    });
    const statusText = `${copy.title} → ${action} · ${copy.receipt} · local only`;
    const bridgeStatus = bridge?.querySelector('[data-comms-bridge-status]');
    if (bridgeStatus) bridgeStatus.textContent = statusText;
    const commsStatus = document.querySelector('[data-comms-status]');
    if (commsStatus) commsStatus.textContent = statusText;

    if (action === 'workflow') routeToLocalTarget('#workflow-studio', `${copy.title} communications workflow proof`);
    if (action === 'cloud') routeToLocalTarget('#cloud-ops-cockpit', `${copy.title} Oyatie Cloud workload proof`);
    if (action === 'audit') {
      activateResourcePanel('audit');
      const commsReceipt = document.querySelector('[data-receipt-source="comms"]');
      if (commsReceipt) setReceiptSource(commsReceipt, 'Work Hub comms receipt bridge');
      window.history.replaceState(null, '', '#audit-ledger');
    }
    if (action === 'draft') {
      if (window.oyaCommsHandoff) {
        window.oyaCommsHandoff({
          destination: copy.route === 'community' ? 'Community' : 'Mail',
          source: 'Work Hub receipt bridge',
          title: `Comms packet · ${copy.title}`,
          body: `${copy.receipt} stitches ${copy.target}. Messenger, Mail, and Community remain local-only FD-001 tenant workload drafts.`,
          audience: 'Ops · CFO · SRE · Governance council',
          kind: 'draft',
          meta: `${copy.receipt} · comms receipt bridge`,
        });
      } else {
        activateSurfaceFromShell('Mail');
      }
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      window.history.replaceState(null, '', '#work-hub');
    }
    if (action === 'seal') {
      button.textContent = 'Handoff sealed';
      button.classList.add('active');
    }
    setProductActivity(action === 'cloud' ? 'cloud' : action === 'workflow' ? 'workflow' : action === 'audit' ? 'evidence' : 'mail', statusText, { source: 'Work Hub comms receipt bridge' });
    window.oyaPushActivity?.({
      title: statusText,
      body: 'Work Hub communication bridge changed browser-local visual state only; no mail, community, workflow, audit, deploy, billing, or cloud mutation occurred.',
      severity: action === 'audit' || action === 'seal' ? 'review' : 'info',
    });
  }

  document.querySelectorAll('[data-comms-bridge-route]').forEach((button) => {
    if (button.dataset.commsBridgeBound === 'true') return;
    button.dataset.commsBridgeBound = 'true';
    button.setAttribute('aria-pressed', String(button.classList.contains('selected')));
    button.addEventListener('click', () => setCommsBridgeRoute(button));
  });

  document.querySelectorAll('[data-comms-bridge-action]').forEach((button) => {
    if (button.dataset.commsBridgeActionBound === 'true') return;
    button.dataset.commsBridgeActionBound = 'true';
    button.addEventListener('click', () => routeCommsBridgeAction(button));
  });

  if (window.location.hash === '#command-palette') openPalette();
  if (window.location.hash === '#notifications') openUtilityPanel('notifications');
  if (window.location.hash === '#settings') openUtilityPanel('settings');
  if (window.location.hash === '#side-peek') openSidePeek();
  if (window.location.hash === '#business-logics') activateBusinessLogics();
  if (window.location.hash === '#payroll-cockpit') activateBusinessLogics();
  if (window.location.hash === '#filing-readiness') activateBusinessLogics();
  if (window.location.hash === '#governance-analytics') activateBusinessLogics();
  if (window.location.hash === '#policy-access') activateCockpitPanel('policy');
  if (window.location.hash === '#finops-pane') activateCockpitPanel('finops');
  if (window.location.hash === '#resource-inventory') activateResourcePanel('inventory');
  if (window.location.hash === '#audit-ledger') activateResourcePanel('audit');
  if (window.location.hash === '#deployment-gates') activateResourcePanel('gates');
  if (window.location.hash === '#modules-title') activateServiceCatalog();
  if (window.location.hash === '#service-catalog') activateServiceCatalog();
  if (window.location.hash === '#service-graph') activateServiceCatalog();
  if (window.location.hash === '#daily-execution') activateDailyExecution();
  if (window.location.hash === '#tasks-title') activateDailyExecution();
  if (window.location.hash === '#schedule-title') activateDailyExecution();
  if (window.location.hash === '#evidence-spine') activateEvidenceConsole();
  if (window.location.hash === '#evidence-ledger') activateEvidenceConsole();
  if (window.location.hash === '#object-graph') activateEvidenceConsole('graph');
  if (window.location.hash === '#identity-workforce-service') activateIdentityPanelFromShell('auth');
  if (window.location.hash === '#identity-auth') activateIdentityPanelFromShell('auth');
  if (window.location.hash === '#identity-sessions') activateIdentityPanelFromShell('sessions');
  if (window.location.hash === '#identity-roles') activateIdentityPanelFromShell('roles');
  if (window.location.hash === '#identity-org') activateIdentityPanelFromShell('org');
  if (window.location.hash === '#identity-employees') activateIdentityPanelFromShell('employees');
  if (window.location.hash === '#identity-onboarding') activateIdentityPanelFromShell('onboarding');
  if (window.location.hash === '#finance-commercial-service') activateFinancePanelFromShell('ledger');
  if (window.location.hash === '#ledger-preview') activateFinancePanelFromShell('ledger');
  if (window.location.hash === '#vendors-spend') activateFinancePanelFromShell('vendors');
  if (window.location.hash === '#billing-tax') activateFinancePanelFromShell('billing');
      if (window.location.hash === '#leave-time') activateFinancePanelFromShell('leave');
  updateInboxSelection();
  updateActivityCount();
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
  const commsSearch = hub?.querySelector('[data-comms-search]');
  const commsStatus = hub?.querySelector('[data-comms-status]');
  const substrateStatus = hub?.querySelector('[data-comms-substrate-status]');
  const handoff = {
    title: hub?.querySelector('[data-comms-handoff-title]'),
    status: hub?.querySelector('[data-comms-handoff-status]'),
    source: hub?.querySelector('[data-comms-handoff-source]'),
    destination: hub?.querySelector('[data-comms-handoff-destination]'),
    audience: hub?.querySelector('[data-comms-handoff-audience]'),
  };
  const boardTemplates = {
    Messenger: root.querySelector('template[data-comms-board-template="Messenger"]'),
    Mail: root.querySelector('template[data-comms-board-template="Mail"]'),
    Community: root.querySelector('template[data-comms-board-template="Community"]'),
  };
  let activeHub = 'Messenger';
  let selectedCommsItem = null;
  let handoffSequence = 0;
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
      kind: button.dataset.commsKind || button.querySelector('small em')?.textContent || 'unread',
      meta: button.querySelector('small b')?.textContent || button.querySelector('.hub-meta')?.textContent || 'Visual-only; no backend send',
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
      button.dataset.commsItem = 'true';
      button.dataset.commsKind = item.kind || inferCommsKind(item, index);
      const chipClass = commsChipClass(button.dataset.commsKind);
      button.innerHTML = `<span class="${chipClass}">${escapeHtml(item.source)}</span><strong>${escapeHtml(item.title)}</strong><p>${escapeHtml(item.body)}</p><small><em>${escapeHtml(button.dataset.commsKind)}</em><b>${escapeHtml(item.meta || 'Visual-only; no backend send')}</b></small>`;
      button.addEventListener('click', () => {
        hubList.querySelectorAll('.hub-item').forEach((node) => node.classList.remove('active'));
        button.classList.add('active');
        renderHubDetail(item);
      });
      hubList.appendChild(button);
    });
    applyCommsSearch();
    renderHubDetail((channels[activeHub] ?? [])[0]);
  }

  function renderHubDetail(item) {
    selectedCommsItem = item || null;
    if (!hubDetail) return;
    if (!item) {
      hubDetail.innerHTML = `<p class="eyebrow">${escapeHtml(activeHub)}</p><h4>No visible items</h4><p>Queue a local draft to preview this channel.</p>`;
      return;
    }
    const kind = item.kind || inferCommsKind(item, 0);
    hubDetail.innerHTML = `<div class="comms-detail-head"><div><p class="eyebrow">${escapeHtml(activeHub)}</p><h4>${escapeHtml(item.title)}</h4></div><span class="${commsChipClass(kind)}">${escapeHtml(kind)}</span></div><p>${escapeHtml(item.body)}</p><span class="hub-meta">${escapeHtml(item.meta || 'Visual-only; no backend send')}</span><dl class="comms-detail-grid"><div><dt>Route</dt><dd>${escapeHtml(activeHub)}</dd></div><div><dt>Workflow</dt><dd>Tenant change approval</dd></div><div><dt>Receipt</dt><dd>REC-WF-7741</dd></div><div><dt>Persistence</dt><dd>Local browser state only</dd></div></dl>`;
  }

  function renderCommsBoard(label) {
    const currentBoard = hub?.querySelector('[data-comms-product-board]');
    if (!currentBoard) return;
    const template = boardTemplates[label];
    if (template?.innerHTML) currentBoard.outerHTML = template.innerHTML;
    const freshBoard = hub?.querySelector('[data-comms-product-board]');
    freshBoard?.querySelectorAll('[data-comms-action]').forEach((button) => {
      if (button.dataset.commsBoardBound === 'true') return;
      button.dataset.commsBoardBound = 'true';
      button.addEventListener('click', () => handleCommsAction(button.dataset.commsAction));
    });
  }

  function activateHub(label) {
    activeHub = label;
    setActiveSurface(label.toLowerCase());
    hub?.querySelectorAll('.hub-tab').forEach((tab) => {
      tab.classList.toggle('active', tab.textContent.trim() === label);
      tab.setAttribute('aria-selected', String(tab.textContent.trim() === label));
    });
    hub?.querySelectorAll('.comms-sidebar [data-hub-route]').forEach((button) => {
      button.classList.toggle('active', button.dataset.hubRoute === label);
    });
    if (commsStatus) commsStatus.textContent = `${label} workspace active · FD-001 workload coordination · local only`;
    if (substrateStatus) substrateStatus.textContent = `${label} route pinned to FD-001 workload · cell-us-east-2 · Oyatie Cloud visual proof`;
    renderCommsBoard(label);
    renderHubList();
    updateHandoff({
      source: activeHub,
      destination: label,
      title: `${label} workspace active`,
      status: `${label} receives local-only draft handoffs from the FD-001 communication bus.`,
      audience: label === 'Mail' ? 'CFO · SRE · Governance' : label === 'Community' ? 'Finance · SRE · People Ops' : 'Ops room · Security · Finance',
    });
  }

  hub?.querySelectorAll('.hub-tab').forEach((tab) => {
    tab.addEventListener('click', () => activateHub(tab.textContent.trim()));
  });

  hub?.querySelectorAll('[data-hub-route]').forEach((button) => {
    button.addEventListener('click', () => activateHub(button.dataset.hubRoute));
  });

  commsSearch?.addEventListener('input', applyCommsSearch);

  hub?.querySelectorAll('[data-comms-filter]').forEach((button) => {
    button.addEventListener('click', () => {
      hub.querySelectorAll('[data-comms-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyCommsSearch();
    });
  });

  function handleCommsAction(action) {
    const labels = {
      'new-thread': 'Local thread shell created for FD-001 workload dogfood',
      'attach-evidence': 'Evidence receipt REC-WF-7741 attached locally',
      directory: 'Directory preview opened in context rail',
      'notification-filter': 'Notification filter active',
      'mark-reviewed': 'Selected item marked reviewed locally',
      'create-task': 'Task draft staged from selected communication',
      'link-workflow': 'Linked to Tenant change approval workflow',
      'send-preview': 'Send preview staged locally; no external delivery',
      'publish-note': 'Community publish preview staged locally',
      'thread-escalate': 'Messenger escalation staged for FD-001 workload owner',
      'thread-to-mail': 'Messenger thread promoted to Mail approval draft locally',
      'thread-receipt': 'Cloud workload receipt attached to Messenger thread',
      'mail-preview': 'Mail preview rendered with FD-001 workload evidence',
      'mail-attach': 'FD-001 dogfood packet attached to Mail draft',
      'community-pin': 'Community governance digest pinned locally',
      'community-poll': 'Community poll opened for workload readiness',
      'community-upvote': 'Community vote recorded locally',
      'community-comment': 'Community comment panel staged locally',
      'community-save': 'Community post saved locally',
      'prove-substrate': 'Oyatie Cloud cell posture preview focused for Work Hub tenant workload',
      'route-cloud': 'FD-001 microservice workload route highlighted across Messenger, Mail, and Community',
      'seal-proof': 'REC-WF-7741 substrate proof sealed locally for visual review',
    };
    const current = selectedCommsItem || (channels[activeHub] ?? [])[0] || {
      title: `${activeHub} local draft`,
      body: 'Local visual-only communication context.',
      meta: 'No backend send',
    };
    if (action === 'thread-to-mail') {
      stageCommsHandoff('Mail', {
        source: activeHub,
        title: `Approval brief · ${current.title}`,
        body: `Promoted from ${activeHub}: ${current.body} Evidence stays attached as REC-WF-7741; delivery remains disabled.`,
        audience: 'CFO · SRE reviewer · Governance council',
        kind: 'draft',
        meta: 'Promoted locally from Messenger thread · no external delivery',
      });
      return;
    }
    if (action === 'publish-note' || action === 'community-poll') {
      stageCommsHandoff('Community', {
        source: activeHub,
        title: action === 'community-poll' ? `Readiness poll · ${current.title}` : `Council note · ${current.title}`,
        body: `Prepared from ${activeHub}: ${current.body} Audience is role-gated; publication remains local-only.`,
        audience: 'Finance · SRE · People Ops · Governance',
        kind: 'draft',
        meta: `${action === 'community-poll' ? 'Poll' : 'Publication'} staged locally · no community post sent`,
      });
      return;
    }
    window.oyaPushActivity?.({
      title: labels[action] || 'Communication action staged',
      body: 'Messenger, Mail, and Community state changed locally for Oyatie Cloud dogfood coordination; no external send occurred.',
      severity: action === 'attach-evidence' || action === 'mail-attach' || action === 'thread-receipt' ? 'review' : 'info',
    });
    if (action === 'thread-to-mail') activateHub('Mail');
    if (action === 'community-pin') activateHub('Community');
    if (action === 'new-thread') {
      channels[activeHub] = [
        {
          source: 'Local thread',
          title: `${activeHub} FD-001 workload coordination`,
          body: 'A new visual-only thread was staged for dogfooding a microservice tenant workload on Oyatie Cloud.',
          kind: 'draft',
          meta: 'Local draft · no backend send',
        },
        ...(channels[activeHub] ?? []),
      ];
      renderHubList();
      updateHandoff({
        source: activeHub,
        destination: activeHub,
        title: `${activeHub} thread shell`,
        status: 'New local thread shell created and retained in this browser session.',
        audience: 'Ops · Finance · Governance',
      });
    }
    if (['prove-substrate', 'route-cloud', 'seal-proof'].includes(action)) {
      hub?.querySelectorAll('.comms-substrate-strip [data-comms-action]').forEach((button) => {
        button.classList.toggle('is-selected', button.dataset.commsAction === action);
      });
      if (substrateStatus) substrateStatus.textContent = `${labels[action]} · active surface ${activeHub} · local only`;
    }
    if (commsStatus) commsStatus.textContent = labels[action] || 'Local communication action staged';
  }

  function updateHandoff({ source = activeHub, destination = activeHub, title = 'Local draft handoff ready', status = 'Draft context is retained in browser state only.', audience = 'Role-visible operators' } = {}) {
    if (handoff.title) handoff.title.textContent = title;
    if (handoff.status) handoff.status.textContent = status;
    if (handoff.source) handoff.source.textContent = source;
    if (handoff.destination) handoff.destination.textContent = destination;
    if (handoff.audience) handoff.audience.textContent = audience;
  }

  function stageCommsHandoff(destination, draft) {
    handoffSequence += 1;
    const item = {
      source: `${draft.source} → ${destination}`,
      title: draft.title || `${destination} local draft`,
      body: draft.body || 'Local visual-only handoff draft.',
      kind: draft.kind || 'draft',
      meta: draft.meta || 'Handoff staged locally · no backend send',
    };
    channels[destination] = [item, ...(channels[destination] ?? [])];
    activateHub(destination);
    updateHandoff({
      source: draft.source || activeHub,
      destination,
      title: `${draft.source || activeHub} → ${destination} handoff #${handoffSequence}`,
      status: `${item.title} preserved across Messenger/Mail/Community locally; no external send.`,
      audience: draft.audience || 'Role-visible operators',
    });
    if (commsStatus) commsStatus.textContent = `${destination} draft handoff staged locally from ${draft.source || activeHub}`;
    window.oyaPushActivity?.({
      title: `${draft.source || activeHub} → ${destination} draft handoff staged`,
      body: item.body,
      severity: 'review',
    });
    const detailCard = hub?.querySelector('.comms-handoff-card');
    detailCard?.classList.add('active');
  }

  window.oyaCommsHandoff = ({ destination = 'Mail', source = 'Workflow Studio', title, body, audience, kind = 'draft', meta } = {}) => {
    stageCommsHandoff(destination, {
      source,
      title,
      body,
      audience,
      kind,
      meta,
    });
  };

  window.addEventListener('oya:comms-handoff', (event) => {
    const detail = event.detail || {};
    window.oyaCommsHandoff(detail);
  });

  hub?.querySelectorAll('[data-comms-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.commsAction;
      handleCommsAction(action);
    });
  });

  renderCommsBoard(activeHub);

  queueButton?.addEventListener('click', () => {
    const body = textarea?.value.trim();
    if (!body) return;
    channels[activeHub] = [{ source: 'Local draft', title: 'Local draft queued', body, kind: 'draft', meta: 'Stored in browser state only' }, ...(channels[activeHub] ?? [])];
    textarea.value = '';
    renderHubList();
    updateHandoff({
      source: activeHub,
      destination: activeHub,
      title: `${activeHub} draft queued`,
      status: 'Composer body stored in local Work Hub state only.',
      audience: activeHub === 'Mail' ? 'CFO · SRE' : activeHub === 'Community' ? 'Governance council' : 'Ops room',
    });
    if (commsStatus) commsStatus.textContent = `${activeHub} FD-001 workload draft queued locally`;
  });

  hub?.querySelector('.composer-actions .secondary')?.addEventListener('click', () => {
    if (textarea) textarea.value = '';
    updateHandoff({
      source: activeHub,
      destination: activeHub,
      title: `${activeHub} composer cleared`,
      status: 'Draft input cleared locally; existing handoff records stay in the visual list.',
      audience: 'Current surface',
    });
    if (commsStatus) commsStatus.textContent = `${activeHub} composer cleared locally`;
  });

  function applyCommsSearch() {
    if (!hubList || !commsSearch) return;
    const term = commsSearch.value.trim().toLowerCase();
    const filter = hub?.querySelector('[data-comms-filter].active')?.dataset.commsFilter || 'all';
    let visible = 0;
    hubList.querySelectorAll('.hub-item').forEach((button) => {
      const kind = button.dataset.commsKind || 'unread';
      const matchesFilter = filter === 'all' || kind === filter;
      const matchesText = !term || button.textContent.toLowerCase().includes(term);
      button.hidden = !(matchesFilter && matchesText);
      if (!button.hidden) visible += 1;
    });
    if (commsStatus) {
      commsStatus.textContent = `${activeHub} workspace active · FD-001 workload coordination · ${visible} visible · ${filter} filter · local only`;
    }
  }

  function inferCommsKind(item, index = 0) {
    const haystack = `${item.source || ''} ${item.title || ''} ${item.body || ''} ${item.meta || ''}`.toLowerCase();
    if (haystack.includes('draft') || haystack.includes('brief') || haystack.includes('send')) return 'draft';
    if (haystack.includes('evidence') || haystack.includes('receipt') || haystack.includes('rec-')) return 'evidence';
    if (index < 2 || haystack.includes('unread') || haystack.includes('blocking')) return 'unread';
    return 'review';
  }

  function commsChipClass(kind) {
    if (kind === 'draft') return 'status-chip ai';
    if (kind === 'evidence') return 'status-chip success';
    if (kind === 'unread') return 'status-chip warning';
    return 'status-chip';
  }

  root.querySelectorAll('[data-workbench-filter]').forEach((button) => {
    button.addEventListener('click', () => {
      const filter = button.dataset.workbenchFilter || 'all';
      root.querySelectorAll('[data-workbench-filter]').forEach((item) => item.classList.toggle('active', item === button));
      root.querySelectorAll('[data-workbench-row]').forEach((row) => {
        const key = row.dataset.workbenchRow || 'all';
        row.hidden = filter !== 'all' && key !== filter;
      });
    });
  });

  root.querySelectorAll('[data-copilot-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const statusNode = root.querySelector('[data-copilot-status]');
      const action = button.dataset.copilotAction;
      if (action === 'dismiss') {
        button.closest('.copilot-card')?.setAttribute('hidden', '');
        if (statusNode) statusNode.textContent = 'Suggestion dismissed locally · no workflow changed';
        return;
      }
      if (statusNode) {
        statusNode.textContent = action === 'apply'
          ? 'Delegation suggestion staged as local workflow draft'
          : 'Governed suggestion action staged locally';
      }
    });
  });

  const identityService = root.querySelector('[data-identity-service]');
  const identityStatus = identityService?.querySelector('[data-identity-status]');
  const employeeSearch = identityService?.querySelector('[data-employee-search]');
  let onboardingPercent = 56;

  function activateIdentityPanel(panel = 'auth') {
    if (!identityService) return;
    identityService.querySelectorAll('[data-identity-tab]').forEach((button) => {
      const active = button.dataset.identityTab === panel;
      button.classList.toggle('active', active);
      if (button.getAttribute('role') === 'tab') button.setAttribute('aria-selected', String(active));
    });
    identityService.querySelectorAll('[data-identity-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.identityPanel === panel);
    });
    const labels = {
      auth: 'Passkey and MFA settings active · local only',
      sessions: 'Session/device register active · no revoke sent',
      roles: 'Role envelope matrix active · visual policy only',
      org: 'Organization profile active · no registry write',
      employees: 'Employee directory active · local filter/search',
      onboarding: 'Workspace setup active · visual checklist',
    };
    if (identityStatus) identityStatus.textContent = labels[panel] || 'Identity workspace active';
  }

  function applyEmployeeSearch() {
    if (!identityService || !employeeSearch) return;
    const term = employeeSearch.value.trim().toLowerCase();
    const activeFilter = identityService.querySelector('[data-employee-filter].active')?.dataset.employeeFilter || 'all';
    identityService.querySelectorAll('[data-employee-row]').forEach((row) => {
      const matchesText = !term || row.textContent.toLowerCase().includes(term);
      const matchesTeam = activeFilter === 'all' || row.dataset.employeeTeam === activeFilter;
      row.hidden = !matchesText || !matchesTeam;
    });
  }

  identityService?.querySelectorAll('[data-identity-tab]').forEach((button) => {
    button.addEventListener('click', () => activateIdentityPanel(button.dataset.identityTab || 'auth'));
  });

  employeeSearch?.addEventListener('input', () => {
    applyEmployeeSearch();
    if (identityStatus) identityStatus.textContent = 'Employee search filtered locally';
  });

  identityService?.querySelectorAll('[data-employee-filter]').forEach((button) => {
    button.addEventListener('click', () => {
      identityService.querySelectorAll('[data-employee-filter]').forEach((item) => item.classList.toggle('active', item === button));
      applyEmployeeSearch();
      if (identityStatus) identityStatus.textContent = `${button.textContent.trim()} filter applied locally`;
    });
  });

  identityService?.querySelectorAll('[data-workforce-anchor-action]').forEach((button) => {
    if (button.dataset.workforceAnchorBound === 'true') return;
    button.dataset.workforceAnchorBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.workforceAnchorAction;
      const card = button.closest('[data-workforce-card]');
      const workforceStatus = identityService.querySelector('[data-workforce-anchor-status]');
      const labels = {
        'route-payroll': 'Payroll impact opened locally from workforce directory',
        'route-workflow': 'Workforce lifecycle workflow opened locally',
        'route-policy': 'PIPA policy envelope opened locally',
        'route-audit': 'Workforce audit trail opened locally',
        'stage-invite': 'Employee invite staged locally',
        'route-leave': 'Leave and time liability opened locally',
        'route-mail': 'Reviewer Mail opened from workforce directory',
        'route-community': 'Community workforce update opened locally',
        'route-evidence': 'Workforce evidence graph opened locally',
        'route-cloud': 'Oyatie Cloud cell proof opened from workforce directory',
      };
      if (card) {
        identityService.querySelectorAll('[data-workforce-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      button.closest('.workforce-anchor-actions, .workforce-anchor-routes')?.querySelectorAll('[data-workforce-anchor-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });
      if (action === 'route-payroll') {
        document.querySelector('#payroll-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#payroll-cockpit');
      }
      if (action === 'route-workflow') {
        document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#workflow-studio');
      }
      if (action === 'route-policy') {
        document.querySelector('[data-cockpit-tab="policy"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#policy-access');
      }
      if (action === 'route-audit') {
        document.querySelector('[data-resource-tab="audit"]')?.click();
        document.querySelector('#resource-audit-console')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#audit-ledger');
      }
      if (action === 'stage-invite') identityService.querySelector('[data-identity-action="add-employee"]')?.click();
      if (action === 'route-leave') {
        document.querySelector('[data-finance-tab="leave"]')?.click();
        document.querySelector('#finance-commercial-service')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#leave-time');
      }
      if (action === 'route-mail') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Community');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-evidence') {
        document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-cloud') {
        document.querySelector('[data-cockpit-tab="topology"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#cloud-ops-cockpit');
      }
      if (workforceStatus) workforceStatus.textContent = labels[action] || 'Workforce action staged locally';
      if (identityStatus) identityStatus.textContent = labels[action] || 'Workforce action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Workforce action staged',
        body: 'Employee directory changed local visual state only; no HRIS, auth, payroll, or cloud mutation occurred.',
        severity: ['route-policy', 'route-audit', 'route-evidence', 'stage-invite'].includes(action) ? 'review' : 'info',
      });
    });
  });

  identityService?.querySelectorAll('[data-onboarding-anchor-action]').forEach((button) => {
    if (button.dataset.onboardingAnchorBound === 'true') return;
    button.dataset.onboardingAnchorBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.onboardingAnchorAction;
      const card = button.closest('[data-onboarding-card]');
      const onboardingStatus = identityService.querySelector('[data-onboarding-anchor-status]');
      const labels = {
        'route-tasks': 'Today queue opened from tenant setup locally',
        'import-employees': 'Employee import staged from onboarding locally',
        'route-cloud': 'Oyatie Cloud readiness opened from tenant setup',
        'route-policy': 'Policy gate opened from tenant setup locally',
        'advance-setup': 'Workspace setup advanced locally',
        'route-evidence': 'Tenant setup evidence opened locally',
        'route-payroll': 'Payroll calendar opened from onboarding locally',
        'route-mail': 'Reviewer Mail opened from onboarding locally',
        'route-community': 'Community launch note opened locally',
        'route-schedule': 'Schedule pressure opened from onboarding locally',
      };
      if (card) {
        identityService.querySelectorAll('[data-onboarding-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      button.closest('.onboarding-anchor-actions, .onboarding-anchor-routes')?.querySelectorAll('[data-onboarding-anchor-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });
      if (action === 'route-tasks') {
        document.querySelector('#tasks-title')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#tasks-title');
      }
      if (action === 'import-employees') {
        activateIdentityPanel('employees');
        identityService.querySelector('[data-identity-action="add-employee"]')?.click();
        window.history.replaceState(null, '', '#identity-employees');
      }
      if (action === 'route-cloud') {
        document.querySelector('[data-cockpit-tab="topology"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#cloud-ops-cockpit');
      }
      if (action === 'route-policy') {
        document.querySelector('[data-cockpit-tab="policy"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#policy-access');
      }
      if (action === 'advance-setup') identityService.querySelector('[data-identity-action="advance-onboarding"]')?.click();
      if (action === 'route-evidence') {
        document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-payroll') {
        document.querySelector('#payroll-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#payroll-cockpit');
      }
      if (action === 'route-mail') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Community');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-schedule') {
        document.querySelector('#schedule-title')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#schedule-title');
      }
      if (onboardingStatus) onboardingStatus.textContent = labels[action] || 'Onboarding action staged locally';
      if (identityStatus) identityStatus.textContent = labels[action] || 'Onboarding action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Onboarding action staged',
        body: 'Workspace setup changed local visual state only; no registry, HRIS, payroll, auth, or cloud mutation occurred.',
        severity: ['route-policy', 'route-evidence', 'advance-setup', 'import-employees'].includes(action) ? 'review' : 'info',
      });
    });
  });

  identityService?.querySelectorAll('[data-employee-action]').forEach((button) => {
    if (button.dataset.employeeActionBound === 'true') return;
    button.dataset.employeeActionBound = 'true';
    button.addEventListener('click', () => {
      const row = button.closest('[data-employee-row]');
      const name = row?.querySelector('strong')?.textContent?.trim() || 'Employee';
      const workforceStatus = identityService.querySelector('[data-workforce-anchor-status]');
      identityService.querySelectorAll('[data-employee-row]').forEach((item) => item.classList.toggle('selected', item === row));
      button.classList.add('active');
      const message = `${name} inspected locally · workforce tenant workload remains read-only`;
      if (workforceStatus) workforceStatus.textContent = message;
      if (identityStatus) identityStatus.textContent = message;
      window.oyaPushActivity?.({
        title: `${name} employee profile inspected`,
        body: 'Employee row selection updated visual state only; no directory backend was contacted.',
        severity: 'info',
      });
    });
  });

  identityService?.querySelectorAll('[data-identity-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.identityAction;
      if (action === 'open-audit') {
        identityService.querySelector('[data-identity-tab="sessions"]')?.click();
        if (identityStatus) identityStatus.textContent = 'Audit trail opened locally';
        return;
      }
      if (action === 'add-passkey') {
        const list = identityService.querySelector('[data-passkey-list]');
        if (list && !list.querySelector('[data-local-passkey]')) {
          const row = document.createElement('div');
          row.className = 'auth-method local';
          row.dataset.localPasskey = 'true';
          row.innerHTML = '<span>＋</span><strong>Apple Watch · passkey draft</strong><small>passkey · pending confirmation · local mock</small><em>now</em>';
          list.prepend(row);
        }
        const score = identityService.querySelector('[data-security-score]');
        if (score) score.textContent = '97/100';
        if (identityStatus) identityStatus.textContent = 'Passkey registration preview staged locally';
        window.oyaPushActivity?.({
          title: 'Passkey registration preview staged',
          body: 'Identity security score changed locally; no real auth mutation occurred.',
          severity: 'review',
        });
        return;
      }
      if (action === 'add-employee') {
        identityService.querySelector('[data-identity-tab="employees"]')?.click();
        const tbody = identityService.querySelector('.employee-directory-table tbody');
        if (tbody && !tbody.querySelector('[data-local-employee]')) {
          const row = document.createElement('tr');
          row.dataset.employeeRow = 'true';
          row.dataset.employeeTeam = 'infrastructure';
          row.dataset.localEmployee = 'true';
          row.innerHTML = '<td><strong>로컬 초대</strong><small>Local invite · emp_draft</small></td><td>Pending teammate</td><td>플랫폼팀</td><td>김지영</td><td>draft</td><td><span class="status-chip warning">초대</span></td>';
          tbody.prepend(row);
        }
        applyEmployeeSearch();
        if (identityStatus) identityStatus.textContent = 'Employee invite row staged locally';
        window.oyaPushActivity?.({
          title: 'Employee invite row staged',
          body: 'A local workforce draft was added to the employee directory.',
          severity: 'info',
        });
        return;
      }
      if (action === 'advance-onboarding') {
        onboardingPercent = Math.min(100, onboardingPercent + 11);
        const percent = identityService.querySelector('[data-onboarding-percent]');
        if (percent) percent.textContent = `${onboardingPercent}%`;
        const progress = percent?.parentElement?.querySelector('.score-bar');
        progress?.style.setProperty('--bar', `${onboardingPercent}%`);
        const activeStep = identityService.querySelector('.onboarding-steps li.active');
        if (activeStep) {
          activeStep.classList.remove('active');
          activeStep.classList.add('done');
          activeStep.nextElementSibling?.classList.add('active');
        }
        if (identityStatus) identityStatus.textContent = 'Workspace setup advanced locally';
        window.oyaPushActivity?.({
          title: 'Workspace setup advanced',
          body: 'Onboarding progress changed in local browser state.',
          severity: 'info',
        });
      }
    });
  });

  identityService?.querySelectorAll('[data-identity-route-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const route = button.dataset.identityRouteAction || 'auth';
      const labels = {
        workflow: 'Identity gate opened Workflow Studio locally',
        mail: 'Identity reviewer mail route opened locally',
        sessions: 'Session audit opened locally',
        onboarding: 'Workspace setup checklist opened locally',
        finance: 'Payroll close opened from identity route locally',
        evidence: 'Identity evidence spine opened locally',
        employees: 'Employee lifecycle opened locally',
      };
      identityService.querySelectorAll('[data-identity-route-action]').forEach((item) => item.classList.toggle('active', item === button));
      if (route === 'workflow') document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
      if (route === 'mail') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Mail');
      }
      if (route === 'sessions' || route === 'onboarding' || route === 'employees') {
        activateIdentityPanel(route);
        identityService.scrollIntoView({ block: 'start' });
      }
      if (route === 'finance') document.querySelector('#finance-commercial-service')?.scrollIntoView({ block: 'start' });
      if (route === 'evidence') document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
      if (identityStatus) identityStatus.textContent = labels[route] || 'Identity route staged locally';
      window.oyaPushActivity?.({
        title: labels[route] || 'Identity route staged',
        body: 'Identity/workforce command center changed local visual state only.',
        severity: route === 'evidence' || route === 'workflow' ? 'review' : 'info',
      });
    });
  });

  identityService?.querySelectorAll('[data-identity-anchor-action]').forEach((button) => {
    if (button.dataset.identityAnchorBound === 'true') return;
    button.dataset.identityAnchorBound = 'true';
    button.addEventListener('click', () => {
      const action = button.dataset.identityAnchorAction;
      const card = button.closest('[data-identity-anchor-card]');
      const panel = button.closest('[data-identity-panel]');
      const localStatus = panel?.querySelector('[data-identity-anchor-status]');
      const labels = {
        'route-roles': 'Role envelope opened from session proof locally',
        'route-evidence': 'Identity evidence graph opened locally',
        'route-cloud': 'Oyatie Cloud identity posture opened locally',
        'route-policy': 'Policy board opened from identity subroute locally',
        'route-mail': 'Reviewer Mail opened from identity subroute',
        'route-audit': 'Audit ledger opened from identity subroute',
        'review-roles': 'Role grant review staged locally',
        'route-workflow': 'Workflow gate opened from role envelope locally',
        'route-community': 'Community note opened from identity subroute',
        'route-payroll': 'Payroll close opened from identity subroute locally',
        'route-onboarding': 'Setup packet opened from organization profile locally',
      };
      if (card && panel) {
        panel.querySelectorAll('[data-identity-anchor-card]').forEach((item) => item.classList.toggle('selected', item === card));
      }
      button.closest('.identity-anchor-actions, .identity-anchor-routes')?.querySelectorAll('[data-identity-anchor-action]').forEach((item) => {
        item.classList.toggle('active', item === button);
      });
      if (action === 'route-roles' || action === 'review-roles') {
        activateIdentityPanel('roles');
        window.history.replaceState(null, '', '#identity-roles');
      }
      if (action === 'route-onboarding') {
        activateIdentityPanel('onboarding');
        window.history.replaceState(null, '', '#identity-onboarding');
      }
      if (action === 'route-evidence') {
        document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-cloud') {
        document.querySelector('[data-cockpit-tab="topology"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#cloud-ops-cockpit');
      }
      if (action === 'route-policy') {
        document.querySelector('[data-cockpit-tab="policy"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#policy-access');
      }
      if (action === 'route-mail') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Mail');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-community') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub('Community');
        window.history.replaceState(null, '', '#work-hub');
      }
      if (action === 'route-audit') {
        document.querySelector('[data-resource-tab="audit"]')?.click();
        document.querySelector('#resource-audit-console')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#audit-ledger');
      }
      if (action === 'route-workflow') {
        document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#workflow-studio');
      }
      if (action === 'route-payroll') {
        document.querySelector('#payroll-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#payroll-cockpit');
      }
      if (localStatus) localStatus.textContent = labels[action] || 'Identity subroute action staged locally';
      if (identityStatus) identityStatus.textContent = labels[action] || 'Identity subroute action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Identity subroute action staged',
        body: 'Identity subroute changed local visual state only; no SSO, IAM, HRIS, payroll, or cloud mutation occurred.',
        severity: ['route-policy', 'route-audit', 'route-evidence', 'review-roles'].includes(action) ? 'review' : 'info',
      });
    });
  });

  const financeService = root.querySelector('[data-finance-service]');
  const financeStatus = financeService?.querySelector('[data-finance-status]');
  const financeCommandStatus = financeService?.querySelector('[data-finance-command-status]');
  const vendorSearch = financeService?.querySelector('[data-vendor-search]');

  function activateFinancePanel(panel = 'ledger') {
    if (!financeService) return;
    financeService.querySelectorAll('[data-finance-tab]').forEach((button) => {
      const active = button.dataset.financeTab === panel;
      button.classList.toggle('active', active);
      if (button.getAttribute('role') === 'tab') button.setAttribute('aria-selected', String(active));
    });
    financeService.querySelectorAll('[data-finance-panel]').forEach((item) => {
      item.classList.toggle('active', item.dataset.financePanel === panel);
    });
    const labels = {
      ledger: 'Ledger close cockpit active · local only',
      vendors: 'Vendor spend controls active · no payment rail',
      billing: 'Billing and tax workspace active · no external send',
      leave: 'Leave and time liability active · no payroll mutation',
    };
    if (financeStatus) financeStatus.textContent = labels[panel] || 'Finance workspace active';
    if (financeCommandStatus) financeCommandStatus.textContent = `${labels[panel] || 'Finance route'} · command board linked`;
  }

  function routeFinanceCommand(route) {
    if (!route || !financeService) return;
    const panelRoutes = new Set(['ledger', 'vendors', 'billing', 'leave']);
    if (panelRoutes.has(route)) {
      activateFinancePanel(route);
      financeService.scrollIntoView({ block: 'start' });
      return;
    }
    if (route === 'workflow') {
      document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
      return;
    }
    if (route === 'mail' || route === 'messenger' || route === 'community') {
      document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
      const label = route === 'mail' ? 'Mail' : route === 'community' ? 'Community' : 'Messenger';
      activateHub(label);
      return;
    }
    if (route === 'cloud') {
      document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
      return;
    }
    if (route === 'evidence') {
      document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
    }
  }

  function applyVendorSearch() {
    if (!financeService || !vendorSearch) return;
    const term = vendorSearch.value.trim().toLowerCase();
    financeService.querySelectorAll('[data-vendor-row]').forEach((row) => {
      row.hidden = Boolean(term) && !row.textContent.toLowerCase().includes(term);
    });
  }

  financeService?.querySelectorAll('[data-finance-tab]').forEach((button) => {
    button.addEventListener('click', () => activateFinancePanel(button.dataset.financeTab || 'ledger'));
  });

  vendorSearch?.addEventListener('input', () => {
    applyVendorSearch();
    if (financeStatus) financeStatus.textContent = 'Vendor table filtered locally';
  });

  financeService?.querySelectorAll('[data-finance-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.financeAction;
      const labels = {
        reconcile: 'Ledger reconciliation preview staged locally',
        'export-pack': 'Close evidence pack export staged locally',
        'approve-vendor': 'Vendor approval routed to local review queue',
        'optimize-spend': 'Spend optimization suggestion staged locally',
        'send-invoice': 'Invoice draft staged locally · no customer send',
        'tax-brief': 'Tax filing brief drafted locally',
        'approve-leave': 'Leave approval preview applied locally',
        'reassign-time': 'Backup assignment staged locally',
        'lock-timesheets': 'Timesheet lock preview staged locally',
      };
      if (action === 'add-vendor') {
        activateFinancePanel('vendors');
        const tbody = financeService.querySelector('.vendor-table tbody');
        if (tbody && !tbody.querySelector('[data-local-vendor]')) {
          const row = document.createElement('tr');
          row.dataset.vendorRow = 'true';
          row.dataset.localVendor = 'true';
          row.innerHTML = '<td><strong>Local vendor draft</strong><small>Contract review · visual-only</small></td><td>Procurement</td><td>₩0</td><td>draft</td><td><span class="status-chip warning">Draft</span></td><td><button type="button">Review</button></td>';
          tbody.prepend(row);
        }
        applyVendorSearch();
        if (financeStatus) financeStatus.textContent = 'Vendor draft row staged locally';
        window.oyaPushActivity?.({
          title: 'Vendor draft row staged',
          body: 'Procurement review item added locally; no vendor record was persisted.',
          severity: 'review',
        });
        return;
      }
      if (financeStatus) financeStatus.textContent = labels[action] || 'Finance action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Finance action staged locally',
        body: 'Finance control state changed locally with no external rail or persistence.',
        severity: action === 'approve-vendor' ? 'blocking' : 'info',
      });
    });
  });

  financeService?.querySelectorAll('[data-finance-anchor-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.financeAnchorAction;
      const panel = button.closest('[data-finance-panel]') || financeService;
      const localStatus = panel.querySelector('[data-finance-anchor-status]');
      const labels = {
        'stage-contract': 'Vendor contract packet staged for FD-001 local procurement review',
        'stage-invoice': 'Invoice packet staged locally · no billing send',
        'tax-brief': 'Tax reviewer brief drafted locally · no filing transport',
        'approve-leave': 'Leave approval preview staged locally · no payroll mutation',
        'reassign-time': 'Coverage reassignment preview staged locally',
        'route-ledger': 'Ledger close route active',
        'route-vendors': 'Vendor spend route active',
        'route-billing': 'Billing and tax route active',
        'route-leave': 'Leave and time route active',
        'route-workflow': 'Workflow proof route active',
        'route-cloud': 'Oyatie Cloud substrate proof route active',
        'route-policy': 'Policy envelope route active',
        'route-audit': 'Audit ledger route active',
        'route-evidence': 'Evidence spine route active',
        'route-mail': 'Reviewer Mail route active',
        'route-community': 'Community route active',
        'route-messenger': 'Messenger route active',
      };

      const selectPanelCard = () => {
        panel.querySelectorAll('[data-finance-anchor-card]').forEach((card) => {
          card.classList.toggle('selected', card === button.closest('[data-finance-anchor-card]'));
        });
        panel.querySelectorAll('[data-finance-anchor-action]').forEach((item) => {
          item.classList.toggle('active', item === button);
        });
      };

      const routeToFinancePanel = (financePanel, hash) => {
        activateFinancePanel(financePanel);
        financeService.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', hash);
      };

      selectPanelCard();
      if (action === 'route-ledger') routeToFinancePanel('ledger', '#ledger-preview');
      if (action === 'route-vendors') routeToFinancePanel('vendors', '#vendors-spend');
      if (action === 'route-billing') routeToFinancePanel('billing', '#billing-tax');
      if (action === 'route-leave') routeToFinancePanel('leave', '#leave-time');
      if (action === 'route-workflow') {
        document.querySelector('#workflow-studio')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#workflow-studio');
      }
      if (action === 'route-cloud') {
        document.querySelector('[data-cockpit-tab="finops"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#finops-pane');
      }
      if (action === 'route-policy') {
        document.querySelector('[data-cockpit-tab="policy"]')?.click();
        document.querySelector('#cloud-ops-cockpit')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#policy-access');
      }
      if (action === 'route-audit') {
        document.querySelector('[data-resource-tab="audit"]')?.click();
        document.querySelector('#resource-audit-console')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#audit-ledger');
      }
      if (action === 'route-evidence') {
        document.querySelector('#evidence-spine')?.scrollIntoView({ block: 'start' });
        window.history.replaceState(null, '', '#evidence-spine');
      }
      if (action === 'route-mail' || action === 'route-messenger' || action === 'route-community') {
        document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
        activateHub(action === 'route-mail' ? 'Mail' : action === 'route-community' ? 'Community' : 'Messenger');
        window.history.replaceState(null, '', '#work-hub');
      }

      if (localStatus) localStatus.textContent = labels[action] || 'Finance anchor action staged locally';
      if (financeStatus) financeStatus.textContent = labels[action] || 'Finance anchor action staged locally';
      window.oyaPushActivity?.({
        title: labels[action] || 'Finance anchor action staged locally',
        body: 'FD-001 commercial workload preview changed local visual state only; no bank, payroll, tax, billing, vendor, or cloud mutation occurred.',
        severity: action?.startsWith('route-') || action === 'stage-contract' ? 'review' : 'info',
      });
    });
  });

  financeService?.querySelectorAll('[data-finance-command-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.financeCommandAction;
      const route = button.dataset.financeRoute;
      const labels = {
        'run-close': 'Close dry-run recomputed: 7 objects, 18 receipts, 3 blockers',
        'attach-proof': 'Reviewer proof packet opened in evidence spine',
        'open-payroll': 'Payroll close object selected',
        'open-tax': 'Tax filing object selected',
        'open-vendor': 'Vendor risk queue selected',
        'open-billing': 'Billing and cash pipeline selected',
        'open-leave': 'Leave liability object selected',
        'open-evidence': 'Evidence packet route selected',
        'stage-invoice': 'Northwind invoice draft staged locally',
        'review-plan': 'Plan change queue staged locally',
        'tax-transport': 'HomeTax transport preview staged locally',
        'bank-match': 'Bank match dry-run selected',
        'route-stripe': 'Stripe approval route selected',
        'route-aws': 'AWS FinOps route selected',
        'route-payroll': 'Payroll bureau workflow route selected',
        'mail-brief': 'Finance close mail brief opened locally',
        'messenger-room': 'Finance close messenger room opened locally',
        'council-note': 'Community council note opened locally',
        'route-ledger': 'Ledger close route active',
        'route-vendors': 'Vendor spend route active',
        'route-billing': 'Billing and tax route active',
        'route-leave': 'Leave and time route active',
        'route-workflow': 'Workflow route active',
        'route-mail': 'Mail route active',
        'route-community': 'Community route active',
        'route-evidence': 'Evidence spine route active',
      };
      financeService.querySelectorAll('[data-finance-command-action]').forEach((item) => {
        item.classList.toggle('is-selected', item === button);
      });
      if (financeCommandStatus) financeCommandStatus.textContent = labels[action] || 'Commercial command route staged';
      if (financeStatus) financeStatus.textContent = labels[action] || 'Commercial command route staged';
      routeFinanceCommand(route);
      window.oyaPushActivity?.({
        title: labels[action] || 'Commercial command route staged',
        body: 'Commercial command center changed local visual state only; no payroll, banking, tax, invoice, or vendor rail executed.',
        severity: action?.includes('route') || route === 'evidence' ? 'review' : 'info',
      });
    });
  });

  const workflow = root.querySelector('#workflow-studio');
  const runChip = workflow?.querySelector('.workflow-run-chip');
  const status = workflow?.querySelector('.workflow-statusbar');
  const nodeToolbar = workflow?.querySelector('.node-toolbar');
  const inspector = workflow?.querySelector('.node-inspector');
  const paletteSearch = workflow?.querySelector('[data-workflow-palette-search]');
  const workflowBoard = workflow?.querySelector('[data-workflow-board]');
  const edgeSvg = workflowBoard?.querySelector('.workflow-board-edges');
  const zoomLabel = workflow?.querySelector('.zoom-controls span');
  const workflowProcessStatus = workflow?.querySelector('[data-workflow-process-status]');
  const workflowOutputBus = workflow?.querySelector('[data-workflow-output-bus]');
  const workflowOutputStatus = workflowOutputBus?.querySelector('[data-workflow-output-status]');
  const workflowOutputReceipt = workflowOutputBus?.querySelector('[data-workflow-output-receipt]');
  const workflowOutputCopy = {
    messenger: {
      destination: 'Messenger',
      title: 'Workflow run note · PROC-PAYROLL-CLOSE',
      status: 'Messenger output selected · ops room draft can receive the FD-001 run preview',
      body: 'Workflow Studio prepared a local ops-room run summary for the April payroll close tenant workload on Oyatie Cloud cell-us-east-2.',
      audience: 'Ops room · SRE · Finance',
      kind: 'draft',
    },
    mail: {
      destination: 'Mail',
      title: 'Approval brief · FD-001 workflow run',
      status: 'Mail output selected · approval brief carries workflow, cloud, and evidence context',
      body: 'Workflow Studio prepared a formal approval brief with guardrails, cell posture, and REC-FD001-WF-018 as a local-only attachment.',
      audience: 'CFO · SRE reviewer · Governance council',
      kind: 'draft',
    },
    community: {
      destination: 'Community',
      title: 'Council digest · FD-001 workload readiness',
      status: 'Community output selected · council digest remains role-gated and unsent',
      body: 'Workflow Studio prepared a governed community digest explaining why FD-001 remains the product goal while Oyatie Cloud proves tenant hosting.',
      audience: 'Finance · SRE · People Ops · Governance',
      kind: 'draft',
    },
    evidence: {
      destination: 'Evidence',
      title: 'Evidence receipt · Workflow output bus',
      status: 'Evidence output selected · receipt spine opened with local FD-001/Oyatie Cloud proof',
      body: 'Workflow Studio staged a receipt showing run preview, human route, product surface draft, and Oyatie Cloud substrate context.',
      audience: 'Audit reviewers · Module governance',
      kind: 'evidence',
    },
  };
  let connectSource = null;
  let localBlocks = 0;
  let zoom = 82;
  let workflowOutputSequence = 18;

  function setWorkflowOutput(route = 'messenger', statusOverride = '') {
    const key = workflowOutputCopy[route] ? route : 'messenger';
    const copy = workflowOutputCopy[key];
    workflowOutputBus?.querySelectorAll('[data-workflow-output-route]').forEach((button) => {
      const selected = button.dataset.workflowOutputRoute === key;
      button.classList.toggle('selected', selected);
      button.classList.toggle('is-selected', selected);
      button.setAttribute('aria-pressed', String(selected));
    });
    if (workflowOutputStatus) workflowOutputStatus.textContent = statusOverride || copy.status;
    if (workflowOutputReceipt) {
      workflowOutputReceipt.textContent = `REC-FD001-WF-${String(workflowOutputSequence).padStart(3, '0')} · ${key === 'evidence' ? 'receipt staged' : `${copy.destination} draft`}`;
    }
    window.oyaSetProductActivity?.(
      key === 'evidence' ? 'evidence' : key,
      statusOverride || copy.status,
      { source: 'Workflow output bus' },
    );
  }

  function stageWorkflowOutput(command = 'run', preferredRoute = 'messenger') {
    workflowOutputSequence += 1;
    const route = workflowOutputCopy[preferredRoute] ? preferredRoute : 'messenger';
    const commandLabel = {
      run: 'Run preview',
      validate: 'Validation preview',
      publish: 'Publish packet',
      simulate: 'Simulation',
      'add-block': 'Draft block',
    }[command] || 'Workflow command';
    setWorkflowOutput(route, `${commandLabel} generated output bundle #${workflowOutputSequence} · FD-001 tenant workload on Oyatie Cloud · local only`);
    workflowOutputBus?.classList.add('active');
    window.oyaPushActivity?.({
      title: `${commandLabel} output bundle staged`,
      body: 'Workflow Studio staged Messenger/Mail/Community/Evidence outputs as local drafts for the FD-001 tenant workload on Oyatie Cloud.',
      severity: command === 'publish' || route === 'evidence' ? 'review' : 'info',
    });
  }

  function routeWorkflowOutput(route = 'messenger') {
    const key = workflowOutputCopy[route] ? route : 'messenger';
    const copy = workflowOutputCopy[key];
    setWorkflowOutput(key, `${copy.destination} route opened from Workflow output bus · local visual state only`);
    if (key === 'evidence') {
      window.oyaRouteProductActivity?.('evidence', 'Workflow output bus');
      window.oyaPushActivity?.({
        title: copy.title,
        body: copy.body,
        severity: 'review',
      });
      return;
    }
    stageCommsHandoff(copy.destination, {
      source: 'Workflow Studio',
      title: copy.title,
      body: `${copy.body} Workflow execution, mail delivery, community publishing, and cloud mutation remain disabled.`,
      audience: copy.audience,
      kind: copy.kind,
      meta: `REC-FD001-WF-${String(workflowOutputSequence).padStart(3, '0')} · routed locally from Workflow output bus`,
    });
    document.querySelector('#work-hub')?.scrollIntoView({ block: 'start' });
    window.history.replaceState(null, '', '#work-hub');
    window.oyaSetProductActivity?.(
      key,
      `${copy.destination} draft received Workflow output bundle #${workflowOutputSequence} · no external send`,
      { source: 'Workflow output bus' },
    );
  }

  function setWorkflowMode(mode) {
    workflow?.querySelectorAll('.workflow-modebar button, .workflow-toolbar button').forEach((button) => {
      button.classList.toggle('active', button.textContent.trim() === mode);
    });
    workflow?.querySelectorAll('.workflow-node-group').forEach((node) => {
      node.classList.toggle('connectable', mode === 'Connect');
      node.classList.toggle('simulating', mode === 'Simulate');
    });
    workflowBoard?.classList.toggle('connectable', mode === 'Connect');
    workflowBoard?.classList.toggle('simulating', mode === 'Simulate');
    workflowBoard?.classList.toggle('selectable', mode === 'Select');
    workflowBoard?.querySelectorAll('[data-workflow-card]').forEach((card) => {
      card.classList.toggle('connectable', mode === 'Connect');
      card.classList.toggle('simulating', mode === 'Simulate');
    });
    workflowBoard?.classList.remove('connecting');
    workflowBoard?.querySelectorAll('.connect-pending').forEach((card) => card.classList.remove('connect-pending'));
    connectSource = null;
    if (runChip) runChip.lastChild.textContent = mode === 'Simulate' ? 'simulation preview' : `draft · ${mode.toLowerCase()} mode`;
    if (status?.lastElementChild) status.lastElementChild.textContent = mode === 'Connect' ? 'Click nodes to visualize links' : mode === 'Simulate' ? 'Previewing run path only' : 'Ready · mock';
    if (workflowProcessStatus) workflowProcessStatus.textContent = `${mode} mode · local visual IDE`;
  }

  function workflowMode() {
    const active = workflow?.querySelector('.workflow-modebar button.active')?.textContent.trim();
    return active || 'Select';
  }

  workflow?.querySelectorAll('.workflow-modebar button, .workflow-toolbar button').forEach((button) => {
    button.addEventListener('click', () => setWorkflowMode(button.textContent.trim()));
  });

  workflow?.querySelectorAll('[data-workflow-output-route]').forEach((button) => {
    button.setAttribute('aria-pressed', String(button.classList.contains('selected')));
    button.addEventListener('click', () => routeWorkflowOutput(button.dataset.workflowOutputRoute));
  });

  workflow?.querySelectorAll('[data-workflow-lens], [data-workflow-overlay], [data-workflow-filter]').forEach((button) => {
    button.addEventListener('click', () => {
      const group = button.dataset.workflowLens ? 'workflowLens' : button.dataset.workflowOverlay ? 'workflowOverlay' : 'workflowFilter';
      workflow.querySelectorAll(`[data-${group.replace(/[A-Z]/g, (letter) => `-${letter.toLowerCase()}`)}]`).forEach((item) => {
        item.classList.toggle('active', item === button);
      });
      const label = button.dataset.workflowLens || button.dataset.workflowOverlay || button.dataset.workflowFilter;
      if (workflowProcessStatus) workflowProcessStatus.textContent = `${label} lens staged locally`;
      if (status?.lastElementChild) status.lastElementChild.textContent = `${label} lens staged locally`;
    });
  });

  workflow?.querySelectorAll('[data-workflow-process-action]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.workflowProcessAction;
      const labels = {
        validate: 'Validation preview passed · no execution',
        simulate: 'Simulation overlay active · local only',
        diff: 'Diff v17 → v18 preview staged',
        publish: 'Publish packet staged locally; no workflow deployed',
        'add-condition': 'Approval condition row staged locally',
      };
      if (action === 'simulate') {
        setWorkflowMode('Simulate');
        workflow?.classList.add('run-previewing');
      }
      if (action === 'validate') workflow?.classList.remove('publish-preview');
      if (action === 'publish') workflow?.classList.add('publish-preview');
      if (workflowProcessStatus) workflowProcessStatus.textContent = labels[action] || 'Workflow command staged locally';
      if (status?.lastElementChild) status.lastElementChild.textContent = labels[action] || 'Workflow command staged locally';
      if (action === 'validate') stageWorkflowOutput('validate', 'mail');
      if (action === 'simulate') stageWorkflowOutput('simulate', 'messenger');
      if (action === 'publish') stageWorkflowOutput('publish', 'community');
      if (action === 'add-condition') stageWorkflowOutput('add-block', 'mail');
      window.oyaPushActivity?.({
        title: labels[action] || 'Workflow command staged locally',
        body: 'Workflow Studio changed local visual state only; no workflow execution or publish occurred.',
        severity: action === 'publish' || action === 'simulate' ? 'review' : 'info',
      });
    });
  });

  workflow?.querySelectorAll('[data-workflow-prop]').forEach((input) => {
    input.addEventListener('input', () => {
      if (workflowProcessStatus) workflowProcessStatus.textContent = `${input.dataset.workflowProp} edited locally`;
      if (status?.lastElementChild) status.lastElementChild.textContent = 'Inspector property edited locally';
    });
    input.addEventListener('change', () => {
      if (workflowProcessStatus) workflowProcessStatus.textContent = `${input.dataset.workflowProp} changed locally`;
    });
  });

  paletteSearch?.addEventListener('input', () => {
    const term = paletteSearch.value.trim().toLowerCase();
    workflow?.querySelectorAll('[data-palette-item]').forEach((button) => {
      button.hidden = Boolean(term) && !button.textContent.toLowerCase().includes(term);
    });
  });

  workflow?.querySelectorAll('.workflow-actions button, .workflow-palette button').forEach((button) => {
    button.addEventListener('click', () => {
      const label = button.textContent.trim();
      const primaryLabel = button.querySelector('span')?.textContent.trim() || label;
      const paletteKind = button.dataset.paletteItem;
      if (label === 'Preview run' || label === 'Run') {
        setWorkflowMode('Simulate');
        workflow?.classList.add('run-previewing');
        stageWorkflowOutput('run', 'messenger');
        updateBoardEdges();
      }
      if (label === 'Clear run') {
        workflow?.classList.remove('run-previewing', 'publish-preview');
        setWorkflowMode('Select');
        setWorkflowOutput('messenger', 'Run preview cleared · output bus reset to Messenger draft route');
      }
      if (label === 'Validate') {
        workflow?.classList.remove('publish-preview');
        if (status?.lastElementChild) status.lastElementChild.textContent = 'Validation preview passed';
        stageWorkflowOutput('validate', 'mail');
      }
      if (label === 'Publish') {
        workflow?.classList.add('publish-preview');
        if (status?.lastElementChild) status.lastElementChild.textContent = 'Publish evidence staged locally';
        stageWorkflowOutput('publish', 'community');
      }
      if (label === 'Add block' || (paletteKind && paletteKind !== 'surface')) {
        localBlocks += 1;
        if (status?.children[1]) status.children[1].textContent = `Local blocks: ${localBlocks}`;
        const node = document.createElement('button');
        node.type = 'button';
        node.textContent = `Draft block ${localBlocks}`;
        node.addEventListener('click', () => renderInspector(node.textContent, 'Local', 'Local visual-only block added in the prototype.'));
        nodeToolbar?.appendChild(node);
        const cardLabel = label === 'Add block' ? `Draft block ${localBlocks}` : `${primaryLabel} ${localBlocks}`;
        addWorkflowCard(cardLabel, label === 'Add block' ? 'Local' : primaryLabel, 'Local visual-only block added from the palette.');
        stageWorkflowOutput('add-block', 'mail');
      }
      if (primaryLabel.includes('Messenger')) activateHub('Messenger');
      if (primaryLabel.includes('Mail')) activateHub('Mail');
      if (primaryLabel.includes('Community')) activateHub('Community');
    });
  });

  workflow?.querySelectorAll('[data-workflow-suggestion]').forEach((button) => {
    button.addEventListener('click', () => {
      const action = button.dataset.workflowSuggestion;
      const suggestion = button.closest('.workflow-ai-suggestion');
      if (action === 'dismiss') {
        suggestion?.setAttribute('hidden', '');
        if (status?.lastElementChild) status.lastElementChild.textContent = 'AI suggestion dismissed locally';
      }
      if (action === 'preview') {
        setWorkflowMode('Simulate');
        workflow?.classList.add('run-previewing');
        if (status?.lastElementChild) status.lastElementChild.textContent = 'Delegation suggestion previewing locally';
      }
      if (action === 'apply') {
        addWorkflowCard('Auto delegation guard', 'AI step', 'Local suggestion block: delegate when SLA exceeds threshold.', 468, 258);
        if (status?.lastElementChild) status.lastElementChild.textContent = 'AI suggestion applied as local draft block';
      }
    });
  });

  nodeToolbar?.querySelectorAll('button').forEach((button) => {
    button.addEventListener('click', () => renderInspector(button.textContent.trim(), 'Selected', 'Selected from the visual workflow canvas.'));
  });

  workflow?.querySelectorAll('[data-palette-item]').forEach((button) => {
    button.draggable = true;
    button.addEventListener('dragstart', (event) => {
      event.dataTransfer?.setData('text/plain', button.querySelector('span')?.textContent.trim() || button.textContent.trim());
      event.dataTransfer?.setData('application/x-oya-node-kind', button.dataset.paletteItem ?? 'primitive');
    });
  });

  workflowBoard?.addEventListener('dragover', (event) => {
    event.preventDefault();
    workflowBoard.classList.add('drag-over');
  });

  workflowBoard?.addEventListener('dragleave', (event) => {
    if (event.target === workflowBoard) workflowBoard.classList.remove('drag-over');
  });

  workflowBoard?.addEventListener('drop', (event) => {
    event.preventDefault();
    workflowBoard.classList.remove('drag-over');
    const label = event.dataTransfer?.getData('text/plain') || 'Dropped block';
    const rect = workflowBoard.getBoundingClientRect();
    addWorkflowCard(label, 'Dropped', 'Dropped from the local palette; no backend workflow mutation.', event.clientX - rect.left - 78, event.clientY - rect.top - 36);
  });

  workflow?.querySelectorAll('.zoom-controls button').forEach((button) => {
    button.addEventListener('click', () => {
      zoom = Math.max(62, Math.min(125, zoom + (button.textContent.trim() === '+' ? 8 : -8)));
      if (zoomLabel) zoomLabel.textContent = `${zoom}%`;
      if (workflowBoard) workflowBoard.style.setProperty('--workflow-board-zoom', String(zoom / 100));
      updateBoardEdges();
    });
  });

  workflowBoard?.querySelectorAll('[data-workflow-card]').forEach(bindWorkflowCard);
  updateBoardEdges();

  function bindWorkflowCard(card) {
    if (card.dataset.workflowCardBound === 'true') return;
    card.dataset.workflowCardBound = 'true';
    let dragState = null;

    card.addEventListener('pointerdown', (event) => {
      if (event.button !== 0) return;
      selectWorkflowCard(card);
      dragState = {
        pointerId: event.pointerId,
        startX: event.clientX,
        startY: event.clientY,
        left: parseFloat(card.style.left || '0'),
        top: parseFloat(card.style.top || '0'),
        moved: false,
      };
      card.setPointerCapture?.(event.pointerId);
    });

    card.addEventListener('pointermove', (event) => {
      if (!dragState || dragState.pointerId !== event.pointerId || !workflowBoard) return;
      const dx = event.clientX - dragState.startX;
      const dy = event.clientY - dragState.startY;
      if (Math.abs(dx) + Math.abs(dy) > 2) dragState.moved = true;
      const maxLeft = Math.max(0, workflowBoard.clientWidth - card.offsetWidth - 12);
      const maxTop = Math.max(0, workflowBoard.clientHeight - card.offsetHeight - 12);
      card.style.left = `${Math.max(8, Math.min(maxLeft, dragState.left + dx))}px`;
      card.style.top = `${Math.max(8, Math.min(maxTop, dragState.top + dy))}px`;
      card.classList.add('dragging');
      updateBoardEdges();
      if (status?.lastElementChild) status.lastElementChild.textContent = 'Node position changed locally';
    });

    card.addEventListener('pointerup', (event) => {
      if (!dragState || dragState.pointerId !== event.pointerId) return;
      card.releasePointerCapture?.(event.pointerId);
      const moved = dragState.moved;
      dragState = null;
      card.classList.remove('dragging');
      if (moved) {
        card.dataset.justDragged = 'true';
        setTimeout(() => delete card.dataset.justDragged, 0);
      }
    });

    card.addEventListener('click', () => {
      if (card.dataset.justDragged === 'true') return;
      if (workflowMode() === 'Connect') {
        connectWorkflowCard(card);
      } else {
        selectWorkflowCard(card);
      }
    });
  }

  function selectWorkflowCard(card) {
    workflowBoard?.querySelectorAll('[data-workflow-card]').forEach((node) => node.classList.remove('active'));
    card.classList.add('active');
    renderInspector(
      card.dataset.nodeLabel || card.textContent.trim(),
      card.dataset.nodeKind || 'Node',
      card.dataset.nodeDesc || 'Selected from the visual workflow canvas.'
    );
  }

  function connectWorkflowCard(card) {
    if (!workflowBoard) return;
    if (!connectSource) {
      connectSource = card;
      card.classList.add('connect-pending');
      workflowBoard.classList.add('connecting');
      if (status?.lastElementChild) status.lastElementChild.textContent = 'Select a target node';
      return;
    }
    if (connectSource !== card) {
      addWorkflowEdge(connectSource.dataset.nodeId, card.dataset.nodeId);
      if (status?.lastElementChild) status.lastElementChild.textContent = 'Local connection preview added';
    }
    connectSource.classList.remove('connect-pending');
    workflowBoard.classList.remove('connecting');
    connectSource = null;
  }

  function addWorkflowCard(label, kind = 'Local', body = 'Local visual-only workflow block.', x, y) {
    if (!workflowBoard) return null;
    const existingCards = Array.from(workflowBoard.querySelectorAll('[data-workflow-card]'));
    const id = `draft-${Date.now()}-${existingCards.length + 1}`;
    const card = document.createElement('button');
    card.type = 'button';
    card.className = 'workflow-card selectable active';
    card.dataset.workflowCard = 'true';
    card.dataset.nodeId = id;
    card.dataset.nodeLabel = label;
    card.dataset.nodeKind = kind;
    card.dataset.nodeDesc = body;
    card.style.left = `${Number.isFinite(x) ? Math.max(8, x) : 72 + existingCards.length * 42}px`;
    card.style.top = `${Number.isFinite(y) ? Math.max(8, y) : 188}px`;
    card.innerHTML = `<span class="board-port in" aria-hidden="true"></span><span class="board-port out" aria-hidden="true"></span><span class="workflow-card-type">${escapeHtml(kind)}</span><strong>${escapeHtml(label)}</strong><small>${escapeHtml(body)}</small>`;
    workflowBoard.querySelectorAll('[data-workflow-card]').forEach((node) => node.classList.remove('active'));
    workflowBoard.appendChild(card);
    bindWorkflowCard(card);
    const previous = existingCards.at(-1);
    if (previous) addWorkflowEdge(previous.dataset.nodeId, id);
    selectWorkflowCard(card);
    if (status?.lastElementChild) status.lastElementChild.textContent = 'Local node added to canvas';
    return card;
  }

  function addWorkflowEdge(fromId, toId) {
    if (!edgeSvg || !fromId || !toId) return;
    const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    path.setAttribute('class', 'workflow-edge workflow-board-edge local-edge');
    path.dataset.edgeFrom = fromId;
    path.dataset.edgeTo = toId;
    path.setAttribute('marker-end', 'url(#workflow-board-arrow)');
    edgeSvg.appendChild(path);
    updateBoardEdges();
  }

  function updateBoardEdges() {
    if (!workflowBoard || !edgeSvg) return;
    edgeSvg.querySelectorAll('[data-edge-from][data-edge-to]').forEach((edge) => {
      const from = workflowBoard.querySelector(`[data-node-id="${cssEscape(edge.dataset.edgeFrom)}"]`);
      const to = workflowBoard.querySelector(`[data-node-id="${cssEscape(edge.dataset.edgeTo)}"]`);
      if (!from || !to) return;
      edge.setAttribute('d', pathBetweenCards(from, to));
    });
  }

  function pathBetweenCards(from, to) {
    const start = cardPoint(from, 'out');
    const end = cardPoint(to, 'in');
    const controlDelta = Math.max(64, Math.abs(end.x - start.x) / 2);
    return `M ${start.x} ${start.y} C ${start.x + controlDelta} ${start.y}, ${end.x - controlDelta} ${end.y}, ${end.x} ${end.y}`;
  }

  function cardPoint(card, side) {
    const left = parseFloat(card.style.left || '0');
    const top = parseFloat(card.style.top || '0');
    const width = card.offsetWidth || 156;
    const height = card.offsetHeight || 72;
    return {
      x: side === 'out' ? left + width : left,
      y: top + height / 2,
    };
  }

  function cssEscape(value) {
    if (window.CSS?.escape) return window.CSS.escape(value || '');
    return String(value || '').replace(/["\\]/g, '\\$&');
  }

  function renderInspector(title, kind, body) {
    if (!inspector) return;
    inspector.innerHTML = `<p class="eyebrow">Selected node</p><h4>${escapeHtml(title)}</h4><p><strong>${escapeHtml(kind)}</strong> · ${escapeHtml(body)}</p>`;
  }

  function escapeHtml(value) {
    return String(value).replace(/[&<>'"]/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[char]));
  }
}
