# Validación de lectura: tiempo jugado en Epic

Registro que pide el plan 0.3.0 (2.4) para leer el tiempo jugado de Epic en cada sincronización.

Estado: **pendiente**. Implementado y desactivado (`epic::PLAYTIME_VALIDATED = false`). Mientras
no se valide, la app no hace la petición y la ficha dice «Epic no lo da».

## Qué se implementó

- Petición: `GET library-service.live.use1a.on.epicgames.com/library/api/public/playtime/account/{accountId}/all`
  con el Bearer de la sesión de Epic. Ruta leída del plugin de GOG Galaxy para Epic
  (`FriendsOfGalaxy/galaxy-integration-epic`); Epic no la documenta. El plan hablaba de una consulta
  GraphQL (`PlaytimeTracking.total`); se eligió la ruta REST porque vive en el mismo
  `library-service` que la biblioteca ya validada (`phase-1.md`). Si no responde, la GraphQL es la
  alternativa.
- Respuesta esperada: lista de `{accountId, artifactId, totalTime}`, `totalTime` en segundos. Un
  objeto (error de Epic) no cuenta como lista vacía.
- Relación con el producto: `artifactId` es el `appName` de los registros de la biblioteca
  (`library-service` los trae todos, 612 distintos en la biblioteca real del 18 de noviembre). Lo
  que no está en la biblioteca se ignora. Sin última sesión.
- Código: `epic/api.rs` (`playtime_raw`, `parse_playtime`), `epic/sync.rs` (`map_playtime`, con
  tests), `epic/commands.rs` (`epic_sync`, `epic_diag_playtime`).

## Cómo validarlo

1. Ajustes → Diagnóstico de conexiones → Epic → «Comprobar el tiempo jugado (validación)».
2. Anotar aquí `entries` (lo que devuelve Epic), `matched` (lo que se relaciona con la biblioteca) y
   la forma (`shape`), sin IDs de cuenta.
3. Comparar las horas de `top` con las que enseña el launcher de Epic para esos juegos. Si
   coinciden (redondeo aparte), la unidad son segundos.
4. Si todo cuadra: poner `PLAYTIME_VALIDATED = true`, sincronizar Epic y comprobar que la ficha de
   uno de esos juegos enseña sus horas.

## Resultado

Sin hacer. Falta una cuenta real de Epic en una sesión con la app abierta.
