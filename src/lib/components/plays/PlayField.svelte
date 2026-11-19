<!--
	The «Play» field of a game sheet: its state in Mis partidas (or none) and a short note. A game
	grouped across stores keeps its play on the product it was added from; a new play goes on the
	first product given.
-->
<script lang="ts">
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import { tr } from '#lib/i18n/index.ts';
	import { ipc, PLAY_STATES, type PlayState } from '#lib/ipc.ts';
	import { keyOf } from '#lib/plays/model.ts';

	let {
		products,
		onchange
	}: {
		products: { store: string; native_id: string }[];
		/** Called with the whole list after a change, so the view can redraw tags or the board. */
		onchange?: (plays: PlayView[]) => void;
	} = $props();

	const s = tr.plays.sheet;
	const NOTE_MAX = 280;
	let plays = $state<PlayView[]>([]);
	let failed = $state<string | null>(null);

	const keys = $derived(products.map(keyOf));
	const play = $derived(plays.find((p) => keys.includes(keyOf(p.product))) ?? null);
	const target = $derived(play?.product ?? products[0] ?? null);
	// Follows the saved note; typing overrides it until the play changes.
	let note = $derived(play?.note ?? '');
	const left = $derived(NOTE_MAX - [...note.trim()].length);

	$effect(() => {
		void keys;
		let live = true;
		ipc
			.plays()
			.then((p) => {
				if (live) plays = p;
			})
			.catch(() => {});
		return () => {
			live = false;
		};
	});

	async function setState(v: string) {
		if (!target) return;
		failed = null;
		try {
			if (v) {
				plays = await ipc.playSet(target.store, target.native_id, v as PlayState);
			} else {
				await ipc.playRemove(target.store, target.native_id);
				plays = await ipc.plays();
			}
			onchange?.(plays);
		} catch {
			failed = tr.plays.actionFailed;
		}
	}

	async function saveNote() {
		if (!play || note.trim() === play.note) return;
		failed = null;
		if (left < 0) {
			failed = s.noteTooLong;
			return;
		}
		try {
			await ipc.playNote(play.product.store, play.product.native_id, note);
			plays = await ipc.plays();
			onchange?.(plays);
		} catch {
			failed = tr.plays.actionFailed;
		}
	}
</script>

{#if target}
	<section class="pf" aria-label={s.play}>
		<label class="row">
			<span class="label">{s.play}</span>
			<select
				class="control"
				class:set={play}
				value={play?.state ?? ''}
				onchange={(e) => setState(e.currentTarget.value)}
			>
				<option value="">{s.noPlay}</option>
				{#each PLAY_STATES as st (st)}<option value={st}>{tr.plays.states[st]}</option>{/each}
			</select>
		</label>
		{#if play}
			<label class="note">
				<span class="label">{s.note}</span>
				<textarea
					rows="2"
					placeholder={s.notePlaceholder}
					bind:value={note}
					onblur={saveNote}
					aria-invalid={left < 0}
					aria-describedby="note-left"></textarea>
				<small id="note-left" class:over={left < 0}>{s.noteLeft(left)}</small>
			</label>
		{/if}
		{#if failed}<p role="alert" class="text-danger">{failed}</p>{/if}
	</section>
{/if}

<style>
	.pf {
		display: grid;
		gap: 10px;
	}
	.row {
		display: grid;
		grid-template-columns: 96px minmax(0, 1fr);
		align-items: center;
		gap: 8px;
	}
	select.set {
		border-color: var(--color-border-strong);
		font-weight: 700;
	}
	.note {
		display: grid;
		gap: 4px;
	}
	textarea {
		width: 100%;
		min-height: 56px;
		padding: 6px 8px;
		border: 2px solid var(--color-border);
		background: var(--color-page);
		color: var(--color-text);
		font: inherit;
		resize: vertical;
	}
	textarea:focus-visible {
		border-color: var(--color-border-strong);
		outline: none;
	}
	small {
		font-size: 12px;
		color: var(--color-muted);
	}
	small.over {
		color: var(--color-danger);
	}
	p {
		margin: 0;
	}
</style>
