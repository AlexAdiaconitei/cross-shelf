// Filters and order of the cleanup review. Pure, tested without a window. Nothing here selects
// anything for removal: it only decides what to show and in which order (spec 8.1 step 4).
import type { MatchView } from '#lib/bindings/MatchView.ts';
import type { ReviewItem } from '#lib/bindings/ReviewItem.ts';
import { normalize } from '#lib/library/filter.ts';

export type ReviewFilter = 'to_review' | 'kept' | 'no_match' | 'repeated' | 'all';

/** Evidence class order: hard evidence first, then candidates, then what is not covered. */
const outcomeRank: Record<string, number> = {
	exact: 0,
	candidate: 1,
	unresolved: 2,
	not_covered: 3,
	family_excluded: 4
};

export function sortMatches(ms: MatchView[]): MatchView[] {
	return [...ms].sort(
		(a, b) =>
			(outcomeRank[a.outcome] ?? 9) - (outcomeRank[b.outcome] ?? 9) ||
			a.held.store.localeCompare(b.held.store) ||
			a.held.native_id.localeCompare(b.held.native_id)
	);
}

/** Best (lowest) rank among an item's matches, or Infinity with none. */
export function bestRank(it: ReviewItem): number {
	return Math.min(...it.matches.map((m) => outcomeRank[m.outcome] ?? 9), Infinity);
}

/** An item asks for a decision when something might cover it and it is not kept. */
export function needsReview(it: ReviewItem): boolean {
	return !it.kept && bestRank(it) <= outcomeRank.unresolved!;
}

export function applyReview(items: ReviewItem[], filter: ReviewFilter, text: string): ReviewItem[] {
	const q = normalize(text);
	return items
		.filter((it) => {
			switch (filter) {
				case 'to_review':
					if (!needsReview(it)) return false;
					break;
				case 'kept':
					if (!it.kept) return false;
					break;
				case 'no_match':
					if (it.matches.length > 0) return false;
					break;
				case 'repeated':
					if (it.repeated_in.length === 0) return false;
					break;
			}
			if (!q) return true;
			const titles = [it.wanted.title, ...it.matches.map((m) => m.held.title)];
			return titles.some((t) => t !== null && normalize(t).includes(q));
		})
		.sort(
			(a, b) =>
				bestRank(a) - bestRank(b) ||
				normalize(a.wanted.title ?? '￿').localeCompare(normalize(b.wanted.title ?? '￿'))
		);
}

export function reviewCounts(items: ReviewItem[]): Record<ReviewFilter, number> {
	return {
		to_review: items.filter(needsReview).length,
		kept: items.filter((i) => i.kept).length,
		no_match: items.filter((i) => i.matches.length === 0).length,
		repeated: items.filter((i) => i.repeated_in.length > 0).length,
		all: items.length
	};
}

export const itemKey = (it: ReviewItem) =>
	`${it.account_id}:${it.wanted.store}:${it.wanted.native_id}`;

/**
 * The exact match that justifies removing the entry: the same store ID, «Es el mismo» or a
 * pack's credited component. A candidate by title never is (spec 7.2). The backend checks the
 * same rule again before building a batch.
 */
export function coveringMatch(it: ReviewItem): MatchView | null {
	if (it.kept) return null;
	return sortMatches(it.matches).find((m) => m.outcome === 'exact') ?? null;
}

export const removable = (it: ReviewItem) => coveringMatch(it) !== null;

export type BatchGroup = { store: string; account_id: string; items: ReviewItem[] };

/** Selected entries grouped by store and account: one batch each. Only removable ones enter. */
export function groupSelection(items: ReviewItem[], selected: ReadonlySet<string>): BatchGroup[] {
	const groups = new Map<string, BatchGroup>();
	for (const it of items) {
		if (!removable(it) || !selected.has(itemKey(it))) continue;
		const k = `${it.wanted.store}:${it.account_id}`;
		const g = groups.get(k) ?? { store: it.wanted.store, account_id: it.account_id, items: [] };
		g.items.push(it);
		groups.set(k, g);
	}
	return [...groups.values()].sort((a, b) => a.store.localeCompare(b.store));
}
