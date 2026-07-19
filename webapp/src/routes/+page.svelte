<script lang="ts">
	import init, { PreSorter, SortStrategy } from 'rust';
	// we need onMount to run init
	import { onMount } from 'svelte';

	let lastupdate = $state(0);
	let outputs: number[] = $state([]);
	let env: PreSorter | null = $state(null);

	onMount(async () => {
		await init(); // init initializes memory addresses needed by WASM and that will be used by JS/TS
		env = PreSorter.new(4, 30, 25, 20, SortStrategy.FirstFitStrategy);
		refresh();
	});

	function add(item: number) {
		let result = env?.add_wasm(item) ?? {};
		console.log(result);
		if ('NotPossible' in result) {
			alert("item can't be added");
		} else if ('NoOutput' in result) {
			//empty
		} else if ('Output' in result) {
			outputs.unshift(result.Output);
		}
		refresh();
	}

	function refresh() {
		lastupdate = Date.now();
	}
</script>

<main>
	<h1>PreSorter test</h1>
	<p>Click buttons to add stack of size n</p>

	{#await init()}
		<p>loading</p>
	{:then}
		{#each { length: 24 }, index}
			<button class="btn" onclick={() => add(index + 1)}>{index + 1}</button>
		{/each}

		{#key lastupdate}
			<p>current output:</p>

			<p style="white-space: pre-wrap; font-family: monospace; font-size: 1.2em;">
				[{'■'.repeat(env?.currentOutput)}{' '.repeat(30 - env?.currentOutput)}] {env?.currentOutput}
			</p>

			<p>buffers:</p>
			<p style="white-space: pre-wrap; font-family: monospace; font-size: 1.2em;">
				{env?.stringstate()}
			</p>
		{/key}

		<h2>Outputs</h2>
		{#each outputs as output, index (index)}
			<p style="white-space: pre-wrap; font-family: monospace;">
				[{'■'.repeat(output)}{' '.repeat(30 - output)}] {output}
			</p>
		{/each}
	{/await}
</main>

<style>
	main {
		font-family: monospace;
	}

	h1 {
	}

	.btn {
		background-color: #04aa6d;
		border: none;
		color: white;
		padding: 20px;
		text-align: center;
		text-decoration: none;
		display: inline-block;
		font-size: 16px;
		margin: 4px 2px;
		border-radius: 25%;
		width: 4em;
		aspect-ratio: 1 / 1;
		transition: 0.2s;
	}

	.btn:hover {
		background-color: orange;
	}
</style>
