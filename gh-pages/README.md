# CrossShelf landing

The public, bilingual landing for the CrossShelf Windows app. Plain HTML, CSS and JavaScript;
no bundler, runtime dependencies, analytics or third-party fonts.

<p>
  <a href="https://ko-fi.com/K3Q5236GOO"><img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support me on Ko-fi"></a>
</p>

## Preview locally

From the repository root, with Node 24 installed:

```sh
node gh-pages/scripts/preview.mjs
```

Open [http://127.0.0.1:4173/cross-shelf/](http://127.0.0.1:4173/cross-shelf/).
The preview uses the real GitHub Pages project path so relative image, font, dictionary and
author-profile URLs can be checked before deployment. Set `PORT` to use another port. Stop
the server with Ctrl+C. Opening `index.html` directly is not supported: the language and release
requests need HTTP.

## Content and design

- English is the default, regardless of the browser language. The EN / ES buttons switch the
  whole page, image descriptions, screenshots and author bio. The choice is remembered locally.
- The design uses the app’s CrossShelf wordmark, Archivo variable typeface, square controls,
  cut-corner store tabs, Steam cyan, GOG magenta and yellow primary action.
- The hero introduces the unified game library (“All your games. One library.”); wishlist cleanup
  is presented as an additional feature.
- The screenshot selector shows the library, wishlist review, connections and the Play log board.
  A section of its own presents Play log with the diary screenshot. All screenshots
  render the actual Svelte UI with real games and official cover artwork, using simulated accounts,
  ownership and wishlists. They do not use the local
  application database, store sessions or personal library. Each image has an example-data banner.
- Epic library support is described separately from its unavailable wishlist integration. Family
  access and confirmed cleanup are explained without implying ownership or automatic removal.
- The last section uses the `author-profile` skill’s canonical photo, bilingual biography and links.
- Without JavaScript the full English page, screenshots, native FAQ and GitHub download links remain
  usable. Language and screenshot controls are hidden until JavaScript is available.

Landing copy lives under `landing` in `src/lib/i18n/en.ts` and `es.ts`, alongside the app’s
translations. Do not edit the generated JSON dictionaries by hand. After changing copy:

```sh
node gh-pages/scripts/sync.mjs
pnpm exec prettier --write gh-pages/index.html gh-pages/i18n
```

The sync command renders `scripts/index.template.html` using the app dictionaries and `about.json`.
Edit that template for markup changes; `index.html` is generated. The command updates
`gh-pages/i18n/*.json`, the English HTML fallback and metadata. Include
these generated changes in the same commit; changes only in `src/` deliberately do not deploy Pages.
Update the canonical author skill first when changing the author biography, then copy its
`about.json`, `alex.webp`, `about-card.js` and `about-card.css` here. Site-specific CSS overrides
live in `site.css`.

The local author renderer omits the redundant “Creator” label; preserve that adaptation when
refreshing the skill assets.

## Latest Windows installer

The distribution repository is **`AlexAdiaconitei/cross-shelf`**, the same account as the author
links.

On each page visit, `site.js` requests the public
[`releases/latest` API](https://docs.github.com/en/rest/releases/releases#get-the-latest-release).
It chooses `CrossShelf_*_x64-setup.exe` and puts its `browser_download_url` directly on the hero
button. The release notes link points to that same release’s tag. No hardcoded version, token or
release-triggered Pages deployment is needed.

Before the repository or a stable release exists, or if GitHub is unavailable, rate limited, or has
no matching installer, the button reads “View Windows downloads” and links to the latest-release
page. It does not claim an installer is available. Without JavaScript it uses the same fallback.
Drafts and prereleases are excluded. Tests use synthetic release metadata; no release is currently
available to verify a real installer download.

If the distribution repository changes, update `release.js`, the static links in `scripts/index.template.html`,
this README and the expected Pages URL together.

## GitHub Pages deployment

Expected URL: **https://alexadiacontiei.github.io/cross-shelf/**.

After creating the public repository, open **Settings → Pages → Build and deployment → Source**
and choose **GitHub Actions**. Push the landing and `.github/workflows/pages.yml` to `main`.
The environment’s branch rules must allow `main`.

The workflow follows GitHub’s
[custom Pages workflow](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
It runs **only on a push to `main` that changes `gh-pages/**`**. It does not run on app-only pushes,
pull requests, releases, tags, manual dispatches or changes only to the workflow itself. A README
or screenshot change inside `gh-pages/` does trigger deployment, as does a deletion there.

Only the public page, scripts, styles, dictionaries, fonts, screenshots and author files are
uploaded. This README, capture tools and test sources are excluded from the artifact. There is no
`gh-pages` Git branch and no build step. Actions are pinned by SHA, checkout does not persist
credentials, and write permissions are restricted to Pages and its deployment identity token.

Devkit’s Node CI and shared actions were reviewed. They install and build the desktop frontend,
so they are not needed for this independent static deployment. The existing app/release workflows
remain separate.

## Regenerate screenshots

The capture tools require the locally installed `design-taste-frontend` skill and Chrome or Edge.
No browser dependency is added to the project. If the skill lives elsewhere, set
`CROSSSHELF_CDP_MODULE` to its absolute `scripts/lib/cdp.mjs` path. `CHROME_PATH` can select the browser.

In one terminal:

```sh
pnpm dev --host 127.0.0.1
```

In another, from the repository root:

```sh
node gh-pages/scripts/capture.mjs
pnpm exec prettier --write gh-pages/assets
```

Set `APP_URL` if Vite uses a different address. The capture script injects an allowlisted,
read-only mock of the Tauri IPC into an isolated browser profile. Unrecognised commands fail;
the Rust backend is never started. It captures library, game detail, review, connections and Play log (board and diary) at
1440 × 900 in both languages, and extracts the app’s store logos and Tabler navigation icons.
The real titles, Steam product pages and artwork URLs are recorded in
[`scripts/games.json`](scripts/games.json). The script downloads the portrait artwork from Steam’s
public CDN and caches it in ignored `.taste-skill/covers/` for repeatable captures. Only the resulting
screenshots are published; visitors do not request game artwork from a third-party CDN. Internet
access is needed on the first capture. The script waits for every cover to decode before capturing.

## Verification

Run the repository checks from the root:

```sh
pnpm lint
pnpm check
pnpm test
pnpm build
```

With the landing preview running, also run:

```sh
node gh-pages/scripts/verify.mjs
```

This tests English and Spanish at 1440, 768, 390 and 360 pixels, image loading, horizontal overflow,
headline wrapping, screenshot selection, saved language, author assets under the project path,
keyboard FAQ operation and the no-JavaScript fallback. It mocks GitHub responses for a valid EXE,
404, rate limiting, server failure and a release without an EXE; it does not download an installer.
Screenshots are saved to ignored `.taste-skill/qa/`. Release-link unit tests also run with `pnpm test`.

The check also validates each how-it-works SVG as a standalone XML image, including exactly one
root namespace declaration. The capture tool uses `XMLSerializer` when exporting navigation icons
so repeated capture runs cannot introduce duplicate `xmlns` attributes.

The deployed URL, GitHub environment permissions and an actual release download must be checked
once the repository exists. Local checks cannot exercise those services before creation.

## Asset notices

Archivo is distributed under the SIL Open Font License; see [OFL.txt](assets/fonts/OFL.txt).
Store logos are copied from the app’s `StoreLogo.svelte` (Steam and Epic: svgl.app; GOG:
svgrepo.com, item 341859). Store names and marks belong to their respective owners; CrossShelf is
an independent project. Navigation icons are from Tabler, MIT; see [Tabler licence](assets/tabler-LICENSE.txt).
Game artwork and trademarks belong to their respective owners. Accounts, ownership, family access
and wishlists in the screenshots are simulated demonstration data. CrossShelf is not affiliated
with or endorsed by the stores, developers or publishers shown. This notice also appears beside
the hero screenshot in both languages. The notice is an attribution and does not grant rights to
redistribute the artwork separately.

## About Me

<img src="https://github.com/AlexAdiaconitei.png?size=160" width="80" align="left" alt="Alex Adiaconitei">

**Alex Adiaconitei**

Software engineer. At work I do backend with Java and Spring Boot; at home I build my own tools in TypeScript, Rust or whatever each problem needs, and I run a VPS and a homelab to learn infrastructure.

<br clear="left">

- Software Engineer
- TypeScript, Rust, Java, Python
- Remote

<p>
  <a href="https://github.com/AlexAdiaconitei"><img src="https://img.shields.io/badge/GitHub-AlexAdiaconitei-181717?style=for-the-badge&logo=github&logoColor=white" alt="GitHub"></a>
  <a href="https://www.linkedin.com/in/alexadiaconitei/"><img src="https://img.shields.io/badge/LinkedIn-Alex%20Adiaconitei-0A66C2?style=for-the-badge&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZmlsbD0id2hpdGUiIGQ9Ik01LjM0IDcuNzJIMS4xNVYyMWg0LjE5VjcuNzJaTTMuMjUgMWEyLjQzIDIuNDMgMCAxIDAgMCA0Ljg2IDIuNDMgMi40MyAwIDAgMCAwLTQuODZaTTIxLjg1IDEzLjM5YzAtNC0yLjE0LTUuODYtNS01Ljg2YTQuMzEgNC4zMSAwIDAgMC0zLjkxIDIuMTVWNy43Mkg4Ljc1VjIxaDQuMTl2LTYuNThjMC0xLjc0LjMzLTMuNDIgMi40OC0zLjQyIDIuMTIgMCAyLjE1IDEuOTggMi4xNSAzLjUzVjIxaDQuMjh2LTcuNjFaIi8%2BPC9zdmc%2B" alt="LinkedIn"></a>
  <a href="https://ko-fi.com/K3Q5236GOO"><img src="https://img.shields.io/badge/Ko--fi-Support%20me-FF5E5B?style=for-the-badge&logo=kofi&logoColor=white" alt="Ko-fi"></a>
</p>

## License

[MIT](../LICENSE).
