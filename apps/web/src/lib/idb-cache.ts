// IndexedDB key-value cache for large data (agents, skills, drafts).
// Replaces localStorage for data that can exceed the 5 MB limit.
// Small config keys (theme, booleans, port numbers) stay in localStorage.

const DB_NAME = 'ship-cache'
const STORE_NAME = 'kv'

let dbPromise: Promise<IDBDatabase> | null = null

function openDb(): Promise<IDBDatabase> {
  if (!dbPromise) {
    dbPromise = new Promise((resolve, reject) => {
      const req = indexedDB.open(DB_NAME, 1)
      req.onupgradeneeded = () => req.result.createObjectStore(STORE_NAME)
      req.onsuccess = () => resolve(req.result)
      req.onerror = () => { dbPromise = null; reject(req.error) }
    })
  }
  return dbPromise
}

export async function idbGet<T>(key: string): Promise<T | undefined> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readonly')
    const req = tx.objectStore(STORE_NAME).get(key)
    req.onsuccess = () => resolve(req.result as T | undefined)
    req.onerror = () => reject(req.error)
  })
}

export async function idbSet<T>(key: string, value: T): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite')
    tx.objectStore(STORE_NAME).put(value, key)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
}

export async function idbDel(key: string): Promise<void> {
  const db = await openDb()
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, 'readwrite')
    tx.objectStore(STORE_NAME).delete(key)
    tx.oncomplete = () => resolve()
    tx.onerror = () => reject(tx.error)
  })
}

/**
 * Migrate a key from localStorage to IndexedDB. Reads from localStorage,
 * writes to IDB, then removes from localStorage. No-op if key doesn't exist.
 */
export async function migrateFromLocalStorage<T>(key: string): Promise<T | undefined> {
  try {
    const raw = window.localStorage.getItem(key)
    if (!raw) return undefined
    const data = JSON.parse(raw) as T
    await idbSet(key, data)
    window.localStorage.removeItem(key)
    return data
  } catch {
    return undefined
  }
}
