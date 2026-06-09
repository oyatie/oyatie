// Archived transition shell rail.
// Navigation labels mirror the product graph while ADR-0393 migrates the
// canonical shell to Leptos/Rust-WASM.
import type { Component } from "solid-js";

interface ShellRailProps {
  activeSection?: string;
}

const ShellRail: Component<ShellRailProps> = (props) => {
  return (
    <aside class="app-rail" aria-label="Product navigation">
      <div class="rail-brand">
        <span class="rail-mark" aria-hidden="true">O</span>
        <div>
          <strong>Oyatie</strong>
          <span>Control Center</span>
        </div>
        <code>v0.1</code>
      </div>

      <section class="rail-proof-card" aria-label="FD-001 and Oyatie Cloud shell proof">
        <p>FD-001 TENANT WORKLOADS</p>
        <strong>Product graph on Oyatie Cloud</strong>
        <span>Messenger · Mail · Community dogfood the substrate.</span>
        <small>REC-WF-7741 · cell-us-east-2 · read-only shell routes</small>
        <div class="rail-proof-actions" aria-label="Persistent shell proof routes">
          <button type="button" class="is-selected" aria-pressed="true">Product graph</button>
          <button type="button">Cloud</button>
          <button type="button">Evidence</button>
          <button type="button">Work hub</button>
        </div>
        <div class="rail-comms-switcher" aria-label="Built-in Work Hub surface routes">
          <button type="button" class="is-selected" aria-pressed="true">Messenger</button>
          <button type="button">Mail</button>
          <button type="button">Community</button>
        </div>
      </section>

      <p class="rail-group">Run the company</p>
      <a class="rail-nav active" href="#shell-main" aria-current="page">
        <span aria-hidden="true">⌂</span>Command center
      </a>
      <a class="rail-nav" href="#command-center-workbench">
        <span aria-hidden="true">▥</span>Action Inbox<em>8</em>
      </a>
      <a class="rail-nav" href="#governance-analytics">
        <span aria-hidden="true">↟</span>Governance analytics
      </a>

      <p class="rail-group">Operate</p>
      <a class="rail-nav" href="#business-logics">
        <span aria-hidden="true">⌬</span>Business Logics<em>17</em>
      </a>
      <a class="rail-nav" href="#tasks-title">
        <span aria-hidden="true">☑</span>Tasks<em>73</em>
      </a>
      <a class="rail-nav" href="#schedule-title">
        <span aria-hidden="true">◷</span>Schedule
      </a>
      <a class="rail-nav" href="#workflow-studio">
        <span aria-hidden="true">⌘</span>Workflow Studio
      </a>
      <a class="rail-nav" href="#work-hub">
        <span aria-hidden="true">✉</span>Messenger · Mail · Community<em>18</em>
      </a>
      <a class="rail-nav" href="#cloud-ops-cockpit">
        <span aria-hidden="true">◫</span>Cloud Ops
      </a>

      <p class="rail-group">Money</p>
      <a class="rail-nav" href="#payroll-cockpit">
        <span aria-hidden="true">₩</span>Payroll
      </a>
      <a class="rail-nav" href="#ledger-preview">
        <span aria-hidden="true">▤</span>Ledger
      </a>
      <a class="rail-nav" href="#vendors-spend">
        <span aria-hidden="true">◇</span>Vendors &amp; spend
      </a>
      <a class="rail-nav" href="#billing-tax">
        <span aria-hidden="true">▧</span>Billing &amp; tax
      </a>
      <a class="rail-nav" href="#finops-pane">
        <span aria-hidden="true">₩</span>FinOps
      </a>

      <p class="rail-group">Compliance</p>
      <a class="rail-nav" href="#filing-readiness">
        <span aria-hidden="true">□</span>Filing readiness<em>2</em>
      </a>
      <a class="rail-nav" href="#audit-ledger">
        <span aria-hidden="true">◱</span>Audit ledger
      </a>
      <a class="rail-nav" href="#policy-access">
        <span aria-hidden="true">⚿</span>Policy &amp; access
      </a>

      <p class="rail-group">People</p>
      <a class="rail-nav" href="#identity-employees">
        <span aria-hidden="true">◎</span>Employees
      </a>
      <a class="rail-nav" href="#leave-time">
        <span aria-hidden="true">◫</span>Leave &amp; time
      </a>
      <a class="rail-nav" href="#identity-workforce-platform">
        <span aria-hidden="true">⚿</span>Auth · Org
      </a>

      <p class="rail-group">Trust</p>
      <a class="rail-nav" href="#resource-inventory">
        <span aria-hidden="true">▤</span>Resource inventory
      </a>
      <a class="rail-nav" href="#modules-title">
        <span aria-hidden="true">▦</span>Service catalog
      </a>
      <a class="rail-nav" href="#evidence-spine">
        <span aria-hidden="true">▥</span>Evidence spine
      </a>
      <a class="rail-nav" href="#deployment-gates">
        <span aria-hidden="true">✓</span>Deployment gates
      </a>
      <a class="rail-nav" href="#ontology-title">
        <span aria-hidden="true">◎</span>Object graph
      </a>
      <a class="rail-nav" href="#intelligence-title">
        <span aria-hidden="true">✦</span>Copilot rail
      </a>

      <div class="workspace-switch">
        <span class="workspace-avatar" aria-hidden="true">N</span>
        <div>
          <strong>Northwind</strong>
          <span>Enterprise · US/EU/KR</span>
        </div>
      </div>
    </aside>
  );
};

export default ShellRail;
