# Validación de escritura: wishlist de Steam

Registro que exige la especificación (8.3) para habilitar «Quitar y añadir deseados» en Steam.

## Contexto

- Fecha: 18 de noviembre de 2026.
- Versión: 0.0.1, `pnpm tauri dev`. Escritor en `src-tauri/src/writers.rs`, máquina de estados en
  `src-tauri/src/ops.rs`.
- Cuenta: la del usuario, conectada en la ventana de login de Steam. Sin IDs de cuenta en este
  documento.
- Petición: `POST store.steampowered.com/api/{addtowishlist|removefromwishlist}` con `sessionid` y
  `appid` y las cookies de la tienda. Éxito solo con `success: true` y una relectura de la wishlist
  que coincida.
- Producto de prueba: `271590` («Grand Theft Auto V Legacy»). Lo eligió el agente a petición del
  usuario y comprobó antes que no estaba en la biblioteca, ni en la familia, ni en la wishlist.

## Permiso

El usuario eligió «Elígelo tú» para el producto y después autorizó «Añadir y quitar» para
`271590`. El agente ejecutó los pasos desde Ajustes → Diagnóstico de conexiones → «Prueba de
escritura», con UI Automation, y comprobó el formulario (tienda, acción e ID) antes de cada envío.

## Operaciones y resultado

| #   | Operación                | Resultado de la app                                 | Relectura (sincronización de Steam)        |
| --- | ------------------------ | --------------------------------------------------- | ------------------------------------------ |
| 1   | Añadir `271590`          | `applied`                                           | 345 entradas, `271590` presente            |
| 2   | Quitar `271590`          | `applied`                                           | (comprobado en el paso 3)                  |
| 3   | Quitar `271590` de nuevo | `already_in_desired_state` (sin petición de cambio) | 344 entradas, `271590` ausente; como antes |

## Conclusión

El valor de la cookie `sessionid` sirve como `g_sessionID` para escribir (era la hipótesis sin
probar de `docs/validation/phase-4.md`). Se habilita Steam en `plans::VALIDATED_WRITES`.

## Límites

- Una sola cuenta y un solo producto.
- No se probó un fallo de red a mitad de envío ni un 429 real; esos caminos están cubiertos por los
  tests de `ops.rs` y `writers.rs`.
- No se probó con una sesión de tienda caducada (debería dar `needs_reauth`).
