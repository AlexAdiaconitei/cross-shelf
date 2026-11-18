<!--
	The signature of the visual world: one coloured tab per store, cut at the corner like a
	magazine platform tab. Filled: you own it. Outline: family or temporary access. Dashed: in
	your wishlist. Yellow dashed: wished here while you already own it elsewhere.
-->
<script lang="ts">
	import { storeName } from '#lib/format.ts';
	import { es } from '#lib/strings.ts';
	import { stores, type Slot, type Store } from '#lib/library/shelf.ts';
	import StoreLogo from './StoreLogo.svelte';

	let {
		slots,
		title,
		wishedElsewhere = false,
		size = 'md'
	}: {
		slots: Record<Store, Slot>;
		title: string;
		wishedElsewhere?: boolean;
		size?: 'sm' | 'md' | 'lg';
	} = $props();
</script>

<ul class="tabs {size}" aria-label={es.shelf.storesOf(title)}>
	{#each stores as s (s)}
		{@const slot = slots[s]}
		{#if slot !== 'none'}
			{@const text = `${storeName(s)}: ${es.shelf.slot[slot]}`}
			<li class="t {s} {slot}" class:repe={slot === 'wish' && wishedElsewhere} title={text}>
				<StoreLogo store={s} size={size === 'lg' ? 18 : 14} />
				<span class="sr-only">{text}</span>
				{#if size !== 'sm'}<span aria-hidden="true">{storeName(s)}</span>{/if}
			</li>
		{/if}
	{/each}
</ul>

<style>
	.tabs {
		display: flex;
		flex-wrap: wrap;
		gap: 3px;
		justify-content: flex-end;
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.t {
		--c: var(--color-steam);
		--on: var(--color-on-store);
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 28px;
		padding: 0 10px 0 9px;
		font-size: 11px;
		font-weight: 800;
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.sm .t {
		padding: 0 7px;
	}
	.lg .t {
		height: 34px;
		font-size: 12px;
		padding: 0 12px 0 11px;
	}
	.gog {
		--c: var(--color-gog);
	}
	.epic {
		--c: var(--color-epic);
		--on: var(--color-on-epic);
	}
	.own {
		background: var(--c);
		color: var(--on);
		clip-path: polygon(0 0, 100% 0, 100% 100%, 9px 100%, 0 calc(100% - 9px));
	}
	.family,
	.temp,
	.unknown {
		color: var(--c);
		box-shadow: inset 0 0 0 2px var(--c);
	}
	.wish {
		color: var(--color-muted);
		outline: 1.5px dashed var(--color-z5);
		outline-offset: -1.5px;
	}
	.wish.repe {
		background: var(--color-next);
		color: var(--color-on-next);
		outline-color: var(--color-on-next);
	}
</style>
