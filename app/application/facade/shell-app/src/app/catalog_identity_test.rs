use super::*;

#[test]
fn served_search_entries_resolve_one_owned_catalog_input_in_every_context() {
    for context in OperatorContext::ALL {
        for state in ["available", "loading", "mismatched"] {
            let mut envelope = server_derived_envelope(context);
            if state == "mismatched" {
                envelope.modules[0].description.push_str(" changed");
            }
            let available = state != "loading";
            let owner = Owner::new();
            let (header, hero, catalog) = owner.with(|| {
                (
                    ShellHeader().into_view().to_html(),
                    HeroPanel().into_view().to_html(),
                    service_catalog_panel(envelope, Signal::derive(move || available))
                        .into_view()
                        .to_html(),
                )
            });
            owner.cleanup();
            let target = "catalog-sample-search";
            for entry in [header, hero] {
                let marker = format!("href=\"#{target}\"");
                assert_eq!(
                    entry.matches(&marker).count(),
                    1,
                    "{context:?}/{state}: shell entry targets Catalog search"
                );
                let offset = entry.find(&marker).unwrap();
                let start = entry[..offset].rfind("<a ").expect("native search anchor");
                let anchor = entry[start..].split_once("</a>").unwrap().0;
                assert!(anchor.contains("Search Catalog samples"));
                for claim in [
                    "aria-haspopup",
                    "data-command-trigger",
                    "<kbd",
                    "⌘K",
                    "Search actions, objects, workflows",
                ] {
                    assert!(!anchor.contains(claim), "search does not advertise {claim}");
                }
            }
            assert_eq!(catalog.matches(&format!("id=\"{target}\"")).count(), 1);
            let offset = catalog.find(&format!("id=\"{target}\"")).unwrap();
            let start = catalog[..offset]
                .rfind("<input ")
                .expect("existing Catalog search input");
            let input = catalog[start..].split_once('>').unwrap().0;
            for marker in [
                "data-catalog-search=\"true\"",
                "data-catalog-focus=\"search\"",
                "type=\"search\"",
                "aria-describedby=\"catalog-availability\"",
            ] {
                assert!(
                    input.contains(marker),
                    "source-owned input retains {marker}"
                );
            }
            assert_eq!(input.contains(" disabled"), state != "available");
            assert!(catalog.contains(&format!("data-catalog-context=\"{}\"", context.id())));
            assert!(catalog.contains(&format!(
                "data-catalog-available=\"{}\"",
                state == "available"
            )));
        }
    }
}

fn assert_catalog_state(envelope: TenantRenderEnvelope, expected: bool) {
    let owner = Owner::new();
    let html = owner.with(|| {
        service_catalog_panel(envelope, Signal::derive(|| true))
            .into_view()
            .to_html()
    });
    owner.cleanup();
    for attribute in ["admitted", "available"] {
        assert!(
            html.contains(&format!("data-catalog-{attribute}=\"{expected}\"")),
            "Catalog {attribute} should be {expected}"
        );
    }
}

#[test]
fn canonical_context_cards_render_available_and_changed_fields_fail_closed() {
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        assert_catalog_state(envelope.clone(), true);
        for index in 0..envelope.modules.len() {
            for field in ["name", "group", "description", "action_label"] {
                let mut changed = envelope.clone();
                let card = &mut changed.modules[index];
                let value = match field {
                    "name" => &mut card.name,
                    "group" => &mut card.group,
                    "description" => &mut card.description,
                    "action_label" => &mut card.action_label,
                    _ => unreachable!(),
                };
                value.push_str(" modified");
                assert_catalog_state(changed, false);
            }
        }
    }
}

fn render_catalog(envelope: TenantRenderEnvelope, available: bool) -> String {
    let owner = Owner::new();
    let html = owner.with(|| {
        service_catalog_panel(envelope, Signal::derive(move || available))
            .into_view()
            .to_html()
    });
    owner.cleanup();
    assert_eq!(html.matches("id=\"service-catalog\"").count(), 1);
    assert_eq!(html.matches("data-catalog-root=\"true\"").count(), 1);
    html
}

fn rendered_source(html: &str) -> String {
    html.split_once("data-catalog-source=\"")
        .unwrap()
        .1
        .split_once('"')
        .unwrap()
        .0
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

#[test]
fn catalog_full_source_changes_for_context_and_same_context_provenance() {
    let mut identities = std::collections::BTreeSet::new();
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        let identity = catalog_source(&envelope);
        let source: serde_json::Value = serde_json::from_str(&identity).unwrap();
        assert_eq!(source.as_array().unwrap().len(), 4);
        assert_eq!(source[0], context.id());
        assert_eq!(source[1], daily_sample_label(&envelope));
        assert_eq!(source[2], HubSample::envelope_source(&envelope));
        let records: Vec<_> = envelope.modules.iter().filter_map(catalog_record).collect();
        assert_eq!(source[3].as_array().unwrap().len(), records.len());
        for (index, record) in records.iter().enumerate() {
            assert_eq!(source[3][index][0], record.id);
            assert_eq!(source[3][index][1].as_array().unwrap().len(), 12);
            assert_eq!(
                source[3][index][1],
                serde_json::to_value(record.fields()).unwrap()
            );
        }
        assert_eq!(catalog_source(&envelope.clone()), identity);
        assert_eq!(
            rendered_source(&render_catalog(envelope.clone(), true)),
            identity
        );
        identities.insert(identity.clone());
        let mut changed = envelope.clone();
        changed
            .server_derivation_note
            .push_str(" changed provenance");
        assert_eq!(changed.context, envelope.context);
        assert_ne!(catalog_source(&changed), identity);
        assert_eq!(
            rendered_source(&render_catalog(changed.clone(), true)),
            catalog_source(&changed)
        );
    }
    assert_eq!(identities.len(), OperatorContext::ALL.len());
}

#[test]
fn catalog_render_keeps_full_source_when_availability_or_admission_changes() {
    for context in OperatorContext::ALL {
        let envelope = server_derived_envelope(context);
        let identity = catalog_source(&envelope);
        for available in [true, false] {
            let html = render_catalog(envelope.clone(), available);
            assert_eq!(rendered_source(&html), identity);
            assert!(html.contains("data-catalog-admitted=\"true\""));
            assert!(html.contains(&format!("data-catalog-available=\"{available}\"")));
        }
        let mut changed = envelope.clone();
        changed.omitted_capability_note = "Changed footer & <sample>".into();
        assert_eq!(catalog_source(&changed), identity);
        let html = render_catalog(changed, true);
        assert!(html.contains("Changed footer &amp; &lt;sample&gt;"));
        assert!(html.contains("data-catalog-available=\"true\""));

        let mut changed = envelope.clone();
        let mut unknown = changed.modules[0].clone();
        unknown.name = "Unrecognized sample".into();
        changed.modules.push(unknown);
        assert_eq!(catalog_source(&changed), identity);
        assert_catalog_state(changed, false);
        let mut missing = envelope.clone();
        missing.modules.remove(0);
        assert_catalog_state(missing, false);
        let mut duplicate = envelope.clone();
        duplicate.modules.push(duplicate.modules[0].clone());
        assert_catalog_state(duplicate, false);
    }
}
