<!-- Page title in the condensed grotesque, with room for counters and the next-step action. -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let { title, figures, action }: { title: string; figures?: Snippet; action?: Snippet } = $props();
</script>

<header class="top">
	<h1 class="headline">{title}</h1>
	{#if figures}<dl class="figs">{@render figures()}</dl>{/if}
	{#if action}<div class="act">{@render action()}</div>{/if}
</header>

<style>
	.top {
		container-type: inline-size;
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		gap: 16px 28px;
		padding: 22px 32px 14px;
		border-bottom: 3px solid var(--color-border-strong);
	}
	h1 {
		margin: 0;
		font-size: 56px;
	}
	.figs {
		display: flex;
		flex-wrap: wrap;
		margin: 0;
	}
	.figs :global(div) {
		padding: 0 16px;
		border-left: 1px solid var(--color-border);
	}
	.figs :global(dt) {
		display: flex;
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
		margin-left: auto;
	}
</style>
