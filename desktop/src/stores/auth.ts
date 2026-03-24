import { LazyStore } from '@tauri-apps/plugin-store'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { apiFetch, pair as pairRequest, UnauthorizedError } from '@/lib/api'

const store: LazyStore = new LazyStore('auth.json')

const TOKEN_KEY = 'token'
const BASE_URL_KEY = 'api_base_url'
export const DEFAULT_BASE_URL = (import.meta.env.VITE_GLOAMY_API_BASE as string | undefined)
  ?? 'http://127.0.0.1:42617'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(null)
  const baseUrl = ref<string>(DEFAULT_BASE_URL)
  const isLoaded = ref(false)

  const isAuthenticated = computed(() => Boolean(token.value))

  async function load() {
    if (isLoaded.value)
      return

    try {
      const storedToken = await store.get<string>(TOKEN_KEY)
      if (typeof storedToken === 'string' && storedToken.trim().length > 0)
        token.value = storedToken

      const storedBaseUrl = await store.get<string>(BASE_URL_KEY)
      if (typeof storedBaseUrl === 'string' && storedBaseUrl.trim().length > 0)
        baseUrl.value = storedBaseUrl
    }
    catch (error) {
      console.error('Failed to load auth state:', error)
    }
    finally {
      isLoaded.value = true
    }
  }

  async function setBaseUrl(value: string) {
    const nextValue = value.trim() || DEFAULT_BASE_URL
    baseUrl.value = nextValue
    await store.set(BASE_URL_KEY, nextValue)
    await store.save()
  }

  async function setToken(value: string) {
    token.value = value
    await store.set(TOKEN_KEY, value)
    await store.save()
  }

  async function clearToken() {
    token.value = null
    await store.set(TOKEN_KEY, '')
    await store.save()
  }

  async function pair(code: string) {
    const response = await pairRequest(baseUrl.value, code)
    await setToken(response.token)
  }

  async function logout() {
    await clearToken()
  }

  async function fetchWithAuth<T = unknown>(path: string, options: RequestInit = {}) {
    try {
      return await apiFetch<T>(baseUrl.value, path, options, token.value)
    }
    catch (error) {
      if (error instanceof UnauthorizedError) {
        await logout()
        window.dispatchEvent(new Event('gloamy-unauthorized'))
      }
      throw error
    }
  }

  return {
    token,
    baseUrl,
    isLoaded,
    isAuthenticated,
    load,
    setBaseUrl,
    setToken,
    clearToken,
    pair,
    logout,
    fetchWithAuth,
  }
})
