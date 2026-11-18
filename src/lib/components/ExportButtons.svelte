<script lang="ts">
	import { save } from '@tauri-apps/plugin-dialog';
	import type { ExportFormat } from '#lib/bindings/ExportFormat.ts';
	import type { ExportWhat } from '#lib/bindings/ExportWhat.ts';
	import { Button } from '#lib/components/ui/index.ts';
	import { ipc } from '#lib/ipc.ts';
	import { es } from '#lib/strings.ts';

	let { what, label }: { what: ExportWhat; label: string } = $props();

	let message = $state('');
	let busy = $state(false);

	async function run(format: ExportFormat) {
		message = '';
		const path = await save({
			defaultPath: `crossshelf-${what === 'library' ? 'biblioteca' : 'wishlists'}.${format}`,
			filters: [{ name: format.toUpperCase(), extensions: [format] }]
		});
		if (!path) return;
		busy = true;
		try {
			message = es.export.done(await ipc.exportData(what, format, path));
		} catch (e) {
			message = es.export.failed(String(e));
		} finally {
			busy = false;
		}
	}
</script>

<div class="flex flex-wrap items-center gap-2" role="group" aria-label={label}>
	<span class="text-sm text-muted">{label}</span>
	<Button size="sm" disabled={busy} onclick={() => run('csv')}>{es.export.csv}</Button>
	<Button size="sm" disabled={busy} onclick={() => run('json')}>{es.export.json}</Button>
	<span class="text-sm text-muted" role="status">{message}</span>
</div>
