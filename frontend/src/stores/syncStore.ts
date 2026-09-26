import { defineStore } from 'pinia'
import { api, ApiError } from '@/api/client'
import { enqueue, getQueue, getLastServerTs, removeOps, setLastServerTs } from '@/services/offlineQueue'
import type { SyncOp } from '@/types'

export type SyncStatus = 'online' | 'offline' | 'pushing' | 'synced' | 'error'

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

    async queue(entity: string, op: SyncOp['op'], payload: unknown) {
      await enqueue(entity, op, payload)
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
        await api.post('/sync/push', { ops: queue })
        await removeOps(queue.map((o) => o.id))
        this.status = 'synced'
        this.lastSyncAt = Date.now()
        this.message = 'All changes synced'
        await this.refreshCount()
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
        const since = await getLastServerTs()
        const res = await api.get<{ server_ts?: number; changes?: unknown; ops?: unknown }>(
          '/sync/pull',
          { since }
        )
        const serverTs = res?.server_ts ?? Date.now()
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
