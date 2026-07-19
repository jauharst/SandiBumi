import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

export default {
  // Lets <script lang="ts"> inside .svelte components use full TypeScript.
  preprocess: vitePreprocess(),
};
