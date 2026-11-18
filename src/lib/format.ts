// Display helpers shared by the views. Plain functions over generated types and the copy module.
import type { CapabilityView } from '#lib/bindings/CapabilityView.ts';
import { es } from '#lib/strings.ts';

const dateTime = new Intl.DateTimeFormat('es-ES', { dateStyle: 'medium', timeStyle: 'short' });
const dateOnly = new Intl.DateTimeFormat('es-ES', { dateStyle: 'medium' });

export const when = (secs: number) => dateTime.format(new Date(secs * 1000));
export const day = (secs: number) => dateOnly.format(new Date(secs * 1000));

export const storeName = (s: string) => es.stores[s as keyof typeof es.stores] ?? s;
export const kindName = (k: string) => es.kinds[k as keyof typeof es.kinds] ?? k;

export function errorText(code: string | null): string {
	return es.errors[(code ?? 'unknown') as keyof typeof es.errors] ?? es.errors.unknown;
}

export type Tone = 'ok' | 'warn' | 'muted';

/** Plain-language state of one capability, with a tone for its marker. */
export function capabilityText(c: CapabilityView): { text: string; tone: Tone } {
	const t = es.capabilityState;
	switch (c.state) {
		case 'complete':
			return { text: t.complete(c.item_count ?? 0, when(c.last_complete_at ?? 0)), tone: 'ok' };
		case 'partial':
			return { text: t.partial, tone: 'warn' };
		case 'failed':
			return c.last_complete_at === null
				? { text: t.failedNoCopy(errorText(c.error_code)), tone: 'warn' }
				: {
						text: t.failedWithCopy(errorText(c.error_code), when(c.last_complete_at)),
						tone: 'warn'
					};
		case 'unavailable':
			return { text: t.unavailable, tone: 'muted' };
		case 'pending':
			return { text: t.pending, tone: 'muted' };
		case 'validated':
			return { text: t.validated, tone: 'ok' };
		case 'not_validated':
			return { text: t.not_validated, tone: 'muted' };
		default:
			return { text: t.not_synced, tone: 'muted' };
	}
}

/** Official pages to open when the app cannot do something itself (spec 6.4). */
export const storeWishlistUrl = {
	steam: 'https://store.steampowered.com/wishlist/',
	gog: 'https://www.gog.com/account/wishlist',
	epic: 'https://store.epicgames.com/wishlist'
} as const;

/** Page of a product in its store, or the store wishlist when the product URL is unknown. */
export function productUrl(store: string, nativeId: string, url: string | null): string {
	if (url) return url;
	if (store === 'steam' && /^\d+$/.test(nativeId))
		return `https://store.steampowered.com/app/${nativeId}`;
	return (
		storeWishlistUrl[store as keyof typeof storeWishlistUrl] ??
		'https://store.steampowered.com/wishlist/'
	);
}

export type CapabilityLine = {
	/** Short state, never a success when the read was partial or failed. */
	state: string;
	tone: Tone;
	count: number | null;
	unit: string | null;
	/** Date of the data shown, when there is any. */
	date: string | null;
};

/** One line of a store card: the count big, the state short, the date of the data shown. */
export function capabilityLine(c: CapabilityView): CapabilityLine {
	const card = es.capabilityCard;
	const unit = card.units[c.capability as keyof typeof card.units] ?? null;
	const date = c.last_complete_at === null ? null : when(c.last_complete_at);
	const count = unit === null ? null : c.item_count;
	switch (c.state) {
		case 'complete':
			return { state: card.complete, tone: 'ok', count, unit, date };
		case 'partial':
			return { state: card.partial, tone: 'warn', count, unit, date: date && card.dataFrom(date) };
		case 'failed':
			return {
				state: card.failed(errorText(c.error_code)),
				tone: 'warn',
				count: date ? count : null,
				unit,
				date: date && card.dataFrom(date)
			};
		default: {
			const t = capabilityText(c);
			return { state: t.text, tone: t.tone, count: null, unit: null, date: null };
		}
	}
}
