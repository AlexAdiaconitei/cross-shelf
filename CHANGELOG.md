# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

The release notes come from this file. The GitHub release for tag `vX.Y.Z` uses
the matching `## [X.Y.Z]` section, and CI refuses a tag without one.

## [Unreleased]

### Added

- Bilingual GitHub Pages landing in `gh-pages/`, with English by default, the app's design and icons, real game artwork in simulated libraries, attribution notices and an author profile. The hero links directly to the latest Windows installer and its release notes when available. A Play log section and screenshots of its board and diary show what you are playing, with hours from Steam.
- Independent Pages deployment only for changes inside `gh-pages/` on `main`, plus local preview, screenshot generation, browser checks and an English maintenance README.
- Play log, a new view in the sidebar: note what you are playing, what is waiting, what you paused, finished or dropped, for any game of your library in any store. See it as a board, a list or a diary. Move games by dragging them, with a line that shows where each card will land, with the menu on each card or with Alt + arrow keys. Add a game by searching your library, or from its sheet. The view you chose is remembered.
- Play time from Steam: each game's hours and last session, read in the same sync with no extra request. Cards and the diary show them; the diary adds what you played this week. Play time from Epic and GOG is built but stays off until it is checked with real accounts; until then the game sheet says «Epic does not give it» and «GOG does not give it».
- Suggestions above the board, from your play time: a game you played in the last two weeks can go to Playing, and one you have not touched in a month can go to Paused. Nothing moves until you accept; a dismissed suggestion does not come back.
- A Time block in each game sheet with the total, the last session and the hours per store, and a link to search the game on HowLongToBeat. The app does not ask HowLongToBeat anything.
- A Play field in the game sheet (Library, Wishlists and Play log) with the state and a short note, and a tag with the state on Library rows.
- Settings → My data: export your plays, notes, matching decisions, «Keep», dismissed suggestions, play time history and settings to a JSON file, and import it on another computer. Importing replaces your data after saving a copy of it. Connections, sessions and cleanup batches are never exported.
- A daily copy of your own data, keeping the last seven, in the `backups` folder.

### Changed

- Your own data and what the stores send now live in two files. Updating moves the synced data to the new cache file by itself: there is no need to sync again. Deleting the synced data in Settings no longer touches anything you decided.
- Results leaves the sidebar. Reach it from Review cleanup («Earlier batches»), or after confirming a batch, and go back with «Review cleanup».

## [0.2.0] - 2026-11-18

First stable version. CrossShelf gets a new design, opens on your library and shows each game's
cover, which stores you own it on and, in the Steam family, who shares it with you. The interface
is now in English and Spanish.

### Added

- Game covers in the Library (small in each row, large in the sheet), in Wishlists and in Review cleanup. They load from each store's website; if one does not load, the game initials show instead, never a broken image. They can be hidden in Settings. They appear after syncing each store again.
- A side sheet for each game in the Library and in Wishlists, with its products and access in each store and a button to open them in the store.
- Steam family access says who shares each game with you, by their public Steam name. If their profile is private, it says how many people.
- The app is in English and Spanish. English is the default; change it in Settings → Appearance. Dates and numbers follow the chosen language.
- CrossShelf logo in the sidebar, in the window title bar and in the Windows icon, and a loading screen with the logo instead of a black window when the app opens.

### Changed

- New design across the whole app: each store has its colour and logo (Steam cyan, GOG magenta, Epic black), and yellow marks only the next step. Headings in condensed Archivo.
- The app opens on the Library. The sidebar folds into an icon rail or opens with sections, the number of wishes to review and the state of each store.
- The Library groups into one row the products with the same title in several stores, with a switch to see them per store. The Steam, GOG, Epic and Steam family tabs become store and access filters that combine.
- Wishlists looks like the Library: cover, store tabs showing where you already own each wish, and a sheet when you click it.
- Review cleanup compares each wish with what you own in two halves, with a verdict stamp, and the batch stays pinned at the bottom.
- Connections shows each count large (games, shared, wishes) with its state and how recent the data is.
- Help texts and data origin move behind an information icon; a warning (a partial or failed sync) shows as an amber icon.
- Filter bars and page headers always take one row: they adapt to the width instead of wrapping, and no view scrolls sideways. The window has a minimum size of 1024 × 640; when it is not wide, the game sheet opens over the list.
- Slim scrollbars that match the design. Right click no longer opens the browser menu (except in text fields), and text can only be selected in the game sheet and in fields.
- Exported CSV and JSON files use English column names (`store`, `product_id`, `title`…).
- The installer is in English and Spanish.

### Fixed

- Syncing Epic takes about 20 seconds instead of more than two minutes: catalog records are requested several at a time.
- The Library capitalises «Steam family», and Settings explains properly why a copy built without the signature does not update itself.

## [0.1.0] - 2026-11-18

First version. CrossShelf brings your Steam, GOG and Epic libraries together and helps you clean
your wishlists when you already own a game you wish for.

### Added

- Connections: connect Steam, GOG and Epic in each store's official window; the app never sees your password. Each store shows the real state of what the app can do with it (library, family, wishlist, removing wishes), with sync, reconnect and disconnect.
- Library of the three stores in tabs (All, Steam, Steam family, GOG and Epic), with search, filters and the date of each store's data. Steam family access is always marked as such, never as bought.
- Name, type (game, DLC, demo, soundtrack, application…) and Windows support of each product, taken from each store's records.
- Steam and GOG wishlists with search, filters, sorting by priority or date and «Open in the store». The Epic one shows as not available, never as an empty list.
- Review cleanup: each wish next to what you own that might cover it, with the reason. «It is the same», «They are different» and «Keep on this wishlist» are saved and survive new syncs. Nothing is preselected and a similar title is never enough to remove anything.
- Removing Steam and GOG wishes from the app: you pick the ones you already own, review why each is removed, and «Remove N wishes now» removes and checks each one by reading the wishlist again. Validated with real accounts.
- Results: history of each batch with the state of each entry; you can cancel what is left or retry what failed.
- Export of library and wishlists to CSV or JSON, without account IDs.
- Settings: family access in the cleanup (off by default), data folder, deleting the synced data, checking for signed updates and connection diagnostics.
- If you connect another account of the same store, the data of the previous one is kept apart and never mixed.
- Everything is stored only on your computer, with the date of each copy, and sessions are encrypted with Windows data protection.
- Installer for Windows 10 and 11 in Spanish, with the MIT licence and third-party notices.
