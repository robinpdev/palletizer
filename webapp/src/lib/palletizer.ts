import { unzipSync, strFromU8 } from 'fflate';
import { readSheet } from 'read-excel-file/browser';
import init, { PreSorter, SortStrategy, runseq } from 'rust';
import type { PalletData, OverallStats, SeqResult, BufferState } from './types';
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

export async function processExcelFile(file: File): Promise<PalletData[]> {
	await init();

	const sheetNames = await getSheetNames(file);
	const targetSheet = sheetNames.find((s) => s.includes('SPREIDING')) || sheetNames[0];
	if (!targetSheet) {
		throw new Error(m.no_valid_worksheet());
	}

	const excel = await readSheet(file, targetSheet);
	let newseq: number[] = [];
	let seqs: number[][] = [];

	for (const row of excel) {
		if (typeof row[1] !== 'number') {
			continue;
		}
		let item = row[1] as number;
		if (item >= 30) {
			if (newseq.length !== 0) {
				seqs.push(newseq);
			}
			newseq = [];
			continue;
		}
		newseq.push(item);
	}
	if (newseq.length > 0) {
		seqs.push(newseq);
	}

	const pallets: PalletData[] = [];
	let palletIdx = 1;

	for (const seq of seqs) {
		if (seq.length === 0) continue;
		const result = runseq(new Uint32Array(seq)) as SeqResult;
		const pallet = calculatePalletStats(result.outputs || [], palletIdx++, seq);
		pallets.push(pallet);
	}

	return pallets;
}

export function calculatePalletStats(
	stacks: number[],
	id: number,
	rawSequence: number[] = []
): PalletData {
	if (stacks.length === 0) {
		return {
			id,
			rawSequence,
			stacks: [],
			satisfactoryCount: 0,
			unsatisfactoryCount: 0,
			satisfactoryPercentage: 0,
			minStackSize: 0,
			maxStackSize: 0,
			avgStackSize: 0
		};
	}

	const satisfactoryCount = stacks.filter((h) => h >= 20).length;
	const unsatisfactoryCount = stacks.length - satisfactoryCount;
	const minStackSize = Math.min(...stacks);
	const maxStackSize = Math.max(...stacks);
	const sum = stacks.reduce((a, b) => a + b, 0);
	const avgStackSize = parseFloat((sum / stacks.length).toFixed(1));
	const satisfactoryPercentage = parseFloat(((satisfactoryCount / stacks.length) * 100).toFixed(1));

	return {
		id,
		rawSequence,
		stacks,
		satisfactoryCount,
		unsatisfactoryCount,
		satisfactoryPercentage,
		minStackSize,
		maxStackSize,
		avgStackSize
	};
}

export function calculateOverallStats(pallets: PalletData[]): OverallStats {
	if (pallets.length === 0) {
		return {
			totalPallets: 0,
			totalStacks: 0,
			satisfactoryCount: 0,
			unsatisfactoryCount: 0,
			satisfactoryPercentage: 0,
			minStackSize: 0,
			maxStackSize: 0,
			avgStackSize: 0
		};
	}

	const allStacks = pallets.flatMap((p) => p.stacks);
	if (allStacks.length === 0) {
		return {
			totalPallets: pallets.length,
			totalStacks: 0,
			satisfactoryCount: 0,
			unsatisfactoryCount: 0,
			satisfactoryPercentage: 0,
			minStackSize: 0,
			maxStackSize: 0,
			avgStackSize: 0
		};
	}

	const satisfactoryCount = allStacks.filter((h) => h >= 20).length;
	const unsatisfactoryCount = allStacks.length - satisfactoryCount;
	const minStackSize = Math.min(...allStacks);
	const maxStackSize = Math.max(...allStacks);
	const sum = allStacks.reduce((a, b) => a + b, 0);
	const avgStackSize = parseFloat((sum / allStacks.length).toFixed(1));
	const satisfactoryPercentage = parseFloat(
		((satisfactoryCount / allStacks.length) * 100).toFixed(1)
	);

	return {
		totalPallets: pallets.length,
		totalStacks: allStacks.length,
		satisfactoryCount,
		unsatisfactoryCount,
		satisfactoryPercentage,
		minStackSize,
		maxStackSize,
		avgStackSize
	};
}

export function getBufferStateForOutput(
	rawSequence: number[],
	targetOutputIndex: number
): BufferState {
	const sorter = PreSorter.new(4, 30, 25, 20, SortStrategy.FirstFitStrategy);
	let outputCount = 0;
	let itemsProcessed = 0;

	for (const item of rawSequence) {
		itemsProcessed++;
		const result = sorter.add_wasm(item) as any;
		if (result && typeof result === 'object' && 'Output' in result) {
			if (outputCount === targetOutputIndex) {
				break;
			}
			outputCount++;
		}
	}

	const bufferTypedArray = sorter.get_buffers() as Uint32Array;
	const buffers: number[] = Array.from(bufferTypedArray);
	const stringState = sorter.stringstate();

	return {
		buffers,
		stringState,
		itemsProcessed
	};
}
