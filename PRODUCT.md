# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

(Tauri 2 sobre WebView2, solo Windows. Ventana de escritorio, no navegador.)

## Users

Persona sin conocimientos técnicos con juegos repartidos entre Steam, GOG y Epic. Usa la app de
dos formas con el mismo peso:

- Consulta: abre la biblioteca unificada para saber si ya tiene un juego y en qué tienda.
- Ritual de limpieza: cada cierto tiempo sincroniza, revisa los deseados que ya posee en otra
  tienda y confirma un lote para quitarlos.

## Product Purpose

Reunir las bibliotecas de Steam, GOG y Epic en un solo sitio y ayudar a limpiar wishlists cuando
el juego ya está en otra tienda. Éxito: la persona entiende qué tiene, dónde, y quita deseados
sobrantes sin miedo a equivocarse.

## Positioning

Funciona en local, sin servidor propio ni proveedor externo. Cada decisión de limpieza se explica
con evidencia (por qué dos productos son el mismo juego) y nada se quita sin un lote revisado y
confirmado. Distingue propiedad, acceso familiar de Steam y accesos temporales.

## Operating Context

- Primera pantalla al abrir: la biblioteca unificada con buscador.
- Vistas: Biblioteca, Wishlists, Revisar limpieza, Resultados, Conexiones, Ajustes
  (`docs/crossshelf-build-prompt.md` §9.1).
- Bibliotecas de miles de entradas; lista virtualizada.
- Feature futura, no implementada: vista para marcar juegos en curso, pendientes y completados
  (forma abierta: kanban o listas). El layout debe dejarle sitio.

## Capabilities and Constraints

- Stack: Tauri 2, SvelteKit estático, Svelte 5, Tailwind 4, Ark UI. Iconos: Tabler (vinculante).
  Fuente: libre; hoy Geist.
- Escritura en wishlists solo donde está validada (`docs/validation/`); si no, «Abrir en la tienda».
- Tres estados vacíos distintos: vacío por filtros, vacío confirmado por la tienda, fallo de
  sincronización. Una sincronización parcial nunca se presenta como éxito.
- Con fixtures activos, franja fija «Datos de ejemplo».
- Ningún token ni secreto llega a la UI.
- Glosario del dominio en `CONTEXT.md`.

## Brand Commitments

- Nombre: CrossShelf. Interfaz en inglés (por defecto) y español, elegible en Ajustes; copy llano en
  los dos idiomas.
- Sin logo ni identidad visual previa vinculante.

## Evidence on Hand

- Datos reales solo de las cuentas del usuario. Sin portadas de juegos en la app hoy.
- Sin testimonios, cifras de uso ni capturas públicas. No inventarlos.

## Product Principles

1. Explicar antes de actuar: toda propuesta de limpieza enseña su evidencia.
2. Nada irreversible sin confirmación explícita de un lote.
3. Decir la verdad sobre el estado de los datos (completo, parcial, fallido, de cuándo).
4. Para una persona no técnica: sin jerga, sin IDs a la vista salvo que hagan falta.

## Accessibility & Inclusion

Teclado completo, foco visible, contraste AA, `prefers-reduced-motion`, tema claro y oscuro por
tokens (§9.3 de la especificación).
