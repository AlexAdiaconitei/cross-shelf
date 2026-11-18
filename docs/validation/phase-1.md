# Fase 1: validación de integraciones

Estado a 17 de noviembre de 2026, versión 0.0.1. Primera ronda de pruebas autenticadas hecha por
el usuario con sus cuentas. Los recuentos se anotan tal como salieron; **ninguno se ha comparado
todavía con la web oficial**, así que ninguna fila está en `funciona` sin reservas.

Leyenda: `funciona`, `funciona con límites`, `no funciona`, `sin probar`.

| Capacidad                      | Steam                                                                                                          | GOG                                                                                                      | Epic                                                                                                                                      |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Login en WebView2              | funciona (contraseña; MFA y QR sin probar)                                                                     | funciona (MFA sin probar)                                                                                | funciona (social sin probar)                                                                                                              |
| Cancelar y reintentar          | sin probar                                                                                                     | sin probar                                                                                               | sin probar                                                                                                                                |
| Sesión guardada tras reiniciar | funciona (el usuario no tuvo que reconectar tras reinicios de `tauri dev`)                                     | funciona (igual)                                                                                         | funciona (igual)                                                                                                                          |
| Renovación / sesión caducada   | sin probar (`session_renewed` siempre `false`)                                                                 | **funciona**: caducidad forzada, `access_token_was_expired: true` y `token_refreshed: true`              | sin probar: `access_token_was_expired` salió `false`                                                                                      |
| Biblioteca propia              | funciona con límites: 254 frente a 244 en el perfil (diferencia 10, probables gratuitos jugados u otros tipos) | funciona con límites: 284 frente a 271 en la web (13 de más, probables DLC o extras)                     | `assets/Windows`: **no funciona** (403 `missing_permission`). `library-service`: 666 registros frente a 502 en la app, **sin clasificar** |
| Familia de Steam               | funciona: grupo encontrado, 557 apps, 95 con varios propietarios; sin comparar con otra fuente                 | no aplica                                                                                                | no aplica                                                                                                                                 |
| Wishlist (lectura)             | funciona: 344, coincide con el usuario                                                                         | funciona con límites: devuelve 1 ID (The Witcher 3) que el usuario quitó de la lista y la web no muestra | sin implementar (la app muestra 12)                                                                                                       |
| Wishlist (escritura)           | sin probar                                                                                                     | sin probar                                                                                               | sin probar                                                                                                                                |

## Resultados de la primera prueba real (17 de noviembre de 2026, versión 0.0.1)

Hecha por el usuario con sus cuentas. Solo se anota lo que informó, sin datos personales.

- **GOG:** login y lectura funcionaron: usuario identificado, 284 productos en una página (todos
  `gog`), 1 deseado, sin errores. «Forzar caducidad» devolvió `null` y la lectura posterior
  funcionó, pero no se anotó `token_refreshed` de esa segunda lectura (la mostrada, sin forzar,
  daba `false`, lo esperado). Desconectar parece funcionar. Faltan: comparar con la web, reinicio
  y cancelación.
- **Steam (login):** el usuario ya estaba con sesión en la ventana, esta se cerró sola (el retorno se
  detecta) y **Conectar** devolvió `needs_reauth`: la tienda no entregó un `webapi_token`. Sin
  mensaje visible en la ventana. Causa sin determinar: faltan las cookies de sesión en la lectura
  nativa o el endpoint no devuelve el token con ellas. Se añadió un diagnóstico sin valores
  (nombres de cookies y forma de la respuesta) al error de Conectar.
  **Resultado del diagnóstico:** las cookies `sessionid` y `steamLoginSecure` sí llegan; el
  endpoint responde HTTP 200 con `data.webapi_token` (518 caracteres) pero **sin `steamid`**. La
  causa era mi código, que exigía ambos. Corregido: el SteamID64 se saca del inicio de la cookie
  `steamLoginSecure`. Tras la corrección, **Conectar y Leer funcionaron**: 254 juegos propios, 344
  deseados, grupo familiar encontrado, 557 apps compartidas (con `include_own`) y 95 con más de un
  propietario. Sin comparar con la web; renovación y reinicio sin probar.
- **Epic:** la ventana de login mostró «se necesita client id». Causa: la URL de login no llevaba
  `clientId`. Corregido con la URL de Legendary (`redirectUrl` hacia `/id/api/redirect` con
  `clientId`). Como ese retorno es una página con JSON y no una navegación, el código se lee con
  `ExecuteScript` de WebView2 desde Rust. Tras la corrección, el usuario informa `connected: true`
  (login y captura del código funcionan). Falta la salida de **Leer…** (biblioteca por tipo),
  caducidad forzada y reinicio.
  **Leer (primera salida real):** cuenta e identidad correctas; `entitlements` 551
  (`AUDIENCE` 534, `ENTITLEMENT` 2, `EXECUTABLE` 15) y **`assets/Windows`: «Epic rejected the
  credentials»**. El token sirve para derechos pero no para el endpoint de activos del launcher
  que usa Legendary. La biblioteca de juegos de Epic **no funciona** con lo implementado.

## Cómo probar (requiere iniciar sesión el usuario)

`pnpm tauri dev` → Ajustes → Diagnóstico de conexiones. Cada tienda tiene Conectar, Leer, Forzar
caducidad (GOG y Epic) y Desconectar. El informe solo muestra recuentos y estados. Comparar los
recuentos con la web oficial de cada tienda y pegar el resultado aquí, sin datos personales.

## Riesgos conocidos antes de probar

- **Steam:** el endpoint `pointssummary/ajaxgetasyncconfig` y la redirección de retorno
  (`/about/?crossshelf=1`) son hipótesis; el login puede no respetar `redir`.
- **Epic:** se espera una navegación a `localhost` con `code` (precedente Heroic). Si Epic muestra
  en su lugar una página con el código, el retorno hay que capturarlo de otra forma.
- **Epic wishlist:** sin implementar hasta ver el login real; es una capacidad aparte.
- **GOG:** la wishlist con el Bearer de Galaxy está respaldada solo por Heroic.

## Resultado de la segunda ronda (17 de noviembre de 2026)

- **Epic `assets/Windows`:** 403 `errors.com.epicgames.common.missing_permission`. Es un permiso, no
  el User-Agent. La excepción del User-Agent del launcher **se retiró**; todas las peticiones llevan
  `CrossShelf/<versión>`.
- **Epic `library-service`:** 666 registros con el mismo token. Es una vía de lectura viable a falta
  de clasificar (juegos, DLC, complementos) con los metadatos y compararla con los 502 de la app.
- **GOG wishlist:** el ID devuelto (`1207664663`, The Witcher 3 según el usuario) estaba en su lista y
  la quitó. El endpoint `wishlist.json` sigue devolviéndolo: la lectura puede incluir entradas que la
  web ya no muestra. Hay que decidir en la fase 2 cómo presentarlo (entrada «sin confirmar»).
- **Steam:** 344 deseados coinciden. 254 juegos propios frente a 244 en el perfil no se puede
  cerrar sin el desglose por tipo (se pidió `include_played_free_games`).

## Decisiones del usuario tras la segunda ronda

- **Biblioteca de Epic:** se usa `library-service` y no los activos del launcher. La clasificación de
  los 666 registros frente a los 502 de la app queda para la fase 2; si no se puede clasificar bien,
  se vuelve a preguntar por Legendary.
- **Wishlist de Epic:** spike preparado (botón «Leer wishlist de la tienda» en el diagnóstico). Abre
  una ventana visible con el perfil de Epic, guarda en la propia página las respuestas GraphQL de
  `Wishlist.wishlistItems` y las lee desde Rust. Solo da `complete` si el total que declara Epic
  coincide con lo recogido. **Sin probar.**

## Tercera ronda (17 de noviembre de 2026)

- **Epic wishlist (primer intento):** la ventana terminó en `store.epicgames.com/wishlist` sin pedir
  login, pero **no se capturó ninguna respuesta GraphQL de wishlist** (`pages_captured: 0`, estado
  `waiting`) y el usuario tuvo que cerrar la ventana. Capacidad **sin funcionar** por ahora. Hipótesis
  sin comprobar: la tienda usa otra operación, XHR, o entrega los datos en el HTML. Se añadió
  diagnóstico sin valores (peticiones por ruta y operación, claves de cada respuesta, menciones de
  `wishlistItems` en el HTML) y el sondeo se limita a 1 minuto.
- **Cancelar:** el usuario informa de `Error (cancelled): Acceso cancelado` al cerrar la ventana de
  login. No dijo en qué tienda; queda anotado como probado en una, sin especificar.
- **Steam 254 frente a 244:** el usuario cree que puede deberse a juegos gratuitos jugados. Sin
  confirmar: hace falta el desglose por tipo en la fase 2.

## Cuarta ronda: wishlist de Epic con diagnóstico (17 de noviembre de 2026)

- La página muestra los 12 deseados al usuario, pero la captura dio `pages_captured: 0`. El
  diagnóstico lo explica: **la tienda pide la wishlist por XHR** (`store.epicgames.com/graphql`,
  respuesta con `data=[Wishlist]`) y mi script solo envolvía `fetch`; además esperaba la lista en
  `Wishlist.wishlistItems.elements` sin comprobar la forma real. La página incluye hCaptcha, así que
  no se descarta que a veces pida una comprobación.
- Cambios: el script envuelve también `XMLHttpRequest`, busca la lista (`elements` con `offerId`)
  dentro de `data.Wishlist` sin depender del nombre exacto y registra solo los nombres de claves de
  la respuesta. La ventana arranca **oculta**; si en 8 s no hay datos se muestra, porque puede
  haber un login o un captcha que resuelve el usuario. Sondeo cada 0,5 s y cierre tras 0,7 s si el
  total de Epic coincide.
- Comprobado con Node: el script no tiene errores de sintaxis y extrae la lista en dos formas de
  respuesta plausibles. **No está probado contra Epic**: la ventana oculta puede comportarse distinto.

## Decisión del usuario: se abandona la wishlist de Epic (17 de noviembre de 2026)

Tras cinco intentos, el usuario decide abandonar la lectura de la wishlist de Epic. Último estado
del spike, para quien lo retome: la página trae la lista incrustada en su HTML (`ssr_mentions: 1`),
no por red; las peticiones capturadas eran de avisos (`wishlistToast`), carrito y precios; la
página incluye hCaptcha y el servicio antibot `talon`; con la ventana oculta tardó más de un
minuto sin completar. El código del spike se **retiró** del repositorio; está en el historial de
git (commits `feat(epic): wishlist spike…` a `fix(epic): read the wishlist from xhr…`).

Consecuencias, que son un recorte de alcance frente a la especificación y se reflejan aquí porque
R09 y R11 son requisitos confirmados:

- R09 queda cumplido para Steam y GOG, no para Epic.
- La GUI mostrará la wishlist de Epic como **no disponible**, con un enlace «Abrir en la tienda»,
  nunca como lista vacía (spec 6.3).
- La limpieza (sección 8) no trabajará con deseados de Epic.
- Si se quiere recuperar, es una capacidad aparte y requiere una nueva decisión.

## Decisión (histórica, retirada): User-Agent del launcher en Epic

Fecha: 17 de noviembre de 2026. Decidido por el usuario tras el fallo de `assets/Windows`. La
petición de activos de Epic usa el User-Agent del launcher (el mismo que Legendary) y solo esa; el
resto de peticiones, incluidas las de Epic, llevan `CrossShelf/<versión>`. Si no resuelve el rechazo,
se vuelve a preguntar (otra vía o plan B con Legendary).

## Segunda ronda (17 de noviembre de 2026)

Comparación del usuario con las webs y apps oficiales:

| Dato            | CrossShelf                                       | Oficial                        | Lectura                                                                                                                                                                                              |
| --------------- | ------------------------------------------------ | ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| GOG biblioteca  | 284 productos                                    | 268 juegos y 3 películas (271) | **Difiere en 13.** El endpoint cuenta lanzamientos, no solo lo que muestra la web (probables DLC o extras). Hay que clasificar por tipo en la fase 2                                                 |
| GOG wishlist    | 1 ID                                             | 0                              | **Difiere.** El endpoint devuelve una entrada que la web no muestra (ficha retirada o similar). Caso previsto en 7.3: se conserva por ID y no es «ya lo tienes». Diagnóstico añadido: `wishlist_ids` |
| Epic biblioteca | assets rechazados; 551 derechos (534 `AUDIENCE`) | 502 en la app de Epic          | **No funciona.** El User-Agent del launcher no cambió el resultado                                                                                                                                   |
| Epic wishlist   | sin implementar                                  | 12                             | Pendiente (capacidad aparte)                                                                                                                                                                         |
| Steam           | 254 / 344 / 557                                  | sin comparar                   | Pendiente                                                                                                                                                                                            |

- **Epic:** el User-Agent del launcher en la petición de activos **no resolvió** el rechazo
  (`assets/Windows` sigue respondiendo «rejected the credentials»). La hipótesis queda descartada o
  insuficiente. El informe ahora incluye el `errorCode` que devuelve Epic, y una segunda vía de
  lectura (`library-service`) con su recuento.
- **Caducidad forzada:** el usuario informa de que tras «Forzar caducidad» la lectura funciona, pero el
  campo `token_refreshed` salió `false` en Epic y no se anotó en GOG. No hay caché en el código. Para
  separar «el token no estaba caducado» de «no se renovó» el informe añade
  `access_token_was_expired`.
