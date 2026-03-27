<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface ToolSpec {
  name: string
  description: string
  parameters: unknown
}

interface CliTool {
  name: string
  path: string
  version: string | null
  category: string
}

const auth = useAuthStore()
const tools = ref<ToolSpec[]>([])
const cliTools = ref<CliTool[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const expandedTool = ref<string | null>(null)

const filteredTools = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return tools.value
  }

  return tools.value.filter(tool =>
    tool.name.toLowerCase().includes(query)
    || tool.description.toLowerCase().includes(query),
  )
})

const filteredCliTools = computed(() => {
  const query = searchQuery.value.trim().toLowerCase()
  if (!query) {
    return cliTools.value
  }

  return cliTools.value.filter(tool =>
    tool.name.toLowerCase().includes(query)
    || tool.category.toLowerCase().includes(query)
    || tool.path.toLowerCase().includes(query),
  )
})

function normalizeToolsResponse(response: ToolSpec[] | { tools?: ToolSpec[] }): ToolSpec[] {
  if (Array.isArray(response)) {
    return response
  }
  if (response && Array.isArray(response.tools)) {
    return response.tools
  }
  return []
}

function normalizeCliToolsResponse(response: CliTool[] | { cli_tools?: CliTool[] }): CliTool[] {
  if (Array.isArray(response)) {
    return response
  }
  if (response && Array.isArray(response.cli_tools)) {
    return response.cli_tools
  }
  return []
}

async function fetchData(showLoading = true) {
  if (showLoading) {
    loading.value = true
  }

  error.value = null

  try {
    const [toolsResponse, cliResponse] = await Promise.all([
      auth.fetchWithAuth<ToolSpec[] | { tools?: ToolSpec[] }>('/api/tools'),
      auth.fetchWithAuth<CliTool[] | { cli_tools?: CliTool[] }>('/api/cli-tools'),
    ])

    tools.value = normalizeToolsResponse(toolsResponse)
    cliTools.value = normalizeCliToolsResponse(cliResponse)
  }
  catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to load tools'
    tools.value = []
    cliTools.value = []
  }
  finally {
    if (showLoading) {
      loading.value = false
    }
  }
}

function toggleExpanded(toolName: string) {
  expandedTool.value = expandedTool.value === toolName ? null : toolName
}

function formatParams(params: unknown): string {
  return JSON.stringify(params ?? {}, null, 2)
}

onMounted(() => {
  fetchData()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30">
      <div class="flex items-center gap-3">
        <div class="relative flex-1 max-w-xl">
          <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search tools..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-card/50 border border-border/50 rounded-xl focus:outline-none focus:ring-1 focus:ring-primary/50 focus:border-primary/50 text-foreground placeholder:text-muted-foreground"
          >
        </div>

        <button
          class="px-3 py-2 text-[12px] font-medium rounded-xl border border-border/60 hover:bg-card/60 transition-colors"
          @click="fetchData()"
        >
          Refresh
        </button>
      </div>
    </div>

    <div class="flex-1 overflow-y-auto px-6 py-4 space-y-6">
      <div v-if="loading" class="space-y-2">
        <div
          v-for="i in 8"
          :key="i"
          class="h-16 rounded-xl bg-card/30 border border-border/30 animate-pulse"
        />
      </div>

      <template v-else>
        <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
          <Icon icon="ph:warning" class="size-4" />
          <span>{{ error }}</span>
        </div>

        <section>
          <div class="flex items-center gap-2 mb-3">
            <Icon icon="ph:toolbox-fill" class="size-4 text-primary" />
            <h2 class="text-[14px] font-semibold text-foreground">Agent Tools ({{ filteredTools.length }})</h2>
          </div>

          <div v-if="filteredTools.length === 0" class="text-[12px] text-muted-foreground">
            No agent tools found.
          </div>

          <div v-else class="grid grid-cols-1 lg:grid-cols-2 gap-3">
            <div
              v-for="tool in filteredTools"
              :key="tool.name"
              class="rounded-xl border border-border/40 bg-card/20 overflow-hidden"
            >
              <button
                class="w-full px-4 py-3 text-left hover:bg-card/40 transition-colors"
                @click="toggleExpanded(tool.name)"
              >
                <div class="flex items-start justify-between gap-3">
                  <div class="min-w-0">
                    <p class="text-[13px] font-medium text-foreground truncate">{{ tool.name }}</p>
                    <p class="text-[12px] text-muted-foreground mt-1 line-clamp-2">{{ tool.description }}</p>
                  </div>
                  <Icon :icon="expandedTool === tool.name ? 'ph:caret-up' : 'ph:caret-down'" class="size-4 text-muted-foreground flex-shrink-0 mt-0.5" />
                </div>
              </button>

              <div v-if="expandedTool === tool.name" class="border-t border-border/30 p-3">
                <p class="text-[11px] font-medium text-muted-foreground mb-2">Parameter schema</p>
                <pre class="text-[11px] leading-relaxed text-foreground font-mono bg-card/40 rounded-lg p-2 overflow-x-auto">{{ formatParams(tool.parameters) }}</pre>
              </div>
            </div>
          </div>
        </section>

        <section>
          <div class="flex items-center gap-2 mb-3">
            <Icon icon="ph:terminal-window" class="size-4 text-primary" />
            <h2 class="text-[14px] font-semibold text-foreground">CLI Tools ({{ filteredCliTools.length }})</h2>
          </div>

          <div v-if="filteredCliTools.length === 0" class="text-[12px] text-muted-foreground">
            No CLI tools found.
          </div>

          <div v-else class="border border-border/40 rounded-xl overflow-hidden">
            <div class="grid grid-cols-[180px_1fr_120px_120px] gap-4 px-4 py-3 bg-card/30 border-b border-border/30 text-[12px] font-medium text-muted-foreground">
              <div>Name</div>
              <div>Path</div>
              <div>Version</div>
              <div>Category</div>
            </div>

            <div
              v-for="tool in filteredCliTools"
              :key="`${tool.name}-${tool.path}`"
              class="grid grid-cols-[180px_1fr_120px_120px] gap-4 px-4 py-3 border-b border-border/20 last:border-b-0 text-[12px] items-center"
            >
              <div class="text-foreground font-medium truncate">{{ tool.name }}</div>
              <div class="text-muted-foreground font-mono truncate" :title="tool.path">{{ tool.path }}</div>
              <div class="text-muted-foreground">{{ tool.version || '—' }}</div>
              <div class="text-muted-foreground truncate">{{ tool.category }}</div>
            </div>
          </div>
        </section>
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
