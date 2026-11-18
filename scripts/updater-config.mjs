#!/usr/bin/env node
// Writes src-tauri/tauri.updater.conf.json for whichever repository is building, so a fork updates
// from its own releases (pattern from Quick Media Organizer).
//
//   node scripts/updater-config.mjs && pnpm tauri build --config src-tauri/tauri.updater.conf.json
//
// Without TAURI_SIGNING_PUBLIC_KEY it writes nothing and exits 0: the build ships without the
// in-app updater instead of failing. Generate the pair once with
// `pnpm tauri signer generate -w ~/.tauri/<app>.key` and store both halves as repository secrets.

import { execFileSync } from 'node:child_process';
import { rmSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const target = join(root, 'src-tauri', 'tauri.updater.conf.json');

const publicKey = process.env.TAURI_SIGNING_PUBLIC_KEY?.trim();
const privateKey = process.env.TAURI_SIGNING_PRIVATE_KEY?.trim();
if (Boolean(publicKey) !== Boolean(privateKey)) {
	throw new Error(
		'TAURI_SIGNING_PUBLIC_KEY and TAURI_SIGNING_PRIVATE_KEY must both be set or both be absent'
	);
}

// A local build must never inherit the configuration of an earlier signed build.
rmSync(target, { force: true });

if (!publicKey) {
	console.log('updater: no signing keys, building without the updater');
	process.exit(0);
}

function repositorySlug() {
	if (process.env.GITHUB_REPOSITORY) return process.env.GITHUB_REPOSITORY;
	const remote = execFileSync('git', ['remote', 'get-url', 'origin'], { encoding: 'utf8' }).trim();
	const match = /github\.com[:/]([^/]+\/[^/]+?)(?:\.git)?$/.exec(remote);
	if (!match) throw new Error(`cannot derive owner/repo from ${remote}`);
	return match[1];
}

const config = {
	bundle: { createUpdaterArtifacts: true },
	plugins: {
		updater: {
			pubkey: publicKey,
			endpoints: [`https://github.com/${repositorySlug()}/releases/latest/download/latest.json`]
		}
	}
};

writeFileSync(target, `${JSON.stringify(config, null, 2)}\n`);
console.log(`updater: ${target}`);
