<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { storeToRefs } from 'pinia'
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { useUIStateStore } from '@/stores/uiState'

const route = useRoute()
const uiStore = useUIStateStore()
const { state } = storeToRefs(uiStore)

const isCollapsed = computed(() => state.value.leftSidebarCollapsed)

function toggleCollapse() {
  uiStore.toggleLeftSidebar()
}
</script>

<template>
  <aside
    v-motion
    :initial="{ opacity: 0, x: -20 }"
    :enter="{ opacity: 1, x: 0, transition: { duration: 300 } }"
    class="h-full flex flex-col border-r border-border bg-card transition-all duration-300 ease-out"
    :class="isCollapsed ? 'w-16' : 'w-60'"
  >
    <!-- Logo -->
    <div class="px-4 h-12 border-b border-border flex items-center">
      <img
        v-if="!isCollapsed"
        src="/sidebarlogo.svg"
        alt="Desktop App"
        class="h-4 w-auto dark:invert dark:brightness-90 dark:sepia dark:-hue-rotate-15 dark:saturate-50"
      />
    </div>

    <!-- Navigation -->
    <nav class="flex-1 py-4 flex flex-col gap-1" :class="isCollapsed ? 'px-2' : 'px-3'">
      <RouterLink
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 100 } }"
        :hovered="{ x: 4 }"
        to="/"
        class="flex items-center gap-3 py-2.5 text-sm font-medium transition-all duration-200 rounded"
        :class="[
          route.path === '/' || route.path === '/home'
            ? 'text-foreground bg-muted/30'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/20',
          isCollapsed ? 'justify-center px-2.5' : 'px-3.5',
        ]"
        :title="isCollapsed ? $t('sidebar.home') : undefined"
      >
        <Icon icon="ph:house" class="size-5 shrink-0" />
        <Transition
          enter-active-class="transition-all duration-200"
          leave-active-class="transition-all duration-150"
          enter-from-class="opacity-0 -translate-x-2"
          leave-to-class="opacity-0 -translate-x-2"
        >
          <span v-if="!isCollapsed">{{ $t('sidebar.home') }}</span>
        </Transition>
      </RouterLink>

      <RouterLink
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 150 } }"
        :hovered="{ x: 4 }"
        to="/canvas"
        class="flex items-center gap-3 py-2.5 text-sm font-medium transition-all duration-200 rounded"
        :class="[
          route.path === '/canvas'
            ? 'text-foreground bg-muted/30'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/20',
          isCollapsed ? 'justify-center px-2.5' : 'px-3.5',
        ]"
        :title="isCollapsed ? 'Canvas' : undefined"
      >
        <Icon icon="ph:pencil-simple" class="size-5 shrink-0" />
        <Transition
          enter-active-class="transition-all duration-200"
          leave-active-class="transition-all duration-150"
          enter-from-class="opacity-0 -translate-x-2"
          leave-to-class="opacity-0 -translate-x-2"
        >
          <span v-if="!isCollapsed">Canvas</span>
        </Transition>
      </RouterLink>
    </nav>

    <!-- Footer with Settings -->
    <div class="py-4 border-t border-border" :class="isCollapsed ? 'px-2' : 'px-3'">
      <RouterLink
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 100 } }"
        :hovered="{ x: 4 }"
        to="/settings"
        class="flex items-center gap-3 py-2.5 text-sm font-medium transition-all duration-200 rounded"
        :class="[
          route.path === '/settings'
            ? 'text-foreground bg-muted/30'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/20',
          isCollapsed ? 'justify-center px-2.5' : 'px-3.5',
        ]"
        :title="isCollapsed ? $t('sidebar.settings') : undefined"
      >
        <Icon icon="ph:gear" class="size-5 shrink-0" />
        <Transition
          enter-active-class="transition-all duration-200"
          leave-active-class="transition-all duration-150"
          enter-from-class="opacity-0 -translate-x-2"
          leave-to-class="opacity-0 -translate-x-2"
        >
          <span v-if="!isCollapsed">{{ $t('sidebar.settings') }}</span>
        </Transition>
      </RouterLink>
    </div>
  </aside>
</template>
