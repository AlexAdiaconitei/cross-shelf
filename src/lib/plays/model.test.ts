import { describe, expect, it } from 'vitest';
import type { LibraryEntry } from '#lib/bindings/LibraryEntry.ts';
import type { PlayView } from '#lib/bindings/PlayView.ts';
import { tr } from '#lib/i18n/index.ts';
import type { DiaryRow } from '#lib/bindings/DiaryRow.ts';
import {
	addable,
	ago,
	columns,
	diaryGroup,
	diaryGroups,
	diarySummary,
	hltbUrl,
	hours,
	playsTab,
	sheetTotal,
	storeTimes,
	timeLine
} from './model.ts';

const DAY = 86_400;
const NOW = 1_800_000_000;

const entry = (id: string, title: string, store = 'steam', owned = true): LibraryEntry => ({
	store,
	account_id: 'a',
	native_id: id,
	title,
	kind: 'game',
	windows: 'yes',
	url: null,
	image_url: null,
	access: owned
		? [{ class: 'owned', owners: [], owner_names: [], exclude_reason: null, observed_at: 1 }]
		: [
				{
					class: 'family_shared',
					owners: ['x'],
					owner_names: [],
					exclude_reason: null,
					observed_at: 1
				}
			]
});

const play = (id: string, state: string, position: number, store = 'steam'): PlayView => ({
	product: {
		store,
		native_id: id,
		title: `Game ${id}`,
		kind: 'game',
		windows: 'yes',
		url: null,
		image_url: null
	},
	state,
	position,
	since: NOW - DAY,
	started_at: null,
	finished_at: null,
	note: '',
	in_library: true,
	minutes: null,
	last_activity: null
});

describe('hours', () => {
	it('tells "the store does not say" apart from zero', () => {
		expect(hours(null)).toBe(tr.plays.noTime);
		expect(hours(0)).toBe(tr.plays.minutes(0));
	});
	it('uses minutes under an hour, one decimal under ten hours, whole hours above', () => {
		expect(hours(45)).toBe(tr.plays.minutes(45));
		expect(hours(90)).toBe(tr.plays.hours('1.5'));
		expect(hours(120)).toBe(tr.plays.hours('2'));
		expect(hours(754)).toBe(tr.plays.hours('13'));
	});
});

describe('ago', () => {
	it('says today, days, months and years', () => {
		expect(ago(NOW - 60, NOW)).toBe(tr.plays.today);
		expect(ago(NOW - 3 * DAY, NOW)).toMatch(/3 days ago/);
		expect(ago(NOW - 70 * DAY, NOW)).toMatch(/2 months ago/);
		expect(ago(NOW - 800 * DAY, NOW)).toMatch(/2 years ago/);
	});
});

describe('timeLine', () => {
	it('joins hours and last session, and is empty when nothing is known', () => {
		expect(timeLine(754, NOW - 3 * DAY, NOW)).toBe(`${tr.plays.hours('13')} · 3 days ago`);
		expect(timeLine(60, null, NOW)).toBe(tr.plays.hours('1'));
		expect(timeLine(null, null, NOW)).toBe('');
	});
});

describe('diaryGroup', () => {
	it('puts each game in this week, this month, long ago or no date', () => {
		expect(diaryGroup(NOW - 2 * DAY, NOW)).toBe('week');
		expect(diaryGroup(NOW - 7 * DAY, NOW)).toBe('week');
		expect(diaryGroup(NOW - 8 * DAY, NOW)).toBe('month');
		expect(diaryGroup(NOW - 31 * DAY, NOW)).toBe('month');
		expect(diaryGroup(NOW - 40 * DAY, NOW)).toBe('older');
		expect(diaryGroup(null, NOW)).toBe('none');
	});
});

describe('playsTab', () => {
	it('keeps a known tab and falls back to the board', () => {
		expect(playsTab('diary')).toBe('diary');
		expect(playsTab('list')).toBe('list');
		expect(playsTab('kanban')).toBe('board');
	});
});

describe('columns', () => {
	it('gives the five states in board order, each in the user order', () => {
		const c = columns([play('1', 'backlog', 2), play('2', 'playing', 1), play('3', 'backlog', 1)]);
		expect(c.map((x) => x.state)).toEqual(['playing', 'backlog', 'paused', 'completed', 'dropped']);
		expect(c[1]!.items.map((p) => p.product.native_id)).toEqual(['3', '1']);
	});
});

describe('addable', () => {
	const library = [
		entry('1', 'Celeste'),
		entry('2', 'Hades'),
		entry('3', 'Hades II', 'steam', false),
		entry('9', 'Céleste Classic', 'gog')
	];
	it('finds only games you hold that are not on the board yet, ignoring accents', () => {
		const got = addable(library, [play('1', 'playing', 1)], 'celeste');
		expect(got.map((e) => `${e.store}:${e.native_id}`)).toEqual(['gog:9']);
	});
	it('includes family access, which you can play, and finds nothing for an empty search', () => {
		expect(addable(library, [], 'hades').map((e) => e.native_id)).toEqual(['2', '3']);
		expect(addable(library, [], '  ')).toEqual([]);
	});
});

describe('diarySummary', () => {
	const row = (
		id: string,
		state: string | null,
		deduced: string | null,
		week: number | null
	): DiaryRow => ({
		product: play(id, 'backlog', 1).product,
		minutes: 100,
		last_activity: NOW,
		week_minutes: week,
		state,
		deduced
	});
	it('adds the week, counts what is in progress and what waits for confirmation', () => {
		const s = diarySummary([
			row('1', 'playing', null, 60),
			row('2', null, 'playing', 30),
			row('3', 'completed', null, null),
			row('4', null, null, 0)
		]);
		expect(s).toEqual({ weekMinutes: 90, inProgress: 2, toConfirm: 1 });
	});
	it('groups rows by last activity, in group order, leaving empty groups out', () => {
		const old = { ...row('5', null, null, null), last_activity: NOW - 40 * DAY };
		const none = { ...row('6', null, null, null), last_activity: null };
		const g = diaryGroups([row('1', 'playing', null, 1), none, old], NOW);
		expect(g.map((x) => [x.group, x.rows.map((r) => r.product.native_id)])).toEqual([
			['week', ['1']],
			['older', ['5']],
			['none', ['6']]
		]);
	});
});

describe('sheet time', () => {
	const t = (store: string, id: string, minutes: number | null, last: number | null = null) => ({
		store,
		native_id: id,
		minutes,
		last_played: last
	});
	it('gives one line per store, telling "does not give it" from zero', () => {
		const lines = storeTimes(
			[
				{ store: 'steam', native_id: '1' },
				{ store: 'epic', native_id: 'e' },
				{ store: 'gog', native_id: 'g' }
			],
			[t('steam', '1', 0), t('epic', 'e', 300)],
			['steam'],
			{}
		);
		expect(lines).toEqual([
			{ store: 'steam', state: 'time', minutes: 0 },
			{ store: 'epic', state: 'not_given', minutes: null },
			{ store: 'gog', state: 'not_given', minutes: null }
		]);
	});
	it('shows GOG loading, failing or answering once it is validated', () => {
		const p = [{ store: 'gog', native_id: 'g' }];
		expect(storeTimes(p, [], ['gog'], {})[0]!.state).toBe('loading');
		expect(storeTimes(p, [], ['gog'], { 'gog:g': 'error' })[0]!.state).toBe('error');
		expect(storeTimes(p, [], ['gog'], { 'gog:g': null })[0]!.state).toBe('not_given');
		expect(storeTimes(p, [], ['gog'], { 'gog:g': 90 })[0]).toEqual({
			store: 'gog',
			state: 'time',
			minutes: 90
		});
	});
	it('adds only known minutes and keeps the latest session', () => {
		const s = sheetTotal(
			[
				{ store: 'steam', state: 'time', minutes: 60 },
				{ store: 'gog', state: 'time', minutes: 30 },
				{ store: 'epic', state: 'not_given', minutes: null }
			],
			[t('steam', '1', 60, 500), t('steam', '2', 0, 900)]
		);
		expect(s).toEqual({ minutes: 90, last: 900 });
		expect(sheetTotal([{ store: 'epic', state: 'not_given', minutes: null }], [])).toEqual({
			minutes: null,
			last: null
		});
	});
	it('links to a HowLongToBeat search, nothing more', () => {
		expect(hltbUrl('Hades II: Deluxe & Co')).toBe(
			'https://howlongtobeat.com/?q=Hades%20II%3A%20Deluxe%20%26%20Co'
		);
	});
});
