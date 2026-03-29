<script setup lang="ts">
import { computed, onMounted, ref, unref } from 'vue'
import { Icon } from '@iconify/vue'
import type { AuthState } from '@/stores/auth'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()

const code = ref('')
const tokenInput = ref('')
const loading = ref(false)
const error = ref('')
const notice = ref('')

const isAuthenticated = computed(() => auth.isAuthenticated)
const isReady = computed(() => auth.isLoaded)
const authState = computed<AuthState>(() => unref(auth.authState) as AuthState)

const daemonPaired = ref<boolean | null>(null)

onMounted(async () => {
  if (!auth.isLoaded)
    await auth.load()

  try {
    const baseUrl = String(unref(auth.baseUrl) ?? '').trim().replace(/\/+$/, '')
    if (!baseUrl)
      return

    const response = await fetch(`${baseUrl}/health`)
    if (!response.ok)
      return

    const data = await response.json().catch(() => null) as any
    if (data && typeof data.paired === 'boolean')
      daemonPaired.value = data.paired
  }
  catch {
  }
})

const resetMessages = () => {
  error.value = ''
  notice.value = ''
}

const handleAuthenticate = async () => {
  resetMessages()
  const trimmedCode = code.value.trim()
  const trimmedToken = tokenInput.value.trim()

  if (trimmedToken && /^[a-f0-9]{64}$/i.test(trimmedToken)) {
    error.value = 'That looks like a hashed token from config.toml. You need the plaintext bearer token (starts with zc_) from a successful /pair response.'
    return
  }

  if (trimmedCode.length >= 6 && daemonPaired.value === true) {
    error.value = 'This daemon is already paired. Pairing codes are only available on first setup. Paste an existing bearer token (zc_...) or clear gateway.paired_tokens in config.toml and restart the daemon to re-pair.'
    return
  }

  if (trimmedCode.length >= 6) {
    loading.value = true
    try {
      await auth.pair(trimmedCode)
      code.value = ''
      tokenInput.value = ''
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : 'Pairing failed'
    }
    finally {
      loading.value = false
    }
    return
  }

  if (trimmedToken) {
    loading.value = true
    try {
      await auth.setToken(trimmedToken)
      tokenInput.value = ''
      code.value = ''
    }
    catch (err) {
      error.value = err instanceof Error ? err.message : 'Save failed'
    }
    finally {
      loading.value = false
    }
    return
  }

  error.value = 'Enter a pairing code or token to continue.'
}

const handleLogout = async () => {
  resetMessages()
  await auth.logout()
}
</script>

<template>
  <div class="h-full w-full bg-background select-none">
    <div class="grid h-full w-full grid-cols-2">
      <section class="min-w-0 flex items-center justify-center px-6 py-8 lg:px-12">
        <div class="w-full max-w-[400px] flex flex-col items-center">
          <span class="text-[28px] leading-none font-semibold tracking-wide text-emerald-600 sidebar-brand mb-2">GLOAMY</span>
          <h1 class="text-3xl font-semibold text-foreground mb-2 font-display text-center">
            {{ isAuthenticated ? 'Connected' : 'Authentication' }}
          </h1>

          <div class="w-full space-y-6">
            <div v-if="!isReady || authState === 'checking'" class="flex justify-center py-6">
              <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground/40" />
            </div>

            <div v-else-if="authState === 'unreachable'" class="text-sm text-red-500 font-bold text-center">
              Daemon unreachable. Start the daemon and confirm the API base URL.
            </div>

            <template v-else-if="!isAuthenticated">
              <div class="space-y-4">
                <div v-if="daemonPaired === true" class="text-xs text-muted-foreground text-center">
                  This daemon is already paired. If you lost your bearer token, clear gateway.paired_tokens in config.toml and restart the daemon to generate a new pairing code.
                </div>

                <div class="space-y-1.5">
                  <label class="text-xs font-medium text-foreground">Pairing code</label>
                  <input
                    v-model="code"
                    type="text"
                    maxlength="6"
                    class="w-full h-10 bg-muted/30 border border-border rounded-xl px-3 text-sm outline-none focus:border-primary/40 transition-all"
                    placeholder="Enter pairing code"
                    @keyup.enter="handleAuthenticate"
                    autofocus
                  >
                </div>

                <div class="flex items-center gap-3 py-2">
                  <div class="flex-1 h-px bg-border"></div>
                  <span class="text-xs font-medium text-muted-foreground">OR</span>
                  <div class="flex-1 h-px bg-border"></div>
                </div>

                <div class="space-y-1.5">
                  <label class="text-xs font-medium text-foreground">Bearer token</label>
                  <input
                    v-model="tokenInput"
                    type="password"
                    class="w-full h-10 bg-muted/30 border border-border rounded-xl px-3 text-sm outline-none focus:border-primary/40 transition-all"
                    placeholder="Paste token (optional)"
                    @keyup.enter="handleAuthenticate"
                  >
                </div>

                <button
                  class="w-full h-10 bg-foreground text-background rounded-xl text-sm font-semibold hover:opacity-90 active:scale-[0.98] transition-all disabled:opacity-40 disabled:active:scale-100"
                  :disabled="loading"
                  @click="handleAuthenticate"
                >
                  <span v-if="!loading">Continue</span>
                  <Icon v-else icon="hugeicons:loading-03" class="size-5 animate-spin mx-auto" />
                </button>
              </div>
            </template>

            <div v-else class="flex flex-col items-center">
              <button
                class="px-8 h-11 border border-border hover:bg-muted/30 rounded-xl text-sm font-medium transition-all active:scale-[0.98]"
                @click="handleLogout"
              >
                Sign Out
              </button>
            </div>

            <div class="min-h-[24px] flex flex-col items-center justify-center">
              <Transition name="fade">
                <p v-if="error" class="text-sm text-red-500 font-bold text-center">
                  {{ error }}
                </p>
                <p v-else-if="notice" class="text-sm text-green-500 font-bold text-center">
                  {{ notice }}
                </p>
              </Transition>
            </div>
          </div>

        </div>
      </section>

      <section class="min-w-0 relative overflow-hidden">
        <img
          src="/logimg.png"
          alt="Gloamy desktop preview"
          class="absolute inset-0 h-full w-full object-cover"
        >
      </section>
    </div>
  </div>
</template>

<style scoped>
.font-display {
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  letter-spacing: -0.02em;
}

.fade-enter-active, .fade-leave-active {
  transition: opacity 0.3s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.fade-slide-enter-active, .fade-slide-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}
.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(10px);
}
.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.expand-enter-active, .expand-leave-active {
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
  max-height: 100px;
  opacity: 1;
  overflow: hidden;
}
.expand-enter-from, .expand-leave-to {
  max-height: 0;
  opacity: 0;
  transform: translateY(-5px);
}
@import url('https://fonts.cdnfonts.com/css/plumby');

.sidebar-brand {
  font-family: 'Plumby', 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}

</style>
