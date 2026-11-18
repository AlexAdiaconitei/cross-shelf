import { describe, expect, it } from 'vitest';
import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
import type { Entry } from './filter.ts';
import { buildShelf, coverOf, firstStore, initials } from './shelf.ts';

const access = (cls: string, owners: string[] = []) => ({
	class: cls,
	owners,
	owner_names: owners.length ? ['Marta'] : [],
	exclude_reason: null,
	observed_at: 0
});

const entry = (store: string, id: string, title: string | null, cls = 'owned', kind = 'game') =>
	({
		store,
		account_id: 'a',
		native_id: id,
		title,
		kind,
		windows: 'yes',
		url: null,
		image_url: null,
		access: [access(cls, cls === 'family_shared' ? ['Marta'] : [])]
	}) satisfies Entry;

const wish = (store: string, id: string, title: string): WishlistEntry => ({
	store,
	account_id: 'a',
	native_id: id,
	title,
	url: null,
	image_url: null,
	priority: null,
	added_at: null,
	confirmed: true,
	already_held_here: false
});

describe('buildShelf', () => {
	it('groups the same title across stores, ignoring case, accents and ™', () => {
		const rows = buildShelf([
			entry('steam', '1', 'Hades™'),
			entry('epic', 'h', 'hades'),
			entry('gog', 'x', 'Pokémon')
		]);
		expect(rows.map((r) => r.title)).toEqual(['Hades™', 'Pokémon']);
		expect(rows[0]!.slots).toEqual({ steam: 'own', gog: 'none', epic: 'own' });
		expect(rows[0]!.products).toHaveLength(2);
	});

	it('never groups a DLC or a soundtrack with its game', () => {
		const rows = buildShelf([
			entry('steam', '1', 'Hades'),
			entry('steam', '2', 'Hades', 'owned', 'soundtrack')
		]);
		expect(rows).toHaveLength(2);
	});

	it('keeps untitled products apart by their id', () => {
		expect(buildShelf([entry('steam', '1', null), entry('steam', '2', null)])).toHaveLength(2);
	});

	it('keeps the strongest access per store and lists family owners once', () => {
		const rows = buildShelf([
			entry('steam', '1', 'Celeste', 'family_shared'),
			entry('steam', '2', 'Celeste', 'family_shared'),
			entry('steam', '3', 'Celeste', 'owned')
		]);
		expect(rows[0]!.slots.steam).toBe('own');
		expect(rows[0]!.familyOwners).toEqual(['Marta']);
		expect(rows[0]!.familyNames).toEqual(['Marta']);
	});

	it('marks a game wished in one store while owned in another', () => {
		const rows = buildShelf(
			[entry('gog', 'g', 'Cyberpunk 2077')],
			[wish('steam', '1091500', 'Cyberpunk 2077'), wish('epic', 'z', 'Not In Library')]
		);
		expect(rows).toHaveLength(1);
		expect(rows[0]!.slots.steam).toBe('wish');
		expect(rows[0]!.wishedElsewhere).toBe(true);
	});

	it('does not call family access a reason to clean up', () => {
		const rows = buildShelf(
			[entry('steam', '1', 'Dead Cells', 'family_shared')],
			[wish('gog', 'd', 'Dead Cells')]
		);
		expect(rows[0]!.wishedElsewhere).toBe(false);
	});
});

describe('helpers', () => {
	it('initials keep two words and drop symbols', () => {
		expect(initials('Baldur’s Gate 3')).toBe('BG');
		expect(initials('Control')).toBe('C');
	});

	it('cover comes from the first product that has one, then from wishes', () => {
		const a = { ...entry('steam', '1', 'Hades'), image_url: null };
		const b = { ...entry('epic', 'h', 'Hades'), image_url: 'https://cdn1.epicgames.com/h' };
		expect(coverOf(buildShelf([a, b])[0]!)).toBe('https://cdn1.epicgames.com/h');
		const w = { ...wish('gog', 'g', 'Hades'), image_url: 'https://images-1.gog-statics.com/x' };
		expect(coverOf(buildShelf([a], [w])[0]!)).toBe('https://images-1.gog-statics.com/x');
		expect(coverOf(buildShelf([a])[0]!)).toBeNull();
	});

	it('first store follows Steam, GOG, Epic order', () => {
		expect(firstStore({ slots: { steam: 'none', gog: 'wish', epic: 'own' } })).toBe('gog');
	});
});
