<script lang="ts">
	import PageHeader from '#lib/components/PageHeader.svelte';
	import { onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { resolve } from '$app/paths';
	import type { Settings } from '#lib/bindings/Settings.ts';
	import ExportButtons from '#lib/components/ExportButtons.svelte';
	import Updates from '#lib/components/Updates.svelte';
	import UserData from '#lib/components/UserData.svelte';
	import { Button, Dialog, InfoTip } from '#lib/components/ui/index.ts';
	import { ipc } from '#lib/ipc.ts';
	import { shell } from '#lib/shell/state.svelte.ts';
	import { isLang, lang, languages, rememberLang, tr } from '#lib/i18n/index.ts';

	type AppStatus = { version: string; schema_version: number; updater: boolean };

	let status = $state<AppStatus | null>(null);
	let failed = $state(false);
	let settings = $state<Settings | null>(null);
	let settingsNote = $state('');
	let dataNote = $state('');
	let clearOpen = $state(false);

	async function setFamily(value: boolean) {
		if (!settings) return;
		try {
			settings = await ipc.saveSettings({ ...settings, include_family_in_cleanup: value });
			settingsNote = tr.settings.saved;
		} catch {
			settingsNote = tr.settings.failed;
		}
	}

	async function setLanguage(value: string) {
		if (!settings || !isLang(value)) return;
		try {
			settings = await ipc.saveSettings({ ...settings, language: value });
			if (rememberLang(value)) location.reload();
		} catch {
			settingsNote = tr.settings.failed;
		}
	}

	async function setCovers(value: boolean) {
		if (!settings) return;
		try {
			settings = await ipc.saveSettings({ ...settings, show_covers: value });
			shell.showCovers = settings.show_covers;
			settingsNote = tr.settings.saved;
		} catch {
			settingsNote = tr.settings.failed;
		}
	}

	async function clearCache() {
		clearOpen = false;
		try {
			await ipc.clearCache();
			dataNote = tr.settings.cleared;
		} catch {
			dataNote = tr.settings.clearFailed;
		}
	}

	onMount(() => {
		invoke<AppStatus>('app_status')
			.then((s) => (status = s))
			.catch(() => (failed = true));
		ipc
			.settings()
			.then((s) => (settings = s))
			.catch(() => (settingsNote = tr.settings.failed));
	});
</script>

<PageHeader title={tr.nav.ajustes} />
<div class="page-body">
	<div class="grid max-w-prose gap-8">
		<section aria-labelledby="cleanup" class="grid gap-2">
			<h2 id="cleanup" class="font-medium">{tr.settings.cleanup}</h2>
			{#if settings}
				<div class="flex items-center gap-1">
					<label class="flex items-center gap-3">
						<input
							type="checkbox"
							class="size-4 accent-accent"
							checked={settings.include_family_in_cleanup}
							onchange={(e) => setFamily(e.currentTarget.checked)}
						/>
						<span>{tr.settings.includeFamily}</span>
					</label>
					<InfoTip label={tr.settings.includeFamily} lines={[tr.settings.includeFamilyHint]} />
				</div>
			{/if}
			<p class="text-sm text-muted" role="status">{settingsNote}</p>
		</section>

		<section aria-labelledby="appearance" class="grid gap-2">
			<h2 id="appearance" class="font-medium">{tr.settings.appearance}</h2>
			{#if settings}
				<div class="flex items-center gap-3">
					<label for="language">{tr.settings.language}</label>
					<select
						id="language"
						class="control"
						value={lang}
						onchange={(e) => setLanguage(e.currentTarget.value)}
					>
						{#each languages as l (l.value)}<option value={l.value}>{l.label}</option>{/each}
					</select>
					<InfoTip label={tr.settings.language} lines={[tr.settings.languageHint]} />
				</div>
				<div class="flex items-center gap-1">
					<label class="flex items-center gap-3">
						<input
							type="checkbox"
							class="size-4 accent-accent"
							checked={settings.show_covers}
							onchange={(e) => setCovers(e.currentTarget.checked)}
						/>
						<span>{tr.settings.showCovers}</span>
					</label>
					<InfoTip label={tr.settings.showCovers} lines={[tr.settings.showCoversHint]} />
				</div>
			{/if}
		</section>

		<section aria-labelledby="export" class="grid gap-3">
			<h2 id="export" class="font-medium">{tr.settings.exportTitle}</h2>
			<ExportButtons what="library" label={tr.library.export} />
			<ExportButtons what="wishlist" label={tr.wishlist.export} />
		</section>

		<UserData />

		<section aria-labelledby="data" class="grid gap-3">
			<h2 id="data" class="font-medium">
				{tr.settings.data}
				<InfoTip label={tr.settings.data} lines={[tr.settings.dataHint]} />
			</h2>
			<div class="flex flex-wrap gap-2">
				<Button
					onclick={() =>
						ipc.openDataFolder().catch(() => (dataNote = tr.settings.openFolderFailed))}
				>
					{tr.settings.openFolder}
				</Button>
				<Button tone="danger" onclick={() => (clearOpen = true)}>{tr.settings.clearCache}</Button>
			</div>
			<p class="text-sm text-muted" role="status">{dataNote}</p>
		</section>

		<section aria-labelledby="updates" class="grid gap-2">
			<h2 id="updates" class="font-medium">{tr.settings.updates}</h2>
			{#if status}<Updates enabled={status.updater} />{/if}
		</section>

		<section aria-labelledby="about" class="grid gap-2">
			<h2 id="about" class="font-medium">{tr.settings.about}</h2>
			{#if failed}
				<p role="alert" class="text-danger">{tr.settings.statusError}</p>
			{:else if status}
				<dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-muted">
					<dt>{tr.settings.version}</dt>
					<dd class="text-text">{status.version}</dd>
				</dl>
			{/if}
			<p>
				<a class="text-accent underline underline-offset-2" href={resolve('/ajustes/diagnostico')}
					>{tr.settings.diagnosticsLink}</a
				>
			</p>
		</section>
	</div>
</div>

<Dialog
	bind:open={clearOpen}
	title={tr.settings.clearTitle}
	description={tr.settings.clearBody}
	closeLabel={tr.connections.cancel}
>
	{#snippet footer()}
		<Button onclick={() => (clearOpen = false)}>{tr.connections.cancel}</Button>
		<Button tone="danger" onclick={clearCache}>{tr.settings.clearCache}</Button>
	{/snippet}
</Dialog>
