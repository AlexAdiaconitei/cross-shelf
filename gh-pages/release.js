export const repository = 'AlexAdiacontiei/cross-shelf';
export const latestRelease = `https://github.com/${repository}/releases/latest`;

/** Only accept the current Windows x64 NSIS installer from this repository. */
export function releaseLinks(release) {
	if (
		!release ||
		release.draft ||
		release.prerelease ||
		typeof release.tag_name !== 'string' ||
		!Array.isArray(release.assets)
	)
		return null;
	const prefix = `https://github.com/${repository}/releases/download/`;
	const installer = release.assets?.find(
		(asset) =>
			typeof asset.name === 'string' &&
			/^CrossShelf_.+_x64-setup\.exe$/i.test(asset.name) &&
			typeof asset.browser_download_url === 'string' &&
			asset.browser_download_url.startsWith(prefix) &&
			asset.browser_download_url.endsWith('.exe')
	);
	if (!installer) return null;
	return {
		download: installer.browser_download_url,
		notes: `https://github.com/${repository}/releases/tag/${encodeURIComponent(release.tag_name)}`,
		version: release.tag_name
	};
}
