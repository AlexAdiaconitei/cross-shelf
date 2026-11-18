import { describe, expect, it } from 'vitest';
import type { CapabilityView } from '#lib/bindings/CapabilityView.ts';
import { tr } from '#lib/i18n/index.ts';
import { capabilityLine } from './format.ts';

const cap = (over: Partial<CapabilityView>): CapabilityView => ({
	capability: 'library',
	state: 'complete',
	error_code: null,
	last_attempt_at: 1_700_000_000,
	last_complete_at: 1_700_000_000,
	item_count: 612,
	...over
});

describe('capabilityLine', () => {
	it('shows the count with its unit when the read is complete', () => {
		const l = capabilityLine(cap({}));
		expect(l).toMatchObject({
			state: tr.capabilityCard.complete,
			tone: 'ok',
			count: 612,
			unit: tr.capabilityCard.units.library
		});
		expect(l.date).not.toBeNull();
	});

	it('never calls a partial read up to date and dates the data shown', () => {
		const l = capabilityLine(cap({ state: 'partial' }));
		expect(l.tone).toBe('warn');
		expect(l.state).toBe(tr.capabilityCard.partial);
		expect(l.date?.startsWith(tr.capabilityCard.dataFrom(''))).toBe(true);
	});

	it('hides the count of a failed read that never had a copy', () => {
		const l = capabilityLine(
			cap({ state: 'failed', error_code: 'network', last_complete_at: null })
		);
		expect(l).toMatchObject({ tone: 'warn', count: null, date: null });
		expect(l.state).toContain(tr.errors.network);
	});

	it('gives write capabilities a state and no count', () => {
		const l = capabilityLine(cap({ capability: 'wishlist_write', state: 'validated' }));
		expect(l).toMatchObject({ count: null, unit: null, tone: 'ok' });
	});
});
