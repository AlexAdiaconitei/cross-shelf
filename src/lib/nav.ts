import {
	IconBooks,
	IconHeart,
	IconHistory,
	IconListCheck,
	IconPlug,
	IconSettings
} from '@tabler/icons-svelte';
import { es } from '#lib/strings.ts';

/** Sections of the sidebar, in order. The folded rail shows the same items without groups. */
export const navGroups = [
	{
		label: es.shell.sections.coleccion,
		items: [
			{ href: '/', label: es.nav.biblioteca, icon: IconBooks },
			{ href: '/wishlists', label: es.nav.wishlists, icon: IconHeart }
		]
	},
	{
		label: es.shell.sections.limpieza,
		items: [
			{ href: '/limpieza', label: es.nav.limpieza, short: 'Limpieza', icon: IconListCheck },
			{ href: '/resultados', label: es.nav.resultados, icon: IconHistory }
		]
	},
	{
		label: es.shell.sections.cuenta,
		items: [
			{ href: '/conexiones', label: es.nav.conexiones, icon: IconPlug },
			{ href: '/ajustes', label: es.nav.ajustes, icon: IconSettings }
		]
	}
] as const;

export type NavHref = (typeof navGroups)[number]['items'][number]['href'];
