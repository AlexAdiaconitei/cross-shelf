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
// Play log: dates relative to the moment of the capture, so «3 days ago» stays true on the image.
const today = Math.floor(Date.now() / 1000);
const day = 86400;
const heldStore = (i) => (i < 5 || i === 7 || i === 10 ? 'steam' : stores[i % 3]);
const hoursOf = (i) =>
	({ 0: 0, 1: 5380, 2: 640, 3: 8200, 4: 1210, 6: 2410, 10: 380, 12: 3900 })[i] ?? null;
const daysAgo = { 1: 4, 2: 45, 3: 90, 4: 2, 6: 1, 10: 6, 12: 20 };
const lastOf = (i) => (daysAgo[i] === undefined ? null : today - day * daysAgo[i]);
const playRows = [
	[6, 'playing', 'Path of Pain left'],
	[1, 'playing', 'Phantom Liberty next'],
	[8, 'backlog', ''],
	[0, 'backlog', ''],
	[2, 'paused', 'Back after the holidays'],
	[12, 'completed', ''],
	[3, 'completed', ''],
	[5, 'dropped', '']
];
const plays = playRows.map(([i, state, note], position) => ({
	product: product(titles[i], i, heldStore(i)),
	state,
	position,
	since: today - day * (10 + position * 7),
	started_at: state === 'backlog' ? null : today - day * (40 + position * 9),
	finished_at: state === 'completed' ? today - day * (20 + position) : null,
	note,
	in_library: true,
	minutes: hoursOf(i),
	last_activity: lastOf(i)
}));
const notesEs = {
	'Path of Pain left': 'Me falta el Camino del Dolor',
	'Phantom Liberty next': 'Luego, Phantom Liberty',
	'Back after the holidays': 'Vuelvo después de las fiestas'
};
const suggestions = [4, 10].map((i) => ({
	product: product(titles[i], i, heldStore(i)),
	target: 'playing',
	reason: 'played_recently',
	current: null,
	last_activity: lastOf(i)
}));
const weekOf = { 6: 380, 1: 240, 4: 150, 10: 90 };
const diary = [...playRows.map(([i]) => i), 4, 10]
	.map((i) => {
		const play = plays.find((p) => p.product.title === titles[i]);
		return {
			product: product(titles[i], i, heldStore(i)),
			minutes: hoursOf(i),
			last_activity: lastOf(i),
			week_minutes: weekOf[i] ?? null,
			state: play?.state ?? null,
			deduced: play ? null : 'playing'
		};
	})
	.sort((a, b) => (b.last_activity ?? 0) - (a.last_activity ?? 0));
const playtime = library.map((e) => {
	const i = titles.indexOf(e.title);
	return {
		store: e.store,
		native_id: e.native_id,
		minutes: e.store === 'steam' ? hoursOf(i) : null,
		last_played: e.store === 'steam' ? lastOf(i) : null
	};
});
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
			settings_get: {
				language,
				include_family_in_cleanup: false,
				show_covers: true,
				plays_tab: 'board'
			},
			plays_list: plays.map((p) => ({
				...p,
				note: language === 'es' ? (notesEs[p.note] ?? p.note) : p.note
			})),
			suggestions_list: suggestions,
			diary_list: diary,
			playtime_of: playtime,
			playtime_stores: ['steam'],
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
			window.__FIXTURES__ = ${JSON.stringify(fixtures)};
			window.__TAURI_INTERNALS__ = { invoke: async (command, args) => {
				const data = window.__FIXTURES__;
				if (!(command in data)) throw new Error('Screenshot fixture does not support: ' + command);
				// Like the backend: one answer per product asked, nothing else.
				if (command === 'playtime_of')
					return args.products.map((p) =>
						data.playtime_of.find((t) => t.store === p.store && t.native_id === p.native_id) ??
						{ ...p, minutes: null, last_played: null }
					);
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
					if (path === '/partidas') return !!document.querySelector('.board .card');
					return document.querySelector('main')?.textContent.includes('Hades');
				}, route);
				if (ready) break;
				await new Promise((resolve) => setTimeout(resolve, 200));
			}
			if (!ready)
				throw new Error(
					`App did not finish rendering ${route}: ${page.consoleErrors.join('\n')} ${await page.eval(() => document.body.innerText.slice(0, 300))}`
				);
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
			if (name === 'detail')
				await page.eval(() => document.querySelectorAll('.shelf-row')[1].click());
			if (name === 'diary') {
				// The third tab of Play log; its label is translated, its position is not.
				await page.eval(() => document.querySelectorAll('.toolbar .segmented .control')[2].click());
				await new Promise((resolve) => setTimeout(resolve, 400));
			}
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
		await shot('plays', '/partidas');
		await shot('diary', '/partidas');
	}
} finally {
	await browser.close();
}
