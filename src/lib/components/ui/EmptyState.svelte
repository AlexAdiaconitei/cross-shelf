<script lang="ts">
	import type { Snippet } from 'svelte';

	let { title, children, action }: { title: string; children?: Snippet; action?: Snippet } =
		$props();
</script>

<div class="grid justify-items-start gap-3 p-7 leading-relaxed text-muted">
	<p class="font-medium text-text">{title}</p>
	{#if children}{@render children()}{/if}
	{#if action}{@render action()}{/if}
</div>
