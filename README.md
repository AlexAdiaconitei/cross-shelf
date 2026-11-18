<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/cross-shelf-horizontal-on-dark.svg">
  <img src="docs/assets/cross-shelf-horizontal-color.svg" alt="CrossShelf" width="360">
</picture>

**Your Steam, GOG and Epic libraries in one place, and wishlists clear of games you already own.**

<p>
  <a href="https://ko-fi.com/K3Q5236GOO"><img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Support me on Ko-fi"></a>
</p>

[![Release](https://img.shields.io/github/v/release/AlexAdiaconitei/cross-shelf?style=flat-square&color=FBD530&labelColor=100F0D)](https://github.com/AlexAdiaconitei/cross-shelf/releases/latest)
[![Windows](https://img.shields.io/badge/Windows-10%20%C2%B7%2011-100F0D?style=flat-square&logo=windows&logoColor=white)](#install)
[![Tauri](https://img.shields.io/badge/Tauri-2-100F0D?style=flat-square&logo=tauri&logoColor=FFC131)](https://tauri.app)
[![License](https://img.shields.io/badge/license-MIT-100F0D?style=flat-square)](LICENSE)

[What it does](#what-it-does) · [Install](#install) · [How to use it](#how-to-use-it) · [Stores](#store-support) · [Privacy](#privacy) · [Development](#development)

**English** · [Español](README.es.md)

</div>

You buy on Steam, get free games on Epic, catch a sale on GOG… and end up with a game on one
store's wishlist that you already own on another. CrossShelf reads your three libraries, shows which
store you own each game on, and helps you remove from your wishlists what is already yours, always
with the evidence in front of you and never without your confirmation.

## What it does

- **One library for three stores.** One row per game, with its cover and a coloured tab per store:
  filled if you own it, outlined if it comes from your Steam family, dashed if it is on your
  wishlist. Search, filters by store, access and type, and a sheet with each store's products.
- **Steam and GOG wishlists**, with priority, date added and where you already own each wish.
- **Review cleanup.** Each wish next to what you own that might cover it, with the reason: same
  store ID, shared catalog or only the title. You decide «It is the same», «They are different» or
  «Keep». Nothing is marked just because it looks alike.
- **Batches you review and confirm.** The app removes the chosen wishes and checks each one by
  reading the wishlist again. Results keeps what happened to every entry.
- **Steam family access** always marked as such, with who shares it. It does not count as yours in
  the cleanup unless you turn it on.
- **Export** of library and wishlists to CSV or JSON.
- **English and Spanish**, switchable in Settings.

> [!NOTE]
> CrossShelf does not install or launch games, does not buy anything and does not transfer
> licences. Removing a wish does not delete any licence.

## Install

1. Download the `CrossShelf_x.y.z_x64-setup.exe` installer from the
   [latest release](https://github.com/AlexAdiaconitei/cross-shelf/releases/latest).
2. Run it. It installs for your user only and does not ask for administrator rights.
3. If WebView2 is missing, the installer downloads it.

> [!WARNING]
> The installer is not signed with a publisher certificate, so Windows SmartScreen may warn you
> the first time. Click **More info** and then **Run anyway**.

Requires Windows 10 22H2 or 11, 64-bit. When a new version is out, the app offers it from
**Settings**.

## How to use it

1. **Connections.** Connect Steam, GOG and Epic. Each store opens its official page in its own
   window: you sign in there and the app never sees your password.
2. **Sync** each store. You will see how many games and wishes there are and how recent the data is.
3. **Library.** Search any game to know whether you own it and where.
4. **Review cleanup.** The yellow button takes you to the wishes you may already own. Decide each
   case, tick the ones you want to remove and confirm the batch.

> [!TIP]
> Change the language in **Settings → Appearance**, where you can also hide the covers. In
> **Wishlist cleanup** you can count Steam family access as a reason to remove a wish.

## Store support

| Capability           | Steam    | GOG            | Epic                       |
| -------------------- | -------- | -------------- | -------------------------- |
| Connect              | Yes      | Yes            | Yes                        |
| Library              | Yes      | Yes            | Yes                        |
| Family               | Yes      | Not applicable | Not applicable             |
| Wishlist             | Yes      | Yes            | No: opens the Epic website |
| Remove or add wishes | From app | From app       | Not available              |
| Covers               | Yes      | Yes            | Yes                        |

«From app» means the app removes the wishes of a confirmed batch and checks each one by reading the
wishlist again. It is validated with real accounts on Steam and GOG
([records](docs/validation/), in Spanish).

## Privacy

- Everything is stored only on your computer, in `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`.
- Sessions are stored encrypted with Windows data protection.
- No telemetry and no CrossShelf account. The app only talks to Steam, GOG and Epic, and covers load
  from each store's website.
- Logs and exports never include passwords, session keys or cookies.

## Development

Tauri 2 + static SvelteKit + Svelte 5 + Rust. Requires Windows 10 22H2 or 11 (x64), Node 24, pnpm 12
and stable Rust.

```sh
pnpm install
pnpm tauri dev                          # the whole app
pnpm lint && pnpm check && pnpm test    # what CI runs
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

| Where                             | What                                                 |
| --------------------------------- | ---------------------------------------------------- |
| `src/lib/i18n/`                   | Interface text, English and Spanish                  |
| `docs/crossshelf-build-prompt.md` | Full specification                                   |
| `PRODUCT.md`, `DESIGN.md`         | Product and visual system                            |
| `CONTEXT.md`                      | Domain glossary                                      |
| `docs/adr/`                       | Architecture decisions                               |
| `docs/plans/`                     | Plans for what comes next                            |
| `docs/validation/`                | Tests with real accounts                             |
| `src/lib/bindings/`               | TypeScript types generated from Rust by `cargo test` |

The project documentation under `docs/` is written in Spanish.

Versions are published by pushing a `vX.Y.Z` tag: CI checks that the version matches in every
manifest, builds the installer and leaves a draft release with the notes from `CHANGELOG.md`.

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
