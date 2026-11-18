# Landing de CrossShelf

Fecha: 18 de noviembre de 2026. Alcance: `gh-pages/`, sus diccionarios y el workflow de Pages.
No se han modificado los adaptadores de tiendas ni habilitado capacidades de escritura.

## Diseño y recursos

Se han usado `design-taste-frontend`, `unslop-ui`, `modern-web-guidance` y `author-profile`.
La fuente de marca es `DESIGN.md`, `src/app.css` y los componentes de la app.

- Dirección: **The page is the object**, semilla **1652958157**. El hero se volvió a sortear una
  vez: la invitación vertical inicial no encajaba con las capturas horizontales ni con la identidad
  de la app. Resultado: **Tabbed folder stack**.
- Idea: las pestañas recortadas que identifican las tiendas en CrossShelf se convierten en las
  pestañas de la carpeta que contiene la captura principal.
- Interacción: selector de Biblioteca, Revisión y Conexiones, además del cambio de idioma.
- Se conserva Archivo, la rampa neutra, Steam cian, GOG magenta, botones cuadrados y amarillo solo
  para descargar. No se añaden fotografías genéricas ni testimonios o estadísticas inventadas.
- Ocho capturas WebP de la interfaz Svelte real a 1440 × 900: biblioteca, ficha, revisión y
  conexiones en los dos idiomas. Los juegos y carátulas son reales; las cuentas, IDs, propiedad,
  acceso familiar y wishlists son simulados. La franja inferior lo indica y hay un aviso bilingüe
  de atribución y ausencia de afiliación junto al hero. Las fuentes oficiales de Steam y su CDN
  están en `gh-pages/scripts/games.json`; las carátulas originales solo se guardan en la caché local
  ignorada. El navegador recibe un IPC simulado con comandos de lectura explícitos. No se arranca
  Rust ni se abre la base de datos de la app.
- Logos de tienda desde `StoreLogo.svelte`; iconos de los pasos desde la navegación Tabler real;
  wordmark desde el kit de marca. Archivo y Tabler conservan sus licencias en los recursos.
- Foto, biografía, datos y enlaces del autor proceden de los assets de `author-profile` en devkit.
  Se elimina el rótulo redundante «Creator» de la tarjeta y se adapta su aspecto con variables CSS.

El inventario que guió la página: tres tiendas concretas, fichas de revista, pestañas recortadas,
acceso familiar distinto de propiedad, deseados en otra tienda, decisiones manuales, lotes
confirmados, datos locales, instalador Windows y dos idiomas.

## Comprobaciones ejecutadas

Vista previa local bajo `/cross-shelf/`, en Chrome headless con perfil temporal aislado:

| Caso                                                                | Evidencia                                                                                                              |
| ------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| Inglés por defecto con configuración regional española              | `gh-pages/scripts/verify.mjs`                                                                                          |
| Inglés y español a 1440, 768, 390 y 360 px                          | Sin desbordamiento horizontal; titular de hasta dos líneas; imágenes cargadas                                          |
| Idioma recordado                                                    | Recarga conserva español                                                                                               |
| Capturas y texto alternativo                                        | Selector cambia entre las tres vistas y usa imágenes del idioma activo                                                 |
| Perfil del autor                                                    | Foto cargada, biografía en los dos idiomas y `about.json` devuelve 200 bajo el prefijo del proyecto                    |
| Teclado                                                             | Enter abre el FAQ nativo desde el foco en `summary`                                                                    |
| Sin JavaScript                                                      | HTML inglés completo, enlaces utilizables y controles dependientes de JS ocultos                                       |
| Última release con instalador                                       | Respuesta sintética: href directo al EXE y notas de esa misma versión                                                  |
| Repo inexistente, límite de peticiones, error de servidor o sin EXE | Respuestas 404, 403, 500 o assets vacíos: enlace a Releases con texto de descargas                                     |
| Selección del asset                                                 | Nueve tests de `release.test.ts`: x64 EXE, exclusión de firmas, archivos, ARM64, borradores, prereleases y URLs ajenas |
| Generación de HTML                                                  | Plantilla independiente; regeneración después de formatear con Prettier y nueva validación en navegador                |

La actualización con juegos reales incluye dieciséis títulos y sus carátulas de Steam. Se han
regenerado las ocho capturas después de decodificar las imágenes. La prueba de navegador comprueba
también que el aviso de biblioteca simulada y ausencia de afiliación aparece en ambos idiomas.

## Corrección de iconos y foco del hero

Se ha reproducido el fallo de los tres iconos de «From scattered to sorted»: `connect.svg`,
`library.svg` y `checklist.svg` contenían dos atributos `xmlns` en la raíz. El exportador añadía
el namespace a un `outerHTML` que ya lo incluía. El navegador headless toleraba ese XML inválido,
por lo que comprobar solo `naturalWidth` no detectaba el defecto.

La regresión en `gh-pages/scripts/verify.mjs` comprueba también una sola declaración de namespace
y el parseo XML de cada SVG. Antes de la corrección falló con los tres nombres; tras ella pasa.
El exportador usa `XMLSerializer` para producir documentos SVG válidos sin añadir atributos a mano.
Se comprueba además la carga de las imágenes en los dos idiomas y cuatro anchuras.

El titular pasa a «All your games. One library.» / «Tus juegos. Una biblioteca.». El subtítulo,
el título del documento y la descripción presentan la biblioteca unificada, la búsqueda y las
tiendas en las que tienes cada juego. La limpieza de wishlists sigue explicada en su sección.

El informe del inspector y las capturas de QA se generan en `.taste-skill/`, ignorado por Git y
Prettier. El comando está documentado en el README de la landing. El escáner de `unslop-ui` no
encuentra indicadores de severidad alta, media o baja en los archivos examinados.

La revisión Svelte de `StoreLogo.svelte` con el autofixer 5.57.3 no devuelve problemas. El componente
solo se ha leído para reutilizar sus recursos; no se ha modificado.

Verificación del repositorio: `pnpm lint`, `pnpm check`, `pnpm test` y `pnpm build` terminan con
código 0. Svelte devuelve 0 errores y 0 avisos; Vitest, 21 archivos y 55 tests aprobados, incluidos
los nueve de descarga de la landing. El inspector final devuelve 0 fallos y tres avisos
justificados abajo. Se han abierto sus siete imágenes de desktop, tablet y móvil.

## Crítica visual

ROUND 1

Seen in one second: identidad de la app y captura de biblioteca, pero titular mal partido.
Idea (one sentence): las pestañas de tienda sostienen la carpeta que contiene la biblioteca.
Live thing: selector de capturas y de idioma.
Layout words by section: hero=carpeta / preview=visor / cleanup=comparación / how=lista / faq=índice / about=autor.
Scores: design 6 · usability 7 · creativity 7 · content 9 → overall 6.0 (cap aplicado).
Caps applied: fallos del inspector; nota global limitada a 6.0 hasta corregirlos.
Top three problems (most visible first), each with the screenshot it shows in and the fix:

1. Titular de tres líneas en desktop y cuatro en tablet: ampliar su columna en tablet y ajustar
   el tamaño máximo; comprobar también 360 px.
2. Texto blanco de Steam por debajo de AA: mantener el cian de marca y usar tinta oscura.
3. Rótulo «Creator» repetido sobre el nombre: quitarlo de la plantilla y del renderer del autor.

ROUND 2

Seen in one second: app de Windows que reúne bibliotecas y ayuda a revisar wishlists.
Idea (one sentence): las pestañas de tienda sostienen la carpeta que contiene la biblioteca.
Live thing: selector con capturas reales de tres vistas en dos idiomas.
Layout words by section: hero=carpeta / preview=visor / cleanup=comparación / how=lista / faq=índice / about=autor.
Scores: design 7 · usability 8.5 · creativity 7 · content 9 → overall 7.7.
Caps applied: diseño limitado a 7 por ausencia deliberada de movimiento al cargar.
Top three problems (most visible first), each with the screenshot it shows in and the fix:

1. Las capturas densas se leen pequeñas en móvil: funcionan como vista general; no se ocultan
   columnas ni se simula una versión móvil inexistente de la app de escritorio.
2. Hay varios cuerpos tipográficos pequeños para metadatos, navegación y leyendas: se conservan
   por función; el texto corrido es de 16 px o más.
3. La página usa capturas estáticas: no se añade un vídeo ajeno al producto para llenar una cuota.

El inspector se ejecuta con `--allow still-page`: `DESIGN.md` pide movimiento de 150–200 ms solo
en cambios de estado y ninguna animación al cargar. La landing aplica feedback a controles y
respeta `prefers-reduced-motion`. Se mantiene el fondo casi blanco de la app pese al aviso
`palette-cream`; no se introduce una paleta crema/serif nueva.

## Despliegue y límites

Devkit no incluye una plantilla específica de Pages. Se han revisado sus workflows Node y las
acciones compartidas: no hacen falta para servir archivos estáticos. Pages usa las acciones
oficiales fijadas por SHA y solo se activa con `push` a `main` y cambios en `gh-pages/**`.
El paquete publicado excluye README, pruebas y herramientas. Las releases no disparan Pages:
el navegador resuelve el instalador actual en cada visita.

El repositorio de distribución es `AlexAdiaconitei/cross-shelf`, la misma cuenta que los enlaces del
autor. (La tarea original lo escribía `AlexAdiacontiei`, una errata: esa cuenta no existe; corregido
el 18 de noviembre de 2026.) El repositorio aún no existe:
no se ha podido ejecutar el workflow remoto, comprobar permisos del entorno, validar la URL
publicada ni descargar un instalador real. Hay que seleccionar GitHub Actions como fuente de
Pages cuando se cree. No se ha creado, publicado ni enviado nada a GitHub.
