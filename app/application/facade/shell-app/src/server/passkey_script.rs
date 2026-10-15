mod credentials;
mod explorer;

pub(super) fn script() -> String {
    format!(
        "{}\n{}\n{}",
        credentials::SCRIPT,
        explorer::script(),
        SCRIPT
    )
}

const SCRIPT: &str = r#"
const signInPanel = document.getElementById('oyatie-sign-in');
if (signInPanel) {
  const find = id => document.getElementById(id);
  const tenant = find('sign-in-tenant');
  const account = find('sign-in-account');
  const status = find('sign-in-status');
  const card = find('verified-ontology-card');
  const invitation = find('passkey-invitation');
  let enabled = false;
  let busy = 0;
  let generation = 0;
  let current = null;
  let sessionPhase = 'anonymous';
  let cookieTenant = null;
  let ceremony = null;
  let tail = Promise.resolve();
  let expiryTimer = null;
  const explorer = createOntologyExplorer({
    valid: (session, version) => stillSelected(session, version) && current?.tenantId === session.tenantId && current?.userId === session.userId && current.expiresAt * 1000 > Date.now(),
    fallback: () => account.focus(),
    expired: text => { generation += 1; clearSession(); message(text, true); },
  });
  const canonical = (value, prefix) => new RegExp(`^${prefix}[A-Za-z0-9_-]+$`).test(value) && value.length <= 256;
  function selected() {
    if (!canonical(tenant.value, 'ten_') || !canonical(account.value, 'usr_')) {
      throw new Error('Enter the tenant ID and account ID provided by your administrator.');
    }
    return { tenantId: tenant.value, userId: account.value };
  }
  function message(text, error = false) {
    status.textContent = text;
    status.dataset.error = String(error);
  }
  function clearCard() {
    card.hidden = true;
    for (const fact of card.querySelectorAll('[data-live-fact]')) fact.textContent = '';
    find('verified-ontology-identity').textContent = '';
    find('verified-ontology-updated').textContent = '';
  }
  function clearLive() { explorer.clear(); clearCard(); }
  function shellContext(phase = sessionPhase) {
    sessionPhase = phase;
    const verified = current && current.expiresAt * 1000 > Date.now();
    let target = null;
    try { target = find(decodeURIComponent(location.hash.slice(1))); } catch {}
    const foundry = target && (target.id === 'foundry-workspace' || find('foundry-workspace').contains(target));
    const accountRoute = target && signInPanel.contains(target);
    const previews = {'#service-catalog':'Catalog samples','#cloud-ops-cockpit':'Cloud samples','#evidence-spine':'Evidence samples','#work-hub':'Work hub samples'};
    const write = (id, text) => { const element = find(id); if (element) element.textContent = text; };
    write('shell-rail-product', foundry ? 'Foundry' : accountRoute ? 'Account & session' : 'Control Center');
    write('shell-breadcrumb-scope', foundry ? 'Foundry' : accountRoute ? 'Oyatie IAM' : 'Sample views');
    write('shell-breadcrumb-destination', foundry ? 'Ontology' : accountRoute ? 'Account & session' : previews[location.hash] || 'Control Center');
    write('shell-session-entry', phase === 'checking' ? 'Checking session' : verified ? 'Account & session' : 'Sign in');
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
  }
  function clearSession() {
    current = null;
    clearTimeout(expiryTimer);
    clearLive();
    shellContext('anonymous');
  }
  function controls() {
    signInPanel.setAttribute('aria-busy', String(busy > 0));
    for (const id of ['passkey-sign-in', 'session-restore', 'passkey-enroll', 'passkey-invitation']) {
      find(id).disabled = !enabled || busy > 0;
    }
    tenant.disabled = busy > 0;
    account.disabled = busy > 0;
    find('ontology-refresh').disabled = busy > 0;
    find('passkey-cancel').hidden = busy === 0;
    find('session-logout').disabled = !enabled;
  }
  async function api(chosen, path, body) {
    const response = await fetch(`/api/identity/${encodeURIComponent(chosen)}/${path}`, {
      method: body === undefined ? 'GET' : 'POST',
      credentials: 'same-origin', cache: 'no-store', redirect: 'error',
      headers: body === undefined ? {} : { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
    });
    if (!response.ok) {
      const error = new Error(response.status === 401 ? 'Session or passkey unavailable. Check your IDs and retry.'
        : response.status === 403 ? 'Access refused. Ask your administrator to check your membership.'
        : 'Sign-in service unavailable. Retry when the service is available.');
      error.status = response.status;
      throw error;
    }
    return response.status === 204 || (path === 'passkeys/enroll' && response.status === 201)
      ? null : response.json();
  }
  function stillSelected(snapshot, version) {
    return version === generation && tenant.value === snapshot.tenantId && account.value === snapshot.userId;
  }
  function enqueue(action) {
    busy += 1;
    controls();
    // One queue per page plus one origin-wide Web Lock. Never abort a fetch that may set a cookie.
    const run = () => navigator.locks.request('oyatie-identity-session', action);
    const task = tail.then(run, run);
    tail = task.catch(() => {});
    task.catch(error => {
      clearSession();
      message(error.name === 'AbortError' || error.name === 'NotAllowedError'
        ? 'Passkey cancelled or unavailable. Retry, or ask for a new invitation if enrollment was submitted.'
        : error.message || 'Sign-in unavailable. Retry.', true);
    }).finally(() => { busy -= 1; ceremony = null; controls(); });
    return task;
  }
  async function verify(snapshot, version, retaining = false) {
    if (!retaining) clearLive();
    const session = await api(snapshot.tenantId, 'session');
    if (!stillSelected(snapshot, version)) return;
    if (session.tenantId !== snapshot.tenantId || session.userId !== snapshot.userId
      || !Number.isSafeInteger(session.expiresAt) || session.expiresAt * 1000 <= Date.now()
      || !Array.isArray(session.roles)) throw new Error('Session does not match this account. Sign in again.');
    cookieTenant = session.tenantId;
    current = session;
    clearTimeout(expiryTimer);
    expiryTimer = setTimeout(() => {
      generation += 1;
      clearSession();
      message('Session expired. Sign in again.', true);
    }, Math.min(session.expiresAt * 1000 - Date.now(), 2147483647));
    const statusCheck = (async () => {
      const response = await fetch(`/api/ontology-status/${encodeURIComponent(snapshot.tenantId)}`, {
        credentials: 'same-origin', cache: 'no-store', redirect: 'error',
      });
      if (!stillSelected(snapshot, version)) return;
      if (!response.ok) {
        clearCard();
        if (response.status === 401) { generation += 1; clearSession(); message('Session revoked or expired. Sign in again.',true); }
        return;
      }
      const facts = await response.json();
      if (!stillSelected(snapshot, version) || session.expiresAt * 1000 <= Date.now()) return;
      if (typeof facts.policyVersion !== 'string' || ['servedTenants','projectionLag','poisonedEntries'].some(key=>!Number.isSafeInteger(facts[key]) || facts[key]<0)) {clearCard();return;}
      for (const fact of card.querySelectorAll('[data-live-fact]')) fact.textContent = String(facts[fact.dataset.liveFact]);
      find('verified-ontology-identity').textContent = `${session.tenantId} · ${session.userId} · authorized by Foundry`;
      find('verified-ontology-updated').textContent = `Checked ${new Date().toLocaleTimeString()}. Refresh to check current access.`;
      card.hidden = false;
    })().catch(()=>{if(stillSelected(snapshot,version))clearCard();});
    void explorer.open(session,version,retaining);
    shellContext('verified');
    await Promise.allSettled([statusCheck]);
    if (stillSelected(snapshot,version) && current) message(`Signed in as ${session.userId}. Each Ontology surface checks its own access. The dashboard remains a sample preview.`);

  }
  find('passkey-form').addEventListener('submit', event => {
    event.preventDefault();
    if (!enabled || busy) return;
    let snapshot;
    try { snapshot = selected(); } catch (error) { message(error.message, true); return; }
    const version = ++generation;
    clearSession();
    enqueue(async () => {
      if (cookieTenant) { await api(cookieTenant, 'session:logout', {}); cookieTenant = null; }
      if (!stillSelected(snapshot, version)) return;
      message('Use your device or security key to sign in.');
      const options = await api(snapshot.tenantId, 'passkeys/authentication-options', { userId: snapshot.userId });
      if (!stillSelected(snapshot, version)) return;
      ceremony = new AbortController();
      const credential = await navigator.credentials.get({ publicKey: browserPublicKey(options, false), signal: ceremony.signal });
      if (!stillSelected(snapshot, version)) return;
      await api(snapshot.tenantId, 'passkeys/authenticate', { challenge: options.challenge, credential: serializeCredential(credential, false) });
      cookieTenant = snapshot.tenantId;
      if (stillSelected(snapshot, version)) await verify(snapshot, version);
    });
  });
  find('passkey-enroll').addEventListener('click', () => {
    if (!enabled || busy) return;
    let snapshot;
    try { snapshot = selected(); } catch (error) { message(error.message, true); return; }
    const version = ++generation;
    clearSession();
    enqueue(async () => {
      const file = invitation.files[0];
      if (!file || file.size > 65536) throw new Error('Choose a valid invitation file from your administrator.');
      let options;
      try { options = JSON.parse(await file.text()); } catch { throw new Error('Invitation file is invalid. Ask your administrator for a new file.'); }
      if (options.tenantId !== snapshot.tenantId || options.userId !== snapshot.userId
        || options.publicKey?.user?.name !== snapshot.userId || options.publicKey?.rp?.id !== location.hostname) {
        throw new Error('Invitation does not match this tenant, account, or site. Ask your administrator for a new file.');
      }
      if (!stillSelected(snapshot, version)) return;
      message('Use your device or security key to add a passkey.');
      ceremony = new AbortController();
      const credential = await navigator.credentials.create({ publicKey: browserPublicKey(options, true), signal: ceremony.signal });
      if (!stillSelected(snapshot, version)) return;
      await api(snapshot.tenantId, 'passkeys/enroll', { challenge: options.challenge, credential: serializeCredential(credential, true) });
      if (stillSelected(snapshot, version)) message('Passkey added. Choose Use passkey to sign in.');
    }).finally(() => { invitation.value = ''; }).catch(() => {});
  });
  function restore() {
    if (!enabled || busy) return;
    let snapshot;
    try { snapshot = selected(); } catch (error) { clearSession(); message(error.message, true); return; }
    const version = ++generation;
    const retaining = current?.tenantId === snapshot.tenantId && current?.userId === snapshot.userId && current.expiresAt * 1000 > Date.now();
    if (retaining) explorer.hold(version); else clearSession();
    shellContext('checking');
    enqueue(() => verify(snapshot, version, retaining));
  }
  find('session-restore').addEventListener('click', restore);
  find('ontology-refresh').addEventListener('click', restore);
  find('passkey-cancel').addEventListener('click', () => {
    generation += 1;
    clearSession();
    ceremony?.abort();
    message('Cancelling. A submitted sign-in will finish before sign-out.');
    logout();
  });
  function logout() {
    if (!enabled) return;
    let chosen = cookieTenant || (canonical(tenant.value, 'ten_') ? tenant.value : null);
    generation += 1;
    clearSession();
    ceremony?.abort();
    enqueue(async () => {
      chosen = cookieTenant || chosen;
      if (!chosen) throw new Error('Enter your tenant ID to sign out.');
      await api(chosen, 'session:logout', {});
      cookieTenant = null;
      message('Signed out. Anonymous sample preview remains available.');
    });
  }
  find('session-logout').addEventListener('click', logout);
  for (const input of [tenant, account]) input.addEventListener('input', () => {
    generation += 1;
    ceremony?.abort();
    clearSession();
    message('Account selection changed. Use passkey or Check session to verify access.');
  });
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' && current && !busy) restore();
  });
  window.addEventListener('hashchange', () => shellContext());
  shellContext();
  setInterval(() => { if (current && !busy && document.visibilityState === 'visible') restore(); }, 30000);
  if (!window.isSecureContext || !navigator.credentials || !navigator.locks || !window.PublicKeyCredential) {
    message('Passkey sign-in needs a secure site and a browser with passkey support. Anonymous preview remains available.', true);
  } else {
    fetch('/api/identity-config', { cache: 'no-store', credentials: 'same-origin', redirect: 'error' })
      .then(response => { if (!response.ok) throw new Error(); return response.json(); })
      .then(config => {
        enabled = config.enabled === true;
        controls();
        message(enabled ? 'Enter your tenant and account to sign in, or Check session to restore verified access.'
          : 'Sign-in is not configured on this host. Anonymous preview remains available.');
      }).catch(() => message('Sign-in service unavailable. Reload to retry. Anonymous preview remains available.', true));
  }
}
"#;

#[cfg(test)]
mod explorer_test;

#[cfg(test)]
mod draft_editor_test;

#[cfg(test)]
mod pending_selection_test;

#[cfg(test)]
mod validation_navigation_test;

#[cfg(test)]
mod generic_model_editor_test;

#[cfg(test)]
mod workbench_context_test;

#[cfg(test)]
mod selected_inspector_test;
