import { noWishFilters, type WishFilters } from './filter.ts';

// Filters live outside the page so they survive going to another view and coming back.
export const wishlistUi = $state<{ filters: WishFilters }>({ filters: { ...noWishFilters } });
