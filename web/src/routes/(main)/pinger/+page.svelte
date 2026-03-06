<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { getPingerStats, getPingerConfig } from '$lib/api';
	import type { PingerStats, PingerConfig } from '$lib/types';
	import Chart from '$lib/Chart.svelte';

	let stats = $state<PingerStats[]>([]);
	let config = $state<PingerConfig | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let interval: ReturnType<typeof setInterval>;

	async function fetchData() {
		const [statsRes, configRes] = await Promise.all([getPingerStats(), getPingerConfig()]);

		if (statsRes.success) {
			stats = statsRes.data;
		} else {
			error = statsRes.error;
		}

		if (configRes.success) {
			config = configRes.data;
		}

		loading = false;
	}

	onMount(() => {
		fetchData();
		interval = setInterval(fetchData, 2000);
	});

	onDestroy(() => {
		if (interval) clearInterval(interval);
	});

	let currentLatency = $derived(stats.length > 0 ? stats[stats.length - 1].latency_ms : null);
	let status = $derived(currentLatency !== null && currentLatency >= 0 ? 'Online' : 'Offline');
	let timestamps = $derived(stats.map((s) => s.timestamp));
	let latencyData = $derived(stats.map((s) => (s.latency_ms >= 0 ? s.latency_ms.toFixed(2) : 0)));
</script>

<div class="pinger-container">
	{#if loading && stats.length === 0}
		<div class="loading">Loading pinger data...</div>
	{:else if error && stats.length === 0}
		<div class="error">Error: {error}</div>
	{:else}
		<div class="two-columns">
			<div class="card">
				<p class="card-title">Pinger Status</p>
				<div class="status-info">
					<div class="info-item">
						<span class="info-label">Status:</span>
						<span class="status-badge {status.toLowerCase()}">{status}</span>
					</div>
					<div class="info-item">
						<span class="info-label">Target:</span>
						<span class="info-value">{config?.target || 'Unknown'}</span>
					</div>
					<div class="info-item">
						<span class="info-label">Source:</span>
						<span class="info-value">{config?.source || 'Unknown'}</span>
					</div>
				</div>
			</div>

			<div class="card">
				<p class="card-title">Current Latency</p>
				<div class="latency-display">
					{#if currentLatency !== null && currentLatency >= 0}
						<span class="latency-value">{currentLatency.toFixed(2)}</span>
						<span class="latency-unit">ms</span>
					{:else}
						<span class="latency-value negative">N/A</span>
					{/if}
				</div>
			</div>
		</div>

		<div class="chart-card">
			<p class="card-title">Real-time Latency (Last 100 points)</p>
			<div style="min-height: 40vh;">
				<Chart
					{timestamps}
					yAxisLabel="Latency (ms)"
					autoScale={true}
					data={[latencyData]}
					labels={['Latency']}
					colors={['#4dabf7']}
					bg_colors={['rgba(77, 171, 247, 0.1)']}
					fills={[true]}
				/>
			</div>
		</div>
	{/if}
</div>

<style>
	.status-badge {
		padding: 2px 8px;
		border-radius: 4px;
		font-weight: bold;
		font-size: 0.9em;
	}
	.status-badge.online {
		background-color: rgba(40, 167, 69, 0.2);
		color: #28a745;
	}
	.status-badge.offline {
		background-color: rgba(220, 53, 69, 0.2);
		color: #dc3545;
	}
	.latency-display {
		display: flex;
		align-items: baseline;
		justify-content: center;
		padding: 20px 0;
	}
	.latency-value {
		font-size: 3em;
		font-weight: bold;
		color: #4dabf7;
	}
	.latency-value.negative {
		color: #dc3545;
	}
	.latency-unit {
		font-size: 1.2em;
		margin-left: 5px;
		color: #868e96;
	}
	.status-info {
		display: flex;
		flex-direction: column;
		gap: 10px;
		margin-top: 10px;
	}
</style>
