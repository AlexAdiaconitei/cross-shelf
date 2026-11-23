<!-- «Move to…»: the five states plus removing the play. Works without dragging. -->
<script lang="ts">
	import { tr } from '#lib/i18n/index.ts';
	import { PLAY_STATES } from '#lib/ipc.ts';

	let {
		title,
		state,
		onmove,
		onremove
	}: {
		title: string;
		state: string;
		onmove: (state: string) => void;
		onremove: () => void;
	} = $props();

	const REMOVE = 'remove';
</script>

<select
	class="mv"
	aria-label={tr.plays.move(title)}
	value={state}
	onchange={(e) => {
		const v = e.currentTarget.value;
		if (v === REMOVE) onremove();
		else onmove(v);
	}}
>
	{#each PLAY_STATES as s (s)}<option value={s}>{tr.plays.states[s]}</option>{/each}
	<option value={REMOVE}>{tr.plays.remove}</option>
</select>

<style>
	.mv {
		flex: 0 1 120px;
		min-width: 0;
		height: 28px;
		max-width: 120px;
		padding: 0 4px;
		font-size: 12px;
		border: 1px solid var(--color-border);
		background: var(--color-page);
		color: var(--color-text);
	}
</style>
