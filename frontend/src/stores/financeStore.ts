import { defineStore } from 'pinia'
import { api, ApiError } from '@/api/client'
import { useSyncStore } from '@/stores/syncStore'
import type { EntityType, FinanceRow } from '@/types'

export interface EntityListState {
  items: FinanceRow[]
  total: number
  page: number
  perPage: number
  search: string
  loading: boolean
  error: string | null
}

function blankList(): EntityListState {
  return { items: [], total: 0, page: 1, perPage: 20, search: '', loading: false, error: null }
}

const ENTITIES: EntityType[] = [
  'banks',
  'assets',
  'expenses',
  'credit-cards',
  'fixed-deposits',
  'investments',
  'incomes'
]

interface FinanceState {
  lists: Record<EntityType, EntityListState>
  summaries: Partial<Record<EntityType, Record<string, number>>>
  summary: Record<string, number>
  summaryLoading: boolean
}

export const useFinanceStore = defineStore('finance', {
  state: (): FinanceState => ({
    lists: Object.fromEntries(ENTITIES.map((e) => [e, blankList()])) as Record<
      EntityType,
      EntityListState
    >,
    summaries: {},
    summary: {},
    summaryLoading: false
  }),

  getters: {
    entityTotal: (s) => (entity: EntityType, valueKeys: string[]): number => {
      const list = s.lists[entity]
      if (!list) return 0
      return list.items.reduce((sum, row) => {
        const key = valueKeys.find((k) => row[k] !== undefined)
        if (!key) return sum
        const v = Number(row[key])
        return sum + (Number.isFinite(v) ? v : 0)
      }, 0)
    }
  },

  actions: {
    async fetchList(
      entity: EntityType,
      opts: { page?: number; perPage?: number; search?: string; reset?: boolean } = {}
    ) {
      const list = this.lists[entity]
      if (!list) return
      list.loading = true
      list.error = null
      if (opts.reset) {
        list.page = 1
        if (opts.search !== undefined) list.search = opts.search
      }
      if (opts.page !== undefined) list.page = opts.page
      if (opts.perPage !== undefined) list.perPage = opts.perPage
      if (opts.search !== undefined && !opts.reset) list.search = opts.search
      try {
        const res = await api.get<FinanceRow[] | { items?: FinanceRow[]; total?: number }>(
          `/${entity}`,
          { page: list.page, perPage: list.perPage, search: list.search }
        )
        if (Array.isArray(res)) {
          list.items = res
          list.total = res.length
        } else {
          list.items = res?.items ?? []
          list.total = Number(res?.total ?? list.items.length)
        }
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          list.error = 'Offline — showing cached rows'
        } else {
          list.error = e instanceof Error ? e.message : 'Failed to load'
        }
      } finally {
        list.loading = false
      }
    },

    async create(entity: EntityType, data: FinanceRow): Promise<FinanceRow> {
      try {
        const created = await api.post<FinanceRow>(`/${entity}`, data)
        const row = created?.id ? created : { ...data, id: Date.now() }
        this.lists[entity]?.items.unshift(row)
        if (this.lists[entity]) this.lists[entity].total += 1
        return row
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const row = { ...data, id: `local-${Date.now()}` }
          this.lists[entity]?.items.unshift(row)
          if (this.lists[entity]) this.lists[entity].total += 1
          const sync = useSyncStore()
          await sync.queue(entity, 'create', data)
          return row
        }
        throw e
      }
    },

    async update(entity: EntityType, id: number | string, data: FinanceRow): Promise<FinanceRow> {
      try {
        const updated = await api.put<FinanceRow>(`/${entity}/${id}`, data)
        const row = updated?.id ? updated : { ...data, id }
        const idx = this.lists[entity]?.items.findIndex((r) => r.id === id) ?? -1
        if (idx >= 0 && this.lists[entity]) this.lists[entity].items[idx] = row
        return row
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          const idx = this.lists[entity]?.items.findIndex((r) => r.id === id) ?? -1
          if (idx >= 0 && this.lists[entity])
            this.lists[entity].items[idx] = { ...this.lists[entity].items[idx], ...data }
          const sync = useSyncStore()
          await sync.queue(entity, 'update', { id, ...data })
          return { ...data, id }
        }
        throw e
      }
    },

    async remove(entity: EntityType, id: number | string): Promise<void> {
      const idx = this.lists[entity]?.items.findIndex((r) => r.id === id) ?? -1
      try {
        await api.del(`/${entity}/${id}`)
        if (idx >= 0 && this.lists[entity]) {
          this.lists[entity].items.splice(idx, 1)
          this.lists[entity].total = Math.max(0, this.lists[entity].total - 1)
        }
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          if (idx >= 0 && this.lists[entity]) {
            this.lists[entity].items.splice(idx, 1)
            this.lists[entity].total = Math.max(0, this.lists[entity].total - 1)
          }
          const sync = useSyncStore()
          await sync.queue(entity, 'delete', { id })
          return
        }
        throw e
      }
    },

    async fetchEntitySummary(entity: EntityType) {
      try {
        const res = await api.get<Record<string, number>>(`/${entity}/summary`)
        if (res) this.summaries[entity] = res
      } catch {
        /* offline */
      }
    },

    async fetchSummary() {
      this.summaryLoading = true
      try {
        const res = await api.get<Record<string, number>>('/incomes/summary')
        const global = await api.get<Record<string, number>>('/expenses/summary').catch(() => null)
        this.summary = { ...(res ?? {}), ...(global ?? {}) }
        await Promise.all(ENTITIES.map((e) => this.fetchEntitySummary(e)))
      } catch {
        /* offline */
      } finally {
        this.summaryLoading = false
      }
    }
  }
})
