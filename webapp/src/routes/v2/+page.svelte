<script lang="ts">
	import { onMount } from 'svelte';
	import { Button, Tag, InlineNotification, Loading, TextInput } from 'carbon-components-svelte';
	import VerticalBarChart from '$lib/components/VerticalBarChart.svelte';
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
	import { m } from '$lib/paraglide/messages';

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
			errorMessage = m.v2_wasm_init_error({ message: err?.message || err });
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
			errorMessage = m.v2_process_error({
				file: file.name,
				message: err?.message || m.v2_invalid_format_or_missing_sheet()
			});
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
		activeFileName = m.v2_demo_filename();

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
				errorMessage = m.v2_demo_error({ message: err?.message || err });
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
				{wasmReady ? m.v2_engine_online() : m.v2_initializing_wasm()}
			</span>
		</div>
		<div class="specs-pill">
			{m.v2_max_stack_height()}
		</div>
	</section>

	{#if errorMessage}
		<InlineNotification
			kind="error"
			title={m.v2_system_error()}
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
					<h2 class="card-title">{m.v2_upload_title()}</h2>
				</div>
				<p class="card-description">
					{m.v2_upload_description()}
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
							<span class="zone-text">{m.v2_calculating_stacks()}</span>
						{:else}
							<div class="upload-icon-wrapper">
								<Upload size={32} />
							</div>
							<span class="zone-primary">{m.v2_drag_drop()}</span>
							<span class="zone-secondary">{m.v2_supports_files()}</span>
						{/if}
					</label>
				</div>

				<div class="demo-trigger-container">
					<span class="divider-text">{m.v2_or_test_demo()}</span>
					<Button
						kind="tertiary"
						size="small"
						icon={Analytics}
						disabled={!wasmReady || isProcessing}
						onclick={runDemoSequence}
					>
						{m.v2_load_demo()}
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
					<span class="file-name-label">{m.v2_source()} <strong>{activeFileName}</strong></span>
				</div>
				<div class="file-actions">
					<Button kind="danger-tertiary" size="small" icon={Reset} onclick={resetData}>
						{m.v2_upload_new_file()}
					</Button>
				</div>
			</div>

			<!-- OVERALL STATISTICS DASHBOARD (SUM OF ALL PALLETS) -->
			<div class="stats-overview-grid">
				<div class="stat-card stat-primary">
					<div class="stat-label">{m.v2_total_pallets()}</div>
					<div class="stat-value">{overallStats.totalPallets}</div>
					<div class="stat-sub">{m.v2_pallet_output_batches()}</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.v2_total_stacks()}</div>
					<div class="stat-value">{overallStats.totalStacks}</div>
					<div class="stat-sub">{m.v2_magazine_stacks_output()}</div>
				</div>

				<div class="stat-card" class:warning-bg={overallStats.satisfactoryPercentage < 75}>
					<div class="stat-label">{m.v2_satisfactory_stacks()}</div>
					<div class="stat-value">
						{overallStats.satisfactoryPercentage}%
					</div>
					<div class="stat-sub">
						{m.v2_height_ge({
							count: overallStats.satisfactoryCount,
							total: overallStats.totalStacks
						})}
					</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.v2_min_max_stack_size()}</div>
					<div class="stat-value minmax">
						<span class="min-val">{overallStats.minStackSize}</span>
						<span class="sep">/</span>
						<span class="max-val">{overallStats.maxStackSize}</span>
					</div>
					<div class="stat-sub">{m.v2_global_height_boundaries()}</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.v2_avg_stack_height()}</div>
					<div class="stat-value">{overallStats.avgStackSize}</div>
					<div class="stat-sub">{m.v2_magazines_per_stack()}</div>
				</div>
			</div>

			<!-- FILTER & CONTROLS -->
			<div class="filter-toolbar">
				<div class="search-box">
					<TextInput
						labelText=""
						placeholder={m.v2_filter_placeholder()}
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
						{showOnlyUnsatisfactory ? '[x]' : '[ ]'}
						{m.v2_show_unsatisfactory()}
					</button>
				</div>
			</div>

			<!-- PALLET LIST DISPLAY -->
			<div class="pallets-container">
				{#if filteredPallets.length === 0}
					<div class="empty-filter-state">{m.v2_no_pallets_match()}</div>
				{:else}
					{#each filteredPallets as pallet (pallet.id)}
						<article class="pallet-card" class:has-unsatisfactory={pallet.unsatisfactoryCount > 0}>
							<!-- PALLET HEADER & METRICS -->
							<header class="pallet-header">
								<div class="pallet-title-group">
									<h3 class="pallet-title">
										{m.v2_pallet_title({ id: pallet.id.toString().padStart(2, '0') })}
									</h3>
									{#if pallet.unsatisfactoryCount > 0}
										<Tag type="red" size="sm" icon={WarningAltFilled}>
											{pallet.unsatisfactoryCount > 1
												? m.v2_unsatisfactory_stacks_many({
														count: pallet.unsatisfactoryCount
													})
												: m.v2_unsatisfactory_stacks_one({
														count: pallet.unsatisfactoryCount
													})}
										</Tag>
									{:else}
										<Tag type="green" size="sm" icon={CheckmarkFilled}
											>{m.v2_fully_satisfactory()}</Tag
										>
									{/if}
								</div>

								<div class="pallet-metrics">
									<div class="metric-item">
										<span class="m-label">{m.v2_stacks_metric()}</span>
										<span class="m-value">{pallet.stacks.length}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.v2_satisfactory_metric()}</span>
										<span class="m-value" class:warn-text={pallet.satisfactoryPercentage < 100}>
											{pallet.satisfactoryPercentage}%
										</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.v2_minmax_metric()}</span>
										<span class="m-value">{pallet.minStackSize} - {pallet.maxStackSize}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.v2_avg_metric()}</span>
										<span class="m-value">{pallet.avgStackSize}</span>
									</div>
								</div>
							</header>

							<!-- PALLET STACKS VISUALIZATION: VERTICAL BAR CHART -->
							<div class="chart-section">
								<div class="chart-instruction">
									<Information size={14} />
									<span>{m.v2_chart_instruction()}</span>
								</div>

								<VerticalBarChart
									stacks={pallet.stacks}
									palletId={pallet.id}
									{inspectorOpen}
									inspectorPalletId={inspectorPallet?.id ?? null}
									{inspectorStackIndex}
									onselect={(index) => openBufferInspector(pallet, index)}
								/>
							</div>

							<!-- COLLAPSIBLE BUFFER INSPECTOR CARD (inline under bar chart) -->
							{#if inspectorOpen && inspectorPallet && inspectorPallet.id === pallet.id}
								<section class="buffer-inspector-card" class:collapsed={inspectorCollapsed}>
									<header class="inspector-card-header">
										<div class="inspector-title-group">
											<span class="inspector-badge">{m.v2_presorter_diagnostics()}</span>
											<h4 class="inspector-main-title">
												{m.v2_inspector_title({
													id: inspectorPallet.id.toString().padStart(2, '0'),
													output: inspectorStackIndex + 1
												})}
											</h4>
											{#if inspectorCollapsed}
												<span
													class="inspector-collapsed-summary"
													class:alert-text={inspectorPallet.stacks[inspectorStackIndex] < 20}
													>{m.v2_height()} {inspectorPallet.stacks[inspectorStackIndex]}</span
												>
											{/if}
										</div>
										<div class="inspector-card-actions">
											<button
												class="collapse-inspector-btn"
												onclick={toggleInspectorCollapsed}
												aria-label={inspectorCollapsed
													? m.v2_expand_inspector()
													: m.v2_collapse_inspector()}
												title={inspectorCollapsed
													? m.v2_expand_inspector()
													: m.v2_collapse_inspector()}
											>
												{#if inspectorCollapsed}<ChevronDown size={20} />{:else}<ChevronUp
														size={20}
													/>{/if}
											</button>
											<button
												class="close-inspector-btn"
												onclick={closeBufferInspector}
												aria-label={m.v2_close_inspector()}
												title={m.v2_close_inspector()}
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
													<span class="s-label">{m.v2_output_height()}</span>
													<span
														class="s-value"
														class:alert-text={inspectorPallet.stacks[inspectorStackIndex] < 20}
													>
														{inspectorPallet.stacks[inspectorStackIndex]}
														{m.v2_magazines()}
													</span>
												</div>
												<div class="summary-item">
													<span class="s-label">{m.v2_status()}</span>
													{#if inspectorPallet.stacks[inspectorStackIndex] < 20}
														<Tag type="red" size="sm">{m.v2_unsatisfactory_status()}</Tag>
													{:else}
														<Tag type="green" size="sm">{m.v2_satisfactory_status()}</Tag>
													{/if}
												</div>
												{#if inspectorBufferState}
													<div class="summary-item">
														<span class="s-label">{m.v2_items_processed()}</span>
														<span class="s-value"
															>{m.v2_items_from_sequence({
																count: inspectorBufferState.itemsProcessed
															})}</span
														>
													</div>
												{/if}
											</div>

											{#if inspectorBufferState}
												<!-- VISUAL BUFFER SLOTS GAUGE (BUFFERS 1 TO 4) -->
												<div class="buffer-slots-container">
													<h5 class="section-subheading">{m.v2_buffer_positions_state()}</h5>
													<div class="buffer-slots-grid">
														{#each inspectorBufferState.buffers as bufHeight, bufIdx}
															{@const fillPercent = (bufHeight / 30) * 100}
															<div class="buffer-slot-card" class:has-items={bufHeight > 0}>
																<div class="slot-header">
																	<span class="slot-title"
																		>{m.v2_buffer_slot({ number: bufIdx + 1 })}</span
																	>
																	<span class="slot-count">{bufHeight} / 30</span>
																</div>

																<div class="slot-gauge-track">
																	<div class="slot-gauge-fill" style="width: {fillPercent}%;"></div>
																</div>

																<div class="slot-status">
																	{bufHeight === 0
																		? m.v2_empty_available()
																		: m.v2_magazines_stacked({ count: bufHeight })}
																</div>
															</div>
														{/each}
													</div>
												</div>

												<!-- ASCII STRING STATE FROM RUST WASM -->
												<div class="ascii-state-container">
													<h5 class="section-subheading">{m.v2_raw_rust_state()}</h5>
													<pre class="ascii-state-box">{inspectorBufferState.stringState}</pre>
												</div>
											{:else}
												<div class="no-buffer-data">
													{m.v2_no_buffer_data()}
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
													{m.v2_prev_output()}
												</Button>

												<span class="nav-indicator">
													{m.v2_stack_of({
														current: inspectorStackIndex + 1,
														total: inspectorPallet.stacks.length
													})}
												</span>

												<Button
													kind="secondary"
													size="small"
													icon={ChevronRight}
													disabled={inspectorStackIndex === inspectorPallet.stacks.length - 1}
													onclick={() => navigateInspector(1)}
												>
													{m.v2_next_output()}
												</Button>
											</div>

											<Button kind="primary" size="small" onclick={closeBufferInspector}>
												{m.v2_close_inspector_btn()}
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
