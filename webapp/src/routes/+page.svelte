<script lang="ts">
  import init, { runseq } from "rust";
  // we need onMount to run init
  import { onMount } from "svelte";

  let x = $state("loading...");

  onMount(async () => {
    await init(); // init initializes memory addresses needed by WASM and that will be used by JS/TS
    x = runseq();
  })
</script>
<h1>Welcome to SvelteKit</h1>
<p>Visit <a href="https://svelte.dev/docs/kit">svelte.dev/docs/kit</a> to read the documentation</p>


<p style="white-space: pre-wrap; font-family: monospace;">{x}</p>
