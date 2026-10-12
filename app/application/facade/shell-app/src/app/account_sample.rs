use super::*;

pub(super) const ACCOUNT_PANELS: [&str; 5] = ["auth", "sessions", "roles", "org", "onboarding"];
pub(super) const SAMPLE_SETUP_STEPS: [(&str, &str); 9] = [
    ("welcome", "Welcome"),
    ("legal-profile", "Legal profile example"),
    ("bank-payment", "Bank and payment example"),
    ("payroll-calendar", "Payroll calendar example"),
    ("insurance", "Insurance example"),
    ("jurisdiction", "Jurisdiction example"),
    ("people-import", "People import example"),
    ("retention-policy", "Retention and policy example"),
    ("review", "Review sample setup"),
];

pub(super) fn account_person_html() -> &'static str {
    r#"<div class="identity-person" aria-label="Sample person and workspace"><span aria-hidden="true">최</span><div><strong>Sample: 최유나 · Choi Yu-na</strong><small>Sample Admin · Oyatie Corp. · live role unavailable</small></div></div><p class="identity-chain">Fixed account examples; no current session, device or audit source is supplied.</p>"#
}

pub(super) fn account_context_html() -> &'static str {
    r#"<p class="screen-anchor">SAMPLE PROVENANCE</p><dl class="service-kv"><div><dt>Sample actor</dt><dd>Choi Yu-na · Admin example</dd></div><div><dt>Sample scope</dt><dd>Auth · organization · setup</dd></div><div><dt>Live authority</dt><dd>Unknown / unavailable</dd></div></dl><p>These examples do not supply grants, credentials, privacy results, bank verification or tenant admission. Real sign-in and live Ontology use the separate Oyatie IAM surface.</p>"#
}

pub(super) fn account_panel_html(panel: &str) -> String {
    let facts = match panel {
        "auth" => r#"<div class="identity-panel-copy"><p class="screen-anchor">SAMPLE ACCOUNT · CREDENTIALS</p><h4 id="auth-panel-title">Sample passkeys and recovery methods</h4><p>Fixed credential examples. Your enrolled credentials and security posture are Unknown / unavailable here.</p></div><div class="auth-grid"><div class="auth-method-list" data-passkey-list="true" aria-label="Sample credentials"><div class="auth-method"><span aria-hidden="true">⌘</span><strong>Sample MacBook Pro · Touch ID</strong><small>Example passkey; registration unavailable</small></div><div class="auth-method"><span aria-hidden="true">◉</span><strong>Sample iPhone · Face ID</strong><small>Example passkey; registration unavailable</small></div><div class="auth-method"><span aria-hidden="true">▣</span><strong>Sample authenticator</strong><small>Example TOTP; activation unavailable</small></div><div class="auth-method"><span aria-hidden="true">⌁</span><strong>Sample recovery codes</strong><small>Example only; no codes are supplied</small></div></div><aside class="security-score-card" aria-label="Live security status unavailable"><p class="screen-anchor">LIVE SECURITY</p><strong data-security-score="true">Unknown</strong><p>No security score or verification is supplied.</p><button type="button" data-identity-action="add-passkey" data-account-focus="add-passkey">Open real passkey invitation</button></aside></div>"#.to_owned() + &identity_command_html(),
        "sessions" => r#"<div class="identity-panel-copy"><p class="screen-anchor">SAMPLE ACCOUNT · DEVICES</p><h4 id="sessions-panel-title">Sample sessions and devices</h4><p>Fixed device examples; these are not your active sessions. Live session identity and device verification are Unknown / unavailable.</p></div><div class="identity-session-grid" aria-label="Sample devices"><article class="session-card"><span class="device-glyph" aria-hidden="true">⌘</span><div><strong>Sample MacBook Pro</strong><small>Chrome · Seoul example</small></div></article><article class="session-card"><span class="device-glyph" aria-hidden="true">◉</span><div><strong>Sample iPhone</strong><small>Safari · Seoul example</small></div></article><article class="session-card"><span class="device-glyph" aria-hidden="true">▣</span><div><strong>Sample Windows device</strong><small>Finance office example</small></div></article></div><ol class="identity-audit-log" aria-label="Static sample audit examples"><li><time>09:14 example</time><strong>Sample credential inspection</strong><span>No challenge or verification occurred</span></li><li><time>08:42 example</time><strong>Sample payroll role inspection</strong><span>No grant changed</span></li></ol>"#.to_owned() + &identity_anchor_html("sessions"),
        "roles" => r#"<div class="identity-panel-copy"><p class="screen-anchor">SAMPLE ACCOUNT · ACCESS</p><h4 id="roles-panel-title">Sample roles and grants</h4><p>Illustrative rows only. Live membership, grants, policy and privacy decisions are Unknown / unavailable.</p></div><table class="role-matrix-table" aria-label="Sample role matrix"><thead><tr><th>Sample role</th><th>Example scope</th><th>Example decision</th><th>Example reason</th></tr></thead><tbody><tr><td>Sample Tenant Admin</td><td>Workspace · users</td><td>Sample allow</td><td>Illustrative administrator</td></tr><tr><td>Sample Payroll Approver</td><td>Payroll close · filing</td><td>Sample review</td><td>Illustrative two person review</td></tr><tr><td>Sample Workflow Builder</td><td>Draft · simulate</td><td>Sample allow</td><td>Illustrative draft scope</td></tr><tr><td>Sample External Vendor</td><td>Employee information</td><td>Sample deny</td><td>Illustrative restricted scope</td></tr></tbody></table>"#.to_owned() + &identity_anchor_html("roles"),
        "org" => r#"<div class="identity-panel-copy"><p class="screen-anchor">SAMPLE ORGANIZATION</p><h4 id="org-panel-title">Sample organization profile</h4><p>Fixed organization, legal, payment and payroll examples. Live legal and bank verification are Unknown / unavailable; no automatic audit recording is supplied.</p></div><div class="org-stat-grid" aria-label="Sample organization counts"><span><small>Sample employees</small><strong>118</strong></span><span><small>Sample monthly payroll</small><strong>₩894,000,000</strong></span><span><small>Sample workflows</small><strong>42</strong></span><span><small>Sample open items</small><strong>7</strong></span></div><div class="org-profile-grid" aria-label="Sample legal and payment facts"><dl><dt>Sample organization</dt><dd>오야티 주식회사</dd><dt>Sample business number</dt><dd>123-45-67890</dd><dt>Sample representative</dt><dd>이재현</dd><dt>Sample address</dt><dd>서울 강남구 테헤란로 521</dd></dl><dl><dt>Sample bank</dt><dd>Shinhan Bank</dd><dt>Sample payment purpose</dt><dd>Payroll · tax · insurance</dd><dt>Live bank verification</dt><dd>Unknown / unavailable</dd><dt>Sample card</dt><dd>****-4081</dd></dl><dl><dt>Sample payroll cycle</dt><dd>Monthly</dd><dt>Sample payment day</dt><dd>25th</dd><dt>Sample attendance cutoff</dt><dd>22nd</dd></dl><dl><dt>Sample pension identifier</dt><dd>12345678901</dd><dt>Sample insurance identifier</dt><dd>234567890</dd><dt>Live registration</dt><dd>Unknown / unavailable</dd></dl></div>"#.to_owned() + &identity_anchor_html("org"),
        "onboarding" => sample_setup_html(),
        _ => panic!("unsupported account sample panel"),
    };
    format!(r#"<div data-account-facts="{panel}">{facts}</div>"#)
}

fn sample_setup_html() -> String {
    let completed = 3;
    let percent = completed * 100 / SAMPLE_SETUP_STEPS.len();
    let steps: String = SAMPLE_SETUP_STEPS.iter().enumerate().map(|(index, (id, label))| format!(
        r#"<li data-setup-step="{id}" class="{class}" aria-current="{current}"><span>{number}. {label}</span><small data-account-local="step-state">{state}</small></li>"#,
        class = if index < completed { "done" } else if index == completed { "active" } else { "" },
        current = if index == completed { "step" } else { "false" }, number = index + 1,
        state = if index < completed { "Sample complete" } else { "Sample incomplete" },
    )).collect();
    format!(
        r#"<div class="identity-panel-copy"><p class="screen-anchor">NINE STEP SAMPLE SETUP</p><h4 id="onboarding-panel-title">Sample workspace setup</h4><p>Inspect and complete nine illustrative steps in page memory. Reload resets this sample. No tenant admission, go-live, employee readiness or legal/bank verification occurs.</p></div>{anchors}<div class="onboarding-flow"><div class="onboarding-progress"><strong data-onboarding-percent="true" data-account-local="count">{completed} of 9 sample steps complete · {percent}%</strong><span class="score-bar" data-account-local="progress" role="progressbar" aria-valuenow="{percent}" aria-valuemin="0" aria-valuemax="100" aria-label="Sample setup completion" aria-valuetext="3 of 9 sample steps complete" style="--bar: {percent}%"><em aria-hidden="true"></em></span><button type="button" data-identity-action="back-onboarding" data-account-focus="setup-back">Inspect previous step</button><button type="button" data-identity-action="advance-onboarding" data-account-focus="setup-advance">Complete sample step</button><button type="button" data-identity-action="reset-onboarding" data-account-focus="setup-reset">Reset sample setup</button></div><article class="setup-current-step"><p class="screen-anchor">INSPECTING SAMPLE STEP</p><h5 data-account-local="current-step">4. Payroll calendar example</h5><p data-account-local="setup-status" role="status">Sample calendar inspection only; no approval or verification occurs.</p></article><ol class="onboarding-steps" aria-label="Nine sample setup steps">{steps}</ol></div>"#,
        anchors = onboarding_anchor_html(),
    )
}

pub(super) fn account_sample_source(envelope: &TenantRenderEnvelope) -> String {
    serde_json::to_string(&(
        envelope.context.id(),
        daily_sample_label(envelope),
        HubSample::envelope_source(envelope),
        (
            account_person_html(),
            account_context_html(),
            ACCOUNT_PANELS.map(|panel| (panel, account_panel_html(panel))),
        ),
    ))
    .expect("complete account sample source serializes")
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn account_channel_templates(envelope: &TenantRenderEnvelope) -> String {
    [HubChannel::Mail, HubChannel::Community]
        .into_iter()
        .map(|channel| {
            format!(
                "<template data-account-hub-template=\"{}\">{}</template>",
                channel.label(),
                communication_hub_html(envelope, channel)
            )
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
pub(super) fn account_channel_templates(_envelope: &TenantRenderEnvelope) -> String {
    String::new()
}
