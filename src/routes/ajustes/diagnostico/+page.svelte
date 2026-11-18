<script lang="ts">
	import PageHeader from '#lib/components/PageHeader.svelte';
	import { invoke } from '@tauri-apps/api/core';
	import WriteTest from '#lib/components/WriteTest.svelte';
	import { tr } from '#lib/i18n/index.ts';

	type Status = { connected: boolean };
	type CommandError = { code: string; message: string };
	type Provider = {
		id: string;
		name: string;
		expire: boolean;
		extras?: { label: string; command: string }[];
	};

	const providers: Provider[] = [
		{
			id: 'gog',
			name: tr.diag.gog,
			expire: true,
			extras: [{ label: tr.diag.sync, command: 'gog_sync' }]
		},
		{
			id: 'steam',
			name: tr.diag.steam,
			expire: false,
			extras: [
				{ label: tr.diag.sync, command: 'steam_sync' },
				{ label: tr.diag.types, command: 'steam_diag_types' }
			]
		},
		{
			id: 'epic',
			name: tr.diag.epic,
			expire: true,
			extras: [{ label: tr.diag.catalog, command: 'epic_diag_catalog' }]
		}
	];

	let connected = $state<Record<string, boolean>>({});
	let outputs = $state<Record<string, string>>({});
	let busy = $state(false);

	async function call<T>(id: string, command: string): Promise<T | undefined> {
		busy = true;
		try {
			const result = await invoke<T>(command);
			outputs[id] = JSON.stringify(result ?? null, null, 2);
			return result;
		} catch (e) {
			const err = e as CommandError;
			outputs[id] = `Error (${err.code ?? tr.errors.unknown}): ${err.message ?? String(e)}`;
		} finally {
			busy = false;
		}
	}

	async function refreshStatus(id: string) {
		const s = await call<Status>(id, `${id}_status`);
		if (s) connected[id] = s.connected;
	}

	async function connect(id: string) {
		const s = await call<Status>(id, `${id}_connect`);
		if (s) connected[id] = s.connected;
	}

	async function disconnect(id: string) {
		await call(id, `${id}_disconnect`);
		await refreshStatus(id);
	}

	$effect(() => {
		for (const p of providers) refreshStatus(p.id);
	});
</script>

<PageHeader title={tr.diag.title} />
<div class="page-body">
	<p class="mb-6 max-w-prose text-muted">{tr.diag.intro}</p>

	<div class="grid max-w-2xl gap-8">
		{#each providers as p (p.id)}
			<section aria-labelledby={p.id} class="grid gap-3">
				<h2 id={p.id} class="font-medium">
					{p.name}
					<span class="ml-2 text-sm text-muted">
						{connected[p.id] ? tr.diag.connected : tr.diag.notConnected}
					</span>
				</h2>
				<div class="flex flex-wrap gap-2">
					<button
						class="rounded-control bg-accent px-3 py-2 text-on-accent disabled:opacity-50"
						disabled={busy}
						onclick={() => connect(p.id)}>{tr.diag.connect}</button
					>
					<button
						class="rounded-control bg-raised px-3 py-2 disabled:opacity-50"
						disabled={busy || !connected[p.id]}
						onclick={() => call(p.id, `${p.id}_diag_run`)}>{tr.diag.run}</button
					>
					{#if p.expire}
						<button
							class="rounded-control bg-raised px-3 py-2 disabled:opacity-50"
							disabled={busy || !connected[p.id]}
							onclick={() => call(p.id, `${p.id}_diag_expire`)}>{tr.diag.expire}</button
						>
					{/if}
					<button
						class="rounded-control bg-raised px-3 py-2 disabled:opacity-50"
						disabled={busy || !connected[p.id]}
						onclick={() => call(p.id, `${p.id}_diag_shape`)}>{tr.diag.shape}</button
					>
					{#each p.extras ?? [] as x (x.command)}
						<button
							class="rounded-control bg-raised px-3 py-2 disabled:opacity-50"
							disabled={busy || !connected[p.id]}
							onclick={() => call(p.id, x.command)}>{x.label}</button
						>
					{/each}
					<button
						class="rounded-control bg-raised px-3 py-2 disabled:opacity-50"
						disabled={busy}
						onclick={() => disconnect(p.id)}>{tr.diag.disconnect}</button
					>
				</div>
				{#if busy}<p class="text-muted" role="status">{tr.diag.running}</p>{/if}
				{#if outputs[p.id]}
					<pre
						class="rounded-panel border border-border bg-panel p-3 font-mono text-sm break-all whitespace-pre-wrap">{outputs[
							p.id
						]}</pre>
				{/if}
			</section>
		{/each}
	</div>

	<div class="mt-10"><WriteTest /></div>
</div>
