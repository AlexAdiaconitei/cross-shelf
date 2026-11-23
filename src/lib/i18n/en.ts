// English copy, the default language. Same shape as `es.ts`, checked by the `Strings` type.
import type { Strings } from './es.ts';

export const en: Strings = {
	landing: {
		pageTitle: 'CrossShelf — All your games. One library.',
		description:
			'Your Steam, GOG and Epic libraries in one Windows app. Browse your collection, search for games and see where you own them. Free and open source.',
		skip: 'Skip to content',
		navPreview: 'The app',
		navHow: 'How it works',
		navAbout: 'The creator',
		navLabel: 'Main navigation',
		languageLabel: 'Page language',
		heroTitle: 'All your games.\nOne library.',
		heroBody:
			'Bring your Steam, GOG and Epic libraries together. Browse your whole collection, find any game and see where you own it.',
		download: 'Download for Windows',
		viewDownloads: 'View Windows downloads',
		releaseNotes: 'Latest release notes',
		releaseFallback: 'Windows 10 / 11 · 64-bit · Downloads on GitHub',
		windows: 'Windows 10 / 11 · 64-bit',
		free: 'Free & open source',
		heroAlt:
			'CrossShelf demo library with real games and the sheet of Cyberpunk 2077: its state in the Play log, a note and the hours played in each store.',
		example: 'Real games · simulated library and accounts',
		exampleBanner: 'DEMO DATA · REAL GAMES · SIMULATED LIBRARY',
		artworkDisclaimer:
			'Demo library: accounts, ownership and wishlists are simulated. Game artwork and trademarks belong to their respective owners. CrossShelf is not affiliated with or endorsed by the stores, developers or publishers shown.',
		storesLabel: 'One library. Three stores.',
		previewTitle: 'See your whole shelf.',
		previewBody:
			'A game gets one row, even when you own it in more than one store. The store tabs tell you where it is yours, shared by family, or still on a wishlist.',
		viewLibrary: 'Library',
		viewReview: 'Wishlist review',
		viewConnections: 'Connections',
		viewPlays: 'Play log',
		previewLabel: 'Choose an app screenshot',
		libraryAlt:
			'CrossShelf demo library: real games grouped across Steam, GOG and Epic, with simulated ownership and Steam family access shown separately.',
		reviewAlt:
			'Example wishlist review: Hades on a simulated Steam wishlist compared with an Epic copy, with evidence and manual review controls.',
		connectionsAlt:
			'Example connections screen with fictional accounts, library counts and the capabilities of each store.',
		libraryCaption: 'Search once. See every store, your own copies and Steam family access.',
		reviewCaption: 'Compare the wish with your copy. Check the edition and choose what to keep.',
		connectionsCaption: 'Connect on each store’s official sign-in page, then sync from the app.',
		playsAlt:
			'Example Play log board: real games in Playing, Backlog, Paused, Completed and Dropped columns, with simulated hours and a tray of suggestions above.',
		playsCaption: 'Board, list or diary. Drag each game to its column; hours come from Steam.',
		owned: 'Owned',
		family: 'Family access',
		wished: 'Wishlisted',
		cleanupTitle: 'Your wishlist. Your call.',
		cleanupBody:
			'A similar title is a reason to look closer. Check the game, edition and platform side by side, then mark only the wishes you want to remove.',
		cleanupPoint: 'Nothing leaves a wishlist until you review and confirm the batch.',
		cleanupAlt:
			'CrossShelf demo review showing real games with simulated wishlist entries, owned copies and the option to keep a wish.',
		playTitle: 'Know what you are playing.',
		playBody:
			'Note what you are playing, what is waiting and what you have finished, whichever store it is in. The diary orders your games by the last time you played and adds up the week’s hours.',
		playPoint: 'Suggestions come from your play time, and nothing moves until you click.',
		diaryAlt:
			'Example diary: real games grouped by last session, with simulated hours, this week’s hours and states to confirm.',
		howTitle: 'From scattered to sorted.',
		howBody: 'No API keys or terminal. Install the app and use the accounts you already have.',
		stepConnect: 'Connect your stores',
		stepConnectBody:
			'Sign in on the official Steam, GOG and Epic pages. CrossShelf never asks for your password.',
		stepSync: 'Bring in your libraries',
		stepSyncBody:
			'Sync, search and filter your games. Steam family access is always labelled separately.',
		stepReview: 'Review before removing',
		stepReviewBody:
			'Choose the Steam and GOG wishes to remove, review the batch and confirm. The app checks each result.',
		localTitle: 'Your shelf lives on your PC.',
		localBody:
			'No CrossShelf account. No telemetry. Your library stays on your computer, and saved sessions are encrypted with Windows data protection.',
		localDetail:
			'Export your library and wishlists to CSV or JSON, and your plays and decisions to a file you can import on another computer.',
		faqTitle: 'Before you install.',
		faqStores: 'What works with each store?',
		faqStoresBody:
			'Read your Steam, GOG and Epic libraries, plus Steam family access. Read and clean Steam and GOG wishlists. For Epic wishlists, the app opens the store website; reading and removal are not available in the app. Hours played come from Steam.',
		faqFamily: 'Does family access count as owning a game?',
		faqFamilyBody:
			'Family access is labelled separately and excluded from cleanup by default. You can include it in Settings, where it remains a separate reason to review a wish.',
		faqWindows: 'What do I need to run it?',
		faqWindowsBody:
			'Windows 10 22H2 or Windows 11, 64-bit. The installer runs for your user without administrator permissions and downloads WebView2 if it is missing.',
		faqWarning: 'Why might Windows show a warning?',
		faqWarningBody:
			'The installer is not signed with a publisher certificate, so Windows SmartScreen may warn on first run. If you downloaded it from this project’s GitHub release, choose “More info”, then “Run anyway”.',
		faqPrice: 'Is CrossShelf free?',
		faqPriceBody:
			'Yes. CrossShelf is free and open source under the MIT licence. It does not install or launch games, buy anything, or transfer game licences.',
		aboutTitle: 'Behind CrossShelf.',
		footerCredit: 'Built by Alex Adiaconitei',
		source: 'Source on GitHub',
		license: 'MIT licence',
		privacy: 'No analytics or tracking cookies.',
		noScript:
			'This is the English version. Enable JavaScript to switch languages, choose screenshots and resolve the direct installer link.'
	},
	appName: 'CrossShelf',
	nav: {
		conexiones: 'Connections',
		biblioteca: 'Library',
		wishlists: 'Wishlists',
		limpieza: 'Review cleanup',
		limpiezaShort: 'Cleanup',
		resultados: 'Results',
		partidas: 'Play log',
		ajustes: 'Settings'
	},
	shell: {
		sections: { coleccion: 'Collection', limpieza: 'Cleanup', cuenta: 'Account' },
		openSidebar: 'Open the sidebar',
		foldSidebar: 'Fold the sidebar',
		stores: 'Stores',
		wishes: 'wishes',
		storeFilter: 'Filter the library by store',
		syncFailed: 'the last sync failed',
		toReview: (n: number) => (n === 1 ? '1 wish to review' : `${n} wishes to review`)
	},
	plays: {
		states: {
			playing: 'Playing',
			backlog: 'Backlog',
			paused: 'Paused',
			completed: 'Completed',
			dropped: 'Dropped'
		},
		stateHints: {
			playing: 'What you are playing now',
			backlog: 'Waiting for its turn',
			paused: 'Left for later',
			completed: 'Finished',
			dropped: 'Not going back to it'
		},
		tabs: { board: 'Board', list: 'List', diary: 'Diary' },
		tabsLabel: 'How to see Play log',
		add: 'Add a game from your library',
		addTip: 'Enter: to Backlog · Shift+Enter: to Playing',
		addNone: (q: string) => `Nothing in your library matches «${q}» that is not here already.`,
		move: (title: string) => `Move «${title}»`,
		remove: 'Remove from Play log',
		keyHint: 'Alt + left and right arrows to change column.',
		dropHint: 'Drag a card here.',
		empty: 'No games here yet',
		emptyBody:
			'Note down what you are playing, what is waiting and what you have finished. Search for a game above or open its sheet in the Library and pick a state. It does not matter which store it is in.',
		started: (ago: string) => `Started ${ago}`,
		finished: (ago: string) => `Finished ${ago}`,
		inState: (state: string, ago: string) => `${state} · ${ago}`,
		notInLibrary: 'No longer in your library',
		noTime: '—',
		minutes: (n: number) => `${n} min`,
		hours: (h: string) => `${h} h`,
		today: 'today',
		suggestions: (n: number) => (n === 1 ? '1 suggestion' : `${n} suggestions`),
		suggestionsFrom: 'from your play time',
		toState: (state: string) => `To ${state}`,
		dismiss: (title: string) => `Dismiss the suggestion for ${title}`,
		reasons: {
			played_recently: (ago: string) => `Played ${ago}`,
			not_played: (ago: string) => `Not played since ${ago}`
		},
		diary: {
			groups: {
				week: 'This week',
				month: 'This month',
				older: 'A while ago',
				none: 'No session date'
			},
			weekHours: 'played this week',
			inProgress: 'in progress',
			toConfirm: 'to confirm',
			source: 'Steam gives the last session; Epic and GOG, only the total.',
			lastSession: 'Last session',
			total: 'Total',
			week: 'Week',
			state: 'State',
			stateOf: (title: string) => `State of ${title}`,
			noState: 'No state',
			confirm: (state: string, title: string) => `Confirm ${state} for ${title}`,
			guess: (state: string) => `${state}?`,
			empty: 'The diary is empty',
			emptyBody:
				'Games with a play or with hours played show up here, by the last time you played them. Sync your stores to bring in the hours.'
		},
		sheet: {
			time: 'Time',
			total: 'Total',
			lastSession: 'Last session',
			byStore: 'By store',
			notGiven: (store: string) => `${store} does not give it`,
			noHours: (store: string) => `${store}: no hours yet`,
			loading: 'Loading…',
			error: (store: string) => `${store} could not be reached`,
			hltb: 'Search on HowLongToBeat',
			hltbHint: 'Opens its website in the browser. The app asks it nothing.',
			play: 'Play',
			noPlay: 'Not in Play log',
			note: 'Note',
			notePlaceholder: 'Where you left off, what is left…',
			noteLeft: (n: number) => `${n} characters left`,
			noteTooLong: 'The note is too long.'
		},
		loadError: 'Your play log could not be read.',
		actionFailed: 'It could not be saved. Try again.'
	},
	shelf: {
		storesOf: (title: string) => `Stores of ${title}`,
		slot: {
			own: 'yours',
			temp: 'temporary access',
			family: 'family access',
			unknown: 'unclassified access',
			wish: 'on your wishlist',
			none: 'not there'
		},
		grouped: 'By game',
		separate: 'By store',
		groupingLabel: 'How to group the library',
		games: 'games',
		family: 'family',
		wishedElsewhere: 'wished in another store',
		onlyFamily: (names: string[], n: number) =>
			names.length
				? `Family · from ${names.join(', ')}`
				: n === 0
					? 'Family'
					: n === 1
						? 'Family · from 1 person'
						: `Family · from ${n} people`,
		title: 'Title',
		access: 'Access',
		stores: 'Stores',
		wishes: 'wishes',
		detail: 'Game sheet',
		closeDetail: 'Close the sheet',
		detailOf: (title: string) => `Sheet of ${title}`,
		products: 'Products per store',
		open: 'Open',
		openIn: (store: string) => `Open in ${store}`,
		reviewThis: 'Review this wish',
		sharedWith: (names: string[], n: number) =>
			names.length
				? `Shared by ${names.join(', ')}`
				: n === 1
					? 'Shared by 1 person'
					: `Shared by ${n} people`,
		dataInfo: 'How recent this data is',
		groupInfo: 'How games are grouped',
		familyNote: (n: number) =>
			`Shared by ${n === 1 ? '1 person' : `${n} people`} in your Steam family. It does not count as yours in the cleanup unless you turn it on in Settings.`,
		inWishlist: 'on your wishlist',
		groupNote:
			'Grouped by title only to show them together. It is not a matching decision: that is reviewed in Cleanup.'
	},
	library: {
		search: 'Search by title',
		store: 'Store',
		allStores: 'All stores',
		scope: 'Access',
		scopeAll: 'All',
		scopeOwn: 'Owned',
		scopeFamily: 'With family access',
		scopeFamilyOnly: 'Family access only',
		kind: 'Type',
		allKinds: 'All types',
		clear: 'Clear filters',
		noData: 'Nothing synced yet',
		noDataBody: 'Connect a store and sync it to see your games here.',
		noMatch: 'No product matches the filters',
		loadError: 'The local database could not be read.',
		showing: (shown: number, total: number) => `Showing ${shown} of ${total}`,
		perStore: (store: string, n: number) => `${store}: ${n}`,
		familyCount: (n: number) => `With family access: ${n}`,
		untitled: (id: string) => `Untitled (ID ${id})`,
		owned: 'In your library',
		familyShared: 'Family access',
		temporary: 'Temporary access',
		unknownAccess: 'Unclassified access',
		owners: (n: number) => (n === 1 ? 'from 1 person' : `from ${n} people`),
		dataFrom: (store: string, date: string) => `${store}: data from ${date}`,
		lastFailed: (store: string, date: string) =>
			`${store}: the last sync failed. Showing data from ${date}.`,
		neverComplete: (store: string) => `${store}: no complete sync yet.`,
		syncFailed: 'The sync failed and there is no copy to show yet.',
		syncFailedBody: 'Check the state of each store in Connections and try again.',
		tabs: {
			all: 'All',
			steam: 'Steam',
			steam_family: 'Steam family',
			gog: 'GOG',
			epic: 'Epic'
		},
		tabsLabel: 'Library to show',
		export: 'Export library'
	},
	wishlist: {
		search: 'Search by title',
		status: 'Show',
		statusAll: 'All',
		statusHeldHere: 'Already in your library of that store',
		statusUnconfirmed: 'Not confirmed on the website',
		statusUntitled: 'Untitled',
		sort: 'Sort by',
		sortPriority: 'Priority',
		sortAdded: 'Date added',
		sortTitle: 'Title',
		noData: 'No wishlist synced yet',
		noDataBody: 'Connect Steam or GOG in Connections and sync.',
		syncFailed: 'No wishlist could be read',
		syncFailedBody:
			'The store did not answer or the session expired. Check Connections; this is not an empty list.',
		confirmedEmpty: 'Your wishlists are empty',
		confirmedEmptyBody: 'The stores confirmed it in the last sync.',
		noMatch: 'No wish matches the filters',
		heldHere: 'Already in your library of this store',
		unconfirmed: 'The store returns it, but its website does not show it',
		priority: (p: number) => `Priority ${p}`,
		added: (d: string) => `Added on ${d}`,
		openStore: 'Open in the store',
		openEpic: 'Open your Epic wishlist on its website',
		epicUnavailable:
			'The Epic wishlist is not available in this version: the store does not let it be read reliably. You can see it on its website.',
		writeNotValidated: (stores: string) =>
			`In ${stores}, removing from the app is not validated yet. Use «Open in the store» and sync again when you are done.`,
		writeValidated:
			'To remove wishes you already own in another store, go to Review cleanup: the app removes them for you and checks each one.',
		export: 'Export wishlists'
	},
	export: {
		csv: 'CSV',
		json: 'JSON',
		done: (n: number) => (n === 1 ? 'Exported 1 row.' : `Exported ${n} rows.`),
		failed: (why: string) => `Could not export: ${why}.`,
		errors: {
			bad_path: 'the path is not valid',
			bad_extension: 'the extension does not match the format',
			io: 'the file could not be written'
		}
	},
	connections: {
		intro: 'Connect each store in its own sign-in window. The app never sees your password.',
		connect: 'Connect',
		reconnect: 'Reconnect',
		sync: 'Sync',
		disconnect: 'Disconnect on this computer',
		disconnectTitle: (store: string) => `Disconnect ${store} on this computer?`,
		disconnectBody:
			'The saved session and the sign-in profile of this store are deleted. Data already synced stays until you delete it in Settings. It does not sign you out on other devices.',
		cancel: 'Cancel',
		connected: 'Connected',
		disconnected: 'Not connected',
		account: (name: string) => `Account: ${name}`,
		accountId: (id: string) => `Account with ID ${id}`,
		lastSync: (d: string) => `Last sync: ${d}`,
		working: 'Working…',
		waitingLogin: 'Waiting for you to sign in in the store window…',
		accountChanged:
			'You connected a different account. The data of the previous account is kept apart and never mixed.',
		otherAccountData: 'Data of another account of this store is kept apart.',
		cancelled: 'Sign-in cancelled.',
		syncDone: 'Sync finished. Check the state of each part below.',
		nextStep: 'Next step: Review cleanup',
		capabilitiesLabel: (store: string) => `What the app can do with ${store}`
	},
	review: {
		intro:
			'Each wish appears next to what you already own that might cover it, with the reason. Nothing is marked on its own: you decide.',
		filters: {
			to_review: 'To review',
			kept: 'Kept',
			no_match: 'No match',
			repeated: 'Repeated across stores',
			all: 'All'
		},
		filtersLabel: 'Which wishes to show',
		search: 'Search by title',
		wanted: 'Wished',
		held: 'You own',
		empty: 'No wishes in this group',
		emptyBody: 'Try another group or sync your stores in Connections.',
		noWishes: 'No wishlist synced yet',
		same: 'It is the same',
		different: 'They are different',
		undo: 'Undo my decision',
		keep: 'Keep on this wishlist',
		kept: 'You keep it on this wishlist',
		unkeep: 'Stop keeping',
		repeatedIn: (where: string) => `You also wish for it in ${where}, without owning it anywhere.`,
		editionSuperior: 'You own a bigger edition. Check what it includes before removing the wish.',
		platformUnknown: 'It is not known whether what you own runs on Windows.',
		familyOwners: (n: number) => (n === 1 ? 'shared by 1 person' : `shared by ${n} people`),
		dataFrom: (d: string) => `Data from ${d}`,
		check: 'Check the edition, platform and kind of access before removing it.',
		decideFailed: 'The decision could not be saved.',
		outcome: {
			exact: 'Same product',
			candidate: 'Possible match',
			unresolved: 'Unresolved',
			not_covered: 'Does not cover it',
			family_excluded: 'In your Steam family: it does not count unless you turn it on in Settings'
		},
		evidence: {
			store_id: 'same store ID',
			external_map: 'they match in the store catalogs',
			manual: 'you decided it',
			bundle_component: 'included in a bundle with its contents confirmed',
			title_candidate: 'only the title matches'
		},
		reasons: {
			different_kind: 'it is another kind of product (DLC, soundtrack, demo or video)',
			remaster: 'one is a remaster or remake of the other',
			wanted_larger_edition: 'you wish for a bigger edition than the one you own',
			editions_differ: 'the editions do not match',
			platform_not_pc: 'what you own is not for PC',
			not_permanent: 'your access is not permanent',
			access_unknown: 'the kind of access you have is not known',
			family_excluded: 'family access does not count by default',
			unnamed_wishlist: 'the wish has no name',
			manually_different: 'you marked them as different',
			tool_not_a_game: 'it is a tool, not the game',
			different_title: 'the title is different'
		},
		access: {
			owned: 'in your library',
			family_shared: 'family access',
			temporary: 'temporary access',
			unknown: 'unclassified access'
		}
	},
	batch: {
		select: 'Include in the batch',
		selectLabel: (title: string) => `Include «${title}» in the batch`,
		selectExact: (n: number) =>
			n === 1 ? 'Select the exact match' : `Select the ${n} exact matches`,
		notCovered:
			'It can only be removed if you own the same product. If it is a possible match and it really is, press «It is the same».',
		evidence: (store: string, why: string) => `You already own it in ${store}: ${why}`,
		selected: (n: number) => (n === 1 ? '1 wish selected' : `${n} wishes selected`),
		reviewBatch: 'Review and remove',
		hint: 'Tick the wishes you want to remove.',
		hintMore:
			'Only wishes with firm evidence can be ticked. Nothing is removed until you confirm the batch.',
		clearSelection: 'Clear selection',
		summary: (parts: string) => `Remove ${parts}`,
		part: (n: number, store: string) => `${n} ${n === 1 ? 'wish' : 'wishes'} from ${store}`,
		account: (store: string, id: string) => `${store} account: ${id}`,
		dialogTitle: 'Review what will be removed',
		dialogBody:
			'They are removed from your wishlist in the store and the app checks each one by reading the wishlist again. Removing a wish does not delete any licence. The app does not undo it: to get it back, add it again in the store.',
		confirm: (n: number) => (n === 1 ? 'Remove 1 wish now' : `Remove ${n} wishes now`),
		sending: 'Removing and checking…',
		cancel: 'Cancel',
		created: 'Batch confirmed. Follow it in Results.',
		errors: {
			empty: 'No entry is selected.',
			store_unsupported: 'This store has no wishlist in this version.',
			account_not_active: 'The account changed since the review. Review again.',
			not_in_wishlist: 'Some entry is no longer on the wishlist. Sync and review again.',
			kept: 'Some entry is marked to keep.',
			not_covered: 'Some entry no longer has an exact match. Review it before removing it.',
			not_connected: 'The store is not connected. Connect it in Connections.',
			needs_reauth: 'The session expired. Reconnect the store in Connections.',
			plan_changed: 'The wishlist changed since the review. The batch was voided: review it again.',
			bad_state: 'This batch no longer allows that action.',
			internal: 'Something failed inside the app. Try again; if it happens again, restart the app.'
		}
	},
	results: {
		intro:
			'Each confirmed batch, with the result of every entry. An entry only counts as removed when the app reads the wishlist again and it is gone.',
		empty: 'No batches yet',
		earlier: (n: number) => `Earlier batches (${n})`,
		emptyBody: 'Create one from Review cleanup.',
		created: (d: string) => `Created on ${d}`,
		reread: 'Read again',
		sendHint:
			'Sending was interrupted. The app removes what is pending in the store and checks it by reading the wishlist again.',
		sendFailed: 'The batch could not be sent',
		rereadHint: 'Syncs the wishlist of this store and checks each entry.',
		discard: 'Discard',
		cancel: 'Cancel what is left',
		retry: 'Retry the failed ones',
		resume: 'Send what is pending',
		openStore: 'Open in the store',
		state: {
			draft: 'Not confirmed',
			confirmed: 'Confirmed: some entries still to send',
			done: 'Finished and checked',
			invalidated: 'Voided: something changed since the review'
		},
		op: {
			pending: 'Pending: remove it in the store and read again',
			pendingSend: 'Pending to send',
			already_in_desired_state: 'No longer on the wishlist (checked by reading again)',
			applied: 'Removed and checked',
			failed: 'Failed: the store did not accept the change. You can retry',
			needs_reauth: 'The session expired: reconnect',
			not_supported: 'The store does not allow doing it from the app',
			unverified: 'Not checked: the store did not confirm the change. You can retry',
			cancelled_not_sent: 'Cancelled before being sent'
		},
		rereadFailed: 'The wishlist could not be read again.',
		actionFailed: 'It could not be done.'
	},
	writeTest: {
		title: 'Wishlist write test (validation)',
		intro:
			'Really changes your wishlist: adds or removes one product of your choice and reads the list again to check it. Use it only with a test product. Removing a wish does not delete any licence.',
		productId: 'Product ID (Steam appid or GOG ID)',
		action: 'Action',
		remove: 'Remove',
		add: 'Add',
		run: 'Run a test',
		confirmTitle: 'Change your real wishlist?',
		confirmBody: (action: string, id: string, store: string) =>
			`${action} product ${id} on your ${store} wishlist. It is a real change to your account.`,
		confirm: 'Yes, make the change',
		result: (status: string) => `Result: ${status}`,
		failed: (why: string) => `Not run: ${why}`
	},
	capabilities: {
		library: 'Library',
		family: 'Steam family',
		wishlist: 'Wishlist',
		wishlist_write: 'Remove and add wishes'
	},
	capabilityCard: {
		complete: 'Up to date',
		partial: 'Incomplete read',
		failed: (why: string) => `Failed: ${why}`,
		dataFrom: (d: string) => `data from ${d}`,
		units: { library: 'games', family: 'shared', wishlist: 'wishes' }
	},
	capabilityState: {
		not_synced: 'Not synced',
		complete: (n: number, d: string) =>
			`Up to date: ${n} ${n === 1 ? 'item' : 'items'}, data from ${d}`,
		partial: 'Incomplete read: what arrived was saved',
		failedWithCopy: (why: string, d: string) => `Failed (${why}). Showing data from ${d}`,
		failedNoCopy: (why: string) => `Failed (${why}). No data yet`,
		unavailable: 'Not available in this version',
		pending: 'Pending: not synced yet',
		not_validated: 'Not validated: for now it is done on the store website',
		validated: 'From the app, after confirming a batch'
	},
	errors: {
		needs_reauth: 'the session expired, reconnect',
		private: 'the list is private or the store does not allow it',
		network: 'no network or the store does not answer',
		provider: 'the store returned an error',
		shape: 'the store answered in a format the app does not recognise',
		incomplete: 'the read was left halfway',
		unknown: 'unknown reason',
		app: 'something failed inside the app',
		in_progress: 'a sign-in window for this store is already open',
		not_connected: 'the store is not connected'
	},
	kinds: {
		game: 'Game',
		movie: 'Movie',
		dlc: 'DLC',
		soundtrack: 'Soundtrack',
		demo: 'Demo',
		tool: 'Tool',
		application: 'Application',
		video: 'Video',
		unknown: 'Unknown type'
	},
	stores: { steam: 'Steam', gog: 'GOG', epic: 'Epic' },
	notYet: {
		title: 'This screen is not available yet',
		body: 'It is built in a later phase. There is no active feature here right now.'
	},
	settings: {
		mine: {
			title: 'My data',
			hint: 'Your plays and notes, matching decisions, «Keep», dismissed suggestions, play time history and settings: what the stores cannot give back. It does not carry your connections or cleanup batches.',
			export: 'Export my data',
			import: 'Import my data',
			exported: 'Your data was exported.',
			importTitle: 'Replace your data with this file?',
			importBody: (date: string) =>
				`File exported on ${date}. It replaces your current data, it does not add to it. A copy of what you have now is saved first in the backups folder. Connections and the synced library do not change.`,
			counts: {
				plays: (n: number) => (n === 1 ? '1 play' : `${n} plays`),
				relations: (n: number) => (n === 1 ? '1 matching decision' : `${n} matching decisions`),
				keeps: (n: number) => (n === 1 ? '1 wish to keep' : `${n} wishes to keep`),
				dismissed: (n: number) =>
					n === 1 ? '1 dismissed suggestion' : `${n} dismissed suggestions`,
				snapshots: (n: number) => (n === 1 ? '1 play time snapshot' : `${n} play time snapshots`),
				settings: (n: number) => (n === 1 ? '1 setting' : `${n} settings`)
			},
			confirm: 'Replace my data',
			imported: 'Your data was imported.',
			errors: {
				not_ours: 'This file is not a CrossShelf data export.',
				too_new: 'This file comes from a newer version of CrossShelf. Update the app first.',
				bad_path: 'The path is not valid.',
				bad_extension: 'The file must be .json.',
				io: 'The file could not be read or written.',
				too_big: 'The file is too big to be a CrossShelf export.',
				internal: 'Something failed inside the app. Your data did not change.'
			}
		},
		about: 'About',
		version: 'Version',
		clearFailed: 'The data could not be deleted. Close and reopen the app and try again.',
		openFolderFailed: 'The data folder could not be opened.',
		statusError: 'The application state could not be read.',
		diagnosticsLink: 'Connection diagnostics (support)',
		cleanup: 'Wishlist cleanup',
		appearance: 'Appearance',
		language: 'Language',
		languageHint: 'The app restarts its window to apply the change.',
		showCovers: 'Show game covers',
		showCoversHint:
			'They load from each store website when you see them. If one does not load, or you turn this off, the game initials are shown.',
		exportTitle: 'Export',
		includeFamily: 'Count Steam family access as a reason to remove a wish',
		includeFamilyHint:
			'Off by default. If you turn it on, it appears as a separate reason, «Available in your Steam family», never as «you already own it».',
		data: 'Local data',
		dataHint:
			'Everything is saved only on this computer. Deleting the synced data does not disconnect the stores: the next sync reads it again.',
		openFolder: 'Open data folder',
		clearCache: 'Delete synced data',
		clearTitle: 'Delete the synced data?',
		clearBody:
			'The libraries and wishlists saved on this computer are deleted. Connections stay active. Nothing changes in the stores.',
		cleared: 'Synced data deleted.',
		updates: 'Updates',
		saved: 'Saved.',
		failed: 'Could not save.'
	},
	updates: {
		unsigned:
			'This copy does not update itself: it was built without the update signature. Published versions look for signed updates.',
		check: 'Check for updates',
		checking: 'Checking…',
		none: 'You have the latest version.',
		found: (v: string) => `A new version is available: ${v}. Your data and connections are kept.`,
		install: (v: string) => `Install ${v} and restart`,
		installing: 'Downloading and installing…',
		progress: (pct: number) => `${pct} %`,
		failed:
			'The update could not be checked or installed. Your current version keeps working; try again later.'
	},
	diag: {
		title: 'Connection diagnostics',
		intro:
			'Support tools. They only show counts and states, never keys or cookies. Sign in in the store window, not here.',
		gog: 'GOG',
		steam: 'Steam',
		epic: 'Epic',
		connect: 'Connect',
		run: 'Read library, wishlist and family',
		expire: 'Force token expiry',
		shape: 'Show the shape of the responses',
		sync: 'Sync and save to the database',
		types: 'Show type and name of a sample (store)',
		catalog: 'Show the shape of the library catalog',
		playtime: 'Check play time (validation)',
		disconnect: 'Disconnect on this computer',
		connected: 'Connected',
		notConnected: 'Not connected',
		running: 'Working…'
	}
};
