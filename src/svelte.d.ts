// Lets `tsc --noEmit` accept imports of .svelte files (vite compiles them for real).
declare module "*.svelte" {
  import type { Component } from "svelte";
  const component: Component<Record<string, any>, Record<string, any>, string>;
  export default component;
}
