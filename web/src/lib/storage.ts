/**
 * Browser-backed key-value store.
 *
 * Mirrors the small async surface the app previously used from
 * `@tauri-apps/plugin-store` (`get`/`set`/`save`/`entries`) so persistence stays
 * a single swappable seam: replacing this class with a server-backed
 * implementation is the only change required to move state off the device.
 *
 * Each instance namespaces its keys under `name`, which keeps the former
 * one-file-per-concern layout (`auth.json`, `settings.json`, `ui-state.json`).
 */

type Snapshot = Record<string, unknown>

const KEY_PREFIX = 'gloamy:'

/** localStorage is absent in private-mode/embedded contexts; degrade to memory. */
function resolveBackend(): Pick<Storage, 'getItem' | 'setItem'> {
  try {
    const probe = `${KEY_PREFIX}__probe__`
    globalThis.localStorage.setItem(probe, '1')
    globalThis.localStorage.removeItem(probe)
    return globalThis.localStorage
  }
  catch {
    const memory = new Map<string, string>()
    return {
      getItem: key => memory.get(key) ?? null,
      setItem: (key, value) => void memory.set(key, value),
    }
  }
}

const backend = resolveBackend()

export class WebStore {
  private readonly storageKey: string
  private cache: Snapshot | null = null

  constructor(name: string) {
    this.storageKey = `${KEY_PREFIX}${name}`
  }

  private read(): Snapshot {
    if (this.cache)
      return this.cache

    let snapshot: Snapshot = {}
    const raw = backend.getItem(this.storageKey)
    if (raw) {
      try {
        const parsed = JSON.parse(raw)
        if (parsed !== null && typeof parsed === 'object' && !Array.isArray(parsed))
          snapshot = parsed as Snapshot
      }
      catch {
        // Corrupt payload: start clean rather than wedging app startup.
      }
    }

    this.cache = snapshot
    return snapshot
  }

  async get<T>(key: string): Promise<T | null> {
    const value = this.read()[key]
    return value === undefined ? null : (value as T)
  }

  async set<T>(key: string, value: T): Promise<void> {
    this.read()[key] = value
    this.flush()
  }

  async entries<T>(): Promise<[string, T][]> {
    return Object.entries(this.read()) as [string, T][]
  }

  /**
   * Retained for call-site parity with the previous lazy store. Writes are
   * already flushed by `set`, so this is a no-op beyond re-persisting.
   */
  async save(): Promise<void> {
    this.flush()
  }

  private flush(): void {
    try {
      backend.setItem(this.storageKey, JSON.stringify(this.read()))
    }
    catch (error) {
      // Quota exceeded or storage disabled mid-session: keep the in-memory
      // value authoritative for this session instead of throwing into callers.
      console.error(`Failed to persist ${this.storageKey}:`, error)
    }
  }
}
