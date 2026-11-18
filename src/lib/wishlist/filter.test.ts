import { describe, expect, it } from 'vitest';
import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
import { applyWishFilters, isWishFiltering, noWishFilters, wishView } from './filter.ts';

let n = 0;
function wish(over: Partial<WishlistEntry>): WishlistEntry {
	n += 1;
	return {
		store: 'steam',
		account_id: 'a',
		native_id: String(n),
		title: 'Título',
		url: null,
		image_url: null,
		priority: null,
		added_at: null,
		confirmed: true,
		already_held_here: false,
		...over
	};
}

function status(over: Partial<SyncStatus>): SyncStatus {
	return {
		store: 'steam',
		account_id: 'a',
		resource: 'wishlist',
		last_state: 'complete',
		last_at: 10,
		last_complete_at: 10,
		...over
	};
}

const list = [
	wish({ title: 'Éxodo™', priority: 2, added_at: 100 }),
	wish({ title: 'Alfa', priority: 0, added_at: 300 }),
	wish({ title: null, store: 'gog' }),
	wish({ title: 'Beta', store: 'gog', confirmed: false, already_held_here: true })
];

describe('applyWishFilters', () => {
	it('searches ignoring case, accents and trademarks', () => {
		const out = applyWishFilters(list, { ...noWishFilters, text: 'EXODO' });
		expect(out.map((w) => w.title)).toEqual(['Éxodo™']);
	});

	it('combines store and status filters predictably', () => {
		const f = { ...noWishFilters, store: 'gog', status: 'unconfirmed' as const };
		expect(applyWishFilters(list, f).map((w) => w.title)).toEqual(['Beta']);
		expect(applyWishFilters(list, { ...f, status: 'untitled' })).toHaveLength(1);
		expect(applyWishFilters(list, { ...f, status: 'held_here' })).toHaveLength(1);
	});

	it('orders by priority with missing priorities last, never invented', () => {
		const ids = applyWishFilters(list, noWishFilters).map((w) => w.title);
		expect(ids.slice(0, 2)).toEqual(['Alfa', 'Éxodo™']);
		expect(ids.at(-1)).toBe(null);
	});

	it('orders by date added, newest first, missing dates last', () => {
		const out = applyWishFilters(list, { ...noWishFilters, sort: 'added' });
		expect(out.map((w) => w.title).slice(0, 2)).toEqual(['Alfa', 'Éxodo™']);
	});

	it('knows when filters are active', () => {
		expect(isWishFiltering(noWishFilters)).toBe(false);
		expect(isWishFiltering({ ...noWishFilters, sort: 'title' })).toBe(false);
		expect(isWishFiltering({ ...noWishFilters, status: 'untitled' })).toBe(true);
	});
});

describe('wishView', () => {
	it('tells a filtered-out list from a list', () => {
		expect(wishView(list, [], [status({})])).toBe('no_match');
		expect(wishView(list, list, [status({})])).toBe('list');
	});

	it('only says empty when every store confirmed it', () => {
		expect(wishView([], [], [status({})])).toBe('confirmed_empty');
		expect(wishView([], [], [])).toBe('no_data');
	});

	it('never shows a failed read as an empty list', () => {
		const failed = status({ store: 'gog', last_state: 'failed', last_complete_at: null });
		expect(wishView([], [], [status({}), failed])).toBe('sync_failed');
	});
});
