<script lang="ts">
	import PageHeader from '#lib/components/PageHeader.svelte';
	import { onMount } from 'svelte';
	import { openUrl } from '@tauri-apps/plugin-opener';
	import type { BatchView } from '#lib/bindings/BatchView.ts';
	import { Button, InfoTip } from '#lib/components/ui/index.ts';
	import { errorText, productUrl, storeName, when } from '#lib/format.ts';
	import { asCommandError, ipc } from '#lib/ipc.ts';
	import { tr } from '#lib/i18n/index.ts';

	const r = tr.results;

	let batches = $state<BatchView[]>([]);
	let loaded = $state(false);
	let loadFailed = $state(false);
	let busy = $state<string | null>(null);
	let notes = $state<Record<string, string>>({});

	const label = <T extends Record<string, string>>(map: T, key: string) =>
		map[key as keyof T] ?? key;

	async function load() {
		try {
			batches = await ipc.plans();
		} catch {
			loadFailed = true;
		} finally {
			loaded = true;
		}
	}

	/** Reads the store's wishlist again, then checks each entry of the batch against it. */
	async function reread(bv: BatchView) {
		if (bv.store !== 'steam' && bv.store !== 'gog') return;
		busy = bv.id;
		delete notes[bv.id];
		try {
			await ipc.sync(bv.store);
			await ipc.planReconcile(bv.id);
		} catch (e) {
			notes[bv.id] = failure(r.rereadFailed, e);
		} finally {
			busy = null;
			await load();
		}
	}

	/** A readable sentence for any command error: never the raw text of the error. */
	function failure(prefix: string, e: unknown): string {
		const code = typeof e === 'string' ? e : asCommandError(e).code;
		if (code in tr.batch.errors) return tr.batch.errors[code as keyof typeof tr.batch.errors];
		return `${prefix} (${errorText(code in tr.errors ? code : 'app')})`;
	}

	async function run(bv: BatchView, job: () => Promise<unknown>, prefix: string = r.actionFailed) {
		busy = bv.id;
		delete notes[bv.id];
		try {
			await job();
		} catch (e) {
			notes[bv.id] = failure(prefix, e);
		} finally {
			busy = null;
			await load();
		}
	}

	/** Sends what is pending, then refreshes the local wishlist so every view agrees. */
	async function sendAndReread(bv: BatchView) {
		await ipc.planExecute(bv.id);
		if (bv.store === 'gog' || bv.store === 'steam') await ipc.sync(bv.store);
	}

	const send = (bv: BatchView) => run(bv, () => sendAndReread(bv), r.sendFailed);
	const retry = (bv: BatchView) =>
		run(
			bv,
			async () => {
				await ipc.planRetry(bv.id);
				await sendAndReread(bv);
			},
			r.sendFailed
		);
	const cancel = (bv: BatchView) => run(bv, () => ipc.planCancel(bv.id));
	const discard = (bv: BatchView) => run(bv, () => ipc.planDiscard(bv.id));

	const retryable = (bv: BatchView) =>
		bv.state === 'done' &&
		bv.write_validated &&
		bv.ops.some((o) => ['failed', 'unverified', 'needs_reauth'].includes(o.status));

	const statusClass = (s: string) =>
		s === 'already_in_desired_state' || s === 'applied'
			? 'text-success'
			: s === 'pending'
				? 'text-muted'
				: 'text-warning';

	onMount(load);
</script>

<PageHeader title={tr.nav.resultados}>
	{#snippet info()}<InfoTip label={tr.nav.resultados} lines={[r.intro]} />{/snippet}
</PageHeader>
<div class="page-body">
	{#if loadFailed}
		<p role="alert" class="text-danger">{tr.library.loadError}</p>
	{:else if loaded && batches.length === 0}
		<div class="rounded-panel border border-border p-6">
			<p class="font-medium">{r.empty}</p>
			<p class="text-muted">{r.emptyBody}</p>
		</div>
	{:else}
		<div class="grid max-w-4xl divide-y divide-border border-y border-border">
			{#each batches as bv (bv.id)}
				<section aria-labelledby="batch-{bv.id}" class="grid gap-3 py-5">
					<header class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
						<h2 id="batch-{bv.id}" class="font-semibold">
							{tr.batch.summary(tr.batch.part(bv.ops.length, storeName(bv.store)))}
						</h2>
						<p class="text-sm text-muted">
							{label(r.state, bv.state)} · {r.created(when(bv.created_at))}
						</p>
					</header>

					<ul class="grid gap-1.5 text-sm">
						{#each bv.ops as o (o.op_id)}
							<li class="flex flex-wrap items-center justify-between gap-2">
								<div class="min-w-0">
									<p class="truncate">{o.title ?? tr.library.untitled(o.native_id)}</p>
									<p class={statusClass(o.status)}>
										{o.status === 'pending' && bv.write_validated
											? r.op.pendingSend
											: label(r.op, o.status)}
									</p>
								</div>
								{#if bv.state === 'confirmed' && o.status === 'pending' && !bv.write_validated}
									<Button
										size="sm"
										onclick={() => openUrl(productUrl(bv.store, o.native_id, null)).catch(() => {})}
										>{r.openStore}</Button
									>
								{/if}
							</li>
						{/each}
					</ul>

					<div class="flex flex-wrap items-center gap-2">
						{#if bv.state === 'confirmed' && bv.write_validated}
							<Button tone="danger" disabled={busy !== null} onclick={() => send(bv)}>
								{r.resume}
							</Button>
							<Button tone="ghost" disabled={busy !== null} onclick={() => cancel(bv)}>
								{r.cancel}
							</Button>
							<InfoTip label={r.sendHint} lines={[r.sendHint]} />
						{:else if retryable(bv)}
							<Button disabled={busy !== null} onclick={() => retry(bv)}>{r.retry}</Button>
						{:else if bv.state === 'confirmed'}
							<Button tone="primary" disabled={busy !== null} onclick={() => reread(bv)}>
								{r.reread}
							</Button>
							<InfoTip label={r.rereadHint} lines={[r.rereadHint]} />
						{/if}
						{#if bv.state === 'draft' || bv.state === 'invalidated'}
							<Button tone="ghost" disabled={busy !== null} onclick={() => discard(bv)}>
								{r.discard}
							</Button>
						{/if}
					</div>
					<p class="text-sm text-warning" role="status">
						{busy === bv.id ? tr.connections.working : (notes[bv.id] ?? '')}
					</p>
				</section>
			{/each}
		</div>
	{/if}
</div>
