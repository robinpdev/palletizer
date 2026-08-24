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
	import { PersistentState } from '@friendofsvelte/state';

	import {
		processExcelFile,
		processSequences,
		calculateOverallStats,
		getBufferStateForOutput,
		arraysum
	} from '$lib/palletizer';
	import type { PalletData, BufferState } from '$lib/types';
	import init, { runseq, SortStrategy } from 'rust';
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

	let nbuffers: number = $state(4);
	let maxheight = new PersistentState('maxheight', 30, 'localStorage');
	let targetheight: number = $state(25);
	let minheight = new PersistentState('minheight', 20, 'localStorage');
	let strategy: SortStrategy = $state(SortStrategy.FirstFitStrategy);

	let overallStats = $derived(calculateOverallStats(pallets, minheight.current));

	/*

	nbuffers: u32,
        maxheight: u32,
        targetheight: u32,
        minheight: u32,
        strategy: SortStrategy,
	*/

	function clampHeightValue(value: number) {
		return Math.max(1, Math.min(100, value));
	}

	function updateHeightValue(kind: 'min' | 'max', delta: number) {
		if (kind === 'min') {
			const nextValue = clampHeightValue(minheight.current + delta);
			minheight.current = Math.min(nextValue, maxheight.current);
			if (maxheight < minheight) {
				maxheight = minheight;
			}
		} else {
			const nextValue = clampHeightValue(maxheight.current + delta);
			maxheight.current = Math.max(nextValue, minheight.current);
			if (minheight > maxheight) {
				minheight = maxheight;
			}
		}
	}

	function handleHeightInput(kind: 'min' | 'max', rawValue: string) {
		if (rawValue === '') return;

		const parsedValue = Number.parseInt(rawValue, 10);
		if (!Number.isFinite(parsedValue)) return;

		const clampedValue = clampHeightValue(parsedValue);
		if (kind === 'min') {
			minheight.current = Math.min(clampedValue, maxheight.current);
			if (maxheight.current < minheight.current) {
				maxheight.current = minheight.current;
			}
		} else {
			maxheight.current = Math.max(clampedValue, minheight.current);
			if (minheight.current > maxheight.current) {
				minheight.current = maxheight.current;
			}
		}
	}

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
			await init('/assets/rust_bg.wasm');
			wasmReady = true;
		} catch (err: any) {
			errorMessage = m.wasm_init_error({ message: err?.message || err });
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
			pallets = await processExcelFile(
				file,
				nbuffers,
				maxheight.current,
				targetheight,
				minheight.current,
				strategy
			);
		} catch (err: any) {
			console.error(err);
			errorMessage = m.process_error({
				file: file.name,
				message: err?.message || m.invalid_format_or_missing_sheet()
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
		activeFileName = m.demo_filename();

		setTimeout(async () => {
			try {
				const demoSeqs = [
					[12, 15, 18, 22, 25, 19, 28, 14, 21, 30, 24, 18, 26, 16],
					[22, 28, 20, 29, 18, 25, 27, 21, 23, 16, 19, 24],
					[15, 14, 19, 28, 29, 24, 22, 21, 25, 17, 26, 30, 15, 22],
					[20, 21, 24, 28, 29, 23, 22, 25, 19, 15, 18, 27]
				];

				pallets = await processSequences(demoSeqs, nbuffers, maxheight.current, targetheight, minheight.current, strategy);
			} catch (err: any) {
				errorMessage = m.demo_error({ message: err?.message || err });
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
			inspectorBufferState = getBufferStateForOutput(
				pallet.rawSequence,
				stackIndex,
				nbuffers,
				maxheight.current,
				targetheight,
				minheight.current,
				strategy,
				pallet.steps
			);
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
				{wasmReady ? m.engine_online() : m.initializing_wasm()}
			</span>
		</div>
		<div class="specs-pill">
			Min / Max: {minheight.current} / {maxheight.current}
		</div>
	</section>

	{#if errorMessage}
		<InlineNotification
			kind="error"
			title={m.system_error()}
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
					<h2 class="card-title">{m.upload_title()}</h2>
				</div>
				<p class="card-description">
					{m.upload_description()}
				</p>

				<div class="height-range-controls">
					<div class="height-control">
						<label class="height-control-label" for="min-height">Min stapel hoogte</label>
						<div class="height-input-group">
							<input
								id="min-height"
								class="height-input"
								type="number"
								inputmode="numeric"
								min="1"
								max="100"
								value={minheight.current}
								oninput={(event) =>
									handleHeightInput('min', (event.currentTarget as HTMLInputElement).value)}
							/>
							<div class="height-stepper">
								<button
									type="button"
									class="height-step-btn"
									aria-label="Increase minimum height"
									onclick={() => updateHeightValue('min', 1)}
								>
									▲
								</button>
								<button
									type="button"
									class="height-step-btn"
									aria-label="Decrease minimum height"
									onclick={() => updateHeightValue('min', -1)}
								>
									▼
								</button>
							</div>
						</div>
					</div>

					<div class="height-control">
						<label class="height-control-label" for="max-height">Max stapel hoogte</label>
						<div class="height-input-group">
							<input
								id="max-height"
								class="height-input"
								type="number"
								inputmode="numeric"
								min="1"
								max="100"
								value={maxheight.current}
								oninput={(event) =>
									handleHeightInput('max', (event.currentTarget as HTMLInputElement).value)}
							/>
							<div class="height-stepper">
								<button
									type="button"
									class="height-step-btn"
									aria-label="Increase maximum height"
									onclick={() => updateHeightValue('max', 1)}
								>
									▲
								</button>
								<button
									type="button"
									class="height-step-btn"
									aria-label="Decrease maximum height"
									onclick={() => updateHeightValue('max', -1)}
								>
									▼
								</button>
							</div>
						</div>
					</div>
				</div>

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
							<span class="zone-text">{m.calculating_stacks()}</span>
						{:else}
							<div class="upload-icon-wrapper">
								<Upload size={32} />
							</div>
							<span class="zone-primary">{m.drag_drop()}</span>
							<span class="zone-secondary">{m.supports_files()}</span>
						{/if}
					</label>
				</div>

				<div class="demo-trigger-container">
					<span class="divider-text">{m.or_test_demo()}</span>
					<Button
						kind="tertiary"
						size="small"
						icon={Analytics}
						disabled={!wasmReady || isProcessing}
						onclick={runDemoSequence}
					>
						{m.load_demo()}
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
					<span class="file-name-label">{m.source()} <strong>{activeFileName}</strong></span>
				</div>
				<div class="file-actions">
					<Button kind="danger-tertiary" size="small" icon={Reset} onclick={resetData}>
						{m.upload_new_file()}
					</Button>
				</div>
			</div>

			<!-- OVERALL STATISTICS DASHBOARD (SUM OF ALL PALLETS) -->
			<div class="stats-overview-grid">
				<div class="stat-card stat-primary">
					<div class="stat-label">{m.total_pallets()}</div>
					<div class="stat-value">{overallStats.totalPallets}</div>
					<div class="stat-sub">{m.pallet_output_batches()}</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.total_stacks()}</div>
					<div class="stat-value">{overallStats.totalStacks}</div>
					<div class="stat-sub">{m.magazine_stacks_output()}</div>
				</div>

				<div class="stat-card" class:warning-bg={overallStats.satisfactoryPercentage < 75}>
					<div class="stat-label">{m.satisfactory_stacks()}</div>
					<div class="stat-value">
						{overallStats.satisfactoryPercentage}%
					</div>
					<div class="stat-sub">
						{m.height_ge({
							count: overallStats.satisfactoryCount,
							total: overallStats.totalStacks
						})}
					</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.min_max_stack_size()}</div>
					<div class="stat-value minmax">
						<span class="min-val">{overallStats.minStackSize}</span>
						<span class="sep">/</span>
						<span class="max-val">{overallStats.maxStackSize}</span>
					</div>
					<div class="stat-sub">{m.global_height_boundaries()}</div>
				</div>

				<div class="stat-card">
					<div class="stat-label">{m.avg_stack_height()}</div>
					<div class="stat-value">{overallStats.avgStackSize}</div>
					<div class="stat-sub">{m.magazines_per_stack()}</div>
				</div>
			</div>

			<!-- FILTER & CONTROLS -->
			<div class="filter-toolbar">
				<div class="search-box">
					<TextInput
						labelText=""
						placeholder={m.filter_placeholder()}
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
						{m.show_unsatisfactory()}
					</button>
				</div>
			</div>

			<!-- PALLET LIST DISPLAY -->
			<div class="pallets-container">
				{#if filteredPallets.length === 0}
					<div class="empty-filter-state">{m.no_pallets_match()}</div>
				{:else}
					{#each filteredPallets as pallet (pallet.id)}
						<article class="pallet-card" class:has-unsatisfactory={pallet.unsatisfactoryCount > 0}>
							<!-- PALLET HEADER & METRICS -->
							<header class="pallet-header">
								<div class="pallet-title-group">
									<h3 class="pallet-title">
										{m.pallet_title({ id: pallet.id.toString().padStart(2, '0') })}
									</h3>
									{#if pallet.unsatisfactoryCount > 0}
										<Tag type="red" size="sm" icon={WarningAltFilled}>
											{pallet.unsatisfactoryCount > 1
												? m.unsatisfactory_stacks_many({
														count: pallet.unsatisfactoryCount
													})
												: m.unsatisfactory_stacks_one({
														count: pallet.unsatisfactoryCount
													})}
										</Tag>
									{:else}
										<Tag type="green" size="sm" icon={CheckmarkFilled}>{m.fully_satisfactory()}</Tag
										>
									{/if}
								</div>

								<div class="pallet-metrics">
									<div class="metric-item">
										<span class="m-label">{m.stacks_metric()}</span>
										<span class="m-value">{pallet.stacks.length}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.satisfactory_metric()}</span>
										<span class="m-value" class:warn-text={pallet.satisfactoryPercentage < 100}>
											{pallet.satisfactoryPercentage}%
										</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.minmax_metric()}</span>
										<span class="m-value">{pallet.minStackSize} - {pallet.maxStackSize}</span>
									</div>
									<div class="metric-item">
										<span class="m-label">{m.avg_metric()}</span>
										<span class="m-value">{pallet.avgStackSize}</span>
									</div>
								</div>
							</header>

							<!-- PALLET STACKS VISUALIZATION: VERTICAL BAR CHART -->
							<div class="chart-section">
								<div class="chart-instruction">
									<Information size={14} />
									<span>{m.chart_instruction()}</span>
								</div>

								<VerticalBarChart
									stacks={pallet.stacks}
									palletId={pallet.id}
									{inspectorOpen}
									inspectorPalletId={inspectorPallet?.id ?? null}
									{inspectorStackIndex}
									onselect={(index) => openBufferInspector(pallet, index)}
									minheight={minheight.current}
									maxheight={maxheight.current}
								/>
							</div>

							<!-- COLLAPSIBLE BUFFER INSPECTOR CARD (inline under bar chart) -->
							{#if inspectorOpen && inspectorPallet && inspectorPallet.id === pallet.id}
								<section class="buffer-inspector-card" class:collapsed={inspectorCollapsed}>
									<header class="inspector-card-header">
										<div class="inspector-title-group">
											<span class="inspector-badge">{m.presorter_diagnostics()}</span>
											<h4 class="inspector-main-title">
												{m.inspector_title({
													id: inspectorPallet.id.toString().padStart(2, '0'),
													output: inspectorStackIndex + 1
												})}
											</h4>
											{#if inspectorCollapsed}
												<span
													class="inspector-collapsed-summary"
													class:alert-text={arraysum(inspectorPallet.stacks[inspectorStackIndex]) <
														minheight.current}
													>{m.height()}
													{arraysum(inspectorPallet.stacks[inspectorStackIndex])}</span
												>
											{/if}
										</div>
										<div class="inspector-card-actions">
											<button
												class="collapse-inspector-btn"
												onclick={toggleInspectorCollapsed}
												aria-label={inspectorCollapsed
													? m.expand_inspector()
													: m.collapse_inspector()}
												title={inspectorCollapsed ? m.expand_inspector() : m.collapse_inspector()}
											>
												{#if inspectorCollapsed}<ChevronDown size={20} />{:else}<ChevronUp
														size={20}
													/>{/if}
											</button>
											<button
												class="close-inspector-btn"
												onclick={closeBufferInspector}
												aria-label={m.close_inspector()}
												title={m.close_inspector()}
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
													<span class="s-label">{m.output_height()}</span>
													<span
														class="s-value"
														class:alert-text={arraysum(
															inspectorPallet.stacks[inspectorStackIndex]
														) < minheight.current}
													>
														{arraysum(inspectorPallet.stacks[inspectorStackIndex])}
														{m.magazines()}
													</span>
												</div>
												<div class="summary-item">
													<span class="s-label">{m.status()}</span>
													{#if arraysum(inspectorPallet.stacks[inspectorStackIndex]) < minheight.current}
														<Tag type="red" size="sm">{m.unsatisfactory()} ({m.toolow()})</Tag>
													{:else if arraysum(inspectorPallet.stacks[inspectorStackIndex]) > maxheight.current}
													    <Tag type="red" size="sm">{m.unsatisfactory()} ({m.toohigh()})</Tag>
													{:else}
														<Tag type="green" size="sm">{m.satisfactory()}</Tag>
													{/if}
												</div>
												{#if inspectorBufferState}
													<div class="summary-item">
														<span class="s-label">{m.items_processed()}</span>
														<span class="s-value"
															>{m.items_from_sequence({
																count: inspectorBufferState.itemsProcessed
															})}</span
														>
													</div>
												{/if}
											</div>

											{#if inspectorBufferState}
												<!-- VISUAL BUFFER SLOTS GAUGE (BUFFERS 1 TO 4) -->
												<div class="buffer-slots-container">
													<h5 class="section-subheading">{m.buffer_positions_state()}</h5>
													<div class="buffer-slots-grid">
														{#each inspectorBufferState.buffers as bundles, bufIdx (bufIdx)}
															{@const bufHeight = arraysum(bundles)}
															<div class="buffer-slot-card" class:has-items={true}>
																<div class="slot-header">
																	<span class="slot-title"
																		>{m.buffer_slot({ number: bufIdx + 1 })}</span
																	>
																	<span class="slot-count">{bufHeight} / {maxheight.current}</span>
																</div>
																<div class="slot-gauge-track">
																	{#each bundles as bufHeight, bundleIdx (bundleIdx)}
																		{@const fillPercent = (bufHeight / maxheight.current) * 100}
																		<div
																			class="slot-gauge-fill"
																			style="width: {fillPercent}%;"
																		></div>
																	{/each}
																</div>
																<div class="slot-status">
																	{bufHeight === 0
																		? m.empty_available()
																		: m.magazines_stacked({ count: bufHeight })}
																</div>
															</div>
														{/each}
													</div>
												</div>

												<!-- ASCII STRING STATE FROM RUST WASM -->
												<!-- <div class="ascii-state-container">
													<h5 class="section-subheading">{m.raw_rust_state()}</h5>
													<pre class="ascii-state-box">{inspectorBufferState.stringState}</pre>
												</div> -->
											{:else}
												<div class="no-buffer-data">
													{m.no_buffer_data()}
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
													{m.prev_output()}
												</Button>

												<span class="nav-indicator">
													{m.stack_of({
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
													{m.next_output()}
												</Button>
											</div>

											<Button kind="primary" size="small" onclick={closeBufferInspector}>
												{m.close_inspector_btn()}
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

	.height-range-controls {
		display: flex;
		gap: 1rem;
		flex-wrap: wrap;
		margin-bottom: 2rem;
	}

	.height-control {
		display: flex;
		flex-direction: column;
		gap: 0.45rem;
		flex: 1;
		min-width: 220px;
	}

	.height-control-label {
		font-size: 0.72rem;
		font-weight: 800;
		letter-spacing: 0.08em;
		color: #525252;
		text-transform: uppercase;
	}

	.height-input-group {
		display: flex;
		align-items: stretch;
		border: 2px solid #0f0f0f;
		background: #ffffff;
	}

	.height-input {
		flex: 1;
		border: none;
		padding: 0.8rem 0.9rem;
		font-size: 0.95rem;
		font-weight: 700;
		color: #0f0f0f;
		min-width: 0;
	}

	.height-input:focus {
		outline: none;
	}

	.height-stepper {
		display: flex;
		flex-direction: column;
		border-left: 1px solid #0f0f0f;
	}

	.height-step-btn {
		background: #f4f4f4;
		border: none;
		padding: 0.35rem 0.55rem;
		cursor: pointer;
		font-size: 0.7rem;
		font-weight: 800;
		color: #0f0f0f;
		line-height: 1;
	}

	.height-step-btn:hover {
		background: #e0e0e0;
	}

	.height-step-btn:first-child {
		border-bottom: 1px solid #0f0f0f;
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
		display: flex;
		flex-direction: row;
		background: #e0e0e0;
		border: 1px solid #0f0f0f;
		position: relative;
	}

	.slot-gauge-fill {
		height: 100%;
		background: #0f0f0f;
		border-right: 2px solid white;

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
