// Client hydration entry point.
import { mount, StartClient } from "@solidjs/start/client";

export default function startClient() {
  return mount(() => <StartClient />, document.getElementById("app")!);
}
