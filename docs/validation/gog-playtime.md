# Validación de lectura: tiempo jugado en GOG

Registro que pide el plan 0.3.0 (2.4) para pedir el tiempo jugado de GOG al abrir la ficha.

Estado: **pendiente**. Implementado y desactivado (`gog::PLAYTIME_VALIDATED = false`). Mientras no
se valide, `gog_playtime` no hace ninguna petición y la ficha dice «GOG no lo da».

## Qué se implementó

- Petición: `GET gameplay.gog.com/games/{productId}/users/{userId}/sessions` con el Bearer de
  Galaxy, solo al abrir la ficha de un juego de GOG, nunca en lote. Ruta leída de Heroic Games
  Launcher (`src/backend/storeManagers/gog/games.ts`); GOG no la documenta.
- Respuesta esperada: objeto con `time_sum`. Se toma como **minutos** hasta confirmarlo. Un 404 se
  lee como «sin registro de Galaxy» (la ficha dice «GOG no lo da»); otra forma es un error, nunca 0.
- La lectura se guarda en la caché (`playtime`) y se reutiliza durante una hora. Si los minutos
  cambian, se añade una foto de horas.
- El comando solo recibe el `native_id`; el token no sale de Rust.
- Código: `gog/library.rs` (`fetch_playtime`, `parse_sessions`, con tests), `gog/commands.rs`
  (`gog_playtime`, `gog_diag_playtime`), `playtime.rs` (`recent`, con tests).

## Cómo validarlo

1. Ajustes → Diagnóstico de conexiones → GOG → «Comprobar el tiempo jugado (validación)».
2. Anotar aquí el resultado: título y horas de los cinco primeros juegos propios (o `null` si GOG no
   tiene registro), sin IDs de cuenta.
3. Comparar esas horas con las que enseña GOG Galaxy. Si cuadran, `time_sum` son minutos; si salen
   60 veces más, son segundos y hay que cambiar `parse_sessions`.
4. Si todo cuadra: poner `PLAYTIME_VALIDATED = true` y abrir la ficha de uno de esos juegos.

## Resultado

Sin hacer. Falta una cuenta real de GOG en una sesión con la app abierta.
