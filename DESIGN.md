# CrossShelf: diseño

Mundo visual **«Ficha de revista de quiosco»**. Cada juego se lee como la ficha técnica de una
revista de videojuegos de los 90, y cada tienda es una pestaña de color en su esquina. Superficie
en modo _Operate_: la navegación, los controles y las tablas son los de siempre; el mundo solo
aporta tipo, paleta, densidad y una firma. Tokens en `src/app.css`. Producto en `PRODUCT.md`.

## Paleta

| Token                                            | Uso                                                                  |
| ------------------------------------------------ | -------------------------------------------------------------------- |
| `z0`–`z10`                                       | Rampa única de neutros. Ningún gris fuera de ella.                   |
| `page`, `panel`, `raised`                        | Fondo, franja secundaria (estado, ficha), hover.                     |
| `border`, `border-strong`                        | Filete fino entre filas; filete de 2–3 px bajo cabeceras y en cajas. |
| `text`, `muted`                                  | Tinta y texto secundario.                                            |
| `rail`, `rail-text`, `rail-muted`, `rail-raised` | Barra lateral, siempre oscura en los dos temas.                      |
| `steam`, `gog`, `epic` (+ `on-store`, `on-epic`) | Color de cada tienda. Solo para identificar la tienda.               |
| `next`, `on-next`, `next-wash`                   | Amarillo: el siguiente paso y los deseados que ya tienes. Nada más.  |
| `success`, `warning`, `danger`                   | Estado. Nunca decoración.                                            |

Claro y oscuro siguen al sistema; `[data-theme]` fuerza uno. En oscuro, Epic pasa a blanco.

## Tipografía

- **Archivo** variable (ejes `wght` y `wdth`), una sola familia.
- `.headline`: anchura 62, peso 900, mayúsculas. Solo títulos de página, de columna y de ficha.
- `.figure`: anchura 72, peso 800. Recuentos.
- `.label`: 11 px, versalitas espaciadas. Rótulos de campo.
- Filas y datos a un solo cuerpo (14 px); el rango lo dan el peso y el filete, no el tamaño.
- Números tabulares en toda la app. Geist Mono solo para el diagnóstico.

## Componentes

- **Pestañas de tienda** (`StoreTabs`), la firma: llena con la esquina cortada = tuyo; contorno =
  acceso familiar o temporal; punteada = en tu wishlist; amarilla punteada = deseado que ya tienes
  en otra tienda.
- **Carátula** (`Cover`): 3:4, pequeña (30 px) en filas, mediana (48 px) en tarjetas, grande en la
  ficha. Las iniciales en el color de la tienda están siempre debajo; la imagen solo aparece cuando
  ha cargado. Un fallo nunca deja ver un icono roto.
- **Logos** (`StoreLogo`): Steam y Epic de svgl.app, GOG de svgrepo.com, en una sola tinta.
- **Botones**: caja recta con borde de 2 px. `primary` en tinta, `next` en amarillo, `quiet`,
  `ghost`, `danger`. Sin radios.
- **Chips** de filtro: borde fino; pulsado en tinta.
- Iconos: Tabler, trazo 1,75.

## Layout

- Barra lateral plegable: plegada es un raíl de 84 px con icono y rótulo corto; abierta (252 px)
  agrupa las secciones (Colección, Limpieza, Cuenta), cuenta los deseados por revisar y enseña el
  estado de cada tienda. La preferencia se recuerda por equipo.
- Cabecera de página (`PageHeader`): titular condensado, recuentos separados por filetes y, a la
  derecha, el siguiente paso en amarillo.
- Biblioteca: tabla densa y virtualizada; al elegir una fila se abre la ficha técnica a la derecha
  (440 px). La tabla esconde columnas por anchura de su contenedor, no de la ventana.
- Revisar limpieza: deseado y lo que ya tienes en dos mitades, sello de veredicto, lote fijo abajo.

## Movimiento

150–200 ms, solo para cambios de estado (hover, plegar la barra, aparecer la carátula). Nada se
anima al cargar. `prefers-reduced-motion` lo apaga todo.

## No hacer

- Usar el amarillo para cualquier otra cosa que el siguiente paso.
- Usar el color de una tienda para algo que no sea esa tienda.
- Radios, sombras de tarjeta, degradados o glass.
- Titulares condensados en datos, botones o etiquetas de fila.
