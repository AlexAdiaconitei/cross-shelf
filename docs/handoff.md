# CrossShelf: traspaso a otro agente

Redactado el 17 de noviembre de 2026 sobre el commit `abfbc9e` (29 commits, árbol limpio, sin push).

**La especificación manda sobre este documento:** `docs/crossshelf-build-prompt.md`. Léela entera
antes de tocar nada. Este fichero solo dice dónde está el trabajo, qué decidió el usuario y qué falta.

## 1. Estado en una frase

El esqueleto instalable, los tres adaptadores de login, la sincronización de Steam y GOG a SQLite y la
vista de Biblioteca existen. **Casi nada está probado contra cuentas reales más allá de la fase 1**, no
hay repo remoto, no se ha instalado en un Windows limpio y las fases 3 a 5 solo tienen sus motores
puros sin conectar. La definición de terminado (sección 14) está lejos.

## 2. Fases

| Fase                       | Estado                                                                     | Qué hay                                                                                                                                                                                                                                         | Qué falta                                                                                                                                                            |
| -------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0. Esqueleto               | **Incompleta** (`docs/phases/0.md`)                                        | Tauri 2 + SvelteKit, NSIS por usuario en español, single-instance, logs con secretos ocultos, SQLite WAL con migraciones y copia previa, workflow de release solo Windows (`.github/workflows/release.yml`), binario de release arranca (smoke) | Repo remoto, claves del updater, instalar 0.0.1 en Windows limpio y actualizar a 0.0.2, firma alterada rechazada, descarga fallida, icono real, ejecutar el workflow |
| 1. Integraciones           | **Cerrada con límites** (`docs/phases/1.md`, `docs/validation/phase-1.md`) | Login, sesión tras reinicio y lectura de las tres tiendas probados por el usuario                                                                                                                                                               | Ver «Límites conocidos» de ese informe                                                                                                                               |
| 2. Bibliotecas y wishlists | **En curso, sin informe**                                                  | Capa de generaciones, sync de GOG y Steam, consultas, vista Biblioteca, escritor CSV                                                                                                                                                            | Ver sección 4                                                                                                                                                        |
| 3. Equivalencias           | Solo motor puro                                                            | `src-tauri/src/matching/` con 24 tests de la tabla 7.3                                                                                                                                                                                          | Conectarlo, persistencia, vista de revisión                                                                                                                          |
| 4. Limpieza                | Solo máquina de estados                                                    | `src-tauri/src/ops.rs` con 12 tests de fallos inyectados                                                                                                                                                                                        | Planes, confirmación, escritores reales validados                                                                                                                    |
| 5. Distribución            | No empezada                                                                |                                                                                                                                                                                                                                                 | Pulido, README, licencias, release, instalación y actualización                                                                                                      |

Verificación local actual: `cargo test` 108 verdes, `pnpm test` 11 verdes, `cargo clippy --all-targets
-- -D warnings`, `pnpm lint` y `pnpm check` limpios.

## 3. Decisiones del usuario (no se renegocian sin preguntar)

- Motor en Rust, sin Python ni sidecar (ADR 0001).
- Workflow de release **propio y solo Windows**; sin firma Authenticode en 0.0.x; repo público
  `AlexAdiaconitei/cross-shelf`; Windows 10 22H2 y 11 x64.
- Epic: biblioteca por `library-service`, **sin Legendary**.
- **Epic: se abandona la wishlist.** Recorta R09 para Epic. La GUI la muestra como «no disponible»
  con enlace a la tienda, nunca como lista vacía. Por tanto no hay escritura de wishlist en Epic y la
  pregunta de 6.4 sobre si es obligatoria queda sin objeto.
- La fase 1 se cerró con cifras sin cuadrar y renovación de Steam y Epic sin probar.
- Las pruebas autenticadas las hace el usuario con sus cuentas; el agente nunca pide contraseñas.
- Excepción retirada: el User-Agent del launcher de Epic no sirvió y ya no se usa.

## 4. Fase 2: lo que falta

Hecho: `src-tauri/src/sync.rs` (generaciones; un fallo, una lectura parcial o una vacía sin confirmar
nunca borra una biblioteca válida; cuentas separadas; migración v2 en `db.rs`), `gog/sync.rs`,
`steam/sync.rs`, `queries.rs`, comandos `gog_sync` y `steam_sync` (accesibles desde Ajustes →
Diagnóstico de conexiones), `src/routes/biblioteca/+page.svelte` con `src/lib/library/filter.ts`.

Pendiente, por orden:

1. **Pruebas del usuario sin hacer:** `gog_sync` (esperado 271 = 268 juegos + 3 películas, wishlist 1),
   `steam_sync`, abrir Biblioteca con datos reales, y las sondas `steam_diag_types` (tipo y nombre de una
   muestra de la tienda) y `epic_diag_catalog` (catálogo de una muestra). **Sus resultados no existen
   todavía.** No inventes los mapeos: pide esas salidas.
2. **Steam:** poner nombre a los 344 deseados y tipo a los 254 juegos (hoy «desconocido»). El tipo de
   los 776 de la lista familiar sale de `app_type` (1 juego, 2 aplicación, 4 herramienta, 8 demo,
   8192 banda sonora; mapeo de memoria, sin contrastar con documentación). Explicar 254 frente a 244.
3. **Epic:** mapear `library-service` (666 registros: `appName`, `namespace`, `catalogItemId`,
   `productId`, `platform[]`, `acquisitionDate`; **sin título ni tipo**) con el catálogo para sacar
   título y categoría. Explicar 666 frente a 502. Si no se puede clasificar bien, preguntar al
   usuario por Legendary (decisión suya).
4. **GOG:** el endpoint de wishlist devuelve una entrada que la web no muestra (The Witcher 3, quitada
   por el usuario). Hoy se guarda con `confirmed = true`; decidir cómo marcarla «sin confirmar».
   La biblioteca de Galaxy (284) no se usa; solo `getFilteredProducts` (271).
5. **Vista Wishlists** (`/wishlists`, hoy es un marcador). Comando `wishlist_list` ya existe y trae la
   pista `already_held_here`. Estados distintos: vacío por filtros, vacío confirmado y fallo.
6. **Vista Conexiones** (`/`, hoy marcador): cuenta, capacidades con estado real, conectar,
   reconectar, desconectar, sincronizar. Hoy todo vive en `Ajustes/Diagnóstico`, que debe quedar
   como menú de soporte.
7. **Biblioteca según 9.1:** pestañas Todas, Steam, Steam familiar, GOG, Epic; conmutador a registros
   separados; fichas agrupadas (el agrupado entre tiendas es fase 3, no inventes «juegos reconocidos»
   antes).
8. **Exportación CSV/JSON con diálogo de guardar**: `src-tauri/src/export.rs` solo tiene el escritor
   CSV (BOM, RFC 4180, neutraliza fórmulas). Faltan comandos, JSON, validación de la ruta, botones y
   el permiso `dialog:allow-save` (ya está en la capability). No exportar IDs de cuenta ni secretos.
9. **Ajustes** (9.1): acceso familiar en limpieza, exportación, datos locales (abrir carpeta, borrar
   caché), actualizaciones, diagnóstico.
10. **Sin red** (A15): comprobar con la red cortada que se ve la última copia con su fecha.
11. **Informe `docs/phases/2.md`** con evidencia ejecutada; criterios A03 a A08 y A15 contra cuentas
    reales.

## 5. Fases 3, 4 y 5

**Fase 3.** Persistir relaciones entre productos (incluidos los rechazos manuales como filas) y
«Mantener»; conectar `matching::classify` a los datos; vista «Revisar limpieza» con comparación lado a
lado, «Es el mismo», «Son diferentes», «Mantener»; nada preseleccionado por umbral; acceso familiar
excluido por defecto y opcional. [PREGUNTAR] el proveedor externo de IDs (recomendación de la
especificación: IsThereAnyDeal opcional; IGDB descartado). Sin fuzzy. Criterios A09 y A10.

**Fase 4.** Tablas de planes y lotes con snapshot sin secretos; confirmación con `<dialog>` modal
ligada a lote, revisión y cuentas; ejecución con `ops.rs`; releer antes y después; sin cola oculta.
Escritura **solo** donde exista `docs/validation/<tienda>-write.md` (fecha, versión, operaciones,
resultado, relectura). Steam: `addtowishlist` y `removefromwishlist` con `sessionid` y cookies. GOG:
`/user/wishlist/add|remove/{id}` son **GET con efecto**; nunca precargarlos. Probar con productos de
prueba que elija el usuario, uno a uno, con su permiso explícito para cada mutación. Mientras no esté
validado el botón es «Abrir en la tienda» (ver el permiso `opener` y su alcance de URLs, aún sin
configurar). Criterios A11 a A14.

**Fase 5.** Pulido con `/impeccable` y `/unslop`, README de usuario en español, avisos de licencias,
`release-prep` para v0.1.0, instalación limpia y actualización 0.1.0 a 0.1.1 con datos y sesiones
conservados. Criterios A01, A02, A16, A17, A18. [PREGUNTAR] licencia antes del primer release.

## 6. Transversal, todavía sin hacer

- **Tipos TS generados desde Rust** (`ts-rs` o `tauri-specta`, pasando la política de 7 días). Hoy los
  tipos del front están escritos a mano, lo que la sección 4.3 prohíbe.
- **Feature de Cargo `fixtures`** y franja «Datos de ejemplo»; CI que falle si un release la incluye.
- **Prueba A18 con secretos centinela** sobre logs, errores y exportaciones (hoy solo `redact.rs`).
- Auditar capabilities con la skill `tauri-security` al cerrar cada fase; comprobar que ninguna ventana
  remota tiene capability.
- Guardar las respuestas crudas de cada sincronización (sin secretos, con rotación).
- Pantalla de diagnóstico con logs redactados y «Abrir carpeta de datos».
- Nombre definitivo y licencia [PREGUNTAR].
- Revisar `/hallmark` (Floor) en la UI y `/modern-web-guidance` por patrón.

## 7. Reglas y trampas ya pagadas

- Commits: `git ofh commit -m "..."`, una línea, minúsculas, **máximo 72 caracteres** (el hook rechaza
  más largos), sin atribución, sin push, con pathspec si no se quiere arrastrar todo.
- Dependencias con `pnpm add` (política de 7 días, no relajar `pnpm-workspace.yaml`). `cargo add`
  para Rust. `reqwest` 0.13 necesita las features `query` y `form` explícitas.
- Imports del front con extensión y alias: `#lib/strings.ts`, `#lib/components/ui/index.ts`. `$lib`
  no existe en este stack.
- `cargo fmt` reescribe líneas: si un parche por texto falla con «no encontrado», relee el fichero.
- **Normalización NFKD:** convierte `™` en «tm». Quita `™ ® © ℠` **antes** de normalizar. Ya falló en
  Rust y en TypeScript.
- En el shell, evita heredocs largos con comillas mezcladas: escribe con la herramienta Write y
  parchea con un script de Python en el directorio temporal.
- IDs de 64 bits siempre como `String`. Nunca leer `.env` ni ficheros de credenciales.
- Los diagnósticos solo muestran recuentos, estados y **nombres** de campos; los valores solo salen de
  campos con 8 valores distintos o menos en listas de 30 o más (`shape.rs`).
- Un mock, un fixture, un botón desactivado o una importación CSV **no** cuentan como integración.
- Cada integración que se dé por buena necesita evidencia ejecutada; apúntala en
  `docs/validation/`.
- Los Stop hooks de esta sesión repitieron «condición no cumplida» durante horas mientras se
  esperaba al usuario. Si la espera es real, di qué necesitas y para.

## 8. Cómo arrancar

```sh
pnpm install
pnpm tauri dev                      # app completa
pnpm lint && pnpm check && pnpm test
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```

Datos en `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\` (`crossshelf.db`, `secrets\*.bin` cifrados
con DPAPI, `webview\<tienda>\`). Diagnóstico: Ajustes → Diagnóstico de conexiones.

## 9. Mapa de ficheros

| Ruta                                                | Qué es                                                                                                |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `src-tauri/src/lib.rs`                              | Arranque, plugins, comandos de lectura (`library_list`, `wishlist_list`, `sync_status_list`)          |
| `src-tauri/src/db.rs`, `sync.rs`, `queries.rs`      | Esquema y migraciones, generaciones, lecturas                                                         |
| `src-tauri/src/secrets.rs`, `redact.rs`, `shape.rs` | DPAPI, ocultación de secretos, formas sin valores                                                     |
| `src-tauri/src/{gog,steam,epic}/`                   | `auth`/`api`, `library`, `sync`, `commands` por tienda                                                |
| `src-tauri/src/matching/`, `ops.rs`, `export.rs`    | Motores puros todavía sin conectar (equivalencias, operaciones, CSV)                                  |
| `src/routes/`                                       | `biblioteca` hecha; `wishlists`, `limpieza`, `resultados` y `/` son marcadores; `ajustes/diagnostico` |
| `src/lib/library/`                                  | Filtro, contadores y rango visible, con tests                                                         |
| `docs/`                                             | Especificación, ADR, informes de fase, validación                                                     |

## 10. Prompt para `/goal`

```text
Continúa la construcción de CrossShelf en D:\PROGRAMACION\VSCode-Workspace\cross-shelf. La
especificación completa y que manda sobre cualquier otra fuente es docs/crossshelf-build-prompt.md:
léela entera. Después lee docs/handoff.md (estado, decisiones del usuario, lo que falta y las trampas
ya pagadas), docs/phases/ y docs/validation/phase-1.md. No repitas trabajo hecho ni renegocies las
decisiones del usuario que constan en el handoff.

Estado: fase 0 incompleta, fase 1 cerrada con límites, fase 2 en curso, motores puros de las fases 3
y 4 sin conectar, fase 5 sin empezar. Continúa por la fase 2 según la sección 4 del handoff y avanza
por las fases 3, 4 y 5 sin saltarte ninguna. Al acabar cada fase deja su informe en docs/phases/ con
evidencia ejecutada (qué se probó, dónde, y qué no).

Reglas: para y pregunta en cada punto [PREGUNTAR] de la especificación y siempre que una prueba de
integración falle. Las pruebas autenticadas las hace el usuario con sus cuentas: prepáralas, dile qué
hacer y qué verás, no le pidas contraseñas y no inventes respuestas que no hayas visto. No des por
hecha ninguna integración [PENDIENTE DE PRUEBA] sin evidencia real y no la sustituyas por mocks,
fixtures ni importación CSV. Ninguna capacidad de escritura se habilita sin su registro en
docs/validation/<tienda>-write.md y sin permiso explícito del usuario para cada mutación de prueba.
Commits con `git ofh commit` (una línea, 72 caracteres como máximo, sin atribución, sin push). Antes
de dar algo por hecho ejecuta pnpm lint, pnpm check, pnpm test, cargo clippy --all-targets -- -D
warnings y cargo test. Para cualquier .svelte usa la skill svelte-code-writer y el autofixer del MCP
de Svelte.

Termina cuando se cumpla la sección 14 de la especificación, o cuando un bloqueo necesite una acción
o una decisión del usuario que no puedas sustituir: en ese caso para, di exactamente qué necesitas y
no repitas el aviso mientras esperas.
```
