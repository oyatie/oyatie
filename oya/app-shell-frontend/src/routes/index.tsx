// Root route — archived operator console transition surface.
// ADR-0393 keeps Leptos/Rust-WASM canonical; this TS route remains as
// read-only migration evidence for shell layout and accessibility checks.
import { onMount, type Component } from "solid-js";
import ShellRail from "~/components/ShellRail";
import ShellHeader from "~/components/ShellHeader";
import DashboardIsland from "~/components/DashboardIsland";

const SHELL_TRANSITION_NOTICE =
  "ADR-0393 transition evidence only: Leptos/Rust-WASM is canonical; this archived shell handles no auth, PHI/PII, workflow execution, cloud/IAM/billing/deploy mutation, or production tenant data.";

const IndexPage: Component = () => {
  onMount(() => {
    document.title = "Oyatie Operations · Cloud/Tenant Control Center";
  });

  return (
    <div class="oya-shell-app">
      <ShellRail />
      <ShellHeader />

      <main
        id="shell-main"
        class="control-center"
        aria-labelledby="shell-title"
        aria-describedby="shell-notice"
      >
        {/* Hero panel — ported from HeroPanel component in app.rs */}
        <section class="hero-panel" aria-labelledby="shell-title">
          <div class="hero-main">
            <div class="page-title-copy">
              <p class="screen-anchor">01 / COMMAND CENTER</p>
              <div class="hero-title-row">
                <h1 id="shell-title">Operations · 2026 May, week 19</h1>
                <span class="hero-lens-chip">● Lens: tenant admin · Finance · 1,000 ppl</span>
              </div>
              <p id="shell-notice" class="shell-notice" role="note">
                이번 주 운영 현황 — 마감, 신고, 인적자원, 결재 대기{" "}
                <span>This week — close, filings, people, approvals.</span>
              </p>
            </div>

            <section class="hero-close-strip" aria-label="FD-001 close command proof">
              <div>
                <p class="screen-anchor">FD-001 CLOSE COMMAND</p>
                <strong>April close proves the product workload on Oyatie Cloud</strong>
                <span>Ready · REC-CLOSE-2026-04 · cell-us-east-2 · read-only evidence</span>
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
              aria-label="ADR-0393 shell migration and read-only evidence model"
            >
              <article class="selected">
                <p class="screen-anchor">SSR SHELL</p>
                <strong>Fast baseline, product graph visible first</strong>
                <span>
                  Navigation, proof copy, tenant posture, and core dashboards render before
                  island hydration. archived SSR transition evidence; Leptos/Rust-WASM is canonical per ADR-0393.
                </span>
              </article>
              <article>
                <p class="screen-anchor">SELECTIVE WASM</p>
                <strong>Only compute-bound widgets use Rust→WASM</strong>
                <span>
                  Workflow Studio canvas, data-grid sort/filter, and client-side crypto are
                  compute islands in the canonical Leptos/Rust-WASM shell per ADR-0393.
                </span>
              </article>
              <article>
                <p class="screen-anchor">LOCAL BOUNDARY</p>
                <strong>Read-only transition surface</strong>
                <span>
                  No workflow execution, external send, IAM, billing, deploy, or cloud
                  mutation is exposed from this archived shell.
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
        <p class="shell-notice" role="note" aria-live="polite">
          {SHELL_TRANSITION_NOTICE}
        </p>

        {/* The main operator-console surface — context-switched, server-derived envelope */}
        <DashboardIsland />
      </main>
    </div>
  );
};

export default IndexPage;
