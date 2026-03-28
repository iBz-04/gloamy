<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useUIStateStore } from '@/stores/uiState'
const route = useRoute()
const uiStore = useUIStateStore()
const { state } = storeToRefs(uiStore)

type LeafNavItem = { icon: string; label: string; to: string }
type GroupNavItem = { icon: string; label: string; children: LeafNavItem[] }

const isCollapsed = computed(() => state.value.leftSidebarCollapsed)

const navItems: Array<LeafNavItem | GroupNavItem> = [
  { icon: 'ph:circles-four-fill', label: 'Dashboard', to: '/' },
  { icon: 'ph:toolbox-fill', label: 'Tools', to: '/tools' },
  { icon: 'ph:calendar-check-fill', label: 'Cron Jobs', to: '/cron-jobs' },
  { icon: 'ph:puzzle-piece-fill', label: 'Integrations', to: '/integrations' },
  { icon: 'ph:archive-fill', label: 'Memory', to: '/memory' },
  { icon: 'ph:sliders-horizontal-fill', label: 'Configuration', to: '/configuration' },
  {
    icon: 'ph:heartbeat-fill',
    label: 'Diagnostics',
    children: [
      { icon: 'ph:pulse-fill', label: 'Logs', to: '/logs' },
      { icon: 'ph:first-aid-kit-fill', label: 'Doctor', to: '/doctor' },
    ],
  },
]

const openGroups = ref<Record<string, boolean>>({
  Diagnostics: true,
})

const toggleGroup = (group: string) => {
  openGroups.value[group] = !openGroups.value[group]
}

const isGroupOpen = (group: string) => openGroups.value[group] ?? false

const isRouteActive = (path: string) => route.path === path

const isGroupActive = (children: LeafNavItem[]) => children.some((child) => isRouteActive(child.to))

const isGroupNavItem = (item: LeafNavItem | GroupNavItem): item is GroupNavItem => 'children' in item

const getNavItemKey = (item: LeafNavItem | GroupNavItem) => (isGroupNavItem(item) ? `group-${item.label}` : item.to)
</script>

<template>
  <aside
    v-motion
    :initial="{ opacity: 0, x: -20 }"
    :enter="{ opacity: 1, x: 0, transition: { duration: 300 } }"
    class="h-full flex flex-col bg-card transition-all duration-300 ease-out select-none"
    :class="isCollapsed ? 'w-16' : 'w-[220px]'"
  >
    <div class="px-4 h-14 flex items-center" data-tauri-drag-region>
      <span v-if="!isCollapsed" class="text-[28px] leading-none font-semibold tracking-wide text-emerald-600 sidebar-brand">GLOAMY</span>
    </div>

    <nav class="flex flex-col gap-0.5 px-2">
      <template v-for="(item, i) in navItems" :key="getNavItemKey(item)">
        <div
          v-if="isGroupNavItem(item)"
          v-motion
          :initial="{ opacity: 0, x: -10 }"
          :enter="{ opacity: 1, x: 0, transition: { delay: 50 + i * 30 } }"
          class="flex flex-col"
        >
          <button
            type="button"
            class="flex items-center gap-3 px-3 py-2 text-[13px] font-medium rounded-md transition-colors duration-150 w-full"
            :class="[
              isGroupActive(item.children)
                ? 'text-foreground'
                : 'text-muted-foreground hover:text-foreground hover:bg-muted/25',
              isCollapsed ? 'justify-center !px-2' : '',
            ]"
            :title="isCollapsed ? item.label : undefined"
            @click="toggleGroup(item.label)"
          >
            <Icon :icon="item.icon" class="size-[18px] shrink-0" />
            <span v-if="!isCollapsed">{{ item.label }}</span>
            <Icon
              v-if="!isCollapsed"
              icon="ph:caret-down"
              class="size-3 ml-auto transition-transform duration-200"
              :class="isGroupOpen(item.label) ? 'rotate-0' : '-rotate-90'"
            />
          </button>

          <div
            v-if="!isCollapsed && isGroupOpen(item.label)"
            v-motion
            :initial="{ opacity: 0, scaleY: 0.85 }"
            :enter="{ opacity: 1, scaleY: 1, transition: { duration: 260, easing: 'easeOut' } }"
            :leave="{ opacity: 0, scaleY: 0.85, transition: { duration: 200, easing: 'easeIn' } }"
            class="mt-0.5 flex flex-col gap-0.5 overflow-hidden origin-top"
          >
            <RouterLink
              v-for="child in item.children"
              :key="child.to"
              :to="child.to"
              class="flex items-center gap-2 pl-10 pr-3 py-1.5 text-[12px] font-medium rounded-md transition-colors duration-150"
              :class="[
                isRouteActive(child.to)
                  ? 'text-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-muted/25',
              ]"
            >
              <Icon :icon="child.icon" class="size-4 shrink-0" />
              <span>{{ child.label }}</span>
            </RouterLink>
          </div>
        </div>

        <RouterLink
          v-else
          v-motion
          :initial="{ opacity: 0, x: -10 }"
          :enter="{ opacity: 1, x: 0, transition: { delay: 50 + i * 30 } }"
          :to="item.to"
          class="flex items-center gap-3 px-3 py-2 text-[13px] font-medium rounded-md transition-colors duration-150"
          :class="[
            isRouteActive(item.to)
              ? 'text-foreground'
              : 'text-muted-foreground hover:text-foreground hover:bg-muted/25',
            isCollapsed ? 'justify-center !px-2' : '',
          ]"
          :title="isCollapsed ? item.label : undefined"
        >
          <Icon :icon="item.icon" class="size-[18px] shrink-0" />
          <span v-if="!isCollapsed">{{ item.label }}</span>
        </RouterLink>
      </template>
    </nav>

    <div class="mt-auto px-2 pb-4 pt-2">
      <RouterLink
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 300 } }"
        to="/settings"
        class="flex items-center gap-3 px-3 py-2 text-[13px] font-medium rounded-md transition-colors duration-150"
        :class="[
          isRouteActive('/settings')
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/25',
          isCollapsed ? 'justify-center !px-2' : '',
        ]"
        :title="isCollapsed ? 'Settings' : undefined"
      >
        <Icon icon="ph:gear-six-fill" class="size-[18px] shrink-0" />
        <span v-if="!isCollapsed">Settings</span>
      </RouterLink>
    </div>
  </aside>
</template>

<style scoped>
@import url('https://fonts.cdnfonts.com/css/plumby');

.sidebar-brand {
  font-family: 'Plumby', 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
}
</style>
