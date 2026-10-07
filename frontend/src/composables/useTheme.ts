import { computed, getCurrentScope, onScopeDispose, readonly, ref, watch } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'
export type ResolvedTheme = 'light' | 'dark'
export type AccentName = 'blue' | 'indigo' | 'violet' | 'sky' | 'rose' | 'amber'
export type Density = 'comfortable' | 'compact'

/** Visual style ids, mirrored by `[data-style=…]` rules in `src/theme/app.css`. */
export type StyleId =
  | 'flexoki'
  | 'minimalism'
  | 'flat'
  | 'material'
  | 'bento'
  | 'glassmorphism'
  | 'neumorphism'
  | 'claymorphism'
  | 'skeuomorphism'
  | 'neobrutalism'
  | 'liquidglass'
  | 'y2k'
  | 'cyberpunk'

export interface AccentPalette {
  name: AccentName
  label: string
  /** Colour of the dot in the settings swatch grid. */
  swatch: string
}

export interface ThemeStyle {
  value: StyleId
  label: string
}

export interface ThemeFamily {
  label: string
  themes: ThemeStyle[]
}

/** One namespaced key holding one JSON object, so state can never be half-written. */
export const THEME_STORAGE_KEY = 'fintrack:theme'

/**
 * These four must stay in sync with the inline bootstrap in `index.html`, which
 * has to apply the theme before the first paint and cannot import this file.
 */
export const DEFAULT_MODE: ThemeMode = 'system'
export const DEFAULT_ACCENT: AccentName = 'blue'
export const DEFAULT_DENSITY: Density = 'comfortable'
export const DEFAULT_STYLE: StyleId = 'flexoki'

export const THEME_MODES: { value: ThemeMode; label: string }[] = [
  { value: 'system', label: 'System' },
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' }
]

export const DENSITIES: { value: Density; label: string }[] = [
  { value: 'comfortable', label: 'Comfortable' },
  { value: 'compact', label: 'Compact' }
]

/** Grouped exactly as the settings picker shows them; the CSS agent keys off `value`. */
export const THEME_FAMILIES: ThemeFamily[] = [
  {
    label: 'Default',
    themes: [{ value: 'flexoki', label: 'Flexoki' }]
  },
  {
    label: 'Modern / Minimal',
    themes: [
      { value: 'minimalism', label: 'Minimalism' },
      { value: 'flat', label: 'Flat Design' },
      { value: 'material', label: 'Material Design' },
      { value: 'bento', label: 'Bento Grid' }
    ]
  },
  {
    label: 'Tactile / Dimensional',
    themes: [
      { value: 'glassmorphism', label: 'Glassmorphism' },
      { value: 'neumorphism', label: 'Neumorphism' },
      { value: 'claymorphism', label: 'Claymorphism' },
      { value: 'skeuomorphism', label: 'Skeuomorphism' }
    ]
  },
  {
    label: 'Artistic / Experimental',
    themes: [
      { value: 'neobrutalism', label: 'Neo-Brutalism' },
      { value: 'liquidglass', label: 'Liquid Glass' },
      { value: 'y2k', label: 'Y2K Retro-Futurism' },
      { value: 'cyberpunk', label: 'Dark Mode / Cyberpunk' }
    ]
  }
]

export const FLAT_STYLES: ThemeStyle[] = THEME_FAMILIES.flatMap((f) => f.themes)

export const STYLE_LABELS: Record<StyleId, string> = Object.fromEntries(
  FLAT_STYLES.map((s) => [s.value, s.label])
) as Record<StyleId, string>

export const ACCENT_PALETTES: AccentPalette[] = [
  { name: 'blue', label: 'Blue', swatch: '#4c8dff' },
  { name: 'indigo', label: 'Indigo', swatch: '#7b6cf6' },
  { name: 'violet', label: 'Violet', swatch: '#a56bf0' },
  { name: 'sky', label: 'Sky', swatch: '#2fb6e0' },
  { name: 'rose', label: 'Rose', swatch: '#f05f86' },
  { name: 'amber', label: 'Amber', swatch: '#e5a13a' }
]

const MODES: ThemeMode[] = ['system', 'light', 'dark']
const ACCENTS: AccentName[] = ACCENT_PALETTES.map((p) => p.name)
const DENSITY_VALUES: Density[] = ['comfortable', 'compact']
const STYLES: StyleId[] = FLAT_STYLES.map((s) => s.value)

/**
 * Matches `--bg` per style in `src/theme/styles/*.css`; drives the pre-paint
 * colour and the browser-chrome meta. The inline bootstrap in `index.html`
 * carries the same table (`styleBg`) — change one, change both.
 */
const PAGE_BG: Record<StyleId, Record<ResolvedTheme, string>> = {
  flexoki: { light: '#fffcf0', dark: '#0d0c0c' },
  minimalism: { light: '#ffffff', dark: '#0f0f0f' },
  flat: { light: '#f8f9fa', dark: '#111315' },
  material: { light: '#f7f9fc', dark: '#101418' },
  bento: { light: '#f6f7fb', dark: '#0c1026' },
  glassmorphism: { light: '#f2f4f8', dark: '#05070f' },
  neumorphism: { light: '#e6e9ef', dark: '#14171c' },
  claymorphism: { light: '#f2ebe2', dark: '#1a1610' },
  skeuomorphism: { light: '#dcdcdc', dark: '#161616' },
  neobrutalism: { light: '#ffffff', dark: '#0a0a0a' },
  liquidglass: { light: '#f1f5ff', dark: '#040814' },
  y2k: { light: '#e8e5ff', dark: '#050510' },
  cyberpunk: { light: '#fafbfc', dark: '#050505' }
}

interface StoredTheme {
  mode: ThemeMode
  accent: AccentName
  density: Density
  style: StyleId
}

function defaults(): StoredTheme {
  return {
    mode: DEFAULT_MODE,
    accent: DEFAULT_ACCENT,
    density: DEFAULT_DENSITY,
    style: DEFAULT_STYLE
  }
}

function prefersDark(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return true
  return window.matchMedia('(prefers-color-scheme: dark)').matches
}

/** Every field is validated against the allowed sets; anything else falls back. */
function readStored(): StoredTheme {
  const out = defaults()
  try {
    const raw = localStorage.getItem(THEME_STORAGE_KEY)
    if (!raw) return out
    const parsed: unknown = JSON.parse(raw)
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) return out
    const bag = parsed as Record<string, unknown>
    if (MODES.includes(bag.mode as ThemeMode)) out.mode = bag.mode as ThemeMode
    if (ACCENTS.includes(bag.accent as AccentName)) out.accent = bag.accent as AccentName
    if (DENSITY_VALUES.includes(bag.density as Density)) out.density = bag.density as Density
    // A bag written before styles existed has no `style` key: it fails this
    // check and keeps the default, which is exactly what we want.
    if (STYLES.includes(bag.style as StyleId)) out.style = bag.style as StyleId
  } catch {
    return defaults()
  }
  return out
}

const stored = readStored()
const mode = ref<ThemeMode>(stored.mode)
const accent = ref<AccentName>(stored.accent)
const density = ref<Density>(stored.density)
const style = ref<StyleId>(stored.style)
const systemDark = ref(prefersDark())

const resolvedTheme = computed<ResolvedTheme>(() =>
  mode.value === 'system' ? (systemDark.value ? 'dark' : 'light') : mode.value
)

function setAttr(name: string, value: string): void {
  const el = document.documentElement
  if (el.getAttribute(name) === value) return
  el.setAttribute(name, value)
}

/** Only touches the DOM when a value actually moved. */
function apply(): void {
  if (typeof document === 'undefined') return
  const theme = resolvedTheme.value
  setAttr('data-theme', theme)
  setAttr('data-accent', accent.value)
  setAttr('data-density', density.value)
  setAttr('data-style', style.value)
  const el = document.documentElement
  const bg = PAGE_BG[style.value][theme]
  if (el.style.getPropertyValue('--boot-bg') !== bg) {
    el.style.setProperty('--boot-bg', bg)
  }
  const meta = document.querySelector('meta[name="theme-color"]')
  if (meta && meta.getAttribute('content') !== bg) {
    meta.setAttribute('content', bg)
  }
}

function persist(): void {
  try {
    localStorage.setItem(
      THEME_STORAGE_KEY,
      JSON.stringify({
        mode: mode.value,
        accent: accent.value,
        density: density.value,
        style: style.value
      })
    )
  } catch {
    /* storage blocked or full: the theme applies, it just is not remembered */
  }
}

let mql: MediaQueryList | null = null
let consumers = 0

function onSystemChange(event: MediaQueryListEvent): void {
  systemDark.value = event.matches
}

function startSync(): void {
  if (mql || typeof window === 'undefined' || typeof window.matchMedia !== 'function') return
  mql = window.matchMedia('(prefers-color-scheme: dark)')
  systemDark.value = mql.matches
  mql.addEventListener('change', onSystemChange)
}

function stopSync(): void {
  if (!mql) return
  mql.removeEventListener('change', onSystemChange)
  mql = null
}

function release(): void {
  consumers = Math.max(0, consumers - 1)
  // In System mode the OS can still flip while nothing is mounted, so the
  // listener stays. With an explicit light/dark there is nothing left to watch.
  if (consumers === 0 && mode.value !== 'system') stopSync()
}

function setMode(next: ThemeMode): void {
  if (!MODES.includes(next) || next === mode.value) return
  mode.value = next
  persist()
}

function setAccent(next: AccentName): void {
  if (!ACCENTS.includes(next) || next === accent.value) return
  accent.value = next
  persist()
}

function setDensity(next: Density): void {
  if (!DENSITY_VALUES.includes(next) || next === density.value) return
  density.value = next
  persist()
}

function setStyle(next: StyleId): void {
  if (!STYLES.includes(next) || next === style.value) return
  style.value = next
  persist()
}

function reset(): void {
  const fresh = defaults()
  mode.value = fresh.mode
  accent.value = fresh.accent
  density.value = fresh.density
  style.value = fresh.style
  try {
    localStorage.removeItem(THEME_STORAGE_KEY)
  } catch {
    /* nothing to clear */
  }
}

export interface ThemeControls {
  mode: ThemeMode
  resolvedTheme: ResolvedTheme
  /** Raw `prefers-color-scheme`, shown on the settings page when mode is System. */
  systemTheme: ResolvedTheme
  accent: AccentName
  density: Density
  style: StyleId
  setMode: (next: ThemeMode) => void
  setAccent: (next: AccentName) => void
  setDensity: (next: Density) => void
  setStyle: (next: StyleId) => void
  reset: () => void
}

const controls: ThemeControls = {
  get mode() {
    return mode.value
  },
  get resolvedTheme() {
    return resolvedTheme.value
  },
  get systemTheme() {
    return systemDark.value ? 'dark' : 'light'
  },
  get accent() {
    return accent.value
  },
  get density() {
    return density.value
  },
  get style() {
    return style.value
  },
  setMode,
  setAccent,
  setDensity,
  setStyle,
  reset
}

/**
 * Shared appearance state, mirrored onto `<html>` as `data-theme`,
 * `data-accent`, `data-density` and `data-style`. Singleton, so any number of
 * components can call it without duplicating listeners or fighting over the DOM
 * attributes.
 */
export function useTheme(): Readonly<ThemeControls> {
  // Only scope-bound callers are counted, otherwise a call from a store or a
  // router guard would hold the refcount up and the listener would never be
  // released. startSync() is idempotent, so calling it either way is safe.
  startSync()
  if (getCurrentScope()) {
    consumers += 1
    onScopeDispose(release)
  }
  return readonly(controls) as Readonly<ThemeControls>
}

watch([resolvedTheme, accent, density, style], apply)
startSync()
apply()
