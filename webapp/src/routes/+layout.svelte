<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import 'carbon-components-svelte/css/white.css';
	import { page } from '$app/state';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>PALLETIZER // SIMULATOR V2</title>
	<link rel="preconnect" href="https://fonts.googleapis.com">
	<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous">
	<link href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:ital,wght@0,100..800;1,100..800&display=swap" rel="stylesheet">
</svelte:head>

<div class="industrial-app-container">
	<header class="industrial-header">
		<div class="header-brand">
			<span class="brand-badge">ROULARTA // LOGISTICS</span>
			<h1 class="brand-title">PALLETIZER SIMULATOR</h1>
		</div>
		<nav class="header-nav">
			<a href="/" class="nav-btn" class:active={page.url.pathname === '/'}>
				<span class="nav-tag">[V1]</span> LEGACY
			</a>
			<a href="/v2" class="nav-btn" class:active={page.url.pathname === '/v2'}>
				<span class="nav-tag">[V2]</span> INDUSTRIAL V2
			</a>
		</nav>
	</header>

	<main class="industrial-main">
		{@render children()}
	</main>
</div>

<style>
	:global(:root) {
		--font-mono: 'JetBrains Mono', 'IBM Plex Mono', 'Courier New', monospace;
		--bg-color: #ffffff;
		--text-primary: #0f0f0f;
		--text-secondary: #525252;
		--border-color: #161616;
		--border-light: #e0e0e0;
		--bg-subtle: #f4f4f4;
		--accent-warning: #da1e28;
		--accent-warning-bg: #fff0f1;
		--accent-success: #198038;
		--accent-success-bg: #defce6;
	}

	:global(body) {
		margin: 0;
		padding: 0;
		background-color: var(--bg-color);
		color: var(--text-primary);
		font-family: var(--font-mono);
		-webkit-font-smoothing: antialiased;
	}

	:global(*, *::before, *::after) {
		box-sizing: border-box;
		font-family: var(--font-mono) !important;
	}

	.industrial-app-container {
		min-height: 100vh;
		display: flex;
		flex-direction: column;
		background: #ffffff;
	}

	.industrial-header {
		display: flex;
		justify-content: space-between;
		align-items: center;
		padding: 1rem 2rem;
		border-bottom: 2px solid var(--border-color);
		background-color: #ffffff;
		position: sticky;
		top: 0;
		z-index: 1000;
	}

	.header-brand {
		display: flex;
		flex-direction: column;
		gap: 0.2rem;
	}

	.brand-badge {
		font-size: 0.65rem;
		font-weight: 700;
		letter-spacing: 0.15em;
		color: var(--text-secondary);
		text-transform: uppercase;
	}

	.brand-title {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 800;
		letter-spacing: -0.02em;
		color: var(--text-primary);
		text-transform: uppercase;
	}

	.header-nav {
		display: flex;
		gap: 0.5rem;
	}

	.nav-btn {
		display: inline-flex;
		align-items: center;
		gap: 0.4rem;
		padding: 0.5rem 1rem;
		font-size: 0.8rem;
		font-weight: 600;
		text-decoration: none;
		color: var(--text-primary);
		border: 1px solid var(--border-color);
		background: #ffffff;
		transition: all 0.15s ease;
	}

	.nav-btn:hover {
		background: #0f0f0f;
		color: #ffffff;
	}

	.nav-btn.active {
		background: #0f0f0f;
		color: #ffffff;
		border-color: #0f0f0f;
	}

	.nav-tag {
		font-size: 0.7rem;
		opacity: 0.8;
	}

	.industrial-main {
		flex: 1;
		padding: 2rem;
		max-width: 1400px;
		width: 100%;
		margin: 0 auto;
	}
</style>
