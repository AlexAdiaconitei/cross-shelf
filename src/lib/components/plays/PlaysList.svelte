<!-- The list: one section per state with rows like the Library's. Same moves as the board. -->
<script lang="ts">
	import type { PlayView } from '#lib/bindings/PlayView.ts';
	import Cover from '#lib/components/store/Cover.svelte';
	import StoreTabs from '#lib/components/store/StoreTabs.svelte';
	import { tr } from '#lib/i18n/index.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { columns, keyOf, sinceText, timeLine } from '#lib/plays/model.ts';
	import MoveMenu from './MoveMenu.svelte';

	let {
		plays,
		slotsOf,
		now,
		selected,
		onselect,
		onmove,
		onremove
	}: {
		plays: PlayView[];
		slotsOf: (p: PlayView) => Record<Store, Slot>;
		now: number;
		selected: string | null;
		onselect: (p: PlayView) => void;
		onmove: (p: PlayView, state: string) => Promise<void>;
		onremove: (p: PlayView) => void;
	} = $props();

	const cols = $derived(columns(plays).filter((c) => c.items.length));
	const title = (p: PlayView) => p.product.title ?? tr.library.untitled(p.product.native_id);
</script>

<div class="list">
	{#each cols as col (col.state)}
		<section aria-labelledby="ls-{col.state}">
			<h2 id="ls-{col.state}" class="headline">
				{tr.plays.states[col.state]} <b class="figure">{col.items.length}</b>
			</h2>
			<ul>
				{#each col.items as p (keyOf(p.product))}
					<li class:sel={selected === keyOf(p.product)}>
						<button type="button" class="row" onclick={() => onselect(p)}>
							<Cover title={title(p)} store={p.product.store as Store} src={p.product.image_url} />
							<b>{title(p)}</b>
							<span class="tm">{timeLine(p.minutes, p.last_activity, now)}</span>
							<small>{p.in_library ? sinceText(p, now) : tr.plays.notInLibrary}</small>
							<em>{p.note}</em>
						</button>
						<StoreTabs slots={slotsOf(p)} title={title(p)} size="sm" />
						<MoveMenu
							title={title(p)}
							state={p.state}
							onmove={(s) => onmove(p, s)}
							onremove={() => onremove(p)}
						/>
					</li>
				{/each}
			</ul>
		</section>
	{/each}
</div>

<style>
	.list {
		flex: 1;
		min-height: 0;
		overflow-y: auto;
		padding: 0 32px 40px;
		container-type: inline-size;
	}
	h2 {
		display: flex;
		gap: 12px;
		align-items: baseline;
		margin: 22px 0 0;
		padding-bottom: 6px;
		font-size: 28px;
		border-bottom: 3px solid var(--color-border-strong);
	}
	h2 b {
		font-size: 20px;
		color: var(--color-muted);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	li {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto 130px;
		gap: 14px;
		align-items: center;
		border-bottom: 1px solid var(--color-border);
	}
	li.sel {
		background: var(--color-raised);
	}
	.row {
		display: grid;
		grid-template-columns: 30px minmax(0, 1.2fr) 130px 180px minmax(0, 1fr);
		gap: 14px;
		align-items: center;
		min-height: 54px;
		padding: 6px 0 6px 6px;
		border: 0;
		background: none;
		color: inherit;
		text-align: left;
	}
	.row:hover {
		background: var(--color-raised);
	}
	.row b,
	.row em {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tm {
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.row small,
	.row em {
		font-style: normal;
		font-size: 13px;
		color: var(--color-muted);
	}
	@container (max-width: 860px) {
		.row {
			grid-template-columns: 30px minmax(0, 1fr) 110px;
		}
		.row small,
		.row em {
			display: none;
		}
	}
</style>
