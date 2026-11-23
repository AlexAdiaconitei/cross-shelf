<!--
	The diary tab (plan 0.3.0, P3 without durations): ordered by activity, not by state. Groups by
	last session; each row carries the play's state (solid) or the deduced one (dashed, «…?»),
	which a click confirms. «+X h» is what was played since a week ago.
-->
<script lang="ts">
	import { IconCheck } from '@tabler/icons-svelte';
	import type { DiaryRow } from '#lib/bindings/DiaryRow.ts';
	import type { ProductRef } from '#lib/bindings/ProductRef.ts';
	import Cover from '#lib/components/store/Cover.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { intl, tr } from '#lib/i18n/index.ts';
	import { PLAY_STATES, type PlayState } from '#lib/ipc.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { ago, diaryGroups, diarySummary, hours, keyOf } from '#lib/plays/model.ts';

	let {
		rows,
		slotsFor,
		now,
		selected,
		onselect,
		onset
	}: {
		rows: DiaryRow[];
		slotsFor: (p: ProductRef) => Record<Store, Slot>;
		now: number;
		selected: string | null;
		onselect: (p: ProductRef) => void;
		/** A state, or `null` to remove the play. */
		onset: (p: ProductRef, state: PlayState | null) => void;
	} = $props();

	const d = tr.plays.diary;
	const groups = $derived(diaryGroups(rows, now));
	const sum = $derived(diarySummary(rows));
	const title = (r: DiaryRow) => r.product.title ?? tr.library.untitled(r.product.native_id);
</script>

<div class="diary">
	<dl class="sum">
		<div>
			<dd class="figure">{hours(sum.weekMinutes)}</dd>
			<dt>{d.weekHours}</dt>
		</div>
		<div>
			<dd class="figure">{sum.inProgress.toLocaleString(intl)}</dd>
			<dt>{d.inProgress}</dt>
		</div>
		<div>
			<dd class="figure">{sum.toConfirm.toLocaleString(intl)}</dd>
			<dt>{d.toConfirm}</dt>
		</div>
		<p class="src">{d.source}</p>
	</dl>

	{#if rows.length === 0}
		<div class="empty">
			<p class="headline">{d.empty}</p>
			<p>{d.emptyBody}</p>
		</div>
	{/if}

	{#each groups as g (g.group)}
		<section aria-labelledby="dg-{g.group}">
			<h2 id="dg-{g.group}" class="headline">{d.groups[g.group]}</h2>
			<ol>
				{#each g.rows as r (keyOf(r.product))}
					<li class:sel={selected === keyOf(r.product)}>
						<button type="button" class="r" onclick={() => onselect(r.product)}>
							<Cover title={title(r)} store={r.product.store as Store} src={r.product.image_url} />
							<b>{title(r)}</b>
							<span class="when"
								><span class="sr-only">{d.lastSession}:</span>
								{r.last_activity === null ? tr.plays.noTime : ago(r.last_activity, now)}</span
							>
							<span class="tot"><span class="sr-only">{d.total}:</span> {hours(r.minutes)}</span>
							<span class="wk"
								>{#if r.week_minutes}<span class="sr-only">{d.week}:</span> +{hours(
										r.week_minutes
									)}{/if}</span
							>
						</button>
						<StoreTabs slots={slotsFor(r.product)} title={title(r)} size="sm" />
						{#if r.state}
							<select
								class="st set"
								aria-label={d.stateOf(title(r))}
								value={r.state}
								onchange={(e) => {
									const v = e.currentTarget.value;
									onset(r.product, v ? (v as PlayState) : null);
								}}
							>
								{#each PLAY_STATES as s (s)}<option value={s}>{tr.plays.states[s]}</option>{/each}
								<option value="">{d.noState}</option>
							</select>
						{:else if r.deduced}
							{@const guess = tr.plays.states[r.deduced as PlayState]}
							<button
								type="button"
								class="st guess"
								aria-label={d.confirm(guess, title(r))}
								onclick={() => onset(r.product, r.deduced as PlayState)}
								>{d.guess(guess)} <IconCheck size={14} stroke={1.75} /></button
							>
						{:else}
							<select
								class="st"
								aria-label={d.stateOf(title(r))}
								value=""
								onchange={(e) => {
									const v = e.currentTarget.value;
									if (v) onset(r.product, v as PlayState);
								}}
							>
								<option value="">{d.noState}</option>
								{#each PLAY_STATES as s (s)}<option value={s}>{tr.plays.states[s]}</option>{/each}
							</select>
						{/if}
					</li>
				{/each}
			</ol>
		</section>
	{/each}
</div>

<style>
	.diary {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding: 0 32px 40px;
		container-type: inline-size;
	}
	.sum {
		display: flex;
		gap: 32px;
		align-items: baseline;
		margin: 0;
		padding: 16px 0 8px;
		border-bottom: 3px solid var(--color-border-strong);
	}
	.sum div {
		display: flex;
		gap: 8px;
		align-items: baseline;
	}
	.sum dd {
		margin: 0;
		font-size: 30px;
	}
	.sum dt {
		color: var(--color-muted);
	}
	.src {
		margin: 0 0 0 auto;
		font-size: 12px;
		color: var(--color-muted);
	}
	.empty {
		display: grid;
		gap: 10px;
		padding: 32px 0;
		max-width: 70ch;
	}
	.empty .headline {
		font-size: 30px;
		margin: 0;
	}
	.empty p {
		margin: 0;
	}
	h2 {
		font-size: 24px;
		margin: 22px 0 0;
		padding-bottom: 4px;
		border-bottom: 2px solid var(--color-border-strong);
	}
	ol {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	li {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto 150px;
		gap: 14px;
		align-items: center;
		border-bottom: 1px solid var(--color-border);
	}
	li.sel {
		background: var(--color-raised);
	}
	.r {
		display: grid;
		grid-template-columns: 30px minmax(0, 1fr) 120px 70px 70px;
		gap: 14px;
		align-items: center;
		min-height: 50px;
		padding: 4px 0 4px 6px;
		border: 0;
		background: none;
		color: inherit;
		text-align: left;
	}
	.r:hover {
		background: var(--color-raised);
	}
	.r b {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.when {
		color: var(--color-muted);
		font-size: 13px;
	}
	.tot,
	.wk {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	.tot {
		font-weight: 700;
	}
	.wk {
		color: var(--color-success);
		font-weight: 700;
		font-size: 13px;
	}
	.st {
		height: 30px;
		font-size: 13px;
		border: 1px solid var(--color-border);
		background: var(--color-page);
		color: var(--color-text);
	}
	.st.set {
		border: 2px solid var(--color-border-strong);
		font-weight: 700;
	}
	.st.guess {
		display: inline-flex;
		gap: 6px;
		align-items: center;
		justify-content: center;
		border: 1.5px dashed var(--color-border-strong);
		background: var(--color-next-wash);
		font-weight: 700;
	}
	@container (max-width: 760px) {
		.r {
			grid-template-columns: 30px minmax(0, 1fr) 70px;
		}
		.when,
		.wk {
			display: none;
		}
		.src {
			display: none;
		}
	}
</style>
