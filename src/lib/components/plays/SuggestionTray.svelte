<!--
	Suggestions from play time, above the board. Folds away; nothing changes without a click:
	«To Playing» / «To Paused» moves the game, the cross dismisses it for good.
-->
<script lang="ts">
	import { IconBulb, IconChevronDown, IconX } from '@tabler/icons-svelte';
	import type { Suggestion } from '#lib/bindings/Suggestion.ts';
	import Cover from '#lib/components/store/Cover.svelte';
	import { button } from '#lib/components/ui/index.ts';
	import { tr } from '#lib/i18n/index.ts';
	import type { PlayState } from '#lib/ipc.ts';
	import type { Store } from '#lib/library/shelf.ts';
	import { ago, keyOf, nowSecs } from '#lib/plays/model.ts';

	let {
		list,
		onaccept,
		ondismiss
	}: {
		list: Suggestion[];
		onaccept: (s: Suggestion) => void;
		ondismiss: (s: Suggestion) => void;
	} = $props();

	let open = $state(true);
	const now = nowSecs();
	const title = (s: Suggestion) => s.product.title ?? tr.library.untitled(s.product.native_id);
	const reason = (s: Suggestion) =>
		tr.plays.reasons[s.reason as keyof typeof tr.plays.reasons]?.(
			s.last_activity === null ? '' : ago(s.last_activity, now)
		) ?? '';
</script>

{#if list.length}
	<section class="tray" aria-label={tr.plays.suggestions(list.length)}>
		<button type="button" class="th" aria-expanded={open} onclick={() => (open = !open)}>
			<IconBulb size={16} stroke={1.75} />
			<b>{tr.plays.suggestions(list.length)}</b>
			<span>{tr.plays.suggestionsFrom}</span>
			<IconChevronDown size={16} stroke={1.75} class="chev" />
		</button>
		{#if open}
			<ul>
				{#each list as s (keyOf(s.product) + s.target)}
					<li>
						<Cover title={title(s)} store={s.product.store as Store} src={s.product.image_url} />
						<span class="sx"><b>{title(s)}</b><small>{reason(s)}</small></span>
						<button
							type="button"
							class={button({ tone: 'next', size: 'sm' })}
							onclick={() => onaccept(s)}
							>{tr.plays.toState(tr.plays.states[s.target as PlayState])}</button
						>
						<button
							type="button"
							class={button({ tone: 'ghost', size: 'icon' })}
							aria-label={tr.plays.dismiss(title(s))}
							title={tr.plays.dismiss(title(s))}
							onclick={() => ondismiss(s)}><IconX size={16} stroke={1.75} /></button
						>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
{/if}

<style>
	.tray {
		margin: 12px 32px 0;
		border: 2px solid var(--color-border-strong);
		background: var(--color-next-wash);
	}
	.th {
		display: flex;
		gap: 8px;
		align-items: center;
		width: 100%;
		min-height: 40px;
		padding: 0 12px;
		border: 0;
		background: none;
		color: inherit;
		text-align: left;
	}
	.th span {
		color: var(--color-muted);
		font-size: 13px;
	}
	.th :global(.chev) {
		margin-left: auto;
		transition: transform 150ms ease-out;
	}
	.th[aria-expanded='false'] :global(.chev) {
		transform: rotate(-90deg);
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 0 12px 12px;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 8px;
	}
	li {
		display: grid;
		grid-template-columns: 30px minmax(0, 1fr) auto auto;
		gap: 10px;
		align-items: center;
		padding: 6px 6px 6px 8px;
		background: var(--color-page);
		border: 1px solid var(--color-border);
	}
	.sx {
		display: grid;
		min-width: 0;
	}
	.sx b {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.sx small {
		font-size: 12px;
		color: var(--color-muted);
	}
	@media (prefers-reduced-motion: reduce) {
		.th :global(.chev) {
			transition: none;
		}
	}
</style>
