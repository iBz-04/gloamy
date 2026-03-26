<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { Icon } from '@iconify/vue'
import { useAuthStore } from '@/stores/auth'

type Severity = 'ok' | 'warn' | 'error'

interface DiagnosticResult {
  name: string
  severity: Severity
  message: string
  suggestion?: string
}

interface DoctorResponse {
  results: DiagnosticResult[]
  summary: {
    ok: number
    warnings: number
    errors: number
  }
}

const auth = useAuthStore()
const results = ref<DiagnosticResult[]>([])
const summary = ref<{ ok: number; warnings: number; errors: number }>({ ok: 0, warnings: 0, errors: 0 })
const loading = ref(true)
const error = ref<string | null>(null)
const activeFilter = ref<'all' | Severity>('all')

const filters: Array<'all' | Severity> = ['all', 'ok', 'warn', 'error']

const filteredResults = computed(() => {
  if (activeFilter.value === 'all') return results.value
  return results.value.filter(r => r.severity === activeFilter.value)
})

function severityIcon(severity: Severity): string {
  switch (severity) {
    case 'ok': return 'ph:check-circle'
    case 'warn': return 'ph:warning'
    case 'error': return 'ph:x-circle'
  }
}

function severityColor(severity: Severity): string {
  switch (severity) {
    case 'ok': return 'text-emerald-500'
    case 'warn': return 'text-amber-500'
    case 'error': return 'text-red-500'
  }
}

function filterLabel(filter: 'all' | Severity): string {
  switch (filter) {
    case 'all': return 'All'
    case 'ok': return 'Passed'
    case 'warn': return 'Warnings'
    case 'error': return 'Errors'
  }
}

function filterCount(filter: 'all' | Severity): number {
  switch (filter) {
    case 'all': return results.value.length
    case 'ok': return summary.value.ok
    case 'warn': return summary.value.warnings
    case 'error': return summary.value.errors
  }
}

async function runDiagnostics() {
  loading.value = true
  error.value = null
  try {
    const response = await auth.fetchWithAuth<DoctorResponse>('/api/doctor', { method: 'POST' })
    results.value = response.results || []
    summary.value = response.summary || { ok: 0, warnings: 0, errors: 0 }
  } catch (err: any) {
    console.error('[doctor] runDiagnostics error:', err)
    error.value = err.message || 'Failed to run diagnostics'
    results.value = []
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  runDiagnostics()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[15px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-4">
      <div class="flex items-center justify-between mb-4">
        <div>
          <h1 class="text-lg font-semibold text-foreground">Diagnostics</h1>
          <p class="text-[13px] text-muted-foreground">System health checks and configuration validation</p>
        </div>
        <button
          @click="runDiagnostics"
          :disabled="loading"
          class="flex items-center gap-1.5 px-3 py-1.5 text-[12px] font-medium rounded-md text-muted-foreground hover:text-foreground transition-colors disabled:opacity-50"
        >
          <Icon icon="ph:arrow-clockwise" class="size-3.5" :class="{ 'animate-spin': loading }" />
          Run Again
        </button>
      </div>

      <!-- Filters -->
      <div class="flex items-center gap-1">
        <button
          v-for="filter in filters"
          :key="filter"
          @click="activeFilter = filter"
          class="px-3 py-1.5 text-[12px] font-medium rounded-md whitespace-nowrap transition-colors flex items-center gap-1.5"
          :class="activeFilter === filter
            ? 'text-foreground'
            : 'text-muted-foreground hover:text-foreground'"
        >
          {{ filterLabel(filter) }}
          <span class="text-[10px] opacity-70">({{ filterCount(filter) }})</span>
        </button>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Skeleton filter tabs -->
      <div class="flex items-center gap-1 mb-6">
        <div v-for="i in 4" :key="i" class="h-6 bg-muted-foreground/10 rounded-sm animate-pulse w-20" />
      </div>
      <!-- Skeleton results -->
      <div v-for="i in 6" :key="i" class="flex items-start gap-3 py-3">
        <div class="size-4 mt-0.5 bg-muted-foreground/10 rounded-sm animate-pulse flex-shrink-0" />
        <div class="flex-1 min-w-0">
          <div class="h-4 bg-muted-foreground/10 rounded-sm animate-pulse w-32 mb-2" />
          <div class="h-3 bg-muted-foreground/10 rounded-sm animate-pulse w-3/4" />
        </div>
      </div>
    </div>

    <!-- Content -->
    <div v-else class="flex-1 overflow-y-auto px-6 py-4">
      <!-- Error -->
      <div v-if="error" class="mb-4 px-3 py-2 text-[12px] text-amber-500 flex items-center gap-2">
        <Icon icon="ph:warning" class="size-4" />
        <span>{{ error }}</span>
      </div>

      <!-- Empty state -->
      <div v-if="filteredResults.length === 0 && !error" class="flex flex-col items-center justify-center py-16">
        <Icon icon="ph:stethoscope" class="size-12 text-muted-foreground/30 mb-3" />
        <p class="text-muted-foreground text-[13px]">No diagnostics to display</p>
      </div>

      <!-- Results list -->
      <div v-else class="space-y-1">
        <div
          v-for="(result, index) in filteredResults"
          :key="index"
          class="flex items-start gap-3 py-3"
        >
          <Icon
            :icon="severityIcon(result.severity)"
            class="size-4 mt-0.5 flex-shrink-0"
            :class="severityColor(result.severity)"
          />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-[13px] font-medium text-foreground">{{ result.name }}</span>
            </div>
            <p class="text-[12px] text-muted-foreground mt-0.5">{{ result.message }}</p>
            <p v-if="result.suggestion" class="text-[11px] text-muted-foreground/70 mt-1">
              {{ result.suggestion }}
            </p>
          </div>
        </div>
      </div>

      <!-- Summary -->
      <div v-if="results.length > 0" class="mt-6 pt-4 text-[11px] text-muted-foreground flex items-center gap-4">
        <span class="flex items-center gap-1">
          <Icon icon="ph:check-circle" class="size-3 text-emerald-500" />
          {{ summary.ok }} passed
        </span>
        <span v-if="summary.warnings > 0" class="flex items-center gap-1">
          <Icon icon="ph:warning" class="size-3 text-amber-500" />
          {{ summary.warnings }} warnings
        </span>
        <span v-if="summary.errors > 0" class="flex items-center gap-1">
          <Icon icon="ph:x-circle" class="size-3 text-red-500" />
          {{ summary.errors }} errors
        </span>
      </div>
    </div>
  </div>
</template>
