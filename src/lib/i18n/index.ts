// Active language. Chosen once when the app starts, before anything is drawn: the setting lives in
// the database (Settings.language) and a copy in localStorage makes it available synchronously.
// Changing it saves both and reloads the window, so every view is drawn in the new language.
import { en } from './en.ts';
import { es, type Strings } from './es.ts';

export type Lang = 'en' | 'es';
export const languages: { value: Lang; label: string }[] = [
	{ value: 'en', label: 'English' },
	{ value: 'es', label: 'Español' }
];

const KEY = 'crossshelf.lang';

function stored(): Lang {
	try {
		const v = globalThis.localStorage?.getItem(KEY);
		if (v === 'en' || v === 'es') return v;
	} catch {
		/* private storage: the default */
	}
	return 'en';
}

export const lang: Lang = stored();
export const tr: Strings = lang === 'es' ? es : en;
/** BCP 47 tag for dates, numbers and sorting. */
export const intl = lang === 'es' ? 'es-ES' : 'en-GB';

export const isLang = (v: string): v is Lang => v === 'en' || v === 'es';

/** Keeps the choice for the next start. Returns true when the window must reload to apply it. */
export function rememberLang(next: Lang): boolean {
	try {
		globalThis.localStorage?.setItem(KEY, next);
	} catch {
		/* the database still holds it */
	}
	return next !== lang;
}
