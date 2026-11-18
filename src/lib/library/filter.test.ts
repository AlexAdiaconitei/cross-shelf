import { describe, expect, it } from 'vitest';
import {
	applyFilters,
	counters,
	isFiltering,
	kindsPresent,
	noFilters,
	normalize,
	viewState,
	visibleRange,
	activeTab,
	tabPreset,
	tabs,
	type Entry
} from './filter';

function entry(over: Partial<Entry> & { classes?: string[] }): Entry {
	const { classes = ['owned'], ...rest } = over;
	return {
		store: 'steam',
		account_id: 'a',
		native_id: String(Math.random()),
		title: 'Título',
		kind: 'game',
		windows: 'yes',
		url: null,
		image_url: null,
		access: classes.map((class_) => ({
			class: class_,
			owners: [],
			exclude_reason: null,
			observed_at: 1
		})),
		...rest
	};
}

const list: Entry[] = [
	entry({ title: 'Pokémon™ Épico', store: 'steam' }),
	entry({ title: 'Fire & Ice', store: 'gog', kind: 'game' }),
	entry({ title: 'Banda sonora', store: 'steam', kind: 'soundtrack', classes: ['family_shared'] }),
	entry({ title: 'Compartido y propio', store: 'steam', classes: ['owned', 'family_shared'] }),
	entry({ title: null, store: 'gog', kind: 'unknown' })
];

describe('normalize', () => {
	it('ignores case, accents and trademark marks', () => {
		expect(normalize('Pokémon™ ÉPICO')).toBe('pokemon epico');
	});
	it('collapses whitespace', () => {
		expect(normalize('  a   b ')).toBe('a b');
	});
});

describe('applyFilters', () => {
	it('searches titles without caring about accents or case', () => {
		expect(applyFilters(list, { ...noFilters, text: 'pokemon epi' })).toHaveLength(1);
		expect(applyFilters(list, { ...noFilters, text: 'FIRE' })).toHaveLength(1);
	});

	it('combines text and filters predictably (all must hold)', () => {
		const f = { ...noFilters, text: 'o', store: 'steam', scope: 'own' as const };
		const titles = applyFilters(list, f).map((e) => e.title);
		expect(titles).toEqual(['Pokémon™ Épico', 'Compartido y propio']);
	});

	it('separates own, family and family-only access', () => {
		expect(applyFilters(list, { ...noFilters, scope: 'family' })).toHaveLength(2);
		expect(applyFilters(list, { ...noFilters, scope: 'own' })).toHaveLength(4);
		const only = applyFilters(list, { ...noFilters, scope: 'family_only' });
		expect(only.map((e) => e.title)).toEqual(['Banda sonora']);
	});

	it('keeps products without a title findable by kind, not by text', () => {
		expect(applyFilters(list, { ...noFilters, kind: 'unknown' })).toHaveLength(1);
		expect(applyFilters(list, { ...noFilters, text: 'x' }).some((e) => e.title === null)).toBe(
			false
		);
	});
});

describe('counters and states', () => {
	it('counts per store and family access over the unfiltered list', () => {
		const shown = applyFilters(list, { ...noFilters, store: 'gog' });
		const c = counters(list, shown);
		expect(c.byStore).toEqual({ steam: 3, gog: 2 });
		expect(c.familyAccess).toBe(2);
		expect([c.total, c.shown]).toEqual([5, 2]);
	});

	it('tells apart nothing synced, no match for the filters, and a list', () => {
		expect(viewState([], [])).toBe('no_data');
		expect(viewState(list, [])).toBe('no_match');
		expect(viewState(list, list)).toBe('list');
	});

	it('shows a failed first sync as a failure, not as an empty library', () => {
		const failed = {
			store: 'steam',
			account_id: 'a',
			resource: 'library',
			last_state: 'failed',
			last_at: 2,
			last_complete_at: null
		};
		expect(viewState([], [], [failed])).toBe('sync_failed');
		expect(viewState([], [], [{ ...failed, last_complete_at: 1 }])).toBe('no_data');
	});
});

describe('tabs', () => {
	it('every tab is a preset that is recognised back', () => {
		for (const t of tabs) expect(activeTab({ ...noFilters, ...tabPreset(t) })).toBe(t);
	});
	it('the family tab shows only Steam entries with family access', () => {
		const out = applyFilters(list, { ...noFilters, ...tabPreset('steam_family') });
		expect(
			out.every((e) => e.store === 'steam' && e.access.some((a) => a.class === 'family_shared'))
		).toBe(true);
		expect(out).toHaveLength(2);
	});

	it('knows when filters are active, and lists the kinds including unknown', () => {
		expect(isFiltering(noFilters)).toBe(false);
		expect(isFiltering({ ...noFilters, text: ' ' })).toBe(false);
		expect(isFiltering({ ...noFilters, scope: 'family' })).toBe(true);
		expect(kindsPresent(list)).toEqual(['game', 'soundtrack', 'unknown']);
	});
});

describe('visibleRange', () => {
	it('renders only the rows in view plus a margin', () => {
		const r = visibleRange(1000, 500, 50, 10_000, 4);
		expect(r).toEqual({ start: 16, end: 34 });
	});
	it('never leaves the list bounds', () => {
		expect(visibleRange(0, 500, 50, 3)).toEqual({ start: 0, end: 3 });
		expect(visibleRange(900_000, 500, 50, 100).end).toBe(100);
		expect(visibleRange(0, 500, 50, 0)).toEqual({ start: 0, end: 0 });
	});
});
