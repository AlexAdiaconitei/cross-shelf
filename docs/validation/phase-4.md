# Fase 4: validación de escritura en wishlists

Estado inicial (antes de las pruebas): **ninguna escritura estaba validada**. `plans::VALIDATED_WRITES`
está vacío y un test impide añadir una tienda sin su `docs/validation/<tienda>-write.md`. Mientras
tanto los lotes usan «Abrir en la tienda» y se comprueban al volver a leer.

## Qué hay preparado

- Escritor de Steam: `POST store.steampowered.com/api/removefromwishlist` (y `addtowishlist`) con
  `sessionid` y `appid` y las cookies de la tienda. Éxito solo con `success: true` y una relectura
  de la wishlist que coincida. **Hipótesis sin probar:** que el `sessionid` de la cookie vale como
  `g_sessionID`.
- Escritor de GOG: `GET embed.gog.com/user/wishlist/remove/{id}` (y `add`) con el Bearer de Galaxy.
  Es un GET con efecto: solo se pide desde aquí, nunca se precarga. Éxito solo si la lista devuelta
  ya no trae el producto y la relectura coincide. **Sin probar** si el Bearer de Galaxy basta o hace
  falta el token web.
- Herramienta de prueba: Ajustes → Diagnóstico de conexiones → «Prueba de escritura». Hace un solo
  cambio en un solo producto, pide confirmación y devuelve el estado (`applied`, `failed`,
  `unverified`, `needs_reauth`…).

## Protocolo (lo hace el usuario, con su permiso para cada cambio)

Por tienda, con un producto de prueba que elijas tú y que no te importe tocar:

1. Anota el ID del producto (appid de Steam, ID de GOG) y comprueba en la web si está en tu
   wishlist.
2. Si no está: «Añadir». Esperado: `applied`. Compruébalo en la web.
3. «Quitar» el mismo producto. Esperado: `applied`. Compruébalo en la web.
4. Repite «Quitar». Esperado: `already_in_desired_state`, sin ninguna petición de cambio.
5. Pega aquí los cuatro resultados. Con eso se escribe `docs/validation/<tienda>-write.md` (fecha,
   versión, operaciones, resultado y relectura) y se habilita la tienda.

Si cualquier paso da algo distinto de lo esperado, se para y se revisa antes de seguir.

## Resultados

18 de noviembre de 2026: **GOG y Steam validados.** Detalle en `gog-write.md` y
`steam-write.md`; informe en `docs/phases/4.md`. Las dos hipótesis se confirmaron: el Bearer de
Galaxy basta en GOG y la cookie `sessionid` vale como `g_sessionID` en Steam.
