import { mount } from "svelte";
import "./app/app.css";
import AppShell from "./app/shell/AppShell.svelte";

const app = mount(AppShell, {
  target: document.getElementById("app")!,
});

export default app;
