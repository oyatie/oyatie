use super::*;

fn envelope(context: OperatorContext) -> TenantRenderEnvelope {
    let mut source = server_derived_envelope(context);
    source.messages = vec![
        MessageItem {
            from: "Ärztin <owner>".into(),
            channel: "Messenger".into(),
            preview: "Evidence & 안녕".into(),
        },
        MessageItem {
            from: "Ärztin <owner>".into(),
            channel: "Messenger".into(),
            preview: "Evidence & 안녕".into(),
        },
        MessageItem {
            from: "Mail <owner>".into(),
            channel: "Mail".into(),
            preview: "Mail & body".into(),
        },
    ];
    source.community = vec![CommunityItem {
        space: "Council".into(),
        topic: "Community <review>".into(),
        activity: "Community & body".into(),
    }];
    source
}

#[test]
fn source_reconciliation_preserves_only_exact_successful_sample() {
    let mutations: Vec<fn(&mut TenantRenderEnvelope)> = vec![
        |e| {
            e.context = if e.context == OperatorContext::TenantAdmin {
                OperatorContext::CorporateOffice
            } else {
                OperatorContext::TenantAdmin
            }
        },
        |e| e.tenant_name.push('x'),
        |e| e.role_name.push('x'),
        |e| e.tenant_class.push('x'),
        |e| e.accreditation.label.push('x'),
        |e| e.accreditation.healthcare_enabled = !e.accreditation.healthcare_enabled,
        |e| e.accreditation.explanation.push('x'),
        |e| e.server_derivation_note.push('x'),
        |e| e.messages[0].from.push('x'),
        |e| e.messages[0].channel.push('x'),
        |e| e.messages[0].preview.push('x'),
        |e| e.community[0].space.push('x'),
        |e| e.community[0].topic.push('x'),
        |e| e.community[0].activity.push('x'),
        |e| {
            e.messages.pop();
        },
        |e| {
            e.community.clear();
        },
    ];
    for context in OperatorContext::ALL {
        let source = envelope(context);
        let mut current = HubSample::from_envelope(&source);
        current.select_channel(HubChannel::Mail);
        current.composer = "exact local draft".into();
        assert!(current.queue());
        current.query = "local".into();
        current.filter = HubFilter::Drafts;
        current.composer = "unfinished body".into();
        current.review_selected();
        assert_eq!(hub_sample_for_envelope(&source, &current), current);
        for mutate in &mutations {
            let mut changed = source.clone();
            mutate(&mut changed);
            let next = hub_sample_for_envelope(&changed, &current);
            assert_ne!(next.source, current.source);
            assert_eq!(next.channel, HubChannel::Messenger);
            assert!(next.drafts.is_empty() && next.query.is_empty() && next.composer.is_empty());
            assert_eq!(next.filter, HubFilter::All);
            assert!(next.reviewed.is_empty());
            assert_eq!(next.selected, next.visible().first().map(|row| row.key));
        }
        assert!(hub_context_available(context, Some(&source), false, false));
        assert!(!hub_context_available(context, Some(&source), true, false));
        assert!(!hub_context_available(context, Some(&source), false, true));
        assert!(!hub_context_available(context, None, false, false));
        for other in OperatorContext::ALL
            .into_iter()
            .filter(|other| *other != context)
        {
            assert!(!hub_context_available(other, Some(&source), false, false));
        }
    }
}

#[test]
fn duplicate_identity_exact_queue_copy_and_clear_are_local() {
    let mut state = HubSample::from_envelope(&envelope(OperatorContext::TenantAdmin));
    let rows = state.visible();
    assert_eq!(rows[0].item, rows[1].item);
    assert_ne!(rows[0].key, rows[1].key);
    let sample_key = rows[1].key;
    let selected = rows[1].item.clone();
    state.selected = Some(sample_key);
    let body = " \n안녕 & <body>\n ";
    state.composer = body.into();
    assert!(state.queue());
    let first = state.selected_row().unwrap();
    assert!(matches!(first.key, HubItemKey::Draft(_)));
    assert_eq!(first.item.body, body);
    assert!(state.composer.is_empty());
    state.selected = Some(sample_key);
    assert_eq!(state.selected_row().unwrap().item, selected);
    state.composer = body.into();
    assert!(state.queue());
    let second = state.selected_row().unwrap();
    assert_ne!(first.key, second.key);
    let before_drafts = state.drafts.clone();
    state.composer = " \n\t ".into();
    assert!(!state.queue());
    assert_eq!(state.drafts, before_drafts);
    assert!(state.outcome.contains("nonempty"));
    let mut before_clear = state.clone();
    before_clear.composer.clear();
    state.composer.clear();
    assert_eq!(state, before_clear);
    for destination in [HubChannel::Mail, HubChannel::Community] {
        state.select_channel(HubChannel::Messenger);
        state.selected = Some(sample_key);
        assert!(state.copy_to(destination));
        let copy = state.selected_row().unwrap();
        assert_eq!(state.channel, destination);
        assert_eq!(copy.item.title, selected.title);
        assert_eq!(copy.item.body, selected.body);
        assert!(copy.item.meta.contains(&selected.source));
        assert!(copy.item.meta.contains(&selected.meta));
        assert!(state.outcome.contains("No message sent or posted"));
    }
    state.selected = None;
    let drafts = state.drafts.clone();
    assert!(!state.copy_to(HubChannel::Mail));
    assert_eq!(state.drafts, drafts);
    assert!(state.outcome.contains("Select a visible"));
}

#[test]
fn one_unicode_projection_drives_filter_selection_and_empty_recovery() {
    let mut state = HubSample::from_envelope(&envelope(OperatorContext::TenantAdmin));
    state.query = " ÄRZTIN ".into();
    state.reconcile_selection();
    assert_eq!(state.visible().len(), 2);
    assert_eq!(state.selected_row().unwrap(), state.visible()[0]);
    state.filter = HubFilter::Drafts;
    state.reconcile_selection();
    assert!(state.visible().is_empty());
    assert!(state.selected_row().is_none());
    state.composer = "안녕 draft".into();
    state.queue();
    state.filter = HubFilter::Drafts;
    assert_eq!(state.visible().len(), 1);
    assert!(matches!(state.visible()[0].key, HubItemKey::Draft(_)));
    state.query = "missing".into();
    assert!(state.selected_row().is_none());
    state.reconcile_selection();
    assert_eq!(state.selected, None);
    state.query.clear();
    state.filter = HubFilter::Evidence;
    state.reconcile_selection();
    assert!(!state.visible().is_empty());
    assert!(
        state
            .visible()
            .iter()
            .all(|row| !matches!(row.key, HubItemKey::Draft(_)))
    );
    state.filter = HubFilter::All;
    state.reconcile_selection();
    assert_eq!(state.visible().len(), state.rows(state.channel).len());
    assert!(state.selected_row().is_some());
}

#[test]
fn shared_ssr_has_complete_escaped_channels_and_related_controlled_tabs() {
    for context in OperatorContext::ALL {
        let source = envelope(context);
        for channel in HubChannel::ALL {
            let html = communication_hub_html(&source, channel);
            assert!(html.contains(&format!("data-comms-board-surface=\"{}\"", channel.label())));
            for marker in [
                "class=\"hub-list\"",
                "class=\"hub-detail\"",
                "class=\"comms-detail-card\"",
                "class=\"hub-composer\"",
            ] {
                assert!(html.contains(marker));
            }
            assert_eq!(html.matches("aria-selected=\"true\"").count(), 1);
            assert_eq!(html.matches("tabindex=\"0\"").count(), 1);
            for related in HubChannel::ALL {
                assert_eq!(
                    html.matches(&format!("id=\"{}\"", hub_tab_id(related)))
                        .count(),
                    1
                );
                assert_eq!(
                    html.matches(&format!("id=\"{}\"", hub_panel_id(related)))
                        .count(),
                    1
                );
                assert!(html.contains(&format!("aria-controls=\"{}\"", hub_panel_id(related))));
                assert!(html.contains(&format!("aria-labelledby=\"{}\"", hub_tab_id(related))));
            }
            let composer = html
                .split_once("id=\"hub-composer-input\"")
                .unwrap()
                .1
                .split_once('>')
                .unwrap()
                .0;
            assert!(composer.contains("disabled"));
            assert!(composer.contains("aria-describedby=\"hub-availability\""));
            assert!(html.contains("matching WebAssembly island"));
            assert!(!html.contains("<owner>") && !html.contains("<review>"));
            let expected = match channel {
                HubChannel::Messenger => "Ärztin &lt;owner&gt;",
                HubChannel::Mail => "Mail &lt;owner&gt;",
                HubChannel::Community => "Community &lt;review&gt;",
            };
            assert!(html.contains(&format!("<strong>{expected}</strong>")));
            assert!(html.contains(&format!("<h4>{expected}</h4>")));
        }
    }
    let html = static_dashboard_html();
    let source = server_derived_envelope(OperatorContext::TenantAdmin);
    let payroll = static_payroll_preview_detail(PayrollPreview::Mail);
    let hub = communication_hub_html(&source, HubChannel::Messenger);
    let opening = r#"<section id="work-hub" class="panel communications-panel"><div class="panel-header"><p class="eyebrow">Messenger · Mail · Community</p><h3>Work hub</h3></div>"#;
    let frame = |body: &str| format!("{opening}{body}</section>");
    let expected = frame(&format!("{payroll}{hub}"));
    let accepts = |document: &str| {
        let before = document.split(&expected).next().unwrap();
        document.matches("id=\"work-hub\"").count() == 1
            && document.matches("id=\"payroll-mail-preview\"").count() == 1
            && document.matches(&expected).count() == 1
            && before.matches("<template").count() == before.matches("</template>").count()
    };
    assert!(accepts(&html) && !accepts(&format!("<template>{expected}</template>")));
    assert!(accepts(&format!("<template>{hub}</template>{html}")));
    let (before, after) = hub.split_once('>').unwrap();
    let inside = format!("{before}>{payroll}{after}");
    for malformed in [
        frame(&inside),
        frame(&format!("{hub}{payroll}")),
        frame(&format!("{payroll}<div></div>")),
        format!("{html}{payroll}"),
        format!("<template>{payroll}{hub}</template>{}", frame("")),
        format!("{payroll}{}", frame(&hub)),
        format!("{expected}{expected}"),
        expected.strip_suffix("</section>").unwrap().to_string(),
    ] {
        assert!(!accepts(&malformed));
    }
}

#[test]
fn hub_manual_focus_and_dom_tab_ownership_do_not_select_each_other() {
    use crate::app::keyboard_navigation::dom_tablist_owned;
    for class in [
        "settings-tabs",
        "identity-tabs",
        "finance-tabs",
        "cockpit-tabs",
        "resource-tabs",
    ] {
        assert!(dom_tablist_owned(class));
    }
    for class in ["hub-tabs", "unknown-tabs", "hub-tabs cockpit-tabs", ""] {
        assert!(!dom_tablist_owned(class));
    }
    let mut state = HubSample::from_envelope(&envelope(OperatorContext::TenantAdmin));
    state.tab_stop = hub_focus_channel(state.channel, "ArrowRight").unwrap();
    assert_eq!(state.channel, HubChannel::Messenger);
    assert_eq!(state.tab_stop, HubChannel::Mail);
    assert_eq!(
        hub_focus_channel(HubChannel::Community, "ArrowRight"),
        Some(HubChannel::Messenger)
    );
    assert_eq!(
        hub_focus_channel(HubChannel::Messenger, "ArrowLeft"),
        Some(HubChannel::Community)
    );
    assert_eq!(
        hub_focus_channel(HubChannel::Mail, "Home"),
        Some(HubChannel::Messenger)
    );
    assert_eq!(
        hub_focus_channel(HubChannel::Mail, "End"),
        Some(HubChannel::Community)
    );
    assert_eq!(hub_focus_channel(HubChannel::Mail, "Enter"), None);
}
