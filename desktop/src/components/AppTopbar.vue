<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const pageLabelMap: Record<string, string> = {
  dashboard: 'Runtime Overview',
  tools: 'Tool Controls',
  'cron-jobs': 'Cron Schedules',
  integrations: 'Integration Catalog',
  memory: 'Memory Records',
  configuration: 'System Config',
  'cost-tracking': 'Cost Tracking',
  logs: 'System Logs',
  doctor: 'Runtime Doctor',
  authentication: 'Access Pairing',
  settings: 'Theme Settings',
}

const pageDescriptor = computed(() => {
  const routeName = typeof route.name === 'string' ? route.name : ''
  return pageLabelMap[routeName] ?? 'Page Overview'
})
</script>

<template>
  <header
    class="h-12 px-4 shrink-0 select-none flex items-center"
    data-tauri-drag-region
  >
    <span class="text-[12px] font-medium tracking-wide text-muted-foreground/90">
      {{ pageDescriptor }}
    </span>
  </header>
</template>
