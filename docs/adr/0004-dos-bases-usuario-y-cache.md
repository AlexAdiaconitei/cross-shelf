# ADR 0004: dos bases SQLite, usuario y caché

Estado: aceptada (19 de noviembre de 2026), decidida por el usuario en el plan 0.3.0.

## Contexto

Hasta la versión 0.2.0 todo vivía en `crossshelf.db`: lo que se rehace sincronizando (productos,
derechos, wishlists, generaciones) junto a lo que solo tiene el usuario (decisiones, «Mantener»,
lotes, ajustes). «Borrar caché» borraba tablas por nombre y lo demás sobrevivía solo porque nadie lo
añadía a esa lista. En 0.3.0 llegan más datos del usuario (partidas, fotos de horas) y la caché
crece con cada sincronización. La especificación (4.4) pide migraciones con copia previa y que una
generación fallida nunca sustituya datos válidos.

## Decisión

Dos ficheros en una sola conexión: `crossshelf.db` (esquema `main`, datos del usuario) y
`crossshelf-cache.db`, adjuntado como `cache`. Las consultas nombran las tablas sin esquema y SQLite
encuentra cada una en su fichero; solo los `CREATE` de la caché llevan `cache.`. Cada fichero tiene
su línea de migraciones y su `user_version`. La migración de usuario v8 copia las tablas
sincronizadas a la caché, con sus IDs, y las borra de `main`: nadie tiene que volver a sincronizar.

| Qué                      | Usuario                           | Caché                                        |
| ------------------------ | --------------------------------- | -------------------------------------------- |
| Antes de migrar          | Copia `.bak-vN`                   | Nada                                         |
| Versión más nueva o rota | Error, la app no arranca          | Se aparta a `.broken` y se crea vacía        |
| `synchronous`            | `FULL`                            | `NORMAL`                                     |
| `auto_vacuum`            | `NONE`                            | `INCREMENTAL`, se recoge tras «Borrar caché» |
| Copias                   | Diaria, rotada, con `VACUUM INTO` | Ninguna                                      |
| Exportar                 | Sí                                | No                                           |

«Borrar caché» vacía todas las tablas del esquema `cache`, sin lista que mantener. Un test recorre
los dos esquemas y falla si una tabla no está clasificada.

## Consecuencias

- Una transacción que toque los dos ficheros no es atómica entre ellos (WAL). Ningún código lo hace
  salvo la v8, que por eso vacía la caché antes de copiar y se puede repetir.
- Sin claves foráneas entre ficheros: las tablas del usuario apuntan a productos por
  (`store`, `native_id`) y sobreviven a que la caché se vacíe.
- `sqlite_sequence` de la caché no se vacía: los lotes guardan el ID de la generación en que se
  revisaron y una generación nueva no debe reutilizarlo.

## Descartado

- Otro motor (sled, redb, JSON): SQLite ya da transacciones, migraciones y consultas, y los datos
  son relacionales.
- Dos conexiones: obligaría a unir en Rust lo que hoy une SQLite.
- Guardar carátulas en disco: la caché de WebView2 ya lo hace y la especificación evita descargar
  imágenes.
