// Archived transition shell header.
// ADR-0393 makes Leptos/Rust-WASM canonical; the local signals here are
// retained only to keep this read-only transition surface testable.
import { createSignal, type Component } from "solid-js";

const ShellHeader: Component = () => {
  const [activeRoute, setActiveRoute] = createSignal<string>("fd001");
  const [activeComms, setActiveComms] = createSignal<string>("Messenger");

  return (
    <header class="app-header" role="banner">
      <div class="top-breadcrumb" aria-label="Breadcrumb">
        <span>Oyatie Cloud</span>
        <span class="sep">/</span>
        <span>Operations</span>
        <span class="sep">/</span>
        <strong>Control Center</strong>
      </div>

      <div class="header-route-strip" aria-label="FD-001 and Oyatie Cloud quick routes">
        {(["fd001", "cloud", "work-hub", "evidence"] as const).map((route) => (
          <button
            type="button"
            class={activeRoute() === route ? "is-selected" : undefined}
            aria-pressed={activeRoute() === route}
            onClick={() => setActiveRoute(route)}
          >
            <span>
              {{ fd001: "FD-001", cloud: "Cloud", "work-hub": "Comms", evidence: "Audit" }[route]}
            </span>
            {{ fd001: "Product graph", cloud: "Substrate", "work-hub": "Work hub", evidence: "Evidence" }[route]}
          </button>
        ))}

        <div class="header-comms-switcher" aria-label="Built-in communications quick routes">
          {(["Messenger", "Mail", "Community"] as const).map((surface) => (
            <button
              type="button"
              class={activeComms() === surface ? "is-selected" : undefined}
              aria-pressed={activeComms() === surface}
              onClick={() => setActiveComms(surface)}
            >
              {surface}
            </button>
          ))}
        </div>

        <small>REC-WF-7741 · local quick routes</small>
      </div>

      <button
        class="command-trigger"
        type="button"
        aria-haspopup="dialog"
        aria-label="Open command palette"
      >
        <span aria-hidden="true">⌕</span>
        <span>Search actions, objects, workflows</span>
        <kbd>⌘K</kbd>
      </button>

      <div class="header-actions" aria-label="Shell status">
        <button type="button" class="header-status">SSR shell</button>
        <button type="button" class="header-status muted">ADR-0393 transition</button>
        <button
          type="button"
          class="header-icon"
          aria-label="Open notifications"
        >
          ◔
          <span class="header-badge" aria-label="3 unread notifications">3</span>
        </button>
        <button type="button" class="header-icon" aria-label="Open settings">
          ⚙
        </button>
      </div>
    </header>
  );
};

export default ShellHeader;
