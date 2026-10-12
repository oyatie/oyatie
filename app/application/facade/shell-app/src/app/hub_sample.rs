use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct HubSample {
    pub(super) source: String,
    pub(super) context: OperatorContext,
    pub(super) provenance: String,
    pub(super) messages: Vec<MessageItem>,
    pub(super) communities: Vec<CommunityItem>,
    pub(super) channel: HubChannel,
    pub(super) tab_stop: HubChannel,
    pub(super) selected: Option<HubItemKey>,
    pub(super) query: String,
    pub(super) filter: HubFilter,
    pub(super) composer: String,
    pub(super) drafts: Vec<LocalDraft>,
    pub(super) next_draft: u64,
    pub(super) reviewed: Vec<HubItemKey>,
    pub(super) preview: bool,
    pub(super) outcome: String,
}

impl HubSample {
    pub(super) fn empty(context: OperatorContext) -> Self {
        Self {
            source: String::new(),
            context,
            provenance: String::new(),
            messages: Vec::new(),
            communities: Vec::new(),
            channel: HubChannel::Messenger,
            tab_stop: HubChannel::Messenger,
            selected: None,
            query: String::new(),
            filter: HubFilter::All,
            composer: String::new(),
            drafts: Vec::new(),
            next_draft: 1,
            reviewed: Vec::new(),
            preview: false,
            outcome: "Sample communications only; no external send or receipt created.".into(),
        }
    }

    pub(super) fn from_envelope(envelope: &TenantRenderEnvelope) -> Self {
        let mut sample = Self::empty(envelope.context);
        sample.source = Self::envelope_source(envelope);
        sample.provenance = format!(
            "{} · {} · {} · {}",
            envelope.context.id(),
            envelope.tenant_name,
            envelope.role_name,
            envelope.server_derivation_note
        );
        sample.messages = envelope.messages.clone();
        sample.communities = envelope.community.clone();
        sample.select_first();
        sample
    }

    pub(super) fn envelope_source(envelope: &TenantRenderEnvelope) -> String {
        serde_json::to_string(&(
            envelope.context,
            &envelope.tenant_name,
            &envelope.role_name,
            &envelope.tenant_class,
            &envelope.accreditation,
            &envelope.server_derivation_note,
            &envelope.messages,
            &envelope.community,
        ))
        .expect("native Work Hub sample source is serializable")
    }

    pub(super) fn queue(&mut self) -> bool {
        if self.composer.trim().is_empty() {
            self.outcome = "Enter a nonempty local draft before queueing.".into();
            return false;
        }
        let body = std::mem::take(&mut self.composer);
        self.insert_draft(
            self.channel,
            "Local draft queued".into(),
            body,
            self.provenance.clone(),
        );
        true
    }

    fn insert_draft(
        &mut self,
        channel: HubChannel,
        title: String,
        body: String,
        provenance: String,
    ) {
        let id = self.next_draft;
        self.next_draft += 1;
        self.drafts.insert(
            0,
            LocalDraft {
                id,
                surface: channel.surface(),
                title,
                body,
                provenance,
            },
        );
        self.select_channel(channel);
        self.selected = Some(HubItemKey::Draft(id));
        self.outcome = format!(
            "{} local draft selected; stored in this browser only. No message sent or posted.",
            channel.label()
        );
    }

    pub(super) fn copy_to(&mut self, channel: HubChannel) -> bool {
        let Some(row) = self.selected_row() else {
            self.outcome = "Select a visible sample or local draft before copying.".into();
            return false;
        };
        let origin = format!(
            "Copied from {} · {} · {} · {}",
            self.channel.label(),
            row.item.source,
            row.item.meta,
            self.provenance
        );
        self.insert_draft(channel, row.item.title, row.item.body, origin);
        true
    }

    pub(super) fn review_selected(&mut self) {
        if let Some(row) = self.selected_row() {
            if !self.reviewed.contains(&row.key) {
                self.reviewed.push(row.key);
            }
            self.outcome = format!(
                "{} marked reviewed in this local sample only; no approval recorded.",
                row.item.title
            );
        }
    }
}

pub(super) fn hub_context_available(
    requested: OperatorContext,
    observed: Option<&TenantRenderEnvelope>,
    loading: bool,
    error: bool,
) -> bool {
    !loading && !error && observed.is_some_and(|envelope| envelope.context == requested)
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    static HUB_HANDOFF: std::cell::RefCell<Option<HubSample>> = const { std::cell::RefCell::new(None) };
}

pub(super) fn hub_sample_for_envelope(
    envelope: &TenantRenderEnvelope,
    current: &HubSample,
) -> HubSample {
    let mut fresh = HubSample::from_envelope(envelope);
    if fresh.source == current.source {
        return current.clone();
    }
    if !current.source.is_empty() {
        fresh.outcome = "Sample source changed; previous local drafts and filters cleared.".into();
    }
    #[cfg(target_arch = "wasm32")]
    return HUB_HANDOFF.with(|saved| {
        let matching = saved
            .borrow()
            .as_ref()
            .filter(|sample| sample.source == fresh.source)
            .cloned();
        let mut next = matching.unwrap_or(fresh);
        if !current.source.is_empty() {
            next.outcome =
                "Sample source changed; previous local drafts and filters cleared.".into();
        }
        *saved.borrow_mut() = Some(next.clone());
        next
    });
    #[cfg(not(target_arch = "wasm32"))]
    fresh
}

pub(super) fn edit_hub_sample(sample: RwSignal<HubSample>, edit: impl FnOnce(&mut HubSample)) {
    let _ = sample.try_update(edit);
    #[cfg(target_arch = "wasm32")]
    if let Some(current) = sample.try_get_untracked() {
        HUB_HANDOFF.with(|saved| *saved.borrow_mut() = Some(current));
    }
}
