<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import type { WriteTestResult } from '#lib/bindings/WriteTestResult.ts';
	import { Button, Dialog } from '#lib/components/ui/index.ts';
	import { tr } from '#lib/i18n/index.ts';

	const t = tr.writeTest;

	let store = $state<'steam' | 'gog'>('steam');
	let productId = $state('');
	let present = $state(false);
	let confirmOpen = $state(false);
	let busy = $state(false);
	let result = $state('');

	const validId = $derived(/^\d+$/.test(productId.trim()));
	const storeLabel = $derived(tr.stores[store]);

	async function run() {
		confirmOpen = false;
		busy = true;
		result = '';
		try {
			const r = await invoke<WriteTestResult>('wishlist_write_test', {
				store,
				nativeId: productId.trim(),
				present
			});
			result = t.result(r.status);
		} catch (e) {
			result = t.failed(String(e));
		} finally {
			busy = false;
		}
	}
</script>

<section aria-labelledby="write-test" class="grid max-w-2xl gap-3">
	<h2 id="write-test" class="font-medium">{t.title}</h2>
	<p class="text-sm text-muted">{t.intro}</p>
	<div class="flex flex-wrap items-end gap-3">
		<fieldset class="grid gap-1 text-sm">
			<legend class="mb-1 text-muted">{tr.library.store}</legend>
			<div class="flex gap-4 py-2">
				<label class="flex items-center gap-1.5">
					<input type="radio" name="write-store" value="steam" bind:group={store} /> Steam
				</label>
				<label class="flex items-center gap-1.5">
					<input type="radio" name="write-store" value="gog" bind:group={store} /> GOG
				</label>
			</div>
		</fieldset>
		<label class="grid gap-1 text-sm">
			<span class="text-muted">{t.productId}</span>
			<input
				class="rounded-control border border-border bg-panel px-3 py-2"
				inputmode="numeric"
				bind:value={productId}
			/>
		</label>
		<fieldset class="grid gap-1 text-sm">
			<legend class="mb-1 text-muted">{t.action}</legend>
			<div class="flex gap-4 py-2">
				<label class="flex items-center gap-1.5">
					<input type="radio" name="write-action" value={false} bind:group={present} />
					{t.remove}
				</label>
				<label class="flex items-center gap-1.5">
					<input type="radio" name="write-action" value={true} bind:group={present} />
					{t.add}
				</label>
			</div>
		</fieldset>
		<Button tone="danger" disabled={busy || !validId} onclick={() => (confirmOpen = true)}>
			{t.run}
		</Button>
	</div>
	<p class="text-sm" role="status">{busy ? tr.diag.running : result}</p>
</section>

<Dialog
	bind:open={confirmOpen}
	title={t.confirmTitle}
	description={t.confirmBody(present ? t.add : t.remove, productId.trim(), storeLabel)}
	closeLabel={tr.connections.cancel}
>
	{#snippet footer()}
		<Button onclick={() => (confirmOpen = false)}>{tr.connections.cancel}</Button>
		<Button tone="danger" onclick={run}>{t.confirm}</Button>
	{/snippet}
</Dialog>
