<!--
	«Add a game from your library»: a combobox over the games you hold that are not on the board.
	Enter adds to Backlog, Shift+Enter to Playing; arrows move through the list, Escape clears.
-->
<script lang="ts">
	import { IconPlus, IconSearch } from '@tabler/icons-svelte';
	import type { LibraryEntry } from '#lib/bindings/LibraryEntry.ts';
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import Cover from '#lib/components/store/Cover.svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import { storeName } from '#lib/format.ts';
	import { tr } from '#lib/i18n/index.ts';
	import type { PlayState } from '#lib/ipc.ts';
	import type { Store } from '#lib/library/shelf.ts';
	import { addable, keyOf } from '#lib/plays/model.ts';

	let {
		library,
		plays,
		onadd
	}: {
		library: LibraryEntry[];
		plays: PlayView[];
		onadd: (e: LibraryEntry, state: PlayState) => void;
	} = $props();

	let q = $state('');
	let active = $state(0);
	const matches = $derived(addable(library, plays, q));

	function add(e: LibraryEntry, state: PlayState) {
		onadd(e, state);
		q = '';
		active = 0;
	}

	function key(e: KeyboardEvent) {
		if (e.key === 'Escape') {
			q = '';
			return;
		}
		if (!matches.length) return;
		if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
			active = (active + (e.key === 'ArrowDown' ? 1 : -1) + matches.length) % matches.length;
			e.preventDefault();
		} else if (e.key === 'Enter') {
			e.preventDefault();
			const m = matches[Math.min(active, matches.length - 1)];
			if (m) add(m, e.shiftKey ? 'playing' : 'backlog');
		}
	}
</script>

<div class="add">
	<label class="control control-search">
		<IconPlus size={16} stroke={1.75} />
		<span class="sr-only">{tr.plays.add}</span>
		<input
			type="search"
			placeholder="{tr.plays.add}…"
			bind:value={q}
			oninput={() => (active = 0)}
			onkeydown={key}
			role="combobox"
			aria-expanded={matches.length > 0}
			aria-controls="add-list"
			aria-autocomplete="list"
			aria-activedescendant={matches.length ? `add-${active}` : undefined}
		/>
	</label>
	{#if matches.length}
		<ul id="add-list" role="listbox" class="pop" aria-label={tr.plays.add}>
			{#each matches as m, i (keyOf(m))}
				<li id="add-{i}" role="option" aria-selected={i === active}>
					<button
						type="button"
						tabindex="-1"
						onclick={() => add(m, 'backlog')}
						onmouseenter={() => (active = i)}
					>
						<Cover title={m.title ?? m.native_id} store={m.store as Store} src={m.image_url} />
						<span class="t">{m.title ?? tr.library.untitled(m.native_id)}</span>
						<span class="st" title={storeName(m.store)}
							><StoreLogo store={m.store} size={14} /></span
						>
					</button>
				</li>
			{/each}
			<li class="tip" role="presentation">{tr.plays.addTip}</li>
		</ul>
	{:else if q.trim()}
		<p class="pop none" role="status">
			<IconSearch size={14} stroke={1.75} />
			{tr.plays.addNone(q.trim())}
		</p>
	{/if}
</div>

<style>
	.add {
		position: relative;
		flex: 1;
		max-width: 420px;
	}
	.add :global(.control-search) {
		width: 100%;
	}
	.pop {
		position: absolute;
		z-index: 30;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		margin: 0;
		padding: 4px;
		list-style: none;
		background: var(--color-page);
		border: 2px solid var(--color-border-strong);
		box-shadow: 0 12px 32px rgb(0 0 0 / 0.18);
	}
	.pop button {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 8px;
		border: 0;
		background: none;
		color: inherit;
		text-align: left;
	}
	[aria-selected='true'] button {
		background: var(--color-raised);
	}
	.t {
		flex: 1;
		min-width: 0;
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.st {
		display: grid;
		color: var(--color-muted);
	}
	.tip,
	.none {
		display: flex;
		gap: 6px;
		align-items: center;
		padding: 6px 8px;
		font-size: 12px;
		color: var(--color-muted);
	}
</style>
