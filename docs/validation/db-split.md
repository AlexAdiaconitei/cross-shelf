# Validación: base de datos en dos ficheros (ADR 0004)

Registro de la migración de usuario v8 sobre una base real y de la medida que pide el plan 0.3.0
(4.4) antes de optimizar más.

## Contexto

- Fecha: 19 de noviembre de 2026. Rama `feat/0.3.0`.
- Base: copia de la `crossshelf.db` v7 real del usuario (con su `-wal`), hecha en una carpeta
  temporal. La original no se abrió para escribir. Sin IDs ni títulos en este documento.
- Test: `db::tests::real_v7_database_migrates_without_losing_rows` (ignorado en CI), con
  `CROSSSHELF_V7_DB=<copia> cargo test real_v7 -- --ignored --nocapture`.

## Resultado

| Comprobación                                   | Resultado                                                                                                   |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Copia `.bak-v7` antes de migrar                | Sí                                                                                                          |
| Filas movidas a la caché                       | 73 generaciones, 1.927 productos, 1.534 derechos, 297 deseados, 5 miembros de familia: las mismas que había |
| Tablas de caché que quedan en `main`           | Ninguna (test de clasificación)                                                                             |
| Datos del usuario (`connection`, `setting`, …) | Idénticos antes y después                                                                                   |
| Biblioteca que ve la app (`list_library`)      | 1.459 filas antes y después: no hace falta resincronizar                                                    |
| Tamaño de `crossshelf.db` tras el `VACUUM`     | 1.093.632 → 86.016 bytes                                                                                    |

## Medida de `library_list`

`list_library` sobre esa biblioteca (1.534 derechos, 1.459 filas), media de cinco llamadas en
caliente: **7,3 ms** en `release`, 16,7 ms en `debug`. Muy por debajo de los ~100 ms a partir de los
que el plan propone filtrar en SQL o paginar: no se optimiza más.

## La app sobre la base migrada

El 19 de noviembre de 2026, `pnpm tauri dev` con un identificador aparte
(`com.alexadiaconitei.cross-shelf.dev`, para no tocar la carpeta de la app instalada) y otra copia
de la misma base v7, sin sesiones de tiendas. Al arrancar migró a v9 con su `.bak-v7` y su primera
copia diaria. La Biblioteca enseñó las 1.459 filas sin sincronizar, Revisar limpieza sus «Lotes
anteriores (4)» y Resultados sus lotes, con Revisar limpieza marcada en la barra.

## Pendiente

- Lo mismo con el instalador de 0.3.0 sobre la carpeta real, cuando se publique.
