# Fase 2: validación de bibliotecas y wishlists

Estado a 17 de noviembre de 2026, versión 0.0.1 (commit `d5477ee` o posterior). Las pruebas con
cuenta real las hace el usuario. Aquí se anota solo lo que informe, sin datos personales.

## Qué hay para probar

- **Conexiones** (`/`): estado de cada tienda y de cada capacidad, con conectar, volver a conectar,
  sincronizar y desconectar. La sincronización de Steam y GOG guarda en la base por generaciones.
- **Biblioteca**: pestañas, búsqueda, filtros, contadores, fecha de los datos, exportación.
- **Wishlists**: Steam y GOG, filtros, orden, estados vacíos distintos, «Abrir en la tienda».
  Epic aparece como no disponible.
- **Ajustes**: acceso familiar en la limpieza, exportación, abrir carpeta de datos, borrar datos
  sincronizados.
- Las respuestas crudas de cada sincronización se guardan en
  `%LOCALAPPDATA%\com.alexadiaconitei.cross-shelf\raw\<tienda>\` (las 3 últimas por recurso).
  Tienen títulos de tus juegos: no se suben al repositorio.

## Protocolo

Arranque: `pnpm tauri dev`. En cada paso, anota lo que ves en pantalla (recuentos y estados; nada
de nombres de cuenta ni IDs).

1. **Conexiones → GOG → Sincronizar.** Esperado: Biblioteca «Al día: 271 elementos» (268 juegos y
   3 películas según la web) y Wishlist «Al día: 1 elementos». Anota también si aparece el nombre
   de la cuenta.
2. **Conexiones → Steam → Sincronizar.** Anota los recuentos de Biblioteca, Familia de Steam y
   Wishlist, y si aparece tu nombre de Steam o solo «Cuenta con ID…».
3. **Biblioteca.** Recorre las pestañas. Comprueba que «Steam familiar» solo trae acceso familiar,
   que la búsqueda ignora acentos y mayúsculas y que «Limpiar filtros» vuelve al total.
4. **Wishlists.** Comprueba los recuentos por tienda, el orden por prioridad y por fecha, y pulsa
   «Abrir en la tienda» en un deseado de Steam y en otro de GOG.
5. **Exportar** biblioteca y wishlists en CSV y en JSON. Abre el CSV en Excel: acentos bien y sin
   columnas de ID de cuenta.
6. **Sin red (A15).** Desactiva la red. Abre Biblioteca y Wishlists: deben verse los datos con su
   fecha. Pulsa Sincronizar en Steam: debe salir «Falló (sin red o la tienda no responde). Se
   muestran los datos del…» y la biblioteca no debe vaciarse. Vuelve a activar la red.
7. **Sondas de diagnóstico** (Ajustes → Diagnóstico de conexiones), para cerrar los mapeos de tipo
   y nombre. Pega la salida tal cual: solo trae nombres de campos y recuentos.
   - Steam → «Ver tipo y nombre de una muestra (tienda)».
   - Epic → «Ver catálogo de una muestra».
8. **Pregunta sobre GOG.** ¿Tienes The Witcher 3 comprado en GOG? Si es así, la entrada que la web
   no muestra se explicaría porque GOG oculta en su web los deseados que ya posees.
9. **Cambio de cuenta (A03).** Solo si tienes una segunda cuenta de alguna tienda: desconecta,
   conecta la otra y comprueba que aparece el aviso de cuenta cambiada y que Biblioteca solo
   muestra la nueva. Si no tienes, dilo y se anota como excepción.

## Resultados

### 18 de noviembre de 2026, con las cuentas del usuario

El usuario informó de que Steam y GOG se listan, y de que la mayoría de los deseados de Steam se
veían sin nombre, solo con el ID. También aclaró que The Witcher 3 lo tiene en Steam y que por eso
lo quitó de la wishlist de GOG. Después pidió que las pruebas las hiciera el agente.

**Cómo se probó.** Se usó la app en marcha (`pnpm tauri dev`, sesiones ya conectadas por el
usuario), manejada con UI Automation de Windows: leer la ventana y pulsar botones. La herramienta
de Orca para manejar la interfaz no se pudo usar porque el antivirus bloquea su script. La base
local se consultó en solo lectura. Las fichas públicas de catálogo de Steam y GOG se consultaron
sin cuenta. No se leyó ningún secreto.

| Paso | Resultado                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | GOG: biblioteca «Al día: 271 elementos», wishlist «Al día: 1 elemento». Se ve el nombre de la cuenta.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| 2    | Steam: biblioteca 254, familia 438, wishlist 344. Antes salía «Cuenta con ID…»: `GetPlayerSummaries` no devolvía el nombre con el token web. **Corregido** leyendo el XML público del perfil; ahora se guarda el nombre.                                                                                                                                                                                                                                                                                                                                                                                                              |
| 3    | Biblioteca: «Mostrando 1459 de 1459 · Steam: 617 · GOG: 271 · Epic: 571 · Con acceso familiar: 438», una fecha por recurso, pestañas Todas, Steam, Steam familiar, GOG y Epic. No se comprobaron uno a uno los filtros en la ventana: los cubren los tests de `filter.ts`.                                                                                                                                                                                                                                                                                                                                                            |
| 4    | Wishlists: «Mostrando 345 de 345 · Steam: 344 · GOG: 1», con nombres. **Corregido:** los deseados de Steam no tenían nombre (la wishlist solo trae `appid`) y ahora lo toman del catálogo público `IStoreBrowseService/GetItems`. Queda 1 sin nombre: la app 2986410 no tiene ficha (`success: 15`). **Corregido también:** «Prioridad 1» salía en casi todo. En la lista real, 321 de 344 tienen `priority: 0` y el resto rangos sueltos (2, 3, 5…), así que 0 quiere decir «sin orden» y ya no se muestra. No se pulsó «Abrir en la tienda».                                                                                        |
| 5    | Exportación hecha por el usuario en la app (diálogo de guardar): `crossshelf-biblioteca.csv/.json` y `crossshelf-wishlists.csv/.json`. Comprobado por el agente: CSV en UTF-8 con BOM, 1459 filas de biblioteca y 345 de wishlists (más la cabecera), los mismos registros en JSON, y ningún SteamID64 ni ID de cuenta de GOG o Epic en los cuatro archivos. No se abrió el CSV en Excel.                                                                                                                                                                                                                                             |
| 6    | **Sin probar:** cortar la red afecta a todo el equipo del usuario. Queda para el usuario.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             |
| 7    | Steam: valores de `type` vistos en fichas reales: 0 juego, 1 demo, 4 DLC, 6 software, 10 hardware y 11 banda sonora. Con ellos ahora hay tipo y soporte de Windows en biblioteca, familia y wishlist. Quedan 3 juegos propios sin tipo (playtests sin ficha). Epic: con el catálogo de la biblioteca entera se sacaron 666 registros y 571 productos distintos (el Marketplace de Unreal repite uno por versión del motor), todos con ficha. Mapeo: 497 juegos, 43 complementos (con `mainGameItem`), 7 aplicaciones y 24 herramientas (contenido del Marketplace). La app de Epic muestra 502; la diferencia de 5 no está explicada. |
| 8    | GOG devuelve `1207664663` con valor `true`. Es «The Witcher 3: Wild Hunt - Complete Edition», con `game_type: "pack"`: un paquete, no el juego. La web no lo muestra, pero el endpoint lo sigue dando por deseado. Ahora se ve con su nombre, como paquete. En «Revisar limpieza» se compara con «The Witcher 3: Wild Hunt — Remastered» de Steam y el motor dice «No lo cubre: remaster». Steam renombró la ficha 292030 y la propia biblioteca de Steam la llama así. La decisión «Es el mismo» queda para el usuario.                                                                                                              |
| 9    | **Sin probar:** necesita una segunda cuenta del usuario.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              |

Otros arreglos vistos en la ventana: el título de la ventana era «Tauri + SvelteKit + Typescript
App» (ahora «CrossShelf»), y salía «1 elementos» en singular.
