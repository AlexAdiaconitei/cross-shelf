<script lang="ts">
	import '../app.css';
	import { page } from '$app/state';
	import { resolve } from '$app/paths';
	import { es } from '#lib/strings.ts';
	import { navItems } from '#lib/nav.ts';

	let { children } = $props();
</script>

<div class="grid h-screen grid-cols-[14rem_1fr] bg-page text-text">
	<nav aria-label={es.appName} class="flex flex-col gap-1 border-r border-border bg-panel p-3">
		<p class="px-3 py-2 text-lg font-semibold">{es.appName}</p>
		<ul class="grid gap-1">
			{#each navItems as item (item.href)}
				{@const current = page.url.pathname === item.href}
				<li>
					<a
						href={resolve(item.href)}
						aria-current={current ? 'page' : undefined}
						class="block rounded-control px-3 py-2 text-muted focus-visible:outline-2 focus-visible:outline-accent aria-[current=page]:bg-raised aria-[current=page]:text-text"
					>
						{item.label}
					</a>
				</li>
			{/each}
		</ul>
	</nav>
	<main class="overflow-y-auto p-6">
		{@render children()}
	</main>
</div>
