// What the shell shows on every view: store states and how many wishes wait for review.
// Loaded once and refreshed after each navigation, so a sync or a batch shows up everywhere.
import { invoke } from '@tauri-apps/api/core';
import type { ConnectionView } from '#lib/bindings/ConnectionView.ts';
import type { ReviewItem } from '#lib/bindings/ReviewItem.ts';
import { isLang, rememberLang } from '#lib/i18n/index.ts';
import { ipc } from '#lib/ipc.ts';
import { needsReview } from '#lib/review/view.ts';

export const shell = $state<{
	connections: ConnectionView[];
	toReview: number;
	showCovers: boolean;
}>({
	connections: [],
	toReview: 0,
	showCovers: true
});

export async function refreshShell() {
	try {
		const [connections, review, settings] = await Promise.all([
			ipc.connections(),
			invoke<ReviewItem[]>('review_list'),
			ipc.settings()
		]);
		shell.connections = connections;
		shell.showCovers = settings.show_covers;
		// The database holds the language; the local copy only makes it available at start.
		if (isLang(settings.language) && rememberLang(settings.language)) location.reload();
		shell.toReview = review.filter(needsReview).length;
	} catch {
		// The views report their own load errors; the shell just keeps the last values.
	}
}

const KEY = 'crossshelf.sidebar';

/** Sidebar open or folded: a per-viewer convenience, best effort. */
export function readSidebarOpen(): boolean {
	try {
		return localStorage.getItem(KEY) === 'open';
	} catch {
		return false;
	}
}

export function saveSidebarOpen(open: boolean) {
	try {
		localStorage.setItem(KEY, open ? 'open' : 'folded');
	} catch {
		/* private storage: keep it for this session only */
	}
}
