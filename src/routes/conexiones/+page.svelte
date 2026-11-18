<script lang="ts">
	import { onMount } from 'svelte';
	import { resolve } from '$app/paths';
	import type { ConnectionView } from '#lib/bindings/ConnectionView.ts';
	import PageHeader from '#lib/components/PageHeader.svelte';
	import StoreLogo from '#lib/components/store/StoreLogo.svelte';
	import { Button, Dialog, InfoTip } from '#lib/components/ui/index.ts';
	import { capabilityLine, errorText, storeName } from '#lib/format.ts';
	import { asCommandError, ipc, type Provider } from '#lib/ipc.ts';
	import { intl, tr } from '#lib/i18n/index.ts';

	let connections = $state<ConnectionView[]>([]);
	let loadFailed = $state(false);
	/** Store being worked on and what is happening, one at a time. */
	let busy = $state<{ provider: string; what: 'login' | 'sync' | 'other' } | null>(null);
	let notes = $state<Record<string, { text: string; warn: boolean }>>({});
	let confirming = $state<Provider | null>(null);
	let confirmOpen = $state(false);

	// Inline separator kept as data: the formatter trims the space before a literal «·».
	const sep = ' · ';

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
						? tr.connections.cancelled
						: err.code in tr.errors
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
			if (r.account_changed) notes[p] = { warn: true, text: tr.connections.accountChanged };
		});

	const sync = (p: Provider) =>
		act(p, 'sync', async () => {
			const r = await ipc.sync(p);
			notes[p] = r.account_changed
				? { warn: true, text: tr.connections.accountChanged }
				: { warn: false, text: tr.connections.syncDone };
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

<PageHeader title={tr.nav.conexiones}>
	{#snippet info()}<InfoTip label={tr.nav.conexiones} lines={[tr.connections.intro]} />{/snippet}
</PageHeader>

{#if loadFailed}
	<p role="alert" class="intro text-danger">{tr.library.loadError}</p>
{/if}

<div class="fichas">
	{#each connections as c (c.provider)}
		{@const p = c.provider as Provider}
		{@const isConnected = c.state === 'connected'}
		{@const working = busy?.provider === p}
		<section aria-labelledby="conn-{p}" class="ficha {p}">
			<h2 id="conn-{p}" class="tab store-cut">
				<StoreLogo store={p} size={20} />{storeName(p)}
			</h2>
			<p class="acct">
				<span class={isConnected ? 'text-success' : 'text-muted'}
					>{isConnected ? tr.connections.connected : tr.connections.disconnected}</span
				>
				{#if c.account_id}
					{sep}{c.display_name
						? tr.connections.account(c.display_name)
						: tr.connections.accountId(c.account_id)}
				{/if}
				{#if c.other_account_data}<InfoTip
						label={storeName(p)}
						lines={[tr.connections.otherAccountData]}
					/>{/if}
			</p>

			<ul class="caps" aria-label={tr.connections.capabilitiesLabel(storeName(p))}>
				{#each c.capabilities as cap (cap.capability)}
					{@const l = capabilityLine(cap)}
					<li class="cap {l.tone}">
						<span class="cl"
							>{tr.capabilities[cap.capability as keyof typeof tr.capabilities] ??
								cap.capability}</span
						>
						{#if l.count !== null}
							<span class="cn"
								><b class="figure">{l.count.toLocaleString(intl)}</b> <small>{l.unit}</small></span
							>
						{/if}
						<span class="cs"
							><i class="dot" aria-hidden="true"></i>{l.state}{#if l.date}{sep}{l.date}{/if}</span
						>
					</li>
				{/each}
			</ul>

			<div class="acts">
				{#if !isConnected || needsReauth(c)}
					<Button tone="primary" disabled={busy !== null} onclick={() => connect(p)}>
						{isConnected || c.account_id ? tr.connections.reconnect : tr.connections.connect}
					</Button>
				{/if}
				{#if isConnected}
					<Button
						tone={needsReauth(c) ? 'quiet' : 'primary'}
						disabled={busy !== null}
						onclick={() => sync(p)}>{tr.connections.sync}</Button
					>
					<Button tone="ghost" disabled={busy !== null} onclick={() => askDisconnect(p)}>
						{tr.connections.disconnect}
					</Button>
				{/if}
			</div>

			<p class="small" role="status">
				{#if working}
					<span class="muted">
						{busy?.what === 'login' ? tr.connections.waitingLogin : tr.connections.working}
					</span>
				{:else if notes[p]}
					<span class={notes[p].warn ? 'text-warning' : 'muted'}>{notes[p].text}</span>
					{#if !notes[p].warn}
						<a class="link" href={resolve('/limpieza')}>{tr.connections.nextStep}</a>
					{/if}
				{/if}
			</p>
		</section>
	{/each}
</div>

<Dialog
	bind:open={confirmOpen}
	title={tr.connections.disconnectTitle(storeName(confirming ?? ''))}
	description={tr.connections.disconnectBody}
	closeLabel={tr.connections.cancel}
>
	{#snippet footer()}
		<Button onclick={() => (confirmOpen = false)}>{tr.connections.cancel}</Button>
		<Button tone="danger" onclick={disconnect}>{tr.connections.disconnect}</Button>
	{/snippet}
</Dialog>

<style>
	.intro {
		max-width: 70ch;
		margin: 0;
		padding: 14px 32px 0;
		color: var(--color-muted);
	}
	.fichas {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
		gap: 24px;
		padding: 20px 32px 40px;
	}
	.ficha {
		display: grid;
		align-content: start;
		gap: 10px;
		padding: 0 18px 16px;
		border: 2px solid var(--color-border-strong);
	}
	.tab {
		display: flex;
		align-items: center;
		gap: 8px;
		width: fit-content;
		margin: 0 0 4px -18px;
		padding: 8px 16px 8px 14px;
		font-size: 15px;
		font-weight: 800;
		text-transform: uppercase;
		background: var(--color-steam);
		color: var(--color-on-store);
	}
	.gog .tab {
		background: var(--color-gog);
	}
	.epic .tab {
		background: var(--color-epic);
		color: var(--color-on-epic);
	}
	.acct {
		margin: 0;
		font-weight: 600;
	}
	.muted {
		color: var(--color-muted);
	}
	.small {
		margin: 0;
		font-size: 13px;
	}
	.caps {
		margin: 4px 0 0;
		padding: 0;
		list-style: none;
		border-top: 2px solid var(--color-border-strong);
	}
	.cap {
		display: grid;
		grid-template-columns: 1fr auto;
		grid-template-areas: 'label count' 'state count';
		align-items: center;
		gap: 2px 12px;
		padding: 10px 0;
		border-bottom: 1px solid var(--color-border);
	}
	.cl {
		grid-area: label;
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.cn {
		grid-area: count;
		display: grid;
		justify-items: end;
		line-height: 1;
	}
	.cn b {
		font-size: 32px;
	}
	.cn small {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--color-muted);
	}
	.cs {
		grid-area: state;
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		color: var(--color-muted);
	}
	.dot {
		flex: none;
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-z5);
	}
	.ok .dot {
		background: var(--color-success);
	}
	.warn .dot {
		background: var(--color-warning);
	}
	.warn .cs {
		color: var(--color-warning);
		font-weight: 600;
	}
	.acts {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		margin-top: 4px;
	}
	.link {
		margin-left: 4px;
		color: var(--color-text);
		text-decoration: underline;
		text-underline-offset: 2px;
	}
</style>
