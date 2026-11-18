# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

The release notes come from this file. The GitHub release for tag `vX.Y.Z` uses
the matching `## [X.Y.Z]` section, and CI refuses a tag without one.

## [Unreleased]

### Added

- Carátulas de los juegos en la Biblioteca (pequeña en cada fila, grande en la ficha) y en Revisar limpieza. Se cargan desde la web de cada tienda; si una no carga se ven las iniciales del juego, nunca una imagen rota. Se pueden desactivar en Ajustes. Aparecen después de volver a sincronizar cada tienda.
- El acceso familiar de Steam dice quién te comparte cada juego, con su nombre público de Steam. Si su perfil es privado, dice cuántas personas.
- Wishlists se ve como la Biblioteca: carátula, pestañas de tienda (incluido dónde ya lo tienes) y ficha lateral al pulsar un deseado.
- Logo de CrossShelf en la barra lateral, en la barra de la ventana y en el icono de Windows.
- Las barras de filtros ya no saltan de línea: «Limpiar filtros» es un icono fijo, los filtros de acceso pasan a un desplegable y las tiendas a solo logo cuando no caben. La ventana tiene un tamaño mínimo de 1024 × 640 y, si no es ancha, la ficha del juego se abre encima de la lista.
- La cabecera de cada vista no salta de línea al abrir la barra lateral: el título y los recuentos se reducen y los menos importantes se ocultan. Ninguna vista tiene scroll horizontal.
- Barras de scroll finas a juego con el diseño. El clic derecho ya no abre el menú del navegador (salvo en campos de texto) y solo se puede seleccionar texto en la ficha del juego y en los campos.
- Pantalla de carga con el logo al abrir la app, en vez de una ventana en negro.

### Changed

- Diseño nuevo de toda la app: cada tienda tiene su color y su logo (Steam cian, GOG magenta, Epic negro), y el amarillo marca solo el siguiente paso. Titulares en Archivo condensada.
- La app se abre en la Biblioteca. La barra lateral se pliega en un raíl de iconos o se abre con secciones, el número de deseados por revisar y el estado de cada tienda.
- Biblioteca agrupa por juego los productos con el mismo título en varias tiendas (con un conmutador para verlos por tienda) y abre una ficha lateral con los productos y accesos de cada juego. Las pestañas Steam, GOG, Epic y Steam familiar pasan a ser filtros de tienda y de acceso que se combinan.
- Conexiones enseña cada recuento en grande (juegos, compartidos, deseados) con su estado y la fecha de los datos.
- Los textos de ayuda y el origen de los datos pasan a un icono de información; un aviso (una sincronización parcial o fallida) se ve como icono ámbar. Las barras de filtros tienen todos sus controles a la misma altura.
- Revisar limpieza compara cada deseado con lo que ya tienes en dos mitades, con un sello de veredicto, y el lote queda fijo abajo.

### Fixed

- Sincronizar Epic tarda unos 20 segundos en vez de más de dos minutos: las fichas del catálogo se piden varias a la vez.
- Biblioteca escribe «familia de Steam» con mayúscula, y Ajustes explica bien por qué una copia construida sin firma no se actualiza sola.

## [0.1.0] - 2026-11-18

Primera versión. CrossShelf reúne tus bibliotecas de Steam, GOG y Epic y te ayuda a limpiar tus
wishlists cuando un juego que deseas ya lo tienes.

### Added

- Conexiones: conecta Steam, GOG y Epic en la ventana oficial de cada tienda; la app nunca ve tu contraseña. Cada tienda muestra el estado real de lo que la app puede hacer con ella (biblioteca, familia, wishlist, quitar deseados), con sincronizar, volver a conectar y desconectar.
- Biblioteca de las tres tiendas por pestañas (Todas, Steam, Steam familiar, GOG y Epic), con búsqueda, filtros y la fecha de los datos de cada tienda. El acceso familiar de Steam se marca siempre como tal, nunca como comprado.
- Nombre, tipo (juego, DLC, demo, banda sonora, aplicación…) y soporte de Windows de cada producto, sacados de las fichas de cada tienda.
- Wishlists de Steam y GOG con búsqueda, filtros, orden por prioridad o fecha y «Abrir en la tienda». La de Epic aparece como no disponible, nunca como lista vacía.
- Revisar limpieza: cada deseado junto a lo que ya tienes y podría cubrirlo, con el motivo. «Es el mismo», «Son diferentes» y «Mantener en esta wishlist» se guardan y sobreviven a nuevas sincronizaciones. Nada se preselecciona y un título parecido nunca basta para quitar nada.
- Quitar deseados de Steam y GOG desde la app: eliges los que ya tienes, revisas por qué se quita cada uno y «Quitar N deseados ahora» los quita y comprueba cada uno leyendo la wishlist otra vez. Validado con cuentas reales.
- Resultados: historial de cada lote con el estado de cada entrada; puedes cancelar lo que falte o reintentar lo que falló.
- Exportación de biblioteca y wishlists a CSV o JSON, sin IDs de cuenta.
- Ajustes: acceso familiar en la limpieza (desactivado por defecto), carpeta de datos, borrar los datos sincronizados, buscar actualizaciones firmadas y diagnóstico de conexiones.
- Si conectas otra cuenta de la misma tienda, los datos de la anterior se guardan aparte y no se mezclan.
- Todo se guarda solo en tu equipo, con la fecha de cada copia, y las sesiones van cifradas con la protección de datos de Windows.
- Instalador para Windows 10 y 11 en español, con la licencia MIT y los avisos de terceros.
