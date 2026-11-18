<script lang="ts">
	import { Field } from '@ark-ui/svelte/field';
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { input } from './variants';

	type Props = Omit<HTMLInputAttributes, 'value' | 'class'> & {
		label: string;
		value?: string;
		hint?: string;
		error?: string;
	};
	let { label, value = $bindable(''), hint, error, ...rest }: Props = $props();
</script>

<!-- Ark wires label, input, hint and error together (ids, aria-describedby, aria-invalid). -->
<Field.Root invalid={!!error} class="grid gap-1.5">
	<Field.Label class="text-sm font-medium">{label}</Field.Label>
	<Field.Input bind:value class={input()} {...rest} />
	{#if hint}<Field.HelperText class="text-xs leading-relaxed text-muted">{hint}</Field.HelperText
		>{/if}
	{#if error}<Field.ErrorText class="text-xs text-danger">{error}</Field.ErrorText>{/if}
</Field.Root>
