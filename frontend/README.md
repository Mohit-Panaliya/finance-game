# Finance Forge — Frontend PWA

Clash-of-Clans-style personal finance game. Vue 3 + TypeScript + Ionic Vue + Vite + Pinia +
Vue Router + vite-plugin-pwa. Every control is game UI — zero native date/select/dialog.

## Commands

```bash
npm install       # deps
npm run dev       # vite dev server (proxies /api → 127.0.0.1:8000)
npm run build     # production build → dist/ (PWA: sw.js + manifest)   [STATUS: ✓ passing]
npm run typecheck # vue-tsc --noEmit                                    [STATUS: ✓ 0 errors]
npm run preview   # preview dist
npm run icons     # regenerate public/icons/*.png from scripts/gen-icons.mjs
```

## File tree

```
frontend/
├── index.html                     # viewport-fit, theme #1a0f00, Lilita One/Baloo 2 fonts
├── package.json                   # Vue3 / Ionic8 / Pinia / Vite5 / vite-plugin-pwa
├── vite.config.ts                 # @ alias, PWA manifest+workbox, /api proxy
├── tsconfig.json / tsconfig.node.json
├── public/
│   ├── icon.svg                   # SVG app icon (hex + emerald)
│   └── icons/icon-{192,512}.png   # generated PNG icons (maskable included)
├── scripts/gen-icons.mjs          # dependency-free PNG encoder (zlib + CRC32)
└── src/
    ├── main.ts                    # pinia + IonicVue(mode ios, no ripple) + router
    ├── App.vue                    # ion-tabs shell, wooden tab bar (hidden on /login /register)
    ├── types.ts                   # Building/Troop/Village/Achievement/SyncOp/…
    ├── entityConfig.ts            # per-entity form fields, icons, value keys, formatters
    ├── env.d.ts
    ├── theme/game.css             # design system: palette, carved text, panels, ALL keyframes
    ├── api/client.ts              # fetch wrapper, Bearer token, ApiError(offline flag)
    ├── services/offlineQueue.ts   # IndexedDB sync queue + localStorage fallback + server ts
    ├── composables/useCountUp.ts  # rAF count-up
    ├── router/index.ts            # all routes + auth guard
    ├── stores/
    │   ├── authStore.ts           # login/register/me/logout, token persistence
    │   ├── gameStore.ts           # village/buildings/troops/battle/achievements/leaderboard
    │   ├── financeStore.ts        # per-entity CRUD + summaries + offline queueing
    │   └── syncStore.ts           # status online|offline|pushing|synced, flush/pull
    ├── components/game/           # ← custom components (see list below)
    └── pages/                     # 11 Ionic pages (ion-content as scroll host only)
```

## Custom components (src/components/game/)

| Component | Replaces | Mechanism |
|---|---|---|
| **GameDatePicker** | `<input type="date">`, native pickers | 3-column roller (day/month/year), CSS `transform: translateY` + bouncy snap transition, drag/swipe/wheel/+/- nudges, gold ornate frame, dark parchment backdrop, chest Confirm/Cancel. Props `modelValue (YYYY-MM-DD)`, `min`/`max` year, `open`; emits `update:modelValue`, `update:open`, `confirm`, `cancel`. |
| **GameSelect** | `<select>` | Closed = carved gem/wood button with rotating chevron; open = teleported bottom sheet, staggered stone-row entrance (`animation-delay` per index), selected row shows ✦ rune, hover gold glow. NO native select anywhere. |
| **GameModal** | `<dialog>` / alert | Teleport-to-body overlay, backdrop blur + dark gradient, panel bounce `cubic-bezier(0.34,1.56,0.64,1)`, ornate banner header, spinning X close, `default`/`header`/`footer` slots, Esc + backdrop dismiss. |
| **GameInput** | text/number inputs (never `type="date"`) | Carved stone field: inner shadow, gold rivets, focus = golden pulse ring animation. Native `<input>` fully skinned (16px font kills mobile zoom). |
| **GameButton** | ion-button styling | 3D pressable (`translateY` on :active), layered `box-shadows` as depth, variants gold/green/red/blue/wood, sizes sm/md/lg, disabled (gray pressed), optional 6-spark sparkle animation. |
| **ResourceBar** | — | HUD gold/elixir/gems/trophies, rAF count-up, flying-coin micro-anim on increase. |
| **XpBar** | — | Hex level badge + shimmer progress + LEVEL UP burst (ParticleBurst) on level change. |
| **FloatingNumber** | — | Absolute `+N` text, `float-up` keyframe rise/fade, emits `done` for parent cleanup. |
| **ParticleBurst** | — | CSS particle explosion (confetti/sparks, radial angles via `--px/--py`), used by achievements + level-up + battle stars. |
| **SyncChip** (bonus) | — | online/offline/pushing/synced pulse chip, tap = force sync. |

Native-control audit: `grep -rn 'type="date"|<select|<dialog|ion-datetime' src` → **0 matches**.

## Screens

- `/login`, `/register` — fortress gate, torch `flame` keyframes, bouncy form panel
- `/` **Village** — 2.5D building tiles (thick borders, drop shadows, idle `bob`, collect
  bubble `pulse-glow`), tap → GameModal action sheet (collect/upgrade/details), floating gold,
  fixed ResourceBar+XpBar HUD. Mapping: bank=⛏️ Gold Mine, asset=🏰 Castle, FD=🗄️ Vault,
  investment=🔮 Wizard Tower, income=🧪 Elixir Collector, expense=⚔️ Barracks,
  credit-card=🧱 Wall.
- `/battle` — squad picker, GameSelect battle type, animated raid (troops `walk-right`,
  buildings `shake`→`explode`, star `pop-star` fills, loot count-up), result chest with
  3-star animation, battle log with stars. Offline → simulated raid + queued to sync.
- `/records` — summary (yearly income shimmer count-up, monthly fixed, net worth,
  ROI donut via SVG `stroke-dashoffset`), 7 chest cards → `/records/:entity` lists with
  ion-item-sliding swipe actions (EDIT/SLAY), FAB → GameModal form (GameSelect +
  GameDatePicker for select/date fields), `/records/:entity/new` opens form directly.
- `/army` — barracks: troop catalog, count stepper, elixir cost, training progress bar
  (shimmer), roster cards with ATK/HP bars + level pips.
- `/achievements` — trophy room: locked=stone grey, unlocked=gold glow, progress rings
  (SVG), CLAIM → ParticleBurst.
- `/leaderboard` — podium 2-1-3 bounce-in, ranked list, own row gold-highlighted.
- `/settings` — profile, realm sync panel (status/queue/last-sync + SYNC NOW/PULL),
  about, leave fortress (logout).

## Offline + PWA

- `vite-plugin-pwa` generateSW: precaches all assets (54 entries), `clientsClaim`,
  runtime cache `/api/*` GETs **stale-while-revalidate** (`api-get-cache`), Google Fonts
  CacheFirst, SPA `navigateFallback`.
- Manifest: name "Finance Forge", `theme_color #1a0f00`, display standalone, 192/512
  PNG + maskable + SVG icons.
- `services/offlineQueue.ts`: raw IndexedDB (`finance-forge` → `sync_queue` + `meta`) with
  localStorage ring-buffer fallback. Each op: `{id(uuid), entity, op, payload, client_ts}`.
- `syncStore`: `online/offline/push` listeners → on `online` flush via
  `POST /api/sync/push {ops}`; on load `GET /api/sync/pull?since=lastServerTs` then
  emits `ff:synced` so stores refresh. Every store mutation catches offline errors and
  queues the op (village collect/upgrade/train/battle, finance CRUD, achievement claim).

## API (same-origin `/api`)

auth `login|register|me` · CRUD `banks, assets, expenses, credit-cards, fixed-deposits,
investments, incomes` (`?page&perPage&search`, `/:id`, `GET /summary`) · game `village,
buildings/collect|upgrade, troops/train, battle, battles, achievements[/:id/claim],
leaderboard, stats` · sync `push`, `pull?since=`.

## Design system (theme/game.css)

Palette: `#2a1a0a` parchment-brown, `#f5c542` gold, `#4caf50` green, `#e74c3c` red,
`#3498db` blue, `#9b59b6` purple, theme `#1a0f00`. All transitions
`cubic-bezier(0.34,1.56,0.64,1)`. Keyframes: bob, pulse-glow, flame, shake, explode,
coin-fly, shimmer, bounce-in, spin-wheel, sparkle, float-up, pop-star, walk-right,
particle-fly, gate-sway, streak. 3–4px borders, multi-layer shadows, noise texture.
Ionic: `mode: 'ios'` (no MD ripple), ripple elements display:none, ion-tab-bar restyled
as wooden plank with gold icons.
