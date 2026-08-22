export interface BufferState {
	buffers: number[][];
	stringState: string;
	itemsProcessed: number;
}

export interface PalletData {
	id: number;
	rawSequence: number[];
	stacks: number[][];
	satisfactoryCount: number;
	unsatisfactoryCount: number;
	satisfactoryPercentage: number;
	minStackSize: number;
	maxStackSize: number;
	avgStackSize: number;
	steps: number | bigint; // amount of stacks before emptying the buffers to finalize the pallet
}

export interface OverallStats {
	totalPallets: number;
	totalStacks: number;
	satisfactoryCount: number;
	unsatisfactoryCount: number;
	satisfactoryPercentage: number;
	minStackSize: number;
	maxStackSize: number;
	avgStackSize: number;
}

export interface SeqResult {
	outputs: number[][];
	steps: bigint | number;
}
