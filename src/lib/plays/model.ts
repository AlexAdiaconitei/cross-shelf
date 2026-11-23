// Mis partidas: pure helpers over what the database returned, tested without a window.
import type { DiaryRow } from '#lib/bindings/DiaryRow.ts';
import type { LibraryEntry } from '#lib/bindings/LibraryEntry.ts';
import type { PlayView } from '#lib/bindings/PlayView.ts';
import type { ProductTime } from '#lib/bindings/ProductTime.ts';
import { intl, tr } from '#lib/i18n/index.ts';
import { PLAY_STATES, type PlayState } from '#lib/ipc.ts';
import { normalize } from '#lib/library/filter.ts';

const DAY = 86_400;

/** Time played. `null` (the store does not say) is never shown as zero. */
export function hours(minutes: number | null): string {
	if (minutes === null) return tr.plays.noTime;
	if (minutes < 60) return tr.plays.minutes(minutes);
	const h = minutes / 60;
	const text =
		h < 10
			? h.toLocaleString(intl, { maximumFractionDigits: 1 })
			: Math.round(h).toLocaleString(intl);
	return tr.plays.hours(text);
}

const rel = new Intl.RelativeTimeFormat(intl, { numeric: 'auto' });

/** "today", "3 days ago", "2 months ago"… for a moment in seconds. */
export function ago(secs: number, now: number): string {
	const days = Math.floor((now - secs) / DAY);
	if (days < 1) return tr.plays.today;
	if (days < 31) return rel.format(-days, 'day');
	if (days < 365) return rel.format(-Math.round(days / 30), 'month');
	return rel.format(-Math.round(days / 365), 'year');
}

/** "13 h · 3 days ago" under a card title; empty when nothing is known. */
export function timeLine(minutes: number | null, last: number | null, now: number): string {
	const parts: string[] = [];
	if (minutes !== null) parts.push(hours(minutes));
	if (last !== null) parts.push(ago(last, now));
	return parts.join(' · ');
}

export type DiaryGroup = 'week' | 'month' | 'older' | 'none';

/** Where the diary lists a game, by its last activity. */
export function diaryGroup(last: number | null, now: number): DiaryGroup {
	if (last === null) return 'none';
	const days = (now - last) / DAY;
	if (days <= 7) return 'week';
	if (days <= 31) return 'month';
	return 'older';
}

export const PLAYS_TABS = ['board', 'list', 'diary'] as const;
export type PlaysTab = (typeof PLAYS_TABS)[number];

export const playsTab = (v: string): PlaysTab =>
	(PLAYS_TABS as readonly string[]).includes(v) ? (v as PlaysTab) : 'board';

export const keyOf = (p: { store: string; native_id: string }) => `${p.store}:${p.native_id}`;

/** The five states in board order, each with its plays in the user's order. */
export function columns(plays: PlayView[]): { state: PlayState; items: PlayView[] }[] {
	return PLAY_STATES.map((state) => ({
		state,
		items: plays.filter((p) => p.state === state).sort((a, b) => a.position - b.position)
	}));
}

/** Library products matching a search that you can play and that are not on the board yet. */
export function addable(library: LibraryEntry[], plays: PlayView[], q: string): LibraryEntry[] {
	const text = normalize(q);
	if (!text) return [];
	const taken = new Set(plays.map((p) => keyOf(p.product)));
	const seen = new Set<string>();
	return library
		.filter((e) => e.access.length > 0 && !taken.has(keyOf(e)))
		.filter((e) => normalize(e.title ?? '').includes(text))
		.filter((e) => !seen.has(keyOf(e)) && seen.add(keyOf(e)))
		.slice(0, 8);
}

/** The date line of a card: started, finished, or how long it has been in its state. */
export function sinceText(p: PlayView, now: number): string {
	if (p.state === 'completed' && p.finished_at !== null)
		return tr.plays.finished(ago(p.finished_at, now));
	if (p.state === 'playing' && p.started_at !== null)
		return tr.plays.started(ago(p.started_at, now));
	return tr.plays.inState(tr.plays.states[p.state as PlayState], ago(p.since, now));
}

export const nowSecs = () => Math.floor(Date.now() / 1000);

const GROUPS: DiaryGroup[] = ['week', 'month', 'older', 'none'];

/** Diary rows by last activity, in group order; empty groups are left out. */
export function diaryGroups(
	rows: DiaryRow[],
	now: number
): { group: DiaryGroup; rows: DiaryRow[] }[] {
	return GROUPS.map((group) => ({
		group,
		rows: rows.filter((r) => diaryGroup(r.last_activity, now) === group)
	})).filter((g) => g.rows.length);
}

/** The diary header: hours this week, games in progress (set or deduced), states to confirm. */
export function diarySummary(rows: DiaryRow[]) {
	return {
		// Null when no game has a read from a week ago: unknown is not zero.
		weekMinutes: rows.some((r) => r.week_minutes !== null)
			? rows.reduce((a, r) => a + (r.week_minutes ?? 0), 0)
			: null,
		inProgress: rows.filter((r) => (r.state ?? r.deduced) === 'playing').length,
		toConfirm: rows.filter((r) => !r.state && r.deduced).length
	};
}

/** One store's line in the sheet's «Time» block. */
export type StoreTime = {
	store: string;
	/** `none`: the store gives hours, but has none for this game (not synced, family access). */
	state: 'time' | 'none' | 'not_given' | 'loading' | 'error';
	minutes: number | null;
};

/** GOG answers per product (`store:id`): minutes, `null` (no record) or `'error'`. */
export type GogAnswers = Record<string, number | null | 'error'>;

/**
 * One line per store of the sheet's products. A store that does not give play time in this
 * build, or gives none for the product, says so; it is never a zero.
 */
export function storeTimes(
	products: { store: string; native_id: string }[],
	times: ProductTime[],
	given: string[],
	gog: GogAnswers
): StoreTime[] {
	const byStore = new Map<string, StoreTime>();
	for (const p of products) {
		if (byStore.has(p.store)) continue;
		let line: StoreTime = { store: p.store, state: 'not_given', minutes: null };
		if (given.includes(p.store)) {
			if (p.store === 'gog') {
				const a = gog[keyOf(p)];
				if (a === undefined) line = { ...line, state: 'loading' };
				else if (a === 'error') line = { ...line, state: 'error' };
				else if (a !== null) line = { ...line, state: 'time', minutes: a };
			} else {
				const m = times.find((t) => t.store === p.store && t.native_id === p.native_id)?.minutes;
				line =
					m !== null && m !== undefined
						? { ...line, state: 'time', minutes: m }
						: { ...line, state: 'none' };
			}
		}
		byStore.set(p.store, line);
	}
	return [...byStore.values()];
}

/** Total of the known minutes (`null` when none is known) and the latest session. */
export function sheetTotal(lines: StoreTime[], times: ProductTime[]) {
	const known = lines.filter((l) => l.state === 'time');
	const last = times.reduce<number | null>(
		(a, t) => (t.last_played !== null && (a === null || t.last_played > a) ? t.last_played : a),
		null
	);
	return {
		minutes: known.length ? known.reduce((a, l) => a + (l.minutes ?? 0), 0) : null,
		last
	};
}

/** HowLongToBeat search for a title. Only a link: the app never asks HowLongToBeat anything. */
export const hltbUrl = (title: string) =>
	`https://howlongtobeat.com/?q=${encodeURIComponent(title)}`;

/**
 * Where a dragged card lands in a column of card keys: ahead of the card under the pointer from
 * its upper half, behind it from its lower half, at the end below every card (`over` null). The
 * dragged card is skipped, so dropping it next to itself keeps its place.
 */
export function dropPlace(
	keys: string[],
	over: number | null,
	lowerHalf: boolean,
	dragged: string
): { before: string } | { end: true } {
	let i = over === null ? keys.length : over + (lowerHalf ? 1 : 0);
	while (keys[i] === dragged) i++;
	const before = keys[i];
	return before === undefined ? { end: true } : { before };
}
