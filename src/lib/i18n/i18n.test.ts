import { describe, expect, it } from 'vitest';
import { en } from './en.ts';
import { es } from './es.ts';
import { lang, tr } from './index.ts';

/** Every key path of a dictionary, functions included, to compare both languages. */
function keys(o: object, prefix = ''): string[] {
	return Object.entries(o).flatMap(([k, v]) =>
		v && typeof v === 'object' ? keys(v, `${prefix}${k}.`) : [`${prefix}${k}`]
	);
}

describe('dictionaries', () => {
	it('English and Spanish have exactly the same keys', () => {
		expect(keys(en).sort()).toEqual(keys(es).sort());
	});

	it('functions take the same parameters in both languages', () => {
		const fns = (o: object, p = ''): [string, number][] =>
			Object.entries(o).flatMap(([k, v]) =>
				typeof v === 'function'
					? [[`${p}${k}`, v.length] as [string, number]]
					: v && typeof v === 'object'
						? fns(v, `${p}${k}.`)
						: []
			);
		expect(fns(en)).toEqual(fns(es));
	});

	it('no text is left empty', () => {
		const empty = (o: object): boolean =>
			Object.values(o).some((v) =>
				typeof v === 'string' ? v.trim() === '' : v && typeof v === 'object' ? empty(v) : false
			);
		expect(empty(en)).toBe(false);
		expect(empty(es)).toBe(false);
	});

	it('English is the default language', () => {
		expect(lang).toBe('en');
		expect(tr).toBe(en);
	});
});
