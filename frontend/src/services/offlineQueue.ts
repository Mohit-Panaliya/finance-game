import type { SyncOp } from '@/types'

const DB_NAME = 'fintrack'
const DB_VERSION = 1
const STORE = 'sync_queue'
const META_STORE = 'meta'
const SERVER_TS_KEY = 'lastServerTs'

let dbPromise: Promise<IDBDatabase> | null = null

function openDB(): Promise<IDBDatabase> {
  if (dbPromise) return dbPromise
  dbPromise = new Promise((resolve, reject) => {
    if (typeof indexedDB === 'undefined') {
      reject(new Error('IndexedDB unavailable'))
      return
    }
    const req = indexedDB.open(DB_NAME, DB_VERSION)
    req.onupgradeneeded = () => {
      const db = req.result
      if (!db.objectStoreNames.contains(STORE)) db.createObjectStore(STORE, { keyPath: 'id' })
      if (!db.objectStoreNames.contains(META_STORE)) db.createObjectStore(META_STORE)
    }
    req.onsuccess = () => resolve(req.result)
    req.onerror = () => reject(req.error ?? new Error('IndexedDB open failed'))
  })
  return dbPromise
}

function txDone(tx: IDBTransaction): Promise<void> {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
    tx.onabort = () => reject(tx.error)
  })
}

export function uuid(): string {
  if (typeof crypto !== 'undefined' && 'randomUUID' in crypto) return crypto.randomUUID()
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0
    const v = c === 'x' ? r : (r & 0x3) | 0x8
    return v.toString(16)
  })
}

export async function enqueue(
  entity: string,
  op: SyncOp['op'],
  payload: unknown,
  entityId?: string
): Promise<SyncOp | null> {
  const record: SyncOp = {
    id: uuid(),
    // The sync endpoint only accepts snake_case table names; entity types use dashes.
    entity: entity.replace(/-/g, '_'),
    entity_id: entityId || uuid(),
    op,
    payload,
    client_ts: new Date().toISOString()
  }
  try {
    const db = await openDB()
    const tx = db.transaction(STORE, 'readwrite')
    tx.objectStore(STORE).put(record)
    await txDone(tx)
    return record
  } catch {
    // Fallback: localStorage ring buffer so we never silently drop mutations
    try {
      const raw = localStorage.getItem('fintrack_sync_fallback')
      const list: SyncOp[] = raw ? JSON.parse(raw) : []
      list.push(record)
      localStorage.setItem('fintrack_sync_fallback', JSON.stringify(list.slice(-200)))
    } catch {
      /* best effort */
    }
    return record
  }
}

export async function getQueue(): Promise<SyncOp[]> {
  const out: SyncOp[] = []
  try {
    const db = await openDB()
    const tx = db.transaction(STORE, 'readonly')
    const req = tx.objectStore(STORE).getAll()
    const rows = await new Promise<SyncOp[]>((resolve, reject) => {
      req.onsuccess = () => resolve((req.result ?? []) as SyncOp[])
      req.onerror = () => reject(req.error)
    })
    out.push(...rows)
  } catch {
    /* fall through to localStorage */
  }
  try {
    const raw = localStorage.getItem('fintrack_sync_fallback')
    if (raw) {
      const fallback = JSON.parse(raw) as SyncOp[]
      const known = new Set(out.map((o) => o.id))
      out.push(...fallback.filter((o) => !known.has(o.id)))
    }
  } catch {
    /* ignore */
  }
  return out.sort((a, b) => a.client_ts.localeCompare(b.client_ts))
}

export async function removeOps(ids: string[]): Promise<void> {
  if (!ids.length) return
  try {
    const db = await openDB()
    const tx = db.transaction(STORE, 'readwrite')
    const store = tx.objectStore(STORE)
    for (const id of ids) store.delete(id)
    await txDone(tx)
  } catch {
    /* ignore */
  }
  try {
    const raw = localStorage.getItem('fintrack_sync_fallback')
    if (raw) {
      const list = JSON.parse(raw) as SyncOp[]
      const idSet = new Set(ids)
      localStorage.setItem('fintrack_sync_fallback', JSON.stringify(list.filter((o) => !idSet.has(o.id))))
    }
  } catch {
    /* ignore */
  }
}

export async function getLastServerTs(): Promise<number> {
  const v = localStorage.getItem(SERVER_TS_KEY)
  return v ? Number(v) || 0 : 0
}

export async function setLastServerTs(ts: number): Promise<void> {
  localStorage.setItem(SERVER_TS_KEY, String(ts))
  try {
    const db = await openDB()
    const tx = db.transaction(META_STORE, 'readwrite')
    tx.objectStore(META_STORE).put(ts, SERVER_TS_KEY)
    await txDone(tx)
  } catch {
    /* localStorage copy is enough */
  }
}
