# Fintrack — Frontend PWA

Personal finance tracker. Vue 3 + TypeScript + Ionic Vue + Vite + Pinia + Vue Router +
vite-plugin-pwa. Mobile-first, portrait, dark. Every form control is a custom component —
**zero native `<select>`, `<input type="date">`, or `<dialog>` anywhere**.

## Run

```bash
npm install
npm run dev          # Vite dev server on :5173, proxies /api → localhost:8080
npm run build        # type-check + production build into dist/
npm run preview      # serve the built dist/
npm run icons        # regenerate public/icons/*.png from scripts/gen-icons.mjs
```

## Layout

```
public/
├── icon.svg                 # app mark (used by the manifest as a scalable icon)
└── icons/                   # 192/512 PNG icons, generated from the same artwork
scripts/gen-icons.mjs        # dependency-free PNG writer for those icons
src/
├── main.ts                  # Ionic + Vue bootstrap, light palette, app.css
├── App.vue                  # app shell: router outlet + fixed 5-tab bottom nav
├── router/index.ts          # routes + token guard (public: /login, /register)
├── theme/app.css            # design system: tokens, cards, rows, bars, charts, nav
├── types.ts                 # Paged/FinanceRow/SyncOp/overview/analysis/summary types
├── entityConfig.ts          # per-entity field maps, icons, accents, value keys
├── api/client.ts            # fetch wrapper (BASE = /api), token helpers, ApiError
├── services/offlineQueue.ts # IndexedDB queue (fintrack → sync_queue + meta)
├── stores/
│   ├── authStore.ts         # login / register / me / logout
│   ├── financeStore.ts      # 7 entity lists + summaries + overview + analysis + CRUD
│   └── syncStore.ts         # push queue, pull deltas, online/offline status
├── composables/useCountUp.ts
├── utils/money.ts           # currency formatting (compact / signed / percent)
├── utils/date.ts            # timezone-safe date + month helpers
├── components/ui/           # AppButton, AppInput, AppSelect, AppDatePicker,
│                            #   AppModal, SyncChip
└── pages/                   # Dashboard, Accounts, AccountDetail, Transactions,
                             #   Analytics, Settings, Login, Register
```

## Custom form components (`src/components/ui/`)

| Component | Replaces | Notes |
| --- | --- | --- |
| **AppSelect** | `<select>` | Closed = flat bordered button; open = teleported bottom sheet with a scrollable option list, selected row marked with a check. No native select. |
| **AppDatePicker** | `<input type="date">` | 3-column day/month/year wheel, `transform: translateY` positioning, +/- nudge buttons, Confirm/Cancel, backdrop dismiss + Esc. |
| **AppModal** | `<dialog>` / `alert()` | Teleport-to-body overlay, scroll-locked body, Esc + backdrop dismiss, `default`/`footer` slots, `sheet` mode, `busy` lock for destructive confirmations. |
| **AppInput** | text/number inputs | Fully skinned `<input>`, 16px font (no iOS zoom), optional label/hint/suffix, money and number variants. |
| **AppButton** | `ion-button` styling | Variants `primary`/`success`/`neutral`/`danger`, sizes `sm`/`md`/`lg`, block mode, disabled state. |
| **SyncChip** | — | Live sync status pill (online/offline/pushing/synced/error) with pending-change count; tap to sync now. |

## Routes

| Path | Screen |
| --- | --- |
| `/dashboard` | Net worth hero, month income/spend/saved, per-group totals, recent activity. |
| `/accounts` | One card per group (banks, assets, deposits, investments, cards) + allocation bar. |
| `/accounts/:entity` | List for one entity: search, sort, swipe-to-reveal Edit/Delete, per-record detail sheet, create/edit form. |
| `/accounts/:entity/new` | Same page with the create form already open. |
| `/transactions` | Unified income + expense timeline with type/category chips, month buckets, tallies. |
| `/analytics` | Net worth, savings rate, income mix, cash-flow chart, top expenses/sources, monthly spend, ROI, deposit maturities, card utilisation. Entity-type filter chips. |
| `/settings` | Account, sync controls, data inventory, about, sign out. |
| `/login`, `/register` | Auth (public). |

`/dashboard`, `/accounts`, `/transactions`, `/analytics`, `/settings` make up the bottom nav.
The router redirects `/` → `/dashboard` and guards everything except `/login` + `/register`,
honouring a `?redirect=` query.

## Data layer

- **Lists** — `GET /api/{entity}?page&perPage&search` returns
  `{ data, total, page, perPage }`. `financeStore` reads `data`.
- **Detail** — `GET /api/{entity}/:id` returns the raw record; the detail sheet renders
  every configured field plus any remaining server columns.
- **Summaries** — `GET /api/{entity}/summary`, stored per-entity so nothing collides.
- **Overview** — `GET /api/overview` → `net_worth`, `yearly_income`, `monthly_expense`,
  `total_gain`, `invested`, `roi`.
- **Analysis** — `GET /api/analysis?entity_types=…` → net-worth breakdown, yearly income
  by type, monthly expenses, ROI + per-type ROI, cash flow, top items, FD maturities,
  credit-card utilisation, savings rate.
- **CRUD** — `POST`/`PUT`/`DELETE /api/{entity}[/:id]`. A failed write is queued as a
  `SyncOp` and replayed on reconnect.
- Entities: `banks`, `assets`, `fixed-deposits`, `investments`, `credit-cards`, `incomes`,
  `expenses`.

## Offline sync

`services/offlineQueue.ts` keeps an IndexedDB queue (`fintrack` → `sync_queue` + `meta`)
with a localStorage fallback. Every op carries `id`, `entity` (snake_case),
`entity_id`, `op` (`create`/`update`/`delete`), `payload`, and `client_ts`.

`syncStore` pushes the queue on `POST /api/sync/push` and pulls with
`GET /api/sync/pull?since=<RFC3339>`; `server_ts` (an RFC3339 string) becomes the next
watermark. It reacts to `online`/`offline` events and records the last successful sync time.

## PWA

- Manifest: name "Fintrack", `theme_color #0f1115`, `display: standalone`,
  `orientation: portrait`, 192/512 icons + maskable.
- Workbox precaches the app shell; `/api/*` uses `StaleWhileRevalidate` under the
  versioned `fintrack-api-v2` cache.

## Design system (`theme/app.css`)

Tokens: `--bg #0f1115`, `--surface` / `--surface-2` / `--surface-3`, `--border`, one blue
accent `#4c8dff` plus semantic success / danger / warning, and radii + shadows.
Shared classes: `page`, `page-head`, `page-title`, `card`, `card-hero`, `card-label`,
`card-value`, `card-foot`, `section`, `section-head`, `section-title`, `row-list`,
`row-item`, `row-icon`, `row-main`, `row-title`, `row-sub`, `row-value`, `row-extra`,
`stat-grid`, `stat-tile`, `bar-track`, `bar`, `bar-fill`, `meta-row`, `chip`, `badge`,
`empty`, `chart`, `ff-hide-scrollbar`. `main.ts` imports Ionic's light palette and
`app.css` overrides the variables, so the app stays dark regardless of system theme.