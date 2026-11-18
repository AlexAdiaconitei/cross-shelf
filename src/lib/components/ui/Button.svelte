<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import { button, type ButtonVariants } from './variants';

	type Props = HTMLButtonAttributes & ButtonVariants & { children: Snippet };
	let { tone, size, class: className, type = 'button', children, ...rest }: Props = $props();
</script>

<button {type} class={button({ tone, size, class: className as string })} {...rest}>
	{@render children()}
</button>
