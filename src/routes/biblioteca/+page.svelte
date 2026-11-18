<script lang="ts">
	import { onMount } from 'svelte';
	import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import { Button } from '#lib/components/ui/index.ts';
	import { kindName, storeName, when } from '#lib/format.ts';
	import { ipc } from '#lib/ipc.ts';
	import { es } from '#lib/strings.ts';
	import {
		activeTab,
		applyFilters,
		tabPreset,
		tabs,
		counters,
		isFiltering,
		kindsPresent,
		noFilters,
		viewState,
		visibleRange,
		type Entry,
		type Scope
	} from '#lib/library/filter.ts';
	import { libraryUi } from '#lib/library/state.svelte.ts';

	const ROW = 64;
	const stores = ['steam', 'gog', 'epic'] as const;

	let entries = $state<Entry[]>([]);
	let status = $state<SyncStatus[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let scrollTop = $state(0);
	let viewport = $state(480);

	const shown = $derived(applyFilters(entries, libraryUi.filters));
	const count = $derived(counters(entries, shown));
	const view = $derived(viewState(entries, shown, status));
	const tab = $derived(activeTab(libraryUi.filters));
	const range = $derived(visibleRange(scrollTop, viewport, ROW, shown.length));
	const slice = $derived(shown.slice(range.start, range.end));
	const kinds = $derived(kindsPresent(entries));

	// One line per store and resource that tells where the shown data really comes from.
	const notices = $derived(
		status
			.filter((s) => s.resource !== 'wishlist')
			.map((s) => {
				const part = es.capabilities[s.resource as keyof typeof es.capabilities] ?? s.resource;
				const name = `${storeName(s.store)} (${part.toLowerCase()})`;
				if (s.last_complete_at === null)
					return { warn: true, text: es.library.neverComplete(name) };
				if (s.last_state !== 'complete')
					return { warn: true, text: es.library.lastFailed(name, when(s.last_complete_at)) };
				return { warn: false, text: es.library.dataFrom(name, when(s.last_complete_at)) };
			})
	);

	function accessLabel(a: Entry['access'][number]): string {
		switch (a.class) {
			case 'owned':
				return es.library.owned;
			case 'family_shared':
				return `${es.library.familyShared} ${es.library.owners(a.owners.length)}`;
			case 'temporary':
				return es.library.temporary;
			default:
				return es.library.unknownAccess;
		}
	}

	async function load() {
		try {
			[entries, status] = await Promise.all([ipc.library(), ipc.syncStatus()]);
		} catch {
			loadFailed = true;
		} finally {
			loaded = true;
		}
	}

	onMount(load);
</script>

<h1 class="mb-4 text-xl font-semibold">{es.nav.biblioteca}</h1>

{#if loadFailed}
	<p role="alert" class="text-danger">{es.library.loadError}</p>
{:else if loaded}
	<div class="mb-4 flex flex-wrap gap-1" role="group" aria-label={es.library.tabsLabel}>
		{#each tabs as t (t)}
			<button
				type="button"
				aria-pressed={tab === t}
				class="cursor-pointer rounded-control px-3 py-1.5 text-sm text-muted hover:bg-raised hover:text-text aria-pressed:bg-raised aria-pressed:text-text"
				onclick={() => Object.assign(libraryUi.filters, tabPreset(t))}>{es.library.tabs[t]}</button
			>
		{/each}
	</div>

	<form
		class="mb-3 flex flex-wrap items-end gap-3"
		role="search"
		onsubmit={(e) => e.preventDefault()}
	>
		<label class="grid gap-1 text-sm">
			<span class="text-muted">{es.library.search}</span>
			<input
				type="search"
				class="rounded-control border border-border bg-panel px-3 py-2"
				bind:value={libraryUi.filters.text}
			/>
		</label>
		<label class="grid gap-1 text-sm">
			<span class="text-muted">{es.library.store}</span>
			<select
				class="rounded-control border border-border bg-panel px-3 py-2"
				bind:value={libraryUi.filters.store}
			>
				<option value="">{es.library.allStores}</option>
				{#each stores as s (s)}<option value={s}>{storeName(s)}</option>{/each}
			</select>
		</label>
		<label class="grid gap-1 text-sm">
			<span class="text-muted">{es.library.scope}</span>
			<select
				class="rounded-control border border-border bg-panel px-3 py-2"
				bind:value={libraryUi.filters.scope}
			>
				{#each [['all', es.library.scopeAll], ['own', es.library.scopeOwn], ['family', es.library.scopeFamily], ['family_only', es.library.scopeFamilyOnly]] as [value, label] (value)}
					<option value={value as Scope}>{label}</option>
				{/each}
			</select>
		</label>
		<label class="grid gap-1 text-sm">
			<span class="text-muted">{es.library.kind}</span>
			<select
				class="rounded-control border border-border bg-panel px-3 py-2"
				bind:value={libraryUi.filters.kind}
			>
				<option value="">{es.library.allKinds}</option>
				{#each kinds as k (k)}<option value={k}>{kindName(k)}</option>{/each}
			</select>
		</label>
		{#if isFiltering(libraryUi.filters)}
			<Button onclick={() => (libraryUi.filters = { ...noFilters })}>{es.library.clear}</Button>
		{/if}
	</form>

	<p class="mb-1 text-sm text-muted" aria-live="polite">
		{es.library.showing(count.shown, count.total)} ·
		{stores
			.filter((s) => count.byStore[s])
			.map((s) => es.library.perStore(storeName(s), count.byStore[s] ?? 0))
			.join(' · ')}
		· {es.library.familyCount(count.familyAccess)}
	</p>
	<ul class="mb-3 grid gap-0.5 text-xs text-muted">
		{#each notices as n (n.text)}
			<li class={n.warn ? 'text-warning' : ''}>{n.text}</li>
		{/each}
	</ul>

	{#if view === 'sync_failed'}
		<div class="rounded-panel border border-border p-6" role="alert">
			<p class="font-medium text-warning">{es.library.syncFailed}</p>
			<p class="text-muted">{es.library.syncFailedBody}</p>
		</div>
	{:else if view === 'no_data'}
		<div class="rounded-panel border border-border p-6">
			<p class="font-medium">{es.library.noData}</p>
			<p class="text-muted">{es.library.noDataBody}</p>
		</div>
	{:else if view === 'no_match'}
		<div class="rounded-panel border border-border p-6">
			<p class="font-medium">{es.library.noMatch}</p>
		</div>
	{:else}
		<!-- A scrollable region must be reachable with the keyboard (WCAG 2.1.1), hence the tabindex. -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex -->
		<div
			class="h-[60vh] overflow-y-auto rounded-panel border border-border"
			tabindex="0"
			role="region"
			aria-label={es.nav.biblioteca}
			bind:clientHeight={viewport}
			onscroll={(e) => (scrollTop = e.currentTarget.scrollTop)}
		>
			<ul
				style="padding-top: {range.start * ROW}px; padding-bottom: {(shown.length - range.end) *
					ROW}px"
			>
				{#each slice as e (`${e.store}:${e.account_id}:${e.native_id}`)}
					<li class="flex h-16 items-center justify-between gap-4 border-b border-border px-4">
						<div class="min-w-0">
							<p class="truncate font-medium">
								{e.title ?? es.library.untitled(e.native_id)}
							</p>
							<p class="truncate text-sm text-muted">
								{storeName(e.store)} · {kindName(e.kind)}
							</p>
						</div>
						<ul class="flex flex-wrap justify-end gap-1 text-xs">
							{#each e.access as a (a.class)}
								<li class="rounded-control bg-raised px-2 py-1">{accessLabel(a)}</li>
							{/each}
						</ul>
					</li>
				{/each}
			</ul>
		</div>
	{/if}

	<div class="mt-4"><ExportButtons what="library" label={es.library.export} /></div>
{/if}
