<div align="center">

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/assets/cross-shelf-horizontal-on-dark.svg">
  <img src="docs/assets/cross-shelf-horizontal-color.svg" alt="CrossShelf" width="360">
</picture>

**Tus bibliotecas de Steam, GOG y Epic en un solo sitio, y tus wishlists limpias de juegos que ya tienes.**

<p>
  <a href="https://ko-fi.com/K3Q5236GOO"><img src="https://ko-fi.com/img/githubbutton_sm.svg" alt="Apóyame en Ko-fi"></a>
</p>

[![Release](https://img.shields.io/github/v/release/AlexAdiaconitei/cross-shelf?style=flat-square&color=FBD530&labelColor=100F0D)](https://github.com/AlexAdiaconitei/cross-shelf/releases/latest)
[![Windows](https://img.shields.io/badge/Windows-10%20%C2%B7%2011-100F0D?style=flat-square&logo=windows&logoColor=white)](#instalar)
[![Tauri](https://img.shields.io/badge/Tauri-2-100F0D?style=flat-square&logo=tauri&logoColor=FFC131)](https://tauri.app)
[![Licencia](https://img.shields.io/badge/licencia-MIT-100F0D?style=flat-square)](LICENSE)

[Qué hace](#qué-hace) · [Instalar](#instalar) · [Cómo se usa](#cómo-se-usa) · [Tiendas](#estado-de-cada-tienda) · [Privacidad](#privacidad) · [Desarrollo](#desarrollo)

[English](README.md) · **Español**

</div>

Compras en Steam, te regalan juegos en Epic, encuentras ofertas en GOG… y acabas con un juego en tu
wishlist de una tienda que ya tienes en otra. CrossShelf lee tus tres bibliotecas, te enseña en qué
tienda tienes cada juego y te ayuda a quitar de tus wishlists lo que ya es tuyo, siempre con la
evidencia delante y nunca sin que lo confirmes.

## Qué hace

- **Una biblioteca para las tres tiendas.** Cada juego en una fila, con su carátula y una pestaña de
  color por tienda: llena si es tuyo, con contorno si es de tu familia de Steam, punteada si está en
  tu wishlist. Búsqueda, filtros por tienda, acceso y tipo, y una ficha con los productos de cada
  tienda.
- **Wishlists de Steam y GOG**, con prioridad, fecha y dónde tienes ya cada deseado.
- **Revisar limpieza.** Cada deseado frente a lo que ya tienes y podría cubrirlo, con el motivo:
  mismo ID de tienda, catálogo común o solo el título. Tú decides «Es el mismo», «Son diferentes» o
  «Mantener». Nada se marca solo por parecerse.
- **Lotes que revisas y confirmas.** La app quita los deseados elegidos y comprueba cada uno leyendo
  la wishlist otra vez. Los lotes anteriores, desde Revisar limpieza, guardan lo que pasó con cada
  entrada.
- **Mis partidas.** Apunta qué estás jugando, qué tienes pendiente y qué has terminado, en tablero,
  lista o un diario ordenado por la última vez que jugaste. Las horas y la última sesión vienen de
  Steam; con el tiempo jugado, la app sugiere mover un juego, y nada cambia sin tu clic.
- **Tiempo en la ficha** de cada juego, por tienda, con un enlace para buscarlo en HowLongToBeat. La
  app nunca le pide nada a HowLongToBeat.
- **Acceso familiar de Steam** marcado siempre como tal, con quién te lo comparte. No cuenta como
  tuyo en la limpieza salvo que lo actives.
- **Exportación** de biblioteca y wishlists a CSV o JSON, y de **Mis datos** (partidas, notas,
  decisiones, ajustes) a un fichero que puedes importar en otro equipo.

> [!NOTE]
> CrossShelf no instala ni abre juegos, no compra y no transfiere licencias. Quitar un deseado no
> borra ninguna licencia.

## Instalar

1. Descarga el instalador `CrossShelf_x.y.z_x64-setup.exe` de la
   [última versión](https://github.com/AlexAdiaconitei/cross-shelf/releases/latest).
2. Ábrelo. Se instala solo para tu usuario y no pide permisos de administrador.
3. Si falta WebView2, el instalador lo descarga.

> [!WARNING]
> El instalador no está firmado con un certificado de editor, así que Windows SmartScreen puede
> avisar la primera vez. Pulsa **Más información** y luego **Ejecutar de todas formas**.

Requisitos: Windows 10 22H2 u 11 de 64 bits. Cuando hay una versión nueva, la app la ofrece desde
**Ajustes**.

## Cómo se usa

1. **Conexiones.** Conecta Steam, GOG y Epic. Cada tienda abre su página oficial en su propia
   ventana: inicias sesión allí y la app nunca ve tu contraseña.
2. **Sincroniza** cada tienda. Verás cuántos juegos y deseados hay y de cuándo son los datos.
3. **Biblioteca.** Busca cualquier juego para saber si lo tienes y dónde.
4. **Revisar limpieza.** El botón amarillo te lleva a los deseados que quizá ya tienes. Decide cada
   caso, marca los que quieras quitar y confirma el lote.

> [!TIP]
> La app está en inglés y en español: cambia el idioma en **Ajustes → Apariencia**. Ahí también
> puedes ocultar las carátulas, y en **Limpieza de wishlists** contar el acceso familiar.

## Estado de cada tienda

| Capacidad                | Steam        | GOG          | Epic                       |
| ------------------------ | ------------ | ------------ | -------------------------- |
| Conectar                 | Sí           | Sí           | Sí                         |
| Biblioteca               | Sí           | Sí           | Sí                         |
| Familia                  | Sí           | No aplica    | No aplica                  |
| Wishlist                 | Sí           | Sí           | No: se abre la web de Epic |
| Quitar o añadir deseados | Desde la app | Desde la app | No disponible              |
| Carátulas                | Sí           | Sí           | Sí                         |
| Tiempo jugado            | Sí           | Aún no       | Aún no                     |

«Desde la app» significa que la app quita los deseados de un lote confirmado y comprueba cada uno
leyendo la wishlist otra vez. Está validado con cuentas reales en Steam y GOG
([registros](docs/validation/)). El tiempo jugado de GOG y Epic está hecho, pero espera la misma
comprobación con cuentas reales; hasta entonces la ficha dice que la tienda no lo da.

## Privacidad

- Todo se guarda solo en tu equipo, en `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`: tus
  datos en `crossshelf.db` (con una copia diaria en `backups\`, se guardan las siete últimas) y lo
  que mandan las tiendas en `crossshelf-cache.db`, que se rehace sincronizando.
- Las sesiones se guardan cifradas con la protección de datos de Windows.
- No hay telemetría ni cuenta de CrossShelf. La app solo habla con Steam, GOG y Epic, y las
  carátulas se cargan desde la web de cada tienda.
- Los registros y las exportaciones no incluyen contraseñas, claves de sesión ni cookies.

## Desarrollo

Tauri 2 + SvelteKit estático + Svelte 5 + Rust. Requisitos: Windows 10 22H2 u 11 (x64), Node 24,
pnpm 12 y Rust estable.

```sh
pnpm install
pnpm tauri dev                          # la app completa
pnpm lint && pnpm check && pnpm test    # lo que ejecuta la CI
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

| Dónde                             | Qué                                                    |
| --------------------------------- | ------------------------------------------------------ |
| `src/lib/i18n/`                   | Textos de la interfaz, inglés y español                |
| `docs/crossshelf-build-prompt.md` | Especificación completa                                |
| `PRODUCT.md`, `DESIGN.md`         | Producto y sistema visual                              |
| `CONTEXT.md`                      | Glosario del dominio                                   |
| `docs/adr/`                       | Decisiones de arquitectura                             |
| `docs/plans/`                     | Planes de lo próximo                                   |
| `docs/validation/`                | Pruebas con cuentas reales                             |
| `src/lib/bindings/`               | Tipos TypeScript generados desde Rust con `cargo test` |

Las versiones se publican al empujar una etiqueta `vX.Y.Z`: la CI comprueba que la versión coincide
en todos los manifiestos, compila el instalador y deja un borrador de release con las notas de
`CHANGELOG.md`.

La landing pública vive en [`gh-pages/`](gh-pages/README.md), separada de la app. Se puede ver con
`node gh-pages/scripts/preview.mjs`. GitHub Pages solo se despliega al cambiar esa carpeta en `main`;
la configuración inicial y los comandos de mantenimiento están en su README en inglés.

## Sobre mí

<img src="https://github.com/AlexAdiaconitei.png?size=160" width="80" align="left" alt="Alex Adiaconitei">

**Alex Adiaconitei**

Software engineer. En el trabajo hago backend con Java y Spring Boot; en casa construyo mis propias herramientas con TypeScript, Rust o lo que pida cada problema, y mantengo un VPS y un homelab con los que aprendo infraestructura.

<br clear="left">

- Software Engineer
- TypeScript, Rust, Java, Python
- Remoto

<p>
  <a href="https://github.com/AlexAdiaconitei"><img src="https://img.shields.io/badge/GitHub-AlexAdiaconitei-181717?style=for-the-badge&logo=github&logoColor=white" alt="GitHub"></a>
  <a href="https://www.linkedin.com/in/alexadiaconitei/"><img src="https://img.shields.io/badge/LinkedIn-Alex%20Adiaconitei-0A66C2?style=for-the-badge&logo=data%3Aimage%2Fsvg%2Bxml%3Bbase64%2CPHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI%2BPHBhdGggZmlsbD0id2hpdGUiIGQ9Ik01LjM0IDcuNzJIMS4xNVYyMWg0LjE5VjcuNzJaTTMuMjUgMWEyLjQzIDIuNDMgMCAxIDAgMCA0Ljg2IDIuNDMgMi40MyAwIDAgMCAwLTQuODZaTTIxLjg1IDEzLjM5YzAtNC0yLjE0LTUuODYtNS01Ljg2YTQuMzEgNC4zMSAwIDAgMC0zLjkxIDIuMTVWNy43Mkg4Ljc1VjIxaDQuMTl2LTYuNThjMC0xLjc0LjMzLTMuNDIgMi40OC0zLjQyIDIuMTIgMCAyLjE1IDEuOTggMi4xNSAzLjUzVjIxaDQuMjh2LTcuNjFaIi8%2BPC9zdmc%2B" alt="LinkedIn"></a>
  <a href="https://ko-fi.com/K3Q5236GOO"><img src="https://img.shields.io/badge/Ko--fi-Ap%C3%B3yame-FF5E5B?style=for-the-badge&logo=kofi&logoColor=white" alt="Ko-fi"></a>
</p>
