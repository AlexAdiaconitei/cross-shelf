<!--
	Side panel with a game's technical sheet: big cover, title, store tabs, a short spec and one
	line per store product with a button to open it in the store. Library and Wishlists share it.
-->
<script lang="ts">
	import type { Snippet } from 'svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import { IconExternalLink, IconX } from '@tabler/icons-svelte';
	import { button } from '#lib/components/ui/index.ts';
	import { productUrl, storeName } from '#lib/format.ts';
	import type { Slot, Store } from '#lib/library/shelf.ts';
	import { tr } from '#lib/i18n/index.ts';
	import Cover from './Cover.svelte';
	import StoreLogo from './StoreLogo.svelte';
	import StoreTabs from './StoreTabs.svelte';

	export type DetailProduct = {
		store: string;
		native_id: string;
		title: string | null;
		url: string | null;
		note: string;
	};

	let {
		title,
		store,
		cover,
		slots,
		wishedElsewhere = false,
		products,
		onclose,
		spec,
		children,
		actions
	}: {
		title: string;
		store: Store;
		cover: string | null;
		slots: Record<Store, Slot>;
		wishedElsewhere?: boolean;
		products: DetailProduct[];
		onclose: () => void;
		spec?: Snippet;
		/** Blocks between the spec and the products: play state, time. */
		children?: Snippet;
		actions?: Snippet;
	} = $props();

	const s = tr.shelf;
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && onclose()} />

<aside class="detail" aria-label={s.detailOf(title)}>
	<div class="dh">
		<p class="label">{s.detail}</p>
		<button
			type="button"
			class={button({ size: 'icon' })}
			onclick={onclose}
			aria-label={s.closeDetail}><IconX size={18} /></button
		>
	</div>
	<div class="hero">
		<Cover {title} {store} src={cover} size="lg" />
		<div class="hx">
			<h2 class="headline selectable">{title}</h2>
			<StoreTabs {slots} {title} {wishedElsewhere} size="lg" />
		</div>
	</div>
	{#if spec}<dl class="spec selectable">{@render spec()}</dl>{/if}
	{@render children?.()}
	<p class="label">{s.products}</p>
	<ul class="prods">
		{#each products as p (p.store + p.native_id)}
			<li class={p.store}>
				<span class="pt"><StoreLogo store={p.store} size={14} /></span>
				<div>
					<b class="selectable">{p.title ?? tr.library.untitled(p.native_id)}</b>
					{#if p.note}<small>{p.note}</small>{/if}
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
	</ul>
	{#if actions}<div class="acts">{@render actions()}</div>{/if}
</aside>

<style>
	.detail {
		z-index: 20;
		min-height: 0;
		overflow-x: hidden;
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 18px 24px 40px;
		border-left: 3px solid var(--color-border-strong);
		background: var(--color-page);
	}
	.dh {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}
	.label {
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
	h2 {
		margin: 0;
		font-size: 38px;
		overflow-wrap: anywhere;
	}
	.spec {
		display: grid;
		grid-template-columns: 96px 1fr;
		margin: 0;
		border-top: 2px solid var(--color-border-strong);
	}
	.spec :global(dt),
	.spec :global(dd) {
		margin: 0;
		padding: 9px 0;
		border-bottom: 1px solid var(--color-border);
	}
	.spec :global(dt) {
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
	.acts {
		display: flex;
		gap: 8px;
	}
</style>
