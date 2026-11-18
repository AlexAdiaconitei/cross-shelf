# CrossShelf: glosario

Lenguaje del dominio. Una palabra, un significado. Definido a partir de la sección 4.5 de la
especificación (`docs/crossshelf-build-prompt.md`).

| Término                      | Significado                                                                                                                                                                                           |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Conexión                     | Cuenta de una tienda (Steam, GOG o Epic) enlazada a la app. Guarda proveedor, ID de cuenta, nombre, avatar, estado por capacidad y una referencia al secreto, nunca el secreto. Una por tienda en v1. |
| Capacidad                    | Algo que una conexión puede hacer: leer biblioteca, leer familia, leer wishlist, escribir wishlist. Cada una tiene su propio estado verificado.                                                       |
| Producto de tienda           | Lo que una tienda vende con su ID nativo (Steam `appid`; GOG `productId`; Epic `namespace` + `catalogItemId` + `offerId`). Es la unidad de toda operación remota.                                     |
| Juego reconocido             | Identidad local que agrupa productos. Nunca sustituye al producto nativo en una operación remota.                                                                                                     |
| Derecho de acceso            | Relación entre una cuenta y un producto: `owned`, `family_shared`, `temporary` o `unknown`. Guarda propietarios y motivo de exclusión si se conocen.                                                  |
| Entrada de wishlist          | Un producto u oferta en la lista de deseados de una cuenta y tienda concretas.                                                                                                                        |
| Relación entre productos     | Afirmación con evidencia y origen: `same`, `edition_of`, `contains`, `remaster_of`, `sequel_of`, `dlc_of`, `different`. Un rechazo manual es una fila.                                                |
| Mantener                     | Preferencia sobre una entrada de wishlist concreta. Independiente de la relación y no la borra.                                                                                                       |
| Generación de sincronización | Resultado de sincronizar un recurso de una cuenta: `complete`, `partial` o `failed`. Una biblioteca válida nunca se sustituye por una generación fallida.                                             |
| Lote de acciones             | Plan de cambios en wishlists, revisado y confirmado, ligado a cuentas y a una revisión. Se ejecuta solo tras confirmar.                                                                               |
| Acceso familiar              | Derecho `family_shared` de Steam Families. Excluido por defecto del criterio «ya lo tienes».                                                                                                          |
