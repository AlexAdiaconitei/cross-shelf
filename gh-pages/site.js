import { latestInstaller, latestRelease, releaseLinks, repository } from './release.js';

let dictionary;
let release;
let currentView = 'library';
const languageKey = 'crossshelf.landing.language';
const languageButtons = document.querySelectorAll('[data-language]');
let languageRequest = 0;

function translate() {
	for (const node of document.querySelectorAll('[data-i18n]')) {
		const value = dictionary[node.dataset.i18n];
		if (typeof value === 'string') node.textContent = value;
	}
	for (const node of document.querySelectorAll('[data-i18n-alt]')) {
		node.alt = dictionary[node.dataset.i18nAlt];
	}
	for (const node of document.querySelectorAll('[data-i18n-label]')) {
		node.setAttribute('aria-label', dictionary[node.dataset.i18nLabel]);
	}
	for (const image of document.querySelectorAll('[data-language-image]')) {
		image.src = `./assets/screenshots/${image.dataset.languageImage}-${document.documentElement.lang}.webp`;
	}
	document.title = dictionary.pageTitle;
	document.querySelector('meta[name="description"]').content = dictionary.description;
	document.querySelector('[data-release-status]').textContent = release
		? `${release.version} · ${dictionary.windows}`
		: dictionary.releaseFallback;
	for (const node of document.querySelectorAll('[data-download-label]')) {
		node.textContent = dictionary.download;
	}
	showView(currentView);
}

async function setLanguage(language) {
	const request = ++languageRequest;
	try {
		const response = await fetch(`./i18n/${language}.json`);
		if (!response.ok) throw new Error('Dictionary unavailable');
		const next = await response.json();
		if (request !== languageRequest) return;
		dictionary = next;
		document.documentElement.lang = language;
		translate();
		languageButtons.forEach((button) =>
			button.setAttribute('aria-pressed', String(button.dataset.language === language))
		);
		try {
			localStorage.setItem(languageKey, language);
		} catch {
			/* The switch still works when browser storage is unavailable. */
		}
	} catch {
		/* The complete English page remains readable without a dictionary request. */
	}
}

function showView(view) {
	currentView = view;
	for (const button of document.querySelectorAll('[data-view]')) {
		button.setAttribute('aria-pressed', String(button.dataset.view === view));
	}
	const screenshot = document.querySelector('[data-showcase]');
	const language = document.documentElement.lang === 'es' ? 'es' : 'en';
	screenshot.src = `./assets/screenshots/${view}-${language}.webp`;
	if (dictionary) {
		screenshot.alt = dictionary[`${view}Alt`];
		document.querySelector('[data-showcase-caption]').textContent = dictionary[`${view}Caption`];
	}
}

languageButtons.forEach((button) => {
	button.hidden = false;
	button.addEventListener('click', () => setLanguage(button.dataset.language));
});
document.querySelectorAll('[data-view]').forEach((button) => {
	button.hidden = false;
	button.addEventListener('click', () => showView(button.dataset.view));
});

let initialLanguage = 'en';
try {
	if (localStorage.getItem(languageKey) === 'es') initialLanguage = 'es';
} catch {
	/* English is the default, including on a Spanish operating system. */
}
await setLanguage(initialLanguage);

try {
	const response = await fetch(`https://api.github.com/repos/${repository}/releases/latest`, {
		headers: { Accept: 'application/vnd.github+json' },
		signal: AbortSignal.timeout(6000)
	});
	if (response.ok) release = releaseLinks(await response.json());
} catch {
	/* No release yet, offline or rate limited: the stable installer link still downloads. */
}
for (const link of document.querySelectorAll('[data-download]')) {
	link.href = release?.download || latestInstaller;
}
document.querySelector('[data-release-notes]').href = release?.notes || latestRelease;
if (dictionary) translate();
