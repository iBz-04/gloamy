<script setup lang="ts">
import type { SSEEvent } from '@/lib/sse'
import { Icon } from '@iconify/vue'
import { computed, nextTick, onMounted, onUnmounted, ref, unref } from 'vue'
import { SSEClient } from '@/lib/sse'
import { useAuthStore } from '@/stores/auth'

type LogLevel = 'debug' | 'info' | 'warn' | 'error'

interface LogEntry {
  id: string
  timestamp: Date
  level: LogLevel
  source: string
  message: string
  type: string
}

type RawLog = Record<string, unknown>

const auth = useAuthStore()
const logs = ref<LogEntry[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const activeLevel = ref<'All' | LogLevel>('All')
const paused = ref(false)
const connected = ref(false)
const autoScroll = ref(true)
const containerRef = ref<HTMLElement | null>(null)

const maxEntries = 500
let logCounter = 0
let sseClient: SSEClient | null = null

const levels: Array<'All' | LogLevel> = ['All', 'debug', 'info', 'warn', 'error']

const filteredLogs = computed(() => {
  let result = logs.value

  if (activeLevel.value !== 'All') {
    result = result.filter(log => log.level === activeLevel.value)
  }

  const query = searchQuery.value.trim().toLowerCase()
  if (query) {
    result = result.filter(log =>
      log.message.toLowerCase().includes(query)
      || log.source.toLowerCase().includes(query)
      || log.type.toLowerCase().includes(query),
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

function asText(value: unknown): string | null {
  if (typeof value === 'string') {
    const trimmed = value.trim()
    return trimmed.length > 0 ? trimmed : null
  }
  if (typeof value === 'number' || typeof value === 'boolean') {
    return String(value)
  }
  return null
}

function stringifyValue(value: unknown): string {
  const text = asText(value)
  if (text !== null) {
    return text
  }

  try {
    return JSON.stringify(value)
  }
  catch {
    return '[unserializable]'
  }
}

function normalizeLevel(value: unknown): LogLevel {
  const normalized = asText(value)?.toLowerCase()
  if (normalized === 'debug') {
    return 'debug'
  }
  if (normalized === 'warn' || normalized === 'warning') {
    return 'warn'
  }
  if (normalized === 'error' || normalized === 'failed') {
    return 'error'
  }
  return 'info'
}

function levelFromEvent(event: SSEEvent): LogLevel {
  const type = String(event.type ?? '').toLowerCase()
  if (type.includes('error')) {
    return 'error'
  }
  if (type.includes('warn')) {
    return 'warn'
  }
  if (type.includes('debug')) {
    return 'debug'
  }
  if (event.success === false) {
    return 'error'
  }
  return 'info'
}

function sourceFromEvent(event: SSEEvent): string {
  return asText(event.component)
    ?? asText(event.provider)
    ?? asText(event.tool)
    ?? asText(event.type)
    ?? 'runtime'
}

function messageFromEvent(event: SSEEvent): string {
  const primary = asText(event.message) ?? asText(event.content) ?? asText(event.data)
  if (primary !== null) {
    return primary
  }

  const details = Object.entries(event)
    .filter(([key]) => !['type', 'timestamp', 'message', 'content', 'data'].includes(key))
    .slice(0, 4)
    .map(([key, value]) => `${key}=${stringifyValue(value)}`)

  if (details.length === 0) {
    return String(event.type ?? 'event')
  }

  return details.join(' | ')
}

function appendLog(entry: LogEntry) {
  logs.value = [...logs.value, entry].slice(-maxEntries)

  if (autoScroll.value) {
    void nextTick(() => {
      const container = containerRef.value
      if (container) {
        container.scrollTop = container.scrollHeight
      }
    })
  }
}

function appendSseEvent(event: SSEEvent) {
  if (paused.value) {
    return
  }

  const timestampRaw = asText(event.timestamp)
  const timestamp = timestampRaw ? new Date(timestampRaw) : new Date()

  logCounter += 1
  appendLog({
    id: `sse-${logCounter}`,
    timestamp: Number.isNaN(timestamp.getTime()) ? new Date() : timestamp,
    level: levelFromEvent(event),
    source: sourceFromEvent(event),
    message: messageFromEvent(event),
    type: String(event.type ?? 'event'),
  })
}

function mapRawLog(log: RawLog, index: number): LogEntry {
  const timestampRaw = asText(log.timestamp)
  const timestamp = timestampRaw ? new Date(timestampRaw) : new Date()

  return {
    id: asText(log.id) ?? `log-${index}`,
    timestamp: Number.isNaN(timestamp.getTime()) ? new Date() : timestamp,
    level: normalizeLevel(log.level),
    source: asText(log.source) ?? asText(log.module) ?? 'system',
    message: asText(log.message) ?? asText(log.msg) ?? stringifyValue(log),
    type: asText(log.type) ?? 'log',
  }
}

function normalizeLogsResponse(response: unknown): RawLog[] {
  if (Array.isArray(response)) {
    return response.filter(item => !!item && typeof item === 'object') as RawLog[]
  }

  if (response && typeof response === 'object' && 'logs' in response) {
    const logsField = (response as { logs?: unknown }).logs
    if (Array.isArray(logsField)) {
      return logsField.filter(item => !!item && typeof item === 'object') as RawLog[]
    }
  }

  return []
}

async function fetchSnapshot(showLoading = true) {
  if (showLoading) {
    loading.value = true
  }

  error.value = null

  try {
    const response = await auth.fetchWithAuth('/api/logs')
    const rawLogs = normalizeLogsResponse(response)
    logs.value = rawLogs.map(mapRawLog).slice(-maxEntries)
  }
  catch (err: unknown) {
    error.value = err instanceof Error ? err.message : 'Failed to load logs'
    logs.value = []
  }
  finally {
    if (showLoading) {
      loading.value = false
    }
  }
}

function connectSse() {
  sseClient?.disconnect()

  sseClient = new SSEClient({
    path: '/api/events',
    getBaseUrl: () => String(unref(auth.baseUrl) ?? '').trim(),
    getToken: () => {
      const token = unref(auth.token)
      return typeof token === 'string' && token.trim().length > 0 ? token : null
    },
  })

  sseClient.onConnect = () => {
    connected.value = true
    error.value = null
  }

  sseClient.onError = (err) => {
    connected.value = false

    if (err instanceof Error) {
      error.value = err.message
      if (err.message.includes('401')) {
        sseClient?.disconnect()
      }
    }
    else {
      error.value = 'Event stream disconnected'
    }
  }

  sseClient.onEvent = (event) => {
    appendSseEvent(event)
  }

  sseClient.connect()
}

function clearLogs() {
  logs.value = []
}

function togglePause() {
  paused.value = !paused.value
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
  if (level === 'All') {
    return 'All'
  }
  return level.charAt(0).toUpperCase() + level.slice(1)
}

function levelIcon(level: LogLevel): string {
  if (level === 'debug') {
    return 'ph:bug'
  }
  if (level === 'info') {
    return 'ph:info'
  }
  if (level === 'warn') {
    return 'ph:warning'
  }
  return 'ph:x-circle'
}

function levelColor(level: LogLevel): string {
  if (level === 'debug') {
    return 'text-slate-400'
  }
  if (level === 'info') {
    return 'text-blue-500'
  }
  if (level === 'warn') {
    return 'text-amber-500'
  }
  return 'text-red-500'
}

function handleScroll() {
  const container = containerRef.value
  if (!container) {
    return
  }

  const distanceFromBottom = container.scrollHeight - container.scrollTop - container.clientHeight
  autoScroll.value = distanceFromBottom < 50
}

onMounted(async () => {
  await fetchSnapshot()
  connectSse()
})

onUnmounted(() => {
  sseClient?.disconnect()
  sseClient = null
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <div class="flex-shrink-0 px-6 pt-6 pb-4 border-b border-border/30 space-y-4">
      <div class="flex items-center justify-between gap-4">
        <div class="relative flex-1 max-w-md">
          <Icon icon="ph:magnifying-glass" class="absolute left-3 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search logs..."
            class="w-full pl-10 pr-4 py-2 text-[13px] bg-transparent focus:outline-none text-foreground placeholder:text-muted-foreground"
          >
        </div>

        <div class="flex items-center gap-2">
          <span class="inline-flex items-center gap-1.5 text-[12px] text-muted-foreground">
            <span class="size-2 rounded-full" :class="connected ? 'bg-emerald-500' : 'bg-red-500'" />
            {{ connected ? 'Connected' : 'Disconnected' }}
          </span>

          <button
            class="px-3 py-1.5 text-[12px] font-medium rounded-md transition-colors"
            :class="paused ? 'text-foreground border border-border/60' : 'text-primary border border-primary/30'"
            @click="togglePause"
          >
            {{ paused ? 'Resume' : 'Pause' }}
          </button>

          <button
            class="px-3 py-1.5 text-[12px] font-medium rounded-md border border-border/60 hover:bg-card/60 transition-colors"
            @click="fetchSnapshot()"
          >
            Refresh
          </button>

          <button
            class="px-3 py-1.5 text-[12px] font-medium rounded-md border border-border/60 hover:bg-card/60 transition-colors"
            @click="clearLogs"
          >
            Clear
          </button>
        </div>
      </div>

      <div class="flex items-center gap-1">
        <button
          v-for="level in levels"
          :key="level"
          class="px-3 py-1.5 text-[12px] font-medium rounded-md whitespace-nowrap transition-colors flex items-center gap-1.5"
          :class="activeLevel === level
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground'"
          @click="activeLevel = level"
        >
          <Icon v-if="level !== 'All'" :icon="levelIcon(level)" class="size-3.5" />
          {{ formatLevelLabel(level) }}
          <span v-if="level !== 'All'" class="text-[10px] opacity-70">({{ levelCounts[level] }})</span>
        </button>
      </div>
    </div>

    <div v-if="loading" class="flex-1 overflow-y-auto px-6 py-4">
      <div
        v-for="i in 8"
        :key="i"
        class="h-10 rounded-xl bg-card/30 border border-border/30 animate-pulse mb-2"
      />
    </div>

    <div
      v-else
      ref="containerRef"
      class="flex-1 overflow-y-auto px-6 py-4"
      @scroll="handleScroll"
    >
      <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
        <Icon icon="ph:warning" class="size-4" />
        <span>{{ error }}</span>
      </div>

      <div v-if="filteredLogs.length === 0" class="flex flex-col items-center justify-center py-16">
        <Icon icon="ph:scroll" class="size-12 text-muted-foreground/30 mb-3" />
        <p class="text-muted-foreground text-[13px]">
          {{ paused ? 'Stream is paused.' : 'No logs found' }}
        </p>
      </div>

      <div v-else>
        <div class="grid grid-cols-[110px_70px_120px_120px_1fr] gap-4 px-4 py-3 text-[12px] font-medium text-muted-foreground border-b border-border/30">
          <div>Time</div>
          <div>Level</div>
          <div>Source</div>
          <div>Type</div>
          <div>Message</div>
        </div>

        <div
          v-for="log in filteredLogs"
          :key="log.id"
          class="grid grid-cols-[110px_70px_120px_120px_1fr] gap-4 px-4 py-2.5 items-center text-[13px] border-b border-border/20 last:border-b-0"
        >
          <div class="text-muted-foreground text-[11px] font-mono">
            <span class="text-muted-foreground/60 mr-1">{{ formatDate(log.timestamp) }}</span>
            {{ formatTimestamp(log.timestamp) }}
          </div>

          <div>
            <span class="inline-flex items-center gap-1 text-[10px] font-medium uppercase tracking-wide" :class="levelColor(log.level)">
              <Icon :icon="levelIcon(log.level)" class="size-3" />
              {{ log.level }}
            </span>
          </div>

          <div class="text-muted-foreground text-[12px] truncate" :title="log.source">
            {{ log.source }}
          </div>

          <div class="text-muted-foreground text-[12px] truncate" :title="log.type">
            {{ log.type }}
          </div>

          <div class="text-foreground font-mono text-[12px] truncate" :title="log.message">
            {{ log.message }}
          </div>
        </div>
      </div>

      <div class="mt-4 text-[11px] text-muted-foreground flex items-center justify-between">
        <span>{{ filteredLogs.length }} of {{ logs.length }} entries</span>
        <span v-if="!autoScroll">Auto-scroll paused</span>
      </div>
    </div>
  </div>
</template>
