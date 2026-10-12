use super::{FragmentActivation, FragmentFocus};

const OLD: &str = "https://preview.invalid/";
const EVIDENCE: &str = "https://preview.invalid/#evidence-spine";

fn activated(old: &str) -> FragmentActivation {
    FragmentActivation::new(old.into(), EVIDENCE.into(), "evidence-spine".into())
}

#[test]
fn earlier_hash_completion_preserves_only_newer_bound_focus() {
    let focus = FragmentFocus {
        url: EVIDENCE.into(),
        time_stamp: Some(533.1),
    };
    assert!(focus.follows_hash(529.3, EVIDENCE, EVIDENCE, true));
    assert!(!focus.follows_hash(533.1, EVIDENCE, EVIDENCE, true));
    assert!(!focus.follows_hash(534.0, EVIDENCE, EVIDENCE, true));
    assert!(!focus.follows_hash(f64::NAN, EVIDENCE, EVIDENCE, true));
    assert!(!focus.follows_hash(529.3, OLD, EVIDENCE, true));
    assert!(!focus.follows_hash(529.3, EVIDENCE, OLD, true));
    assert!(!focus.follows_hash(529.3, EVIDENCE, EVIDENCE, false));
}

#[test]
fn mount_focus_snapshot_does_not_invent_native_event_ordering() {
    let focus = FragmentFocus {
        url: EVIDENCE.into(),
        time_stamp: None,
    };
    assert!(!focus.follows_hash(529.3, EVIDENCE, EVIDENCE, true));
}

#[test]
fn automatic_mount_preserves_initial_body_and_same_fragment_focus() {
    let mut remembered = None;
    FragmentFocus::remember_mount_focus(&mut remembered, None);
    assert!(FragmentFocus::permits_mount_restore(
        true,
        remembered.as_ref(),
        EVIDENCE,
        None,
        "evidence-spine"
    ));
    FragmentFocus::remember_mount_focus(
        &mut remembered,
        Some(FragmentFocus {
            url: EVIDENCE.into(),
            time_stamp: None,
        }),
    );
    assert!(FragmentFocus::permits_mount_restore(
        true,
        remembered.as_ref(),
        EVIDENCE,
        Some("evidence-spine"),
        "evidence-spine"
    ));
}

#[test]
fn clean_listener_retry_preserves_pre_mount_search_focus() {
    let url = "https://preview.invalid/#work-hub";
    let mut remembered = None;
    FragmentFocus::remember_mount_focus(
        &mut remembered,
        Some(FragmentFocus {
            url: url.into(),
            time_stamp: None,
        }),
    );
    // Clean rollback leaves the owner unready before mount removes the search node.
    assert!(!FragmentFocus::permits_mount_restore(
        false,
        remembered.as_ref(),
        url,
        Some(""),
        "work-hub"
    ));
    // The successful post-mount retry observes BODY, preserving the search intent.
    FragmentFocus::remember_mount_focus(&mut remembered, None);
    let focus = remembered.as_ref().expect("pre-mount search intent");
    assert_eq!(focus.url, url);
    assert_eq!(focus.time_stamp, None);
    assert!(!FragmentFocus::permits_mount_restore(
        true,
        remembered.as_ref(),
        url,
        Some(""),
        "work-hub"
    ));
}

#[test]
fn mount_snapshot_accepts_newer_non_body_focus() {
    let mut remembered = Some(FragmentFocus {
        url: OLD.into(),
        time_stamp: None,
    });
    FragmentFocus::remember_mount_focus(
        &mut remembered,
        Some(FragmentFocus {
            url: EVIDENCE.into(),
            time_stamp: None,
        }),
    );
    assert!(FragmentFocus::permits_mount_restore(
        true,
        remembered.as_ref(),
        EVIDENCE,
        Some("evidence-spine"),
        "evidence-spine"
    ));
    assert_eq!(remembered.as_ref().unwrap().time_stamp, None);
}

#[test]
fn automatic_mount_refuses_prior_control_and_later_external_focus() {
    let url = "https://preview.invalid/#work-hub";
    for (focused_id, time_stamp) in [("", None), ("oyatie-sign-in", Some(533.1))] {
        let focus = FragmentFocus {
            url: url.into(),
            time_stamp,
        };
        assert!(!FragmentFocus::permits_mount_restore(
            true,
            Some(&focus),
            url,
            Some(focused_id),
            "work-hub"
        ));
    }
}

#[test]
fn automatic_mount_refuses_changed_fragment_url_or_destination() {
    let focus = FragmentFocus {
        url: EVIDENCE.into(),
        time_stamp: None,
    };
    assert!(!FragmentFocus::permits_mount_restore(
        true,
        Some(&focus),
        OLD,
        Some("evidence-spine"),
        "evidence-spine"
    ));
    assert!(!FragmentFocus::permits_mount_restore(
        true,
        Some(&focus),
        EVIDENCE,
        Some("workflow-studio"),
        "evidence-spine"
    ));
}

#[test]
fn failed_fragment_listener_installation_does_not_license_mount_focus() {
    assert!(!FragmentFocus::permits_mount_restore(
        false,
        None,
        EVIDENCE,
        None,
        "evidence-spine"
    ));
    let focus = FragmentFocus {
        url: EVIDENCE.into(),
        time_stamp: None,
    };
    assert!(!FragmentFocus::permits_mount_restore(
        false,
        Some(&focus),
        EVIDENCE,
        Some("evidence-spine"),
        "evidence-spine"
    ));
}

#[test]
fn missing_fragment_activation_survives_own_navigation_and_matches_fresh_target() {
    let mut activation = activated(OLD);
    assert!(activation.popstate(true, EVIDENCE));
    assert!(activation.hashchange(true, OLD, EVIDENCE, EVIDENCE));
    assert!(activation.permits_restore(
        EVIDENCE,
        EVIDENCE,
        "evidence-spine",
        "evidence-spine",
        true,
        true
    ));
}

#[test]
fn own_hash_closes_pop_allowance_even_without_own_pop() {
    let mut activation = activated(OLD);
    assert!(activation.hashchange(true, OLD, EVIDENCE, EVIDENCE));
    assert!(!activation.popstate(true, EVIDENCE));
}

#[test]
fn own_pop_and_hash_do_not_license_later_same_url_history() {
    let mut activation = activated(OLD);
    assert!(activation.popstate(true, EVIDENCE));
    assert!(activation.hashchange(true, OLD, EVIDENCE, EVIDENCE));
    assert!(!activation.popstate(true, EVIDENCE));
    assert!(!activation.hashchange(true, OLD, EVIDENCE, EVIDENCE));
}

#[test]
fn same_url_activation_survives_synthetic_workflow_cancellation_only() {
    let mut activation = activated(EVIDENCE);
    assert!(activation.popstate(false, EVIDENCE));
    assert!(!activation.popstate(true, EVIDENCE));
    assert!(!activation.popstate(false, "https://preview.invalid/#workflow-studio"));
}

#[test]
fn deferred_activation_refuses_changed_anchor_target_url_and_unrelated_focus() {
    let activation = activated(OLD);
    assert!(!activation.permits_restore(
        EVIDENCE,
        EVIDENCE,
        "evidence-spine",
        "evidence-spine",
        true,
        false
    ));
    assert!(!activation.permits_restore(
        EVIDENCE,
        EVIDENCE,
        "evidence-spine",
        "evidence-spine",
        false,
        true
    ));
    assert!(!activation.permits_restore(
        OLD,
        EVIDENCE,
        "evidence-spine",
        "evidence-spine",
        true,
        true
    ));
    assert!(!activation.permits_restore(
        EVIDENCE,
        OLD,
        "evidence-spine",
        "evidence-spine",
        true,
        true
    ));
    assert!(!activation.permits_restore(
        EVIDENCE,
        EVIDENCE,
        "workflow-studio",
        "evidence-spine",
        true,
        true
    ));
    assert!(!activation.permits_restore(
        EVIDENCE,
        EVIDENCE,
        "evidence-spine",
        "workflow-studio",
        true,
        true
    ));
}

#[test]
fn deferred_ontology_focus_stays_with_its_bridge() {
    let url = "https://preview.invalid/#ontology-command-console";
    let activation =
        FragmentActivation::new(OLD.into(), url.into(), "ontology-command-console".into());
    assert!(!activation.permits_restore(
        url,
        url,
        "ontology-command-console",
        "ontology-command-console",
        true,
        true
    ));
}
