import { describe, expect, it } from 'vitest';
import type { MatchView } from '#lib/bindings/MatchView.ts';
import type { ReviewItem } from '#lib/bindings/ReviewItem.ts';
import {
	applyReview,
	coveringMatch,
	groupSelection,
	removable,
	itemKey,
	needsReview,
	reviewCounts,
	sortMatches
} from './view.ts';

const ref = (store: string, id: string, title: string | null) => ({
	store,
	native_id: id,
	title,
	kind: 'game',
	windows: 'yes',
	url: null,
	image_url: null
});

function match(outcome: string, id = 'g1', title = 'Lumen Drift'): MatchView {
	return {
		held: ref('gog', id, title),
		access: 'owned',
		family_owners: 0,
		outcome,
		edition_superior: false,
		platform_unknown: false,
		reason: null,
		evidence: null,
		manual: null
	};
}

function item(id: string, title: string | null, matches: MatchView[], over = {}): ReviewItem {
	return {
		account_id: 'a',
		wanted: ref('steam', id, title),
		priority: null,
		added_at: null,
		observed_at: 1,
		kept: false,
		keep_reason: null,
		matches,
		repeated_in: [],
		...over
	};
}

const items = [
	item('1', 'Zeta', [match('candidate')]),
	item('2', 'Alfa', [match('not_covered')]),
	item('3', 'Beta', [match('exact')]),
	item('4', 'Gamma', []),
	item('5', 'Delta', [match('candidate')], { kept: true }),
	item('6', 'Épsilon', [match('family_excluded')], { repeated_in: [ref('gog', 'x', 'Épsilon')] })
];

describe('review view', () => {
	it('asks for review only when something might cover the wish and it is not kept', () => {
		expect(items.filter(needsReview).map((i) => i.wanted.native_id)).toEqual(['1', '3']);
	});

	it('orders by evidence class, hard evidence first', () => {
		const out = applyReview(items, 'all', '');
		expect(out.map((i) => i.wanted.native_id).slice(0, 3)).toEqual(['3', '5', '1']);
		expect(out.at(-1)?.wanted.native_id).toBe('4');
		expect(sortMatches([match('not_covered'), match('exact')])[0]?.outcome).toBe('exact');
	});

	it('filters kept, unmatched and repeated wishes apart', () => {
		const ids = (f: Parameters<typeof applyReview>[1]) =>
			applyReview(items, f, '').map((i) => i.wanted.native_id);
		expect(ids('kept')).toEqual(['5']);
		expect(ids('no_match')).toEqual(['4']);
		expect(ids('repeated')).toEqual(['6']);
		expect(reviewCounts(items)).toEqual({
			to_review: 2,
			kept: 1,
			no_match: 1,
			repeated: 1,
			all: 6
		});
	});

	it('searches wanted and held titles, ignoring accents', () => {
		expect(applyReview(items, 'all', 'epsilon').map((i) => i.wanted.native_id)).toEqual(['6']);
		expect(applyReview(items, 'all', 'lumen')).toHaveLength(5);
	});
});

describe('groupSelection', () => {
	it('makes one group per store and account with only removable entries', () => {
		const gogWish = { ...item('9', 'Nu', [match('exact')]), wanted: ref('gog', 'g9', 'Nu') };
		const all = [...items, gogWish];
		const sel = new Set(all.map(itemKey));
		const groups = groupSelection(all, sel);
		// Of the Steam wishes only «Beta» has an exact match: candidates, unmatched, family
		// excluded and kept entries never enter, even if selected.
		expect(groups.map((g) => [g.store, g.items.map((i) => i.wanted.native_id)])).toEqual([
			['gog', ['g9']],
			['steam', ['3']]
		]);
	});

	it('a candidate becomes removable only after «Es el mismo» turns it exact', () => {
		const candidate = item('1', 'Zeta', [match('candidate'), match('not_covered', 'g2')]);
		expect(removable(candidate)).toBe(false);
		const decided = item('1', 'Zeta', [{ ...match('exact'), manual: 'same' }]);
		expect(coveringMatch(decided)?.held.native_id).toBe('g1');
		expect(removable({ ...decided, kept: true })).toBe(false);
	});

	it('is empty when nothing is selected: nothing is preselected', () => {
		expect(groupSelection(items, new Set())).toEqual([]);
	});
});
