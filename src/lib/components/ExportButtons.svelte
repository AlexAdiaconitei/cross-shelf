<script lang="ts">
	import { save } from '@tauri-apps/plugin-dialog';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.ts';
	import type { ExportWhat } from '#lib/bindings/ExportWhat.ts';
	import { Button } from '#lib/components/ui/index.ts';
	import { ipc } from '#lib/ipc.ts';
	import { tr } from '#lib/i18n/index.ts';

	let { what, label }: { what: ExportWhat; label: string } = $props();

	let message = $state('');
	let busy = $state(false);

	async function run(format: ExportFormat) {
		message = '';
		const path = await save({
			defaultPath: `crossshelf-${what === 'library' ? 'library' : 'wishlists'}.${format}`,
			filters: [{ name: format.toUpperCase(), extensions: [format] }]
		});
		if (!path) return;
		busy = true;
		try {
			message = tr.export.done(await ipc.exportData(what, format, path));
		} catch (e) {
			const code = String(e);
			message = tr.export.failed(
				tr.export.errors[code as keyof typeof tr.export.errors] ?? tr.errors.app
			);
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex flex-wrap items-center gap-2" role="group" aria-label={label}>
	<span class="text-sm text-muted">{label}</span>
	<Button size="sm" disabled={busy} onclick={() => run('csv')}>{tr.export.csv}</Button>
	<Button size="sm" disabled={busy} onclick={() => run('json')}>{tr.export.json}</Button>
	<span class="text-sm text-muted" role="status">{message}</span>
</div>
