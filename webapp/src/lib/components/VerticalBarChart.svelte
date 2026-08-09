<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	interface Props {
		stacks: number[];
		palletId: number;
		inspectorOpen: boolean;
		inspectorPalletId: number | null;
		inspectorStackIndex: number;
		onselect?: (index: number) => void;
	}

	let { stacks, palletId, inspectorOpen, inspectorPalletId, inspectorStackIndex, onselect }: Props =
		$props();

	let plotAreaEl: HTMLDivElement;
	let trackEl: HTMLDivElement;
	let thumbEl: HTMLDivElement;

	let scrollLeft = $state(0);
	let clientWidth = $state(0);
	let scrollWidth = $state(0);
	let dragging = $state(false);

	let startClientX = 0;
	let startScrollLeft = 0;

	let hasOverflow = $derived(scrollWidth > clientWidth);
	let thumbWidthPct = $derived(hasOverflow ? Math.max(12, (clientWidth / scrollWidth) * 100) : 100);
	let thumbOffsetPct = $derived(
		hasOverflow ? (scrollLeft / (scrollWidth - clientWidth)) * (100 - thumbWidthPct) : 0
	);
	let maxScroll = $derived(Math.max(0, scrollWidth - clientWidth));

	$effect(() => {
		const el = plotAreaEl;
		if (!el) return;
		// re-measure whenever the stack set changes
		void stacks;
		const measure = () => {
			clientWidth = el.clientWidth;
			scrollWidth = el.scrollWidth;
		};
		measure();
		const ro = new ResizeObserver(measure);
		ro.observe(el);
		return () => ro.disconnect();
	});

	function handleScroll() {
		if (plotAreaEl) scrollLeft = plotAreaEl.scrollLeft;
	}

	function onTrackPointerDown(e: PointerEvent) {
		if (e.target === thumbEl || !plotAreaEl) return;
		const trackRect = trackEl.getBoundingClientRect();
		const ratio = (e.clientX - trackRect.left) / trackRect.width;
		const maxScroll = scrollWidth - clientWidth;
		plotAreaEl.scrollLeft = ratio * maxScroll;
	}

	function onThumbPointerDown(e: PointerEvent) {
		if (!plotAreaEl || !thumbEl) return;
		e.preventDefault();
		dragging = true;
		startClientX = e.clientX;
		startScrollLeft = plotAreaEl.scrollLeft;
		thumbEl.setPointerCapture(e.pointerId);
	}

	function onThumbPointerMove(e: PointerEvent) {
		if (!dragging || !plotAreaEl || !trackEl) return;
		const trackRect = trackEl.getBoundingClientRect();
		const thumbWidthPx = trackRect.width * (thumbWidthPct / 100);
		const maxTravel = trackRect.width - thumbWidthPx;
		const maxScroll = scrollWidth - clientWidth;
		const dRatio = maxTravel > 0 ? (e.clientX - startClientX) / maxTravel : 0;
		plotAreaEl.scrollLeft = startScrollLeft + dRatio * maxScroll;
	}

	function onThumbPointerUp(e: PointerEvent) {
		if (!dragging) return;
		dragging = false;
		if (thumbEl?.hasPointerCapture(e.pointerId)) {
			thumbEl.releasePointerCapture(e.pointerId);
		}
	}

	function onThumbKeydown(e: KeyboardEvent) {
		if (!plotAreaEl) return;
		const step = Math.max(46, Math.round(clientWidth * 0.2));
		if (e.key === 'ArrowLeft') {
			e.preventDefault();
			plotAreaEl.scrollLeft -= step;
		} else if (e.key === 'ArrowRight') {
			e.preventDefault();
			plotAreaEl.scrollLeft += step;
		} else if (e.key === 'Home') {
			e.preventDefault();
			plotAreaEl.scrollLeft = 0;
		} else if (e.key === 'End') {
			e.preventDefault();
			plotAreaEl.scrollLeft = plotAreaEl.scrollWidth;
		}
	}
</script>

<div class="vertical-chart-container">
	<!-- Y-AXIS SCALE & REFERENCE LINES -->
	<div class="chart-y-axis">
		<span class="y-label y-max">30</span>
		<span class="y-label y-target">20</span>
		<span class="y-label y-mid">10</span>
		<span class="y-label y-zero">0</span>
	</div>

	<div class="chart-body">
		<div class="chart-plot-area" bind:this={plotAreaEl} onscroll={handleScroll}>
			<!-- THRESHOLD REFERENCE LINE AT Y=20 (66.66% height) -->
			<div class="threshold-line" title={m.chart_threshold_title()}>
				<span class="threshold-tag">{m.chart_target_threshold()}</span>
			</div>

			<!-- VERTICAL BARS -->
			<div class="vertical-bars-grid">
				{#each stacks as height, stackIdx (stackIdx)}
					{@const isUnsatisfactory = height < 20}
					{@const barHeightPercent = Math.min(100, Math.max(0, (height / 30) * 100))}
					{@const isSelected =
						inspectorOpen && inspectorPalletId === palletId && inspectorStackIndex === stackIdx}

					<button
						type="button"
						class="vertical-bar-column"
						class:unsatisfactory={isUnsatisfactory}
						class:selected={isSelected}
						onclick={() => onselect?.(stackIdx)}
						title={m.chart_stack_title({ number: stackIdx + 1, height })}
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
							<span class="bar-warning-dot" title={m.chart_unsatisfactory_title()}>!</span>
						{/if}
					</button>
				{/each}
			</div>
		</div>

		<!-- CUSTOM SCROLLBAR (replaces native horizontal scrollbar) -->
		<div
			class="custom-scrollbar-track"
			bind:this={trackEl}
			class:overflowing={hasOverflow}
			role="presentation"
			onpointerdown={onTrackPointerDown}
		>
			<div
				class="custom-scrollbar-thumb"
				bind:this={thumbEl}
				class:dragging
				role="slider"
				tabindex="0"
				aria-label={m.chart_scroll_aria()}
				aria-valuemin={0}
				aria-valuemax={maxScroll}
				aria-valuenow={Math.min(scrollLeft, maxScroll)}
				style="left: {thumbOffsetPct}%; width: {thumbWidthPct}%;"
				onpointerdown={onThumbPointerDown}
				onpointermove={onThumbPointerMove}
				onpointerup={onThumbPointerUp}
				onpointercancel={onThumbPointerUp}
				onkeydown={onThumbKeydown}
			></div>
		</div>
	</div>
</div>

<style>
	.vertical-chart-container {
		display: flex;
		height: 270px;
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
		padding-bottom: 24px;
	}

	.y-target {
		color: #da1e28;
	}

	.chart-body {
		flex: 1;
		min-width: 0;
		display: flex;
		flex-direction: column;
	}

	/* PLOT AREA (native scrollbar hidden) */
	.chart-plot-area {
		flex: 1;
		min-height: 0;
		position: relative;
		border-left: 2px solid #0f0f0f;
		background: #ffffff;
		max-width: 100%;
		overflow-x: scroll;
		overflow-y: hidden;
		scrollbar-width: none;
		-ms-overflow-style: none;
		padding-bottom: 24px;
	}

	.chart-plot-area::-webkit-scrollbar {
		display: none;
		width: 0;
		height: 0;
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

	.bar-value-label {
		font-size: 0.75rem;
		font-weight: 900;
		margin-bottom: 4px;
		color: #0f0f0f;
	}

	.bar-value-label.alert-text {
		color: #da1e28;
	}

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

	/* CUSTOM SCROLLBAR (always visible) */
	.custom-scrollbar-track {
		flex: 0 0 auto;
		height: 16px;
		background: #f4f4f4;
		border-left: 2px solid #0f0f0f;
		border-bottom: 2px solid #0f0f0f;
		position: relative;
		cursor: pointer;
		user-select: none;
	}

	.custom-scrollbar-thumb {
		position: absolute;
		top: 2px;
		bottom: 2px;
		left: 0;
		background: #0f0f0f;
		border-radius: 2px;
		cursor: grab;
		transition: background 0.15s ease;
	}

	.custom-scrollbar-thumb:hover {
		background: #393939;
	}

	.custom-scrollbar-thumb.dragging {
		background: #6f6f6f;
		cursor: grabbing;
	}

	.custom-scrollbar-track.overflowing .custom-scrollbar-thumb::after {
		content: '';
		position: absolute;
		top: 50%;
		left: 50%;
		transform: translate(-50%, -50%);
		width: 20px;
		height: 2px;
		border-radius: 2px;
		background: rgba(255, 255, 255, 0.5);
	}
</style>
