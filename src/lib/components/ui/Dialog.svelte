<script lang="ts">
	import { Dialog } from '@ark-ui/svelte/dialog';
	import { Portal } from '@ark-ui/svelte/portal';
	import { IconX } from '@tabler/icons-svelte';
	import type { Snippet } from 'svelte';
	import { button } from './variants';

	type Props = {
		open?: boolean;
		title: string;
		description?: string;
		/** `sheet` slides from the bottom on phones and is a side panel from `sm` up. */
		variant?: 'dialog' | 'sheet';
		closeLabel?: string;
		children?: Snippet;
		footer?: Snippet;
	};
	let {
		open = $bindable(false),
		title,
		description,
		variant = 'dialog',
		closeLabel = 'Close',
		children,
		footer
	}: Props = $props();

	const positioner = {
		dialog: 'fixed inset-0 z-51 grid place-items-center overflow-y-auto p-4',
		sheet: 'fixed inset-0 z-51 flex items-end sm:items-stretch sm:justify-end'
	};
	const content = {
		dialog: 'w-full max-w-120 rounded-dialog max-h-[calc(100dvh-2rem)]',
		sheet: 'w-full rounded-t-dialog max-h-[90dvh] sm:max-h-none sm:w-96 sm:rounded-none'
	};
</script>

<!-- Focus trap, Escape, scroll lock and return of focus come from Ark. -->
<Dialog.Root bind:open lazyMount unmountOnExit>
	<Portal>
		<Dialog.Backdrop class="fixed inset-0 z-50 bg-black/65" />
		<Dialog.Positioner class={positioner[variant]}>
			<Dialog.Content
				class="relative overflow-y-auto border border-border bg-panel p-6 shadow-overlay {content[
					variant
				]}"
			>
				<header class="mb-5 flex items-start justify-between gap-3">
					<div>
						<Dialog.Title class="text-lg font-semibold tracking-tight">{title}</Dialog.Title>
						{#if description}
							<Dialog.Description class="mt-1.5 text-sm leading-relaxed text-muted">
								{description}
							</Dialog.Description>
						{/if}
					</div>
					<Dialog.CloseTrigger
						aria-label={closeLabel}
						class={button({ tone: 'ghost', size: 'icon' })}
					>
						<IconX size={18} />
					</Dialog.CloseTrigger>
				</header>
				{@render children?.()}
				{#if footer}
					<footer class="mt-6 flex flex-wrap justify-end gap-2.5">{@render footer()}</footer>
				{/if}
			</Dialog.Content>
		</Dialog.Positioner>
	</Portal>
</Dialog.Root>
