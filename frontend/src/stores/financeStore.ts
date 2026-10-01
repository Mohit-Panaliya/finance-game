import { defineStore } from 'pinia'
import { api, ApiError } from '@/api/client'
import { ENTITY_SNAKE } from '@/entityConfig'
import type { AnalysisResponse, EntityType, FinanceRow, OverviewResponse, Paged } from '@/types'

export interface EntityListState {
  items: FinanceRow[]
  total: number
  loading: boolean
  error: string
  fetchedAt: number
}

const PAGE_SIZE = 100

function emptyList(): EntityListState {
  return { items: [], total: 0, loading: false, error: '', fetchedAt: 0 }
}

export const useFinanceStore = defineStore('finance', {
  state: () => ({
    lists: {
      banks: emptyList(),
      assets: emptyList(),
      'credit-cards': emptyList(),
      'fixed-deposits': emptyList(),
      investments: emptyList(),
      incomes: emptyList(),
      expenses: emptyList()
    } as Record<EntityType, EntityListState>,
    summaries: {} as Partial<Record<EntityType, Record<string, unknown>>>,
    analysis: null as AnalysisResponse | null,
    analysisLoading: false,
    analysisError: '',
    overview: null as OverviewResponse | null,
    overviewLoading: false,
    overviewError: ''
  }),

  getters: {
    /** Cached rows for one entity. */
    rows: (state) => (entity: EntityType): FinanceRow[] => state.lists[entity]?.items ?? [],

    count: (state) => (entity: EntityType): number => state.lists[entity]?.total ?? 0,

    /** Sum of a numeric column across a cached list. */
    sumOf: (state) => (entity: EntityType, key: string): number =>
      (state.lists[entity]?.items ?? []).reduce<number>((acc: number, row: FinanceRow) => {
        const n = Number(row[key])
        return acc + (Number.isFinite(n) ? n : 0)
      }, 0),

    isEmpty: (state) => (entity: EntityType): boolean => {
      const list = state.lists[entity]
      return !list || (list.total === 0 && list.items.length === 0)
    }
  },

  actions: {
    /**
     * Every list endpoint answers with `{ data, total, page, perPage }` — the
     * payload key is `data`, not `items`.
     */
    async fetchList(entity: EntityType, search = ''): Promise<void> {
      const list = this.lists[entity]
      list.loading = true
      list.error = ''
      try {
        const query: Record<string, string | number> = { page: 1, perPage: PAGE_SIZE }
        if (search.trim()) query.search = search.trim()
        const res = await api.get<Paged<FinanceRow>>(`/${entity}`, query)
        list.items = Array.isArray(res?.data) ? res.data : []
        list.total = Number(res?.total ?? list.items.length) || 0
        list.fetchedAt = Date.now()
      } catch (e) {
        list.error = e instanceof ApiError ? e.message : 'Could not load records'
      } finally {
        list.loading = false
      }
    },

    async fetchSummary(entity: EntityType): Promise<void> {
      try {
        const res = await api.get<Record<string, unknown>>(`/${entity}/summary`)
        this.summaries[entity] = res
      } catch {
        /* summaries are decorative — a failure must not break the page */
      }
    },

    /** One `Promise.allSettled` fan-out for all 7 entities, instead of 7 chained round-trips. */
    async fetchAll(): Promise<void> {
      const entities: EntityType[] = [
        'banks',
        'assets',
        'credit-cards',
        'fixed-deposits',
        'investments',
        'incomes',
        'expenses'
      ]
      await Promise.allSettled([
        ...entities.map((e) => this.fetchList(e)),
        ...entities.map((e) => this.fetchSummary(e))
      ])
    },

    async fetchAnalysis(): Promise<void> {
      this.analysisLoading = true
      this.analysisError = ''
      try {
        this.analysis = await api.get<AnalysisResponse>('/analysis')
      } catch (e) {
        this.analysis = null
        this.analysisError = e instanceof ApiError ? e.message : 'Could not load analysis'
      } finally {
        this.analysisLoading = false
      }
    },

    /**
     * `entityTypes` is the snake_case filter the endpoint accepts. Kept optional
     * so the default request stays unchanged.
     */
    async fetchAnalysisFiltered(entityTypes: string[] = []): Promise<void> {
      this.analysisLoading = true
      this.analysisError = ''
      try {
        const query: Record<string, string> = {}
        if (entityTypes.length) query.entity_types = entityTypes.map((e) => (ENTITY_SNAKE as Record<string, string>)[e] ?? e).join(',')
        this.analysis = await api.get<AnalysisResponse>('/analysis', query)
      } catch (e) {
        this.analysisError = e instanceof ApiError ? e.message : 'Could not load analysis'
      } finally {
        this.analysisLoading = false
      }
    },

    async fetchOverview(): Promise<void> {
      this.overviewLoading = true
      this.overviewError = ''
      try {
        this.overview = await api.get<OverviewResponse>('/overview')
      } catch (e) {
        this.overview = null
        this.overviewError = e instanceof ApiError ? e.message : 'Could not load overview'
      } finally {
        this.overviewLoading = false
      }
    },

    /**
     * Show a row that only exists in the offline queue, so a queued create or
     * delete is visible immediately instead of vanishing until the next fetch.
     */
    stageLocal(entity: EntityType, row: FinanceRow): void {
      const list = this.lists[entity]
      if (!list.items.some((r) => String(r.id) === String(row.id))) {
        list.items = [row, ...list.items]
      }
      list.total += 1
      void this.fetchSummary(entity)
    },

    dropLocal(entity: EntityType, id: string): void {
      const list = this.lists[entity]
      const before = list.items.length
      list.items = list.items.filter((r) => String(r.id) !== id)
      if (list.items.length !== before) list.total = Math.max(0, list.total - 1)
      void this.fetchSummary(entity)
    },

    /** Patch a queued row in place after an offline edit. */
    patchLocal(entity: EntityType, id: string, data: Record<string, unknown>): void {
      const list = this.lists[entity]
      list.items = list.items.map((r) => (String(r.id) === id ? { ...r, ...data } : r))
    },

    async create(entity: EntityType, data: Record<string, unknown>): Promise<FinanceRow> {
      const created = await api.post<FinanceRow>(`/${entity}`, data)
      await this.fetchList(entity)
      await this.fetchSummary(entity)
      void this.fetchOverview()
      return created?.id ? created : { ...data, id: created?.id }
    },

    async update(entity: EntityType, id: string, data: Record<string, unknown>): Promise<void> {
      await api.put<FinanceRow>(`/${entity}/${id}`, data)
      await this.fetchList(entity)
      await this.fetchSummary(entity)
      void this.fetchOverview()
    },

    async remove(entity: EntityType, id: string): Promise<void> {
      await api.del(`/${entity}/${id}`)
      await this.fetchList(entity)
      await this.fetchSummary(entity)
      void this.fetchOverview()
    },

    async refresh(): Promise<void> {
      await this.fetchAll()
      await Promise.allSettled([this.fetchOverview(), this.fetchAnalysis()])
    },

    /** Clears every cached list — used by sign-out. */
    reset() {
      this.lists = {
        banks: emptyList(),
        assets: emptyList(),
        'credit-cards': emptyList(),
        'fixed-deposits': emptyList(),
        investments: emptyList(),
        incomes: emptyList(),
        expenses: emptyList()
      }
      this.summaries = {}
      this.analysis = null
      this.overview = null
    }
  }
})