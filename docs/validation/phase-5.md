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

## Decisión del usuario (18 de noviembre de 2026)

No se sube nada a GitHub todavía: no quedan minutos de Actions este mes. El desarrollo se cierra en
local con un instalador construido en esta máquina; el repo remoto, los secretos, el push del tag y
el CI vienen después. Con eso, A17 (actualización firmada desde un release) queda bloqueado hasta
entonces.

## Resultados

- `release-prep` hecho en local: commit `e430418` («chore(release): prepare 0.1.0») y tag anotado
  `v0.1.0`, sin push. `devkit verify-version v0.1.0`: los 3 manifiestos coinciden. Lint, check,
  tests (29 de front, 177 de Rust), build y clippy en verde.
- Instalador local `src-tauri/target/release/bundle/nsis/CrossShelf_0.1.0_x64-setup.exe`
  (4.513.385 bytes, SHA-256 `01A9D388124DA80F0B80191132B6E3834033D7EE10DEAAC22B259F8E6965B38E`),
  sin firma del updater: Ajustes dice que esta copia no se actualiza sola.
- **Instalación en el equipo del usuario** (18 nov 2026, Windows 11 Pro 26200; no es un Windows
  limpio, así que no cuenta como A16). El usuario instaló el paquete; el agente lo comprobó con UI
  Automation propia (Orca sigue bloqueado por el antivirus) y desde PowerShell:
  - Instalado por usuario en `%LOCALAPPDATA%\CrossShelf\` (`cross-shelf.exe`, `LICENSE.txt`,
    `THIRD_PARTY_NOTICES.md`, `uninstall.exe`); entrada de desinstalación en HKCU con versión 0.1.0.
    La reinstalación silenciosa (`/S`) del agente terminó con código 0 sin pedir administrador.
  - Los datos siguen en `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`, fuera de la carpeta de
    instalación. El paquete abrió las sesiones cifradas que dejó la versión de desarrollo: las tres
    tiendas salen conectadas con su cuenta.
  - Sincronizar desde el paquete: Steam (254 + 438 familia + 345 deseados), GOG (271 + 0) y Epic
    (571) terminan con «Sincronización terminada».
  - Epic tardaba unos 150 s (una petición de catálogo por namespace, en serie). Arreglado en
    `afce82c` (seis a la vez): 21 s medidos en el paquete reinstalado, con el catálogo íntegro
    (5.200.318 bytes, igual que antes).
  - Biblioteca: 1459 elementos con la fecha de cada fuente. «Abrir en la tienda» en un deseado de
    Steam abrió su ficha en el navegador. Ajustes dice que la copia no se actualiza sola.
  - Dos textos corregidos en `0aaed4b` («familia de Steam» con mayúscula; aviso de copia sin firma
    que ya no la llama «versión de desarrollo»).
- Instalador reconstruido con esos arreglos: 4.518.315 bytes, SHA-256
  `C9AD2FD97028AEA53B9870C70509D3AE103E391500895B48E2B449006177AA23`. El tag `v0.1.0` sigue en
  `e430418`: los arreglos van en `[Unreleased]`.
- A01 y A02 (conectar desde cero en el paquete, cancelar y reintentar): sin probar. Necesitan el
  login del usuario en cada tienda.
- A16 (Windows limpio) y A17 (actualización firmada): sin probar, esperan a GitHub.
