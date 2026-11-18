<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { IconArrowRight, IconFilterOff, IconSearch } from '@tabler/icons-svelte';
	import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
	import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import Cover from '#lib/components/store/Cover.svelte';
	import DetailPanel from '#lib/components/store/DetailPanel.svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { InfoTip, button, chip } from '#lib/components/ui/index.ts';
	import { kindName, storeName, when } from '#lib/format.ts';
	import { ipc } from '#lib/ipc.ts';
	import {
		applyFilters,
		counters,
		isFiltering,
		kindsPresent,
		noFilters,
		viewState,
		visibleRange,
		type Entry,
		type Scope
	} from '#lib/library/filter.ts';
	import {
		buildShelf,
		coverOf,
		firstStore,
		stores,
		type Shelf,
		type Store
	} from '#lib/library/shelf.ts';
	import { libraryUi } from '#lib/library/state.svelte.ts';
	import { shell } from '#lib/shell/state.svelte.ts';
	import { es } from '#lib/strings.ts';

	const ROW = 56;
	const s = es.shelf;
	const COLS = '30px minmax(0, 1fr) 130px 240px 230px';

	let entries = $state<Entry[]>([]);
	let wishlist = $state<WishlistEntry[]>([]);
	let status = $state<SyncStatus[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let scrollTop = $state(0);
	let viewport = $state(480);
	let selected = $state<string | null>(null);

	const shown = $derived(applyFilters(entries, libraryUi.filters));
	const count = $derived(counters(entries, shown));
	const view = $derived(viewState(entries, shown, status));
	const kinds = $derived(kindsPresent(entries));
	const allRows = $derived(buildShelf(entries, wishlist));
	const rows = $derived(buildShelf(shown, wishlist));
	/** Separate records: one row per product, same row shape. */
	const records = $derived(
		shown.map((e) => ({
			...buildShelf(
				[e],
				wishlist.filter((w) => w.store !== e.store)
			)[0]!,
			key: `${e.store}:${e.account_id}:${e.native_id}`
		}))
	);
	const list = $derived(libraryUi.grouped ? rows : records);
	const range = $derived(visibleRange(scrollTop, viewport, ROW, list.length));
	const slice = $derived(list.slice(range.start, range.end));
	const sel = $derived(list.find((r) => r.key === selected) ?? null);

	const scopes: [Scope, string][] = [
		['all', es.library.scopeAll],
		['own', es.library.scopeOwn],
		['family', es.library.scopeFamily],
		['family_only', es.library.scopeFamilyOnly]
	];

	// Where the shown data really comes from, one line per store and resource, behind an icon.
	const notices = $derived(
		status
			.filter((x) => x.resource !== 'wishlist')
			.map((x) => {
				const part = es.capabilities[x.resource as keyof typeof es.capabilities] ?? x.resource;
				// Only the first letter: «Familia de Steam» keeps its proper noun.
				const name = `${storeName(x.store)} (${part.charAt(0).toLowerCase()}${part.slice(1)})`;
				if (x.last_complete_at === null)
					return { warn: true, text: es.library.neverComplete(name) };
				if (x.last_state !== 'complete')
					return { warn: true, text: es.library.lastFailed(name, when(x.last_complete_at)) };
				return { warn: false, text: es.library.dataFrom(name, when(x.last_complete_at)) };
			})
	);

	function accessLine(r: Shelf): string {
		const held = stores.filter((x) => r.slots[x] !== 'none' && r.slots[x] !== 'wish');
		const first = held[0];
		if (!first) return '';
		if (r.slots[first] === 'family') return s.onlyFamily(r.familyNames, r.familyOwners.length);
		return es.library[
			r.slots[first] === 'own' ? 'owned' : r.slots[first] === 'temp' ? 'temporary' : 'unknownAccess'
		];
	}

	const accessName = (cls: string) => es.review.access[cls as keyof typeof es.review.access] ?? cls;

	const detailProducts = (r: Shelf) => [
		...r.products.map((p) => ({ ...p, note: p.access.map((a) => accessName(a.class)).join(', ') })),
		...r.wished.map((w) => ({ ...w, note: s.inWishlist }))
	];

	function toggleStore(x: Store) {
		libraryUi.filters.store = libraryUi.filters.store === x ? '' : x;
	}

	async function load() {
		try {
			[entries, status, wishlist] = await Promise.all([
				ipc.library(),
				ipc.syncStatus(),
				ipc.wishlist().catch(() => [])
			]);
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
		<PageHeader title={es.nav.biblioteca}>
			{#snippet figures()}
				<div>
					<dt>{s.games}</dt>
					<dd>{allRows.length.toLocaleString('es')}</dd>
				</div>
				{#each stores as x (x)}
					<div>
						<dt><StoreLogo store={x} size={12} /> {storeName(x)}</dt>
						<dd>{(count.byStore[x] ?? 0).toLocaleString('es')}</dd>
					</div>
				{/each}
				<div>
					<dt>{s.family}</dt>
					<dd>{count.familyAccess.toLocaleString('es')}</dd>
				</div>
			{/snippet}
			{#snippet action()}
				{#if shell.toReview > 0}
					<a href={resolve('/limpieza')} class={button({ tone: 'next' })}
						>{es.shell.toReview(shell.toReview)} <IconArrowRight size={16} /></a
					>
				{/if}
			{/snippet}
		</PageHeader>

		{#if loadFailed}
			<p role="alert" class="page-body text-danger">{es.library.loadError}</p>
		{:else if loaded}
			<form class="toolbar" role="search" onsubmit={(e) => e.preventDefault()}>
				<label class="control control-search">
					<IconSearch size={16} />
					<span class="sr-only">{es.library.search}</span>
					<input
						type="search"
						placeholder="{es.library.search}…"
						bind:value={libraryUi.filters.text}
					/>
				</label>
				<div class="chips when-roomy" role="group" aria-label={es.library.scope}>
					{#each scopes as [k, label] (k)}
						<button
							type="button"
							class={chip()}
							aria-pressed={libraryUi.filters.scope === k}
							onclick={() => (libraryUi.filters.scope = k)}>{label}</button
						>
					{/each}
				</div>
				<select
					class="control when-tight"
					aria-label={es.library.scope}
					bind:value={libraryUi.filters.scope}
				>
					{#each scopes as [k, label] (k)}<option value={k}>{es.library.scope}: {label}</option
						>{/each}
				</select>
				<select class="control" aria-label={es.library.kind} bind:value={libraryUi.filters.kind}>
					<option value="">{es.library.allKinds}</option>
					{#each kinds as k (k)}<option value={k}>{kindName(k)}</option>{/each}
				</select>
				<button
					type="button"
					class="control icon"
					disabled={!isFiltering(libraryUi.filters)}
					aria-label={es.library.clear}
					title={es.library.clear}
					onclick={() => (libraryUi.filters = { ...noFilters })}><IconFilterOff size={16} /></button
				>
				<div class="push segmented" role="group" aria-label={es.library.store}>
					{#each stores as x (x)}
						<button
							type="button"
							class="control store {x}"
							aria-pressed={libraryUi.filters.store === x}
							onclick={() => toggleStore(x)}
							title={storeName(x)}
						>
							<StoreLogo store={x} size={14} /><span class="store-label">{storeName(x)}</span>
						</button>
					{/each}
				</div>
			</form>

			<div class="shelf-status">
				<div class="segmented" role="group" aria-label={s.groupingLabel}>
					<button
						type="button"
						class="control"
						aria-pressed={libraryUi.grouped}
						onclick={() => (libraryUi.grouped = true)}>{s.grouped}</button
					>
					<button
						type="button"
						class="control"
						aria-pressed={!libraryUi.grouped}
						onclick={() => (libraryUi.grouped = false)}>{s.separate}</button
					>
				</div>
				<InfoTip label={s.groupInfo} lines={[s.groupNote]} />
				<p class="push" aria-live="polite">{es.library.showing(count.shown, count.total)}</p>
				<InfoTip
					label={s.dataInfo}
					lines={notices.map((n) => n.text)}
					warn={notices.some((n) => n.warn)}
				/>
			</div>

			{#if view === 'sync_failed'}
				<div class="empty" role="alert">
					<p class="headline">{es.library.syncFailed}</p>
					<p>{es.library.syncFailedBody}</p>
					<a href={resolve('/conexiones')} class={button()}>{es.nav.conexiones}</a>
				</div>
			{:else if view === 'no_data'}
				<div class="empty">
					<p class="headline">{es.library.noData}</p>
					<p>{es.library.noDataBody}</p>
					<a href={resolve('/conexiones')} class={button({ tone: 'primary' })}
						>{es.nav.conexiones}</a
					>
				</div>
			{:else if view === 'no_match'}
				<div class="empty">
					<p class="headline">{es.library.noMatch}</p>
					<button
						type="button"
						class={button()}
						onclick={() => (libraryUi.filters = { ...noFilters })}>{es.library.clear}</button
					>
				</div>
			{:else}
				<div class="shelf" style="--cols: {COLS}">
					<div class="shelf-head" aria-hidden="true">
						<span></span><span>{s.title}</span><span class="wide-only">{es.library.kind}</span><span
							class="wide-only">{s.access}</span
						><span class="text-right">{s.stores}</span>
					</div>
					<!-- A scrollable region must be reachable with the keyboard (WCAG 2.1.1). -->
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<div
						class="shelf-scroll"
						tabindex="0"
						role="region"
						aria-label={es.nav.biblioteca}
						bind:clientHeight={viewport}
						onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
					>
						<ul
							style="padding-top: {range.start * ROW}px; padding-bottom: {(list.length -
								range.end) *
								ROW}px"
						>
							{#each slice as r (r.key)}
								<li>
									<button
										type="button"
										class="shelf-row"
										class:repe={r.wishedElsewhere}
										aria-current={selected === r.key ? 'true' : undefined}
										onclick={() => (selected = selected === r.key ? null : r.key)}
									>
										<Cover title={r.title} store={firstStore(r)} src={coverOf(r)} />
										<span class="shelf-title">{r.title}</span>
										<span class="shelf-muted wide-only">{kindName(r.kind)}</span>
										<span class="shelf-muted wide-only"
											>{accessLine(r)}{#if r.wishedElsewhere}<b class="text-text">
													· {s.wishedElsewhere}</b
												>{/if}</span
										>
										<StoreTabs
											slots={r.slots}
											title={r.title}
											wishedElsewhere={r.wishedElsewhere}
										/>
									</button>
								</li>
							{/each}
						</ul>
					</div>
				</div>
			{/if}
			<div class="foot"><ExportButtons what="library" label={es.library.export} /></div>
		{/if}
	</section>

	{#if sel}
		<DetailPanel
			title={sel.title}
			store={firstStore(sel)}
			cover={coverOf(sel)}
			slots={sel.slots}
			wishedElsewhere={sel.wishedElsewhere}
			products={detailProducts(sel)}
			onclose={() => (selected = null)}
		>
			{#snippet spec()}
				<dt>{es.library.kind}</dt>
				<dd>{kindName(sel.kind)}</dd>
				{#if sel.familyOwners.length}
					<dt>{s.family}</dt>
					<dd>{s.sharedWith(sel.familyNames, sel.familyOwners.length)}</dd>
				{/if}
			{/snippet}
			{#snippet actions()}
				{#if sel.wishedElsewhere}
					<a href={resolve('/limpieza')} class={button({ tone: 'next' })}>{s.reviewThis}</a>
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
	/* display comes from .when-roomy in app.css, so the toolbar can swap chips for a select. */
	.chips {
		gap: 6px;
	}
	@container (max-width: 860px) {
		.shelf :global(.shelf-head),
		.shelf :global(.shelf-row) {
			grid-template-columns: 30px minmax(0, 1fr) auto;
		}
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
