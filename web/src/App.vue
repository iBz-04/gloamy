<script setup lang="ts">
import { onBeforeUnmount, onMounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { ConfigProvider } from 'reka-ui'
import { Icon } from '@iconify/vue'
import AppSidebar from '@/components/AppSidebar.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import { TooltipProvider } from '@/components/ui/tooltip'

import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const auth = useAuthStore()

onMounted(async () => {
  if (!auth.isLoaded)
    await auth.load()
})

// Watch for authentication state changes to handle logout/login redirection
watch(
  () => ({ authenticated: auth.isAuthenticated, loaded: auth.isLoaded }),
  ({ authenticated, loaded }) => {
    if (!loaded)
      return

    const currentPath = router.currentRoute.value.path
    if (!authenticated) {
      if (currentPath !== '/authentication') {
        router.replace('/authentication')
      }
    }
    else {
      if (currentPath === '/authentication') {
        const redirect = router.currentRoute.value.query.redirect
        const nextPath = typeof redirect === 'string' && redirect.trim().length > 0 ? redirect : '/'
        router.replace(nextPath)
      }
    }
  },
  { immediate: true },
)

const handleUnauthorized = () => {
  if (router.currentRoute.value.path !== '/authentication')
    router.push('/authentication')
}

function reloadWindow() {
  window.location.reload()
}

onMounted(() => {
  window.addEventListener('gloamy-unauthorized', handleUnauthorized)
})

onBeforeUnmount(() => {
  window.removeEventListener('gloamy-unauthorized', handleUnauthorized)
})
</script>

<template>
  <ConfigProvider>
    <TooltipProvider>
      <div v-if="auth.isLoaded" class="h-screen w-full bg-background text-foreground font-sans">
        <!-- Main Layout (Authenticated) -->
        <div v-if="auth.isAuthenticated && $route.path !== '/authentication'" class="flex h-full w-full">
          <AppSidebar />
          <main class="flex-1 overflow-hidden flex flex-col">
            <AppTopbar />
            <div class="flex-1 overflow-hidden">
              <router-view v-slot="{ Component, route }">
                <component :is="Component" v-if="Component" :key="route.fullPath" />
                <div v-else class="h-full w-full flex items-center justify-center px-6">
                  <div class="max-w-md w-full p-6 rounded-[4px] border border-border/50 bg-card/20 text-center">
                    <h3 class="text-lg font-medium text-foreground mb-2">Page failed to mount</h3>
                    <p class="text-sm text-muted-foreground mb-6">
                      The route changed, but no page component was available for render. Reload the window to recover the view tree.
                    </p>
                    <button
                      class="px-4 py-2 bg-primary text-primary-foreground rounded-[4px] text-sm font-medium hover:opacity-90 transition-opacity"
                      @click="reloadWindow"
                    >
                      Reload Window
                    </button>
                  </div>
                </div>
              </router-view>
            </div>
          </main>
        </div>

        <!-- Auth/Lock Layout (Unauthenticated or Auth Page) -->
        <div v-else class="h-full w-full">
          <router-view v-if="$route.path === '/authentication'" v-slot="{ Component }">
            <component :is="Component" v-if="Component" />
            <div v-else class="h-full w-full flex items-center justify-center px-6">
              <div class="max-w-md w-full p-6 rounded-[4px] border border-border/50 bg-card/20 text-center">
                <h3 class="text-lg font-medium text-foreground mb-2">Authentication view unavailable</h3>
                <p class="text-sm text-muted-foreground mb-6">
                  The authentication route did not resolve to a page component. Reload the window to retry.
                </p>
                <button
                  class="px-4 py-2 bg-primary text-primary-foreground rounded-[4px] text-sm font-medium hover:opacity-90 transition-opacity"
                  @click="reloadWindow"
                >
                  Reload Window
                </button>
              </div>
            </div>
          </router-view>
          <div v-else class="h-full w-full flex items-center justify-center">
            <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground/40" />
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-else class="h-screen w-full flex items-center justify-center bg-background">
        <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground/40" />
      </div>
    </TooltipProvider>
  </ConfigProvider>
</template>
