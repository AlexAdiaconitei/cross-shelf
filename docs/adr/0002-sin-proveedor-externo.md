# ADR 0002: sin proveedor externo de IDs en v1

Estado: aceptada (17 de noviembre de 2026), decidida por el usuario en la fase 3.

## Contexto

La especificación (7.2) deja en manos del usuario si se usa un proveedor externo que relacione IDs
de tiendas distintas. La recomendación era IsThereAnyDeal como opción; IGDB quedaba descartado
porque exige un secreto de cliente que no puede ir en el instalador.

## Decisión

v1 no usa ningún proveedor externo. Las coincidencias entre tiendas salen de:

1. el mismo ID de tienda;
2. la decisión manual del usuario («Es el mismo», «Son diferentes»), guardada en
   `product_relation` con origen `manual`;
3. el título normalizado, solo como candidato que el usuario confirma.

## Consecuencias

- La app no envía a terceros nada de las bibliotecas ni de las wishlists.
- Sin proveedor, casi todo lo que cruza tiendas llega como «Posible coincidencia» y necesita la
  revisión del usuario. Nada se preselecciona.
- El esquema ya admite el origen `external_map`, así que un proveedor futuro no cambia el modelo.
