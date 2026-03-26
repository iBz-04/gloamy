<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  source: string
  message: string
}

const auth = useAuthStore()
const logs = ref<LogEntry[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const activeLevel = ref<'All' | LogLevel>('All')
const autoRefresh = ref(true)
const refreshInterval = ref<ReturnType<typeof setInterval> | null>(null)

const levels: Array<'All' | LogLevel> = ['All', 'debug', 'info', 'warn', 'error']

const filteredLogs = computed(() => {
  let result = logs.value

  if (activeLevel.value !== 'All') {
    result = result.filter(log => log.level === activeLevel.value)
  }

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(log =>
      log.message.toLowerCase().includes(q) ||
      log.source.toLowerCase().includes(q)
    )
  }

  return result
})

const levelCounts = computed(() => {
  const counts: Record<LogLevel, number> = { debug: 0, info: 0, warn: 0, error: 0 }
  for (const log of logs.value) {
    counts[log.level]++
  }
  return counts
})

function levelIcon(level: LogLevel): string {
  switch (level) {
    case 'debug': return 'ph:bug'
    case 'info': return 'ph:info'
    case 'warn': return 'ph:warning'
    case 'error': return 'ph:x-circle'
  }
}

function levelColor(level: LogLevel): string {
  switch (level) {
    case 'debug': return 'text-slate-400'
    case 'info': return 'text-blue-500'
    case 'warn': return 'text-amber-500'
    case 'error': return 'text-red-500'
  }
}

function formatTimestamp(date: Date): string {
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  const seconds = date.getSeconds().toString().padStart(2, '0')
  const ms = date.getMilliseconds().toString().padStart(3, '0')
  return `${hours}:${minutes}:${seconds}.${ms}`
}

function formatDate(date: Date): string {
  const day = date.getDate()
  const month = date.toLocaleString('en-US', { month: 'short' })
  return `${month} ${day}`
}

function formatLevelLabel(level: string): string {
  if (level === 'All') return 'All'
  return level.charAt(0).toUpperCase() + level.slice(1)
}

async function fetchLogs() {
  error.value = null
  try {
    const response = await auth.fetchWithAuth<{ logs: any[] } | any[]>('/api/logs')
    let rawLogs: any[] = []
    if (Array.isArray(response)) {
      rawLogs = response
    } else if (response && 'logs' in response) {
      rawLogs = response.logs
    }

    logs.value = rawLogs.map((log, index) => ({
      id: log.id ?? `log-${index}`,
      timestamp: new Date(log.timestamp ?? Date.now()),
      level: (log.level ?? 'info') as LogLevel,
      source: log.source ?? log.module ?? 'system',
      message: log.message ?? log.msg ?? '',
    }))
  } catch (err: any) {
    console.error('[logs] fetchLogs error:', err)
    error.value = err.message || 'Failed to load logs'
    logs.value = []
  } finally {
    loading.value = false
  }
}

function clearLogs() {
  logs.value = []
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value
  if (autoRefresh.value) {
    startAutoRefresh()
  } else {
    stopAutoRefresh()
  }
}

function startAutoRefresh() {
  if (refreshInterval.value) return
  refreshInterval.value = setInterval(() => {
    fetchLogs()
  }, 5000)
}

function stopAutoRefresh() {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
    refreshInterval.value = null
  }
}

onMounted(() => {
  fetchLogs()
  if (autoRefresh.value) {
    startAutoRefresh()
  }
})

onUnmounted(() => {
  stopAutoRefresh()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4">
      <!-- Search and controls -->
      <div class="flex items-center justify-between gap-4 mb-4">
        <div class="relative flex-1 max-w-md">
          <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search logs..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-transparent focus:outline-none text-foreground placeholder:text-muted-foreground"
          />
        </div>

        <div class="flex items-center gap-2">
          <button
            @click="toggleAutoRefresh"
            class="flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md transition-colors"
            :class="autoRefresh ? 'text-primary' : 'text-muted-foreground hover:text-foreground'"
          >
            <Icon :icon="autoRefresh ? 'ph:pause' : 'ph:play'" class="size-3.5" />
            {{ autoRefresh ? 'Live' : 'Paused' }}
          </button>
          <button
            @click="fetchLogs"
            class="flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md text-muted-foreground hover:text-foreground transition-colors"
          >
            <Icon icon="ph:arrow-clockwise" class="size-3.5" />
            Refresh
          </button>
          <button
            @click="clearLogs"
            class="flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md text-muted-foreground hover:text-foreground transition-colors"
          >
            <Icon icon="ph:trash" class="size-3.5" />
            Clear
          </button>
        </div>
      </div>

      <!-- Level filters -->
      <div class="flex items-center gap-1">
        <button
          v-for="level in levels"
          :key="level"
          @click="activeLevel = level"
          class="px-3 py-1.5 text-[12px] font-medium rounded-md whitespace-nowrap transition-colors flex items-center gap-1.5"
          :class="activeLevel === level
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground'"
        >
          <Icon v-if="level !== 'All'" :icon="levelIcon(level)" class="size-3.5" />
          {{ formatLevelLabel(level) }}
          <span v-if="level !== 'All'" class="text-[10px] opacity-70">({{ levelCounts[level] }})</span>
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Table Header Skeleton -->
      <div class="grid grid-cols-[100px_70px_100px_1fr] gap-4 px-4 py-3 text-[12px] font-medium text-muted-foreground">
        <div>Time</div>
        <div>Level</div>
        <div>Source</div>
        <div>Message</div>
      </div>
      <!-- Skeleton rows -->
      <div
        v-for="i in 8"
        :key="i"
        class="grid grid-cols-[100px_70px_100px_1fr] gap-4 px-4 py-2.5 items-center"
      >
        <div class="h-3 bg-muted-foreground/10 rounded-sm animate-pulse w-16" />
        <div class="h-3 bg-muted-foreground/10 rounded-sm animate-pulse w-10" />
        <div class="h-3 bg-muted-foreground/10 rounded-sm animate-pulse w-14" />
        <div class="h-3 bg-muted-foreground/10 rounded-sm animate-pulse w-full" />
      </div>
    </div>

    <!-- Content -->
    <div v-else class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Error banner -->
      <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
        <Icon icon="ph:warning" class="size-4" />
        <span>{{ error }}</span>
      </div>

      <!-- Empty state -->
      <div v-if="filteredLogs.length === 0" class="flex flex-col items-center justify-center py-16">
        <Icon icon="ph:scroll" class="size-12 text-muted-foreground/30 mb-3" />
        <p class="text-muted-foreground text-[13px]">No logs found</p>
      </div>

      <!-- Logs table -->
      <div v-else>
        <!-- Table Header -->
        <div class="grid grid-cols-[100px_70px_100px_1fr] gap-4 px-4 py-3 text-[12px] font-medium text-muted-foreground">
          <div>Time</div>
          <div>Level</div>
          <div>Source</div>
          <div>Message</div>
        </div>

        <!-- Table Rows -->
        <div
          v-for="log in filteredLogs"
          :key="log.id"
          class="grid grid-cols-[100px_70px_100px_1fr] gap-4 px-4 py-2.5 items-center text-[13px] group"
        >
          <!-- Timestamp -->
          <div class="text-muted-foreground text-[11px] font-mono">
            <span class="text-muted-foreground/60 mr-1">{{ formatDate(log.timestamp) }}</span>
            {{ formatTimestamp(log.timestamp) }}
          </div>

          <!-- Level -->
          <div class="flex items-center">
            <span
              class="inline-flex items-center gap-1 text-[10px] font-medium uppercase tracking-wide"
              :class="levelColor(log.level)"
            >
              <Icon :icon="levelIcon(log.level)" class="size-3" />
              {{ log.level }}
            </span>
          </div>

          <!-- Source -->
          <div class="text-muted-foreground text-[12px] truncate">
            {{ log.source }}
          </div>

          <!-- Message -->
          <div class="text-foreground truncate font-mono text-[12px]">
            {{ log.message }}
          </div>
        </div>
      </div>

      <!-- Footer stats -->
      <div class="mt-4 text-[11px] text-muted-foreground">
        <span>{{ filteredLogs.length }} of {{ logs.length }} entries</span>
      </div>
    </div>
  </div>
</template>
