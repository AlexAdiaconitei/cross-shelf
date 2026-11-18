// All user-facing copy lives here so a translation later touches one module (spec 4.1).
export const es = {
	appName: 'CrossShelf',
	nav: {
		conexiones: 'Conexiones',
		biblioteca: 'Biblioteca',
		wishlists: 'Wishlists',
		limpieza: 'Revisar limpieza',
		resultados: 'Resultados',
		ajustes: 'Ajustes'
	},
	shell: {
		sections: { coleccion: 'Colección', limpieza: 'Limpieza', cuenta: 'Cuenta' },
		openSidebar: 'Abrir la barra lateral',
		foldSidebar: 'Plegar la barra lateral',
		stores: 'Tiendas',
		wishes: 'deseados',
		storeFilter: 'Filtrar la biblioteca por tienda',
		syncFailed: 'falló la última sincronización',
		toReview: (n: number) => (n === 1 ? '1 deseado por revisar' : `${n} deseados por revisar`)
	},
	shelf: {
		storesOf: (title: string) => `Tiendas de ${title}`,
		slot: {
			own: 'tuyo',
			temp: 'acceso temporal',
			family: 'acceso familiar',
			unknown: 'acceso sin clasificar',
			wish: 'en tu wishlist',
			none: 'no está'
		},
		grouped: 'Por juego',
		separate: 'Por tienda',
		groupingLabel: 'Cómo agrupar la biblioteca',
		games: 'juegos',
		family: 'familia',
		wishedElsewhere: 'deseado en otra tienda',
		onlyFamily: (names: string[], n: number) =>
			names.length
				? `Familia · de ${names.join(', ')}`
				: n === 0
					? 'Familia'
					: n === 1
						? 'Familia · de 1 persona'
						: `Familia · de ${n} personas`,
		title: 'Título',
		access: 'Acceso',
		stores: 'Tiendas',
		wishes: 'deseados',
		detail: 'Ficha técnica',
		closeDetail: 'Cerrar la ficha',
		detailOf: (title: string) => `Ficha de ${title}`,
		products: 'Productos por tienda',
		open: 'Abrir',
		openIn: (store: string) => `Abrir en ${store}`,
		reviewThis: 'Revisar este deseado',
		sharedWith: (names: string[], n: number) =>
			names.length
				? `Compartido por ${names.join(', ')}`
				: n === 1
					? 'Compartido por 1 persona'
					: `Compartido por ${n} personas`,
		dataInfo: 'De cuándo son estos datos',
		groupInfo: 'Cómo se agrupan',
		familyNote: (n: number) =>
			`Lo comparte ${n === 1 ? '1 persona' : `${n} personas`} de tu familia de Steam. No cuenta como tuyo en la limpieza salvo que lo actives en Ajustes.`,
		inWishlist: 'en tu wishlist',
		groupNote:
			'Se agrupan por título solo para verlo junto. No es una decisión de equivalencia: eso se revisa en Limpieza.'
	},
	library: {
		search: 'Buscar por título',
		store: 'Tienda',
		allStores: 'Todas las tiendas',
		scope: 'Acceso',
		scopeAll: 'Todo',
		scopeOwn: 'Propio',
		scopeFamily: 'Con acceso familiar',
		scopeFamilyOnly: 'Solo acceso familiar',
		kind: 'Tipo',
		allKinds: 'Todos los tipos',
		clear: 'Limpiar filtros',
		noData: 'Todavía no hay nada sincronizado',
		noDataBody: 'Conecta una tienda y sincroniza para ver aquí tus juegos.',
		noMatch: 'Ningún producto coincide con los filtros',
		loadError: 'No se pudo leer la base de datos local.',
		showing: (shown: number, total: number) => `Mostrando ${shown} de ${total}`,
		perStore: (store: string, n: number) => `${store}: ${n}`,
		familyCount: (n: number) => `Con acceso familiar: ${n}`,
		untitled: (id: string) => `Sin título (ID ${id})`,
		owned: 'En tu biblioteca',
		familyShared: 'Acceso familiar',
		temporary: 'Acceso temporal',
		unknownAccess: 'Acceso sin clasificar',
		owners: (n: number) => (n === 1 ? 'de 1 persona' : `de ${n} personas`),
		dataFrom: (store: string, date: string) => `${store}: datos del ${date}`,
		lastFailed: (store: string, date: string) =>
			`${store}: la última sincronización falló. Se muestran datos del ${date}.`,
		neverComplete: (store: string) => `${store}: aún no hay una sincronización completa.`,
		syncFailed: 'La sincronización falló y todavía no hay ninguna copia que mostrar.',
		syncFailedBody: 'Mira el estado de cada tienda en Conexiones y vuelve a intentarlo.',
		tabs: {
			all: 'Todas',
			steam: 'Steam',
			steam_family: 'Steam familiar',
			gog: 'GOG',
			epic: 'Epic'
		},
		tabsLabel: 'Biblioteca a mostrar',
		export: 'Exportar biblioteca'
	},
	wishlist: {
		search: 'Buscar por título',
		status: 'Mostrar',
		statusAll: 'Todos',
		statusHeldHere: 'Ya en tu biblioteca de esa tienda',
		statusUnconfirmed: 'Sin confirmar en la web',
		statusUntitled: 'Sin título',
		sort: 'Ordenar por',
		sortPriority: 'Prioridad',
		sortAdded: 'Fecha de alta',
		sortTitle: 'Título',
		noData: 'Todavía no hay ninguna wishlist sincronizada',
		noDataBody: 'Conecta Steam o GOG en Conexiones y sincroniza.',
		syncFailed: 'No se pudo leer ninguna wishlist',
		syncFailedBody:
			'La tienda no respondió o la sesión caducó. Mira Conexiones; no es una lista vacía.',
		confirmedEmpty: 'Tus wishlists están vacías',
		confirmedEmptyBody: 'Las tiendas lo han confirmado en la última sincronización.',
		noMatch: 'Ningún deseado coincide con los filtros',
		heldHere: 'Ya está en tu biblioteca de esta tienda',
		unconfirmed: 'La tienda lo devuelve, pero su web no lo muestra',
		priority: (p: number) => `Prioridad ${p}`,
		added: (d: string) => `Añadido el ${d}`,
		openStore: 'Abrir en la tienda',
		openEpic: 'Abrir tu wishlist de Epic en su web',
		epicUnavailable:
			'La wishlist de Epic no está disponible en esta versión: la tienda no deja leerla de forma fiable. Puedes verla en su web.',
		writeNotValidated: (stores: string) =>
			`En ${stores}, quitar desde la app aún no está validado. Usa «Abrir en la tienda» y vuelve a sincronizar al terminar.`,
		writeValidated:
			'Para quitar los deseados que ya tienes en otra tienda, ve a Revisar limpieza: la app los quita por ti y lo comprueba.',
		export: 'Exportar wishlists'
	},
	export: {
		csv: 'CSV',
		json: 'JSON',
		done: (n: number) => (n === 1 ? 'Exportada 1 fila.' : `Exportadas ${n} filas.`),
		failed: (why: string) => `No se pudo exportar: ${why}.`
	},
	connections: {
		intro:
			'Conecta cada tienda en su propia ventana de inicio de sesión. La app nunca ve tu contraseña.',
		connect: 'Conectar',
		reconnect: 'Volver a conectar',
		sync: 'Sincronizar',
		disconnect: 'Desconectar en este equipo',
		disconnectTitle: (store: string) => `¿Desconectar ${store} en este equipo?`,
		disconnectBody:
			'Se borran la sesión guardada y el perfil de inicio de sesión de esta tienda. Los datos ya sincronizados se quedan hasta que los borres en Ajustes. No cierra la sesión en otros dispositivos.',
		cancel: 'Cancelar',
		connected: 'Conectada',
		disconnected: 'Sin conectar',
		account: (name: string) => `Cuenta: ${name}`,
		accountId: (id: string) => `Cuenta con ID ${id}`,
		lastSync: (d: string) => `Última sincronización: ${d}`,
		working: 'Trabajando…',
		waitingLogin: 'Esperando a que inicies sesión en la ventana de la tienda…',
		accountChanged:
			'Has conectado otra cuenta. Los datos de la cuenta anterior se guardan aparte y no se mezclan.',
		otherAccountData: 'Hay datos de otra cuenta de esta tienda guardados aparte.',
		cancelled: 'Inicio de sesión cancelado.',
		syncDone: 'Sincronización terminada. Revisa el estado de cada parte abajo.',
		nextStep: 'Siguiente paso: Revisar limpieza',
		capabilitiesLabel: (store: string) => `Qué puede hacer la app con ${store}`
	},
	review: {
		intro:
			'Cada deseado aparece junto a lo que ya tienes y que podría cubrirlo, con el motivo. Nada se marca solo: decides tú.',
		filters: {
			to_review: 'Por revisar',
			kept: 'Mantenidos',
			no_match: 'Sin coincidencia',
			repeated: 'Repetidos entre tiendas',
			all: 'Todos'
		},
		filtersLabel: 'Qué deseados mostrar',
		search: 'Buscar por título',
		wanted: 'Deseado',
		held: 'Ya tienes',
		empty: 'No hay deseados en este grupo',
		emptyBody: 'Prueba con otro grupo o sincroniza tus tiendas en Conexiones.',
		noWishes: 'Todavía no hay wishlists sincronizadas',
		same: 'Es el mismo',
		different: 'Son diferentes',
		undo: 'Deshacer mi decisión',
		keep: 'Mantener en esta wishlist',
		kept: 'Lo mantienes en esta wishlist',
		unkeep: 'Dejar de mantener',
		repeatedIn: (where: string) => `También lo deseas en ${where}, sin tenerlo en ninguna tienda.`,
		editionSuperior:
			'Tienes una edición superior. Comprueba qué incluye antes de quitar el deseado.',
		platformUnknown: 'No se sabe si lo que tienes es para Windows.',
		familyOwners: (n: number) =>
			n === 1 ? 'compartido por 1 persona' : `compartido por ${n} personas`,
		dataFrom: (d: string) => `Datos del ${d}`,
		check: 'Comprueba edición, plataforma y tipo de acceso antes de quitarlo.',
		decideFailed: 'No se pudo guardar la decisión.',
		outcome: {
			exact: 'Mismo producto',
			candidate: 'Posible coincidencia',
			unresolved: 'Sin resolver',
			not_covered: 'No lo cubre',
			family_excluded: 'En tu familia de Steam: no cuenta salvo que lo actives en Ajustes'
		},
		evidence: {
			store_id: 'mismo ID de tienda',
			external_map: 'coinciden en el catálogo de las tiendas',
			manual: 'lo decidiste tú',
			bundle_component: 'incluido en un pack con su contenido acreditado',
			title_candidate: 'solo coincide el título'
		},
		reasons: {
			different_kind: 'es otro tipo de producto (DLC, banda sonora, demo o vídeo)',
			remaster: 'uno es remaster o remake del otro',
			wanted_larger_edition: 'deseas una edición mayor que la que tienes',
			editions_differ: 'las ediciones no coinciden',
			platform_not_pc: 'lo que tienes no es para PC',
			not_permanent: 'tu acceso no es permanente',
			access_unknown: 'no se sabe qué tipo de acceso tienes',
			family_excluded: 'el acceso familiar no cuenta por defecto',
			unnamed_wishlist: 'el deseado no tiene nombre',
			manually_different: 'marcaste que son diferentes',
			tool_not_a_game: 'es una herramienta, no el juego',
			different_title: 'el título es distinto'
		},
		access: {
			owned: 'en tu biblioteca',
			family_shared: 'acceso familiar',
			temporary: 'acceso temporal',
			unknown: 'acceso sin clasificar'
		}
	},
	batch: {
		select: 'Incluir en el lote',
		selectLabel: (title: string) => `Incluir «${title}» en el lote`,
		selectExact: (n: number) =>
			n === 1 ? 'Seleccionar la coincidencia exacta' : `Seleccionar las ${n} coincidencias exactas`,
		notCovered:
			'Solo se puede quitar si tienes el mismo producto. Si es una posible coincidencia y lo es, pulsa «Es el mismo».',
		evidence: (store: string, why: string) => `Ya lo tienes en ${store}: ${why}`,
		selected: (n: number) => (n === 1 ? '1 deseado seleccionado' : `${n} deseados seleccionados`),
		reviewBatch: 'Revisar y quitar',
		hint: 'Marca los deseados que quieras quitar.',
		hintMore:
			'Solo se pueden marcar los que tienen evidencia firme. Nada se quita hasta que confirmes el lote.',
		clearSelection: 'Quitar selección',
		summary: (parts: string) => `Quitar ${parts}`,
		part: (n: number, store: string) => `${n} ${n === 1 ? 'deseado' : 'deseados'} de ${store}`,
		account: (store: string, id: string) => `Cuenta de ${store}: ${id}`,
		dialogTitle: 'Revisa lo que se va a quitar',
		dialogBody:
			'Se quitan de tu wishlist en la tienda y la app comprueba cada uno leyendo la wishlist otra vez. Quitar un deseado no borra ninguna licencia. La app no lo deshace: para recuperarlo, vuelve a añadirlo en la tienda.',
		confirm: (n: number) => (n === 1 ? 'Quitar 1 deseado ahora' : `Quitar ${n} deseados ahora`),
		sending: 'Quitando y comprobando…',
		cancel: 'Cancelar',
		created: 'Lote confirmado. Sigue en Resultados.',
		errors: {
			empty: 'No hay ninguna entrada seleccionada.',
			store_unsupported: 'Esta tienda no tiene wishlist en esta versión.',
			account_not_active: 'La cuenta cambió desde que se revisó. Vuelve a revisar.',
			not_in_wishlist: 'Alguna entrada ya no está en la wishlist. Sincroniza y vuelve a revisar.',
			kept: 'Alguna entrada está marcada para mantener.',
			not_covered:
				'Alguna entrada ya no tiene una coincidencia exacta. Revísala antes de quitarla.',
			not_connected: 'La tienda no está conectada. Conéctala en Conexiones.',
			needs_reauth: 'La sesión caducó. Vuelve a conectar la tienda en Conexiones.',
			plan_changed:
				'La wishlist cambió desde la revisión. El lote se ha anulado: vuelve a revisarlo.',
			bad_state: 'Este lote ya no admite esa acción.',
			internal: 'Algo falló dentro de la app. Vuelve a intentarlo; si se repite, reinicia la app.'
		}
	},
	results: {
		intro:
			'Cada lote confirmado, con el resultado de cada entrada. Una entrada solo cuenta como quitada cuando la app vuelve a leer la wishlist y ya no está.',
		empty: 'Todavía no hay lotes',
		emptyBody: 'Crea uno desde Revisar limpieza.',
		created: (d: string) => `Creado el ${d}`,
		reread: 'Volver a leer',
		sendHint:
			'El envío se interrumpió. La app quita lo pendiente en la tienda y lo comprueba leyendo la wishlist otra vez.',
		sendFailed: 'No se pudo enviar el lote',
		rereadHint: 'Sincroniza la wishlist de esta tienda y comprueba cada entrada.',
		discard: 'Descartar',
		cancel: 'Cancelar lo que falta',
		retry: 'Reintentar las que fallaron',
		resume: 'Enviar lo pendiente',
		openStore: 'Abrir en la tienda',
		state: {
			draft: 'Sin confirmar',
			confirmed: 'Confirmado: falta enviar algunas entradas',
			done: 'Terminado y comprobado',
			invalidated: 'Anulado: algo cambió desde la revisión'
		},
		op: {
			pending: 'Pendiente: quítalo en la tienda y vuelve a leer',
			pendingSend: 'Pendiente de enviar',
			already_in_desired_state: 'Ya no está en la wishlist (comprobado al volver a leer)',
			applied: 'Quitado y comprobado',
			failed: 'Falló: la tienda no aceptó el cambio. Puedes reintentarlo',
			needs_reauth: 'La sesión caducó: vuelve a conectar',
			not_supported: 'La tienda no permite hacerlo desde la app',
			unverified: 'Sin comprobar: la tienda no confirmó el cambio. Puedes reintentarlo',
			cancelled_not_sent: 'Cancelado antes de enviarse'
		},
		rereadFailed: 'No se pudo volver a leer la wishlist.',
		actionFailed: 'No se pudo hacer.'
	},
	writeTest: {
		title: 'Prueba de escritura en una wishlist (validación)',
		intro:
			'Cambia de verdad tu wishlist: añade o quita un solo producto que elijas y vuelve a leer la lista para comprobarlo. Úsala solo con un producto de prueba. Quitar un deseado no borra ninguna licencia.',
		productId: 'ID del producto (appid de Steam o ID de GOG)',
		action: 'Acción',
		remove: 'Quitar',
		add: 'Añadir',
		run: 'Ejecutar una prueba',
		confirmTitle: '¿Cambiar tu wishlist real?',
		confirmBody: (action: string, id: string, store: string) =>
			`${action} el producto ${id} en tu wishlist de ${store}. Es un cambio real en tu cuenta.`,
		confirm: 'Sí, hacer el cambio',
		result: (status: string) => `Resultado: ${status}`,
		failed: (why: string) => `No se ejecutó: ${why}`
	},
	capabilities: {
		library: 'Biblioteca',
		family: 'Familia de Steam',
		wishlist: 'Wishlist',
		wishlist_write: 'Quitar y añadir deseados'
	},
	capabilityCard: {
		complete: 'Al día',
		partial: 'Lectura incompleta',
		failed: (why: string) => `Falló: ${why}`,
		dataFrom: (d: string) => `datos del ${d}`,
		units: { library: 'juegos', family: 'compartidos', wishlist: 'deseados' }
	},
	capabilityState: {
		not_synced: 'Sin sincronizar',
		complete: (n: number, d: string) =>
			`Al día: ${n} ${n === 1 ? 'elemento' : 'elementos'}, datos del ${d}`,
		partial: 'Lectura incompleta: se guardó lo que llegó',
		failedWithCopy: (why: string, d: string) => `Falló (${why}). Se muestran los datos del ${d}`,
		failedNoCopy: (why: string) => `Falló (${why}). Todavía no hay datos`,
		unavailable: 'No disponible en esta versión',
		pending: 'Pendiente: aún no se sincroniza',
		not_validated: 'Sin validar: de momento se hace en la web de la tienda',
		validated: 'Desde la app, tras confirmar un lote'
	},
	errors: {
		needs_reauth: 'la sesión caducó, vuelve a conectar',
		private: 'la lista es privada o la tienda no da permiso',
		network: 'sin red o la tienda no responde',
		provider: 'la tienda devolvió un error',
		shape: 'la tienda respondió con un formato que la app no reconoce',
		incomplete: 'la lectura quedó a medias',
		unknown: 'motivo desconocido',
		app: 'algo falló dentro de la app'
	},
	kinds: {
		game: 'Juego',
		movie: 'Película',
		dlc: 'Contenido adicional',
		soundtrack: 'Banda sonora',
		demo: 'Demo',
		tool: 'Herramienta',
		application: 'Aplicación',
		video: 'Vídeo',
		unknown: 'Tipo desconocido'
	},
	stores: { steam: 'Steam', gog: 'GOG', epic: 'Epic' },
	notYet: {
		title: 'Esta pantalla aún no está disponible',
		body: 'Se construye en una fase posterior. Ahora mismo no hay ninguna función activa aquí.'
	},
	settings: {
		about: 'Acerca de',
		version: 'Versión',
		clearFailed:
			'No se pudieron borrar los datos. Cierra y vuelve a abrir la app e inténtalo otra vez.',
		openFolderFailed: 'No se pudo abrir la carpeta de datos.',
		statusError: 'No se pudo leer el estado de la aplicación.',
		diagnosticsLink: 'Diagnóstico de conexiones (soporte)',
		cleanup: 'Limpieza de wishlists',
		appearance: 'Apariencia',
		showCovers: 'Mostrar las carátulas de los juegos',
		showCoversHint:
			'Se cargan desde la web de cada tienda cuando las ves. Si una no carga, o lo desactivas, se muestran las iniciales del juego.',
		exportTitle: 'Exportar',
		includeFamily: 'Contar el acceso familiar de Steam como motivo para quitar un deseado',
		includeFamilyHint:
			'Desactivado por defecto. Si lo activas, aparece como un motivo aparte, «Disponible en tu familia de Steam», nunca como «ya lo tienes».',
		data: 'Datos locales',
		dataHint:
			'Todo se guarda solo en este equipo. Borrar los datos sincronizados no desconecta las tiendas: la próxima sincronización los vuelve a leer.',
		openFolder: 'Abrir carpeta de datos',
		clearCache: 'Borrar datos sincronizados',
		clearTitle: '¿Borrar los datos sincronizados?',
		clearBody:
			'Se borran las bibliotecas y wishlists guardadas en este equipo. Las conexiones siguen activas. No toca nada en las tiendas.',
		cleared: 'Datos sincronizados borrados.',
		updates: 'Actualizaciones',
		saved: 'Guardado.',
		failed: 'No se pudo guardar.'
	},
	updates: {
		unsigned:
			'Esta copia no se actualiza sola: se construyó sin la firma de actualizaciones. Las versiones publicadas buscan actualizaciones firmadas.',
		check: 'Buscar actualizaciones',
		checking: 'Buscando…',
		none: 'Tienes la última versión.',
		found: (v: string) => `Hay una versión nueva: ${v}. Tus datos y conexiones se conservan.`,
		install: (v: string) => `Instalar ${v} y reiniciar`,
		installing: 'Descargando e instalando…',
		progress: (pct: number) => `${pct} %`,
		failed:
			'No se pudo buscar o instalar la actualización. Tu versión actual sigue funcionando; inténtalo más tarde.'
	},
	diag: {
		title: 'Diagnóstico de conexiones',
		intro:
			'Herramientas de soporte. Solo muestran recuentos y estados, nunca claves ni cookies. Inicia sesión en la ventana de la tienda, no aquí.',
		gog: 'GOG',
		steam: 'Steam',
		epic: 'Epic',
		connect: 'Conectar',
		run: 'Leer biblioteca, wishlist y familia',
		expire: 'Forzar caducidad del token',
		shape: 'Ver forma de las respuestas',
		sync: 'Sincronizar y guardar en la base',
		types: 'Ver tipo y nombre de una muestra (tienda)',
		catalog: 'Ver forma del catálogo de la biblioteca',
		disconnect: 'Desconectar en este equipo',
		connected: 'Conectado',
		notConnected: 'Sin conectar',
		running: 'Trabajando…'
	}
} as const;
