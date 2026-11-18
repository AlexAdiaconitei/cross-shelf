// Groups store products into one row per game for display (spec 9.1, «fichas agrupadas»).
// The key is the product kind plus the normalized title. It only decides what is drawn
// together: it is never evidence that two products are the same game, and the cleanup never
// reads it. The view keeps a switch to separate records.
import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
import { normalize, type Entry } from './filter.ts';

export const stores = ['steam', 'gog', 'epic'] as const;
export type Store = (typeof stores)[number];

/** What one store says about one game, strongest first. */
export type Slot = 'own' | 'temp' | 'family' | 'unknown' | 'wish' | 'none';

const strength: Slot[] = ['own', 'temp', 'family', 'unknown', 'wish', 'none'];
const fromAccess: Record<string, Slot> = {
	owned: 'own',
	temporary: 'temp',
	family_shared: 'family',
	unknown: 'unknown'
};

export type Shelf = {
	key: string;
	title: string;
	kind: string;
	slots: Record<Store, Slot>;
	products: Entry[];
	wished: WishlistEntry[];
	/** Wished in a store while you own it in another one: worth a cleanup review. */
	wishedElsewhere: boolean;
	familyOwners: string[];
};

export const shelfKey = (kind: string, title: string | null, nativeId: string) =>
	`${kind}|${title ? normalize(title) : `#${nativeId}`}`;

const stronger = (a: Slot, b: Slot) => (strength.indexOf(a) <= strength.indexOf(b) ? a : b);

export function buildShelf(entries: Entry[], wishlist: WishlistEntry[] = []): Shelf[] {
	const rows = new Map<string, Shelf>();
	for (const e of entries) {
		const key = shelfKey(e.kind, e.title, e.native_id);
		let row = rows.get(key);
		if (!row) {
			row = {
				key,
				title: e.title ?? e.native_id,
				kind: e.kind,
				slots: { steam: 'none', gog: 'none', epic: 'none' },
				products: [],
				wished: [],
				wishedElsewhere: false,
				familyOwners: []
			};
			rows.set(key, row);
		}
		row.products.push(e);
		const s = e.store as Store;
		if (!(s in row.slots)) continue;
		for (const a of e.access) {
			row.slots[s] = stronger(row.slots[s], fromAccess[a.class] ?? 'unknown');
			if (a.class === 'family_shared') row.familyOwners.push(...a.owners);
		}
		if (e.access.length === 0) row.slots[s] = stronger(row.slots[s], 'unknown');
	}
	// Wishlist entries only mark rows already shown: the library lists what you hold.
	for (const w of wishlist) {
		const row = rows.get(shelfKey('game', w.title, w.native_id));
		if (!row) continue;
		row.wished.push(w);
		const s = w.store as Store;
		if (s in row.slots && row.slots[s] === 'none') row.slots[s] = 'wish';
	}
	for (const row of rows.values()) {
		row.familyOwners = [...new Set(row.familyOwners)];
		const owned = stores.some((s) => row.slots[s] === 'own');
		row.wishedElsewhere = owned && stores.some((s) => row.slots[s] === 'wish');
	}
	return [...rows.values()].sort((a, b) =>
		normalize(a.title).localeCompare(normalize(b.title), 'es')
	);
}

/** Two-letter initials for the cover fallback. */
export function initials(title: string): string {
	return title
		.replace(/[^\p{L}\p{N} ]/gu, '')
		.split(' ')
		.filter(Boolean)
		.slice(0, 2)
		.map((w) => w[0]!.toUpperCase())
		.join('');
}

/** The store whose colour a game wears: the first one where it appears. */
export const firstStore = (row: Pick<Shelf, 'slots'>): Store =>
	stores.find((s) => row.slots[s] !== 'none') ?? 'steam';
