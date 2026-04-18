<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from 'vue'
import { useLocalStorage } from '@vueuse/core'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'
import type { StatusResponse, CostSummary } from '@/lib/types'

interface TrendPoint {
  ts: number
  value: number
}

interface CostTimelinePointResponse {
  ts_ms: number
  tokens: number
}

const TREND_HISTORY_KEY = 'dashboard-token-timeline-v3'
const TIMELINE_REFRESH_INTERVAL_MS = 60_000
const MAX_VISIBLE_TREND_POINTS = 90

const auth = useAuthStore()
const status = ref<StatusResponse | null>(null)
const cost = ref<CostSummary | null>(null)
const trendHistory = useLocalStorage<TrendPoint[]>(TREND_HISTORY_KEY, [])
const refreshTimer = ref<number | null>(null)
const trendRefreshTimer = ref<number | null>(null)
const loading = ref(true)
const error = ref<string | null>(null)

function isCostSummary(value: unknown): value is CostSummary {
  if (!value || typeof value !== 'object' || Array.isArray(value)) {
    return false
  }

  const candidate = value as Partial<CostSummary>
  return typeof candidate.session_cost_usd === 'number'
    && typeof candidate.daily_cost_usd === 'number'
    && typeof candidate.monthly_cost_usd === 'number'
    && typeof candidate.total_tokens === 'number'
    && typeof candidate.request_count === 'number'
    && !!candidate.by_model
    && typeof candidate.by_model === 'object'
}

function formatUSD(value: number): string {
  return `$${value.toFixed(4)}`
}

const maxCost = computed(() => {
  if (!cost.value) return 0.001
  return Math.max(
    cost.value.session_cost_usd,
    cost.value.daily_cost_usd,
    cost.value.monthly_cost_usd,
    0.001
  )
})

const healthStatus = (s: string) => {
  const st = s.toLowerCase()
  if (st === 'ok' || st === 'healthy') return 'success'
  if (st === 'warn' || st === 'warning' || st === 'degraded') return 'warning'
  return 'error'
}

const providerIcon = computed(() => {
  const provider = (status.value?.provider || '').toLowerCase()
  if (provider.includes('openai')) return 'simple-icons:openai'
  if (provider.includes('anthropic') || provider.includes('claude')) return 'simple-icons:anthropic'
  if (provider.includes('gemini') || provider.includes('google')) return 'simple-icons:googlegemini'
  if (provider.includes('groq')) return 'simple-icons:groq'
  if (provider.includes('mistral')) return 'simple-icons:mistralai'
  if (provider.includes('xai') || provider.includes('grok')) return 'simple-icons:xai'
  if (provider.includes('openrouter')) return 'simple-icons:openrouter'
  if (provider.includes('ollama')) return 'simple-icons:ollama'
  return 'hugeicons:cpu'
})

const componentCount = computed(() => {
  if (!status.value) return 0
  return Object.keys(status.value.health.components).length
})

const healthyCount = computed(() => {
  if (!status.value) return 0
  return Object.values(status.value.health.components)
    .filter(c => healthStatus(c.status) === 'success')
    .length
})

const healthLabel = computed(() => {
  if (componentCount.value === 0) return 'No data'
  if (healthyCount.value === componentCount.value) return 'Excellent'
  if (healthyCount.value >= Math.ceil(componentCount.value * 0.6)) return 'Stable'
  return 'Degraded'
})

const healthFilledBars = computed(() => {
  if (componentCount.value === 0) return 0
  return Math.round((healthyCount.value / componentCount.value) * 40)
})

const healthRatio = computed(() => {
  if (componentCount.value === 0) return 0
  return healthyCount.value / componentCount.value
})

const healthProgressColor = computed(() => {
  const low = { r: 106, g: 58, b: 42 }
  const high = { r: 45, g: 79, b: 30 }
  const t = Math.min(Math.max(healthRatio.value, 0), 1)
  const r = Math.round(low.r + (high.r - low.r) * t)
  const g = Math.round(low.g + (high.g - low.g) * t)
  const b = Math.round(low.b + (high.b - low.b) * t)
  return `rgb(${r}, ${g}, ${b})`
})

const visibleTrendHistory = computed(() => {
  const history = trendHistory.value
  if (history.length === 0) return []

  const firstNonZeroIndex = history.findIndex(point => point.value > 0)
  const trimmed = firstNonZeroIndex === -1
    ? history
    : history.slice(firstNonZeroIndex)

  return trimmed.slice(-MAX_VISIBLE_TREND_POINTS)
})

const trendChangePercent = computed(() => {
  const history = visibleTrendHistory.value
  if (history.length < 2) return 0
  const first = history[0]?.value ?? 0
  const last = history[history.length - 1]?.value ?? 0
  if (first <= 0) return 0
  return ((last - first) / first) * 100
})

const trendClass = computed(() => trendChangePercent.value >= 0 ? 'text-emerald-500' : 'text-rose-500')

function channelIcon(name: string): string {
  const n = String(name).toLowerCase()
  if (n.includes('slack')) return 'logos:slack-icon'
  if (n.includes('telegram')) return 'logos:telegram'
  if (n.includes('discord')) return 'logos:discord-icon'
  if (n.includes('whatsapp')) return 'logos:whatsapp-icon'
  if (n.includes('signal')) return 'logos:signal-icon'
  if (n.includes('matrix')) return 'logos:matrix'
  if (n.includes('teams')) return 'logos:microsoft-teams'
  if (n.includes('irc')) return 'logos:irc'
  if (n.includes('cli')) return 'hugeicons:computer-terminal-01'
  if (n.includes('web')) return 'hugeicons:globe'
  if (n.includes('clawdtalk')) return 'hugeicons:chat-01'
  return 'hugeicons:plug-01'
}

function normalizeTrendPoint(point: CostTimelinePointResponse): TrendPoint | null {
  if (!point || !Number.isFinite(point.ts_ms) || !Number.isFinite(point.tokens))
    return null

  return {
    ts: Math.floor(point.ts_ms),
    value: Math.max(Math.floor(point.tokens), 0),
  }
}

async function fetchTrendTimeline() {
  try {
    const response = await auth.fetchWithAuth<{ timeline: CostTimelinePointResponse[] }>('/api/cost/timeline')
    const mapped = (response.timeline ?? [])
      .map(normalizeTrendPoint)
      .filter((point): point is TrendPoint => point !== null)
      .sort((a, b) => a.ts - b.ts)

    trendHistory.value = mapped
  } catch (err) {
    console.error('[dashboard] fetchTrendTimeline error:', err)
  }
}

async function fetchData(showLoading = false) {
  if (showLoading) {
    loading.value = true
    error.value = null
    status.value = null
    cost.value = null
  }
  try {
    const [s, c] = await Promise.all([
      auth.fetchWithAuth<StatusResponse>('/api/status'),
      auth.fetchWithAuth<{ cost: CostSummary } | CostSummary>('/api/cost'),
    ])

    console.log('[dashboard] raw status response:', JSON.stringify(s))
    console.log('[dashboard] raw cost response:', JSON.stringify(c))

    status.value = s

    const latestCost = c && typeof c === 'object' && !Array.isArray(c) && 'cost' in c
      ? (c as { cost: CostSummary }).cost
      : (c as CostSummary)

    if (!isCostSummary(latestCost)) {
      throw new Error('Dashboard received an invalid cost payload.')
    }

    console.log('[dashboard] resolved cost object:', JSON.stringify(latestCost))
    cost.value = latestCost
  } catch (err: any) {
    console.error('[dashboard] fetchData error:', err)
    error.value = err.message || 'Failed to load dashboard'
  } finally {
    if (showLoading) {
      loading.value = false
    }
  }
}

const sparklinePoints = computed(() => {
  if (visibleTrendHistory.value.length === 0) return ''
  const values = visibleTrendHistory.value.map(point => Math.max(Math.floor(point.value), 0))
  const data: number[] = values.length === 1
    ? [values[0] ?? 0, values[0] ?? 0]
    : values
  const w = 600
  const h = 160
  const min = data.reduce((acc, n) => n < acc ? n : acc, data[0] ?? 0)
  const max = data.reduce((acc, n) => n > acc ? n : acc, data[0] ?? 0)
  const range = Math.max(max - min, 1)
  const pts = data.map((v, i) => {
    const x = (i / (data.length - 1)) * w
    const normalized = (v - min) / range
    const y = h - normalized * (h - 20) - 10
    return `${x},${y}`
  })
  return pts.join(' ')
})

onMounted(async () => {
  await Promise.all([
    fetchData(true),
    fetchTrendTimeline(),
  ])
  refreshTimer.value = window.setInterval(() => {
    fetchData(false)
  }, 10000)
  trendRefreshTimer.value = window.setInterval(() => {
    fetchTrendTimeline()
  }, TIMELINE_REFRESH_INTERVAL_MS)
})

onUnmounted(() => {
  if (refreshTimer.value !== null) {
    window.clearInterval(refreshTimer.value)
  }
  if (trendRefreshTimer.value !== null) {
    window.clearInterval(trendRefreshTimer.value)
  }
})
</script>

<template>
  <div class="h-full flex flex-col overflow-y-auto px-6 py-6 bg-background text-[15px]">
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground" />
    </div>

    <div v-else-if="error" class="flex-1 flex items-center justify-center px-6">
      <div class="max-w-md w-full p-6 rounded-[4px] border border-border/50 bg-destructive/5 text-center">
        <h3 class="text-lg font-medium text-foreground mb-2">Failed to load Dashboard</h3>
        <p class="text-sm text-muted-foreground mb-6">{{ error }}</p>
        <button
          @click="fetchData(true)"
          class="px-4 py-2 bg-primary text-primary-foreground rounded-[4px] text-sm font-medium hover:opacity-90 transition-opacity"
        >
          Try Again
        </button>
      </div>
    </div>

    <div v-else-if="status && cost" class="space-y-2.5 w-full">
      <!-- Main stats card with two columns -->
      <section>
        <div class="rounded-lg border border-border/50 bg-card/20 flex">
          <!-- Left: Health score with bar chart -->
          <div class="flex-1 p-4">
            <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-1">
              <Icon icon="hugeicons:activity-02" class="size-3.5" />
              <span>Health</span>
            </div>
            <div class="text-[28px] font-medium text-foreground tracking-tight leading-none">
              {{ healthyCount }}/{{ componentCount }}
            </div>
            <div class="text-[12px] text-muted-foreground mt-0.5 mb-3">
              {{ healthLabel }}
            </div>
            <!-- Vertical bar chart -->
            <div class="flex items-end gap-[2px] h-[32px]">
              <div 
                v-for="(_, i) in 40" 
                :key="i" 
                class="w-[3px] rounded-[1px]"
                :class="i < healthFilledBars ? 'h-full' : 'bg-border h-full'"
                :style="i < healthFilledBars ? { backgroundColor: healthProgressColor } : undefined"
              />
            </div>
          </div>

          <!-- Divider -->
          <div class="w-px bg-border/50 my-3" />

          <!-- Right: Provider info -->
          <div class="flex-1 p-4">
            <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-1">
              <Icon :icon="providerIcon" class="size-3.5" />
              <span>{{ status.provider || 'Provider' }}</span>
              <span class="text-[11px] text-muted-foreground/80">Provider</span>
            </div>
            <div class="text-[28px] font-medium text-foreground tracking-tight leading-none">
              {{ cost.total_tokens.toLocaleString() }}
            </div>
            <div class="text-[11px] text-muted-foreground mt-1">tokens</div>
            <div class="mt-6 flex items-center justify-between">
              <span class="text-[12px] text-muted-foreground">runtime trend</span>
              <span class="text-[12px]" :class="trendClass">{{ Math.abs(trendChangePercent).toFixed(1) }}%{{ trendChangePercent >= 0 ? '↑' : '↓' }}</span>
            </div>
          </div>
        </div>
      </section>

      <!-- Token Timeline chart -->
      <section>
        <div class="p-4 rounded-lg border border-border/50 bg-card/20">
          <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-0.5">
            <Icon icon="hugeicons:coins-01" class="size-3.5" />
            <span>Token Usage</span>
          </div>
          <div class="text-[24px] font-medium text-foreground tracking-tight leading-none">{{ cost.total_tokens.toLocaleString() }}</div>
          <div class="flex items-center gap-1.5 mt-1 mb-3">
            <span class="text-[12px] text-emerald-500">↑ {{ visibleTrendHistory.length }}</span>
            <span class="text-[12px] text-muted-foreground">daily samples</span>
          </div>
          <svg viewBox="0 0 600 160" class="w-full h-[200px]" preserveAspectRatio="none">
            <polyline
              :points="sparklinePoints"
              fill="none"
              stroke="currentColor"
              stroke-width="1.5"
              stroke-linejoin="round"
              stroke-linecap="round"
              class="text-foreground"
            />
          </svg>
        </div>
      </section>

      <div class="grid grid-cols-3 gap-2.5 auto-rows-[320px]">
        <!-- Cost Section -->
        <section>
          <div class="p-4 rounded-lg border border-border/50 bg-card/20 h-full">
            <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-3">
              <Icon icon="hugeicons:wallet-01" class="size-3.5" />
              <span>Cost Overview</span>
            </div>

            <div class="space-y-4">
              <div v-for="item in [
                { label: 'Session', value: cost.session_cost_usd, color: 'bg-foreground' },
                { label: 'Daily', value: cost.daily_cost_usd, color: 'bg-emerald-500' },
                { label: 'Monthly', value: cost.monthly_cost_usd, color: 'bg-primary' }
              ]" :key="item.label" class="space-y-1.5">
                <div class="flex justify-between items-end">
                  <span class="text-[12px] text-muted-foreground">{{ item.label }}</span>
                  <span class="text-[13px] font-mono text-foreground">{{ formatUSD(item.value) }}</span>
                </div>
                <div class="h-[8px] w-full bg-border overflow-hidden rounded-full">
                  <div
                    class="h-full transition-all duration-1000"
                    :class="item.color"
                    :style="{ width: `${Math.max((item.value / maxCost) * 100, 2)}%` }"
                  />
                </div>
              </div>
            </div>

            <div class="mt-4 pt-3 border-t border-border/50 grid grid-cols-2 gap-3">
              <div>
                <div class="text-[11px] text-muted-foreground mb-0.5">Total Cost</div>
                <div class="text-[18px] font-mono text-foreground tracking-tight">{{ formatUSD(cost.monthly_cost_usd) }}</div>
              </div>
              <div class="text-right">
                <div class="text-[11px] text-muted-foreground mb-0.5">Requests</div>
                <div class="text-[18px] font-mono text-foreground tracking-tight">{{ cost.request_count.toLocaleString() }}</div>
              </div>
            </div>
          </div>
        </section>

        <!-- Channels -->
        <section>
          <div class="p-4 rounded-lg border border-border/50 bg-card/20 h-full flex flex-col">
            <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-2">
              <Icon icon="hugeicons:chat-01" class="size-3.5" />
              <span>Channels</span>
            </div>
            <div class="space-y-0 flex-1 min-h-0 overflow-y-auto pr-1">
              <template v-for="(active, name) in status.channels" :key="name">
                <div v-if="active" class="flex items-center justify-between py-2 border-b border-border/30 last:border-0">
                  <div class="flex items-center gap-2">
                    <Icon :icon="channelIcon(String(name))" class="size-3.5" />
                    <span class="text-[13px] text-foreground capitalize">{{ name }}</span>
                  </div>
                  <span class="text-[11px] text-emerald-500">Active</span>
                </div>
              </template>
              <p v-if="!Object.values(status.channels).some(active => active)" class="text-[12px] text-muted-foreground py-1">
                No active channels
              </p>
            </div>
          </div>
        </section>

        <!-- Components -->
        <section>
          <div class="p-4 rounded-lg border border-border/50 bg-card/20 h-full flex flex-col">
            <div class="flex items-center gap-1.5 text-[12px] text-muted-foreground mb-2">
              <Icon icon="hugeicons:grid" class="size-3.5" />
              <span>Components</span>
            </div>
            <div class="space-y-0 flex-1 min-h-0 overflow-y-auto pr-1">
              <div
                v-for="(comp, name) in status.health.components"
                :key="name"
                class="flex items-center justify-between py-2 border-b border-border/30 last:border-0"
              >
                <span class="text-[13px] text-foreground capitalize">{{ name }}</span>
                <span
                  class="size-[10px] rounded-full"
                  :class="{
                    'bg-emerald-500': healthStatus(comp.status) === 'success',
                    'bg-orange-500': healthStatus(comp.status) === 'warning',
                    'bg-destructive': healthStatus(comp.status) === 'error',
                  }"
                />
              </div>
              <p v-if="Object.keys(status.health.components).length === 0" class="text-[12px] text-muted-foreground py-1">
                No components
              </p>
            </div>
          </div>
        </section>
      </div>
    </div>

    <div v-else class="flex-1 flex items-center justify-center px-6">
      <div class="max-w-md w-full p-6 rounded-[4px] border border-border/50 bg-card/20 text-center">
        <h3 class="text-lg font-medium text-foreground mb-2">Dashboard data unavailable</h3>
        <p class="text-sm text-muted-foreground mb-6">
          The page loaded, but the runtime summary payload was incomplete. Reload the panel to retry.
        </p>
        <button
          @click="fetchData(true)"
          class="px-4 py-2 bg-primary text-primary-foreground rounded-[4px] text-sm font-medium hover:opacity-90 transition-opacity"
        >
          Reload Dashboard
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.font-mono {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}
</style>
