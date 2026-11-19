<!--
	One play on the board: medium cover, title, play time and last session when known, the date
	line, the note, store tabs and the «Move to…» menu. Alt+←/→ moves it a column.
-->
<script lang="ts">
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import Cover from '#lib/components/store/Cover.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { tr } from '#lib/i18n/index.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { keyOf, sinceText, timeLine } from '#lib/plays/model.ts';
	import MoveMenu from './MoveMenu.svelte';

	let {
		p,
		slots,
		now,
		selected,
		hint,
		onselect,
		onkey,
		onmove,
		onremove
	}: {
		p: PlayView;
		slots: Record<Store, Slot>;
		now: number;
		selected: boolean;
		hint: string;
		onselect: () => void;
		onkey: (e: KeyboardEvent) => void;
		onmove: (state: string) => void;
		onremove: () => void;
	} = $props();

	const title = $derived(p.product.title ?? tr.library.untitled(p.product.native_id));
	const time = $derived(timeLine(p.minutes, p.last_activity, now));
</script>

<article class="card" aria-current={selected ? 'true' : undefined}>
	<button
		type="button"
		class="open"
		data-play={keyOf(p.product)}
		onclick={onselect}
		onkeydown={onkey}
		aria-describedby={hint}
	>
		<Cover {title} store={p.product.store as Store} src={p.product.image_url} size="md" />
		<span class="ct">
			<b>{title}</b>
			{#if time}<span class="tm">{time}</span>{/if}
			<small>{sinceText(p, now)}</small>
			{#if !p.in_library}<small class="gone">{tr.plays.notInLibrary}</small>{/if}
			{#if p.note}<em>{p.note}</em>{/if}
		</span>
	</button>
	<div class="cf">
		<StoreTabs {slots} {title} size="sm" />
		<MoveMenu {title} state={p.state} {onmove} {onremove} />
	</div>
</article>

<style>
	.card {
		background: var(--color-page);
		border: 1px solid var(--color-border);
		cursor: grab;
	}
	.card[aria-current='true'] {
		box-shadow: inset 0 0 0 2px var(--color-border-strong);
	}
	.open {
		width: 100%;
		display: flex;
		gap: 10px;
		align-items: flex-start;
		padding: 10px 10px 6px;
		border: 0;
		background: none;
		color: inherit;
		text-align: left;
	}
	.ct {
		display: grid;
		gap: 2px;
		min-width: 0;
	}
	.ct b {
		font-weight: 700;
		line-height: 1.2;
		overflow-wrap: anywhere;
	}
	.tm {
		font-size: 13px;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.ct small {
		font-size: 12px;
		color: var(--color-muted);
	}
	.ct small.gone {
		color: var(--color-warning);
	}
	.ct em {
		font-style: normal;
		font-size: 12px;
		margin-top: 2px;
		overflow-wrap: anywhere;
	}
	.cf {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 6px;
		padding: 0 10px 10px;
	}
	.cf :global(.tabs) {
		justify-content: flex-start;
	}
</style>
