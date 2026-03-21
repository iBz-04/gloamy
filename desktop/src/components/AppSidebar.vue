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

const navItems = [
  { icon: 'ph:note-pencil', label: 'New task', to: '/' },
  { icon: 'ph:magnifying-glass', label: 'Search', to: '/search' },
  { icon: 'ph:users-three', label: 'Agents', to: '/agents' },
  { icon: 'ph:bookmark-simple', label: 'Library', to: '/library' },
]

const projects = [
  { name: 'Work', icon: 'ph:flask' },
  { name: 'Calendar', icon: 'ph:calendar-blank' },
]

const tasks = [
  { label: 'Optimize onboarding flow', icon: 'ph:arrow-arc-right', color: 'text-blue-500' },
  { label: 'Prepare Q3 product roadmap', icon: 'ph:note-pencil', color: 'text-muted-foreground' },
  { label: 'Analyze user feedback from beta test', icon: 'ph:note-pencil', color: 'text-muted-foreground' },
  { label: 'Investigate slow page load reports', icon: 'ph:note-pencil', color: 'text-muted-foreground' },
  { label: 'Document API integration guidelin...', icon: 'ph:note-pencil', color: 'text-muted-foreground' },
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
      <Icon icon="ph:circuitry" class="size-5 text-foreground shrink-0" />
      <span v-if="!isCollapsed" class="text-sm font-semibold tracking-tight text-foreground">gloamy</span>
    </div>

    <nav class="flex flex-col gap-0.5 px-2">
      <RouterLink
        v-for="(item, i) in navItems"
        :key="item.to"
        v-motion
        :initial="{ opacity: 0, x: -10 }"
        :enter="{ opacity: 1, x: 0, transition: { delay: 50 + i * 30 } }"
        :to="item.to"
        class="flex items-center gap-3 px-3 py-2 text-[13px] font-medium rounded-md transition-colors duration-150"
        :class="[
          route.path === item.to
            ? 'text-foreground bg-muted/40'
            : 'text-muted-foreground hover:text-foreground hover:bg-muted/25',
          isCollapsed ? 'justify-center !px-2' : '',
        ]"
        :title="isCollapsed ? item.label : undefined"
      >
        <Icon :icon="item.icon" class="size-[18px] shrink-0" />
        <span v-if="!isCollapsed">{{ item.label }}</span>
      </RouterLink>
    </nav>

    <div v-if="!isCollapsed" class="mt-6 px-2 flex-1 overflow-y-auto">
      <div class="flex items-center justify-between px-3 mb-1">
        <span class="text-[11px] font-medium text-muted-foreground uppercase tracking-wider">Projects</span>
        <button class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:plus" class="size-3.5" />
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
          <Icon icon="ph:funnel-simple" class="size-3.5" />
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
        <button class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:sliders-horizontal" class="size-4" />
        </button>
        <button v-if="!isCollapsed" class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:squares-four" class="size-4" />
        </button>
        <button v-if="!isCollapsed" class="text-muted-foreground hover:text-foreground transition-colors">
          <Icon icon="ph:bookmark-simple" class="size-4" />
        </button>
      </div>
    </div>
  </aside>
</template>
