// Root route — the operator console command center.
// Port of crates/oya-application-shell-frontend-prototype/src/app.rs :: App + HeroPanel.
// SSR-rendered on first load; DashboardIsland hydrates client-side via createResource.
import { onMount, type Component } from "solid-js";
import ShellRail from "~/components/ShellRail";
import ShellHeader from "~/components/ShellHeader";
import DashboardIsland from "~/components/DashboardIsland";

const PROTOTYPE_NOTICE =
  "Prototype/demo only: no backend, no real auth, no PHI/PII, and no workflow execution · visual shell covers close, workflow, people, mail, messenger, and community.";

const IndexPage: Component = () => {
  onMount(() => {
    document.title = "Oyatie Operations · Cloud/Tenant Control Center";
  });

  return (
    <div class="oya-prototype-app">
      <ShellRail />
      <ShellHeader />

      <main
        id="shell-main"
        class="control-center"
        aria-labelledby="prototype-title"
        aria-describedby="prototype-notice"
      >
        {/* Hero panel — ported from HeroPanel component in app.rs */}
        <section class="hero-panel" aria-labelledby="prototype-title">
          <div class="hero-main">
            <div class="page-title-copy">
              <p class="screen-anchor">01 / COMMAND CENTER</p>
              <div class="hero-title-row">
                <h1 id="prototype-title">Operations · 2026 May, week 19</h1>
                <span class="hero-lens-chip">● Lens: tenant admin · Finance · 1,000 ppl</span>
              </div>
              <p id="prototype-notice" class="demo-notice" role="note">
                이번 주 운영 현황 — 마감, 신고, 인적자원, 결재 대기{" "}
                <span>This week — close, filings, people, approvals.</span>
              </p>
            </div>

            <section class="hero-close-strip" aria-label="FD-001 close command proof">
              <div>
                <p class="screen-anchor">FD-001 CLOSE COMMAND</p>
                <strong>April close proves the product workload on Oyatie Cloud</strong>
                <span>Ready · REC-CLOSE-2026-04 · cell-us-east-2 · local command only</span>
              </div>
              <div class="hero-close-actions" aria-label="Close package routes">
                <button type="button">Stage close</button>
                <button type="button">Ledger</button>
                <button type="button">Cloud proof</button>
                <button type="button">Evidence</button>
              </div>
            </section>

            <section
              class="render-architecture-strip"
              aria-label="SolidJS SSR shell and selective WASM hydration model"
            >
              <article class="selected">
                <p class="screen-anchor">SSR SHELL</p>
                <strong>Fast baseline, product graph visible first</strong>
                <span>
                  Navigation, proof copy, tenant posture, and core dashboards render before
                  island hydration. SolidJS streaming SSR per ADR-0372.
                </span>
              </article>
              <article>
                <p class="screen-anchor">SELECTIVE WASM</p>
                <strong>Only compute-bound widgets use Rust→WASM</strong>
                <span>
                  Workflow Studio canvas, data-grid sort/filter, and client-side crypto are
                  WASM modules mounted into this TS shell per ADR-0372 D3.
                </span>
              </article>
              <article>
                <p class="screen-anchor">LOCAL BOUNDARY</p>
                <strong>Visually functional, deliberately unwired</strong>
                <span>
                  No workflow execution, external send, IAM, billing, deploy, or cloud
                  mutation in this prototype.
                </span>
              </article>
            </section>
          </div>

          <div class="hero-side">
            <div class="hero-copy page-actions">
              <button type="button">New action</button>
              <button type="button">Search ⌘K</button>
              <button type="button" class="primary">Close April →</button>
            </div>
          </div>
        </section>

        {/* Prototype notice */}
        <p class="demo-notice" role="note" aria-live="polite">
          {PROTOTYPE_NOTICE}
        </p>

        {/* The main operator-console surface — context-switched, server-derived envelope */}
        <DashboardIsland />
      </main>
    </div>
  );
};

export default IndexPage;
