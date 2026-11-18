# CrossShelf

Aplicación de escritorio para Windows que reúne tus bibliotecas de Steam, GOG y Epic y te ayuda a
limpiar tus wishlists cuando un juego que deseas ya lo tienes en otra tienda.

> Versión en desarrollo. Todavía no hay una versión publicada para instalar.

## Qué hace

- **Conecta tus cuentas** con la página oficial de cada tienda, en su propia ventana. La app nunca
  ve tu contraseña ni te pide que copies códigos.
- **Muestra tus bibliotecas** juntas o por tienda, con búsqueda y filtros. Incluye la biblioteca
  familiar de Steam, siempre marcada como «Acceso familiar» y nunca como comprada.
- **Muestra tus wishlists** de Steam y GOG.
- **Revisa la limpieza**: pone cada deseado junto a lo que ya tienes y podría cubrirlo, con el
  motivo. Tú decides «Es el mismo», «Son diferentes» o «Mantener en esta wishlist». Nada se marca
  solo.
- **Prepara lotes de limpieza** que revisas y confirmas. Una entrada solo cuenta como quitada cuando
  la app vuelve a leer la wishlist y ya no está.
- **Exporta** biblioteca y wishlists a CSV o JSON.

## Qué no hace

No instala ni abre juegos, no compra, no transfiere licencias y no borra nada de tus bibliotecas.
Quitar un deseado no borra ninguna licencia.

## Estado de cada tienda

| Capacidad                | Steam        | GOG          | Epic                                  |
| ------------------------ | ------------ | ------------ | ------------------------------------- |
| Conectar                 | Sí           | Sí           | Sí                                    |
| Biblioteca               | Sí           | Sí           | Sí                                    |
| Familia                  | Sí           | No aplica    | No aplica                             |
| Wishlist                 | Sí           | Sí           | No disponible: se abre la web de Epic |
| Quitar o añadir deseados | Desde la app | Desde la app | No disponible                         |

«Desde la app» significa que la app quita los deseados de un lote confirmado y comprueba cada uno
leyendo la wishlist otra vez (validado en Steam y GOG el 18 de noviembre de 2026).

## Privacidad

- Todo se guarda solo en tu equipo, en `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`.
- Las sesiones se guardan cifradas con la protección de datos de Windows.
- No hay telemetría ni cuenta de CrossShelf. La app solo habla con Steam, GOG y Epic.
- Los registros y las exportaciones no incluyen contraseñas, claves de sesión ni cookies.

## Licencia

MIT. Ver `LICENSE`. El software de terceros que incluye está en `THIRD_PARTY_NOTICES.md`.

## Para desarrollo

Requisitos: Windows 10 22H2 u 11 (x64), Node 24, pnpm 12 y Rust estable.

```sh
pnpm install
pnpm tauri dev                      # la app completa
pnpm lint && pnpm check && pnpm test
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

La especificación está en `docs/crossshelf-build-prompt.md`, las decisiones en `docs/adr/`, los
informes de cada fase en `docs/phases/` y las pruebas con cuentas reales en `docs/validation/`.
Los tipos TypeScript de los comandos se generan desde Rust con `cargo test` en `src/lib/bindings/`.
