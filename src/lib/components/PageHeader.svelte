<!-- Page title in the condensed grotesque, with room for counters and the next-step action. -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		info,
		figures,
		action
	}: { title: string; info?: Snippet; figures?: Snippet; action?: Snippet } = $props();
</script>

<!-- One row at every width: the header gives way by its own width, it never wraps. -->
<header class="top">
	<div class="ttl">
		<h1 class="headline">{title}</h1>
		{#if info}{@render info()}{/if}
	</div>
	{#if figures}<dl class="figs">{@render figures()}</dl>{/if}
	{#if action}<div class="act">{@render action()}</div>{/if}
</header>

<style>
	.top {
		container-type: inline-size;
		display: flex;
		flex-wrap: nowrap;
		align-items: flex-end;
		gap: 28px;
		padding: 22px 32px 14px;
		border-bottom: 3px solid var(--color-border-strong);
	}
	.ttl {
		flex: none;
		display: flex;
		align-items: center;
		gap: 6px;
	}
	h1 {
		margin: 0;
		font-size: 56px;
		white-space: nowrap;
	}
	.figs {
		display: flex;
		flex-wrap: nowrap;
		min-width: 0;
		margin: 0;
		overflow: hidden;
	}
	.figs :global(div) {
		padding: 0 16px;
		border-left: 1px solid var(--color-border);
	}
	.figs :global(dt) {
		display: flex;
		white-space: nowrap;
		align-items: center;
		gap: 5px;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--color-muted);
	}
	.figs :global(dd) {
		margin: 0;
		font-size: 26px;
		font-variation-settings: 'wdth' 75;
		font-weight: 800;
	}
	.act {
		flex: none;
		margin-left: auto;
		white-space: nowrap;
	}
	/* Narrower: smaller title and figures. */
	@container (max-width: 1000px) {
		.top {
			gap: 18px;
		}
		h1 {
			font-size: 44px;
		}
		.figs :global(div) {
			padding: 0 10px;
		}
		.figs :global(dd) {
			font-size: 20px;
		}
	}
	/* Narrower still: keep the first three figures (the totals). */
	@container (max-width: 860px) {
		.figs :global(div:nth-child(n + 4)) {
			display: none;
		}
	}
	/* Too narrow for figures at all: title and next step only. */
	@container (max-width: 680px) {
		.figs {
			display: none;
		}
		h1 {
			font-size: 38px;
		}
	}
</style>
