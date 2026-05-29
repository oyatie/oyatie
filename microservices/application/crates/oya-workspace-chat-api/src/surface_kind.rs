//! Workspace surface-kind classification for the 14 P06 surfaces.
//!
//! `WorkspaceSurfaceKind` enumerates every Workspace Axis surface defined in
//! M03-P06 (SPEC.md §4). Consumers use the `surface_tag` constant to embed a
//! stable routing key in audit-chain rows, OpenAPI operationId blocks, and
//! policy Cedar resource URIs without taking any external dependency.
// ADR-0083 Tier 3: tests legitimately use `.unwrap()` / `.expect()` /
// `panic!()` to assert invariants under the `cfg(test)` exemption.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// All 14 Workspace Axis surfaces defined in M03-P06 (SPEC.md §4).
///
/// Each variant maps to exactly one surface row in the SPEC and carries a
/// stable lowercase dot-separated tag used in routing, audit, and Cedar.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum WorkspaceSurfaceKind {
    /// SMTP/IMAP/JMAP inbound and outbound mail.
    Mail,
    /// CalDAV calendar event store.
    Calendar,
    /// Yrs-CRDT collaborative document editor.
    Docs,
    /// Yrs-CRDT collaborative spreadsheet.
    Sheets,
    /// Yrs-CRDT collaborative presentation.
    Slides,
    /// Static site publishing.
    Sites,
    /// Object + metadata drive with KMS-shred.
    Drive,
    /// WebRTC SFU video/audio meeting.
    Meet,
    /// Persistent threaded messaging.
    Chat,
    /// Form builder and submission ingest.
    Forms,
    /// CardDAV address book.
    AddressBook,
    /// Task list with due-date and assignee.
    Tasks,
    /// Rich-text note store.
    Notes,
    /// Neural machine translation invoke.
    Translate,
}

impl WorkspaceSurfaceKind {
    /// Stable lowercase dot-separated routing tag.
    ///
    /// The tag is embedded verbatim in audit-chain `surface` fields, OpenAPI
    /// `x-oya-surface` extensions, and Cedar resource URIs. It must not change
    /// once shipped.
    pub const fn surface_tag(self) -> &'static str {
        match self {
            Self::Mail => "workspace.mail",
            Self::Calendar => "workspace.calendar",
            Self::Docs => "workspace.docs",
            Self::Sheets => "workspace.sheets",
            Self::Slides => "workspace.slides",
            Self::Sites => "workspace.sites",
            Self::Drive => "workspace.drive",
            Self::Meet => "workspace.meet",
            Self::Chat => "workspace.chat",
            Self::Forms => "workspace.forms",
            Self::AddressBook => "workspace.address-book",
            Self::Tasks => "workspace.tasks",
            Self::Notes => "workspace.notes",
            Self::Translate => "workspace.translate",
        }
    }

    /// All 14 variants in SPEC.md §4 row order.
    pub const ALL: [WorkspaceSurfaceKind; 14] = [
        Self::Mail,
        Self::Calendar,
        Self::Docs,
        Self::Sheets,
        Self::Slides,
        Self::Sites,
        Self::Drive,
        Self::Meet,
        Self::Chat,
        Self::Forms,
        Self::AddressBook,
        Self::Tasks,
        Self::Notes,
        Self::Translate,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_array_has_exactly_14_variants() {
        assert_eq!(WorkspaceSurfaceKind::ALL.len(), 14);
    }

    #[test]
    fn surface_tags_are_unique_and_prefixed() {
        let mut tags = std::collections::BTreeSet::new();
        for kind in WorkspaceSurfaceKind::ALL {
            let tag = kind.surface_tag();
            assert!(
                tag.starts_with("workspace."),
                "tag must start with workspace.: {tag}"
            );
            assert!(tags.insert(tag), "duplicate tag: {tag}");
        }
    }

    #[test]
    fn surface_tags_match_spec_rows() {
        assert_eq!(WorkspaceSurfaceKind::Mail.surface_tag(), "workspace.mail");
        assert_eq!(
            WorkspaceSurfaceKind::Calendar.surface_tag(),
            "workspace.calendar"
        );
        assert_eq!(WorkspaceSurfaceKind::Docs.surface_tag(), "workspace.docs");
        assert_eq!(
            WorkspaceSurfaceKind::Sheets.surface_tag(),
            "workspace.sheets"
        );
        assert_eq!(
            WorkspaceSurfaceKind::Slides.surface_tag(),
            "workspace.slides"
        );
        assert_eq!(WorkspaceSurfaceKind::Sites.surface_tag(), "workspace.sites");
        assert_eq!(WorkspaceSurfaceKind::Drive.surface_tag(), "workspace.drive");
        assert_eq!(WorkspaceSurfaceKind::Meet.surface_tag(), "workspace.meet");
        assert_eq!(WorkspaceSurfaceKind::Chat.surface_tag(), "workspace.chat");
        assert_eq!(WorkspaceSurfaceKind::Forms.surface_tag(), "workspace.forms");
        assert_eq!(
            WorkspaceSurfaceKind::AddressBook.surface_tag(),
            "workspace.address-book"
        );
        assert_eq!(WorkspaceSurfaceKind::Tasks.surface_tag(), "workspace.tasks");
        assert_eq!(WorkspaceSurfaceKind::Notes.surface_tag(), "workspace.notes");
        assert_eq!(
            WorkspaceSurfaceKind::Translate.surface_tag(),
            "workspace.translate"
        );
    }

    #[test]
    fn ordering_is_deterministic_and_spec_row_order() {
        let kinds: Vec<_> = WorkspaceSurfaceKind::ALL.to_vec();
        assert_eq!(kinds[0], WorkspaceSurfaceKind::Mail);
        assert_eq!(kinds[8], WorkspaceSurfaceKind::Chat);
        assert_eq!(kinds[13], WorkspaceSurfaceKind::Translate);
    }
}
