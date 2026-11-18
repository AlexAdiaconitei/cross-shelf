<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { IconArrowRight, IconExternalLink, IconSearch, IconX } from '@tabler/icons-svelte';
	import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
	import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import Cover from '#lib/components/store/Cover.svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { button, chip, input } from '#lib/components/ui/index.ts';
	import { kindName, productUrl, storeName, when } from '#lib/format.ts';
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
	import { buildShelf, firstStore, stores, type Shelf, type Store } from '#lib/library/shelf.ts';
	import { libraryUi } from '#lib/library/state.svelte.ts';
	import { shell } from '#lib/shell/state.svelte.ts';
	import { es } from '#lib/strings.ts';

	const ROW = 56;
	const s = es.shelf;

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

	// One line per store and resource that tells where the shown data really comes from.
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
		if (r.slots[first] === 'family') return s.onlyFamily(r.familyOwners.length);
		return es.library[
			r.slots[first] === 'own' ? 'owned' : r.slots[first] === 'temp' ? 'temporary' : 'unknownAccess'
		];
	}

	const accessName = (cls: string) => es.review.access[cls as keyof typeof es.review.access] ?? cls;

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

<svelte:window onkeydown={(e) => e.key === 'Escape' && (selected = null)} />

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
			<p role="alert" class="pad text-danger">{es.library.loadError}</p>
		{:else if loaded}
			<form class="bar" role="search" onsubmit={(e) => e.preventDefault()}>
				<label class="search">
					<IconSearch size={16} />
					<span class="sr-only">{es.library.search}</span>
					<input
						type="search"
						class={input({ class: 'border-0 px-0' })}
						placeholder="{es.library.search}…"
						bind:value={libraryUi.filters.text}
					/>
				</label>
				<div class="group" role="group" aria-label={es.library.scope}>
					{#each scopes as [k, label] (k)}
						<button
							type="button"
							class={chip()}
							aria-pressed={libraryUi.filters.scope === k}
							onclick={() => (libraryUi.filters.scope = k)}>{label}</button
						>
					{/each}
				</div>
				<label class="kind">
					<span class="sr-only">{es.library.kind}</span>
					<select bind:value={libraryUi.filters.kind}>
						<option value="">{es.library.allKinds}</option>
						{#each kinds as k (k)}<option value={k}>{kindName(k)}</option>{/each}
					</select>
				</label>
				{#if isFiltering(libraryUi.filters)}
					<button
						type="button"
						class={button({ tone: 'ghost', size: 'sm' })}
						onclick={() => (libraryUi.filters = { ...noFilters })}>{es.library.clear}</button
					>
				{/if}
				<div class="push sf" role="group" aria-label={es.library.store}>
					{#each stores as x (x)}
						<button
							type="button"
							class="sft {x}"
							aria-pressed={libraryUi.filters.store === x}
							onclick={() => toggleStore(x)}
							title={storeName(x)}
						>
							<StoreLogo store={x} size={14} /><span class="sfl">{storeName(x)}</span>
						</button>
					{/each}
				</div>
			</form>

			<div class="note">
				<ul aria-label={es.library.tabsLabel}>
					{#each notices as n (n.text)}<li class:warn={n.warn}>{n.text}</li>{/each}
				</ul>
				<div class="seg" role="group" aria-label={s.groupingLabel}>
					<button
						type="button"
						aria-pressed={libraryUi.grouped}
						onclick={() => (libraryUi.grouped = true)}>{s.grouped}</button
					>
					<button
						type="button"
						aria-pressed={!libraryUi.grouped}
						onclick={() => (libraryUi.grouped = false)}>{s.separate}</button
					>
				</div>
				<p aria-live="polite">{es.library.showing(count.shown, count.total)}</p>
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
				<div class="tbl">
					<div class="hd" aria-hidden="true">
						<span class="span2">{s.title}</span><span class="c-kind">{es.library.kind}</span><span
							class="c-acc">{s.access}</span
						><span class="r">{s.stores}</span>
					</div>
					<!-- A scrollable region must be reachable with the keyboard (WCAG 2.1.1). -->
					<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
					<div
						class="scroll"
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
										class="row"
										class:repe={r.wishedElsewhere}
										aria-current={selected === r.key ? 'true' : undefined}
										onclick={() => (selected = selected === r.key ? null : r.key)}
									>
										<Cover title={r.title} store={firstStore(r)} />
										<span class="t">{r.title}</span>
										<span class="c-kind muted">{kindName(r.kind)}</span>
										<span class="c-acc muted"
											>{accessLine(r)}{#if r.wishedElsewhere}<b>
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
			<div class="pad foot">
				<ExportButtons what="library" label={es.library.export} />
				{#if libraryUi.grouped}<p class="muted">{s.groupNote}</p>{/if}
			</div>
		{/if}
	</section>

	{#if sel}
		<aside class="detail" aria-label={s.detailOf(sel.title)}>
			<div class="dh">
				<p class="label">{s.detail}</p>
				<button
					type="button"
					class={button({ size: 'icon' })}
					onclick={() => (selected = null)}
					aria-label={s.closeDetail}><IconX size={18} /></button
				>
			</div>
			<div class="hero">
				<Cover title={sel.title} store={firstStore(sel)} size="lg" />
				<div class="hx">
					<h2 class="headline">{sel.title}</h2>
					<StoreTabs
						slots={sel.slots}
						title={sel.title}
						wishedElsewhere={sel.wishedElsewhere}
						size="lg"
					/>
				</div>
			</div>
			<dl class="spec">
				<dt>{es.library.kind}</dt>
				<dd>{kindName(sel.kind)}</dd>
				<dt>{s.access}</dt>
				<dd>
					{stores
						.filter((x) => sel.slots[x] !== 'none')
						.map((x) => `${storeName(x)}: ${s.slot[sel.slots[x]]}`)
						.join(' · ')}
				</dd>
				{#if sel.familyOwners.length}
					<dt>{s.family}</dt>
					<dd>{s.familyNote(sel.familyOwners.length)}</dd>
				{/if}
			</dl>
			<p class="label">{s.products}</p>
			<ul class="prods">
				{#each sel.products as p (p.store + p.native_id)}
					<li class={p.store}>
						<span class="pt"><StoreLogo store={p.store} size={14} /></span>
						<div>
							<b>{p.title ?? es.library.untitled(p.native_id)}</b>
							<small>{p.access.map((a) => accessName(a.class)).join(', ')}</small>
						</div>
						<button
							type="button"
							class={button({ size: 'sm' })}
							aria-label={s.openIn(storeName(p.store))}
							onclick={() => openUrl(productUrl(p.store, p.native_id, p.url)).catch(() => {})}
							>{s.open} <IconExternalLink size={14} /></button
						>
					</li>
				{/each}
				{#each sel.wished as w (w.store + w.native_id)}
					<li class={w.store}>
						<span class="pt"><StoreLogo store={w.store} size={14} /></span>
						<div>
							<b>{w.title ?? es.library.untitled(w.native_id)}</b><small>{s.inWishlist}</small>
						</div>
						<button
							type="button"
							class={button({ size: 'sm' })}
							aria-label={s.openIn(storeName(w.store))}
							onclick={() => openUrl(productUrl(w.store, w.native_id, w.url)).catch(() => {})}
							>{s.open} <IconExternalLink size={14} /></button
						>
					</li>
				{/each}
			</ul>
			{#if sel.wishedElsewhere}
				<a href={resolve('/limpieza')} class={button({ tone: 'next', class: 'self-start' })}
					>{s.reviewThis}</a
				>
			{/if}
		</aside>
	{/if}
</div>

<style>
	.page {
		flex: 1;
		min-height: 0;
		display: grid;
		grid-template-columns: minmax(0, 1fr);
	}
	.page.withDetail {
		grid-template-columns: minmax(0, 1fr) 440px;
	}
	.main {
		min-width: 0;
		min-height: 0;
		display: flex;
		flex-direction: column;
		container-type: inline-size;
	}
	.pad {
		padding: 16px 32px;
	}
	.bar {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
		padding: 12px 32px;
		border-bottom: 1px solid var(--color-border);
	}
	.search {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 300px;
		padding: 0 10px;
		border: 2px solid var(--color-text);
	}
	.search input {
		min-height: 32px;
		outline: none;
	}
	.group {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.kind select {
		height: 32px;
		border: 1px solid var(--color-z5);
		background: var(--color-page);
		padding: 0 6px;
	}
	.push {
		margin-left: auto;
	}
	.sf {
		display: flex;
		gap: 3px;
	}
	.sft {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 30px;
		padding: 0 10px;
		border: 0;
		background: var(--color-raised);
		color: var(--color-muted);
		font-size: 12px;
		font-weight: 800;
		text-transform: uppercase;
	}
	.sft.steam[aria-pressed='true'] {
		background: var(--color-steam);
		color: var(--color-on-store);
	}
	.sft.gog[aria-pressed='true'] {
		background: var(--color-gog);
		color: var(--color-on-store);
	}
	.sft.epic[aria-pressed='true'] {
		background: var(--color-epic);
		color: var(--color-on-epic);
	}
	.note {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 18px;
		padding: 8px 32px;
		font-size: 12px;
		color: var(--color-muted);
		background: var(--color-panel);
	}
	.note ul {
		display: flex;
		flex-wrap: wrap;
		gap: 2px 18px;
		margin: 0;
		padding: 0;
		list-style: none;
		flex: 1;
	}
	.note .warn {
		color: var(--color-warning);
		font-weight: 700;
	}
	.note p {
		margin: 0;
	}
	.seg {
		display: flex;
		border: 1.5px solid var(--color-text);
	}
	.seg button {
		border: 0;
		background: none;
		padding: 3px 10px;
		font-size: 12px;
		font-weight: 700;
		color: var(--color-text);
	}
	.seg button[aria-pressed='true'] {
		background: var(--color-text);
		color: var(--color-page);
	}
	.tbl {
		flex: 1;
		min-height: 260px;
		display: flex;
		flex-direction: column;
		padding: 0 32px;
	}
	.hd,
	.row {
		display: grid;
		grid-template-columns: 30px minmax(0, 1fr) 130px 240px 230px;
		gap: 14px;
		align-items: center;
	}
	.span2 {
		grid-column: span 2;
	}
	.hd {
		height: 34px;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.08em;
		color: var(--color-muted);
		border-bottom: 2px solid var(--color-border-strong);
	}
	.r {
		text-align: right;
	}
	.scroll {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
	}
	.scroll ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.row {
		width: 100%;
		height: 56px;
		border: 0;
		border-bottom: 1px solid var(--color-border);
		background: none;
		color: inherit;
		text-align: left;
		font: inherit;
		transition: background-color 120ms ease-out;
	}
	.row:hover {
		background: var(--color-panel);
	}
	.row.repe {
		background: var(--color-next-wash);
	}
	.row[aria-current='true'] {
		background: var(--color-panel);
		box-shadow: inset 0 0 0 2px var(--color-border-strong);
	}
	.t {
		font-weight: 700;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.muted {
		color: var(--color-muted);
	}
	.c-acc b {
		color: var(--color-text);
	}
	@container (max-width: 860px) {
		.hd,
		.row {
			grid-template-columns: 30px minmax(0, 1fr) 210px;
		}
		.c-kind,
		.c-acc,
		.sfl {
			display: none;
		}
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
		display: flex;
		flex-wrap: wrap;
		gap: 8px 24px;
		align-items: center;
		border-top: 1px solid var(--color-border);
	}
	.foot p {
		margin: 0;
		font-size: 12px;
	}
	.detail {
		min-height: 0;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 18px 24px 40px;
		border-left: 3px solid var(--color-border-strong);
	}
	.dh {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.dh .label,
	.detail > .label {
		margin: 0;
	}
	.hero {
		display: grid;
		grid-template-columns: 180px 1fr;
		gap: 18px;
		align-items: end;
	}
	.hx {
		display: grid;
		gap: 12px;
		justify-items: start;
	}
	.hx :global(.tabs) {
		justify-content: flex-start;
	}
	.hx h2 {
		margin: 0;
		font-size: 38px;
		overflow-wrap: anywhere;
	}
	.spec {
		display: grid;
		grid-template-columns: 90px 1fr;
		margin: 0;
		border-top: 2px solid var(--color-border-strong);
	}
	.spec dt,
	.spec dd {
		margin: 0;
		padding: 9px 0;
		border-bottom: 1px solid var(--color-border);
	}
	.spec dt {
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--color-muted);
	}
	.prods {
		display: grid;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.prods li {
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 8px;
		background: var(--color-panel);
	}
	.prods div {
		flex: 1;
		min-width: 0;
		display: grid;
	}
	.prods small {
		color: var(--color-muted);
	}
	.pt {
		display: grid;
		place-items: center;
		flex: none;
		width: 32px;
		height: 32px;
		background: var(--color-steam);
		color: var(--color-on-store);
	}
	.gog .pt {
		background: var(--color-gog);
	}
	.epic .pt {
		background: var(--color-epic);
		color: var(--color-on-epic);
	}
</style>
