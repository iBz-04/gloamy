<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface Integration {
  name: string
  description: string
  category: string
  status: 'Available' | 'Active' | 'ComingSoon'
}

const auth = useAuthStore()
const integrations = ref<Integration[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const activeCategory = ref('All')

const categories = computed(() => {
  const values = Array.from(new Set(integrations.value.map(item => item.category))).sort((a, b) => a.localeCompare(b))
  return ['All', ...values]
})

const filteredIntegrations = computed(() => {
  let result = integrations.value

  if (activeCategory.value !== 'All') {
    result = result.filter(item => item.category === activeCategory.value)
  }

  const query = searchQuery.value.trim().toLowerCase()
  if (query) {
    result = result.filter(item =>
      item.name.toLowerCase().includes(query)
      || item.description.toLowerCase().includes(query)
      || item.category.toLowerCase().includes(query),
    )
  }

  return result
})

const groupedIntegrations = computed(() => {
  const groups: Record<string, Integration[]> = {}

  for (const integration of filteredIntegrations.value) {
    if (!groups[integration.category]) {
      groups[integration.category] = []
    }
    const category = groups[integration.category]
    if (category) {
      category.push(integration)
    }
  }

  return Object.fromEntries(
    Object.entries(groups).sort(([a], [b]) => a.localeCompare(b)),
  ) as Record<string, Integration[]>
})

function normalizeResponse(response: Integration[] | { integrations?: Integration[] }): Integration[] {
  if (Array.isArray(response)) {
    return response
  }
  if (response && Array.isArray(response.integrations)) {
    return response.integrations
  }
  return []
}

function statusIcon(status: Integration['status']): string {
  if (status === 'Active') {
    return 'ph:check-circle-fill'
  }
  if (status === 'Available') {
    return 'ph:plus-circle-fill'
  }
  return 'ph:clock-countdown-fill'
}

function statusClass(status: Integration['status']): string {
  if (status === 'Active') {
    return 'text-emerald-500'
  }
  if (status === 'Available') {
    return 'text-sky-500'
  }
  return 'text-muted-foreground'
}

function statusLabel(status: Integration['status']): string {
  if (status === 'ComingSoon') {
    return 'Coming Soon'
  }
  return status
}

async function fetchIntegrations(showLoading = true) {
  if (showLoading) {
    loading.value = true
  }

  error.value = null

  try {
    const response = await auth.fetchWithAuth<Integration[] | { integrations?: Integration[] }>('/api/integrations')
    integrations.value = normalizeResponse(response)
  }
  catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to load integrations'
    integrations.value = []
  }
  finally {
    if (showLoading) {
      loading.value = false
    }
  }
}

onMounted(() => {
  fetchIntegrations()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30 space-y-4">
      <div class="flex items-center gap-3">
        <div class="relative flex-1 max-w-xl">
          <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search integrations..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          >
        </div>

        <button
          class="px-3 py-2 text-[12px] font-medium rounded-xl border border-border/60 hover:bg-card/60 transition-colors"
          @click="fetchIntegrations()"
        >
          Refresh
        </button>
      </div>

      <div class="flex items-center gap-1 overflow-x-auto pb-1">
        <button
          v-for="cat in categories"
          :key="cat"
          class="px-3 py-1.5 text-[12px] font-medium rounded-xl whitespace-nowrap transition-colors"
          :class="activeCategory === cat
            ? 'bg-foreground text-background'
            : 'text-muted-foreground hover:text-foreground hover:bg-card/50'"
          @click="activeCategory = cat"
        >
          {{ cat }}
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-4">
      <div v-if="loading" class="space-y-2">
        <div
          v-for="i in 8"
          :key="i"
          class="h-20 rounded-xl bg-card/30 border border-border/30 animate-pulse"
        />
      </div>

      <template v-else>
        <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
          <Icon icon="ph:warning" class="size-4" />
          <span>{{ error }}</span>
        </div>

        <div v-if="Object.keys(groupedIntegrations).length === 0" class="flex items-center justify-center h-full">
          <p class="text-muted-foreground text-[12px]">No integrations found.</p>
        </div>

        <div v-else class="space-y-6">
          <section v-for="(integrationList, groupName) in groupedIntegrations" :key="groupName">
            <h2 class="font-sans text-[20px] font-medium text-foreground tracking-tight mb-3">{{ groupName }}</h2>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3">
              <div
                v-for="integration in integrationList"
                :key="integration.name"
                class="p-4 rounded-2xl border border-border/40 bg-card/20 hover:bg-card/40 hover:border-border/60 transition-all"
              >
                <div class="flex items-center gap-2 pr-2 mb-1">
                  <Icon icon="ph:puzzle-piece-fill" class="size-[16px] text-primary shrink-0" />
                  <h3 class="font-sans text-[15px] font-medium text-foreground truncate">{{ integration.name }}</h3>
                </div>

                <p class="text-[13px] text-muted-foreground line-clamp-2 mb-3">
                  {{ integration.description }}
                </p>

                <div class="flex items-center justify-between gap-2">
                  <span
                    class="inline-flex items-center gap-1.5 text-[11px] font-medium"
                    :class="statusClass(integration.status)"
                  >
                    <Icon :icon="statusIcon(integration.status)" class="size-3.5" />
                    {{ statusLabel(integration.status) }}
                  </span>
                  <span class="text-[11px] text-muted-foreground truncate" :title="integration.category">
                    {{ integration.category }}
                  </span>
                </div>
              </div>
            </div>
          </section>
        </div>
      </template>
    </div>
  </div>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
