<!--
	Mis partidas (plan 0.3.0, variant P1): what you are playing, have pending or finished, over the
	games of your library in any store. Board or list, with the suggestions from play time above.
	The tab is remembered in the settings.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { IconLayoutKanban, IconList, IconNotebook } from '@tabler/icons-svelte';
	import type { DiaryRow } from '#lib/bindings/DiaryRow.ts';
	import type { LibraryEntry } from '#lib/bindings/LibraryEntry.ts';
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import type { ProductRef } from '#lib/bindings/ProductRef.ts';
	import type { Suggestion } from '#lib/bindings/Suggestion.ts';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import AddPlay from '#lib/components/plays/AddPlay.svelte';
	import Board from '#lib/components/plays/Board.svelte';
	import Diary from '#lib/components/plays/Diary.svelte';
	import PlaysList from '#lib/components/plays/PlaysList.svelte';
	import SuggestionTray from '#lib/components/plays/SuggestionTray.svelte';
	import DetailPanel from '#lib/components/store/DetailPanel.svelte';
	import GameTime from '#lib/components/plays/GameTime.svelte';
	import PlayField from '#lib/components/plays/PlayField.svelte';
	import { intl, tr } from '#lib/i18n/index.ts';
	import { ipc, PLAY_STATES, type PlayState } from '#lib/ipc.ts';
	import { buildShelf, coverOf, firstStore, type Slot, type Store } from '#lib/library/shelf.ts';
	import { kindName } from '#lib/format.ts';
	import { keyOf, nowSecs, playsTab, type PlaysTab } from '#lib/plays/model.ts';

	/** Below this width five columns no longer fit: the board shows as the list. */
	const BOARD_MIN = 880;

	let plays = $state<PlayView[]>([]);
	let suggestions = $state<Suggestion[]>([]);
	let diary = $state<DiaryRow[]>([]);
	let library = $state<LibraryEntry[]>([]);
	let tab = $state<PlaysTab>('board');
	let loaded = $state(false);
	let loadFailed = $state(false);
	let actionFailed = $state(false);
	let selected = $state<string | null>(null);
	let width = $state(1200);
	const now = nowSecs();

	const shelf = $derived(buildShelf(library));
	const rowOf = $derived(
		new Map(shelf.flatMap((r) => r.products.map((p) => [keyOf(p), r] as const)))
	);
	const shown = $derived(tab === 'board' && width < BOARD_MIN ? 'list' : tab);
	const counts = $derived(
		PLAY_STATES.map((s) => [s, plays.filter((p) => p.state === s).length] as const)
	);
	const sel = $derived(
		plays.find((p) => keyOf(p.product) === selected)?.product ??
			diary.find((r) => keyOf(r.product) === selected)?.product ??
			null
	);
	const selRow = $derived(sel ? rowOf.get(keyOf(sel)) : undefined);

	/** Store tabs of a card: every store of the same game in the library, like the Library row. */
	function slotsFor(p: ProductRef): Record<Store, Slot> {
		const row = rowOf.get(keyOf(p));
		if (row) return row.slots;
		const none: Record<Store, Slot> = { steam: 'none', gog: 'none', epic: 'none' };
		return { ...none, [p.store]: 'unknown' };
	}
	const slotsOf = (p: PlayView) => slotsFor(p.product);
	const toggle = (p: ProductRef) => (selected = selected === keyOf(p) ? null : keyOf(p));

	async function act(f: () => Promise<void>) {
		actionFailed = false;
		try {
			await f();
		} catch {
			actionFailed = true;
		}
	}

	/** Suggestions and diary depend on the plays: both are read again after every change. */
	const refreshSuggestions = async () => {
		[suggestions, diary] = await Promise.all([ipc.suggestions(), ipc.diary()]);
	};

	const setState = (p: ProductRef, state: PlayState | null) =>
		act(async () => {
			if (state) plays = await ipc.playSet(p.store, p.native_id, state);
			else {
				await ipc.playRemove(p.store, p.native_id);
				plays = await ipc.plays();
			}
			await refreshSuggestions();
		});

	/** Moves a card: to the top of `state`, ahead of `before`, or to the end of the column. */
	const move = (p: PlayView, state: string, place: { before?: PlayView; end?: boolean } = {}) =>
		act(async () => {
			const b = place.before?.product;
			plays = await ipc.playSet(
				p.product.store,
				p.product.native_id,
				state as PlayState,
				b && { store: b.store, native_id: b.native_id },
				place.end
			);
			await refreshSuggestions();
		});

	const remove = (p: PlayView) =>
		act(async () => {
			await ipc.playRemove(p.product.store, p.product.native_id);
			if (selected === keyOf(p.product)) selected = null;
			plays = await ipc.plays();
			await refreshSuggestions();
		});

	const add = (e: LibraryEntry, state: PlayState) =>
		act(async () => {
			plays = await ipc.playSet(e.store, e.native_id, state);
			await refreshSuggestions();
		});

	const accept = (s: Suggestion) =>
		act(async () => {
			plays = await ipc.playSet(s.product.store, s.product.native_id, s.target as PlayState);
			await refreshSuggestions();
		});

	const dismiss = (s: Suggestion) =>
		act(async () => {
			await ipc.suggestionDismiss(
				s.product.store,
				s.product.native_id,
				s.target as 'playing' | 'paused'
			);
			await refreshSuggestions();
		});

	async function choose(next: PlaysTab) {
		tab = next;
		try {
			const s = await ipc.settings();
			await ipc.saveSettings({ ...s, plays_tab: next });
		} catch {
			/* only the memory of the tab is lost */
		}
	}

	onMount(async () => {
		try {
			const [p, s, d, l, settings] = await Promise.all([
				ipc.plays(),
				ipc.suggestions(),
				ipc.diary(),
				ipc.library(),
				ipc.settings()
			]);
			plays = p;
			suggestions = s;
			diary = d;
			library = l;
			tab = playsTab(settings.plays_tab);
		} catch {
			loadFailed = true;
		} finally {
			loaded = true;
		}
	});

	const tabs: [PlaysTab, typeof IconList][] = [
		['board', IconLayoutKanban],
		['list', IconList],
		['diary', IconNotebook]
	];
</script>

<div class="page" class:withDetail={sel}>
	<section class="main" bind:clientWidth={width}>
		<PageHeader title={tr.nav.partidas}>
			{#snippet figures()}
				{#each counts as [s, n] (s)}
					<div>
						<dt>{tr.plays.states[s]}</dt>
						<dd>{n.toLocaleString(intl)}</dd>
					</div>
				{/each}
			{/snippet}
		</PageHeader>

		{#if loadFailed}
			<p role="alert" class="page-body text-danger">{tr.plays.loadError}</p>
		{:else if loaded}
			<div class="toolbar">
				<AddPlay {library} {plays} onadd={add} />
				<div class="push segmented" role="group" aria-label={tr.plays.tabsLabel}>
					{#each tabs as [t, Icon] (t)}
						<button type="button" class="control" aria-pressed={tab === t} onclick={() => choose(t)}
							><Icon size={16} stroke={1.75} />{tr.plays.tabs[t]}</button
						>
					{/each}
				</div>
			</div>
			{#if actionFailed}<p role="alert" class="err text-danger">{tr.plays.actionFailed}</p>{/if}

			<SuggestionTray list={suggestions} onaccept={accept} ondismiss={dismiss} />

			{#if shown === 'diary'}
				<Diary rows={diary} {slotsFor} {now} {selected} onselect={toggle} onset={setState} />
			{:else if plays.length === 0}
				<div class="empty">
					<p class="headline">{tr.plays.empty}</p>
					<p>{tr.plays.emptyBody}</p>
				</div>
			{:else if shown === 'board'}
				<Board
					{plays}
					{slotsOf}
					{now}
					{selected}
					onselect={(p) => toggle(p.product)}
					onmove={move}
					onremove={remove}
				/>
			{:else}
				<PlaysList
					{plays}
					{slotsOf}
					{now}
					{selected}
					onselect={(p) => toggle(p.product)}
					onmove={move}
					onremove={remove}
				/>
			{/if}
		{/if}
	</section>

	{#if sel}
		{@const title = sel.title ?? tr.library.untitled(sel.native_id)}
		<DetailPanel
			{title}
			store={selRow ? firstStore(selRow) : (sel.store as Store)}
			cover={selRow ? coverOf(selRow) : sel.image_url}
			slots={slotsFor(sel)}
			products={selRow
				? selRow.products.map((p) => ({ ...p, note: '' }))
				: [{ ...sel, note: tr.plays.notInLibrary }]}
			onclose={() => (selected = null)}
		>
			{#snippet spec()}
				<dt>{tr.library.kind}</dt>
				<dd>{kindName(sel.kind)}</dd>
			{/snippet}
			<PlayField
				products={[sel]}
				onchange={async (p) => {
					plays = p;
					await refreshSuggestions().catch(() => {});
				}}
			/>
			<GameTime {title} products={selRow ? selRow.products : [sel]} />
		</DetailPanel>
	{/if}
</div>

<style>
	.page {
		position: relative;
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
	}
	.page.withDetail {
		grid-template-columns: minmax(0, 1fr) 440px;
	}
	@media (max-width: 1279px) {
		.page.withDetail {
			grid-template-columns: minmax(0, 1fr);
		}
		.page.withDetail > :global(.detail) {
			position: absolute;
			inset: 0 0 0 auto;
			width: min(440px, 90%);
			box-shadow: -16px 0 40px rgb(0 0 0 / 0.35);
		}
	}
	.main {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
	}
	.err {
		margin: 8px 32px 0;
	}
	.empty {
		display: grid;
		gap: 10px;
		padding: 40px 32px;
		max-width: 70ch;
	}
	.empty .headline {
		font-size: 30px;
		margin: 0;
	}
	.empty p {
		margin: 0;
	}
</style>
