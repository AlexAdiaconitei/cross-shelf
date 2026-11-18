import { describe, expect, it } from 'vitest';
import { releaseLinks, repository } from './release.js';

const installer = {
	name: 'CrossShelf_2.7.0_x64-setup.exe',
	browser_download_url: `https://github.com/${repository}/releases/download/v2.7.0/CrossShelf_2.7.0_x64-setup.exe`
};
const release = { tag_name: 'v2.7.0', draft: false, prerelease: false, assets: [installer] };

describe('landing release downloads', () => {
	it('links directly to the versioned x64 EXE and notes from the same latest release', () => {
		expect(releaseLinks(release)).toEqual({
			download: installer.browser_download_url,
			notes: `https://github.com/${repository}/releases/tag/v2.7.0`,
			version: 'v2.7.0'
		});
	});
	it('ignores signatures, archives and other architectures, regardless of asset order', () => {
		expect(
			releaseLinks({
				...release,
				assets: [
					{ ...installer, name: 'CrossShelf_2.7.0_x64-setup.exe.sig' },
					{ ...installer, name: 'CrossShelf_2.7.0_arm64-setup.exe' },
					{ ...installer, name: 'CrossShelf_2.7.0_x64.zip' },
					installer
				]
			})?.download
		).toBe(installer.browser_download_url);
	});
	it.each([
		null,
		{},
		{ ...release, draft: true },
		{ ...release, prerelease: true },
		{ ...release, assets: [] },
		{ ...release, assets: {} },
		{
			...release,
			assets: [{ ...installer, browser_download_url: 'https://example.com/installer.exe' }]
		}
	])('keeps the releases-page fallback when no suitable public installer exists: %j', (value) => {
		expect(releaseLinks(value)).toBeNull();
	});
});
