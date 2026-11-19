<script lang="ts">
	import '../app.css';
	import { onMount } from 'svelte';
	import { afterNavigate } from '$app/navigation';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import {
		IconAlertTriangle,
		IconLayoutSidebarLeftCollapse,
		IconLayoutSidebarLeftExpand
	} from '@tabler/icons-svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import { storeName } from '#lib/format.ts';
	import { navGroups } from '#lib/nav.ts';
	import {
		readSidebarOpen,
		refreshShell,
		saveSidebarOpen,
		shell
	} from '#lib/shell/state.svelte.ts';
	import logoSmall from '#lib/assets/brand/cross-shelf-small-on-dark.svg';
	import logoWide from '#lib/assets/brand/cross-shelf-horizontal-on-dark.svg';
	import { intl, lang, tr } from '#lib/i18n/index.ts';

	let { children } = $props();
	let open = $state(false);

	onMount(() => {
		open = readSidebarOpen();
		document.documentElement.lang = lang;
		// No browser menu on right click: it offers reload and print, not app actions. Text fields
		// keep it for copy and paste, and dev builds keep it for the inspector.
		const noMenu = (e: MouseEvent) => {
			const t = e.target as HTMLElement | null;
			if (import.meta.env.DEV || t?.closest('input, textarea, [contenteditable]')) return;
			e.preventDefault();
		};
		document.addEventListener('contextmenu', noMenu);
		// The static splash of app.html: fade it out now that the app is on screen.
		const splash = document.getElementById('splash');
		if (splash) {
			splash.classList.add('out');
			setTimeout(() => splash.remove(), 250);
		}
	});
	afterNavigate(refreshShell);

	function toggle() {
		open = !open;
		saveSidebarOpen(open);
	}

	/** Resultados has no item of its own: it lives under Revisar limpieza (plan 0.3.0, 2.7). */
	const isCurrent = (href: string) =>
		href === '/'
			? page.url.pathname === '/'
			: page.url.pathname.startsWith(href) ||
				(href === '/limpieza' && page.url.pathname.startsWith('/resultados'));

	/** One block per store: library size and whether its data can be trusted as shown. */
	const storeBlocks = $derived(
		shell.connections.map((c) => {
			const lib = c.capabilities.find((x) => x.capability === 'library');
			const why =
				c.state !== 'connected'
					? tr.connections.disconnected
					: lib?.state === 'partial'
						? tr.capabilityState.partial
						: lib?.state === 'failed'
							? tr.shell.syncFailed
							: '';
			return { store: c.provider, count: lib?.item_count ?? null, why };
		})
	);
</script>

<div class="shell" class:open>
	<nav class="side" aria-label={tr.appName}>
		<div class="brand">
			{#if open}
				<img class="mark wide" src={logoWide} alt={tr.appName} />
			{:else}
				<img class="mark small" src={logoSmall} alt={tr.appName} />
			{/if}
			<button
				type="button"
				class="fold"
				onclick={toggle}
				aria-expanded={open}
				aria-label={open ? tr.shell.foldSidebar : tr.shell.openSidebar}
				title={open ? tr.shell.foldSidebar : tr.shell.openSidebar}
			>
				{#if open}<IconLayoutSidebarLeftCollapse size={20} />{:else}<IconLayoutSidebarLeftExpand
						size={20}
					/>{/if}
			</button>
		</div>

		{#each navGroups as group, gi (group.label)}
			{#if open}<p class="label grp">{group.label}</p>{/if}
			<ul class="items" class:last={gi === navGroups.length - 1}>
				{#each group.items as item (item.href)}
					<li>
						<a
							href={resolve(item.href)}
							class="it"
							aria-current={isCurrent(item.href) ? 'page' : undefined}
							title={open ? undefined : item.label}
						>
							<span class="ic">
								<item.icon size={open ? 18 : 22} stroke={1.75} />
								{#if !open && item.href === '/limpieza' && shell.toReview}<em>{shell.toReview}</em
									>{/if}
							</span>
							<span class="tx">{open || !('short' in item) ? item.label : item.short}</span>
							{#if open && item.href === '/limpieza' && shell.toReview}<em class="badge"
									>{shell.toReview}</em
								>{/if}
						</a>
					</li>
				{/each}
			</ul>
		{/each}

		{#if open && storeBlocks.length}
			<div class="stores">
				<p class="label grp">{tr.shell.stores}</p>
				{#each storeBlocks as b (b.store)}
					<a href={resolve('/conexiones')} class="st {b.store}">
						<span class="tab store-cut"
							><StoreLogo store={b.store} size={14} />{storeName(b.store)}</span
						>
						<b>{b.count?.toLocaleString(intl) ?? '—'}</b>
						{#if b.why}<span class="w" title={b.why}><IconAlertTriangle size={15} /></span>{/if}
					</a>
					{#if b.why}<p class="why">{storeName(b.store)}: {b.why}</p>{/if}
				{/each}
			</div>
		{/if}
	</nav>

	<main>
		{@render children()}
	</main>
</div>

<style>
	.shell {
		display: grid;
		grid-template-columns: 84px minmax(0, 1fr);
		height: 100vh;
		background: var(--color-page);
		color: var(--color-text);
		transition: grid-template-columns 200ms cubic-bezier(0.16, 1, 0.3, 1);
	}
	.shell.open {
		grid-template-columns: 252px minmax(0, 1fr);
	}
	main {
		min-width: 0;
		min-height: 0;
		overflow-y: auto;
		/* Never a horizontal scroll: views fit their width; anything that still overflows is clipped. */
		overflow-x: clip;
		display: flex;
		flex-direction: column;
	}
	.side {
		display: flex;
		flex-direction: column;
		background: var(--color-rail);
		color: var(--color-rail-muted);
		padding: 14px 0 18px;
		overflow: hidden auto;
	}
	.open .side {
		padding: 14px 12px 18px;
	}
	.brand {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		margin-bottom: 10px;
	}
	.open .brand {
		flex-direction: row;
		justify-content: space-between;
		padding: 0 6px;
	}
	/* Brand kit: small mark on the folded rail, horizontal lockup when open (both on-dark). */
	.mark {
		display: block;
	}
	.mark.small {
		width: 36px;
		height: 36px;
	}
	.mark.wide {
		height: 30px;
		width: auto;
	}
	.fold {
		display: grid;
		place-items: center;
		width: 34px;
		height: 30px;
		border: 0;
		background: none;
		color: inherit;
		transition: background-color 150ms ease-out;
	}
	.fold:hover {
		color: var(--color-z0);
		background: var(--color-rail-raised);
	}
	.grp {
		margin: 14px 8px 4px;
		color: var(--color-z5);
	}
	.items {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
	}
	.items.last {
		margin-top: auto;
	}
	.open .items {
		align-items: stretch;
	}
	.open .items.last {
		margin-top: 0;
	}
	.it {
		width: 74px;
		display: grid;
		justify-items: center;
		gap: 3px;
		padding: 7px 2px;
		color: inherit;
		text-decoration: none;
		font-size: 10.5px;
		font-weight: 600;
		line-height: 1.1;
		text-align: center;
	}
	.it:hover,
	.it[aria-current='page'] {
		color: var(--color-z0);
	}
	.ic {
		position: relative;
		display: grid;
		place-items: center;
		width: 40px;
		height: 32px;
		transition: background-color 150ms ease-out;
	}
	.it[aria-current='page'] .ic {
		background: var(--color-next);
		color: var(--color-on-next);
	}
	.ic em,
	.badge {
		font-style: normal;
		font-weight: 800;
		background: var(--color-next);
		color: var(--color-on-next);
	}
	.ic em {
		position: absolute;
		top: -4px;
		right: -8px;
		font-size: 10px;
		padding: 0 4px;
	}
	.open .it {
		width: 100%;
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 36px;
		padding: 0 8px;
		font-size: 14px;
		text-align: left;
	}
	.open .ic {
		width: auto;
		height: auto;
		background: none;
		color: inherit;
	}
	.open .tx {
		flex: 1;
	}
	.open .it[aria-current='page'] {
		background: var(--color-next);
		color: var(--color-on-next);
	}
	.open .it[aria-current='page'] .ic {
		background: none;
		color: inherit;
	}
	.open .it[aria-current='page'] .badge {
		background: var(--color-on-next);
		color: var(--color-next);
	}
	.badge {
		font-size: 12px;
		padding: 1px 7px;
	}
	.stores {
		margin-top: auto;
		display: grid;
		gap: 4px;
	}
	.st {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
		padding-right: 8px;
		background: var(--color-rail-raised);
		color: var(--color-z1);
		text-decoration: none;
	}
	.st b {
		margin-left: auto;
		font-size: 15px;
	}
	.st .w {
		display: grid;
		color: oklch(0.8 0.13 75);
	}
	.tab {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		height: 36px;
		padding: 0 12px 0 10px;
		background: var(--color-steam);
		color: var(--color-on-store);
		font-size: 12px;
		font-weight: 800;
		text-transform: uppercase;
	}
	.gog .tab {
		background: var(--color-gog);
	}
	.epic .tab {
		background: var(--color-z1);
		color: var(--color-z10);
	}
	.why {
		margin: 0 4px 4px;
		font-size: 12px;
		color: oklch(0.8 0.13 75);
	}
</style>
