use super::*;

#[derive(Clone, Copy)]
pub(super) enum PayrollPreview {
    Finance,
    Workflow,
    Mail,
    Evidence,
}

impl PayrollPreview {
    fn detail(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Finance => (
                "payroll-finance-preview",
                "Payroll finance preview",
                "Sample payroll payable: ₩894,000,000. Sample withholding: ₩118,400,000. These amounts are illustrative.",
            ),
            Self::Workflow => (
                "payroll-workflow-preview",
                "Payroll workflow gate preview",
                "Sample gate: Finance close reviews a four-insurance change before April close. No workflow run has started.",
            ),
            Self::Mail => (
                "payroll-mail-preview",
                "Payroll Mail brief preview",
                "Sample brief for Finance close: review the four-insurance change before April close. This brief has not been queued or sent.",
            ),
            Self::Evidence => (
                "payroll-evidence-preview",
                "Payroll evidence preview",
                "Example receipt: REC-PAY-2026-04. No receipt has been created or sealed.",
            ),
        }
    }
}

pub(super) fn payroll_preview_detail(destination: PayrollPreview) -> impl IntoView {
    let (id, title, body) = destination.detail();
    view! {
        <article
            id=id
            class="payroll-preview-detail"
            tabindex="-1"
            aria-labelledby=format!("{id}-title")
            data-payroll-preview-context="sample-april-close"
        >
            <p class="screen-anchor">"PAYROLL · SAMPLE APRIL CLOSE"</p>
            <h4 id=format!("{id}-title")>{title}</h4>
            <p>"Preview only. These are sample figures, not your tenant's payroll run."</p>
            <p>{body}</p>
            <dl class="service-kv">
                <div><dt>"Sample period"</dt><dd>"2026-04"</dd></div>
                <div><dt>"Payroll result"</dt><dd>"No live payroll result connected"</dd></div>
                <div><dt>"Workforce"</dt><dd>"118 sample employees; not validated"</dd></div>
                <div><dt>"Receipt"</dt><dd>"Example only; not created"</dd></div>
            </dl>
            <a href="#payroll-cockpit">"Return to payroll preview"</a>
        </article>
    }
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_payroll_preview_detail(destination: PayrollPreview) -> String {
    payroll_preview_detail(destination).into_view().to_html()
}

pub(super) fn payroll_preview_card() -> impl IntoView {
    view! {
        <article id="payroll-cockpit" class="service-card payroll-card" tabindex="-1">
                <p class="screen-anchor">"PAYROLL CLOSE · PREVIEW"</p>
                <h4>"Sample 2026-04 payroll close"</h4>
                <p class="service-card-brief">"Local sample connecting Finance, Workflow, Mail, and Evidence previews. No payroll run or receipt has been created."</p>
                <div class="service-metric-row">
                    <span><strong>"73%"</strong><small>"sample close progress"</small></span>
                    <span><strong>"5.4d"</strong><small>"sample cycle time"</small></span>
                    <span><strong>"₩2.18M"</strong><small>"sample cost of delay"</small></span>
                </div>
                <ol class="service-checklist">
                    <li><span class="status-chip">"sample blocker"</span>"4대보험 변동 확인 필요"</li>
                    <li><span class="status-chip">"sample brief"</span>"Payroll reminder example; not queued or sent"</li>
                    <li><span class="status-chip">"example receipt"</span>"REC-PAY-2026-04; not created"</li>
                </ol>
                <div class="service-card-actions">
                    <a href="#payroll-finance-preview">"Finance cockpit"</a>
                    <a href="#payroll-workflow-preview">"Workflow gate"</a>
                    <a href="#payroll-mail-preview">"Mail brief"</a>
                    <a href="#payroll-evidence-preview">"Evidence"</a>
                </div>
            </article>
    }
}

#[cfg(any(feature = "ssr", test))]
pub(super) fn static_payroll_preview_card() -> String {
    payroll_preview_card().into_view().to_html()
}
