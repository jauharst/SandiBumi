<!--
  Template for new Svelte components in SandiBumi.

  Svelte is available for NEW complex UI (endpoint matrices, settings panels, rich
  tables); existing vanilla-TS panels stay as they are. A component mounts into any
  DOM element the current code already owns:

    import { mount, unmount } from "svelte";
    import Example from "../components/Example.svelte";

    const instance = mount(Example, {
      target: hostElement,                      // any HTMLElement (modal body, pane, …)
      props: { label: "Runs", onPick: (n) => console.log(n) },
    });
    // later, when the dialog/pane closes:
    unmount(instance);

  Style rules: use the app's CSS variables (var(--panel-bg), var(--text), var(--accent),
  var(--border)) so every theme — including Pertamina — repaints the component for free.
-->
<script lang="ts">
  let { label = "Counter", onPick = (_n: number) => {} }: {
    label?: string;
    onPick?: (n: number) => void;
  } = $props();

  let count = $state(0);

  function bump(): void {
    count += 1;
    onPick(count);
  }
</script>

<div class="svelte-example">
  <span>{label}: {count}</span>
  <button class="lp-btn" onclick={bump}>+1</button>
</div>

<style>
  .svelte-example {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text);
  }
</style>
