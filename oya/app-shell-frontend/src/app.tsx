// Root application shell for the archived SolidJS transition surface.
// ADR-0393 makes the Leptos/Rust-WASM portal shell canonical; this surface stays
// SSR/typecheck/build-testable as migration evidence only.
import { Router } from "@solidjs/router";
import { FileRoutes } from "@solidjs/start/router";
import { Suspense } from "solid-js";
import "./styles/tokens.css";
import "./styles/app.css";

export default function App() {
  return (
    <Router
      root={(props) => (
        <>
          <a class="skip-link" href="#shell-main">
            Skip to dashboard
          </a>
          <Suspense>{props.children}</Suspense>
        </>
      )}
    >
      <FileRoutes />
    </Router>
  );
}
