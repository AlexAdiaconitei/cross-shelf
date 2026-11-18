import { mkdir, readFile, writeFile } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { en } from '../../src/lib/i18n/en.ts';
import { es } from '../../src/lib/i18n/es.ts';

// Uses the browser driver bundled with the locally installed design-taste-frontend skill.
const driver = process.env.CROSSSHELF_CDP_MODULE
	? pathToFileURL(process.env.CROSSSHELF_CDP_MODULE)
	: new URL('../../.agents/skills/design-taste-frontend/scripts/lib/cdp.mjs', import.meta.url);
const { Browser } = await import(driver.href);
const root = new URL('../', import.meta.url);
const origin = process.env.APP_URL || 'http://127.0.0.1:1420';
await mkdir(new URL('assets/screenshots/', root), { recursive: true });

const source = await readFile(
	new URL('../../src/lib/components/store/StoreLogo.svelte', import.meta.url),
	'utf8'
);
const logos = [...source.matchAll(/<svg\b[\s\S]*?<\/svg>/g)];
for (const [i, store] of ['steam', 'gog', 'epic'].entries()) {
	const svg = logos[i][0]
		.replace(/\{size\}/g, '"24"')
		.replace('id={maskId}', 'id="steam"')
		.replace('url(#{maskId})', 'url(#steam)')
		.replace('<svg', '<svg xmlns="http://www.w3.org/2000/svg"');
	await writeFile(new URL(`assets/${store}.svg`, root), svg);
}

const games = JSON.parse(await readFile(new URL('scripts/games.json', root), 'utf8'));
const titles = games.map((game) => game.title);
const coverCache = new URL('../../.taste-skill/covers/', import.meta.url);
await mkdir(coverCache, { recursive: true });
const covers = [];
for (const game of games) {
	const cacheFile = new URL(`${game.steamAppId}.jpg`, coverCache);
	let bytes;
	try {
		bytes = await readFile(cacheFile);
	} catch {
		const response = await fetch(game.artwork, { signal: AbortSignal.timeout(20000) });
		if (!response.ok || !response.headers.get('content-type')?.startsWith('image/')) {
			throw new Error(`Cover unavailable: ${game.title} (${response.status})`);
		}
		bytes = Buffer.from(await response.arrayBuffer());
		await writeFile(cacheFile, bytes);
	}
	covers.push(`data:image/jpeg;base64,${bytes.toString('base64')}`);
}
const now = 1795003200;
const stores = ['steam', 'gog', 'epic'];
function product(title, i, store) {
	return {
		store,
		native_id: `example-${i}`,
		title,
		kind: 'game',
		windows: 'yes',
		url: null,
		image_url: covers[i]
	};
}
const library = titles.flatMap((title, i) => {
	const held = i < 5 ? ['steam', 'gog'] : i === 7 || i === 10 ? ['steam'] : [stores[i % 3]];
	return held.map((store) => ({
		...product(title, i, store),
		account_id: `example-${store}`,
		access: [
			{
				class: i === 7 || i === 10 ? 'family_shared' : 'owned',
				owners: i === 7 || i === 10 ? ['example-family'] : [],
				owner_names: i === 7 || i === 10 ? ['Player Two'] : [],
				exclude_reason: null,
				observed_at: now
			}
		]
	}));
});
const review = [5, 8, 11, 14].map((i) => {
	const held = library.find((entry) => entry.title === titles[i]);
	return {
		account_id: 'example-steam',
		wanted: product(titles[i], i, 'steam'),
		priority: i,
		added_at: now - 86400 * i,
		observed_at: now,
		kept: false,
		keep_reason: null,
		repeated_in: [],
		matches: [
			{
				held,
				access: 'owned',
				family_owners: 0,
				outcome: 'candidate',
				edition_superior: false,
				platform_unknown: false,
				reason: null,
				evidence: 'title_candidate',
				manual: null
			}
		]
	};
});
const wishlist = review.map((item) => ({
	...item.wanted,
	account_id: item.account_id,
	priority: item.priority,
	added_at: item.added_at,
	confirmed: true,
	already_held_here: false
}));
const connections = stores.map((provider) => ({
	provider,
	state: 'connected',
	account_id: `example-${provider}`,
	display_name: 'Example Player',
	last_sync_at: now,
	other_account_data: false,
	capabilities: ['library', 'family', 'wishlist', 'wishlist_write'].map((capability) => ({
		capability,
		state:
			(capability === 'family' && provider !== 'steam') ||
			(provider === 'epic' && capability.startsWith('wishlist'))
				? 'unavailable'
				: capability === 'wishlist_write'
					? 'validated'
					: 'complete',
		error_code: null,
		last_attempt_at: now,
		last_complete_at: now,
		item_count:
			capability === 'library'
				? library.filter((e) => e.store === provider).length
				: capability === 'family'
					? 2
					: wishlist.filter((w) => w.store === provider).length
	}))
}));
const browser = await Browser.launch();
try {
	for (const language of ['en', 'es']) {
		const page = await browser.newPage({ width: 1440, height: 900 });
		await page.send('Emulation.setEmulatedMedia', {
			features: [{ name: 'prefers-color-scheme', value: 'light' }]
		});
		const fixtures = {
			library_list: library,
			wishlist_list: wishlist,
			review_list: review,
			connections_list: connections,
			plans_list: [],
			settings_get: { language, include_family_in_cleanup: false, show_covers: true },
			sync_status_list: stores.map((store) => ({
				store,
				account_id: `example-${store}`,
				resource: 'library',
				last_state: 'complete',
				last_at: now,
				last_complete_at: now
			}))
		};
		await page.send('Page.addScriptToEvaluateOnNewDocument', {
			source: `
			window.__SCREENSHOT_BANNER__ = ${JSON.stringify({ en: en.landing.exampleBanner, es: es.landing.exampleBanner })};
			localStorage.setItem('crossshelf.lang', ${JSON.stringify(language)});
			localStorage.setItem('crossshelf.sidebar', 'open');
			window.__TAURI_INTERNALS__ = { invoke: async (command) => {
				const data = ${JSON.stringify(fixtures)};
				if (!(command in data)) throw new Error('Screenshot fixture does not support: ' + command);
				return structuredClone(data[command]);
			} };
		`
		});
		async function shot(name, route) {
			await page.goto(origin + route, { settleMs: 1400 });
			let ready = false;
			for (let attempt = 0; attempt < 150; attempt++) {
				ready = await page.eval((path) => {
					if (document.getElementById('splash')) return false;
					if (path === '/') return !!document.querySelector('.shelf-row');
					if (path === '/conexiones') return document.querySelectorAll('.ficha').length === 3;
					return document.querySelector('main')?.textContent.includes('Hades');
				}, route);
				if (ready) break;
				await new Promise((resolve) => setTimeout(resolve, 200));
			}
			if (!ready)
				throw new Error(`App did not finish rendering ${route}: ${page.consoleErrors.join('\n')}`);
			await page.eval(async () => {
				await document.fonts.ready;
				const banner = document.createElement('div');
				banner.textContent =
					document.documentElement.lang === 'es'
						? window.__SCREENSHOT_BANNER__.es
						: window.__SCREENSHOT_BANNER__.en;
				banner.style.cssText =
					'position:fixed;bottom:0;left:0;right:0;background:#100f0d;color:white;padding:6px 16px;font:12px Arial;text-align:center;z-index:9999;letter-spacing:1px';
				document.body.append(banner);
			});
			if (name === 'detail') await page.eval(() => document.querySelector('.shelf-row').click());
			await page.eval(async () => {
				await Promise.all([...document.images].map((image) => image.decode()));
				await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
			});
			if (page.consoleErrors.length) throw new Error(page.consoleErrors.join('\n'));
			await page.screenshot({
				path: new URL(`assets/screenshots/${name}-${language}.webp`, root),
				format: 'webp',
				quality: 88
			});
			console.log(`Captured ${name}-${language}.webp`);
		}
		await shot('library', '/');
		if (language === 'en') {
			for (const [name, route] of Object.entries({
				connect: '/conexiones',
				library: '/',
				checklist: '/limpieza'
			})) {
				const svg = await page.eval(
					(href) =>
						new XMLSerializer().serializeToString(
							document.querySelector(`.side a[href="${href}"] svg`)
						),
					route
				);
				await writeFile(new URL(`assets/${name}.svg`, root), svg);
			}
		}
		await shot('detail', '/');
		await shot('review', '/limpieza');
		await shot('connections', '/conexiones');
	}
} finally {
	await browser.close();
}
