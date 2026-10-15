pub(super) const SCRIPT: &str = r#"  function shellContext(phase = sessionPhase, destination = null) {
    sessionPhase = phase;
    const verified = current && current.expiresAt * 1000 > Date.now();
    let target = destination;
    if (!target) try { target = find(decodeURIComponent(location.hash.slice(1))); } catch {}
    const foundry = target && (target.id === 'foundry-workspace' || find('foundry-workspace').contains(target));
    const accountRoute = target && signInPanel.contains(target);
    const shell = find('oyatie-shell'), frame = foundry ? 'foundry' : accountRoute ? 'account' : 'samples';
    if (shell && find('console-shell')?.parentElement === shell) {
      const changed = shell.getAttribute('data-shell-frame') !== frame;
      shell.setAttribute('data-shell-frame', frame);
      for (const [id, surface] of [['shell-sample-intro','samples'],['dashboard-island-root','samples'],['oyatie-sign-in','account'],['foundry-workspace','foundry']]) {
        const node = find(id);
        if (node?.parentElement !== find('console-shell')) continue;
        if (frame !== 'samples' && surface !== frame) node.setAttribute('data-shell-inactive','');
        else node.removeAttribute('data-shell-inactive');
      }
      if (changed && find('shell-navigation-menu')) find('shell-navigation-menu').open = frame !== 'foundry';
      find('shell-skip-link')?.setAttribute('href', foundry ? '#foundry-workspace' : accountRoute ? '#oyatie-sign-in' : '#console-shell');
    }
    const previews = {'#service-catalog':'Catalog samples','#cloud-ops-cockpit':'Cloud samples','#evidence-spine':'Evidence samples','#work-hub':'Work hub samples'};
    const write = (id, text) => { const element = find(id); if (element) element.textContent = text; };
    write('shell-rail-product', foundry ? 'Foundry' : accountRoute ? 'Account & session' : 'Control Center');
    write('shell-breadcrumb-scope', foundry ? 'Foundry' : accountRoute ? 'Oyatie IAM' : 'Sample views');
    write('shell-breadcrumb-destination', foundry ? 'Ontology' : accountRoute ? 'Account & session' : previews[location.hash] || 'Control Center');
    write('shell-session-entry', phase === 'checking' ? 'Checking session' : verified ? 'Account & session' : 'Sign in');
    write('shell-header-identity', phase === 'checking' ? 'Checking session' : verified ? `${current.tenantId} · ${current.userId}` : foundry || accountRoute ? 'No verified account' : 'Anonymous sample preview');
    write('shell-account-mark', verified ? '◎' : 'S');
    write('shell-account-title', verified ? current.tenantId : phase === 'checking' ? 'Checking session' : foundry || accountRoute ? 'No verified account' : 'Sample workspace');
    write('shell-account-detail', verified ? current.userId : phase === 'checking' ? 'Awaiting verification' : foundry || accountRoute ? 'Sign-in required' : 'Anonymous preview');
    write('shell-rail-scope', foundry ? 'FOUNDRY / ONTOLOGY' : accountRoute ? 'OYATIE IAM' : verified ? 'SAMPLE DESTINATIONS' : 'ANONYMOUS SAMPLE VIEWS');
    write('shell-rail-title', foundry ? 'Ontology workspace' : accountRoute ? 'Account & session' : 'Explore shell samples');
    write('shell-rail-description', foundry ? 'Model and object access are checked separately. Sample destinations remain previews.' : accountRoute ? 'Use this account panel to check your session, change IDs or sign out.' : 'Catalog, cloud, evidence and Work Hub samples. Live operations are unavailable.');
    write('shell-rail-status', phase === 'checking' ? 'Checking session · access not yet verified' : verified ? 'Session verified · access checked per surface' : foundry || accountRoute ? 'Sign-in required · no live data' : 'Navigation only · no live operation');
    write('shell-route-status', foundry ? 'Access checked per surface' : verified ? 'Sample destinations · preview only' : 'Navigation only · no live operation');
    find('shell-rail-context')?.setAttribute('aria-label', foundry ? 'Foundry workspace context' : accountRoute ? 'Account and session' : 'Sample destinations');
    const main = find('console-shell');
    main?.setAttribute('aria-labelledby', foundry ? 'foundry-workspace-title' : accountRoute ? 'sign-in-title' : 'console-title');
    main?.setAttribute('aria-describedby', foundry ? (find('live-ontology-explorer').hidden ? 'live-ontology-access-notice' : 'live-ontology-status') : accountRoute ? 'sign-in-status' : 'console-notice');
    explorer.measure();
  }
"#;
