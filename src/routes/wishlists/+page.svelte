<script lang="ts">
	import PageHeader from '#lib/components/PageHeader.svelte';
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
	import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import { Button } from '#lib/components/ui/index.ts';
	import { day, productUrl, storeName, storeWishlistUrl, when } from '#lib/format.ts';
	import { ipc } from '#lib/ipc.ts';
	import { visibleRange } from '#lib/library/filter.ts';
	import { es } from '#lib/strings.ts';
	import {
		applyWishFilters,
		isWishFiltering,
		noWishFilters,
		wishView,
		type WishFilter,
		type WishSort
	} from '#lib/wishlist/filter.ts';
	import { wishlistUi } from '#lib/wishlist/state.svelte.ts';

	const ROW = 72;
	const stores = ['steam', 'gog'] as const;
	const statuses: [WishFilter, string][] = [
		['all', es.wishlist.statusAll],
		['held_here', es.wishlist.statusHeldHere],
		['unconfirmed', es.wishlist.statusUnconfirmed],
		['untitled', es.wishlist.statusUntitled]
	];
	const sorts: [WishSort, string][] = [
		['priority', es.wishlist.sortPriority],
		['added', es.wishlist.sortAdded],
		['title', es.wishlist.sortTitle]
	];

	let entries = $state<WishlistEntry[]>([]);
	let status = $state<SyncStatus[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let scrollTop = $state(0);
	let viewport = $state(480);

	const shown = $derived(applyWishFilters(entries, wishlistUi.filters));
	const view = $derived(wishView(entries, shown, status));
	const range = $derived(visibleRange(scrollTop, viewport, ROW, shown.length));
	const slice = $derived(shown.slice(range.start, range.end));
	const perStore = $derived(
		stores.map((s) => [s, entries.filter((w) => w.store === s).length] as const)
	);

	const notices = $derived(
		status
			.filter((s) => s.resource === 'wishlist')
			.map((s) => {
				const name = storeName(s.store);
				if (s.last_complete_at === null)
					return { warn: true, text: es.library.neverComplete(name) };
				if (s.last_state !== 'complete')
					return { warn: true, text: es.library.lastFailed(name, when(s.last_complete_at)) };
				return { warn: false, text: es.library.dataFrom(name, when(s.last_complete_at)) };
			})
	);

	/** Stores whose wishlist writing is validated, from each connection's real capability. */
	let writeState = $state<Record<string, string>>({});
	const unvalidated = $derived(stores.filter((s) => writeState[s] !== 'validated'));

	async function load() {
		try {
			const [w, st, conns] = await Promise.all([
				ipc.wishlist(),
				ipc.syncStatus(),
				ipc.connections()
			]);
			[entries, status] = [w, st];
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

	const open = (url: string) => openUrl(url).catch(() => {});

	onMount(load);
</script>

<PageHeader title={es.nav.wishlists} />
<div class="page-body">
	<aside class="mb-4 grid max-w-prose gap-2 text-sm text-muted">
		{#if loaded && unvalidated.length < stores.length}
			<p>
				{es.wishlist.writeValidated}
				<a class="text-accent underline underline-offset-2" href={resolve('/limpieza')}
					>{es.nav.limpieza}</a
				>
			</p>
		{/if}
		{#if loaded && unvalidated.length > 0}
			<p>{es.wishlist.writeNotValidated(unvalidated.map((s) => storeName(s)).join(' y '))}</p>
		{/if}
		<p>
			{es.wishlist.epicUnavailable}
			<button
				type="button"
				class="cursor-pointer text-accent underline underline-offset-2"
				onclick={() => open(storeWishlistUrl.epic)}>{es.wishlist.openStore} (Epic)</button
			>
		</p>
	</aside>

	{#if loadFailed}
		<p role="alert" class="text-danger">{es.library.loadError}</p>
	{:else if loaded}
		<form
			class="mb-3 flex flex-wrap items-end gap-3"
			role="search"
			onsubmit={(e) => e.preventDefault()}
		>
			<label class="grid gap-1 text-sm">
				<span class="text-muted">{es.wishlist.search}</span>
				<input
					type="search"
					class="rounded-control border border-border bg-panel px-3 py-2"
					bind:value={wishlistUi.filters.text}
				/>
			</label>
			<label class="grid gap-1 text-sm">
				<span class="text-muted">{es.library.store}</span>
				<select
					class="rounded-control border border-border bg-panel px-3 py-2"
					bind:value={wishlistUi.filters.store}
				>
					<option value="">{es.library.allStores}</option>
					{#each stores as s (s)}<option value={s}>{storeName(s)}</option>{/each}
				</select>
			</label>
			<label class="grid gap-1 text-sm">
				<span class="text-muted">{es.wishlist.status}</span>
				<select
					class="rounded-control border border-border bg-panel px-3 py-2"
					bind:value={wishlistUi.filters.status}
				>
					{#each statuses as [value, label] (value)}<option {value}>{label}</option>{/each}
				</select>
			</label>
			<label class="grid gap-1 text-sm">
				<span class="text-muted">{es.wishlist.sort}</span>
				<select
					class="rounded-control border border-border bg-panel px-3 py-2"
					bind:value={wishlistUi.filters.sort}
				>
					{#each sorts as [value, label] (value)}<option {value}>{label}</option>{/each}
				</select>
			</label>
			{#if isWishFiltering(wishlistUi.filters)}
				<Button
					onclick={() => (wishlistUi.filters = { ...noWishFilters, sort: wishlistUi.filters.sort })}
					>{es.library.clear}</Button
				>
			{/if}
		</form>

		<p class="mb-1 text-sm text-muted" aria-live="polite">
			{es.library.showing(shown.length, entries.length)} ·
			{perStore
				.filter(([, n]) => n > 0)
				.map(([s, n]) => es.library.perStore(storeName(s), n))
				.join(' · ')}
		</p>
		<ul class="mb-3 grid gap-0.5 text-xs text-muted">
			{#each notices as n (n.text)}
				<li class={n.warn ? 'text-warning' : ''}>{n.text}</li>
			{/each}
		</ul>

		{#if view === 'no_data'}
			<div class="rounded-panel border border-border p-6">
				<p class="font-medium">{es.wishlist.noData}</p>
				<p class="text-muted">{es.wishlist.noDataBody}</p>
			</div>
		{:else if view === 'sync_failed'}
			<div class="rounded-panel border border-border p-6" role="alert">
				<p class="font-medium text-warning">{es.wishlist.syncFailed}</p>
				<p class="text-muted">{es.wishlist.syncFailedBody}</p>
			</div>
		{:else if view === 'confirmed_empty'}
			<div class="rounded-panel border border-border p-6">
				<p class="font-medium">{es.wishlist.confirmedEmpty}</p>
				<p class="text-muted">{es.wishlist.confirmedEmptyBody}</p>
			</div>
		{:else if view === 'no_match'}
			<div class="rounded-panel border border-border p-6">
				<p class="font-medium">{es.wishlist.noMatch}</p>
			</div>
		{:else}
			<!-- A scrollable region must be reachable with the keyboard (WCAG 2.1.1), hence the tabindex. -->
			<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
			<div
				class="h-[60vh] overflow-y-auto rounded-panel border border-border"
				tabindex="0"
				role="region"
				aria-label={es.nav.wishlists}
				bind:clientHeight={viewport}
				onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
			>
				<ul
					style="padding-top: {range.start * ROW}px; padding-bottom: {(shown.length - range.end) *
						ROW}px"
				>
					{#each slice as w (`${w.store}:${w.account_id}:${w.native_id}`)}
						<li class="flex h-18 items-center justify-between gap-4 border-b border-border px-4">
							<div class="min-w-0">
								<p class="truncate font-medium">{w.title ?? es.library.untitled(w.native_id)}</p>
								<p class="truncate text-sm text-muted">
									{storeName(w.store)}
									{#if w.priority !== null}· {es.wishlist.priority(w.priority)}{/if}
									{#if w.added_at !== null}· {es.wishlist.added(day(w.added_at))}{/if}
								</p>
								{#if w.already_held_here || !w.confirmed}
									<p class="truncate text-xs text-warning">
										{w.already_held_here ? es.wishlist.heldHere : ''}
										{w.already_held_here && !w.confirmed ? ' · ' : ''}
										{!w.confirmed ? es.wishlist.unconfirmed : ''}
									</p>
								{/if}
							</div>
							<Button size="sm" onclick={() => open(productUrl(w.store, w.native_id, w.url))}
								>{es.wishlist.openStore}</Button
							>
						</li>
					{/each}
				</ul>
			</div>
		{/if}

		<div class="mt-4"><ExportButtons what="wishlist" label={es.wishlist.export} /></div>
	{/if}
</div>
