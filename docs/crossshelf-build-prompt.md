# CrossShelf: prompt de construcción

> Nombre provisional. Prompt autosuficiente para un agente que construye la aplicación desde cero en
> `D:\PROGRAMACION\VSCode-Workspace\cross-shelf`. Fecha de redacción: 17 de noviembre de 2026.

Cada afirmación lleva una de estas marcas:

- **[CONFIRMADO]**: requisito expresado por el usuario. No se negocia sin preguntarle.
- **[DECISIÓN]**: elección técnica tomada al redactar este prompt. Puedes cambiarla solo si una
  prueba la refuta; en ese caso documenta la evidencia y pregunta antes de cambiar de rumbo.
- **[PENDIENTE DE PRUEBA]**: capacidad con precedentes en código de terceros pero sin prueba
  autenticada en esta app. No la presentes como hecha hasta tener la evidencia descrita.
- **[PREGUNTAR]**: decisión que corresponde al usuario. Pregunta cuando llegue la fase que la
  necesita, con opciones y una recomendación; no antes ni después.

---

## 0. Cómo usar este prompt

1. Lee este documento entero antes de tocar nada.
2. Lee también, como referencia de mecanismos (no como requisitos nuevos), los informes en
   `C:/Users/Alex/Downloads/games-library/game-list-extractor/analysis/`. Prioridad cuando
   discrepen: este prompt > `informe-producto-limpieza-wishlists.md` > informes técnicos
   específicos (`auth-*.md`, `steam-families-research.md`, `wishlist-write-*.md`,
   `viabilidad-svelte-python-windows.md`) > históricos (`viabilidad-escritorio.md`,
   `viabilidad-wishlists-web.md`, `wasm-pages-research.md`). Los históricos recomiendan pywebview y
   Pages; esa recomendación está sustituida.
3. `analisis-wishlist-steam.md`, `comparison.json`, `steam-details.json` y los CSV de `results/`
   contienen datos personales de una cuenta real. **No los copies al repositorio, a fixtures, a
   tests ni a la documentación.** Úsalos solo para entender qué casos de comparación existen; los
   criterios ya están extraídos en la sección 7.
4. Trabaja por fases (sección 10). Cada fase termina con evidencia ejecutada y un informe corto de
   qué se probó, dónde y qué queda sin probar.

---

## 1. Objetivo y público

**[CONFIRMADO]** Aplicación de escritorio para Windows que reúne las bibliotecas de Steam, GOG y
Epic, muestra la biblioteca familiar de Steam y ayuda a limpiar las wishlists cuando el usuario ya
tiene ese juego en otra tienda.

El centro del producto es una decisión explicable:

> «Este deseado parece cubierto por este juego que ya tienes aquí. Comprueba edición, plataforma y
> tipo de acceso antes de quitarlo.»

Unificar listados es el medio. El objetivo es evitar compras repetidas y revisar wishlists con
criterio.

**Público [CONFIRMADO]:** personas sin conocimientos de programación. El recorrido habitual se hace
entero desde la GUI:

- sin terminal, `.env`, Python, Node ni Legendary instalados a mano;
- sin API keys personales ni SteamID escrito a mano;
- sin copiar URLs, códigos de autorización, cookies ni JSON.

Los códigos MFA o Steam Guard que pida el proveedor dentro de su propia página son interacción
normal y sí están permitidos.

**Fuera de alcance [CONFIRMADO por el informe]:** instalar o lanzar juegos, descargas, partidas
guardadas, compras, transferir licencias, gestionar miembros de la familia, borrar juegos de una
biblioteca. Quitar de la wishlist no elimina ninguna licencia. Los endpoints de biblioteca y
familia son de solo lectura.

---

## 2. Requisitos confirmados

| ID  | Requisito                                                                                   |
| --- | ------------------------------------------------------------------------------------------- |
| R01 | Windows como primera plataforma.                                                            |
| R02 | Interfaz Svelte + TypeScript con un motor local. (Ver 4.2: el motor pasa de Python a Rust.) |
| R03 | Instalador para usuarios finales y actualizaciones desde GitHub Releases.                   |
| R04 | Todo el recorrido habitual desde la GUI.                                                    |
| R05 | Biblioteca propia de Steam, GOG y Epic.                                                     |
| R06 | Cada biblioteca por separado y una vista combinada.                                         |
| R07 | Búsqueda y filtros.                                                                         |
| R08 | Biblioteca familiar de Steam.                                                               |
| R09 | Wishlists de las tres plataformas.                                                          |
| R10 | Detectar deseados que ya estén en una biblioteca de otra plataforma.                        |
| R11 | Añadir y quitar deseados desde la GUI donde sea técnicamente viable y esté validado.        |

---

## 3. Skills y herramientas del agente

Antes de cada tipo de trabajo, carga la skill que le corresponde. Si una skill y este prompt chocan,
manda este prompt.

### 3.1 Devkit (`devkit`, `D:\PROGRAMACION\devkit`)

- `project-bootstrap`: arranque del proyecto con `devkit new desktop-tauri`. Sigue su guion, pero
  el grill ya está resuelto en la sección 4.1; no vuelvas a preguntar lo que aquí está decidido.
- Perfil de skills del stack: `devkit skills desktop-tauri` (instala `base`, `svelte`, `ui`,
  `rust`). Añade `devkit skills domain` (`domain-modeling`, `codebase-design`, `tdd`, `prototype`,
  `to-spec`, `to-tickets`).
- `release-prep`: para cortar cada versión. Nunca `gh release create` a mano.
- Normas de devkit que aplican: `rules/agent-workflow.md`, `rules/commits.md` (`git ofh commit`,
  Conventional Commits de una línea, sin atribución, sin push salvo que se pida),
  `rules/releases.md`, `rules/security.md`, `rules/dependencies.md` (política de 7 días; nunca la
  relajes para que pase una instalación).

### 3.2 Skills pedidas expresamente

| Skill                  | Cuándo                                                                                                                                                                                                                                                                          |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/find-skills`         | Al entrar en un dominio sin skill instalada (WebView2 COM, protobuf en Rust, rusqlite, DPAPI, NSIS). Busca antes de improvisar. Instala solo tras leer el `SKILL.md` y sus scripts, a nivel de proyecto, y anota el origen en `skills-lock.json`.                               |
| `/modern-web-guidance` | Antes de implementar cada patrón de UI o JS de cliente: diálogos de confirmación, tablas grandes y virtualización, formularios, filtros, foco y accesibilidad, `content-visibility`, fechas. Una búsqueda por patrón.                                                           |
| `/impeccable`          | Skill principal de diseño de la app. Es una superficie de modo _Operate_: precisión y contención antes que espectáculo. Úsala para dirección, `PRODUCT.md`/`DESIGN.md`, estados vacíos y de error, crítica y pulido final. Lee su `craft-floor.md` antes de cada edición de UI. |
| `/hallmark`            | Aplica su _Floor_ (tokens en `:root`, sin `transition: all`, `:focus-visible` inmediato, `overflow-x: clip`) a toda la UI. Úsala completa solo si se hace una landing o página pública del proyecto.                                                                            |
| `/unslop`              | Todo texto que lea una persona: copy de la GUI, mensajes de error, README, CHANGELOG, notas de release, informes de fase. No es para código.                                                                                                                                    |

Orden en trabajo de UI: `/impeccable` (dirección) → `/modern-web-guidance` (patrón) → implementar →
`svelte-autofixer` del MCP de Svelte → `/impeccable` (crítica) → `/unslop` sobre el copy. Para
cualquier `.svelte` o `.svelte.ts` usa la skill `svelte-code-writer` y el MCP oficial de Svelte.

### 3.3 Skills de Tauri (instaladas en `.claude/skills/`, origen `full-stack-skills/tauri-skills`)

Son guías cortas que apuntan a la documentación oficial. La autoridad es
`https://v2.tauri.app/llms.txt` y las páginas que enlazan; verifica cada API contra la versión de
Tauri instalada.

| Skill                                                                        | Para qué en CrossShelf                                                                              |
| ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `tauri`, `tauri-concept`                                                     | Modelo de procesos, ciclo de vida, comandos y eventos.                                              |
| `tauri-ipc`                                                                  | Comandos tipados, `Channel` para progreso, tipos TS generados desde Rust.                           |
| `tauri-config`                                                               | `tauri.conf.json`, CSP estricta, `identifier` definitivo, bundle NSIS.                              |
| `tauri-security`, `tauri-framework-security`, `tauri-app-plugin-permissions` | Capabilities por ventana, ninguna capability para ventanas remotas de login, auditoría de permisos. |
| `tauri-window`, `tauri-app-window-state`                                     | Ventanas de login separadas, `data_directory` por proveedor, estado de la ventana principal.        |
| `tauri-app-single-instance`                                                  | Una sola instancia: evita dos procesos escribiendo la base de datos o mutando wishlists a la vez.   |
| `tauri-app-updater`, `tauri-app-process`                                     | Updater firmado desde Releases, reinicio controlado.                                                |
| `tauri-build`                                                                | Bundle NSIS por usuario, WebView2, firma.                                                           |
| `tauri-app-dialog`, `tauri-app-opener`                                       | Selector de destino de exportación; «Abrir en la tienda».                                           |
| `tauri-app-logging`                                                          | Logs con redacción de secretos.                                                                     |
| `tauri-app-shell`                                                            | Solo si se activa el plan B de Epic (sidecar). Por defecto no se usa.                               |

---

## 4. Arquitectura

### 4.1 Stack y arranque [DECISIÓN]

- Stack `desktop-tauri` de devkit: Tauri 2 + SvelteKit estático (`ssr = false`, adapter-static) +
  Svelte 5 con runas + TypeScript estricto + Tailwind 4 + Ark UI + Tabler + Geist. Es la decisión
  D7 de devkit y el precedente de Quick Media Organizer y GolanPlayer.
- Nombre de repo `cross-shelf`, nombre visible «CrossShelf», identificador
  `com.alexadiaconitei.cross-shelf`. **El identificador no puede cambiar después sin perder datos
  y la cadena de updates.** [PREGUNTAR] si el nombre provisional pasa a definitivo antes del primer
  release público; el identificador se fija ya.
- UI y documentación en español. Cadenas de UI centralizadas en un módulo para poder traducir más
  adelante; sin librería de i18n en v1.
- Arranque: `devkit new desktop-tauri <dir> --name "CrossShelf"`. La carpeta destino ya contiene
  `.agents/`, `.claude/`, `skills-lock.json` y `docs/`. Si el generador rechaza la carpeta no vacía,
  genera en una carpeta temporal y mueve el resultado, conservando esos cuatro elementos. Después
  sigue «Después de generar» del README del stack (plugins `updater`, `process`, `opener`,
  `dialog`, `single-instance`, `window-state`, `log`; CSP; icono; claves del updater).
- Proyecto en `projects.json` de devkit al terminar el arranque, como dice `project-bootstrap`.

### 4.2 Motor: Rust, no Python [DECISIÓN, sustituye la parte Python de R02]

El usuario pidió evaluar Svelte + Python. La evaluación concluye que el motor debe ser Rust dentro
del propio proceso Tauri. Razones, por peso:

1. **Las sesiones viven en WebView2, y WebView2 es del proceso Rust.** Las cookies HttpOnly de
   Steam y Epic solo se leen con las APIs nativas del webview (`cookies_for_url`, COM de WebView2).
   La tienda de Epic devuelve 403 a HTTP directo y su wishlist probablemente solo se pueda leer
   desde una página de la tienda con sesión. Con un motor Python habría que pasar cookies y tokens
   por un canal entre procesos y aun así algunas lecturas tendrían que ocurrir en el webview. Con
   Rust, sesión, red y dominio están en el mismo proceso y los secretos no cruzan ninguna frontera.
2. **Un solo binario.** Sin PyInstaller no hay segundo ejecutable que firmar, que cerrar antes de
   actualizar, que arranque lento al descomprimirse ni que dispare falsos positivos del antivirus;
   esto último pesa mucho con un público no técnico y un editor sin reputación en SmartScreen.
3. **Encaja con devkit:** la regla del stack es «la lógica de dominio está en Rust» y los sidecars
   son para binarios externos. SQLite embebido con `rusqlite` es la elección de devkit para
   escritorio.
4. **Lo reutilizable del repo Python es conocimiento, no código.** Son ~314 líneas de CLI
   (`input`, `print`, `sys.exit`, rutas relativas) que se reescriben enteras de todos modos. Los
   endpoints, el `client_id` público de Galaxy y los formatos se portan a Rust.
5. **Legendary es GPLv3.** Empaquetarlo o importarlo obliga a planificar licencia y fuentes, y su
   `user.json` guarda tokens en claro fuera de nuestro almacén protegido. Reimplementar en Rust el
   intercambio OAuth de Epic y la lectura de activos y catálogo (protocolo documentado por el
   código de Legendary, que se lee para entenderlo y no se copia) elimina ambos problemas.

Alternativas descartadas:

- **TypeScript en el front como motor:** los secretos acabarían en el contexto de la UI y CORS
  bloquea las tiendas. No.
- **Sidecar Node:** mismos costes que Python sin la ventaja de Legendary. No.

**Plan B acotado [PENDIENTE DE PRUEBA]:** si en la fase 1 la biblioteca Epic en Rust no alcanza la
cobertura de Legendary (DLC, namespaces, plataformas) y el arreglo no cabe en la fase, se empaqueta
Legendary como sidecar solo para la biblioteca Epic, con `tauri-app-shell` en lista blanca
estricta, su licencia GPLv3 y fuentes en el instalador, y una nota explícita de que su `user.json`
no está en el almacén protegido. Es decisión del usuario: [PREGUNTAR] con la evidencia.

### 4.3 Capas y responsabilidades

```text
Svelte (UI)          solo pinta, filtra en memoria lo ya cargado y pide operaciones
   │  comandos Tauri tipados + Channel de eventos (progreso, resultados por entrada)
Rust: capa app       comandos #[tauri::command] finos, validación de entrada, cancelación
Rust: dominio        catálogo, identidad, cobertura, planes de limpieza, reconciliación
Rust: adaptadores    steam, steam_family, gog, epic_library, epic_store  (trait por capacidad)
Rust: sesión         ventanas de login, perfiles WebView2, cookies, tokens, almacén protegido
Rust: persistencia   SQLite (rusqlite, WAL, migraciones), exportación CSV/JSON
```

Reglas:

- Ningún token, cookie ni código de autorización llega a Svelte, a los eventos, a los logs ni a las
  exportaciones. La UI recibe identidad de cuenta (nombre, avatar, ID opaco) y estado de
  capacidades.
- Los adaptadores exponen operaciones por capacidad (`read_library`, `read_family`,
  `read_wishlist`, `set_wishlist_membership`). Endpoints, cookies y formatos no salen del adaptador.
- Tipos TS generados desde Rust (`ts-rs` o `tauri-specta`, el que esté estable y pase la política
  de 7 días). No mantener tipos duplicados a mano.
- **IDs de 64 bits como `String` en todo el recorrido** (SteamID64, `family_groupid`,
  `owner_steamids`): `#[serde(with = ...)]` a string en Rust, `string` en TS, `TEXT` en SQLite.
  Test que lo verifique con un valor mayor que `Number.MAX_SAFE_INTEGER`.
- Operaciones largas: devuelven un `operation_id`, emiten progreso por `Channel` y se cancelan con
  un token de cancelación. Cancelar detiene lo no enviado; no revierte lo aplicado.
- HTTP con `reqwest` (rustls). User-Agent honesto con nombre y versión de la app. Concurrencia
  limitada por adaptador; respetar `Retry-After`; backoff exponencial acotado. Sin cuotas
  inventadas.

### 4.4 Persistencia [DECISIÓN]

- Base de datos SQLite en `app_local_data_dir` (`%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\`),
  fuera de la carpeta que sustituye el instalador. WAL. Migraciones versionadas y copia de la base
  antes de cada migración. Sin promesa de rollback de datos al bajar de versión.
- Perfiles WebView2 por proveedor en `app_local_data_dir/webview/<proveedor>/`.
- Secretos fuera de SQLite: cifrados con DPAPI (`CryptProtectData`, ámbito usuario) en ficheros por
  conexión, o en el Administrador de credenciales de Windows si caben (tiene límite de tamaño por
  credencial; las respuestas de token de GOG y Epic pueden no caber). Escritura atómica (fichero
  temporal + rename) y renovaciones serializadas por conexión.
- Preferencias de UI triviales (filtros recordados, ancho de columnas) pueden ir en el store de
  Tauri; nada de datos de cuenta en `localStorage`.
- Exportación CSV (UTF-8 con BOM, RFC 4180, como el extractor actual) y JSON, siempre a un destino
  elegido con el diálogo de archivo.

### 4.5 Modelo de datos

Concreta este modelo con `domain-modeling` y deja el glosario en `CONTEXT.md` antes de escribir
adaptadores.

| Entidad                      | Debe preservar                                                                                                                                                                                                                    |
| ---------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Conexión                     | Proveedor, ID de cuenta, nombre, avatar, estado por capacidad, referencia al secreto (nunca el secreto), última sincronización.                                                                                                   |
| Producto de tienda           | ID nativo compuesto (Steam `appid`; GOG `productId`; Epic `namespace` + `catalogItemId` + `offerId` cuando se conozcan), título, tipo, edición, plataformas conocidas o «desconocida», URL, procedencia de cada dato.             |
| Juego reconocido             | Identidad local estable que agrupa productos. No sustituye al producto nativo en ninguna operación remota.                                                                                                                        |
| Derecho de acceso            | Cuenta beneficiaria, producto, clase (`owned`, `family_shared`, `temporary`, `unknown`), propietarios si se conocen, motivo de exclusión (también valores desconocidos), fecha de observación.                                    |
| Entrada de wishlist          | Cuenta, tienda, producto u oferta, prioridad y fecha si el proveedor las da, último estado observado.                                                                                                                             |
| Relación entre productos     | Clase (`same`, `edition_of`, `contains`, `remaster_of`, `sequel_of`, `dlc_of`, `different`), evidencia, origen (`store_id`, `external_map`, `manual`, `heuristic`), fecha. Un rechazo manual es una fila, no la ausencia de fila. |
| Preferencia «Mantener»       | Entrada de wishlist concreta y motivo opcional. Independiente de la relación.                                                                                                                                                     |
| Generación de sincronización | Recurso, cuenta, estado (`complete`, `partial`, `failed`), páginas completadas y fallidas, recuento, timestamps.                                                                                                                  |
| Lote de acciones             | Plan revisado, revisión confirmada, cuenta, snapshot previo sin secretos, resultado por operación.                                                                                                                                |

Guarda las respuestas crudas de cada sincronización (sin secretos, con rotación) para diagnosticar
cambios de esquema.

---

## 5. Autenticación y sesiones

### 5.1 Contrato de experiencia [CONFIRMADO]

**Conectar plataforma → página real del proveedor en una ventana de la app → verificación habitual
→ cuenta identificada con nombre y avatar → sincronizar.**

La app nunca muestra un formulario propio de contraseña. Si una variante de login no funciona en
WebView2, se explica en la GUI y se diseña una alternativa con retorno real. Pegar códigos, JSON o
cookies no cumple el recorrido; como mucho existe como herramienta de soporte oculta en un menú
de diagnóstico, nunca como camino principal.

### 5.2 Ventana de login [DECISIÓN]

- `WebviewWindow` independiente por intento de conexión, con `data_directory` propio del
  proveedor. **Ninguna capability** apunta a estas ventanas ni a orígenes remotos: la página del
  proveedor no puede invocar comandos de la app.
- `on_navigation` valida esquema, host y ruta exactos del retorno (no expresiones regulares sobre
  la URL completa) y lo asocia al `attempt_id` vigente; cada intento se completa una sola vez. Usa
  `state` donde el flujo lo admita.
- Cookies con las APIs nativas del webview (`cookies_for_url` de Tauri o la interfaz COM de
  WebView2 vía `with_webview`). JavaScript de la página no las lee.
- Ventanas emergentes (login social de Epic, captchas): manejarlas dentro de la misma ventana o
  en ventanas hijas con el mismo perfil. [PENDIENTE DE PRUEBA] en WebView2.
- Cancelar o cerrar la ventana deja el intento en `cancelled` sin bloquear otras conexiones.

### 5.3 Por proveedor

| Proveedor | Flujo                                                                                                                                                                                                                                                                                                                                                                                                                                                    | Estado                                     |
| --------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| Steam     | Ventana en la tienda oficial; tras el login, obtener `steamid` y `webapi_token` de la sesión (Playnite lo hace en la tienda; Augmented Steam usa `/pointssummary/ajaxgetasyncconfig`). El token caduca: se reobtiene de las cookies mientras la sesión valga; si no, «Volver a conectar». OpenID solo identifica y no sirve aquí.                                                                                                                        | [PENDIENTE DE PRUEBA]                      |
| GOG       | `auth.gog.com/auth` con el `client_id` público de Galaxy y `redirect_uri=https://embed.gog.com/on_login_success?origin=client`; capturar `code` en la navegación; intercambiar en `auth.gog.com/token`; guardar la respuesta completa (access, refresh, `expires_in`, hora de login); renovar con `refresh_token` y guardar la respuesta nueva de forma atómica.                                                                                         | [PENDIENTE DE PRUEBA], el mejor respaldado |
| Epic      | `https://www.epicgames.com/id/login?responseType=code`; capturar el `code` de la navegación de retorno (Heroic lo detecta aunque la carga a `localhost` falle); intercambio OAuth con las credenciales públicas del launcher que usa Legendary; renovación con `refresh_token` y `expires_at` del servidor. La sesión web de la tienda, necesaria para la wishlist, es otra capacidad con su propio estado, aunque se intente aprovechar el mismo login. | [PENDIENTE DE PRUEBA]                      |

No importes la sesión del Epic Games Launcher ni cookies del navegador habitual del usuario.

### 5.4 Estados que entiende la GUI

Sin conectar · Esperando login · Conectado (con capacidades verificadas, no un booleano) ·
Sincronizando · Biblioteca lista y wishlist pendiente · Sesión caducada (intentar renovar, luego
«Volver a conectar») · Lista privada o permiso insuficiente (nunca se muestra como lista vacía) ·
Sin red o proveedor caído (última copia con su fecha) · Cambio de cuenta.

### 5.5 Desconectar y cambiar de cuenta

- «Desconectar en este equipo» borra secretos y el perfil WebView2 del proveedor. No afirma
  revocación remota. Enlaza a la gestión de dispositivos del proveedor cuando exista.
- Una cuenta por tienda en v1. [DECISIÓN] Multi-cuenta queda fuera.
- Antes de cada sincronización y de cada lote de cambios se comprueba el ID de cuenta. Si cambió,
  los datos de la cuenta anterior se separan (no se mezclan ni se borran en silencio) y los planes
  pendientes quedan invalidados.

---

## 6. Bibliotecas, familia y wishlists

### 6.1 Bibliotecas propias

| Tienda | Vía                                                                                                                                                                                                                                                                            | Estado                |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------- |
| Steam  | `IPlayerService/GetOwnedGames/v1/` con `access_token` (no `key`), `include_appinfo`, `include_played_free_games`. Playnite combina con `IClientCommService/GetClientAppList` y `dynamicstore/userdata/` porque una sola fuente no cubre todo: mide la diferencia en la fase 1. | [PENDIENTE DE PRUEBA] |
| GOG    | `galaxy-library.gog.com/users/{user_id}/releases` con Bearer, paginado por `next_page_token` (Heroic). Alternativa: `embed.gog.com/account/getFilteredProducts` (la del extractor actual). Las entradas de otras plataformas integradas en Galaxy no son compras GOG.          | [PENDIENTE DE PRUEBA] |
| Epic   | Activos del launcher + catálogo para metadatos (protocolo de Legendary). Guardar `namespace`, `catalogItemId`, `app_name`, plataformas. Lo que no sea solo Android no implica soporte Windows: plataforma «desconocida» si faltan datos.                                       | [PENDIENTE DE PRUEBA] |

Reglas comunes: licencias detectadas, no solo instaladas. Separar permanente, temporal (fin de
semana gratuito, prueba, suscripción) y desconocido. Deduplicar por ID nativo. Nunca reemplazar
una biblioteca válida por una vacía si falla auth, una página o el parsing.

### 6.2 Steam Families [CONFIRMADO lectura; mecanismo PENDIENTE DE PRUEBA]

1. `IFamilyGroupsService/GetFamilyGroupForUser/v1/` con el token y el SteamID del usuario.
2. `is_not_member_of_any_group` es un estado normal («No perteneces a ninguna familia»), no un
   error. Una invitación pendiente no da acceso.
3. `IFamilyGroupsService/GetSharedLibraryApps/v1/` con `family_groupid`, `steamid`, `language` y
   los booleanos `include_own`, `include_excluded`, `include_non_games` enviados de forma
   explícita. `max_apps` alto; detectar truncamiento. No existe cursor: no inventes `page`.
4. Conservar por app `owner_steamids[]` (strings), `exclude_reason` (también valores
   desconocidos), `app_type` y fechas.
5. Unificar por `appid` conservando varios derechos: un juego puede ser propio y además compartido.
   No sumar bibliotecas públicas de familiares ni pedir sus cuentas.

Presentación: «Acceso familiar», con propietarios y motivo de exclusión cuando existan. Nunca «Lo
tienes comprado» ni «Puedes jugar ahora»: la lista no garantiza copia libre ni restricciones
actuales. Disponibilidad en vivo queda fuera de v1.

**Regla de limpieza [DECISIÓN]:** el acceso familiar está excluido por defecto del criterio «ya lo
tienes». Una opción en Ajustes y en la pantalla de revisión lo incluye como motivo separado
(«Disponible en tu familia de Steam»), con etiqueta distinta. Perder la familia retira esos
derechos en la siguiente sincronización sin borrar fichas ni decisiones de equivalencia.

### 6.3 Wishlists: lectura [CONFIRMADO]

| Tienda | Vía                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Estado                               |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------ |
| Steam  | `IWishlistService/GetWishlist/v1` con `access_token` (Augmented Steam). Conservar `appid`, prioridad, fecha de alta. Enriquecer nombres sin perder entradas retiradas o sin metadatos. Comprobar si el servicio acepta parámetros de consulta o `input_json` antes de meter protobuf.                                                                                                                                                                                                                                | [PENDIENTE DE PRUEBA], lista privada |
| GOG    | `https://embed.gog.com/user/wishlist.json` con Bearer de Galaxy (Heroic); claves de `data.wishlist`. Enriquecer título, URL y tipo con la API de productos.                                                                                                                                                                                                                                                                                                                                                          | [PENDIENTE DE PRUEBA]                |
| Epic   | Sesión web de la tienda. Precedente (BAKLOG): cargar la wishlist en un perfil persistente y leer `data.Wishlist.wishlistItems.elements` de las respuestas GraphQL, o el estado React Query del HTML. En Tauri: ventana oculta o visible de la tienda con el perfil de Epic, y extracción desde Rust con `ExecuteScript` de WebView2 vía `with_webview`, sin exponer el bridge a la página. Si la tienda pide un desafío interactivo, se le muestra al usuario; **no se intenta evadir detección de automatización.** | [PENDIENTE DE PRUEBA], mayor riesgo  |

La vista combinada agrupa por juego reconocido, pero cada entrada sigue siendo de una cuenta, una
tienda y un producto u oferta. Quitar el deseado de Steam no toca el de GOG salvo selección
explícita.

Una wishlist vacía solo se muestra como vacía si el proveedor lo confirma con una respuesta válida.
Privada, sin permiso, sesión caducada o error de parsing tienen mensajes propios.

### 6.4 Wishlists: escritura [R11 CONFIRMADO; cada mecanismo PENDIENTE DE PRUEBA]

| Tienda | Mecanismo investigado                                                                                                                                                                                                                                                                                                                  | Estado inicial del botón              |
| ------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------- |
| Steam  | POST a `store.steampowered.com/api/addtowishlist` y `/api/removefromwishlist`, cuerpo `sessionid` + `appid`, cookies de la tienda. `sessionid` sale de `g_sessionID` en una página de la tienda. Éxito solo con `success` en el JSON. El token de lectura no autoriza esto.                                                            | Desactivado hasta la fase 4           |
| GOG    | `/user/wishlist/add/{productId}` y `/user/wishlist/remove/{productId}`: **GET con efecto de escritura**. Nunca precargar ni seguir esos enlaces. El cliente web usa un Bearer obtenido de su sesión (`api.gog.com/user/accessToken.json`); probar primero el Bearer de Galaxy y, si no basta, el token web. Validar la lista devuelta. | Desactivado hasta la fase 4           |
| Epic   | La tienda lo ofrece. Solo hay una captura GraphQL de 2022 (`addToWishlist`, `removeFromWishlist` con `namespace`, `offerId` y un enum `RemoveOperation` de valor desconocido). **No inventes el enum, cabeceras ni CSRF.** La propia ayuda de Epic reconoce fallos al quitar desde su tienda.                                          | Desactivado; puede quedarse así en v1 |

Mientras una capacidad de escritura no esté validada, el botón es **«Abrir en la tienda»**: abre la
ficha o la wishlist oficial en el navegador, explica qué hacer y ofrece «Volver a leer» al
regresar. Nunca se marca una limpieza como hecha por haber abierto la tienda.

Añadir a otra tienda exige su producto u oferta real. En v1 se añade solo desde una ficha ya
identificada (por ejemplo, un juego reconocido que existe en esa tienda). Un buscador de catálogos
de las tres tiendas no está pedido.

**[PREGUNTAR] en la fase 4:** si la escritura Epic es obligatoria para el primer release. Si lo es,
la fase 4 no se cierra sin ella; si no, v1 sale con «Abrir en Epic».

---

## 7. Reconocimiento entre tiendas

### 7.1 Tres preguntas separadas

1. **Identidad:** ¿son el mismo juego o versiones relacionadas?
2. **Cobertura:** ¿lo que tengo cubre la edición, el contenido y la plataforma que deseo?
3. **Preferencia:** aunque esté cubierto, ¿quiero mantener este deseado (logros, mods, DRM,
   colección)?

La respuesta a la primera no responde las otras dos.

### 7.2 Jerarquía de evidencia [DECISIÓN]

1. ID exacto de tienda (mismo producto) o referencia cruzada fiable.
2. Mapeo externo por IDs, con proveedor, versión y fecha.
3. Decisión manual del usuario («Es el mismo», «Son diferentes»), persistente entre
   sincronizaciones.
4. Coincidencia de título normalizado y metadatos (desarrollador, año): **candidata**.
5. Similitud difusa: solo para proponer revisión.

Cada coincidencia muestra su origen. Una puntuación heurística no es un porcentaje de certeza y no
se presenta como tal. «Exacto» se reserva para el nivel 1; una coincidencia que solo aparece tras
quitar palabras de edición no es exacta.

Normalización de títulos para candidatos: Unicode NFKD sin diacríticos, minúsculas, sin ™ ® ©,
`&` → `and`, puntuación a espacio. **Nunca** eliminar números, numerales romanos, años ni
subtítulos para decidir identidad. Quitar palabras de edición (Definitive, GOTY, Gold, Complete,
Deluxe, Remastered…) solo sirve para encontrar candidatos y marca la relación como «edición por
revisar».

Proveedor externo: [PREGUNTAR] en la fase 3. Recomendación: IsThereAnyDeal (lookup entre IDs de
tienda) como opcional, tras evaluar cobertura de IDs Epic, autenticación y condiciones de uso.
IGDB exige un secreto de cliente de Twitch que no puede ir en el instalador: descartado salvo que
el usuario decida otra cosa. La app debe funcionar sin proveedor externo, mostrar «Sin resolver» y
dejar corregir.

### 7.3 Casos que deben quedar diferenciados

Ejemplos anonimizados, con «Título» como juego genérico:

| Caso                                                                                                | Tratamiento por defecto                                                                              |
| --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| «Título» deseado en Steam, «Título™» poseído en GOG, identidad respaldada                           | Candidato claro. Requiere selección del usuario.                                                     |
| «Título» deseado, «Título: Gold Edition» poseído                                                    | Candidato «edición superior»: mostrar el contenido conocido de la edición. No se deduce del nombre.  |
| «Título: Deluxe» deseado, «Título» base poseído                                                     | No cubierto. Mostrar la diferencia.                                                                  |
| «Título Remastered» o «Título: Ultra Deluxe» frente a «Título»                                      | Productos distintos. No cubierto.                                                                    |
| «Título II» o «Título 2» frente a «Título»                                                          | Distintos. Los números nunca se eliminan.                                                            |
| «Título: Prologue», demo o banda sonora frente a «Título»                                           | No equivalentes en ningún sentido.                                                                   |
| DLC frente a juego base                                                                             | No equivalentes.                                                                                     |
| Bundle o colección que contiene «Título»                                                            | Cobertura solo si los componentes están acreditados.                                                 |
| Títulos parecidos de juegos distintos (una letra de diferencia, o «The Outer X» frente a «Outer X») | Nunca candidatos por similitud alta sin otra evidencia.                                              |
| Licencia solo Android frente a deseado de PC                                                        | No cubre. Se compara PC con PC por defecto.                                                          |
| Solo acceso familiar                                                                                | Categoría aparte, excluida por defecto.                                                              |
| Prueba, fin de semana gratuito o suscripción                                                        | No es propiedad permanente. Estado desconocido si no se puede acreditar.                             |
| Herramientas, ramas de prueba o servidores dedicados en la biblioteca                               | No acreditan poseer el juego.                                                                        |
| Deseado sin nombre (ficha retirada)                                                                 | Se conserva por ID, se intenta recuperar el nombre y se ofrece revisar aparte; no es «ya lo tienes». |
| Mismo juego deseado en dos tiendas sin poseerlo                                                     | «Deseado repetido entre tiendas», no «ya lo tienes».                                                 |

Acciones por candidato: **«Es el mismo»**, **«Son diferentes»**, **«Mantener en esta wishlist»**. La
última es preferencia sobre una entrada y no borra la relación.

### 7.4 Pruebas del motor de equivalencias

Tabla de casos con fixtures sintéticos (títulos inventados) para cada fila de 7.3, más: acentos y
mayúsculas, marcas comerciales, `&`, subtítulos con dos puntos o guion, números romanos frente a
arábigos, años. Ninguna fila de 7.3 puede producir un candidato «exacto» falso. Sin datos reales
del usuario en los fixtures.

---

## 8. Limpieza y cambios remotos

### 8.1 Flujo

1. Conectar y sincronizar las cuentas a comparar.
2. Elegir wishlist(s) objetivo y bibliotecas de referencia; acceso familiar aparte.
3. Ver candidatos con: producto deseado, producto poseído, tienda, edición, tipo de acceso, motivo,
   fecha de los datos.
4. Resolver ambiguos y marcar lo que se mantiene. **Nada se preselecciona por umbral.**
5. Seleccionar entradas concretas y revisar el resumen: «Quitar 8 deseados de Steam y 2 de GOG en
   la cuenta X». Confirmación dentro de la app ligada a ese lote, esa revisión y esas cuentas.
6. Verificar precondiciones, ejecutar, mostrar progreso y resultado por entrada.
7. Releer y actualizar la vista. Exportar el resultado si se quiere.

### 8.2 Reglas de mutación [DECISIÓN]

- El plan guarda cuenta, tienda, ID exacto, acción (estado deseado `present`/`absent`, nunca un
  toggle), evidencia y snapshot de referencia. Ninguna sincronización ni búsqueda ejecuta un plan.
- Antes de aplicar: misma cuenta conectada, wishlist objetivo releída, evidencia de propiedad
  vigente (refrescar o pedir revisión explícita si solo hay caché). Si algo cambió, se recalcula y
  se pide revisar el plan nuevo. Nunca se cambia en silencio lo confirmado.
- Si la entrada ya está en el estado deseado: «Sin cambios», sin petición.
- Escrituras serializadas por cuenta, con pausa entre peticiones y `Retry-After`.
- Éxito solo con respuesta semántica válida (Steam `success`, GOG lista devuelta sin el producto,
  Epic sin `errors` aunque el HTTP sea 200) **y** relectura posterior.
- Timeout tras enviar: releer antes de reintentar. Un timeout no demuestra que no se aplicara.
- Estados por operación: `applied`, `already_in_desired_state`, `failed`, `needs_reauth`,
  `not_supported`, `unverified`, `cancelled_not_sent`. Textos llanos en la GUI.
- Doble clic, reintentos y reconexiones no envían dos veces la misma operación (idempotencia local
  por `operation_id`).
- Historial local de lotes con snapshot previo sin secretos. «Reintentar» solo sobre pendientes o
  fallidas, tras reconciliar. «Deshacer» = volver a añadir con confirmación, donde esté soportado,
  avisando de que posición y fecha originales pueden perderse.
- **Sin cola oculta:** al recuperar la red no se ejecuta nada pendiente sin una nueva revisión.
- **Sin borrado autónomo por similitud, jamás.**

### 8.3 Habilitación de capacidades de escritura

Una capacidad de escritura se habilita en código solo cuando existe un registro de validación en
`docs/validation/<proveedor>-write.md` con fecha, versión de la app, operaciones probadas,
resultado y relectura (sin datos personales). Sin ese registro, la UI muestra «Abrir en la tienda».
En ejecución, además, se comprueban las precondiciones de sesión (cookies, `sessionid`, token
vigente).

---

## 9. Interfaz

### 9.1 Vistas

| Vista            | Contenido                                                                                                                              |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| Conexiones       | Steam, GOG, Epic: cuenta, capacidades con su estado, última sincronización, conectar, reconectar, desconectar, sincronizar.            |
| Biblioteca       | Todas, Steam, Steam familiar, GOG, Epic. Fichas agrupadas con detalle de productos y accesos; conmutador a registros separados.        |
| Wishlists        | Por tienda o todas; estado de coincidencia con bibliotecas, edición y control de añadir/quitar (o «Abrir en la tienda»).               |
| Revisar limpieza | Candidatos explicados, comparación lado a lado, «Es el mismo», «Son diferentes», «Mantener», selección y resumen del lote.             |
| Resultados       | Resultado por entrada del lote, reintento de fallidas tras reconciliar, historial de lotes.                                            |
| Ajustes          | Acceso familiar en limpieza, exportación, datos locales (abrir carpeta, borrar caché), actualizaciones, diagnóstico (logs redactados). |

### 9.2 Búsqueda, filtros y contadores

- Texto sobre título, alias y nombre mostrado, sin distinguir mayúsculas ni acentos, combinable con
  filtros de forma predecible.
- Filtros: tienda, biblioteca o wishlist, propio frente a familiar, plataforma conocida, tipo de
  producto (incluido «desconocido», visible), «ya tienes en otra tienda», «solo familia», «por
  revisar», «sin coincidencia», «mantenidos», «metadatos incompletos».
- Orden por título; en wishlist por prioridad o fecha solo donde existan; en limpieza por clase de
  evidencia.
- Contadores separados: juegos reconocidos, productos o licencias por tienda, accesos familiares.
  Conteo filtrado y total. «Limpiar filtros». Los filtros se mantienen al volver del detalle.
- Vacío por filtros, vacío confirmado por el proveedor y fallo de sincronización son tres estados
  distintos con textos distintos.
- Las bibliotecas pueden tener miles de entradas: lista virtualizada, búsqueda sin bloquear la UI.
- Precio, géneros, descuentos, logros y tiempo jugado no son obligatorios en v1.

### 9.3 Calidad de UI

Accesible con teclado entero, foco visible, contraste AA, `prefers-reduced-motion`, tema claro y
oscuro por tokens. Confirmaciones de lote con `<dialog>` modal (ver `/modern-web-guidance`), no
`window.confirm`. Dirección visual con `/impeccable` y el _Floor_ de `/hallmark`. Copy en español
llano revisado con `/unslop`: nada de «sincronización exitosa» cuando fue parcial.

### 9.4 Datos simulados

Se permiten fixtures para desarrollar UI y contratos, con estas condiciones:

- Viven detrás de una feature de Cargo `fixtures`, desactivada en release. CI falla si un build de
  release la incluye.
- Cuando están activos, la ventana muestra una franja fija «Datos de ejemplo».
- Un mock, un botón deshabilitado, un fixture o una importación CSV **no** cuentan como integración
  hecha en ningún informe de fase.
- La importación CSV/JSON no sustituye a ninguna conexión. Si se implementa, es una función extra
  claramente etiquetada.

---

## 10. Fases

Cada fase acaba con: verificación de CI en local (`pnpm lint`, `pnpm check`, `pnpm test`,
`pnpm build`, `cargo clippy -- -D warnings`, `cargo test`), commits por paso lógico, CHANGELOG en
`[Unreleased]` y un informe en `docs/phases/<n>.md` (pasado por `/unslop`) con lo probado, dónde, y
lo no probado.

### Fase 0: esqueleto que se instala y se actualiza

- Arranque con devkit (4.1), skills (3.1), `CONTEXT.md` con el glosario de 4.5, ADR de la decisión
  4.2.
- Ventana principal vacía con navegación, single-instance, logs redactados, base de datos con la
  primera migración.
- Pipeline de release solo Windows. El `release-tauri.yml` reutilizable de devkit construye Windows
  y macOS sin parámetro de plataformas. [PREGUNTAR]: añadir un input `platforms` al workflow de
  `AlexAdiaconitei/actions` (cambio en devkit, recomendado) o un workflow propio del proyecto.
- Claves del updater: el usuario las genera con `pnpm tauri signer generate` y las sube como
  secretos. **Tú nunca ves la clave privada.** Firma Authenticode: [PREGUNTAR] (certificado, coste,
  momento); el updater funciona sin ella, SmartScreen avisará.
- [PREGUNTAR] repo de distribución público (necesario para que el updater descargue sin
  credenciales) y Windows mínimo (recomendado: Windows 10 22H2 y 11, x64; ARM64 fuera de v1).

**Salida:** v0.0.1 instalada con el NSIS en una máquina o VM Windows sin herramientas de desarrollo,
actualizada a v0.0.2 desde un borrador de release. Prueba también: paquete con firma alterada
rechazado, descarga fallida que deja usable la versión instalada.

### Fase 1: pruebas de integración (decide la viabilidad real)

Spikes con código de producción mínimo, no prototipos desechables, detrás de un menú de
diagnóstico:

1. Login de las tres tiendas en WebView2 con contraseña, MFA o Steam Guard, QR de Steam y login
   social de Epic. Cancelar, reintentar, reiniciar la app con sesión guardada, sesión caducada.
2. Biblioteca propia de las tres, con conteos comparados con la web oficial.
3. Steam Families: sin familia, con familia, apps con varios propietarios y excluidas.
4. Wishlist de las tres, privada, con varias páginas.
5. Persistencia de secretos con DPAPI y renovación GOG y Epic.

**Las pruebas autenticadas requieren al usuario.** Prepara cada prueba, explícale qué tiene que
hacer y qué verás, y deja que inicie sesión él. No pidas contraseñas por chat. Las respuestas
reales que guardes como fixtures se anonimizan (IDs, nombres, avatares) antes de entrar al repo.

**Salida:** `docs/validation/phase-1.md` con una tabla capacidad × resultado (`funciona`,
`funciona con límites`, `no funciona`) y evidencia. Si algo no funciona, para y [PREGUNTAR] con
alternativas y coste (incluido el plan B de 4.2). No construyas la UI completa antes de este punto.

### Fase 2: bibliotecas y wishlists

Sincronización persistente por recurso con generaciones (`complete`, `partial`, `failed`), caché
con fecha visible, uso sin red, vistas de 9.1 (salvo limpieza), búsqueda, filtros, contadores,
exportación CSV/JSON con diálogo.

**Salida:** A03 a A08 y A15 pasan contra cuentas reales.

### Fase 3: equivalencias y revisión

Motor de 7, relaciones manuales persistentes, preferencia «Mantener», vista de revisión, decisión
sobre proveedor externo.

**Salida:** A09 y A10 pasan; la tabla de 7.4 completa en tests.

### Fase 4: limpieza integrada

Planes, confirmación, ejecución y reconciliación (8). Escritura Steam y GOG validada con productos
de prueba que elija el usuario, uno a uno, con su permiso explícito para cada mutación. Epic según
la respuesta a la pregunta de 6.4. «Abrir en la tienda» para lo no validado.

**Salida:** A11 a A14 pasan; registros de 8.3 escritos.

### Fase 5: distribución y cierre

Pulido de UI (`/impeccable` crítica + `/unslop`), README de usuario en español, avisos de licencias
de terceros, `release-prep` para v0.1.0, borrador de release, instalación limpia y actualización
v0.1.0 → v0.1.1 con datos y sesiones conservados.

**Salida:** A01, A02, A16, A17 y A18 pasan; release en borrador lista para que el usuario la
publique.

---

## 11. Pruebas de aceptación

| ID  | Comprobación                                                                                                                                         |
| --- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| A01 | Una persona instala y conecta las tres cuentas desde la GUI sin herramientas de desarrollo ni códigos manuales.                                      |
| A02 | MFA, Steam Guard, QR y los logins sociales declarados como soportados funcionan en el paquete; cancelar y reintentar no deja intentos colgados.      |
| A03 | Reiniciar, caducar la sesión y cambiar de cuenta dan estados correctos, sin mezclar datos ni vaciar bibliotecas por error.                           |
| A04 | La biblioteca propia incluye juegos no instalados y distingue tipos de acceso cuando se conocen.                                                     |
| A05 | Families con una sola conexión: sin familia, varios propietarios, exclusiones. Nada familiar figura como comprado.                                   |
| A06 | Una ficha agrupada conserva todos sus productos y accesos; lo no resuelto queda separado; contadores coherentes.                                     |
| A07 | Texto y filtros se combinan en vistas por tienda y combinadas; vacío por filtros ≠ vacío confirmado ≠ fallo.                                         |
| A08 | Wishlists con varias páginas, próximos lanzamientos, productos retirados, IDs sin nombre y listas vacías legítimas.                                  |
| A09 | Original y remaster, secuelas, DLC y base, móvil y PC, bundles y ediciones incompletas no generan limpieza falsa.                                    |
| A10 | Relaciones manuales positivas y negativas y «Mantener» sobreviven a nuevas sincronizaciones y no se confunden entre sí.                              |
| A11 | Un deseado en dos tiendas tiene acciones separadas; la confirmación afecta solo a las entradas y cuentas previstas.                                  |
| A12 | Añadir y quitar probados con productos de prueba autorizados, con estado remoto verificado. Ninguna mutación real en CI.                             |
| A13 | Timeout tras envío, error GraphQL con 200, rate limit, doble clic, cancelación y éxito parcial no producen falsos éxitos ni reintentos ciegos.       |
| A14 | Cuenta cambiada, snapshot viejo o lista editada desde fuera invalidan o reconcilian el plan antes de ejecutar.                                       |
| A15 | Sin red se consulta la última copia con fecha; al volver la red no se ejecuta ningún borrado sin revisión.                                           |
| A16 | Instalación en Windows limpio sin Python, Node, Rust ni Legendary; WebView2 se instala si falta; datos fuera de la carpeta de instalación.           |
| A17 | v0.1.0 → v0.1.1 conserva datos y sesiones; paquete mal firmado rechazado; descarga fallida deja la versión instalada usable.                         |
| A18 | Logs, errores, exportaciones y diagnóstico sin tokens, contraseñas ni cookies (test automático con secretos centinela); ventanas remotas sin bridge. |

Tests automáticos: parsing contra respuestas reales anonimizadas, contratos de IPC, motor de
equivalencias, máquina de estados de operaciones con fallos inyectados (timeout tras envío, 200
con errores, 429, cancelación). Las pruebas autenticadas son manuales u opt-in
(`#[ignore]` + variable de entorno), nunca en CI.

---

## 12. Seguridad y privacidad

- CSP estricta (`default-src 'self'`), sin `asset:` salvo necesidad con `scope` acotado.
- Capabilities: solo la ventana `main`, permisos por comando; ninguna para ventanas remotas.
  Audita con `tauri-security` al cerrar cada fase.
- Redacción de secretos centralizada en el logger; test con valores centinela.
- Ninguna telemetría. Datos locales por defecto, sin servicio propio ni cuenta de CrossShelf.
- Credenciales públicas de cliente (Galaxy, launcher de Epic) en el código con un comentario de
  origen; no son secretos, no se presentan como tales.
- Nada de técnicas de evasión de detección de automatización. Si un proveedor pide un desafío, lo
  ve el usuario.
- No leas `.env` ni ficheros de credenciales (norma de devkit). La clave del updater solo vive en
  los secretos del repo.

---

## 13. Licencias y procedencia

- No se copia código de Playnite, Heroic, gogdl, Legendary, Augmented Steam ni BAKLOG. Se leen para
  entender protocolos y se cita el commit en comentarios o ADR cuando una decisión dependa de ellos.
- El repo de origen (`Cadetto92/game-list-extractor`) no tiene `LICENSE`. Como se reimplementa en
  Rust, no se reutiliza su código; si se copiara algo, [PREGUNTAR] antes.
- Licencia de CrossShelf: [PREGUNTAR] antes del primer release público.
- Si se activa el plan B de Epic, Legendary (GPLv3) exige incluir licencia, aviso y acceso a sus
  fuentes en la distribución.

---

## 14. Definición de terminado

CrossShelf v0.1.0 está terminado cuando:

1. Existe un instalador NSIS por usuario, en español, construido por CI desde un tag, en un borrador
   de GitHub Releases con `latest.json` firmado.
2. A01–A18 tienen evidencia ejecutada y anotada (dónde, cuándo, con qué versión), o una excepción
   aceptada por el usuario por escrito en `docs/validation/`.
3. Cada capacidad (biblioteca, familia, wishlist, añadir, quitar × tienda) aparece en la GUI con su
   estado real. Ninguna capacidad no validada tiene un botón que aparente funcionar.
4. Ninguna integración de la tabla de la fase 1 está sustituida en silencio por mocks, fixtures o
   importación CSV.
5. CI en verde: lint, check, tests de front y Rust, clippy, build; build de release sin la feature
   `fixtures`.
6. README de usuario, CHANGELOG, avisos de terceros y ADRs al día, con el copy revisado por
   `/unslop`.
7. Ningún dato personal de las cuentas de prueba en el repositorio.

Lo que no esté terminado se dice tal cual en el informe final, con qué falta y por qué.
