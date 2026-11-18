import { tv, type VariantProps } from 'tailwind-variants';

/** Variants live here, not in the components, so a `<a>` can look like a button too. */
export const button = tv({
	base: [
		'inline-flex min-h-9 cursor-pointer items-center justify-center gap-2 rounded-control border px-3',
		'text-sm font-medium whitespace-nowrap transition-colors duration-150',
		'disabled:cursor-not-allowed disabled:opacity-55'
	],
	variants: {
		tone: {
			primary: 'border-accent bg-accent text-on-accent hover:brightness-110',
			quiet: 'border-border bg-transparent text-text hover:bg-raised',
			ghost: 'border-transparent bg-transparent text-muted hover:bg-raised hover:text-text',
			danger: 'border-danger bg-transparent text-danger hover:bg-raised'
		},
		size: {
			md: '',
			sm: 'min-h-7.5 px-2.5 text-xs',
			icon: 'size-9 px-0'
		}
	},
	defaultVariants: { tone: 'quiet', size: 'md' }
});

export type ButtonVariants = VariantProps<typeof button>;

export const input = tv({
	base: [
		'min-h-10 w-full rounded-control border border-border bg-page px-3 text-sm text-text',
		'placeholder:text-muted aria-invalid:border-danger'
	]
});

export const panel = 'min-w-0 overflow-hidden rounded-panel border border-border bg-panel';
