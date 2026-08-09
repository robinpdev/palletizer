<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, Tag, InlineNotification, Loading, TextInput } from 'carbon-components-svelte';
	import Upload from 'carbon-icons-svelte/lib/Upload.svelte';
	import Reset from 'carbon-icons-svelte/lib/Reset.svelte';
	import CheckmarkFilled from 'carbon-icons-svelte/lib/CheckmarkFilled.svelte';
	import WarningAltFilled from 'carbon-icons-svelte/lib/WarningAltFilled.svelte';
	import Analytics from 'carbon-icons-svelte/lib/Analytics.svelte';
	import Box from 'carbon-icons-svelte/lib/Box.svelte';
	import Information from 'carbon-icons-svelte/lib/Information.svelte';
	import Close from 'carbon-icons-svelte/lib/Close.svelte';
	import ChevronLeft from 'carbon-icons-svelte/lib/ChevronLeft.svelte';
	import ChevronRight from 'carbon-icons-svelte/lib/ChevronRight.svelte';
	import ChevronUp from 'carbon-icons-svelte/lib/ChevronUp.svelte';
	import ChevronDown from 'carbon-icons-svelte/lib/ChevronDown.svelte';

	import {
		processExcelFile,
		calculateOverallStats,
		calculatePalletStats,
		getBufferStateForOutput
	} from '$lib/palletizer';
	import type { PalletData, BufferState } from '$lib/types';
	import init, { runseq } from 'rust';

	let files = $state<FileList | null>(null);
	let pallets = $state<PalletData[]>([]);
	let isProcessing = $state(false);
	let errorMessage = $state<string | null>(null);
	let activeFileName = $state<string | null>(null);
	let wasmReady = $state(false);

	// Filter & Search states
	let searchQuery = $state('');
	let showOnlyUnsatisfactory = $state(false);

	// Buffer Inspector card state
	let inspectorOpen = $state(false);
	let inspectorCollapsed = $state(false);
	let inspectorPallet = $state<PalletData | null>(null);
	let inspectorStackIndex = $state<number>(0);
	let inspectorBufferState = $state<BufferState | null>(null);

	let overallStats = $derived(calculateOverallStats(pallets));

	let filteredPallets = $derived(
		pallets.filter((pallet) => {
			if (showOnlyUnsatisfactory && pallet.unsatisfactoryCount === 0) {
				return false;
			}
			if (searchQuery.trim()) {
				const q = searchQuery.toLowerCase();
				const matchesId =
					`pallet #${pallet.id}`.toLowerCase().includes(q) || `pallet ${pallet.id}`.includes(q);
				const matchesStack = pallet.stacks.some((s) => s.toString() === q);
				return matchesId || matchesStack;
			}
			return true;
		})
	);

	onMount(async () => {
		try {
			await init();
			wasmReady = true;
		} catch (err: any) {
			errorMessage = `Failed to initialize WASM module: ${err?.message || err}`;
		}
	});

	async function handleFileUpload(event: Event) {
		const target = event.target as HTMLInputElement;
		if (!target.files || target.files.length === 0) return;

		const file = target.files[0];
		activeFileName = file.name;
		isProcessing = true;
		errorMessage = null;

		try {
			pallets = await processExcelFile(file);
		} catch (err: any) {
			console.error(err);
			errorMessage = `Error processing Excel file "${file.name}": ${err?.message || 'Invalid format or missing sheet'}`;
			pallets = [];
		} finally {
			isProcessing = false;
		}
	}

	function resetData() {
		pallets = [];
		files = null;
		activeFileName = null;
		errorMessage = null;
		searchQuery = '';
		showOnlyUnsatisfactory = false;
		closeBufferInspector();
	}

	function runDemoSequence() {
		isProcessing = true;
		errorMessage = null;
		activeFileName = 'DEMO_SEQUENCE_SPREIDING.xlsx (Simulated)';

		setTimeout(() => {
			try {
				const demoSeqs = [
					[12, 15, 18, 22, 25, 19, 28, 14, 21, 30, 24, 18, 26, 16],
					[22, 28, 20, 29, 18, 25, 27, 21, 23, 16, 19, 24],
					[15, 14, 19, 28, 29, 24, 22, 21, 25, 17, 26, 30, 15, 22],
					[20, 21, 24, 28, 29, 23, 22, 25, 19, 15, 18, 27]
				];

				const newPallets: PalletData[] = [];
				let id = 1;
				for (const seq of demoSeqs) {
					const result = runseq(new Uint32Array(seq)) as { outputs: number[] };
					newPallets.push(calculatePalletStats(result.outputs || [], id++, seq));
				}
				pallets = newPallets;
			} catch (err: any) {
				errorMessage = `Demo simulation error: ${err?.message || err}`;
			} finally {
				isProcessing = false;
			}
		}, 300);
	}

	function openBufferInspector(pallet: PalletData, stackIndex: number) {
		if (inspectorOpen && inspectorPallet?.id === pallet.id && inspectorStackIndex === stackIndex) {
			closeBufferInspector();
			return;
		}
		inspectorPallet = pallet;
		inspectorStackIndex = stackIndex;
		inspectorCollapsed = false;
		if (pallet.rawSequence && pallet.rawSequence.length > 0) {
			inspectorBufferState = getBufferStateForOutput(pallet.rawSequence, stackIndex);
		} else {
			inspectorBufferState = null;
		}
		inspectorOpen = true;
	}

	function closeBufferInspector() {
		inspectorOpen = false;
		inspectorCollapsed = false;
		inspectorPallet = null;
		inspectorStackIndex = 0;
		inspectorBufferState = null;
	}

	function toggleInspectorCollapsed() {
		if (!inspectorOpen) return;
		inspectorCollapsed = !inspectorCollapsed;
	}

	function navigateInspector(delta: number) {
		if (!inspectorPallet) return;
		const nextIndex = inspectorStackIndex + delta;
		if (nextIndex >= 0 && nextIndex < inspectorPallet.stacks.length) {
			openBufferInspector(inspectorPallet, nextIndex);
		}
	}
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && closeBufferInspector()} />

<div class="v2-page-layout">
	<!-- TOP INDUSTRIAL STATUS BAR -->
	<section class="industrial-status-bar">
		<div class="status-indicator">
			<span class="pulse-dot" class:ready={wasmReady}></span>
			<span class="status-text">
				{wasmReady ? '[RUST PRE-SORTER ENGINE ONLINE]' : 'INITIALIZING WASM...'}
			</span>
		</div>
		<div class="specs-pill">
			MAX STACK HEIGHT: <strong>30</strong> | MIN TARGET: <strong>20 (SATISFACTORY)</strong>
		</div>
	</section>

	{#if errorMessage}
		<InlineNotification
			kind="error"
			title="SYSTEM ERROR:"
			subtitle={errorMessage}
			on:close={() => (errorMessage = null)}
			style="margin-bottom: 1.5rem;"
		/>
	{/if}

	<!-- UPLOAD SECTION -->
	{#if pallets.length === 0}
		<section class="upload-hero-section">
			<div class="upload-card">
				<div class="card-header">
					<Upload size={24} />
					<h2 class="card-title">01 // UPLOAD EXCEL BUNDLE PROFILE</h2>
				</div>
				<p class="card-description">
					Select an Excel file containing bundle sequence data (e.g. sheet containing <code
						>SPREIDING</code
					>). The WASM palletizer module will process stack distributions and generate output
					pallets.
				</p>

				<div class="file-drop-zone">
					<input
						type="file"
						id="excel-input"
						accept=".xlsx, .xls"
						onchange={handleFileUpload}
						disabled={!wasmReady || isProcessing}
						class="file-input-hidden"
					/>
					<label
						for="excel-input"
						class="drop-zone-label"
						class:disabled={!wasmReady || isProcessing}
					>
						{#if isProcessing}
							<Loading small />
							<span class="zone-text">CALCULATING PALLET STACKS...</span>
						{:else}
							<div class="upload-icon-wrapper">
								<Upload size={32} />
							</div>
							<span class="zone-primary">DRAG & DROP EXCEL FILE HERE OR CLICK TO BROWSE</span>
							<span class="zone-secondary">Supports .XLSX / .XLS spreadsheet files</span>
						{/if}
					</label>
				</div>

				<div class="demo-trigger-container">
					<span class="divider-text">OR TEST WITH SYSTEM DEMO</span>
					<Button
						kind="tertiary"
						size="small"
						icon={Analytics}
						disabled={!wasmReady || isProcessing}
						onclick={runDemoSequence}
					>
						LOAD SIMULATED BUNDLE DEMO
					</Button>
				</div>
			</div>
		</section>
	{:else}
		<!-- RESULTS & DASHBOARD SECTION -->
		<section class="dashboard-section">
			<!-- TOP CONTROL / FILE BAR -->
			<div class="active-file-bar">
				<div class="file-info">
					<Box size={18} />
					<span class="file-name-label">SOURCE: <strong>{activeFileName}</strong></span>
				</div>
				<div class="file-actions">
					<Button kind="danger-tertiary" size="small" icon={Reset} onclick={resetData}>
						UPLOAD NEW FILE
					</Button>
				</div>
			</div>

			<!-- OVERALL STATISTICS DASHBOARD (SUM OF ALL PALLETS) -->
			<div class="stats-overview-grid">
				<div class="stat-card stat-primary">
					<div class="stat-label">TOTAL PALLETS</div>
					<div class="stat-value">{overallStats.totalPallets}</div>
					<div class="stat-sub">Pallet Output Batches</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">TOTAL STACKS</div>
					<div class="stat-value">{overallStats.totalStacks}</div>
					<div class="stat-sub">Magazine Stacks Output</div>
				</div>

				<div class="stat-card" class:warning-bg={overallStats.satisfactoryPercentage < 75}>
					<div class="stat-label">SATISFACTORY STACKS</div>
					<div class="stat-value">
						{overallStats.satisfactoryPercentage}%
					</div>
					<div class="stat-sub">
						{overallStats.satisfactoryCount} / {overallStats.totalStacks} (Height ≥ 20)
					</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">MIN / MAX STACK SIZE</div>
					<div class="stat-value minmax">
						<span class="min-val">{overallStats.minStackSize}</span>
						<span class="sep">/</span>
						<span class="max-val">{overallStats.maxStackSize}</span>
					</div>
					<div class="stat-sub">Global Height Boundaries</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">AVG STACK HEIGHT</div>
					<div class="stat-value">{overallStats.avgStackSize}</div>
					<div class="stat-sub">Magazines per Stack</div>
				</div>
			</div>

			<!-- FILTER & CONTROLS -->
			<div class="filter-toolbar">
				<div class="search-box">
					<TextInput
						labelText=""
						placeholder="FILTER PALLETS OR STACK SIZES..."
						size="sm"
						bind:value={searchQuery}
					/>
				</div>
				<div class="toggle-box">
					<button
						class="filter-toggle-btn"
						class:active={showOnlyUnsatisfactory}
						onclick={() => (showOnlyUnsatisfactory = !showOnlyUnsatisfactory)}
					>
						{showOnlyUnsatisfactory ? '[x]' : '[ ]'} SHOW ONLY UNSATISFACTORY PALLETS (&lt; 20)
					</button>
				</div>
			</div>

			<!-- PALLET LIST DISPLAY -->
			<div class="pallets-container">
				{#if filteredPallets.length === 0}
					<div class="empty-filter-state">NO PALLETS MATCH THE CURRENT FILTER CRITERIA.</div>
				{:else}
					{#each filteredPallets as pallet (pallet.id)}
						<article class="pallet-card" class:has-unsatisfactory={pallet.unsatisfactoryCount > 0}>
							<!-- PALLET HEADER & METRICS -->
							<header class="pallet-header">
								<div class="pallet-title-group">
									<h3 class="pallet-title">PALLET #{pallet.id.toString().padStart(2, '0')}</h3>
									{#if pallet.unsatisfactoryCount > 0}
										<Tag type="red" size="sm" icon={WarningAltFilled}>
											{pallet.unsatisfactoryCount} UNSATISFACTORY STACK{pallet.unsatisfactoryCount >
											1
												? 'S'
												: ''}
										</Tag>
									{:else}
										<Tag type="green" size="sm" icon={CheckmarkFilled}>100% SATISFACTORY</Tag>
									{/if}
								</div>

								<div class="pallet-metrics">
									<div class="metric-item">
										<span class="m-label">STACKS:</span>
										<span class="m-value">{pallet.stacks.length}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">SATISFACTORY:</span>
										<span class="m-value" class:warn-text={pallet.satisfactoryPercentage < 100}>
											{pallet.satisfactoryPercentage}%
										</span>
									</div>
									<div class="metric-item">
										<span class="m-label">MIN / MAX:</span>
										<span class="m-value">{pallet.minStackSize} - {pallet.maxStackSize}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">AVG:</span>
										<span class="m-value">{pallet.avgStackSize}</span>
									</div>
								</div>
							</header>

							<!-- PALLET STACKS VISUALIZATION: VERTICAL BAR CHART -->
							<div class="chart-section">
								<div class="chart-instruction">
									<Information size={14} />
									<span
										>CLICK ON ANY VERTICAL BAR TO INSPECT THE PRESORTER BUFFER STATE AFTER THAT
										OUTPUT</span
									>
								</div>

								<div class="vertical-chart-container">
									<!-- Y-AXIS SCALE & REFERENCE LINES -->
									<div class="chart-y-axis">
										<span class="y-label y-max">30</span>
										<span class="y-label y-target">20</span>
										<span class="y-label y-mid">10</span>
										<span class="y-label y-zero">0</span>
									</div>

									<div class="chart-plot-area">
										<!-- THRESHOLD REFERENCE LINE AT Y=20 (66.66% height) -->
										<div class="threshold-line" title="Minimum Satisfactory Height = 20">
											<span class="threshold-tag">TARGET THRESHOLD (20)</span>
										</div>

										<!-- VERTICAL BARS -->
										<div class="vertical-bars-grid">
											{#each pallet.stacks as height, stackIdx}
												{@const isUnsatisfactory = height < 20}
												{@const barHeightPercent = Math.min(100, Math.max(0, (height / 30) * 100))}
												{@const isSelected =
													inspectorOpen &&
													inspectorPallet?.id === pallet.id &&
													inspectorStackIndex === stackIdx}

												<button
													type="button"
													class="vertical-bar-column"
													class:unsatisfactory={isUnsatisfactory}
													class:selected={isSelected}
													onclick={() => openBufferInspector(pallet, stackIdx)}
													title="Stack #{stackIdx +
														1}: height {height} magazines. Click to inspect presorter buffer state!"
												>
													<!-- NUMERIC HEIGHT VALUE ABOVE BAR -->
													<div class="bar-value-label" class:alert-text={isUnsatisfactory}>
														{height}
													</div>

													<!-- VERTICAL BAR TRACK AND FILL -->
													<div class="vertical-bar-track">
														<div
															class="vertical-bar-fill"
															class:unsatisfactory-fill={isUnsatisfactory}
															style="height: {barHeightPercent}%;"
														>
															<div class="bar-top-cap"></div>
														</div>
													</div>

													<!-- X-AXIS LABEL BELOW BAR -->
													<div class="bar-x-label">#{stackIdx + 1}</div>

													{#if isUnsatisfactory}
														<span class="bar-warning-dot" title="Unsatisfactory (< 20)">!</span>
													{/if}
												</button>
											{/each}
										</div>
									</div>
								</div>
							</div>

							<!-- COLLAPSIBLE BUFFER INSPECTOR CARD (inline under bar chart) -->
							{#if inspectorOpen && inspectorPallet && inspectorPallet.id === pallet.id}
								<section class="buffer-inspector-card" class:collapsed={inspectorCollapsed}>
									<header class="inspector-card-header">
										<div class="inspector-title-group">
											<span class="inspector-badge">PRESORTER DIAGNOSTICS</span>
											<h4 class="inspector-main-title">
												PALLET #{inspectorPallet.id.toString().padStart(2, '0')} // OUTPUT #{inspectorStackIndex +
													1}
											</h4>
											{#if inspectorCollapsed}
												<span
													class="inspector-collapsed-summary"
													class:alert-text={inspectorPallet.stacks[inspectorStackIndex] < 20}
													>HEIGHT: {inspectorPallet.stacks[inspectorStackIndex]}</span
												>
											{/if}
										</div>
										<div class="inspector-card-actions">
											<button
												class="collapse-inspector-btn"
												onclick={toggleInspectorCollapsed}
												aria-label={inspectorCollapsed ? 'Expand Inspector' : 'Collapse Inspector'}
												title={inspectorCollapsed ? 'Expand Inspector' : 'Collapse Inspector'}
											>
												{#if inspectorCollapsed}<ChevronDown size={20} />{:else}<ChevronUp
														size={20}
													/>{/if}
											</button>
											<button
												class="close-inspector-btn"
												onclick={closeBufferInspector}
												aria-label="Close Inspector"
												title="Close Inspector"
											>
												<Close size={20} />
											</button>
										</div>
									</header>

									{#if !inspectorCollapsed}
										<div class="inspector-body">
											<!-- STACK DETAILS BAR -->
											<div class="inspector-stack-summary">
												<div class="summary-item">
													<span class="s-label">OUTPUT HEIGHT:</span>
													<span
														class="s-value"
														class:alert-text={inspectorPallet.stacks[inspectorStackIndex] < 20}
													>
														{inspectorPallet.stacks[inspectorStackIndex]} MAGAZINES
													</span>
												</div>
												<div class="summary-item">
													<span class="s-label">STATUS:</span>
													{#if inspectorPallet.stacks[inspectorStackIndex] < 20}
														<Tag type="red" size="sm">UNSATISFACTORY (&lt; 20)</Tag>
													{:else}
														<Tag type="green" size="sm">SATISFACTORY (≥ 20)</Tag>
													{/if}
												</div>
												{#if inspectorBufferState}
													<div class="summary-item">
														<span class="s-label">ITEMS PROCESSED:</span>
														<span class="s-value"
															>{inspectorBufferState.itemsProcessed} items from sequence</span
														>
													</div>
												{/if}
											</div>

											{#if inspectorBufferState}
												<!-- VISUAL BUFFER SLOTS GAUGE (BUFFERS 1 TO 4) -->
												<div class="buffer-slots-container">
													<h5 class="section-subheading">BUFFER POSITIONS STATE (4 SLOTS)</h5>
													<div class="buffer-slots-grid">
														{#each inspectorBufferState.buffers as bufHeight, bufIdx}
															{@const fillPercent = (bufHeight / 30) * 100}
															<div class="buffer-slot-card" class:has-items={bufHeight > 0}>
																<div class="slot-header">
																	<span class="slot-title">BUFFER #{bufIdx + 1}</span>
																	<span class="slot-count">{bufHeight} / 30</span>
																</div>

																<div class="slot-gauge-track">
																	<div class="slot-gauge-fill" style="width: {fillPercent}%;"></div>
																</div>

																<div class="slot-status">
																	{bufHeight === 0
																		? 'EMPTY (AVAILABLE)'
																		: `${bufHeight} MAGAZINES STACKED`}
																</div>
															</div>
														{/each}
													</div>
												</div>

												<!-- ASCII STRING STATE FROM RUST WASM -->
												<div class="ascii-state-container">
													<h5 class="section-subheading">RAW RUST PRE-SORTER MEMORY STATE</h5>
													<pre class="ascii-state-box">{inspectorBufferState.stringState}</pre>
												</div>
											{:else}
												<div class="no-buffer-data">
													No raw sequence data available for this pallet output.
												</div>
											{/if}
										</div>

										<!-- FOOTER / NAVIGATION CONTROLS -->
										<footer class="inspector-footer">
											<div class="nav-controls">
												<Button
													kind="secondary"
													size="small"
													icon={ChevronLeft}
													disabled={inspectorStackIndex === 0}
													onclick={() => navigateInspector(-1)}
												>
													PREV OUTPUT
												</Button>

												<span class="nav-indicator">
													STACK {inspectorStackIndex + 1} OF {inspectorPallet.stacks.length}
												</span>

												<Button
													kind="secondary"
													size="small"
													icon={ChevronRight}
													disabled={inspectorStackIndex === inspectorPallet.stacks.length - 1}
													onclick={() => navigateInspector(1)}
												>
													NEXT OUTPUT
												</Button>
											</div>

											<Button kind="primary" size="small" onclick={closeBufferInspector}>
												CLOSE INSPECTOR
											</Button>
										</footer>
									{/if}
								</section>
							{/if}
						</article>
					{/each}
				{/if}
			</div>
		</section>
	{/if}
</div>

<style>
	.v2-page-layout {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
		width: 100%;
	}

	/* TOP STATUS BAR */
	.industrial-status-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.75rem 1.25rem;
		background: #0f0f0f;
		color: #ffffff;
		border: 1px solid #161616;
		font-size: 0.75rem;
	}

	.status-indicator {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-weight: 700;
		letter-spacing: 0.05em;
	}

	.pulse-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background-color: #da1e28;
		box-shadow: 0 0 8px #da1e28;
	}

	.pulse-dot.ready {
		background-color: #24a148;
		box-shadow: 0 0 8px #24a148;
	}

	.specs-pill {
		color: #a8a8a8;
		font-size: 0.7rem;
	}

	.specs-pill strong {
		color: #ffffff;
	}

	/* UPLOAD SECTION */
	.upload-hero-section {
		display: flex;
		justify-content: center;
		padding: 2rem 0;
	}

	.upload-card {
		width: 100%;
		max-width: 800px;
		background: #ffffff;
		border: 2px solid #0f0f0f;
		padding: 2.5rem;
		box-shadow: 8px 8px 0px #0f0f0f;
	}

	.card-header {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		margin-bottom: 0.75rem;
		border-bottom: 2px solid #0f0f0f;
		padding-bottom: 0.75rem;
	}

	.card-title {
		margin: 0;
		font-size: 1.2rem;
		font-weight: 800;
		letter-spacing: -0.01em;
	}

	.card-description {
		margin: 0 0 2rem 0;
		font-size: 0.85rem;
		color: #525252;
		line-height: 1.5;
	}

	.card-description code {
		background: #e0e0e0;
		padding: 0.15rem 0.4rem;
		font-weight: 700;
		color: #0f0f0f;
	}

	.file-drop-zone {
		position: relative;
		margin-bottom: 2rem;
	}

	.file-input-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		border: 0;
	}

	.drop-zone-label {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 3rem 2rem;
		border: 2px dashed #0f0f0f;
		background: #f4f4f4;
		cursor: pointer;
		transition: all 0.2s ease;
		text-align: center;
	}

	.drop-zone-label:hover:not(.disabled) {
		background: #e0e0e0;
		border-style: solid;
	}

	.drop-zone-label.disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.upload-icon-wrapper {
		margin-bottom: 1rem;
		color: #0f0f0f;
	}

	.zone-primary {
		font-weight: 800;
		font-size: 0.9rem;
		letter-spacing: 0.02em;
		color: #0f0f0f;
		margin-bottom: 0.4rem;
	}

	.zone-secondary {
		font-size: 0.75rem;
		color: #6f6f6f;
	}

	.zone-text {
		font-weight: 700;
		font-size: 0.85rem;
		letter-spacing: 0.05em;
	}

	.demo-trigger-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 1rem;
		border-top: 1px dashed #e0e0e0;
		padding-top: 1.5rem;
	}

	.divider-text {
		font-size: 0.7rem;
		font-weight: 700;
		color: #8d8d8d;
		letter-spacing: 0.1em;
	}

	/* DASHBOARD RESULTS */
	.dashboard-section {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.active-file-bar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 0.85rem 1.25rem;
		background: #f4f4f4;
		border: 1px solid #0f0f0f;
	}

	.file-info {
		display: flex;
		align-items: center;
		gap: 0.6rem;
		font-size: 0.85rem;
	}

	.file-name-label strong {
		color: #0f0f0f;
		text-transform: uppercase;
	}

	/* OVERALL STATS GRID */
	.stats-overview-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
		gap: 1rem;
	}

	.stat-card {
		background: #ffffff;
		border: 2px solid #0f0f0f;
		padding: 1.25rem;
		display: flex;
		flex-direction: column;
		gap: 0.4rem;
		position: relative;
		box-shadow: 4px 4px 0px #0f0f0f;
	}

	.stat-card.stat-primary {
		background: #0f0f0f;
		color: #ffffff;
	}

	.stat-card.stat-primary .stat-label,
	.stat-card.stat-primary .stat-sub {
		color: #a8a8a8;
	}

	.stat-card.warning-bg {
		border-color: #da1e28;
		background: #fff0f1;
	}

	.stat-label {
		font-size: 0.65rem;
		font-weight: 800;
		letter-spacing: 0.1em;
		color: #525252;
		text-transform: uppercase;
	}

	.stat-value {
		font-size: 2.2rem;
		font-weight: 900;
		line-height: 1;
		letter-spacing: -0.03em;
	}

	.stat-value.minmax {
		font-size: 1.8rem;
		display: flex;
		align-items: center;
		gap: 0.4rem;
	}

	.min-val {
		color: #da1e28;
	}

	.max-val {
		color: #198038;
	}

	.sep {
		color: #8d8d8d;
		font-weight: 400;
	}

	.stat-sub {
		font-size: 0.7rem;
		color: #6f6f6f;
		font-weight: 500;
	}

	/* FILTER TOOLBAR */
	.filter-toolbar {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 1rem;
		padding: 0.75rem 1rem;
		background: #ffffff;
		border: 1px solid #161616;
	}

	.search-box {
		flex: 1;
		min-width: 250px;
	}

	.filter-toggle-btn {
		background: #ffffff;
		border: 1px solid #0f0f0f;
		padding: 0.4rem 0.8rem;
		font-size: 0.75rem;
		font-weight: 700;
		cursor: pointer;
		transition: all 0.15s ease;
	}

	.filter-toggle-btn:hover {
		background: #f4f4f4;
	}

	.filter-toggle-btn.active {
		background: #da1e28;
		color: #ffffff;
		border-color: #da1e28;
	}

	/* PALLETS LIST */
	.pallets-container {
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.empty-filter-state {
		padding: 3rem;
		text-align: center;
		background: #f4f4f4;
		border: 1px dashed #0f0f0f;
		font-weight: 700;
		font-size: 0.85rem;
		color: #525252;
	}

	.pallet-card {
		background: #ffffff;
		border: 2px solid #0f0f0f;
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		box-shadow: 4px 4px 0px #0f0f0f;
	}

	.pallet-card.has-unsatisfactory {
		border-left: 6px solid #da1e28;
	}

	.pallet-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		flex-wrap: wrap;
		gap: 1rem;
		border-bottom: 1px solid #e0e0e0;
		padding-bottom: 0.85rem;
	}

	.pallet-title-group {
		display: flex;
		align-items: center;
		gap: 0.8rem;
	}

	.pallet-title {
		margin: 0;
		font-size: 1.1rem;
		font-weight: 900;
		letter-spacing: 0.02em;
	}

	.pallet-metrics {
		display: flex;
		gap: 1.5rem;
		flex-wrap: wrap;
	}

	.metric-item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.75rem;
	}

	.m-label {
		color: #6f6f6f;
		font-weight: 600;
	}

	.m-value {
		font-weight: 800;
		color: #0f0f0f;
	}

	.m-value.warn-text {
		color: #da1e28;
	}

	/* CHART SECTION & VERTICAL BAR CHART */
	.chart-section {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
		background: #fcfcfc;
		border: 1px solid #e0e0e0;
		padding: 1.25rem;
	}

	.chart-instruction {
		display: flex;
		align-items: center;
		gap: 0.5rem;
		font-size: 0.7rem;
		font-weight: 700;
		color: #525252;
		letter-spacing: 0.02em;
	}

	.vertical-chart-container {
		display: flex;
		height: 260px;
		gap: 0.75rem;
		margin-top: 0.5rem;
		position: relative;
	}

	/* Y-AXIS SCALE */
	.chart-y-axis {
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		align-items: flex-end;
		width: 24px;
		font-size: 0.65rem;
		font-weight: 800;
		color: #6f6f6f;
		padding-bottom: 24px; /* Align with x-axis labels */
	}

	.y-target {
		color: #da1e28;
	}

	/* PLOT AREA */
	.chart-plot-area {
		flex: 1;
		position: relative;
		border-left: 2px solid #0f0f0f;
		border-bottom: 2px solid #0f0f0f;
		background: #ffffff;
		max-width: 100%;
		overflow-x: auto;
		scrollbar-width: thin;
		scrollbar-color: #0f0f0f #f4f4f4;
		scrollbar-gutter: stable;
		padding-bottom: 24px; /* Space for x-axis labels */
	}

	/* OPTIMIZED CHART SCROLLBAR (webkit / chromium) */
	.chart-plot-area::-webkit-scrollbar {
		height: 10px;
	}

	.chart-plot-area::-webkit-scrollbar-track {
		background: #f4f4f4;
		border: 1px solid #e0e0e0;
		border-radius: 5px;
	}

	.chart-plot-area::-webkit-scrollbar-thumb {
		background: #0f0f0f;
		border-radius: 5px;
		border: 2px solid #f4f4f4;
	}

	.chart-plot-area::-webkit-scrollbar-thumb:hover {
		background: #393939;
	}

	.chart-plot-area::-webkit-scrollbar-thumb:active {
		background: #6f6f6f;
	}

	.chart-plot-area::-webkit-scrollbar-button {
		display: none;
	}

	.chart-plot-area::-webkit-scrollbar-corner {
		background: #f4f4f4;
	}

	/* THRESHOLD HORIZONTAL REFERENCE LINE (Y = 20) */
	.threshold-line {
		position: absolute;
		bottom: calc(66.666% + 8px);
		left: 0;
		right: 0;
		height: 2px;
		background: repeating-linear-gradient(
			90deg,
			#da1e28,
			#da1e28 6px,
			transparent 6px,
			transparent 12px
		);
		pointer-events: none;
		z-index: 5;
	}

	.threshold-tag {
		position: absolute;
		right: 8px;
		top: -18px;
		font-size: 0.6rem;
		font-weight: 800;
		color: #da1e28;
		background: #fff0f1;
		padding: 0.1rem 0.4rem;
		border: 1px solid #da1e28;
	}

	/* VERTICAL BARS GRID */
	.vertical-bars-grid {
		display: flex;
		align-items: flex-end;
		justify-content: space-around;
		height: 100%;
		width: max-content;
		min-width: 100%;
		padding: 0 0.5rem;
		gap: 0.5rem;
	}

	/* INDIVIDUAL VERTICAL COLUMN BUTTON */
	.vertical-bar-column {
		flex: 0 0 46px;
		width: 46px;
		height: 100%;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		align-items: center;
		background: transparent;
		border: none;
		padding: 0;
		cursor: pointer;
		position: relative;
		margin-bottom: 1em;
		transition: transform 0.15s ease;
	}

	.vertical-bar-column:hover {
		transform: translateY(-2px);
	}

	.vertical-bar-column.selected .vertical-bar-track {
		outline: 3px solid #0f0f0f;
		outline-offset: 2px;
	}

	.vertical-bar-column.selected.unsatisfactory .vertical-bar-track {
		outline-color: #da1e28;
	}

	/* NUMERIC VALUE LABEL ABOVE BAR */
	.bar-value-label {
		font-size: 0.75rem;
		font-weight: 900;
		margin-bottom: 4px;
		color: #0f0f0f;
	}

	.bar-value-label.alert-text {
		color: #da1e28;
	}

	/* BAR TRACK AND FILL */
	.vertical-bar-track {
		width: 100%;
		height: 100%;
		background: #f4f4f4;
		border: 1px solid #0f0f0f;
		display: flex;
		flex-direction: column;
		justify-content: flex-end;
		position: relative;
		overflow: hidden;
	}

	.vertical-bar-fill {
		width: 100%;
		background: #0f0f0f;
		position: relative;
		transition: height 0.3s cubic-bezier(0.16, 1, 0.3, 1);
	}

	.vertical-bar-fill.unsatisfactory-fill {
		background: #da1e28;
	}

	.bar-top-cap {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		height: 3px;
		background: #ffffff;
		opacity: 0.4;
	}

	/* X-AXIS LABEL BELOW BAR */
	.bar-x-label {
		position: absolute;
		bottom: -22px;
		font-size: 0.65rem;
		font-weight: 700;
		color: #6f6f6f;
	}

	.bar-warning-dot {
		position: absolute;
		top: 22px;
		font-size: 0.7rem;
		font-weight: 900;
		color: #ffffff;
		background: #da1e28;
		width: 14px;
		height: 14px;
		border-radius: 50%;
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 6;
	}

	/* BUFFER INSPECTOR CARD (inline, collapsible, under bar chart) */
	.buffer-inspector-card {
		background: #ffffff;
		border: 2px solid #0f0f0f;
		box-shadow: 4px 4px 0px #0f0f0f;
		display: flex;
		flex-direction: column;
		overflow: hidden;
	}

	.inspector-card-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		gap: 1rem;
		padding: 1rem 1.25rem;
		background: #0f0f0f;
		color: #ffffff;
		border-bottom: 2px solid #0f0f0f;
	}

	.buffer-inspector-card.collapsed .inspector-card-header {
		border-bottom: none;
	}

	.inspector-title-group {
		display: flex;
		align-items: center;
		gap: 0.8rem;
		flex-wrap: wrap;
	}

	.inspector-card-actions {
		display: flex;
		align-items: center;
		gap: 0.5rem;
	}

	.inspector-badge {
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: #a8a8a8;
	}

	.inspector-main-title {
		margin: 0;
		font-size: 1.05rem;
		font-weight: 900;
		letter-spacing: 0.02em;
	}

	.inspector-collapsed-summary {
		font-size: 0.7rem;
		font-weight: 800;
		letter-spacing: 0.05em;
		color: #ffffff;
		background: #262626;
		border: 1px solid #525252;
		padding: 0.2rem 0.5rem;
	}

	.inspector-collapsed-summary.alert-text {
		color: #ffb3b8;
		border-color: #da1e28;
		background: rgba(218, 30, 40, 0.2);
	}

	.collapse-inspector-btn,
	.close-inspector-btn {
		background: transparent;
		border: 1px solid #525252;
		color: #ffffff;
		cursor: pointer;
		padding: 0.35rem;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.15s ease;
	}

	.collapse-inspector-btn:hover,
	.close-inspector-btn:hover {
		background: #262626;
		border-color: #ffffff;
	}

	.inspector-body {
		padding: 1.5rem;
		display: flex;
		flex-direction: column;
		gap: 1.5rem;
	}

	.inspector-stack-summary {
		display: flex;
		gap: 1.5rem;
		flex-wrap: wrap;
		padding: 0.85rem 1rem;
		background: #f4f4f4;
		border: 1px solid #0f0f0f;
	}

	.summary-item {
		display: flex;
		align-items: center;
		gap: 0.4rem;
		font-size: 0.8rem;
	}

	.s-label {
		color: #525252;
		font-weight: 700;
	}

	.s-value {
		font-weight: 900;
		color: #0f0f0f;
	}

	.section-subheading {
		margin: 0 0 0.75rem 0;
		font-size: 0.75rem;
		font-weight: 800;
		letter-spacing: 0.08em;
		color: #0f0f0f;
		text-transform: uppercase;
	}

	/* BUFFER SLOTS GRID */
	.buffer-slots-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 0.85rem;
	}

	.buffer-slot-card {
		background: #ffffff;
		border: 1px solid #0f0f0f;
		padding: 0.85rem;
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
	}

	.buffer-slot-card.has-items {
		border-width: 2px;
		background: #fcfcfc;
	}

	.slot-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		font-size: 0.75rem;
		font-weight: 800;
	}

	.slot-count {
		color: #525252;
	}

	.slot-gauge-track {
		height: 12px;
		background: #e0e0e0;
		border: 1px solid #0f0f0f;
		position: relative;
	}

	.slot-gauge-fill {
		height: 100%;
		background: #0f0f0f;
		transition: width 0.3s ease;
	}

	.slot-status {
		font-size: 0.65rem;
		font-weight: 700;
		color: #6f6f6f;
	}

	/* ASCII STATE */
	.ascii-state-box {
		margin: 0;
		padding: 1rem;
		background: #0f0f0f;
		color: #00ff66;
		font-family: 'JetBrains Mono', monospace;
		font-size: 0.85rem;
		border: 1px solid #0f0f0f;
		white-space: pre-wrap;
		line-height: 1.4;
	}

	.no-buffer-data {
		padding: 2rem;
		text-align: center;
		background: #f4f4f4;
		font-weight: 700;
		color: #6f6f6f;
	}

	.inspector-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 1.5rem;
		background: #f4f4f4;
		border-top: 1px solid #0f0f0f;
	}

	.nav-controls {
		display: flex;
		align-items: center;
		gap: 1rem;
	}

	.nav-indicator {
		font-size: 0.75rem;
		font-weight: 800;
		color: #0f0f0f;
	}
</style>
