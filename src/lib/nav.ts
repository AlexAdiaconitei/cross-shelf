import { es } from '#lib/strings.ts';

export const navItems = [
	{ href: '/', label: es.nav.conexiones },
	{ href: '/biblioteca', label: es.nav.biblioteca },
	{ href: '/wishlists', label: es.nav.wishlists },
	{ href: '/limpieza', label: es.nav.limpieza },
	{ href: '/resultados', label: es.nav.resultados },
	{ href: '/ajustes', label: es.nav.ajustes }
] as const;
