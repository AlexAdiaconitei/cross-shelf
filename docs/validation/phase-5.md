# Fase 5: instalación, actualización y release

Estado a 18 de noviembre de 2026: **preparada, sin ejecutar**. Necesita acciones del usuario: repo
remoto, claves del updater y una máquina Windows limpia.

## Qué hay hecho

- Instalador NSIS por usuario, en español: `pnpm tauri build` en esta máquina produce
  `CrossShelf_0.0.1_x64-setup.exe` (4,3 MiB) con `THIRD_PARTY_NOTICES.md` y `LICENSE.txt` dentro.
- `.github/workflows/release.yml`: solo Windows; con un tag `v*` pasa lint, check, tests, clippy y
  el arranque de prueba, y crea un **borrador** de release con el instalador y `latest.json`
  firmado si existen los secretos del updater.
- A18 en tests: `a18.rs` (centinelas en `Debug` de tokens y sesiones, en el registro y en los
  códigos de error), `redact.rs`, exportaciones sin IDs de cuenta (comprobado además con los
  archivos reales del usuario). Las ventanas de login no tienen ninguna capability (solo `main`) y
  Tauri 2 rechaza el IPC desde orígenes remotos.

## Pasos del usuario

1. **Repo remoto.** Crear `AlexAdiaconitei/cross-shelf` público en GitHub y dar permiso para añadir
   el remoto y hacer push de `main`.
2. **Claves del updater** (la clave privada nunca la ve el agente):
   `pnpm tauri signer generate -w ~/.tauri/cross-shelf.key`, y en el repo, Settings → Secrets →
   Actions: `TAURI_SIGNING_PRIVATE_KEY` (contenido del archivo), `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   y `TAURI_SIGNING_PUBLIC_KEY` (contenido de `cross-shelf.key.pub`).
3. **Release v0.1.0.** Con permiso, el agente ejecuta `release-prep` (CHANGELOG, versiones, commit y
   tag `v0.1.0`) y el usuario hace push del tag. CI deja un borrador con el instalador.
4. **A16, instalación limpia.** En una VM o PC Windows 10 22H2/11 x64 sin Node, Rust, Python ni
   Legendary: descargar el instalador del borrador, instalar. Esperado: se instala sin pedir
   administrador, instala WebView2 si falta, y los datos quedan en
   `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`, no en la carpeta de instalación.
5. **A01 y A02.** En esa instalación: conectar Steam (con Steam Guard o QR), GOG y Epic (con el
   login que uses: contraseña, MFA o social). Cancelar una vez y reintentar. Esperado: las tres
   conectadas desde la GUI, sin códigos manuales.
6. **A17, actualización.** El agente prepara `v0.1.1`; tras su borrador, publicar `v0.1.0` y
   `v0.1.1` (el updater lee el último publicado). Esperado: la app ofrece actualizar, conserva
   datos y sesiones. Prueba de firma alterada y de descarga cortada: se preparan entonces.

## Resultados

Sin resultados todavía.
