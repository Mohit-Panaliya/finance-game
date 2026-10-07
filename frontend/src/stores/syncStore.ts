import { defineStore } from 'pinia'
import { api, ApiError } from '@/api/client'
import { useFinanceStore } from '@/stores/financeStore'
import { enqueue, getQueue, getLastServerTs, removeOps, setLastServerTs } from '@/services/offlineQueue'
import type { SyncOp } from '@/types'

export type SyncStatus = 'online' | 'offline' | 'pushing' | 'synced' | 'error'

/**
 * Replaying one of these moves a bank or card balance through the ledger even though
 * no `banks`/`credit-cards` row was pushed, so those two lists are re-fetched after
 * the push lands.
 */
const LEDGER_SYNC_ENTITIES = ['incomes', 'expenses', 'assets', 'investments']

interface SyncState {
  status: SyncStatus
  queueCount: number
  lastSyncAt: number | null
  message: string
  initialized: boolean
}

export const useSyncStore = defineStore('sync', {
  state: (): SyncState => ({
    status: typeof navigator !== 'undefined' && navigator.onLine ? 'online' : 'offline',
    queueCount: 0,
    lastSyncAt: null,
    message: '',
    initialized: false
  }),

  actions: {
    async init() {
      if (this.initialized) return
      this.initialized = true
      await this.refreshCount()
      window.addEventListener('online', () => {
        this.status = 'online'
        this.message = 'Back online'
        void this.flush()
        void this.pull()
      })
      window.addEventListener('offline', () => {
        this.status = 'offline'
        this.message = 'Offline — changes queued'
      })
      await this.pull()
      await this.flush()
    },

    async refreshCount() {
      const q = await getQueue()
      this.queueCount = q.length
    },

    async queue(entity: string, op: SyncOp['op'], payload: unknown, entityId?: string) {
      await enqueue(entity, op, payload, entityId)
      await this.refreshCount()
      if (!navigator.onLine) this.status = 'offline'
    },

    async flush(): Promise<boolean> {
      const queue = await getQueue()
      this.queueCount = queue.length
      if (!queue.length) {
        if (navigator.onLine && this.status !== 'pushing') this.status = 'synced'
        return true
      }
      if (!navigator.onLine) {
        this.status = 'offline'
        return false
      }
      this.status = 'pushing'
      this.message = `Pushing ${queue.length} change${queue.length > 1 ? 's' : ''}…`
      try {
        const res = await api.post<{ ok?: boolean; results?: Array<{ status?: string }> }>(
          '/sync/push',
          { ops: queue }
        )
        await removeOps(queue.map((o) => o.id))
        const rejected = (res?.results ?? []).filter((r) => String(r.status ?? '').startsWith('rejected'))
        this.status = 'synced'
        this.lastSyncAt = Date.now()
        this.message = rejected.length
          ? `Synced · ${rejected.length} change${rejected.length > 1 ? 's' : ''} rejected`
          : 'All changes synced'
        await this.refreshCount()
        if (queue.some((o) => LEDGER_SYNC_ENTITIES.includes(o.entity))) {
          await useFinanceStore().refreshLinkedBalances()
        }
        return true
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          this.status = 'offline'
          this.message = 'Offline — changes queued'
        } else {
          this.status = 'error'
          this.message = e instanceof Error ? e.message : 'Sync failed'
        }
        await this.refreshCount()
        return false
      }
    },

    async pull(): Promise<boolean> {
      if (!navigator.onLine) {
        this.status = 'offline'
        return false
      }
      try {
        const last = await getLastServerTs()
        // server_ts is an RFC3339 string and the endpoint compares `since` as a
        // string, so the stored epoch number has to be converted before sending.
        // On a first sync there is no watermark yet, so the param is omitted.
        const since = last > 0 ? new Date(last).toISOString() : undefined
        const res = await api.get<{ server_ts?: string; changes?: unknown; ops?: unknown }>(
          '/sync/pull',
          since ? { since } : {}
        )
        const raw = res?.server_ts
        const serverTs = typeof raw === 'string' ? Date.parse(raw) || Date.now() : Number(raw) || Date.now()
        await setLastServerTs(serverTs)
        this.lastSyncAt = Date.now()
        if (this.status !== 'pushing') this.status = 'synced'
        this.message = 'Pulled latest'
        window.dispatchEvent(new CustomEvent('ff:synced', { detail: res }))
        return true
      } catch (e) {
        if (e instanceof ApiError && e.offline) {
          this.status = 'offline'
          return false
        }
        this.status = 'error'
        this.message = e instanceof Error ? e.message : 'Pull failed'
        return false
      }
    },

    async syncAll() {
      await this.pull()
      await this.flush()
    }
  }
})
