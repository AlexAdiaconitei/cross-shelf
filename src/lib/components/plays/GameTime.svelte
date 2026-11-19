<!--
	The «Time» block of a game sheet: total hours, last session, hours per store («Steam 12 h ·
	Epic does not give it») and a link to search the game on HowLongToBeat. The app never asks
	HowLongToBeat anything (ADR 0002). GOG is asked when the sheet opens, one product at a time.
-->
<script lang="ts">
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { IconExternalLink } from '@tabler/icons-svelte';
	import type { ProductTime } from '#lib/bindings/ProductTime.ts';
	import { button } from '#lib/components/ui/index.ts';
	import { storeName } from '#lib/format.ts';
	import { tr } from '#lib/i18n/index.ts';
	import { ipc } from '#lib/ipc.ts';
	import {
		ago,
		hltbUrl,
		hours,
		keyOf,
		nowSecs,
		sheetTotal,
		storeTimes,
		type GogAnswers
	} from '#lib/plays/model.ts';

	let { title, products }: { title: string; products: { store: string; native_id: string }[] } =
		$props();

	const s = tr.plays.sheet;
	const now = nowSecs();
	let times = $state<ProductTime[]>([]);
	let given = $state<string[]>([]);
	let gog = $state<GogAnswers>({});

	$effect(() => {
		const ps = products.map((p) => ({ store: p.store, native_id: p.native_id }));
		let live = true;
		gog = {};
		void (async () => {
			try {
				const [t, g] = await Promise.all([ipc.playtimeOf(ps), ipc.playtimeStores()]);
				if (!live) return;
				times = t;
				given = g;
				if (!g.includes('gog')) return;
				for (const p of ps.filter((x) => x.store === 'gog')) {
					const answer = await ipc.gogPlaytime(p.native_id).catch(() => 'error' as const);
					if (live) gog = { ...gog, [keyOf(p)]: answer };
				}
			} catch {
				/* the block stays empty: nothing known is not zero */
			}
		})();
		return () => {
			live = false;
		};
	});

	const lines = $derived(storeTimes(products, times, given, gog));
	const total = $derived(sheetTotal(lines, times));
	const lineText = (l: (typeof lines)[number]) =>
		l.state === 'time'
			? hours(l.minutes)
			: l.state === 'loading'
				? s.loading
				: l.state === 'error'
					? s.error(storeName(l.store))
					: s.notGiven(storeName(l.store));
</script>

<section class="time" aria-label={s.time}>
	<p class="label">{s.time}</p>
	<dl>
		<dt>{s.total}</dt>
		<dd class="big">{hours(total.minutes)}</dd>
		<dt>{s.lastSession}</dt>
		<dd>{total.last === null ? tr.plays.noTime : ago(total.last, now)}</dd>
		<dt>{s.byStore}</dt>
		<dd>
			{#each lines as l, i (l.store)}{#if i > 0}<span class="sep" aria-hidden="true">
						·
					</span>{/if}<span class:muted={l.state !== 'time'} class:err={l.state === 'error'}
					>{#if l.state === 'time'}{storeName(l.store)}
					{/if}{lineText(l)}</span
				>{/each}
		</dd>
	</dl>
	<div class="hltb">
		<button
			type="button"
			class={button({ size: 'sm' })}
			onclick={() => openUrl(hltbUrl(title)).catch(() => {})}
			>{s.hltb} <IconExternalLink size={14} stroke={1.75} /></button
		>
		<small>{s.hltbHint}</small>
	</div>
</section>

<style>
	.time {
		display: grid;
		gap: 8px;
	}
	.label {
		margin: 0;
	}
	dl {
		display: grid;
		grid-template-columns: 96px 1fr;
		margin: 0;
		border-top: 2px solid var(--color-border-strong);
	}
	dt,
	dd {
		margin: 0;
		padding: 9px 0;
		border-bottom: 1px solid var(--color-border);
	}
	dt {
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--color-muted);
	}
	dd {
		font-variant-numeric: tabular-nums;
	}
	.big {
		font-weight: 800;
	}
	.muted {
		color: var(--color-muted);
	}
	.err {
		color: var(--color-warning);
	}
	.sep {
		color: var(--color-muted);
	}
	.hltb {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.hltb small {
		color: var(--color-muted);
		font-size: 12px;
	}
</style>
