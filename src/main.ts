import { mount } from "svelte";
import "@fontsource-variable/bricolage-grotesque";
import "@fontsource-variable/hanken-grotesk";
import "@fontsource-variable/spline-sans-mono";
// Loaded so the overlay font picker can preview each choice in its own face.
import "@fontsource-variable/quicksand";
import "@fontsource-variable/fraunces";
import "./app/app.css";
import AppShell from "./app/shell/AppShell.svelte";

const app = mount(AppShell, {
  target: document.getElementById("app")!,
});

export default app;
