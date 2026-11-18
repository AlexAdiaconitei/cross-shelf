import { readFile, writeFile, mkdir } from 'node:fs/promises';
import { en } from '../../src/lib/i18n/en.ts';
import { es } from '../../src/lib/i18n/es.ts';

const root = new URL('../', import.meta.url);
const escape = (s) =>
	s
		.replaceAll('&', '&amp;')
		.replaceAll('<', '&lt;')
		.replaceAll('>', '&gt;')
		.replaceAll('"', '&quot;');
await mkdir(new URL('i18n/', root), { recursive: true });
for (const [lang, dictionary] of Object.entries({ en: en.landing, es: es.landing })) {
	await writeFile(
		new URL(`i18n/${lang}.json`, root),
		JSON.stringify(dictionary, null, '\t') + '\n'
	);
}
const profile = JSON.parse(await readFile(new URL('about.json', root), 'utf8'));
const values = {
	...en.landing,
	authorName: profile.name,
	authorBio: profile.bio.en,
	...Object.fromEntries(profile.facts.en.map((fact, i) => [`authorFact${i}`, fact])),
	authorGithub: profile.links.find((link) => link.icon === 'github').url,
	authorLinkedin: profile.links.find((link) => link.icon === 'linkedin').url,
	authorKofi: profile.links.find((link) => link.icon === 'kofi').url
};
const template = await readFile(new URL('scripts/index.template.html', root), 'utf8');
const html = template.replace(/\{\{(\w+)\}\}/g, (_, key) => {
	if (typeof values[key] !== 'string') throw new Error(`Unknown template value: ${key}`);
	return escape(values[key]);
});
await writeFile(new URL('index.html', root), html);
console.log('Landing dictionaries and English HTML updated from src/lib/i18n.');
