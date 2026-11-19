# Plan: Mis partidas

> Ampliado por `docs/plans/0.3.0.md` (tiempo jugado, Diario, sugerencias, dos bases de datos). Donde
> discrepen, manda ese plan; en particular, la tabla `play` va en la migración de usuario **v9**.

Estado: **listo para implementar** en la versión 0.3.0. Diseño validado en el prototipo de la rama
`prototype/ui-ficha` (`src/lib/prototype/Partidas.svelte`, variante D). Las decisiones marcadas
como confirmadas las tomó el usuario el 18 de noviembre de 2026.

## Qué es

Una vista para apuntar qué estás jugando, qué tienes pendiente y qué has terminado, sobre los
juegos de tu biblioteca, tengas cada uno en la tienda que sea. Se ve como tablero (una columna por
estado) o como lista agrupada, y se cambia con un conmutador.

## Decisiones

| Decisión             | Valor                                                                                    | Origen     |
| -------------------- | ---------------------------------------------------------------------------------------- | ---------- |
| De qué cuelga        | De un **producto** (`store`, `native_id`), el que elijas. No se crea «juego reconocido». | Confirmada |
| Qué se guarda        | Estado, fechas automáticas, nota corta y **orden manual** dentro de cada estado.         | Confirmada |
| Estados              | Jugando, Pendiente, En pausa, Completado, Abandonado.                                    | Prototipo  |
| Forma                | Tablero y lista, con conmutador; la elección se recuerda.                                | Prototipo  |
| Historial de cambios | No en v1.                                                                                | Confirmada |
| Exportación          | No en v1.                                                                                | Confirmada |

## Modelo de datos

Migración v7 en `src-tauri/src/db.rs`:

```sql
CREATE TABLE play (
    store TEXT NOT NULL,
    native_id TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('playing','backlog','paused','completed','dropped')),
    position REAL NOT NULL,          -- orden dentro del estado; se reparte al reordenar
    since INTEGER NOT NULL,          -- entró en el estado actual (segundos)
    started_at INTEGER,              -- primera vez en «Jugando»
    finished_at INTEGER,             -- última vez en «Completado»
    note TEXT NOT NULL DEFAULT '',
    updated_at INTEGER NOT NULL,
    PRIMARY KEY (store, native_id)
);
CREATE INDEX play_state ON play(state, position);
```

- Sin `account_id`: la partida es de la persona, no de una cuenta. Si el producto deja de estar en
  la biblioteca, la partida se conserva y la UI la marca como «ya no está en tu biblioteca».
- Borrar la caché (Ajustes) **no** borra las partidas: son datos del usuario, no de la sincronización.
  Revisar `data_clear_cache` para excluir la tabla.
- `position` como `REAL` para insertar entre dos tarjetas sin reescribir la columna entera. Se
  renormaliza (1, 2, 3…) cuando la diferencia baja de 1e-6.

## Comandos Tauri

Nuevo módulo `src-tauri/src/plays.rs` con tests, y comandos en `app_commands.rs`:

| Comando       | Entrada                                         | Efecto                                                                  |
| ------------- | ----------------------------------------------- | ----------------------------------------------------------------------- |
| `plays_list`  | —                                               | `Vec<PlayView>`: partida + `ProductRef` (título, carátula, tienda).     |
| `play_set`    | `store`, `native_id`, `state`, `before?`        | Crea o mueve; va arriba del estado o delante de `before`. Fechas solas. |
| `play_note`   | `store`, `native_id`, `note` (≤ 280 caracteres) | Guarda la nota.                                                         |
| `play_remove` | `store`, `native_id`                            | Quita la partida.                                                       |

Reglas de fechas (las del prototipo): entrar en «Jugando» fija `started_at` si no existía; entrar en
«Completado» fija `finished_at`; salir de «Completado» lo borra. `since` cambia solo si cambia el
estado. Tipos exportados con ts-rs a `src/lib/bindings/`.

## Interfaz

- **Ruta** `/partidas`, en el grupo «Colección» de la barra lateral, con `IconLayoutKanban`.
- **Cabecera**: recuento por estado (se reduce como el resto de cabeceras).
- **Barra**: buscador «Añadir un juego de tu biblioteca…» (Intro a Pendiente, Mayús+Intro a
  Jugando), conmutador Tablero/Lista.
- **Tablero**: cinco columnas con su titular condensado y recuento; «Jugando» con cabecera en
  tinta. Tarjeta: carátula mediana, título, fecha relativa («Empezado hace 23 días»), nota,
  pestañas de tienda y menú «Mover a…».
- **Lista**: secciones por estado con filas como las de la Biblioteca.
- **Mover**: arrastrar entre columnas y dentro de una columna; el menú de la tarjeta; Alt+←/→ con
  la tarjeta enfocada. Teclado completo, sin depender del arrastre.
- **Ficha**: la misma `DetailPanel`, con un campo «Partida» (estado) y la nota editable.
- **Integración**: en Biblioteca y Wishlists, la ficha también tiene el campo «Partida»; en la
  Biblioteca, la fila muestra una etiqueta con el estado.
- **Vacío**: explica para qué sirve y cómo añadir el primer juego.
- Con la ventana estrecha el tablero hace scroll **dentro** de cada columna; las columnas se
  estrechan hasta un mínimo y nunca hay scroll horizontal de la vista (se pasa a lista por debajo
  de ese mínimo).

## Pasos y commits

1. `feat(plays): play table and commands` — migración, `plays.rs`, comandos, tests de reglas de
   fechas, orden y conservación al borrar caché.
2. `feat(ui): mis partidas board and list` — ruta, vista, añadir, mover, estados vacíos.
3. `feat(ui): play state in the game sheet` — campo en `DetailPanel` y etiqueta en la Biblioteca.
4. Documentación: CHANGELOG, `DESIGN.md` (tarjeta de partida), README (qué hace).

## Pruebas

- Rust: reglas de fechas, inserción delante de otra tarjeta, renormalización de `position`, la
  partida sobrevive a `data_clear_cache` y a que el producto salga de la biblioteca.
- Frontend (vitest): búsqueda para añadir (solo juegos tuyos y no añadidos), texto de fechas
  relativas, agrupación por estado.
- A mano en la app: arrastrar, teclado (Alt+flechas), menú, conmutador, ventana mínima 1024 × 640
  sin scroll horizontal.

## Abierto

- ¿Una partida puede reunir varias tiendas del mismo juego en la tarjeta (Hades en Steam y Epic)?
  Propuesta: la tarjeta muestra las pestañas de todas las tiendas del mismo título, como la
  Biblioteca agrupada, pero la partida sigue anclada al producto elegido.
- ¿Tiempo de juego de Steam (`playtime_forever`)? Fuera de v1 (spec 9.2), pero encaja aquí más
  adelante.
