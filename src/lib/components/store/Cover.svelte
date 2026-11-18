<!--
	Portrait cover (3:4). The initials in the store colour are always underneath; the image only
	shows once it has loaded, and a failed one is dropped, so a broken-image icon never appears.
-->
<script lang="ts">
	import { initials, type Store } from '#lib/library/shelf.ts';
	import { shell } from '#lib/shell/state.svelte.ts';
	import StoreLogo from './StoreLogo.svelte';

	let {
		title,
		store,
		src = null,
		size = 'sm'
	}: { title: string; store: Store; src?: string | null; size?: 'sm' | 'md' | 'lg' } = $props();

	// The setting in Ajustes turns every cover off at once; the initials stay.
	const url = $derived(shell.showCovers ? src : null);
	let status = $state<'loading' | 'ok' | 'failed'>('loading');
	$effect(() => {
		void url;
		status = 'loading';
	});
</script>

<span class="cv {size} {store}" class:shown={status === 'ok'} aria-hidden="true">
	<span class="headline fb">{initials(title)}</span>
	{#if size === 'lg'}<span class="logo"><StoreLogo {store} size={18} /></span>{/if}
	{#if url && status !== 'failed'}
		<img
			src={url}
			alt=""
			loading="lazy"
			decoding="async"
			draggable="false"
			referrerpolicy="no-referrer"
			onload={() => (status = 'ok')}
			onerror={() => (status = 'failed')}
		/>
	{/if}
</span>

<style>
	.cv {
		position: relative;
		display: grid;
		place-items: center;
		flex: none;
		aspect-ratio: 3 / 4;
		overflow: hidden;
		background: var(--color-steam);
		color: var(--color-on-store);
	}
	.gog {
		background: var(--color-gog);
	}
	.epic {
		background: var(--color-epic);
		color: var(--color-on-epic);
	}
	.sm {
		width: 30px;
	}
	.md {
		width: 48px;
	}
	.lg {
		width: 100%;
		max-width: 240px;
	}
	.fb {
		font-size: 13px;
	}
	.md .fb {
		font-size: 18px;
	}
	.lg .fb {
		font-size: 72px;
	}
	.logo {
		position: absolute;
		left: 12px;
		bottom: 12px;
	}
	img {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
		object-fit: cover;
		opacity: 0;
		transition: opacity 200ms ease-out;
	}
	.shown img {
		opacity: 1;
	}
	.shown .logo {
		display: none;
	}
</style>
