<script lang="ts">
	import init, { PreSorter, SortStrategy, runseq } from 'rust';
	import * as XLSX from 'xlsx';
	import readExcelFile, { readSheet, parseSheetData } from 'read-excel-file/browser';
	// we need onMount to run init
	import { onMount } from 'svelte';

	import { unzipSync, strFromU8 } from 'fflate';
	import { m } from '$lib/paraglide/messages';

	export async function getSheetNames(file: File): Promise<string[]> {
		const buffer = new Uint8Array(await file.arrayBuffer());

		const files = unzipSync(buffer);

		const workbookXml = files['xl/workbook.xml'];

		if (!workbookXml) {
			throw new Error(m.invalid_xlsx_file());
		}

		const xml = strFromU8(workbookXml);

		const document = new DOMParser().parseFromString(xml, 'application/xml');

		return Array.from(document.querySelectorAll('sheet'))
			.map((sheet) => sheet.getAttribute('name'))
			.filter((name): name is string => name !== null);
	}

	let lastupdate = $state(0);
	let outputs: number[][] = $state([]);
	let env: PreSorter | null = $state(null);
	let files: FileList | null = $state(null);

	onMount(async () => {
		await init(); // init initializes memory addresses needed by WASM and that will be used by JS/TS
		env = PreSorter.new(4, 30, 25, 20, SortStrategy.FirstFitStrategy);
		refresh();
	});

	function add(item: number) {
		let result = env?.add_wasm(item) ?? {};
		console.log(result);
		if ('NotPossible' in result) {
			alert(m.v1_item_cannot_added());
		} else if ('NoOutput' in result) {
			//empty
		} else if ('Output' in result) {
			if (outputs.length == 0) outputs.push([]);
			outputs[0].unshift(result.Output);
		}
		refresh();
	}

	function refresh() {
		lastupdate = Date.now();
	}

	type SeqResult = {
		outputs: number[];
		steps: bigint;
	};

	async function testseq() {
		if (files && files.item(0)) {
			const sheetname = (await getSheetNames(files[0])).filter((s) => s.includes('SPREIDING'))[0];
			const excel = await readSheet(files[0], sheetname);
			let newseq: number[] = [];
			let seqs: number[][] = [];
			for (const row of excel) {
				if (typeof row[1] != 'number') {
					continue;
				}
				let item = row[1] as number;
				if (item >= 30) {
					if (newseq.length != 0) seqs.push(newseq);
					newseq = [];
					continue;
				}
				newseq.push(item);
			}
			seqs.push(newseq);

			console.log(seqs);

			for (let seq of seqs) {
				let result = runseq(new Uint32Array(seq)) as SeqResult;
				outputs.push(result.outputs);
			}
		}
	}
</script>

<main>
	<h1>{m.v1_test_title()}</h1>
	<p>{m.v1_click_buttons_hint()}</p>

	{#await init()}
		<p>{m.v1_loading()}</p>
	{:then}
		<input type="file" id="input" bind:files />
		<button onclick={() => testseq()}>{m.v1_test_seq()}</button>

		<br />

		{#each { length: 24 }, index}
			<button class="btn" onclick={() => add(index + 1)}>{index + 1}</button>
		{/each}

		{#key lastupdate}
			<!-- <p>current output:</p>

			<p style="white-space: pre-wrap; font-family: monospace; font-size: 1.2em;">
				[{'■'.repeat(env?.currentOutput)}{' '.repeat(30 - env?.currentOutput)}] {env?.currentOutput}
			</p> -->

			<p>{m.v1_buffers()}</p>
			<p style="white-space: pre-wrap; font-family: monospace; font-size: 1.2em;">
				{env?.stringstate()}
			</p>
		{/key}

		<h2>{m.v1_outputs()}</h2>
		{#each outputs as outputseq, index (index)}
			<h2>{m.v1_pallet_label({ number: index + 1 })}</h2>
			{#each outputseq as output, i2 (i2)}
				<p
					style="white-space: pre-wrap; font-family: monospace; color: {output < 20
						? 'orange'
						: 'default'}"
				>
					[{'■'.repeat(output)}{' '.repeat(30 - output)}] {output}
				</p>
			{/each}
		{/each}
	{/await}
</main>

<style>
	main {
		font-family: monospace;
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
