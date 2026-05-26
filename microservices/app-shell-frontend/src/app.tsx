// Root application shell.
// Ported from crates/oya-application-shell-frontend-prototype/src/app.rs (App component).
// SSR-first, streaming hydration per ADR-0372 D1 + ADR-0067 §5.
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
