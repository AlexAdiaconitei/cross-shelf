// Search, filters and counters of the library view. Pure functions over what the database
// returned, so they are tested without a window. Text matching ignores case and accents (spec 9.2).

import type { Access as AccessRow } from '#lib/bindings/Access.ts';
import type { LibraryEntry } from '#lib/bindings/LibraryEntry.ts';
import type { SyncStatus } from '#lib/bindings/SyncStatus.ts';

export type Access = AccessRow;
export type Entry = LibraryEntry;

export type Scope = 'all' | 'own' | 'family' | 'family_only';

export type Filters = {
	text: string;
	store: string;
	scope: Scope;
	kind: string;
};

export const noFilters: Filters = { text: '', store: '', scope: 'all', kind: '' };

export function normalize(s: string): string {
	// Marks go first: NFKD would turn ™ into the letters "tm".
	return s
		.replace(/[™®©℠]/g, '')
		.normalize('NFKD')
		.replace(/\p{M}/gu, '')
		.toLowerCase()
		.replace(/\s+/g, ' ')
		.trim();
}

const hasClass = (e: Entry, c: string) => e.access.some((a) => a.class === c);

function matchesScope(e: Entry, scope: Scope): boolean {
	switch (scope) {
		case 'all':
			return true;
		case 'own':
			return hasClass(e, 'owned');
		case 'family':
			return hasClass(e, 'family_shared');
		case 'family_only':
			return hasClass(e, 'family_shared') && !hasClass(e, 'owned');
	}
}

export function isFiltering(f: Filters): boolean {
	return f.text.trim() !== '' || f.store !== '' || f.scope !== 'all' || f.kind !== '';
}

export function applyFilters(entries: Entry[], f: Filters): Entry[] {
	const q = normalize(f.text);
	return entries.filter((e) => {
		if (f.store && e.store !== f.store) return false;
		if (f.kind && e.kind !== f.kind) return false;
		if (!matchesScope(e, f.scope)) return false;
		if (q && !normalize(e.title ?? '').includes(q)) return false;
		return true;
	});
}

export type Counters = {
	/** Products or licences per store, over the unfiltered list. */
	byStore: Record<string, number>;
	/** Entries that carry family access, over the unfiltered list. */
	familyAccess: number;
	total: number;
	shown: number;
};

export function counters(all: Entry[], shown: Entry[]): Counters {
	const byStore: Record<string, number> = {};
	let familyAccess = 0;
	for (const e of all) {
		byStore[e.store] = (byStore[e.store] ?? 0) + 1;
		if (hasClass(e, 'family_shared')) familyAccess += 1;
	}
	return { byStore, familyAccess, total: all.length, shown: shown.length };
}

/** Distinct product kinds present, with "unknown" always visible when it occurs (spec 9.2). */
export function kindsPresent(entries: Entry[]): string[] {
	return [...new Set(entries.map((e) => e.kind))].sort();
}

export type View = 'no_data' | 'sync_failed' | 'no_match' | 'list';

/**
 * Different empty states: nothing synced, a failed sync with no copy to show, filters that
 * excluded everything, or a list.
 */
export function viewState(all: Entry[], shown: Entry[], status: SyncStatus[] = []): View {
	if (all.length === 0) {
		const failedNeverRead = status.some(
			(s) => s.resource !== 'wishlist' && s.last_state === 'failed' && s.last_complete_at === null
		);
		return failedNeverRead ? 'sync_failed' : 'no_data';
	}
	return shown.length === 0 ? 'no_match' : 'list';
}

/** Tabs of the library view (spec 9.1). Each is a preset of store and access scope. */
export type Tab = 'all' | 'steam' | 'steam_family' | 'gog' | 'epic';

export const tabs: Tab[] = ['all', 'steam', 'steam_family', 'gog', 'epic'];

export function tabPreset(tab: Tab): Pick<Filters, 'store' | 'scope'> {
	switch (tab) {
		case 'all':
			return { store: '', scope: 'all' };
		case 'steam':
			return { store: 'steam', scope: 'own' };
		case 'steam_family':
			return { store: 'steam', scope: 'family' };
		default:
			return { store: tab, scope: 'all' };
	}
}

/** The tab whose preset the filters match, if any. */
export function activeTab(f: Filters): Tab | null {
	return (
		tabs.find((t) => {
			const p = tabPreset(t);
			return p.store === f.store && p.scope === f.scope;
		}) ?? null
	);
}

/** The window of rows to render for a fixed row height. */
export function visibleRange(
	scrollTop: number,
	viewport: number,
	rowHeight: number,
	count: number,
	overscan = 6
): { start: number; end: number } {
	const first = Math.floor(scrollTop / rowHeight) - overscan;
	const last = Math.ceil((scrollTop + viewport) / rowHeight) + overscan;
	return { start: Math.max(0, first), end: Math.min(count, Math.max(0, last)) };
}
