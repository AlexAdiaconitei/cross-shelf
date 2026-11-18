#!/usr/bin/env node
// Resolves a release from a tag and refuses it unless every manifest in the repository carries
// the same version and CHANGELOG.md has a section for it.
//
//   node <devkit>/actions/scripts/verify-version.mjs v1.2.0   (from the project root)
//
// Checked when present: package.json, src-tauri/tauri.conf.json, src-tauri/Cargo.toml,
// Cargo.toml ([package] or [workspace.package]), pyproject.toml, VERSION, Directory.Build.props.
// Extra files: VERIFY_VERSION_FILES="apps/web/package.json,crates/agent/Cargo.toml".
//
// In GitHub Actions it writes version, tag, prerelease and notes_file to $GITHUB_OUTPUT.

import { appendFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ref = process.argv[2] ?? process.env.GITHUB_REF_NAME ?? '';
const version = ref.replace(/^refs\/tags\//, '').replace(/^v/, '');

if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version)) {
	console.error(`::error::"${ref}" is not a vMAJOR.MINOR.PATCH[-pre] tag`);
	process.exit(1);
}

const readers = {
	json: (text) => JSON.parse(text).version,
	toml: (text) => {
		// The version of [package] or [workspace.package], whichever the file declares.
		const section = /^\[(workspace\.)?package\]\s*$([\s\S]*?)(?=^\[|$(?![\s\S]))/m.exec(text);
		return section && /^version\s*=\s*"([^"]+)"/m.exec(section[2])?.[1];
	},
	pyproject: (text) => /^\[project\][\s\S]*?^version\s*=\s*"([^"]+)"/m.exec(text)?.[1],
	plain: (text) => text.trim(),
	msbuild: (text) => /<Version>([^<]+)<\/Version>/.exec(text)?.[1]
};

const candidates = [
	['package.json', 'json'],
	['src-tauri/tauri.conf.json', 'json'],
	['src-tauri/Cargo.toml', 'toml'],
	['Cargo.toml', 'toml'],
	['pyproject.toml', 'pyproject'],
	['VERSION', 'plain'],
	['Directory.Build.props', 'msbuild'],
	...(process.env.VERIFY_VERSION_FILES ?? '')
		.split(',')
		.filter(Boolean)
		.map((file) => [
			file.trim(),
			file.endsWith('.json') ? 'json' : file.endsWith('.toml') ? 'toml' : 'plain'
		])
];

const mismatches = [];
let checked = 0;
for (const [file, kind] of candidates) {
	if (!existsSync(file)) continue;
	const found = readers[kind](readFileSync(file, 'utf8'));
	if (found === undefined || found === null) continue; // e.g. a virtual Cargo workspace
	checked++;
	// A prerelease may keep its base version in the manifests (Tauri does not accept "-alpha").
	if (found !== version && found !== version.split('-')[0])
		mismatches.push(`${file} says ${found}`);
}

if (mismatches.length) {
	console.error(`::error::tag v${version} does not match: ${mismatches.join(', ')}`);
	process.exit(1);
}

const here = dirname(fileURLToPath(import.meta.url));
const notes = spawnSync(process.execPath, [join(here, 'changelog-section.mjs'), version], {
	encoding: 'utf8'
});
if (notes.status !== 0) {
	console.error(`::error::${notes.stderr.trim()}`);
	process.exit(1);
}

const notesFile = join(process.env.RUNNER_TEMP ?? '.', 'release-notes.md');
writeFileSync(notesFile, notes.stdout);

const outputs = {
	version,
	tag: `v${version}`,
	prerelease: String(version.includes('-')),
	notes_file: notesFile
};
if (process.env.GITHUB_OUTPUT) {
	appendFileSync(
		process.env.GITHUB_OUTPUT,
		Object.entries(outputs)
			.map(([k, v]) => `${k}=${v}\n`)
			.join('')
	);
}
console.log(`release v${version}: ${checked} manifest(s) agree, notes in ${notesFile}`);
