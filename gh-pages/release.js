export const repository = 'AlexAdiaconitei/cross-shelf';
export const latestRelease = `https://github.com/${repository}/releases/latest`;
/**
 * The latest installer under a name that never changes. The release workflow uploads a copy of
 * the NSIS installer as CrossShelf-setup.exe, and GitHub redirects this URL to the newest one, so
 * the button downloads directly even without JavaScript or when the API is unavailable.
 */
export const latestInstaller = `${latestRelease}/download/CrossShelf-setup.exe`;

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
