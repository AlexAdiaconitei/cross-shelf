<script lang="ts">
	import { onMount } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { invoke } from '@tauri-apps/api/core';
	import type { BatchView } from '#lib/bindings/BatchView.ts';
	import type { Decision } from '#lib/bindings/Decision.ts';
	import type { MatchView } from '#lib/bindings/MatchView.ts';
	import type { ReviewItem } from '#lib/bindings/ReviewItem.ts';
	import { Button, Dialog } from '#lib/components/ui/index.ts';
	import { asCommandError, ipc } from '#lib/ipc.ts';
	import { day, kindName, storeName } from '#lib/format.ts';
	import {
		applyReview,
		coveringMatch,
		groupSelection,
		itemKey,
		removable,
		reviewCounts,
		sortMatches,
		type ReviewFilter
	} from '#lib/review/view.ts';
	import { es } from '#lib/strings.ts';

	const filters: ReviewFilter[] = ['to_review', 'kept', 'no_match', 'repeated', 'all'];
	const r = es.review;

	let items = $state<ReviewItem[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let filter = $state<ReviewFilter>('to_review');
	let text = $state('');
	let note = $state('');

	const selected = new SvelteSet<string>();
	let drafts = $state<BatchView[]>([]);
	let batchOpen = $state(false);
	let batchNote = $state('');
	let confirming = $state(false);
	/** Display name of each store's active account, so the dialog names it instead of an ID. */
	let accountNames = $state<Record<string, string | null>>({});

	const shown = $derived(applyReview(items, filter, text));
	const groups = $derived(groupSelection(items, selected));
	const selectedCount = $derived(groups.reduce((n, g) => n + g.items.length, 0));
	const summary = $derived(
		es.batch.summary(
			groups.map((g) => es.batch.part(g.items.length, storeName(g.store))).join(' y ')
		)
	);
	const b = es.batch;
	const removableItems = $derived(items.filter(removable));
	/** Why each selected entry is safe to remove, keyed like the batch operations. */
	const evidenceFor = $derived(
		new Map(
			groups.flatMap((g) =>
				g.items.map((it) => {
					const m = coveringMatch(it);
					const why = m?.evidence ? label(r.evidence, m.evidence) : label(r.outcome, 'exact');
					const line = m ? b.evidence(storeName(m.held.store), why) : '';
					return [`${g.store}:${it.wanted.native_id}`, line] as const;
				})
			)
		)
	);
	const toRemove = $derived(drafts.reduce((n, d) => n + d.ops.length, 0));

	const batchError = (e: unknown) => {
		const code = typeof e === 'string' ? e : asCommandError(e).code;
		return b.errors[code as keyof typeof b.errors] ?? b.errors.internal;
	};

	function selectAllExact() {
		for (const it of removableItems) selected.add(itemKey(it));
	}

	/** Drafts are created first, so the dialog shows exactly what will be confirmed. */
	async function openBatch() {
		batchNote = '';
		const made: BatchView[] = [];
		try {
			accountNames = Object.fromEntries(
				(await ipc.connections()).map((c) => [c.provider, c.display_name])
			);
			for (const g of groups) {
				made.push(
					await ipc.planCreate(
						g.store,
						g.account_id,
						g.items.map((i) => i.wanted.native_id)
					)
				);
			}
			drafts = made;
			batchOpen = true;
		} catch (e) {
			for (const d of made) await ipc.planDiscard(d.id).catch(() => {});
			batchNote = batchError(e);
		}
	}

	async function cancelBatch() {
		batchOpen = false;
		for (const d of drafts) await ipc.planDiscard(d.id).catch(() => {});
		drafts = [];
	}

	/**
	 * One step: confirm each batch and send it. Each entry is removed and checked by reading the
	 * wishlist back; Resultados shows what happened to every one of them.
	 */
	async function confirmBatch() {
		if (confirming) return;
		confirming = true;
		let reached = false;
		try {
			for (const d of drafts) {
				const c = await ipc.planConfirm(d.id, d.revision);
				reached = true;
				if (c.write_validated) {
					await ipc.planExecute(c.id);
					if (c.store === 'steam' || c.store === 'gog') await ipc.sync(c.store);
				}
			}
			batchOpen = false;
			drafts = [];
			selected.clear();
			await goto(resolve('/resultados'));
		} catch (e) {
			batchNote = batchError(e);
			batchOpen = false;
			drafts = [];
			// Once a batch is confirmed its state lives in Resultados, even if sending stopped.
			if (reached) await goto(resolve('/resultados'));
			else await load();
		} finally {
			confirming = false;
		}
	}
	const counts = $derived(reviewCounts(items));

	const label = <T extends Record<string, string>>(map: T, key: string | null) =>
		key === null ? '' : (map[key as keyof T] ?? key);

	async function load() {
		try {
			items = await invoke<ReviewItem[]>('review_list');
		} catch {
			loadFailed = true;
		} finally {
			loaded = true;
		}
	}

	async function decide(it: ReviewItem, m: MatchView, decision: Decision) {
		note = '';
		try {
			await invoke('review_decide', {
				wanted: { store: it.wanted.store, native_id: it.wanted.native_id },
				held: { store: m.held.store, native_id: m.held.native_id },
				decision
			});
			await load();
			// «Es el mismo» is the decision to remove it: select it, the user can still untick it.
			// Any other decision takes it out of the selection if it no longer qualifies.
			const now = items.find((x) => itemKey(x) === itemKey(it));
			if (decision === 'same' && now && removable(now)) selected.add(itemKey(it));
			else if (!now || !removable(now)) selected.delete(itemKey(it));
		} catch {
			note = r.decideFailed;
		}
	}

	async function keep(it: ReviewItem, value: boolean) {
		note = '';
		try {
			await invoke('review_keep', {
				accountId: it.account_id,
				entry: { store: it.wanted.store, native_id: it.wanted.native_id },
				keep: value,
				reason: null
			});
			if (value) selected.delete(itemKey(it));
			await load();
		} catch {
			note = r.decideFailed;
		}
	}

	function outcomeClass(o: string): string {
		if (o === 'exact' || o === 'candidate') return 'text-success';
		if (o === 'unresolved') return 'text-warning';
		return 'text-muted';
	}

	onMount(load);
</script>

<h1 class="mb-2 text-xl font-semibold">{es.nav.limpieza}</h1>
<p class="mb-5 max-w-prose text-muted">{r.intro}</p>

{#if loadFailed}
	<p role="alert" class="text-danger">{es.library.loadError}</p>
{:else if loaded}
	<div class="mb-3 flex flex-wrap gap-1" role="group" aria-label={r.filtersLabel}>
		{#each filters as f (f)}
			<button
				type="button"
				aria-pressed={filter === f}
				class="cursor-pointer rounded-control px-3 py-1.5 text-sm text-muted hover:bg-raised hover:text-text aria-pressed:bg-raised aria-pressed:text-text"
				onclick={() => (filter = f)}
			>
				{r.filters[f]} <span class="tabular-nums">({counts[f]})</span>
			</button>
		{/each}
	</div>

	<label class="mb-4 grid max-w-sm gap-1 text-sm">
		<span class="text-muted">{r.search}</span>
		<input
			type="search"
			class="rounded-control border border-border bg-panel px-3 py-2"
			bind:value={text}
		/>
	</label>

	<p class="mb-2 text-sm text-warning" role="status">{note} {batchNote}</p>

	{#if selectedCount > 0}
		<div
			class="sticky top-0 z-10 mb-3 flex flex-wrap items-center gap-3 rounded-panel border border-border bg-panel p-3"
		>
			<p class="text-sm">{b.selected(selectedCount)}: {summary}</p>
			<Button tone="primary" size="sm" onclick={openBatch}>{b.reviewBatch}</Button>
			<Button tone="ghost" size="sm" onclick={() => selected.clear()}>{b.clearSelection}</Button>
		</div>
	{:else if removableItems.length > 0}
		<div class="mb-3">
			<Button size="sm" onclick={selectAllExact}>{b.selectExact(removableItems.length)}</Button>
		</div>
	{/if}

	{#if items.length === 0}
		<div class="rounded-panel border border-border p-6">
			<p class="font-medium">{r.noWishes}</p>
			<p class="text-muted">{es.wishlist.noDataBody}</p>
		</div>
	{:else if shown.length === 0}
		<div class="rounded-panel border border-border p-6">
			<p class="font-medium">{r.empty}</p>
			<p class="text-muted">{r.emptyBody}</p>
		</div>
	{:else}
		<ul class="grid max-w-5xl divide-y divide-border border-y border-border">
			{#each shown as it (`${it.account_id}:${it.wanted.store}:${it.wanted.native_id}`)}
				{@const wantedTitle = it.wanted.title ?? es.library.untitled(it.wanted.native_id)}
				<li class="grid gap-3 py-4">
					<div class="flex flex-wrap items-start justify-between gap-3">
						<div class="min-w-0">
							<p class="text-xs text-muted">{r.wanted} · {storeName(it.wanted.store)}</p>
							<p class="font-medium">{wantedTitle}</p>
							<p class="text-xs text-muted">{r.dataFrom(day(it.observed_at))}</p>
						</div>
						<div class="flex flex-wrap items-center gap-3">
							{#if removable(it)}
								<label class="flex items-center gap-2 text-sm">
									<input
										type="checkbox"
										class="size-4 accent-accent"
										aria-label={b.selectLabel(wantedTitle)}
										checked={selected.has(itemKey(it))}
										onchange={(e) =>
											e.currentTarget.checked
												? selected.add(itemKey(it))
												: selected.delete(itemKey(it))}
									/>
									<span aria-hidden="true">{b.select}</span>
								</label>
							{/if}
							{#if it.kept}
								<span class="text-sm text-muted">{r.kept}</span>
								<Button
									size="sm"
									tone="ghost"
									aria-label={`${r.unkeep}: ${wantedTitle}`}
									onclick={() => keep(it, false)}>{r.unkeep}</Button
								>
							{:else}
								<Button
									size="sm"
									aria-label={`${r.keep}: ${wantedTitle}`}
									onclick={() => keep(it, true)}>{r.keep}</Button
								>
							{/if}
						</div>
					</div>
					{#if !it.kept && !removable(it) && it.matches.some((m) => m.outcome === 'candidate')}
						<p class="text-sm text-muted">{b.notCovered}</p>
					{/if}

					{#if it.repeated_in.length > 0}
						<p class="text-sm text-muted">
							{r.repeatedIn(it.repeated_in.map((p) => storeName(p.store)).join(', '))}
						</p>
					{/if}

					{#if it.matches.length > 0}
						<ul class="grid gap-2">
							{#each sortMatches(it.matches) as m (`${m.held.store}:${m.held.native_id}:${m.access}`)}
								<li class="grid gap-2 rounded-panel bg-panel p-3 sm:grid-cols-[1fr_auto]">
									<div class="min-w-0 text-sm">
										<p class="text-xs text-muted">
											{r.held} · {storeName(m.held.store)} · {kindName(m.held.kind)} · {label(
												r.access,
												m.access
											)}
											{#if m.family_owners > 0}({r.familyOwners(m.family_owners)}){/if}
										</p>
										<p class="font-medium">
											{m.held.title ?? es.library.untitled(m.held.native_id)}
										</p>
										<p class={outcomeClass(m.outcome)}>
											{label(r.outcome, m.outcome)}{#if m.evidence}: {label(
													r.evidence,
													m.evidence
												)}{/if}{#if m.reason && m.outcome !== 'family_excluded'}. {label(
													r.reasons,
													m.reason
												)}{/if}
										</p>
										{#if m.edition_superior}<p class="text-muted">{r.editionSuperior}</p>{/if}
										{#if m.platform_unknown}<p class="text-muted">{r.platformUnknown}</p>{/if}
										{#if m.outcome === 'candidate' || m.outcome === 'exact'}
											<p class="text-muted">{r.check}</p>
										{/if}
									</div>
									{#if m.outcome !== 'family_excluded'}
										{@const heldTitle = m.held.title ?? es.library.untitled(m.held.native_id)}
										<div class="flex flex-wrap items-start gap-2">
											{#if m.manual}
												<Button
													size="sm"
													tone="ghost"
													aria-label={`${r.undo}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'clear')}
												>
													{r.undo}
												</Button>
											{:else}
												<Button
													size="sm"
													aria-label={`${r.same}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'same')}>{r.same}</Button
												>
												<Button
													size="sm"
													aria-label={`${r.different}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'different')}
												>
													{r.different}
												</Button>
											{/if}
										</div>
									{/if}
								</li>
							{/each}
						</ul>
					{/if}
				</li>
			{/each}
		</ul>
	{/if}
{/if}

<Dialog
	bind:open={
		() => batchOpen,
		(v) => {
			// While sending, closing must not discard batches that are already confirmed.
			if (!v && !confirming) cancelBatch();
		}
	}
	title={b.dialogTitle}
	description={b.dialogBody}
	closeLabel={b.cancel}
>
	<div class="grid gap-4 text-sm">
		{#each drafts as d (d.id)}
			<section class="grid gap-1">
				<p class="font-medium">
					{b.summary(b.part(d.ops.length, storeName(d.store)))}
				</p>
				<p class="text-xs text-muted">
					{b.account(storeName(d.store), accountNames[d.store] ?? d.account_id)}
				</p>
				<ul class="grid gap-1.5">
					{#each d.ops as o (o.op_id)}
						<li>
							<p>{o.title ?? es.library.untitled(o.native_id)}</p>
							<p class="text-xs text-muted">{evidenceFor.get(`${d.store}:${o.native_id}`)}</p>
						</li>
					{/each}
				</ul>
			</section>
		{/each}
		{#if confirming}
			<p class="text-muted" role="status">{b.sending}</p>
		{/if}
	</div>
	{#snippet footer()}
		<Button disabled={confirming} onclick={cancelBatch}>{b.cancel}</Button>
		<Button tone="danger" disabled={confirming} onclick={confirmBatch}>{b.confirm(toRemove)}</Button
		>
	{/snippet}
</Dialog>
