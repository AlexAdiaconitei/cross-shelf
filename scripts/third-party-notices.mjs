// Writes THIRD_PARTY_NOTICES.md: every third-party package that ships inside the Windows build,
// with its version, licence and source. Rust crates come from `cargo metadata` (normal
// dependencies reachable from the app, Windows x64 only); front-end packages from
// `pnpm licenses list --prod`. Run it before each release: `node scripts/third-party-notices.mjs`.
import { execSync } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
// Fixed command lines, no user input: a shell is needed so Windows finds pnpm.cmd.
const run = (cmd, args, cwd) =>
	execSync([cmd, ...args].join(' '), { cwd, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });

function rustCrates() {
	const meta = JSON.parse(
		run(
			'cargo',
			[
				'metadata',
				'--format-version',
				'1',
				'--locked',
				'--filter-platform',
				'x86_64-pc-windows-msvc'
			],
			`${root}/src-tauri`
		)
	);
	const byId = new Map(meta.packages.map((p) => [p.id, p]));
	const nodes = new Map(meta.resolve.nodes.map((n) => [n.id, n]));
	const rootId = meta.resolve.root;
	const seen = new Set();
	const stack = [rootId];
	while (stack.length) {
		const id = stack.pop();
		if (seen.has(id)) continue;
		seen.add(id);
		for (const d of nodes.get(id)?.deps ?? []) {
			// Only what is linked into the app: no dev- or build-only dependencies.
			if (d.dep_kinds.some((k) => k.kind === null)) stack.push(d.pkg);
		}
	}
	seen.delete(rootId);
	return [...seen]
		.map((id) => byId.get(id))
		.map((p) => ({
			name: p.name,
			version: p.version,
			license: p.license ?? (p.license_file ? `ver ${p.license_file}` : 'sin declarar'),
			source: p.repository ?? `https://crates.io/crates/${p.name}`
		}));
}

function npmPackages() {
	const byLicense = JSON.parse(run('pnpm', ['licenses', 'list', '--prod', '--json'], root));
	return Object.entries(byLicense).flatMap(([license, pkgs]) =>
		pkgs.flatMap((p) =>
			p.versions.map((version) => ({
				name: p.name,
				version,
				license,
				source: p.homepage ?? `https://www.npmjs.com/package/${p.name}`
			}))
		)
	);
}

const sortRows = (rows) =>
	rows.sort((a, b) => a.name.localeCompare(b.name) || a.version.localeCompare(b.version));

function table(rows) {
	const esc = (s) => String(s).replaceAll('|', '\\|');
	return [
		'| Paquete | Versión | Licencia | Origen |',
		'| --- | --- | --- | --- |',
		...rows.map((r) => `| ${esc(r.name)} | ${r.version} | ${esc(r.license)} | ${esc(r.source)} |`)
	].join('\n');
}

const rust = sortRows(rustCrates());
const npm = sortRows(npmPackages());
const out = `# Avisos de terceros

CrossShelf incluye el software de terceros que se lista aquí, cada uno bajo su propia licencia.
Generado con \`node scripts/third-party-notices.mjs\`; no se edita a mano.

## Motor (Rust): ${rust.length} paquetes

${table(rust)}

## Interfaz (JavaScript): ${npm.length} paquetes

${table(npm)}
`;
writeFileSync(`${root}/THIRD_PARTY_NOTICES.md`, out);
console.log(`THIRD_PARTY_NOTICES.md: ${rust.length} crates, ${npm.length} npm packages`);
