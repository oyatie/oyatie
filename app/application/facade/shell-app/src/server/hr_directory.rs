pub(super) const SCRIPT: &str = r#"
(() => {
  let current = null;
  const normalize = (value) => value.normalize('NFC').trim().toLocaleLowerCase();
  const rows = (root) => [...root.querySelectorAll('[data-employee-row]')];
  const profiles = (root) => [...root.querySelectorAll('[data-employee-inspection]')];
  function source(root) {
    const banner = document.querySelector('.envelope-banner');
    const tenant = banner?.querySelector('h2');
    return JSON.stringify([root.dataset.employeeSource, tenant?.textContent,
      tenant?.nextElementSibling?.textContent, rows(root).map((row) =>
        [row.dataset.employeeId, row.dataset.employeeTeam, row.dataset.employeeSearchText,
         [...row.cells].slice(0, 6).map((cell) => cell.textContent)])]);
  }
  function remember() {
    if (!current) return;
    current.query = current.root.querySelector('[data-employee-search]').value;
    current.open = profiles(current.root).filter((detail) => detail.open)
      .map((detail) => detail.dataset.employeeInspection);
    current.panelActive = current.root.closest('[role="tabpanel"]')?.classList.contains('active');
  }
  function focusKey(target) {
    if (target.matches('[data-employee-scroll]')) return ['scroll'];
    if (target.matches('[data-employee-search]')) return ['search'];
    if (target.matches('[data-employee-filter]')) return ['filter', target.dataset.employeeFilter];
    if (target.matches('[data-employee-clear]')) return ['clear'];
    const detail = target.closest('[data-employee-inspection]');
    if (detail) return [target.matches('[data-employee-close]') ? 'close' : 'inspect',
      detail.dataset.employeeInspection];
    return null;
  }
  function focusTarget(root, key) {
    if (!key) return null;
    if (key[0] === 'scroll') return root.querySelector('[data-employee-scroll]');
    if (key[0] === 'search') return root.querySelector('[data-employee-search]');
    if (key[0] === 'clear') return root.querySelector('[data-employee-clear]');
    if (key[0] === 'filter') return [...root.querySelectorAll('[data-employee-filter]')]
      .find((button) => button.dataset.employeeFilter === key[1]);
    const detail = profiles(root).find((item) => item.dataset.employeeInspection === key[1]);
    if (!detail || detail.closest('[data-employee-row]').hidden) return null;
    return detail.querySelector(key[0] === 'close' && detail.open
      ? '[data-employee-close]' : 'summary');
  }
  function restoreScroll() {
    const scroller = current?.root.querySelector('[data-employee-scroll]');
    if (scroller?.isConnected && scroller.clientWidth)
      scroller.scrollLeft = current.scrollLeft;
  }
  function apply() {
    const root = current.root;
    const query = normalize(current.query);
    let visible = 0;
    const all = rows(root);
    all.forEach((row) => {
      row.hidden = !((current.filter === 'all' || row.dataset.employeeTeam === current.filter)
        && normalize(row.dataset.employeeSearchText).includes(query));
      if (row.hidden) row.querySelector('details').open = false;
      else visible += 1;
    });
    root.querySelectorAll('[data-employee-filter]').forEach((button) => {
      const selected = button.dataset.employeeFilter === current.filter;
      button.setAttribute('aria-pressed', String(selected));
      button.classList.toggle('active', selected);
    });
    root.querySelector('[data-employee-count]').textContent =
      `${visible} matching sample employees · ${visible} of ${all.length} shown`;
    root.querySelector('[data-employee-empty]').hidden = visible !== 0;
    remember();
  }
  function synchronize() {
    const root = document.querySelector('[data-employee-directory="sample"]');
    if (!root) return;
    const key = source(root);
    if (current?.root === root && current.key === key) return;
    remember();
    const previous = current;
    const same = previous?.key === key;
    const savedFocus = same ? previous.focus : null;
    const activeBefore = document.activeElement;
    current = {root, key, query: same ? previous.query : '',
      filter: same ? previous.filter : 'all', open: same ? previous.open : [],
      scrollLeft: same ? previous.scrollLeft : 0,
      focus: same ? previous.focus : null, reset: same ? previous.reset : !!previous};
    root.querySelector('[data-employee-search]').value = current.query;
    profiles(root).forEach((detail) => {
      detail.open = current.open.includes(detail.dataset.employeeInspection);
    });
    root.querySelector('[data-employee-reset]').hidden = !current.reset;
    apply();
    if (same && previous.root !== root && previous.panelActive) {
      const panel = root.closest('[role="tabpanel"]');
      const tab = document.getElementById(panel.getAttribute('aria-labelledby'));
      if (tab?.getAttribute('aria-controls') === panel.id && !panel.classList.contains('active')) {
        if (tab.closest('[role="tablist"]').hasAttribute('data-tablist-wired')) tab.click();
        else revealDirectory(panel, false);
        if (activeBefore?.isConnected && activeBefore !== document.body) activeBefore.focus({preventScroll: true});
      }
    }
    if (same && previous.root !== root && (!activeBefore || activeBefore === document.body ||
        !activeBefore.isConnected || activeBefore === previous.handoffPanelFocus)) {
      const target = focusTarget(root, savedFocus);
      if (target?.isConnected && target.getClientRects().length && current.key === source(root))
        target.focus({preventScroll: true});
    }
    restoreScroll();
  }
  function directoryTarget(event) {
    if (!(event.target instanceof Element) || !event.target.isConnected) return null;
    const root = event.target.closest('[data-employee-directory="sample"]');
    if (!root || root !== document.querySelector('[data-employee-directory="sample"]')) return null;
    synchronize();
    return root === current?.root ? event.target : null;
  }
  document.addEventListener('input', (event) => {
    const target = directoryTarget(event);
    if (!target?.matches('[data-employee-search]')) return;
    current.query = target.value;
    apply();
  });
  document.addEventListener('click', (event) => {
    const target = directoryTarget(event);
    if (!target) return;
    const filter = target.closest('[data-employee-filter]');
    if (filter) {
      current.filter = filter.dataset.employeeFilter;
      apply();
    } else if (target.closest('[data-employee-clear]')) {
      current.query = '';
      current.filter = 'all';
      current.root.querySelector('[data-employee-search]').value = '';
      apply();
    } else if (target.closest('[data-employee-close]')) {
      const detail = target.closest('details');
      detail.open = false;
      detail.querySelector('summary').focus();
      remember();
    }
  });
  document.addEventListener('toggle', (event) => {
    const target = directoryTarget(event);
    if (target?.matches('[data-employee-inspection]')) remember();
  }, true);
  document.addEventListener('scroll', (event) => {
    const target = directoryTarget(event);
    if (target?.matches('[data-employee-scroll]') && target.clientWidth)
      current.scrollLeft = target.scrollLeft;
  }, true);
  document.addEventListener('focusin', (event) => {
    const target = directoryTarget(event);
    const replacement = document.querySelector('[data-employee-directory="sample"]');
    if (current && !current.root.isConnected && replacement?.isConnected &&
        event.target === replacement.closest('[role="tabpanel"]') &&
        source(replacement) === current.key) {
      current.handoffPanelFocus = event.target;
      return;
    }
    if (current) current.focus = target ? focusKey(target) : null;
  });
  document.addEventListener('keydown', (event) => {
    if (!['Enter', ' ', 'Spacebar'].includes(event.key) ||
        !(event.target instanceof Element)) return;
    const tab = event.target;
    const panel = document.getElementById(tab.getAttribute('aria-controls'));
    if (tab.isConnected && tab.matches('[role="tab"][data-identity-tab="employees"]') &&
        tab.getAttribute('aria-selected') === 'true' &&
        panel?.getAttribute('aria-labelledby') === tab.id &&
        panel.querySelector('[data-employee-directory="sample"]'))
      requestAnimationFrame(restoreScroll);
  });

  function revealDirectory(target, moveFocus) {
    const panel = target?.closest('[role="tabpanel"]');
    if (!panel?.querySelector('[data-employee-directory="sample"]')) return;
    const tab = document.getElementById(panel.getAttribute('aria-labelledby'));
    const tablist = tab?.closest('[role="tablist"]');
    if (!tablist || tablist.hasAttribute('data-tablist-wired') ||
        tab.getAttribute('aria-controls') !== panel.id) return;
    [...tablist.querySelectorAll('[role="tab"]')].forEach((candidate) => {
      const selected = candidate === tab;
      candidate.setAttribute('aria-selected', String(selected));
      candidate.tabIndex = selected ? 0 : -1;
      candidate.classList.toggle('active', selected);
      document.getElementById(candidate.getAttribute('aria-controls'))?.classList.toggle('active', selected);
    });
    if (moveFocus) {
      if (!target.hasAttribute('tabindex')) target.tabIndex = -1;
      target.focus({preventScroll: true});
      target.scrollIntoView();
    }
  }
  function restoreDirectoryFragment() {
    let target;
    try { target = document.getElementById(decodeURIComponent(window.location.hash.slice(1))); }
    catch { return; }
    revealDirectory(target, true);
    if (target?.closest('[role="tabpanel"]')?.querySelector('[data-employee-directory="sample"]'))
      requestAnimationFrame(restoreScroll);
  }
  document.addEventListener('click', (event) => {
    if (!(event.target instanceof Element) || !event.target.isConnected) return;
    const tab = event.target.closest('[data-identity-tab="employees"]');
    if (tab) {
      const service = tab.closest('[data-identity-service]');
      const panel = service?.querySelector('[data-identity-panel="employees"]');
      const tablist = service?.querySelector('[role="tablist"]');
      const controlled = document.getElementById(panel?.getAttribute('aria-labelledby'));
      if (!tablist || controlled?.parentElement !== tablist ||
          controlled.getAttribute('role') !== 'tab' ||
          controlled.getAttribute('aria-controls') !== panel.id) return;
      if (tab.getAttribute('role') === 'tab' && tab !== controlled) return;
      if (tab.getAttribute('role') !== 'tab') {
        controlled.click();
        if (controlled.isConnected && !controlled.disabled &&
            controlled.getAttribute('aria-selected') === 'true' &&
            controlled.getAttribute('aria-controls') === panel.id && panel.isConnected &&
            panel.getAttribute('aria-labelledby') === controlled.id &&
            panel.classList.contains('active') && document.hasFocus() &&
            document.activeElement !== controlled)
          controlled.focus({preventScroll: true});
      }
      requestAnimationFrame(restoreScroll);
    }
  });
  window.addEventListener('hashchange', restoreDirectoryFragment);
  window.addEventListener('popstate', restoreDirectoryFragment);
  const host = document.getElementById('dashboard-island-root');
  if (host) new MutationObserver(synchronize).observe(host, {childList: true, subtree: true});
  synchronize();
  restoreDirectoryFragment();
})();
"#;
