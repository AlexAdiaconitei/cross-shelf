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
	import { IconHistory, IconSearch } from '@tabler/icons-svelte';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import Cover from '#lib/components/store/Cover.svelte';
	import { Button, Dialog, InfoTip, button, chip } from '#lib/components/ui/index.ts';
	import type { Store } from '#lib/library/shelf.ts';
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
	import { tr } from '#lib/i18n/index.ts';

	const filters: ReviewFilter[] = ['to_review', 'kept', 'no_match', 'repeated', 'all'];
	const r = tr.review;

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
		tr.batch.summary(
			groups.map((g) => tr.batch.part(g.items.length, storeName(g.store))).join(' y ')
		)
	);
	const b = tr.batch;
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

	/** Verdict stamp colour: firm evidence, worth a look, or not covering it. */
	function tone(o: string): 'ok' | 'maybe' | 'no' {
		if (o === 'exact') return 'ok';
		if (o === 'candidate' || o === 'unresolved') return 'maybe';
		return 'no';
	}
	/** Secondary facts of a wish, shown behind an icon instead of under the title. */
	function wantedNotes(it: ReviewItem): string[] {
		return [
			r.dataFrom(day(it.observed_at)),
			...(it.repeated_in.length > 0
				? [r.repeatedIn(it.repeated_in.map((x) => storeName(x.store)).join(', '))]
				: []),
			...(!it.kept && !removable(it) && it.matches.some((m) => m.outcome === 'candidate')
				? [b.notCovered]
				: [])
		];
	}

	function matchNotes(m: MatchView): string[] {
		return [
			...(m.edition_superior ? [r.editionSuperior] : []),
			...(m.platform_unknown ? [r.platformUnknown] : []),
			...(m.outcome === 'candidate' || m.outcome === 'exact' ? [r.check] : [])
		];
	}

	// Inline separator kept as data: the formatter trims the space before a literal «·».
	const sep = ' · ';

	/** Earlier batches, for the secondary link to Resultados. Hidden while there are none. */
	let batchCount = $state(0);

	onMount(() => {
		void load();
		ipc
			.plans()
			.then((b) => (batchCount = b.length))
			.catch(() => {});
	});
</script>

<PageHeader title={tr.nav.limpieza}>
	{#snippet info()}<InfoTip label={tr.nav.limpieza} lines={[r.intro]} />{/snippet}
	{#snippet action()}
		{#if batchCount > 0}
			<a href={resolve('/resultados')} class={button({ tone: 'ghost', size: 'sm' })}
				><IconHistory size={16} stroke={1.75} />{tr.results.earlier(batchCount)}</a
			>
		{/if}
	{/snippet}
</PageHeader>

<div class="wrap">
	{#if loadFailed}
		<p role="alert" class="text-danger">{tr.library.loadError}</p>
	{:else if loaded}
		<div class="toolbar">
			<div class="chips when-roomy" role="group" aria-label={r.filtersLabel}>
				{#each filters as f (f)}
					<button
						type="button"
						class={chip()}
						aria-pressed={filter === f}
						onclick={() => (filter = f)}>{r.filters[f]} · {counts[f]}</button
					>
				{/each}
			</div>
			<select class="control when-tight" aria-label={r.filtersLabel} bind:value={filter}>
				{#each filters as f (f)}<option value={f}
						>{tr.wishlist.status}: {r.filters[f]} · {counts[f]}</option
					>{/each}
			</select>
			<label class="control control-search">
				<IconSearch size={16} />
				<span class="sr-only">{r.search}</span>
				<input type="search" placeholder="{r.search}…" bind:value={text} />
			</label>
			{#if removableItems.length > 0 && selectedCount === 0}
				<button type="button" class="control push" onclick={selectAllExact}
					>{b.selectExact(removableItems.length)}</button
				>
			{/if}
		</div>

		{#if note || batchNote}<p class="text-warning" role="status">{note} {batchNote}</p>{/if}

		{#if items.length === 0}
			<div class="empty">
				<p class="headline">{r.noWishes}</p>
				<p>{tr.wishlist.noDataBody}</p>
			</div>
		{:else if shown.length === 0}
			<div class="empty">
				<p class="headline">{r.empty}</p>
				<p>{r.emptyBody}</p>
			</div>
		{:else}
			<ul class="cmp">
				{#each shown as it (`${it.account_id}:${it.wanted.store}:${it.wanted.native_id}`)}
					{@const wantedTitle = it.wanted.title ?? tr.library.untitled(it.wanted.native_id)}
					<li class="item" class:picked={selected.has(itemKey(it))}>
						<div class="pick">
							{#if removable(it)}
								<input
									type="checkbox"
									aria-label={b.selectLabel(wantedTitle)}
									checked={selected.has(itemKey(it))}
									onchange={(e) =>
										e.currentTarget.checked
											? selected.add(itemKey(it))
											: selected.delete(itemKey(it))}
								/>
							{/if}
						</div>
						<div class="half">
							<Cover
								title={wantedTitle}
								store={it.wanted.store as Store}
								src={it.wanted.image_url}
							/>
							<div class="min-w-0">
								<p class="label">{r.wanted} · {storeName(it.wanted.store)}</p>
								<p class="tt">
									{wantedTitle}
									<InfoTip label={wantedTitle} lines={wantedNotes(it)} />
								</p>
							</div>
						</div>
						<div class="matches">
							{#each sortMatches(it.matches) as m (`${m.held.store}:${m.held.native_id}:${m.access}`)}
								{@const heldTitle = m.held.title ?? tr.library.untitled(m.held.native_id)}
								<div class="match">
									<p class="stamp {tone(m.outcome)}">{label(r.outcome, m.outcome)}</p>
									<div class="half">
										<Cover title={heldTitle} store={m.held.store as Store} src={m.held.image_url} />
										<div class="min-w-0">
											<p class="label">
												{r.held} · {storeName(m.held.store)} · {kindName(m.held.kind)}
											</p>
											<p class="tt">
												{heldTitle}
												<InfoTip label={heldTitle} lines={matchNotes(m)} />
											</p>
											<p class="muted xs">
												{label(r.access, m.access)}{#if m.family_owners > 0}
													({r.familyOwners(m.family_owners)}){/if}{#if m.evidence}{sep}{label(
														r.evidence,
														m.evidence
													)}{/if}{#if m.reason && m.outcome !== 'family_excluded'}{sep}{label(
														r.reasons,
														m.reason
													)}{/if}
											</p>
										</div>
									</div>
									{#if m.outcome !== 'family_excluded'}
										<div class="acts">
											{#if m.manual}
												<Button
													size="sm"
													tone="ghost"
													aria-label={`${r.undo}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'clear')}>{r.undo}</Button
												>
											{:else}
												<Button
													size="sm"
													aria-label={`${r.same}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'same')}>{r.same}</Button
												>
												<Button
													size="sm"
													aria-label={`${r.different}: ${wantedTitle} y ${heldTitle}`}
													onclick={() => decide(it, m, 'different')}>{r.different}</Button
												>
											{/if}
										</div>
									{/if}
								</div>
							{:else}
								<p class="stamp no">{r.filters.no_match}</p>
							{/each}
						</div>
						<div class="keep">
							{#if it.kept}
								<span class="muted xs">{r.kept}</span>
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
					</li>
				{/each}
			</ul>
		{/if}
	{/if}
</div>

{#if loaded && !loadFailed}
	<footer class="lote" class:on={selectedCount > 0}>
		<p>
			{#if selectedCount > 0}<b>{b.selected(selectedCount)}: {summary}</b>{:else}{b.hint}
				<InfoTip label={b.hint} lines={[b.hintMore]} />{/if}
		</p>
		{#if selectedCount > 0}
			<Button tone="ghost" size="sm" onclick={() => selected.clear()}>{b.clearSelection}</Button>
		{/if}
		<Button tone="primary" disabled={selectedCount === 0} onclick={openBatch}
			>{b.reviewBatch}</Button
		>
	</footer>
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
							<p>{o.title ?? tr.library.untitled(o.native_id)}</p>
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

<style>
	.wrap {
		flex: 1;
		padding: 0 32px 24px;
	}
	.toolbar {
		margin: 0 -32px;
	}
	/* display comes from .when-roomy in app.css, so the toolbar can swap chips for a select. */
	.chips {
		gap: 6px;
	}
	.tt :global(.tip-trigger) {
		margin-left: 2px;
	}
	.empty {
		display: grid;
		gap: 8px;
		padding: 32px 0;
	}
	.empty .headline {
		margin: 0;
		font-size: 30px;
	}
	.empty p {
		margin: 0;
	}
	.cmp {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.item {
		display: grid;
		grid-template-columns: 28px minmax(0, 1fr) minmax(0, 1.6fr) auto;
		gap: 18px;
		align-items: start;
		padding: 16px 0;
		border-bottom: 1px solid var(--color-border);
	}
	.item.picked {
		background: var(--color-next-wash);
		box-shadow:
			-32px 0 0 var(--color-next-wash),
			32px 0 0 var(--color-next-wash);
	}
	.pick input {
		width: 18px;
		height: 18px;
		margin-top: 4px;
		accent-color: var(--color-text);
	}
	.half {
		display: flex;
		gap: 12px;
		align-items: flex-start;
	}
	.half p {
		margin: 0;
	}
	.tt {
		font-size: 16px;
		font-weight: 700;
	}
	.muted {
		color: var(--color-muted);
	}
	.xs {
		font-size: 12px;
	}
	.matches {
		display: grid;
		gap: 12px;
	}
	.match {
		display: grid;
		grid-template-columns: 150px minmax(0, 1fr);
		gap: 8px 16px;
		align-items: start;
	}
	.match .acts {
		grid-column: 2;
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.stamp {
		margin: 0;
		padding: 5px 8px;
		border: 2px solid currentColor;
		text-align: center;
		font-size: 13px;
		font-weight: 800;
		font-variation-settings: 'wdth' 70;
		text-transform: uppercase;
	}
	.stamp.ok {
		color: var(--color-success);
	}
	.stamp.maybe {
		color: var(--color-warning);
	}
	.stamp.no {
		width: 150px;
		color: var(--color-muted);
		border-style: dashed;
	}
	.keep {
		display: grid;
		justify-items: end;
		gap: 4px;
	}
	.lote {
		position: sticky;
		bottom: 0;
		display: flex;
		align-items: center;
		gap: 12px;
		padding: 12px 32px;
		background: var(--color-page);
		border-top: 3px solid var(--color-border-strong);
	}
	.lote.on {
		background: var(--color-next-wash);
	}
	.lote p {
		flex: 1;
		margin: 0;
	}
</style>
