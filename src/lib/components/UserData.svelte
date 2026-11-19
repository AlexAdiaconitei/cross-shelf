<!--
	Ajustes › My data: export to a JSON file, or import one. Importing shows what the file brings
	and asks before replacing anything; the backend copies the user file first.
-->
<script lang="ts">
	import { open, save } from '@tauri-apps/plugin-dialog';
	import type { UserDataSummary } from '#lib/bindings/UserDataSummary.ts';
	import { Button, Dialog, InfoTip } from '#lib/components/ui/index.ts';
	import { when } from '#lib/format.ts';
	import { tr } from '#lib/i18n/index.ts';
	import { ipc } from '#lib/ipc.ts';
	import { refreshShell } from '#lib/shell/state.svelte.ts';

	const m = tr.settings.mine;
	let message = $state('');
	let busy = $state(false);
	let pending = $state<{ path: string; summary: UserDataSummary } | null>(null);
	let confirmOpen = $state(false);

	const errorText = (e: unknown) =>
		m.errors[String(e) as keyof typeof m.errors] ?? m.errors.internal;

	const stamp = () => new Date().toISOString().slice(0, 10);

	async function exportData() {
		message = '';
		const path = await save({
			defaultPath: `crossshelf-my-data-${stamp()}.json`,
			filters: [{ name: 'JSON', extensions: ['json'] }]
		});
		if (!path) return;
		busy = true;
		try {
			await ipc.userdataExport(path);
			message = m.exported;
		} catch (e) {
			message = errorText(e);
		} finally {
			busy = false;
		}
	}

	async function pick() {
		message = '';
		const path = await open({
			multiple: false,
			directory: false,
			filters: [{ name: 'JSON', extensions: ['json'] }]
		});
		if (typeof path !== 'string') return;
		try {
			pending = { path, summary: await ipc.userdataPreview(path) };
			confirmOpen = true;
		} catch (e) {
			message = errorText(e);
		}
	}

	async function importData() {
		if (!pending) return;
		confirmOpen = false;
		busy = true;
		try {
			await ipc.userdataImport(pending.path);
			message = m.imported;
			// The language may have changed with the settings: the shell reloads if so.
			await refreshShell();
		} catch (e) {
			message = errorText(e);
		} finally {
			busy = false;
			pending = null;
		}
	}

	const lines = (s: UserDataSummary) => [
		m.counts.plays(s.plays),
		m.counts.relations(s.relations),
		m.counts.keeps(s.keeps),
		m.counts.dismissed(s.dismissed),
		m.counts.snapshots(s.snapshots),
		m.counts.settings(s.settings)
	];
</script>

<section aria-labelledby="mine" class="grid gap-3">
	<h2 id="mine" class="font-medium">
		{m.title}
		<InfoTip label={m.title} lines={[m.hint]} />
	</h2>
	<div class="flex flex-wrap gap-2">
		<Button disabled={busy} onclick={exportData}>{m.export}</Button>
		<Button disabled={busy} onclick={pick}>{m.import}</Button>
	</div>
	<p class="text-sm text-muted" role="status">{message}</p>
</section>

<Dialog
	bind:open={confirmOpen}
	title={m.importTitle}
	description={pending ? m.importBody(when(pending.summary.exported_at)) : ''}
	closeLabel={tr.connections.cancel}
>
	{#if pending}
		<ul class="grid list-disc gap-1 pl-5 text-sm">
			{#each lines(pending.summary) as l (l)}<li>{l}</li>{/each}
		</ul>
	{/if}
	{#snippet footer()}
		<Button onclick={() => (confirmOpen = false)}>{tr.connections.cancel}</Button>
		<Button tone="danger" onclick={importData}>{m.confirm}</Button>
	{/snippet}
</Dialog>
