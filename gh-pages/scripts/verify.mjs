import assert from 'node:assert/strict';
import { mkdir } from 'node:fs/promises';
import { pathToFileURL } from 'node:url';
import { latestInstaller, repository } from '../release.js';

const driver = process.env.CROSSSHELF_CDP_MODULE
	? pathToFileURL(process.env.CROSSSHELF_CDP_MODULE)
	: new URL('../../.agents/skills/design-taste-frontend/scripts/lib/cdp.mjs', import.meta.url);
const { Browser } = await import(driver.href);
const url = process.env.LANDING_URL || 'http://127.0.0.1:4173/cross-shelf/';
const out = new URL('../../.taste-skill/qa/', import.meta.url);
await mkdir(out, { recursive: true });
const browser = await Browser.launch();
const download = `https://github.com/${repository}/releases/download/v9.0.0/CrossShelf_9.0.0_x64-setup.exe`;
try {
	const page = await browser.newPage();
	let status = 200;
	let payload = {
		tag_name: 'v9.0.0',
		assets: [{ name: 'CrossShelf_9.0.0_x64-setup.exe', browser_download_url: download }]
	};
	await page.send('Emulation.setLocaleOverride', { locale: 'es-ES' });
	await page.send('Fetch.enable', { patterns: [{ urlPattern: 'https://api.github.com/*' }] });
	browser.on(page.sessionId, 'Fetch.requestPaused', (event) => {
		page.send('Fetch.fulfillRequest', {
			requestId: event.requestId,
			responseCode: status,
			responseHeaders: [
				{ name: 'Content-Type', value: 'application/json' },
				{ name: 'Access-Control-Allow-Origin', value: '*' }
			],
			body: Buffer.from(JSON.stringify(payload)).toString('base64')
		});
	});
	await page.goto(url);
	const invalidIcons = await page.eval(async () => {
		const results = [];
		for (const name of ['connect', 'library', 'checklist']) {
			const response = await fetch(`./assets/${name}.svg`);
			const source = await response.text();
			const svg = new DOMParser().parseFromString(source, 'image/svg+xml');
			const namespaces = source.match(/<svg\b[^>]*>/)?.[0].match(/\sxmlns\s*=/g)?.length;
			if (
				!response.ok ||
				namespaces !== 1 ||
				svg.querySelector('parsererror') ||
				svg.documentElement.namespaceURI !== 'http://www.w3.org/2000/svg'
			)
				results.push(name);
		}
		return results;
	});
	assert.deepEqual(invalidIcons, [], 'How-it-works icons must be valid standalone SVG images');
	assert.equal(
		await page.eval(() => document.documentElement.lang),
		'en',
		'English default on a Spanish system'
	);
	assert.equal(await page.eval(() => document.querySelector('[data-download]').href), download);
	assert.equal(
		await page.eval(() => document.querySelector('[data-release-notes]').href),
		`https://github.com/${repository}/releases/tag/v9.0.0`
	);
	for (const width of [1440, 768, 390, 360]) {
		await page.setViewport({ width, height: 900 });
		for (const language of ['en', 'es']) {
			await page.eval(
				(lang) => document.querySelector(`[data-language="${lang}"]`).click(),
				language
			);
			for (let attempt = 0; attempt < 30; attempt++) {
				if (
					await page.eval(
						(lang) =>
							document.documentElement.lang === lang &&
							document
								.querySelector('.about-bio')
								.textContent.includes(lang === 'es' ? 'En el trabajo' : 'At work'),
						language
					)
				)
					break;
				await new Promise((resolve) => setTimeout(resolve, 100));
			}
			const state = await page.eval(async () => {
				await document.fonts.ready;
				for (const image of document.images) image.loading = 'eager';
				await Promise.all([...document.images].map((image) => image.decode().catch(() => {})));
				const title = document.querySelector('h1');
				return {
					language: document.documentElement.lang,
					overflow: document.documentElement.scrollWidth > innerWidth,
					broken: [...document.images]
						.filter((image) => !image.naturalWidth)
						.map((image) => image.src),
					lines:
						title.getBoundingClientRect().height / parseFloat(getComputedStyle(title).lineHeight),
					photo: document.querySelector('.about-photo').naturalWidth,
					bio: document.querySelector('.about-bio').textContent,
					disclaimer: document.querySelector('[data-i18n="artworkDisclaimer"]').textContent
				};
			});
			assert.equal(state.language, language);
			assert.equal(state.overflow, false, `${width}/${language}: horizontal overflow`);
			assert.deepEqual(state.broken, [], `${width}/${language}: broken images`);
			assert.ok(state.lines <= 2.1, `${width}/${language}: hero wraps to ${state.lines} lines`);
			assert.ok(state.photo > 0);
			assert.ok(
				state.disclaimer.includes(language === 'es' ? 'no está afiliado' : 'not affiliated')
			);
			await page.screenshot({ path: new URL(`${width}-${language}.jpg`, out), fullPage: true });
			console.log(`PASS ${width}px / ${language}: layout, images, headline, author`);
		}
	}
	for (const view of ['review', 'connections', 'library']) {
		await page.eval((name) => document.querySelector(`[data-view="${name}"]`).click(), view);
		assert.ok(
			await page.eval(
				(name) => document.querySelector('[data-showcase]').src.endsWith(`${name}-es.webp`),
				view
			)
		);
	}
	await page.goto(url);
	assert.equal(
		await page.eval(() => document.documentElement.lang),
		'es',
		'Language choice persists'
	);
	assert.equal(await page.eval(async () => (await fetch('./about.json')).status), 200);
	const summary = await page.eval(() => {
		const details = document.querySelector('details');
		details.open = false;
		details.querySelector('summary').focus();
		return details.open;
	});
	assert.equal(summary, false);
	await page.send('Page.bringToFront');
	await page.send('Input.dispatchKeyEvent', {
		type: 'keyDown',
		key: 'Enter',
		code: 'Enter',
		text: '\r',
		windowsVirtualKeyCode: 13
	});
	await page.send('Input.dispatchKeyEvent', {
		type: 'keyUp',
		key: 'Enter',
		code: 'Enter',
		windowsVirtualKeyCode: 13
	});
	assert.equal(
		await page.eval(() => document.querySelector('details').open),
		true,
		'FAQ works with keyboard'
	);
	for (const code of [404, 403, 500]) {
		status = code;
		await page.goto(url);
		assert.equal(
			await page.eval(() => document.querySelector('[data-download]').href),
			latestInstaller
		);
	}
	status = 200;
	payload = { tag_name: 'v9.0.0', assets: [] };
	await page.goto(url);
	assert.equal(
		await page.eval(() => document.querySelector('[data-download]').href),
		latestInstaller
	);
	assert.deepEqual(page.consoleErrors, []);
	const noScript = await browser.newPage({ width: 360, height: 900 });
	await noScript.send('Emulation.setScriptExecutionDisabled', { value: true });
	await noScript.goto(url);
	const fallback = await noScript.eval(() => ({
		title: document.querySelector('h1').textContent.trim(),
		languageButtons: [...document.querySelectorAll('[data-language]')].every(
			(button) => button.hidden
		),
		link: document.querySelector('[data-download]').href
	}));
	assert.ok(fallback.title.includes('All your games.'));
	assert.equal(fallback.languageButtons, true);
	assert.equal(fallback.link, latestInstaller);
	console.log(
		'PASS release success / 404 / rate limit / server failure / no EXE; language persistence; keyboard FAQ; base path; no-JS fallback'
	);
} finally {
	await browser.close();
}
