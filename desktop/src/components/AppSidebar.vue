<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { ToolCaseIcon } from '@hugeicons/core-free-icons'
import { HugeiconsIcon } from '@hugeicons/vue'
import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useUIStateStore } from '@/stores/uiState'
const route = useRoute()
const uiStore = useUIStateStore()
const { state } = storeToRefs(uiStore)

type LeafNavItem = { icon?: string; hugeIcon?: object; label: string; to: string }
type GroupNavItem = { icon: string; label: string; children: LeafNavItem[] }

const isCollapsed = computed(() => state.value.leftSidebarCollapsed)

const navItems: Array<LeafNavItem | GroupNavItem> = [
  { icon: 'hugeicons:dashboard-square-01', label: 'Dashboard', to: '/' },
  { icon: 'hugeicons:chat-01', label: 'Cowork', to: '/cowork' },
  { hugeIcon: ToolCaseIcon, label: 'Tools', to: '/tools' },
  { icon: 'hugeicons:calendar-03', label: 'Cron Jobs', to: '/cron-jobs' },
  { icon: 'hugeicons:puzzle', label: 'Integrations', to: '/integrations' },
  { icon: 'hugeicons:archive', label: 'Memory', to: '/memory' },
  { icon: 'hugeicons:sliders-horizontal', label: 'Configuration', to: '/configuration' },
  {
    icon: 'hugeicons:activity-02',
    label: 'Diagnostics',
    children: [
      { icon: 'hugeicons:pulse-01', label: 'Logs', to: '/logs' },
      { icon: 'hugeicons:first-aid-kit', label: 'Doctor', to: '/doctor' },
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

const historyItems = [
  { label: 'Agent loop refactor', icon: 'hugeicons:message-01' },
  { label: 'UI polishing', icon: 'hugeicons:message-01' },
  { label: 'New API routes', icon: 'hugeicons:message-01' },
]
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
              icon="hugeicons:arrow-down-01"
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
              <Icon :icon="child.icon ?? 'hugeicons:circle'" class="size-4 shrink-0" />
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
          <HugeiconsIcon
            v-if="item.hugeIcon"
            :icon="item.hugeIcon"
            :size="18"
            color="currentColor"
            :stroke-width="1.5"
            class="shrink-0"
          />
          <Icon v-else :icon="item.icon ?? 'hugeicons:circle'" class="size-[18px] shrink-0" />
          <span v-if="!isCollapsed">{{ item.label }}</span>
        </RouterLink>
      </template>
    </nav>

    <div class="mt-8 flex flex-col px-2">
      <div
        v-if="!isCollapsed"
        v-motion
        :initial="{ opacity: 0 }"
        :enter="{ opacity: 1, transition: { delay: 300 } }"
        class="flex items-center justify-between px-3 mb-2"
      >
        <span class="text-[14px] font-medium text-muted-foreground/80">History</span>
        <div class="flex items-center gap-3 text-muted-foreground/70">
          <button class="hover:text-foreground transition-colors" title="Filter">
            <Icon icon="hugeicons:filter-horizontal" class="size-[15px]" />
          </button>
          <button class="hover:text-foreground transition-colors" title="New folder">
            <Icon icon="hugeicons:folder-add" class="size-[15px]" />
          </button>
        </div>
      </div>

      <div class="flex flex-col gap-0.5">
        <button
          v-for="(item, i) in historyItems"
          :key="i"
          v-motion
          :initial="{ opacity: 0, x: -10 }"
          :enter="{ opacity: 1, x: 0, transition: { delay: 350 + i * 30 } }"
          class="flex items-center gap-3 px-3 py-2 text-[13px] font-medium rounded-md transition-colors duration-150 w-full"
          :class="[
            isCollapsed ? 'justify-center !px-2' : '',
            'text-muted-foreground hover:text-foreground hover:bg-muted/25'
          ]"
          :title="isCollapsed ? item.label : undefined"
        >
          <Icon :icon="item.icon" class="size-4 shrink-0" />
          <span v-if="!isCollapsed" class="truncate">{{ item.label }}</span>
        </button>
      </div>
    </div>

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
        <Icon icon="hugeicons:settings-02" class="size-[18px] shrink-0" />
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
