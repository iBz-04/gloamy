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
              <router-view v-slot="{ Component }">
                <Transition name="page" mode="out-in">
                  <component :is="Component" />
                </Transition>
              </router-view>
            </div>
          </main>
        </div>

        <!-- Auth/Lock Layout (Unauthenticated or Auth Page) -->
        <div v-else class="h-full w-full">
          <router-view v-if="$route.path === '/authentication'" v-slot="{ Component }">
            <Transition name="page" mode="out-in">
              <component :is="Component" />
            </Transition>
          </router-view>
          <div v-else class="h-full w-full flex items-center justify-center">
            <Icon icon="ph:circle-notch" class="size-6 animate-spin text-muted-foreground/40" />
          </div>
        </div>
      </div>

      <!-- Loading State -->
      <div v-else class="h-screen w-full flex items-center justify-center bg-background">
        <Icon icon="ph:circle-notch" class="size-6 animate-spin text-muted-foreground/40" />
      </div>
    </TooltipProvider>
  </ConfigProvider>
</template>

<style>
.page-enter-active,
.page-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
