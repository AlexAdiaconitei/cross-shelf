// Search, filters, order and empty states of the Wishlists view. Pure, tested without a window.
import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';
import type { WishlistEntry } from '#lib/bindings/WishlistEntry.ts';
import { normalize } from '#lib/library/filter.ts';

export type WishFilter = 'all' | 'held_here' | 'unconfirmed' | 'untitled';
export type WishSort = 'priority' | 'added' | 'title';

export type WishFilters = {
	text: string;
	store: string;
	status: WishFilter;
	sort: WishSort;
};

export const noWishFilters: WishFilters = { text: '', store: '', status: 'all', sort: 'priority' };

export function isWishFiltering(f: WishFilters): boolean {
	return f.text.trim() !== '' || f.store !== '' || f.status !== 'all';
}

function matchesStatus(w: WishlistEntry, s: WishFilter): boolean {
	switch (s) {
		case 'all':
			return true;
		case 'held_here':
			return w.already_held_here;
		case 'unconfirmed':
			return !w.confirmed;
		case 'untitled':
			return w.title === null;
	}
}

// Missing values always sort last, whatever the order: a priority or date the store does not
// give is never invented.
function byNullable(a: number | null, b: number | null, desc: boolean): number {
	if (a === null && b === null) return 0;
	if (a === null) return 1;
	if (b === null) return -1;
	return desc ? b - a : a - b;
}

const titleKey = (w: WishlistEntry) => normalize(w.title ?? '￿');

export function applyWishFilters(entries: WishlistEntry[], f: WishFilters): WishlistEntry[] {
	const q = normalize(f.text);
	const out = entries.filter((w) => {
		if (f.store && w.store !== f.store) return false;
		if (!matchesStatus(w, f.status)) return false;
		if (q && !normalize(w.title ?? '').includes(q)) return false;
		return true;
	});
	return out.sort((a, b) => {
		const c =
			f.sort === 'priority'
				? byNullable(a.priority, b.priority, false)
				: f.sort === 'added'
					? byNullable(a.added_at, b.added_at, true)
					: 0;
		return c || titleKey(a).localeCompare(titleKey(b)) || a.native_id.localeCompare(b.native_id);
	});
}

export type WishView = 'no_data' | 'sync_failed' | 'confirmed_empty' | 'no_match' | 'list';

/**
 * Four different empty states (spec 6.3, 9.2): nothing synced yet, a failed sync with no copy,
 * an empty list the store confirmed, and filters that hide everything.
 */
export function wishView(
	all: WishlistEntry[],
	shown: WishlistEntry[],
	status: SyncStatus[]
): WishView {
	if (all.length > 0) return shown.length === 0 ? 'no_match' : 'list';
	const wl = status.filter((s) => s.resource === 'wishlist');
	// A store that failed without any good copy wins: "empty" would hide that it was never read.
	if (wl.some((s) => s.last_complete_at === null && s.last_state === 'failed'))
		return 'sync_failed';
	if (wl.length > 0 && wl.every((s) => s.last_complete_at !== null)) return 'confirmed_empty';
	return 'no_data';
}
