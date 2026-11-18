<!--
	Extra information behind an icon: shown on hover and on keyboard focus, read by screen
	readers through the trigger's label. `warn` turns the icon amber so a problem (a partial or
	failed sync) is still visible at a glance; the details stay in the tip.
-->
<script lang="ts">
	import { Portal } from '@ark-ui/svelte/portal';
	import { Tooltip } from '@ark-ui/svelte/tooltip';
	import { IconAlertTriangle, IconInfoCircle } from '@tabler/icons-svelte';

	let {
		label,
		lines,
		warn = false,
		size = 16
	}: { label: string; lines: string[]; warn?: boolean; size?: number } = $props();
</script>

{#if lines.length}
	<Tooltip.Root openDelay={150} closeDelay={100} positioning={{ placement: 'bottom-start' }}>
		<Tooltip.Trigger
			type="button"
			class="tip-trigger {warn ? 'warn' : ''}"
			aria-label={`${label}: ${lines.join('. ')}`}
		>
			{#if warn}<IconAlertTriangle {size} />{:else}<IconInfoCircle {size} />{/if}
		</Tooltip.Trigger>
		<Portal>
			<Tooltip.Positioner>
				<Tooltip.Content class="tip-content">
					<ul>
						{#each lines as l (l)}<li>{l}</li>{/each}
					</ul>
				</Tooltip.Content>
			</Tooltip.Positioner>
		</Portal>
	</Tooltip.Root>
{/if}

<style>
	:global(.tip-trigger) {
		display: inline-grid;
		place-items: center;
		width: 26px;
		height: 26px;
		padding: 0;
		border: 0;
		background: none;
		color: var(--color-muted);
		cursor: help;
		vertical-align: middle;
	}
	:global(.tip-trigger:hover) {
		color: var(--color-text);
	}
	:global(.tip-trigger.warn) {
		color: var(--color-warning);
	}
	:global(.tip-content) {
		z-index: 60;
		max-width: 360px;
		padding: 10px 12px;
		background: var(--color-z10);
		color: var(--color-z1);
		font-size: 12px;
		line-height: 1.45;
		box-shadow: var(--shadow-overlay);
	}
	:global(.tip-content ul) {
		display: grid;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
</style>
