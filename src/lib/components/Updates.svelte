<script lang="ts">
	import { check, type Update } from '@tauri-apps/plugin-updater';
	import { relaunch } from '@tauri-apps/plugin-process';
	import { Button } from '#lib/components/ui/index.ts';
	import { es } from '#lib/strings.ts';

	/** Whether this build carries the signed updater configuration (from `app_status`). */
	let { enabled }: { enabled: boolean } = $props();

	const t = es.updates;
	let phase = $state<'idle' | 'checking' | 'none' | 'found' | 'installing' | 'failed'>('idle');
	let found = $state<Update | null>(null);
	let progress = $state('');

	async function look() {
		phase = 'checking';
		try {
			found = await check();
			phase = found ? 'found' : 'none';
		} catch {
			phase = 'failed';
		}
	}

	/** Downloads, verifies the signature (the plugin refuses a bad one) and restarts. */
	async function install() {
		if (!found) return;
		phase = 'installing';
		let total = 0;
		let done = 0;
		try {
			await found.downloadAndInstall((e) => {
				if (e.event === 'Started') total = e.data.contentLength ?? 0;
				if (e.event === 'Progress') {
					done += e.data.chunkLength;
					progress = total > 0 ? t.progress(Math.round((done / total) * 100)) : '';
				}
			});
			await relaunch();
		} catch {
			// A failed or refused download leaves the installed version as it was.
			phase = 'failed';
		}
	}
</script>

{#if !enabled}
	<p class="text-sm text-muted">{t.unsigned}</p>
{:else}
	<div class="flex flex-wrap items-center gap-3">
		{#if phase === 'found' && found}
			<Button tone="primary" onclick={install}>{t.install(found.version)}</Button>
		{:else}
			<Button disabled={phase === 'checking' || phase === 'installing'} onclick={look}>
				{t.check}
			</Button>
		{/if}
	</div>
	<p class="text-sm text-muted" role="status">
		{#if phase === 'checking'}{t.checking}
		{:else if phase === 'none'}{t.none}
		{:else if phase === 'found' && found}{t.found(found.version)}
		{:else if phase === 'installing'}{t.installing} {progress}
		{:else if phase === 'failed'}{t.failed}
		{/if}
	</p>
{/if}
