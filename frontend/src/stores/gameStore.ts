import { defineStore } from 'pinia'
import { api, ApiError } from '@/api/client'
import { useSyncStore } from '@/stores/syncStore'
import type {
  Achievement,
  BattleRecord,
  Building,
  GameStats,
  LeaderboardRow,
  Resources,
  Troop,
  Village,
  VillageResponse
} from '@/types'

interface GameState {
  village: Village | null
  buildings: Building[]
  troops: Troop[]
  achievements: Achievement[]
  unlockedAchievements: Array<number | string>
  battles: BattleRecord[]
  leaderboard: LeaderboardRow[]
  stats: GameStats | null
  resources: Resources
  level: number
  xp: number
  xpToNext: number
  loading: boolean
  error: string | null
  lastCollect: { amount: number; at: number } | null
  lastBattle: {
    result: 'victory' | 'defeat'
    stars: number
    loot: number
    simulated: boolean
  } | null
}

const DEFAULT_RESOURCES: Resources = { gold: 0, gems: 0, elixir: 0, trophies: 0 }

function readResources(v: Village | null): Resources {
  if (!v) return { ...DEFAULT_RESOURCES }
  const nested = (v.resources ?? {}) as Partial<Resources>
  return {
    gold: Number(v.gold ?? nested.gold ?? 0),
    gems: Number(v.gems ?? nested.gems ?? 0),
    elixir: Number(v.elixir ?? nested.elixir ?? 0),
    trophies: Number(v.trophies ?? nested.trophies ?? 0)
  }
}

export const useGameStore = defineStore('game', {
  state: (): GameState => ({
    village: null,
    buildings: [],
    troops: [],
    achievements: [],
    unlockedAchievements: [],
    battles: [],
    leaderboard: [],
    stats: null,
    resources: { ...DEFAULT_RESOURCES },
    level: 1,
    xp: 0,
    xpToNext: 100,
    loading: false,
    error: null,
    lastCollect: null,
    lastBattle: null
  }),

  getters: {
    readyBuildings: (s): Building[] =>
      s.buildings.filter((b) => b.production_ready || b.can_collect || b.ready),
    canAfford: (s) => (cost: number, currency: 'gold' | 'elixir' | 'gems' = 'gold') =>
      s.resources[currency] >= cost,
    troopsTotal: (s): number => s.troops.reduce((sum, t) => sum + Number(t.count ?? 0), 0)
  },

  actions: {
    async fetchVillage() {
      this.loading = true
      this.error = null
      try {
        const res = await api.get<VillageResponse>('/game/village')
        this.applyVillage(res)
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          this.error = 'Offline — showing last village'
        } else {
          this.error = e instanceof Error ? e.message : 'Failed to load village'
        }
      } finally {
        this.loading = false
      }
    },

    applyVillage(res: VillageResponse | null | undefined) {
      if (!res) return
      this.village = res.village ?? null
      this.buildings = res.buildings ?? []
      this.troops = res.troops ?? []
      this.stats = res.stats ?? null
      const ua = res.unlocked_achievements ?? []
      this.unlockedAchievements = ua.map((a) =>
        typeof a === 'object' && a !== null ? (a as Achievement).id ?? '' : a
      )
      this.resources = readResources(this.village)
      this.level = Number(this.village?.level ?? this.stats?.level ?? 1)
      this.xp = Number(this.village?.xp ?? this.village?.experience ?? this.stats?.xp ?? 0)
      this.xpToNext = Number(this.village?.xp_to_next ?? this.village?.next_level_xp ?? 100) || 100
    },

    async collect(buildingId: number | string): Promise<number> {
      try {
        const res = await api.post<{ gained?: number; amount?: number; village?: Village; buildings?: Building[] }>(
          '/game/buildings/collect',
          { building_id: buildingId }
        )
        const gained = Number(res?.gained ?? res?.amount ?? 0)
        if (res?.village) this.village = res.village
        if (res?.buildings) this.buildings = res.buildings
        else {
          const b = this.buildings.find((x) => x.id === buildingId)
          if (b) b.production_ready = false
        }
        this.resources = readResources(this.village)
        if (!this.village) this.resources.gold += gained
        this.lastCollect = { amount: gained, at: Date.now() }
        return gained
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const b = this.buildings.find((x) => x.id === buildingId)
          const gained = Number(b?.collectible ?? b?.produced_amount ?? 50)
          this.resources.gold += gained
          if (b) b.production_ready = false
          this.lastCollect = { amount: gained, at: Date.now() }
          const sync = useSyncStore()
          await sync.queue('game.buildings', 'update', { action: 'collect', building_id: buildingId })
          return gained
        }
        throw e
      }
    },

    async upgrade(buildingId: number | string): Promise<boolean> {
      try {
        const res = await api.post<{ village?: Village; buildings?: Building[]; building?: Building }>(
          '/game/buildings/upgrade',
          { building_id: buildingId }
        )
        if (res?.village) this.village = res.village
        if (res?.buildings) this.buildings = res.buildings
        else if (res?.building) {
          const idx = this.buildings.findIndex((b) => b.id === buildingId)
          if (idx >= 0) this.buildings[idx] = res.building as Building
        }
        this.resources = readResources(this.village)
        return true
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const sync = useSyncStore()
          await sync.queue('game.buildings', 'update', { action: 'upgrade', building_id: buildingId })
          const b = this.buildings.find((x) => x.id === buildingId)
          if (b) b.level = Number(b.level ?? 1) + 1
          return true
        }
        throw e
      }
    },

    async trainTroop(troopType: string, count: number): Promise<boolean> {
      try {
        const res = await api.post<{ troops?: Troop[]; village?: Village }>('/game/troops/train', {
          troop_type: troopType,
          count
        })
        if (res?.troops) this.troops = res.troops
        else {
          const t = this.troops.find((x) => (x.troop_type ?? x.type) === troopType)
          if (t) t.count = Number(t.count ?? 0) + count
          else this.troops.push({ troop_type: troopType, count })
        }
        if (res?.village) {
          this.village = res.village
          this.resources = readResources(this.village)
        } else {
          const cost = this.troopCost(troopType) * count
          this.resources.elixir = Math.max(0, this.resources.elixir - cost)
        }
        return true
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const t = this.troops.find((x) => (x.troop_type ?? x.type) === troopType)
          if (t) t.count = Number(t.count ?? 0) + count
          else this.troops.push({ troop_type: troopType, count })
          const sync = useSyncStore()
          await sync.queue('game.troops', 'create', { troop_type: troopType, count })
          return true
        }
        throw e
      }
    },

    troopCost(type: string): number {
      const catalog: Record<string, number> = {
        barbarian: 25,
        archer: 50,
        giant: 150,
        wizard: 300,
        dragon: 800,
        goblin: 20
      }
      const t = this.troops.find((x) => (x.troop_type ?? x.type) === type)
      return Number(t?.cost ?? catalog[type] ?? 50)
    },

    async attack(battleType: string, troopIds: Array<number | string>) {
      try {
        const res = await api.post<{
          result?: string
          stars?: number
          loot?: number
          gold_looted?: number
          village?: Village
        }>('/game/battle', { battle_type: battleType, troop_ids: troopIds })
        const result = (res?.result ?? 'victory').toLowerCase().includes('defeat')
          ? ('defeat' as const)
          : ('victory' as const)
        this.lastBattle = {
          result,
          stars: Math.max(0, Math.min(3, Number(res?.stars ?? 3))),
          loot: Number(res?.loot ?? res?.gold_looted ?? 0),
          simulated: false
        }
        if (res?.village) {
          this.village = res.village
          this.resources = readResources(this.village)
        } else {
          this.resources.gold += this.lastBattle.loot
        }
        void this.fetchBattles()
        return this.lastBattle
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const power = troopIds.length
          const stars = power >= 6 ? 3 : power >= 3 ? 2 : power > 0 ? 1 : 0
          const loot = 120 + power * 85 + Math.floor(Math.random() * 200)
          this.lastBattle = {
            result: stars > 0 ? 'victory' : 'defeat',
            stars,
            loot,
            simulated: true
          }
          this.resources.gold += loot
          const sync = useSyncStore()
          await sync.queue('game.battles', 'create', { battle_type: battleType, troop_ids: troopIds })
          return this.lastBattle
        }
        throw e
      }
    },

    async fetchBattles() {
      try {
        const res = await api.get<BattleRecord[] | { items?: BattleRecord[] }>('/game/battles')
        this.battles = Array.isArray(res) ? res : (res?.items ?? [])
      } catch {
        /* offline — keep cache */
      }
    },

    async fetchAchievements() {
      try {
        const res = await api.get<Achievement[] | { items?: Achievement[] }>('/game/achievements')
        const list = Array.isArray(res) ? res : (res?.items ?? [])
        this.achievements = list
        const unlocked = list.filter((a) => a.unlocked).map((a) => a.id)
        if (unlocked.length) this.unlockedAchievements = unlocked
      } catch {
        /* offline */
      }
    },

    async claimAchievement(id: number | string): Promise<boolean> {
      try {
        const res = await api.post<{ achievement?: Achievement; village?: Village }>(
          `/game/achievements/${id}/claim`
        )
        if (res?.achievement) {
          const idx = this.achievements.findIndex((a) => a.id === id)
          if (idx >= 0) this.achievements[idx] = res.achievement as Achievement
        } else {
          const a = this.achievements.find((x) => x.id === id)
          if (a) a.claimed = true
        }
        if (res?.village) {
          this.village = res.village
          this.resources = readResources(this.village)
        }
        if (!this.unlockedAchievements.includes(id)) this.unlockedAchievements.push(id)
        return true
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const a = this.achievements.find((x) => x.id === id)
          if (a) a.claimed = true
          if (!this.unlockedAchievements.includes(id)) this.unlockedAchievements.push(id)
          const sync = useSyncStore()
          await sync.queue('game.achievements', 'update', { action: 'claim', id })
          return true
        }
        throw e
      }
    },

    async fetchLeaderboard() {
      try {
        const res = await api.get<LeaderboardRow[] | { items?: LeaderboardRow[] }>('/game/leaderboard')
        const rows = Array.isArray(res) ? res : (res?.items ?? [])
        this.leaderboard = rows.map((r, i) => ({ rank: r.rank ?? r.position ?? i + 1, ...r }))
      } catch {
        /* offline */
      }
    },

    async fetchStats() {
      try {
        const res = await api.get<GameStats>('/game/stats')
        if (res) this.stats = res
      } catch {
        /* offline */
      }
    },

    async fetchAll() {
      await Promise.all([
        this.fetchVillage(),
        this.fetchBattles(),
        this.fetchAchievements(),
        this.fetchLeaderboard(),
        this.fetchStats()
      ])
    }
  }
})
