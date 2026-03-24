export class UnauthorizedError extends Error {
  constructor() {
    super('Unauthorized')
    this.name = 'UnauthorizedError'
  }
}

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

export async function apiFetch<T = unknown>(
  baseUrl: string,
  path: string,
  options: RequestInit = {},
  token?: string | null,
): Promise<T> {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl)
  const normalizedPath = normalizePath(path)
  const url = normalizedPath.startsWith('http')
    ? normalizedPath
    : `${normalizedBaseUrl}${normalizedPath}`

  const headers = new Headers(options.headers)

  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  if (
    options.body
    && typeof options.body === 'string'
    && !headers.has('Content-Type')
  ) {
    headers.set('Content-Type', 'application/json')
  }

  const response = await fetch(url, { ...options, headers })

  if (response.status === 401) {
    throw new UnauthorizedError()
  }

  if (!response.ok) {
    const text = await response.text().catch(() => '')
    throw new Error(`API ${response.status}: ${text || response.statusText}`)
  }

  if (response.status === 204) {
    return undefined as unknown as T
  }

  return response.json() as Promise<T>
}

export async function pair(baseUrl: string, code: string): Promise<{ token: string }> {
  const normalizedBaseUrl = normalizeBaseUrl(baseUrl)
  const response = await fetch(`${normalizedBaseUrl}/pair`, {
    method: 'POST',
    headers: { 'X-Pairing-Code': code },
  })

  if (!response.ok) {
    const text = await response.text().catch(() => '')
    throw new Error(`Pairing failed (${response.status}): ${text || response.statusText}`)
  }

  return response.json() as Promise<{ token: string }>
}
