# CrossShelf

Aplicación de escritorio para Windows que reúne las bibliotecas de Steam, GOG y Epic y ayuda a limpiar wishlists cuando el juego ya está en otra tienda. Pensada para personas sin conocimientos técnicos. Tauri 2 + SvelteKit estático + Svelte 5 + Rust.
La especificación completa es `docs/crossshelf-build-prompt.md` y manda sobre cualquier otra fuente. Documentación en español. La interfaz está en inglés (por defecto) y español: todo texto visible va
en `src/lib/i18n/en.ts` y `es.ts`, con las mismas claves (lo comprueba `i18n.test.ts`).

## Commands

```sh
pnpm tauri dev          # la app completa (Vite + Rust)
pnpm check              # svelte-check / tsc
pnpm lint               # Prettier check + ESLint over the whole repository
pnpm test               # unit tests (no Docker)
pnpm build
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

Before calling any change done, run exactly what CI's verify job runs: `pnpm lint`, `pnpm check`,
`pnpm test`. `prettier --check` on the touched files is not enough.

## Architecture

```
Svelte (solo pinta) -> comandos Tauri tipados -> dominio -> adaptadores (steam, gog, epic) -> SQLite
```

- Ningún token, cookie ni código de autorización llega a Svelte, a los logs ni a las exportaciones.
  Todo log pasa por `src-tauri/src/redact.rs`.
- Los IDs de 64 bits (SteamID64, `family_groupid`) son `String` en todo el recorrido.
- Nada se quita de una wishlist sin un lote revisado y confirmado. Nunca borrado por similitud.
- Una capacidad de escritura solo se habilita con registro en `docs/validation/`. Mocks, fixtures
  e importación CSV no cuentan como integración hecha.
- Glosario del dominio en `CONTEXT.md`.

## Working on a task

1. Read the task and what it references (backlog entry, plan, ADR). The task sets the scope.
   Anything else is noted as pending, not done on the way.
2. For anything that changes contracts, migrations or protocols, present a short plan with the
   files to touch and the blocking questions before implementing.
3. Implement with tests. Done means every acceptance criterion has executed evidence. Say what
   was tested, where, and what remains untested.
4. Update the docs that the change makes stale in the same commit: README, `docs/`, `.env.example`,
   this file. User-visible changes go under `## [Unreleased]` in `CHANGELOG.md`.

## Commits

- Conventional Commits, one line, lowercase, at most 72 characters:
  `fix(auth): reject expired invitations`.
- No `Co-Authored-By`, `Claude-Session` or any other attribution line.
- One commit per logical step. Commit with a pathspec (`git ofh commit -m "..." -- <paths>`), then
  check `git show --stat HEAD`. Reformatting of untouched files goes in its own `style:` commit.
- Commit dates stay outside Monday to Friday, 08:00-19:00, so always commit through
  `git ofh commit`. The hooks refuse anything else. A date that has to be fixed moves forward to
  after 19:00, never back to before 08:00: use `git ofh fix`, not a hand-picked date.
- Never push, tag or create releases unless asked. CI creates releases from the tag.

## Never

- Read `.env*` (except `.env.example`), credentials, keys or anything under `secrets/`.
- Use npm or npx. This repository uses pnpm with a 7-day minimum release age and exact versions.
  Add dependencies with `pnpm add <name>` and let the policy pick the version.
- Relax `pnpm-workspace.yaml` (release age, trust policy, allowBuilds) without asking.
