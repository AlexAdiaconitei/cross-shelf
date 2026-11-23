<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import {
		IconAlertTriangle,
		IconArrowRight,
		IconExternalLink,
		IconFilterOff,
		IconSearch
	} from '@tabler/icons-svelte';
	import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
	import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import Cover from '#lib/components/store/Cover.svelte';
	import DetailPanel from '#lib/components/store/DetailPanel.svelte';
	import PlayField from '#lib/components/plays/PlayField.svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { InfoTip, button } from '#lib/components/ui/index.ts';
	import { day, storeName, storeWishlistUrl, when } from '#lib/format.ts';
	import { ipc } from '#lib/ipc.ts';
	import { visibleRange, type Entry } from '#lib/library/filter.ts';
	import { buildShelf, shelfKey, type Shelf, type Slot, type Store } from '#lib/library/shelf.ts';
	import { shell } from '#lib/shell/state.svelte.ts';
	import { intl, tr } from '#lib/i18n/index.ts';
	import {
		applyWishFilters,
		isWishFiltering,
		noWishFilters,
		wishView,
		type WishFilter,
		type WishSort
	} from '#lib/wishlist/filter.ts';
	import { wishlistUi } from '#lib/wishlist/state.svelte.ts';

	const ROW = 56;
	const COLS = '30px minmax(0, 1fr) 110px 130px 28px 230px';
	const w$ = tr.wishlist;
	const wishStores = ['steam', 'gog'] as const;
	const statuses: [WishFilter, string][] = [
		['all', w$.statusAll],
		['held_here', w$.statusHeldHere],
		['unconfirmed', w$.statusUnconfirmed],
		['untitled', w$.statusUntitled]
	];
	const sorts: [WishSort, string][] = [
		['priority', w$.sortPriority],
		['added', w$.sortAdded],
		['title', w$.sortTitle]
	];

	let entries = $state<WishlistEntry[]>([]);
	let library = $state<Entry[]>([]);
	let status = $state<SyncStatus[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let scrollTop = $state(0);
	let viewport = $state(480);
	let selected = $state<string | null>(null);
	/** Stores whose wishlist writing is validated, from each connection's real capability. */
	let writeState = $state<Record<string, string>>({});

	const keyOf = (w: WishlistEntry) => `${w.store}:${w.account_id}:${w.native_id}`;
	const shown = $derived(applyWishFilters(entries, wishlistUi.filters));
	const view = $derived(wishView(entries, shown, status));
	const range = $derived(visibleRange(scrollTop, viewport, ROW, shown.length));
	const slice = $derived(shown.slice(range.start, range.end));
	const sel = $derived(entries.find((w) => keyOf(w) === selected) ?? null);
	const perStore = $derived(
		wishStores.map((s) => [s, entries.filter((w) => w.store === s).length] as const)
	);
	const unvalidated = $derived(wishStores.filter((s) => writeState[s] !== 'validated'));

	/** The library row of the same game, to show where you already have it. */
	const held = $derived(new Map(buildShelf(library, entries).map((r) => [r.key, r])));
	const rowOf = (w: WishlistEntry): Shelf | undefined =>
		held.get(shelfKey('game', w.title, w.native_id));

	function slotsOf(w: WishlistEntry): Record<Store, Slot> {
		const base = rowOf(w)?.slots ?? { steam: 'none', gog: 'none', epic: 'none' };
		const slots = { ...base };
		const s = w.store as Store;
		// In Wishlists the wished store always reads as a wish; the others show what you hold.
		slots[s] = 'wish';
		return slots;
	}
	const ownedElsewhere = (w: WishlistEntry) => {
		const r = rowOf(w);
		return (
			!!r &&
			(Object.entries(r.slots) as [Store, Slot][]).some(([s, v]) => s !== w.store && v === 'own')
		);
	};
	const titleOf = (w: WishlistEntry) => w.title ?? tr.library.untitled(w.native_id);
	const flags = (w: WishlistEntry) => [
		...(w.already_held_here ? [w$.heldHere] : []),
		...(!w.confirmed ? [w$.unconfirmed] : [])
	];

	const notices = $derived(
		status
			.filter((s) => s.resource === 'wishlist')
			.map((s) => {
				const name = storeName(s.store);
				if (s.last_complete_at === null)
					return { warn: true, text: tr.library.neverComplete(name) };
				if (s.last_state !== 'complete')
					return { warn: true, text: tr.library.lastFailed(name, when(s.last_complete_at)) };
				return { warn: false, text: tr.library.dataFrom(name, when(s.last_complete_at)) };
			})
	);
	const about = $derived([
		...(unvalidated.length < wishStores.length ? [w$.writeValidated] : []),
		...(unvalidated.length > 0
			? [w$.writeNotValidated(unvalidated.map((s) => storeName(s)).join(' y '))]
			: []),
		w$.epicUnavailable
	]);

	function detailProducts(w: WishlistEntry) {
		const r = rowOf(w);
		const own = (r?.products ?? []).map((p) => ({
			...p,
			note: p.access
				.map((a) => tr.review.access[a.class as keyof typeof tr.review.access] ?? a.class)
				.join(', ')
		}));
		return [{ ...w, note: tr.shelf.inWishlist }, ...own];
	}

	async function load() {
		try {
			const [w, st, conns, lib] = await Promise.all([
				ipc.wishlist(),
				ipc.syncStatus(),
				ipc.connections(),
				ipc.library().catch(() => [])
			]);
			[entries, status, library] = [w, st, lib];
			writeState = Object.fromEntries(
				conns.map((c) => [
					c.provider,
					c.capabilities.find((x) => x.capability === 'wishlist_write')?.state ?? ''
				])
			);
		} catch {
			loadFailed = true;
		} finally {
			loaded = true;
		}
	}

	onMount(load);
</script>

<div class="page" class:withDetail={sel}>
	<section class="main">
		<PageHeader title={tr.nav.wishlists}>
			{#snippet info()}
				<InfoTip label={tr.nav.wishlists} lines={about} />
			{/snippet}
			{#snippet figures()}
				<div>
					<dt>{tr.shell.wishes}</dt>
					<dd>{entries.length.toLocaleString(intl)}</dd>
				</div>
				{#each perStore as [x, n] (x)}
					<div>
						<dt><StoreLogo store={x} size={12} /> {storeName(x)}</dt>
						<dd>{n.toLocaleString(intl)}</dd>
					</div>
				{/each}
			{/snippet}
			{#snippet action()}
				{#if shell.toReview > 0}
					<a href={resolve('/limpieza')} class={button({ tone: 'next' })}
						>{tr.shell.toReview(shell.toReview)} <IconArrowRight size={16} /></a
					>
				{/if}
			{/snippet}
		</PageHeader>

		{#if loadFailed}
			<p role="alert" class="page-body text-danger">{tr.library.loadError}</p>
		{:else if loaded}
			<form class="toolbar" role="search" onsubmit={(e) => e.preventDefault()}>
				<label class="control control-search">
					<IconSearch size={16} />
					<span class="sr-only">{w$.search}</span>
					<input type="search" placeholder="{w$.search}…" bind:value={wishlistUi.filters.text} />
				</label>
				<select class="control" aria-label={w$.status} bind:value={wishlistUi.filters.status}>
					{#each statuses as [value, label] (value)}<option {value}>{w$.status}: {label}</option
						>{/each}
				</select>
				<select class="control" aria-label={w$.sort} bind:value={wishlistUi.filters.sort}>
					{#each sorts as [value, label] (value)}<option {value}>{w$.sort}: {label}</option>{/each}
				</select>
				<button
					type="button"
					class="control icon"
					disabled={!isWishFiltering(wishlistUi.filters)}
					aria-label={tr.library.clear}
					title={tr.library.clear}
					onclick={() => (wishlistUi.filters = { ...noWishFilters, sort: wishlistUi.filters.sort })}
					><IconFilterOff size={16} /></button
				>
				<div class="push segmented" role="group" aria-label={tr.library.store}>
					{#each wishStores as x (x)}
						<button
							type="button"
							class="control store {x}"
							aria-pressed={wishlistUi.filters.store === x}
							onclick={() => (wishlistUi.filters.store = wishlistUi.filters.store === x ? '' : x)}
							title={storeName(x)}
							><StoreLogo store={x} size={14} /><span class="store-label">{storeName(x)}</span
							></button
						>
					{/each}
				</div>
				<button
					type="button"
					class="control"
					onclick={() => openUrl(storeWishlistUrl.epic).catch(() => {})}
					title={w$.openEpic}
					aria-label={w$.openEpic}
					><StoreLogo store="epic" size={14} /><IconExternalLink size={14} /></button
				>
			</form>

			<div class="shelf-status">
				<p aria-live="polite">{tr.library.showing(shown.length, entries.length)}</p>
				<span class="push"></span>
				<InfoTip
					label={tr.shelf.dataInfo}
					lines={notices.map((n) => n.text)}
					warn={notices.some((n) => n.warn)}
				/>
			</div>

			{#if view === 'no_data'}
				<div class="empty">
					<p class="headline">{w$.noData}</p>
					<p>{w$.noDataBody}</p>
					<a href={resolve('/conexiones')} class={button({ tone: 'primary' })}
						>{tr.nav.conexiones}</a
					>
				</div>
			{:else if view === 'sync_failed'}
				<div class="empty" role="alert">
					<p class="headline">{w$.syncFailed}</p>
					<p>{w$.syncFailedBody}</p>
				</div>
			{:else if view === 'confirmed_empty'}
				<div class="empty">
					<p class="headline">{w$.confirmedEmpty}</p>
					<p>{w$.confirmedEmptyBody}</p>
				</div>
			{:else if view === 'no_match'}
				<div class="empty">
					<p class="headline">{w$.noMatch}</p>
				</div>
			{:else}
				<div class="shelf" style="--cols: {COLS}">
					<div class="shelf-head" aria-hidden="true">
						<span></span><span>{tr.shelf.title}</span><span class="wide-only"
							>{w$.sortPriority}</span
						><span class="wide-only">{w$.sortAdded}</span><span></span><span class="text-right"
							>{tr.shelf.stores}</span
						>
					</div>
					<!-- A scrollable region must be reachable with the keyboard (WCAG 2.1.1). -->
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<div
						class="shelf-scroll"
						tabindex="0"
						role="region"
						aria-label={tr.nav.wishlists}
						bind:clientHeight={viewport}
						onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
					>
						<ul
							style="padding-top: {range.start * ROW}px; padding-bottom: {(shown.length -
								range.end) *
								ROW}px"
						>
							{#each slice as w (keyOf(w))}
								{@const t = titleOf(w)}
								{@const repe = ownedElsewhere(w)}
								<li>
									<button
										type="button"
										class="shelf-row"
										class:repe
										aria-current={selected === keyOf(w) ? 'true' : undefined}
										onclick={() => (selected = selected === keyOf(w) ? null : keyOf(w))}
									>
										<Cover title={t} store={w.store as Store} src={w.image_url} />
										<span class="shelf-title">{t}</span>
										<span class="shelf-muted wide-only"
											>{w.priority === null ? '—' : w$.priority(w.priority)}</span
										>
										<span class="shelf-muted wide-only"
											>{w.added_at === null ? '—' : day(w.added_at)}</span
										>
										<span>
											{#if flags(w).length}<span class="flag" title={flags(w).join('. ')}
													><IconAlertTriangle size={16} /><span class="sr-only"
														>{flags(w).join('. ')}</span
													></span
												>{/if}
										</span>
										<StoreTabs slots={slotsOf(w)} title={t} wishedElsewhere={repe} />
									</button>
								</li>
							{/each}
						</ul>
					</div>
				</div>
			{/if}
			<div class="foot"><ExportButtons what="wishlist" label={w$.export} /></div>
		{/if}
	</section>

	{#if sel}
		{@const t = titleOf(sel)}
		<DetailPanel
			title={t}
			store={sel.store as Store}
			cover={sel.image_url}
			slots={slotsOf(sel)}
			wishedElsewhere={ownedElsewhere(sel)}
			products={detailProducts(sel)}
			onclose={() => (selected = null)}
		>
			{#snippet spec()}
				<dt>{tr.library.store}</dt>
				<dd>{storeName(sel.store)}</dd>
				{#if sel.priority !== null}<dt>{w$.sortPriority}</dt>
					<dd>{sel.priority}</dd>{/if}
				{#if sel.added_at !== null}<dt>{w$.sortAdded}</dt>
					<dd>{day(sel.added_at)}</dd>{/if}
				{#if flags(sel).length}<dt>{w$.status}</dt>
					<dd>{flags(sel).join('. ')}</dd>{/if}
			{/snippet}
			<PlayField products={[sel]} />
			{#snippet actions()}
				{#if ownedElsewhere(sel)}
					<a href={resolve('/limpieza')} class={button({ tone: 'next' })}>{tr.shelf.reviewThis}</a>
				{/if}
			{/snippet}
		</DetailPanel>
	{/if}
</div>

<style>
	.page {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
	}
	.page {
		position: relative;
	}
	.page.withDetail {
		grid-template-columns: minmax(0, 1fr) 440px;
	}
	/* Not wide enough for list and sheet side by side: the sheet lies over the list. */
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
		container-type: inline-size;
	}
	@container (max-width: 860px) {
		.shelf :global(.shelf-head),
		.shelf :global(.shelf-row) {
			grid-template-columns: 30px minmax(0, 1fr) 28px auto;
		}
	}
	.flag {
		display: inline-grid;
		color: var(--color-warning);
	}
	.shelf-status p {
		margin: 0;
	}
	.empty {
		display: grid;
		justify-items: start;
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
	.foot {
		padding: 12px 32px;
		border-top: 1px solid var(--color-border);
	}
</style>
