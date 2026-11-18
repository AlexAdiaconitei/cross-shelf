import { noFilters, type Filters } from './filter.ts';

// Filters live outside the page so they survive going to another view and coming back.
export const libraryUi = $state<{ filters: Filters; grouped: boolean }>({
	filters: { ...noFilters },
	grouped: true
});
