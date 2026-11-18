# 0003. Licencia MIT y nombre definitivo

Fecha: 18 de noviembre de 2026. Estado: aceptada por el usuario.

## Contexto

La especificación deja al usuario la licencia (sección 13) y si el nombre provisional pasa a
definitivo (sección 6) antes del primer release público. Las dependencias no incluyen nada GPL
(`THIRD_PARTY_NOTICES.md`) y no se usa Legendary.

## Decisión

- Licencia **MIT** (`LICENSE`), declarada en `package.json` y `src-tauri/Cargo.toml` e incluida en
  el instalador como `LICENSE.txt`, junto a los avisos de terceros.
- El nombre **CrossShelf** y el identificador `com.alexadiaconitei.cross-shelf` son definitivos: la
  cadena de actualizaciones depende de ellos.

## Consecuencias

Cualquiera puede usar, modificar y redistribuir el código con el aviso de copyright. Cambiar el
nombre o el identificador más adelante exigiría una migración de datos y rompería las
actualizaciones automáticas.
