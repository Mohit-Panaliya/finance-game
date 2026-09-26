export type EntityType =
  | 'banks'
  | 'assets'
  | 'expenses'
  | 'credit-cards'
  | 'fixed-deposits'
  | 'investments'
  | 'incomes'

export interface FinanceRow {
  id?: number | string
  [key: string]: unknown
}

export interface Paged<T> {
  items: T[]
  total?: number
  page?: number
  per_page?: number
}

export interface User {
  id?: number | string
  email?: string
  name?: string
  username?: string
}

export interface Resources {
  gold: number
  gems: number
  elixir: number
  trophies: number
}

export interface Village {
  id?: number | string
  level?: number
  xp?: number
  experience?: number
  xp_to_next?: number
  next_level_xp?: number
  gold?: number
  gems?: number
  elixir?: number
  trophies?: number
  resources?: Partial<Resources>
  [key: string]: unknown
}

export type BuildingType =
  | 'bank'
  | 'asset'
  | 'fixed-deposit'
  | 'investment'
  | 'income'
  | 'expense'
  | 'credit-card'

export interface Building {
  id: number | string
  type: string
  name?: string
  level: number
  production_ready?: boolean
  can_collect?: boolean
  ready?: boolean
  ready_at?: string
  produced_amount?: number
  collectible?: number
  upgrade_cost?: number
  upgrade_gold_cost?: number
  upgrade_elixir_cost?: number
  production_rate?: number
  entity_id?: number | string
  [key: string]: unknown
}

export interface Troop {
  id?: number | string
  troop_type?: string
  type?: string
  name?: string
  count?: number
  level?: number
  attack?: number
  hp?: number
  damage?: number
  health?: number
  training?: number
  training_count?: number
  ready?: boolean
  icon?: string
  cost?: number
  [key: string]: unknown
}

export interface BattleRecord {
  id: number | string
  battle_type?: string
  result?: string
  stars?: number
  loot?: number
  gold_looted?: number
  created_at?: string
  [key: string]: unknown
}

export interface Achievement {
  id: number | string
  title?: string
  name?: string
  description?: string
  icon?: string
  xp_reward?: number
  gold_reward?: number
  gems_reward?: number
  unlocked?: boolean
  claimed?: boolean
  progress?: number
  max_progress?: number
  target?: number
  [key: string]: unknown
}

export interface LeaderboardRow {
  rank?: number
  position?: number
  user_id?: number | string
  name?: string
  username?: string
  trophies?: number
  level?: number
  is_me?: boolean
  [key: string]: unknown
}

export interface GameStats {
  level?: number
  xp?: number
  battles_won?: number
  battles_lost?: number
  total_loot?: number
  troops_trained?: number
  buildings_upgraded?: number
  [key: string]: unknown
}

export interface VillageResponse {
  village: Village
  buildings: Building[]
  troops: Troop[]
  unlocked_achievements: Array<number | string> | Achievement[]
  stats: GameStats
}

export interface SyncOp {
  id: string
  entity: string
  op: 'create' | 'update' | 'delete'
  payload: unknown
  client_ts: string
}
