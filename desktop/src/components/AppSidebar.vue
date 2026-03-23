<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { storeToRefs } from 'pinia'
import { computed, ref } from 'vue'
import { useRoute } from 'vue-router'
import { useUIStateStore } from '@/stores/uiState'
import ThemeSwitch from '@/components/ThemeSwitch.vue'

const route = useRoute()
const uiStore = useUIStateStore()
const { state } = storeToRefs(uiStore)

type LeafNavItem = { icon: string; label: string; to: string }
type GroupNavItem = { icon: string; label: string; children: LeafNavItem[] }

const isCollapsed = computed(() => state.value.leftSidebarCollapsed)

const navItems: Array<LeafNavItem | GroupNavItem> = [
  { icon: 'ph:chat-circle-text-fill', label: 'Agent Chat', to: '/' },
  { icon: 'ph:circles-four-fill', label: 'Dashboard', to: '/agent-chat' },
  { icon: 'ph:toolbox-fill', label: 'Tools', to: '/tools' },
  { icon: 'ph:calendar-check-fill', label: 'Cron Jobs', to: '/cron-jobs' },
  { icon: 'ph:puzzle-piece-fill', label: 'Integrations', to: '/integrations' },
  { icon: 'ph:archive-fill', label: 'Memory', to: '/memory' },
  { icon: 'ph:sliders-horizontal-fill', label: 'Configuration', to: '/configuration' },
  { icon: 'ph:megaphone-simple-fill', label: 'Cost Tracking', to: '/cost-tracking' },
  {
    icon: 'ph:heartbeat-fill',
    label: 'Diagnostics',
    children: [
      { icon: 'ph:pulse-fill', label: 'Logs', to: '/logs' },
      { icon: 'ph:first-aid-kit-fill', label: 'Doctor', to: '/doctor' },
    ],
  },
  { icon: 'ph:link-simple-fill', label: 'Authentication/Pairing', to: '/authentication' },
  { icon: 'ph:gear-six-fill', label: 'Settings & Theme', to: '/settings' },
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

const projects = [
  { name: 'Work', icon: 'ph:flask-fill' },
  { name: 'Calendar', icon: 'ph:calendar-blank-fill' },
]

const tasks = [
  { label: 'Optimize onboarding flow', icon: 'ph:arrow-arc-right-fill', color: 'text-blue-500' },
  { label: 'Prepare Q3 product roadmap', icon: 'ph:note-pencil-fill', color: 'text-muted-foreground' },
  { label: 'Analyze user feedback from beta test', icon: 'ph:note-pencil-fill', color: 'text-muted-foreground' },
  { label: 'Investigate slow page load reports', icon: 'ph:note-pencil-fill', color: 'text-muted-foreground' },
  { label: 'Document API integration guidelin...', icon: 'ph:note-pencil-fill', color: 'text-muted-foreground' },
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
    <div class="px-4 h-14 flex items-center gap-2" data-tauri-drag-region>
      <Icon icon="mdi:owl" class="size-5 text-foreground shrink-0" />
      <span v-if="!isCollapsed" class="text-sm font-semibold tracking-tight text-foreground">GLOAMY</span>
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
                ? 'text-foreground bg-muted/40'
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
                  ? 'text-foreground bg-muted/40'
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
              ? 'text-foreground bg-muted/40'
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

    <div v-if="!isCollapsed" class="mt-6 px-2 flex-1 overflow-y-auto">
      <div class="flex items-center justify-between px-3 mb-1">
        <span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Projects</span>
        <button class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:plus-fill" class="size-3.5" />
        </button>
      </div>
      <div class="flex flex-col gap-0.5">
        <button
          v-for="project in projects"
          :key="project.name"
          class="flex items-center gap-3 px-3 py-1.5 text-[13px] text-muted-foreground hover:text-foreground hover:bg-muted/25 rounded-md transition-colors duration-150"
        >
          <Icon :icon="project.icon" class="size-4 shrink-0" />
          <span>{{ project.name }}</span>
        </button>
      </div>

      <div class="flex items-center justify-between px-3 mt-5 mb-1">
        <span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider flex items-center gap-1">
          All tasks
          <Icon icon="ph:caret-down" class="size-3" />
        </span>
        <button class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:funnel-simple-fill" class="size-3.5" />
        </button>
      </div>
      <div class="flex flex-col gap-0.5">
        <button
          v-for="task in tasks"
          :key="task.label"
          class="flex items-center gap-3 px-3 py-1.5 text-[13px] text-muted-foreground hover:text-foreground hover:bg-muted/25 rounded-md transition-colors duration-150 text-left"
        >
          <Icon :icon="task.icon" :class="['size-4 shrink-0', task.color]" />
          <span class="truncate">{{ task.label }}</span>
        </button>
      </div>
    </div>

    <div class="mt-auto border-t border-border px-2 py-2 flex items-center" :class="isCollapsed ? 'justify-center' : 'justify-between px-4'">
      <div class="flex items-center gap-3">
        <ThemeSwitch />
        <button v-if="!isCollapsed" class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:squares-four-fill" class="size-4" />
        </button>
        <button v-if="!isCollapsed" class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:bookmark-simple-fill" class="size-4" />
        </button>
      </div>
    </div>
  </aside>
</template>
