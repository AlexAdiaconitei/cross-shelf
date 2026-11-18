# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

The release notes come from this file. The GitHub release for tag `vX.Y.Z` uses
the matching `## [X.Y.Z]` section, and CI refuses a tag without one.

## [Unreleased]

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
