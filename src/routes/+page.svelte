<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import type { ConnectionView } from '#lib/bindings/ConnectionView.ts';
	import { Button, Dialog } from '#lib/components/ui/index.ts';
	import { capabilityText, errorText, storeName, when } from '#lib/format.ts';
	import { asCommandError, ipc, type Provider } from '#lib/ipc.ts';
	import { es } from '#lib/strings.ts';

	let connections = $state<ConnectionView[]>([]);
	let loadFailed = $state(false);
	/** Store being worked on and what is happening, one at a time. */
	let busy = $state<{ provider: string; what: 'login' | 'sync' | 'other' } | null>(null);
	let notes = $state<Record<string, { text: string; warn: boolean }>>({});
	let confirming = $state<Provider | null>(null);
	let confirmOpen = $state(false);

	const toneClass = { ok: 'text-success', warn: 'text-warning', muted: 'text-muted' };

	async function load() {
		try {
			connections = await ipc.connections();
			loadFailed = false;
		} catch {
			loadFailed = true;
		}
	}

	function needsReauth(c: ConnectionView): boolean {
		return c.capabilities.some((x) => x.state === 'failed' && x.error_code === 'needs_reauth');
	}

	async function act(p: Provider, what: 'login' | 'sync' | 'other', job: () => Promise<unknown>) {
		busy = { provider: p, what };
		delete notes[p];
		try {
			await job();
		} catch (e) {
			const err = asCommandError(e);
			notes[p] = {
				warn: true,
				text:
					err.code === 'cancelled'
						? es.connections.cancelled
						: err.code in es.errors
							? errorText(err.code)
							: err.code === 'internal'
								? errorText('app')
								: err.message
			};
		} finally {
			busy = null;
			await load();
		}
	}

	const connect = (p: Provider) =>
		act(p, 'login', async () => {
			const r = await ipc.connect(p);
			if (r.account_changed) notes[p] = { warn: true, text: es.connections.accountChanged };
		});

	const sync = (p: Provider) =>
		act(p, 'sync', async () => {
			const r = await ipc.sync(p);
			notes[p] = r.account_changed
				? { warn: true, text: es.connections.accountChanged }
				: { warn: false, text: es.connections.syncDone };
		});

	function askDisconnect(p: Provider) {
		confirming = p;
		confirmOpen = true;
	}

	async function disconnect() {
		const p = confirming;
		confirmOpen = false;
		if (p) await act(p, 'other', () => ipc.disconnect(p));
	}

	onMount(load);
</script>

<h1 class="mb-2 text-xl font-semibold">{es.nav.conexiones}</h1>
<p class="mb-6 max-w-prose text-muted">{es.connections.intro}</p>

{#if loadFailed}
	<p role="alert" class="text-danger">{es.library.loadError}</p>
{/if}

<div class="grid max-w-3xl divide-y divide-border border-y border-border">
	{#each connections as c (c.provider)}
		{@const p = c.provider as Provider}
		{@const isConnected = c.state === 'connected'}
		{@const working = busy?.provider === p}
		<section aria-labelledby="conn-{p}" class="grid gap-3 py-5">
			<header class="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-1">
				<h2 id="conn-{p}" class="text-lg font-semibold">{storeName(p)}</h2>
				<p class="text-sm {isConnected ? 'text-success' : 'text-muted'}">
					{isConnected ? es.connections.connected : es.connections.disconnected}
				</p>
			</header>

			{#if c.account_id}
				<p class="text-sm text-muted">
					{c.display_name
						? es.connections.account(c.display_name)
						: es.connections.accountId(c.account_id)}
					{#if c.last_sync_at}· {es.connections.lastSync(when(c.last_sync_at))}{/if}
				</p>
			{/if}

			<dl
				class="grid grid-cols-[minmax(9rem,auto)_1fr] gap-x-4 gap-y-1.5 text-sm"
				aria-label={es.connections.capabilitiesLabel(storeName(p))}
			>
				{#each c.capabilities as cap (cap.capability)}
					{@const s = capabilityText(cap)}
					<dt class="text-muted">
						{es.capabilities[cap.capability as keyof typeof es.capabilities] ?? cap.capability}
					</dt>
					<dd class={toneClass[s.tone]}>{s.text}</dd>
				{/each}
			</dl>

			{#if c.other_account_data}
				<p class="text-sm text-muted">{es.connections.otherAccountData}</p>
			{/if}

			<div class="flex flex-wrap gap-2">
				{#if !isConnected || needsReauth(c)}
					<Button tone="primary" disabled={busy !== null} onclick={() => connect(p)}>
						{isConnected || c.account_id ? es.connections.reconnect : es.connections.connect}
					</Button>
				{/if}
				{#if isConnected}
					<Button
						tone={needsReauth(c) ? 'quiet' : 'primary'}
						disabled={busy !== null}
						onclick={() => sync(p)}>{es.connections.sync}</Button
					>
					<Button tone="ghost" disabled={busy !== null} onclick={() => askDisconnect(p)}>
						{es.connections.disconnect}
					</Button>
				{/if}
			</div>

			<p class="text-sm" role="status">
				{#if working}
					<span class="text-muted">
						{busy?.what === 'login' ? es.connections.waitingLogin : es.connections.working}
					</span>
				{:else if notes[p]}
					<span class={notes[p].warn ? 'text-warning' : 'text-muted'}>{notes[p].text}</span>
					{#if !notes[p].warn}
						<a class="ml-1 text-accent underline underline-offset-2" href={resolve('/limpieza')}
							>{es.connections.nextStep}</a
						>
					{/if}
				{/if}
			</p>
		</section>
	{/each}
</div>

<Dialog
	bind:open={confirmOpen}
	title={es.connections.disconnectTitle(storeName(confirming ?? ''))}
	description={es.connections.disconnectBody}
	closeLabel={es.connections.cancel}
>
	{#snippet footer()}
		<Button onclick={() => (confirmOpen = false)}>{es.connections.cancel}</Button>
		<Button tone="danger" onclick={disconnect}>{es.connections.disconnect}</Button>
	{/snippet}
</Dialog>
