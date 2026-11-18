# Validación de escritura: wishlist de GOG

Registro que exige la especificación (8.3) para habilitar «Quitar y añadir deseados» en GOG.

## Contexto

- Fecha: 18 de noviembre de 2026.
- Versión: 0.0.1, `pnpm tauri dev`, commit `0a05ed0` (más cambios sin commit que no tocan el escritor) (escritor en
  `src-tauri/src/writers.rs`, máquina de estados en `src-tauri/src/ops.rs`).
- Cuenta: la del usuario, conectada con el login de Galaxy. Sin IDs de cuenta en este documento.
- Petición: `GET embed.gog.com/user/wishlist/{remove|add}/{id}` con el Bearer de Galaxy. Éxito solo
  si la lista que devuelve GOG ya trae el producto en el estado pedido, y una relectura de la
  wishlist coincide.
- Producto de prueba: `1207664663` («The Witcher 3: Wild Hunt - Complete Edition», un paquete). El
  endpoint de wishlist lo daba como deseado con valor `true` aunque la web no lo mostrara.

## Permiso

El usuario autorizó por escrito quitar ese producto («Sí, quítala»). Las pruebas las hizo el
usuario desde Ajustes → Diagnóstico de conexiones → «Prueba de escritura», porque la automatización
de la interfaz se colgó en el selector de tienda antes de enviar nada. La prueba de «añadir» la
decidió y la ejecutó el propio usuario.

## Operaciones y resultado

| #   | Operación                    | Resultado de la app                                 | Relectura (pantalla Wishlists / Conexiones) |
| --- | ---------------------------- | --------------------------------------------------- | ------------------------------------------- |
| 1   | Quitar `1207664663`          | `applied`                                           | La wishlist de GOG pasó a 0 entradas        |
| 2   | Quitar `1207664663` otra vez | `already_in_desired_state` (sin petición de cambio) | 0                                           |
| 3   | Añadir `1207664663`          | `applied`                                           | Volvió a 1 entrada                          |

Resultados tal como los pegó el usuario: «Resultado: applied», «Resultado:
already_in_desired_state», «Prueba con añadir: Resultado: applied», «Despues de quitar el listar
salia 0 y despues de añadir volvia a 1».

## Conclusión

El Bearer de Galaxy basta para escribir en la wishlist de GOG (era la hipótesis sin probar de
`docs/validation/phase-4.md`). Se habilita GOG en `plans::VALIDATED_WRITES`. Los lotes de GOG
confirmados los envía la app, operación a operación, y cada una solo cuenta como hecha tras leer la
wishlist de nuevo.

## Límites

- Una sola cuenta y un solo producto (un paquete, no un juego suelto).
- No se probó un fallo de red a mitad de envío ni un 429 real; esos caminos están cubiertos por los
  tests de `ops.rs` y `writers.rs`, no por esta prueba.
- Steam sigue sin validar.
