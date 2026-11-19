<!--
	The board: one column per state, «Playing» headed in ink. Cards move by dragging (between and
	inside columns), with their «Move to…» menu, or with Alt+←/→ on a focused card. Each column
	scrolls on its own; the board never scrolls sideways.
-->
<script lang="ts">
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import { tr } from '#lib/i18n/index.ts';
	import { PLAY_STATES, type PlayState } from '#lib/ipc.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { columns, keyOf } from '#lib/plays/model.ts';
	import PlayCard from './PlayCard.svelte';

	let {
		plays,
		slotsOf,
		now,
		selected,
		onselect,
		onmove,
		onremove
	}: {
		plays: PlayView[];
		slotsOf: (p: PlayView) => Record<Store, Slot>;
		now: number;
		selected: string | null;
		onselect: (p: PlayView) => void;
		onmove: (p: PlayView, state: string, before?: PlayView) => Promise<void>;
		onremove: (p: PlayView) => void;
	} = $props();

	const cols = $derived(columns(plays));

	let dragging = $state<PlayView | null>(null);
	let over = $state<PlayState | null>(null);

	function drop(state: PlayState, before?: PlayView) {
		const d = dragging;
		dragging = null;
		over = null;
		if (!d || (before && keyOf(before.product) === keyOf(d.product))) return;
		void onmove(d, state, before);
	}

	async function cardKey(e: KeyboardEvent, p: PlayView) {
		if (!e.altKey || (e.key !== 'ArrowLeft' && e.key !== 'ArrowRight')) return;
		const i = PLAY_STATES.indexOf(p.state as PlayState);
		const next = PLAY_STATES[i + (e.key === 'ArrowRight' ? 1 : -1)];
		if (!next) return;
		e.preventDefault();
		await onmove(p, next);
		// Keep the focus on the moved card.
		document.querySelector<HTMLElement>(`[data-play="${CSS.escape(keyOf(p.product))}"]`)?.focus();
	}
</script>

<div class="board">
	{#each cols as col (col.state)}
		<section
			class="col {col.state}"
			class:over={over === col.state}
			aria-labelledby="col-{col.state}"
			ondragover={(e) => {
				e.preventDefault();
				over = col.state;
			}}
			ondragleave={(e) => {
				if (!e.currentTarget.contains(e.relatedTarget as Node)) over = null;
			}}
			ondrop={(e) => {
				e.preventDefault();
				drop(col.state);
			}}
		>
			<header>
				<h2 id="col-{col.state}" class="headline">{tr.plays.states[col.state]}</h2>
				<b class="figure">{col.items.length}</b>
			</header>
			<ul>
				{#each col.items as p (keyOf(p.product))}
					<li
						draggable="true"
						class:drag={dragging === p}
						ondragstart={(e) => {
							dragging = p;
							e.dataTransfer?.setData('text/plain', keyOf(p.product));
						}}
						ondragend={() => {
							dragging = null;
							over = null;
						}}
						ondrop={(e) => {
							e.preventDefault();
							e.stopPropagation();
							drop(col.state, p);
						}}
					>
						<PlayCard
							{p}
							slots={slotsOf(p)}
							{now}
							selected={selected === keyOf(p.product)}
							hint="hint-{col.state}"
							onselect={() => onselect(p)}
							onkey={(e) => cardKey(e, p)}
							onmove={(s) => onmove(p, s)}
							onremove={() => onremove(p)}
						/>
					</li>
				{:else}
					<li class="drop">{tr.plays.stateHints[col.state]}. {tr.plays.dropHint}</li>
				{/each}
			</ul>
			<p id="hint-{col.state}" class="sr-only">{tr.plays.keyHint}</p>
		</section>
	{/each}
</div>

<style>
	.board {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: repeat(5, minmax(0, 1fr));
		gap: 10px;
		padding: 16px 24px 24px;
	}
	.col {
		display: flex;
		flex-direction: column;
		min-width: 0;
		min-height: 0;
		background: var(--color-panel);
		transition: box-shadow 150ms ease-out;
	}
	.col.over {
		box-shadow: inset 0 0 0 2px var(--color-border-strong);
	}
	.col header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 12px 8px;
		border-bottom: 3px solid var(--color-border-strong);
	}
	.col.playing header {
		background: var(--color-accent);
		color: var(--color-on-accent);
		border-bottom-color: var(--color-accent);
	}
	h2 {
		margin: 0;
		font-size: 24px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	header b {
		font-size: 22px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 8px;
		display: grid;
		gap: 8px;
		align-content: start;
		overflow-y: auto;
		flex: 1;
	}
	li.drag {
		opacity: 0.4;
	}
	.drop {
		padding: 14px 10px;
		font-size: 12px;
		color: var(--color-muted);
		border: 1.5px dashed var(--color-z5);
	}
	@media (prefers-reduced-motion: reduce) {
		.col {
			transition: none;
		}
	}
</style>
