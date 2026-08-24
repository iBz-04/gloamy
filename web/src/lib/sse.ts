export interface SSEEvent {
  type: string
  timestamp?: string
  [key: string]: unknown
}

export type SSEEventHandler = (event: SSEEvent) => void
export type SSEErrorHandler = (error: Event | Error) => void

export interface SSEClientOptions {
  baseUrl?: string
  getBaseUrl?: () => string | null | undefined
  path?: string
  token?: string | null
  getToken?: () => string | null | undefined
  reconnectDelay?: number
  maxReconnectDelay?: number
  autoReconnect?: boolean
}

const DEFAULT_RECONNECT_DELAY = 1000
const MAX_RECONNECT_DELAY = 30000

function normalizeBaseUrl(input: string): string {
  return input.trim().replace(/\/+$/, '')
}

function normalizePath(path: string): string {
  if (!path) {
    return ''
  }
  if (path.startsWith('http://') || path.startsWith('https://')) {
    return path
  }
  return path.startsWith('/') ? path : `/${path}`
}

function resolveUrl(baseUrl: string, path: string): string {
  const normalizedPath = normalizePath(path)
  if (normalizedPath.startsWith('http://') || normalizedPath.startsWith('https://')) {
    return normalizedPath
  }

  const normalizedBaseUrl = normalizeBaseUrl(baseUrl)
  if (!normalizedBaseUrl) {
    return normalizedPath
  }

  return `${normalizedBaseUrl}${normalizedPath}`
}

export class SSEClient {
  private controller: AbortController | null = null
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private currentDelay: number
  private intentionallyClosed = false

  public onEvent: SSEEventHandler | null = null
  public onError: SSEErrorHandler | null = null
  public onConnect: (() => void) | null = null

  private readonly baseUrl: string
  private readonly getBaseUrl: (() => string | null | undefined) | null
  private readonly path: string
  private readonly token: string | null
  private readonly getToken: (() => string | null | undefined) | null
  private readonly reconnectDelay: number
  private readonly maxReconnectDelay: number
  private readonly autoReconnect: boolean

  constructor(options: SSEClientOptions = {}) {
    this.baseUrl = options.baseUrl ?? ''
    this.getBaseUrl = options.getBaseUrl ?? null
    this.path = options.path ?? '/api/events'
    this.token = options.token ?? null
    this.getToken = options.getToken ?? null
    this.reconnectDelay = options.reconnectDelay ?? DEFAULT_RECONNECT_DELAY
    this.maxReconnectDelay = options.maxReconnectDelay ?? MAX_RECONNECT_DELAY
    this.autoReconnect = options.autoReconnect ?? true
    this.currentDelay = this.reconnectDelay
  }

  connect(): void {
    this.intentionallyClosed = false
    this.clearReconnectTimer()
    this.controller = new AbortController()

    const baseUrl = this.getBaseUrl?.() ?? this.baseUrl
    const token = this.getToken?.() ?? this.token
    const url = resolveUrl(baseUrl ?? '', this.path)

    const headers: Record<string, string> = {
      Accept: 'text/event-stream',
    }

    if (token) {
      headers.Authorization = `Bearer ${token}`
    }

    fetch(url, {
      headers,
      signal: this.controller.signal,
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`SSE connection failed: ${response.status}`)
        }
        if (!response.body) {
          throw new Error('SSE response has no body')
        }

        this.currentDelay = this.reconnectDelay
        this.onConnect?.()

        return this.consumeStream(response.body)
      })
      .catch((err: unknown) => {
        if (err instanceof DOMException && err.name === 'AbortError') {
          return
        }
        this.onError?.(err instanceof Error ? err : new Error(String(err)))
        this.scheduleReconnect()
      })
  }

  disconnect(): void {
    this.intentionallyClosed = true
    this.clearReconnectTimer()
    if (this.controller) {
      this.controller.abort()
      this.controller = null
    }
  }

  private async consumeStream(body: ReadableStream<Uint8Array>): Promise<void> {
    const reader = body.getReader()
    const decoder = new TextDecoder()
    let buffer = ''

    try {
      for (;;) {
        const { done, value } = await reader.read()
        if (done) {
          break
        }

        buffer += decoder.decode(value, { stream: true })

        const parts = buffer.split('\n\n')
        buffer = parts.pop() ?? ''

        for (const part of parts) {
          this.parseEvent(part)
        }
      }
    }
    catch (err: unknown) {
      if (err instanceof DOMException && err.name === 'AbortError') {
        return
      }
      this.onError?.(err instanceof Error ? err : new Error(String(err)))
    }
    finally {
      reader.releaseLock()
    }

    this.scheduleReconnect()
  }

  private parseEvent(raw: string): void {
    let eventType = 'message'
    const dataLines: string[] = []

    for (const line of raw.split('\n')) {
      if (line.startsWith('event:')) {
        eventType = line.slice(6).trim()
      }
      else if (line.startsWith('data:')) {
        dataLines.push(line.slice(5).trim())
      }
    }

    if (dataLines.length === 0) {
      return
    }

    const dataStr = dataLines.join('\n')

    try {
      const parsed = JSON.parse(dataStr) as SSEEvent
      parsed.type = parsed.type ?? eventType
      this.onEvent?.(parsed)
    }
    catch {
      this.onEvent?.({ type: eventType, data: dataStr })
    }
  }

  private scheduleReconnect(): void {
    if (this.intentionallyClosed || !this.autoReconnect) {
      return
    }

    this.reconnectTimer = setTimeout(() => {
      this.currentDelay = Math.min(this.currentDelay * 2, this.maxReconnectDelay)
      this.connect()
    }, this.currentDelay)
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }
}
