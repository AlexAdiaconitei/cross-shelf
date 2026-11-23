<!--
	The board: one column per state, «Playing» headed in ink. Cards move by dragging (between and
	inside columns; a line shows where the card will land), with their «Move to…» menu, or with
	Alt+←/→ on a focused card. Each column scrolls on its own; the board never scrolls sideways.
-->
<script lang="ts">
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import { tr } from '#lib/i18n/index.ts';
	import { PLAY_STATES, type PlayState } from '#lib/ipc.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { columns, dropPlace, keyOf } from '#lib/plays/model.ts';
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
		onmove: (
			p: PlayView,
			state: string,
			place?: { before?: PlayView; end?: boolean }
		) => Promise<void>;
		onremove: (p: PlayView) => void;
	} = $props();

	const cols = $derived(columns(plays));

	let dragging = $state<PlayView | null>(null);
	/** Where the dragged card would land: its column, and the card it would go ahead of. */
	let target = $state<{ state: PlayState; before: string | null } | null>(null);

	/** Follows the pointer over a column: `index` is the card under it, null below every card. */
	function aim(e: DragEvent, state: PlayState, items: PlayView[], index: number | null) {
		if (!dragging) return;
		e.preventDefault();
		if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
		let lower = true;
		if (index !== null) {
			const r = (e.currentTarget as HTMLElement).getBoundingClientRect();
			lower = e.clientY > r.top + r.height / 2;
		}
		const place = dropPlace(
			items.map((p) => keyOf(p.product)),
			index,
			lower,
			keyOf(dragging.product)
		);
		target = { state, before: 'before' in place ? place.before : null };
	}

	function drop(items: PlayView[]) {
		const d = dragging;
		const t = target;
		stop();
		if (!d || !t) return;
		const before = t.before === null ? undefined : items.find((p) => keyOf(p.product) === t.before);
		void onmove(d, t.state, before ? { before } : { end: true });
	}

	function stop() {
		dragging = null;
		target = null;
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
			class:over={target?.state === col.state}
			aria-labelledby="col-{col.state}"
			ondragover={(e) => aim(e, col.state, col.items, null)}
			ondragleave={(e) => {
				if (target?.state === col.state && !e.currentTarget.contains(e.relatedTarget as Node))
					target = null;
			}}
			ondrop={(e) => {
				e.preventDefault();
				drop(col.items);
			}}
		>
			<header>
				<h2 id="col-{col.state}" class="headline">{tr.plays.states[col.state]}</h2>
				<b class="figure">{col.items.length}</b>
			</header>
			<ul class:mark-end={target?.state === col.state && target.before === null}>
				{#each col.items as p, i (keyOf(p.product))}
					<li
						draggable="true"
						class:drag={dragging === p}
						class:mark-before={target?.state === col.state && target.before === keyOf(p.product)}
						ondragstart={(e) => {
							dragging = p;
							if (e.dataTransfer) {
								e.dataTransfer.effectAllowed = 'move';
								e.dataTransfer.setData('text/plain', keyOf(p.product));
							}
						}}
						ondragend={stop}
						ondragover={(e) => {
							e.stopPropagation();
							aim(e, col.state, col.items, i);
						}}
						ondrop={(e) => {
							e.preventDefault();
							e.stopPropagation();
							drop(col.items);
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
		grid-template-columns: minmax(0, 1fr);
		gap: 8px;
		align-content: start;
		overflow-x: hidden;
		overflow-y: auto;
		flex: 1;
	}
	li {
		position: relative;
	}
	li.drag {
		opacity: 0.4;
	}
	/* Where the card will land: a line in ink between two cards, or under the last one. */
	li.mark-before::before,
	ul.mark-end::after {
		content: '';
		display: block;
		height: 3px;
		background: var(--color-border-strong);
	}
	li.mark-before::before {
		position: absolute;
		left: 0;
		right: 0;
		top: -6px;
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
