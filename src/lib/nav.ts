import {
	IconBooks,
	IconHeart,
	IconLayoutKanban,
	IconListCheck,
	IconPlug,
	IconSettings
} from '@tabler/icons-svelte';
import { tr } from '#lib/i18n/index.ts';

/** Sections of the sidebar, in order. The folded rail shows the same items without groups. */
export const navGroups = [
	{
		label: tr.shell.sections.coleccion,
		items: [
			{ href: '/', label: tr.nav.biblioteca, icon: IconBooks },
			{ href: '/wishlists', label: tr.nav.wishlists, icon: IconHeart },
			{ href: '/partidas', label: tr.nav.partidas, icon: IconLayoutKanban }
		]
	},
	{
		label: tr.shell.sections.limpieza,
		items: [
			{
				href: '/limpieza',
				label: tr.nav.limpieza,
				short: tr.nav.limpiezaShort,
				icon: IconListCheck
			}
		]
	},
	{
		label: tr.shell.sections.cuenta,
		items: [
			{ href: '/conexiones', label: tr.nav.conexiones, icon: IconPlug },
			{ href: '/ajustes', label: tr.nav.ajustes, icon: IconSettings }
		]
	}
] as const;

export type NavHref = (typeof navGroups)[number]['items'][number]['href'];
