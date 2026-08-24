<script setup lang="ts">
import { Icon } from '@iconify/vue'
import { computed, onMounted, ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

interface CronJob {
  id: string
  name: string | null
  command: string
  schedule?: {
    kind: 'cron' | 'at' | 'every'
    expr?: string
    tz?: string | null
    at?: string
    every_ms?: number
  }
  expression?: string
  next_run: string
  last_run: string | null
  last_status: string | null
  enabled: boolean
  job_type?: string
  delete_after_run?: boolean
}

interface CronAddBody {
  name?: string
  schedule: string
  command: string
  tz?: string
}

type ScheduleMode = 'daily' | 'weekdays' | 'weekly' | 'monthly' | 'interval' | 'custom'

const auth = useAuthStore()
const jobs = ref<CronJob[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const showAddModal = ref(false)
const submitting = ref(false)
const formName = ref('')
const formCommand = ref('')
const formError = ref<string | null>(null)
const selectedJob = ref<CronJob | null>(null)
const deletingJob = ref(false)
const formScheduleMode = ref<ScheduleMode>('daily')
const formTime = ref('09:00')
const formWeekday = ref('1')
const formMonthDay = ref('1')
const formIntervalMinutes = ref('15')
const formCustomCron = ref('')
const localTimeZone = detectLocalTimeZone()
const formTimezone = ref(localTimeZone)

const searchQuery = ref('')
const filteredJobs = computed(() => {
  if (!searchQuery.value) return jobs.value
  const query = searchQuery.value.toLowerCase()
  return jobs.value.filter(j => 
    (j.name?.toLowerCase().includes(query)) ||
    (j.command.toLowerCase().includes(query)) ||
    (j.job_type?.toLowerCase().includes(query))
  )
})

const weekdayOptions = [
  { value: '0', label: 'Sun', summary: 'Sunday' },
  { value: '1', label: 'Mon', summary: 'Monday' },
  { value: '2', label: 'Tue', summary: 'Tuesday' },
  { value: '3', label: 'Wed', summary: 'Wednesday' },
  { value: '4', label: 'Thu', summary: 'Thursday' },
  { value: '5', label: 'Fri', summary: 'Friday' },
  { value: '6', label: 'Sat', summary: 'Saturday' },
] as const
const scheduleModeOptions = [
  { value: 'daily', label: 'Daily', description: 'Every day at a set time' },
  { value: 'weekdays', label: 'Weekdays', description: 'Monday through Friday' },
  { value: 'weekly', label: 'Weekly', description: 'One day each week' },
  { value: 'monthly', label: 'Monthly', description: 'Same day each month' },
  { value: 'interval', label: 'Interval', description: 'Every few minutes or hours' },
  { value: 'custom', label: 'Custom', description: 'Enter raw cron yourself' },
] as const satisfies ReadonlyArray<{ value: ScheduleMode, label: string, description: string }>
const intervalOptions = [
  { value: '5', label: '5 min' },
  { value: '15', label: '15 min' },
  { value: '30', label: '30 min' },
  { value: '60', label: '1 hour' },
  { value: '120', label: '2 hours' },
] as const

const availableTimeZones = computed(() => {
  const common = [
    localTimeZone,
    'UTC',
    'Europe/London',
    'Europe/Istanbul',
    'America/New_York',
    'America/Los_Angeles',
    'Asia/Tokyo',
    'Asia/Singapore',
    'Australia/Sydney',
  ]
  const supportedValuesOf = (Intl as unknown as {
    supportedValuesOf?: (input: string) => string[]
  }).supportedValuesOf
  const supported = supportedValuesOf ? supportedValuesOf('timeZone') : []

  return Array.from(new Set([...common.filter(Boolean), ...supported]))
})

function jobLabel(job: CronJob): string {
  const name = (job.name ?? '').trim()
  if (name.length > 0)
    return name
  return job.command
}

function formatDateTime(value: string | null): string {
  if (!value)
    return '—'
  const date = new Date(value)
  if (Number.isNaN(date.getTime()))
    return '—'
  const formatted = date.toLocaleString('en-US', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true
  })
  return formatted.replace(', ', ' at ')
}

function jobPillClass(job: CronJob): string {
  if (!job.enabled)
    return 'bg-muted/50 text-muted-foreground border-border/50'

  const status = (job.last_status ?? '').toLowerCase()
  if (status === 'ok' || status === 'success')
    return 'bg-emerald-500/10 text-emerald-600 border-emerald-500/20'
  if (status === 'error' || status === 'failed')
    return 'bg-destructive/10 text-destructive border-destructive/20'

  return 'bg-primary/10 text-primary border-primary/20'
}

function detectLocalTimeZone(): string {
  return Intl.DateTimeFormat().resolvedOptions().timeZone || 'UTC'
}

function parseTimeParts(value: string): { hour: number, minute: number } | null {
  const [hourRaw, minuteRaw] = value.split(':')
  const hour = Number.parseInt(hourRaw ?? '', 10)
  const minute = Number.parseInt(minuteRaw ?? '', 10)

  if (
    !Number.isInteger(hour)
    || !Number.isInteger(minute)
    || hour < 0
    || hour > 23
    || minute < 0
    || minute > 59
  ) {
    return null
  }

  return { hour, minute }
}

function cronForInterval(totalMinutes: number): string {
  if (!Number.isInteger(totalMinutes) || totalMinutes <= 0)
    return ''

  if (totalMinutes < 60)
    return `*/${totalMinutes} * * * *`

  if (totalMinutes % 60 === 0) {
    const hours = totalMinutes / 60
    return hours === 1 ? '0 * * * *' : `0 */${hours} * * *`
  }

  return ''
}

function chipButtonClass(active: boolean): string {
  if (active) {
    return 'border-primary/30 bg-primary/6 text-foreground shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]'
  }

  return 'border-border/45 bg-background/75 text-muted-foreground hover:border-border/70 hover:bg-card/35 hover:text-foreground'
}

const normalizedTimezone = computed(() => formTimezone.value.trim())

const scheduleExpression = computed(() => {
  if (formScheduleMode.value === 'custom')
    return formCustomCron.value.trim()

  const parsedTime = parseTimeParts(formTime.value)
  if (!parsedTime)
    return ''

  const { hour, minute } = parsedTime

  if (formScheduleMode.value === 'daily')
    return `${minute} ${hour} * * *`

  if (formScheduleMode.value === 'weekdays')
    return `${minute} ${hour} * * 1-5`

  if (formScheduleMode.value === 'weekly')
    return `${minute} ${hour} * * ${formWeekday.value}`

  if (formScheduleMode.value === 'monthly') {
    const day = Number.parseInt(formMonthDay.value, 10)
    if (!Number.isInteger(day) || day < 1 || day > 28)
      return ''

    return `${minute} ${hour} ${day} * *`
  }

  if (formScheduleMode.value === 'interval') {
    const interval = Number.parseInt(formIntervalMinutes.value, 10)
    return cronForInterval(interval)
  }

  return ''
})

function formatSchedule(job: CronJob): string {
  if (!job.schedule) {
    return job.expression ?? '—'
  }

  if (job.schedule.kind === 'cron') {
    const expr = job.schedule.expr ?? job.expression ?? '—'
    return job.schedule.tz ? `${expr} (${job.schedule.tz})` : expr
  }

  if (job.schedule.kind === 'at') {
    return `Once at ${formatDateTime(job.schedule.at ?? null)}`
  }

  if (job.schedule.kind === 'every') {
    const everyMs = job.schedule.every_ms
    if (!everyMs)
      return 'Interval'

    if (everyMs % 3_600_000 === 0)
      return `Every ${everyMs / 3_600_000}h`
    if (everyMs % 60_000 === 0)
      return `Every ${everyMs / 60_000}m`
    if (everyMs % 1_000 === 0)
      return `Every ${everyMs / 1_000}s`
    return `Every ${everyMs}ms`
  }

  return '—'
}

function resetAddForm() {
  formName.value = ''
  formCommand.value = ''
  formScheduleMode.value = 'daily'
  formTime.value = '09:00'
  formWeekday.value = '1'
  formMonthDay.value = '1'
  formIntervalMinutes.value = '15'
  formCustomCron.value = ''
  formTimezone.value = localTimeZone
  formError.value = null
}

async function addJob() {
  const schedule = scheduleExpression.value.trim()
  if (!schedule || !formCommand.value.trim()) {
    formError.value = 'Choose a schedule and command.'
    return
  }

  if (!normalizedTimezone.value) {
    formError.value = 'Timezone is required.'
    return
  }

  submitting.value = true
  formError.value = null

  const payload: CronAddBody = {
    schedule,
    command: formCommand.value.trim(),
    tz: normalizedTimezone.value,
  }

  if (formName.value.trim()) {
    payload.name = formName.value.trim()
  }

  try {
    await auth.fetchWithAuth('/api/cron', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    })
    showAddModal.value = false
    resetAddForm()
    await fetchJobs()
  }
  catch (err: any) {
    formError.value = err?.message || 'Failed to add cron job'
  }
  finally {
    submitting.value = false
  }
}

async function deleteSelectedJob() {
  if (!selectedJob.value)
    return

  deletingJob.value = true
  try {
    await auth.fetchWithAuth(`/api/cron/${encodeURIComponent(selectedJob.value.id)}`, {
      method: 'DELETE',
    })
    selectedJob.value = null
    await fetchJobs()
  }
  catch (err: any) {
    error.value = err?.message || 'Failed to delete cron job'
  }
  finally {
    deletingJob.value = false
  }
}

async function fetchJobs() {
  loading.value = true
  error.value = null
  try {
    const response = await auth.fetchWithAuth<{ jobs: CronJob[] } | CronJob[]>('/api/cron')
    if (Array.isArray(response)) {
      jobs.value = response
    }
    else if (response && 'jobs' in response) {
      jobs.value = response.jobs
    }
    else {
      jobs.value = []
    }
  }
  catch (err: any) {
    console.error('[cron-jobs] fetchJobs error:', err)
    error.value = err.message || 'Failed to load cron jobs'
    jobs.value = []
  }
  finally {
    loading.value = false
  }
}

onMounted(() => {
  fetchJobs()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden bg-background text-[13px]">
    <!-- Header -->
    <div class="flex-shrink-0 px-6 pt-6 pb-2 border-b border-border/20">
      <div class="flex items-center justify-between mb-4">
        <h1 class="font-sans text-[20px] font-semibold text-foreground">
          Tasks
        </h1>
        <button
          class="flex items-center gap-1.5 px-3 py-1.5 bg-primary text-primary-foreground rounded-xl text-[12px] font-medium hover:opacity-90 transition-opacity"
          @click="showAddModal = true"
        >
          <Icon icon="hugeicons:add-01" class="size-3.5" />
          Add Job
        </button>
      </div>

      <!-- Action Bar -->
      <div class="flex items-center justify-between mt-1">
        <div class="relative w-full max-w-[320px]">
          <Icon icon="hugeicons:search-02" class="absolute left-2.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground" />
          <input 
            v-model="searchQuery" 
            type="text" 
            placeholder="Search tasks..." 
            class="w-full pl-8 pr-3 py-1.5 text-[13px] bg-background border border-border/50 rounded-xl placeholder:text-muted-foreground/60 focus:outline-none focus:border-primary/40 focus:ring-1 focus:ring-primary/20 transition-all"
          >
        </div>
        <div class="flex items-center gap-1.5 text-[12px] font-medium text-muted-foreground">
          <Icon icon="hugeicons:task-01" class="size-4" />
          Total tasks: <span class="text-foreground ml-0.5">{{ jobs.length }}</span>
        </div>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="loading" class="flex-1 flex items-center justify-center">
      <Icon icon="hugeicons:loading-03" class="size-6 animate-spin text-muted-foreground" />
    </div>

    <!-- Table View -->
    <div v-else class="flex-1 overflow-hidden flex flex-col px-6 pb-6 pt-4">
      <div v-if="error" class="mb-3 rounded-xl border border-destructive/20 bg-destructive/5 px-3 py-2 text-[12px] text-destructive flex items-center justify-between shrink-0">
        <span>{{ error }}</span>
        <button class="text-[12px] font-medium hover:underline" @click="fetchJobs">
          Retry
        </button>
      </div>

      <div class="flex-1 rounded-2xl border border-border/30 overflow-auto bg-card/10">
        <table class="w-full text-left border-collapse">
          <thead class="sticky top-0 bg-card/95 backdrop-blur-sm z-10 border-b border-border/30">
            <tr>
              <th class="py-3 px-4 font-medium text-muted-foreground text-[12px] w-[35%]">
                <div class="flex items-center gap-1.5">
                  <Icon icon="hugeicons:number-symbol-square" class="size-3.5" />
                  Task name
                </div>
              </th>
              <th class="py-3 px-4 font-medium text-muted-foreground text-[12px] w-[20%]">
                <div class="flex items-center gap-1.5">
                  <Icon icon="hugeicons:user-circle" class="size-3.5" />
                  Run by
                </div>
              </th>
              <th class="py-3 px-4 font-medium text-muted-foreground text-[12px] w-[15%]">
                <div class="flex items-center gap-1.5">
                  <Icon icon="hugeicons:loading-01" class="size-3.5" />
                  Status
                </div>
              </th>
              <th class="py-3 px-4 font-medium text-muted-foreground text-[12px] w-[15%]">
                <div class="flex items-center gap-1.5">
                  <Icon icon="hugeicons:clock-01" class="size-3.5" />
                  Date created
                </div>
              </th>
              <th class="py-3 px-4 font-medium text-muted-foreground text-[12px] w-[15%]">
                <div class="flex items-center gap-1.5">
                  <Icon icon="hugeicons:time-update" class="size-3.5" />
                  Last updated
                </div>
              </th>
            </tr>
          </thead>
          <tbody>
            <tr 
              v-for="job in filteredJobs" 
              :key="job.id"
              class="border-b border-border/20 hover:bg-card/40 transition-colors cursor-pointer group last:border-0"
              @click="selectedJob = job"
            >
              <td class="py-3 px-4 text-[13px] text-foreground font-medium truncate max-w-0">
                {{ jobLabel(job) }}
              </td>
              <td class="py-3 px-4 text-[13px] text-muted-foreground">
                <div class="flex items-center gap-1.5">
                  <Icon :icon="job.job_type === 'slide_builder' ? 'hugeicons:presentation-02' : 'hugeicons:bot'" class="size-4" />
                  <span class="truncate">{{ job.job_type ?? 'System Daemon' }}</span>
                </div>
              </td>
              <td class="py-3 px-4">
                <div class="inline-flex items-center gap-1.5 px-2 py-1 rounded-[10px] border text-[11px] font-medium" :class="jobPillClass(job)">
                  <Icon :icon="job.enabled ? 'hugeicons:play-circle' : 'hugeicons:moon-02'" class="size-3" />
                  {{ job.enabled ? (job.last_status ?? 'Active') : 'Inactive' }}
                </div>
              </td>
              <td class="py-3 px-4 text-[12px] text-muted-foreground whitespace-nowrap">
                {{ formatDateTime(job.next_run) }}
              </td>
              <td class="py-3 px-4 text-[12px] text-muted-foreground whitespace-nowrap">
                {{ formatDateTime(job.last_run) }}
              </td>
            </tr>
            <tr v-if="jobs.length === 0 && !loading">
              <td colspan="5" class="py-12 text-center">
                <div class="flex flex-col items-center justify-center gap-2 text-muted-foreground">
                  <Icon icon="hugeicons:task-01" class="size-8 opacity-20" />
                  <span class="text-[13px]">No tasks found.</span>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div v-if="showAddModal" class="fixed inset-0 z-40 bg-black/30 backdrop-blur-[1.5px] flex items-center justify-center p-4">
      <div class="w-full max-w-[580px] max-h-[82vh] overflow-y-auto rounded-[20px] border border-border/35 bg-card/92 p-6 shadow-[0_20px_55px_-28px_rgba(0,0,0,0.72)] motion-safe:animate-in motion-safe:fade-in-0 motion-safe:zoom-in-95 motion-safe:duration-200 motion-safe:ease-out">
        <div class="flex items-center justify-between mb-6">
          <h2 class="font-sans text-[16px] font-medium text-foreground">
            Add Scheduled Job
          </h2>
          <button
            class="p-1.5 rounded-xl text-muted-foreground transition-colors duration-150 hover:text-foreground hover:bg-muted/45"
            @click="showAddModal = false; resetAddForm()"
          >
            <Icon icon="hugeicons:cancel-01" class="size-4" />
          </button>
        </div>

        <div class="space-y-5">
          <label class="block">
            <span class="mb-1.5 block text-[13px] text-muted-foreground">Name (optional)</span>
            <input
              v-model="formName"
              type="text"
              class="w-full rounded-xl border border-border/45 bg-background/90 px-3.5 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
              placeholder="Nightly Backup"
            >
          </label>

          <div class="space-y-1.5">
            <span class="block text-[13px] text-muted-foreground mb-1.5">Schedule pattern</span>
            <div class="relative">
              <select
                v-model="formScheduleMode"
                class="w-full appearance-none rounded-xl border border-border/45 bg-background/90 pl-3.5 pr-8 py-2.5 text-[13px] font-medium transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10 cursor-pointer text-foreground"
                @change="formError = null"
              >
                <option v-for="option in scheduleModeOptions" :key="option.value" :value="option.value">
                  {{ option.label }} - {{ option.description }}
                </option>
              </select>
              <Icon icon="hugeicons:arrow-down-01-sharp" class="absolute right-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
            </div>
          </div>

          <div class="space-y-5">
            <div v-if="formScheduleMode !== 'interval' && formScheduleMode !== 'custom'" class="grid gap-4 sm:grid-cols-2">
              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Time</span>
                <input
                  v-model="formTime"
                  type="time"
                  class="w-full rounded-xl border border-border/45 bg-background/90 px-3.5 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                >
              </label>

              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Timezone</span>
                <div class="relative">
                  <select
                    v-model="formTimezone"
                    class="w-full appearance-none rounded-xl border border-border/45 bg-background/90 pl-3.5 pr-8 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10 cursor-pointer text-foreground"
                  >
                    <option v-for="tz in availableTimeZones" :key="tz" :value="tz">{{ tz }}</option>
                  </select>
                  <Icon icon="hugeicons:arrow-down-01-sharp" class="absolute right-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
                </div>
              </label>
            </div>

            <div v-if="formScheduleMode === 'weekly'" class="space-y-2">
              <p class="text-[13px] text-muted-foreground">
                Day of week
              </p>
              <div class="grid grid-cols-4 gap-2 sm:grid-cols-7">
                <button
                  v-for="option in weekdayOptions"
                  :key="option.value"
                  type="button"
                  class="rounded-xl border px-2 py-2 text-[12px] font-medium transition-all duration-150"
                  :class="chipButtonClass(formWeekday === option.value)"
                  @click="formWeekday = option.value"
                >
                  {{ option.label }}
                </button>
              </div>
            </div>

            <div v-if="formScheduleMode === 'monthly'" class="grid gap-4 sm:grid-cols-[minmax(0,160px)_1fr]">
              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Day of month</span>
                <input
                  v-model="formMonthDay"
                  type="number"
                  min="1"
                  max="28"
                  class="w-full rounded-xl border border-border/45 bg-background/90 px-3.5 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                >
              </label>
              <div class="flex items-end pb-2.5">
                <p class="text-[12px] text-muted-foreground">
                  Limited to 1-28 so it always exists in every month.
                </p>
              </div>
            </div>

            <div v-if="formScheduleMode === 'interval'" class="space-y-4">
              <div class="space-y-2">
                <p class="text-[13px] text-muted-foreground">
                  Repeat every
                </p>
                <div class="grid grid-cols-3 gap-2 sm:grid-cols-5">
                  <button
                    v-for="option in intervalOptions"
                    :key="option.value"
                    type="button"
                    class="rounded-xl border px-2.5 py-2 text-[12px] font-medium transition-all duration-150"
                    :class="chipButtonClass(formIntervalMinutes === option.value)"
                    @click="formIntervalMinutes = option.value"
                  >
                    {{ option.label }}
                  </button>
                </div>
              </div>

              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Timezone</span>
                <div class="relative">
                  <select
                    v-model="formTimezone"
                    class="w-full appearance-none rounded-xl border border-border/45 bg-background/90 pl-3.5 pr-8 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10 cursor-pointer text-foreground"
                  >
                    <option v-for="tz in availableTimeZones" :key="tz" :value="tz">{{ tz }}</option>
                  </select>
                  <Icon icon="hugeicons:arrow-down-01-sharp" class="absolute right-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
                </div>
              </label>
            </div>

            <div v-if="formScheduleMode === 'custom'" class="space-y-4">
              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Custom cron expression</span>
                <input
                  v-model="formCustomCron"
                  type="text"
                  class="w-full rounded-xl border border-border/45 bg-background/90 px-3.5 py-2.5 font-mono text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10"
                  placeholder="0 9 * * *"
                >
                <p class="mt-1.5 text-[11px] text-muted-foreground">Format: <code>minute hour day month weekday</code>.</p>
              </label>

              <label class="block">
                <span class="mb-1.5 block text-[13px] text-muted-foreground">Timezone</span>
                <div class="relative">
                  <select
                    v-model="formTimezone"
                    class="w-full appearance-none rounded-xl border border-border/45 bg-background/90 pl-3.5 pr-8 py-2.5 text-[13px] transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10 cursor-pointer text-foreground"
                  >
                    <option v-for="tz in availableTimeZones" :key="tz" :value="tz">{{ tz }}</option>
                  </select>
                  <Icon icon="hugeicons:arrow-down-01-sharp" class="absolute right-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
                </div>
              </label>
            </div>

          </div>

          <div class="space-y-1.5">
            <span class="block text-[13px] text-muted-foreground mb-1.5">Task Action</span>
            <div class="relative">
              <select
                v-model="formCommand"
                class="w-full appearance-none rounded-xl border border-border/45 bg-background/90 pl-3.5 pr-8 py-2.5 text-[13px] font-medium transition-all duration-150 focus:border-primary/35 focus:outline-none focus:ring-2 focus:ring-primary/10 cursor-pointer text-foreground"
              >
                <option value="" disabled>Select an action...</option>
                <option value="python3 scripts/slide_builder.py">Generate Slide Deck</option>
                <option value="python3 scripts/backup.py">System Backup</option>
                <option value="git fetch --all">Data Sync</option>
                <option value="node scripts/cleanup.js">System Cleanup</option>
              </select>
              <Icon icon="hugeicons:arrow-down-01-sharp" class="absolute right-3.5 top-1/2 -translate-y-1/2 size-4 text-muted-foreground pointer-events-none" />
            </div>
            <p class="mt-1.5 text-[11px] text-muted-foreground">Select the predefined system action to run on this schedule.</p>
          </div>

          <p v-if="formError" class="text-[13px] text-destructive">
            {{ formError }}
          </p>
        </div>

        <div class="mt-8 flex items-center justify-end gap-3 border-t border-border/20 pt-5">
          <button
            class="rounded-xl border border-border/45 px-4 py-2 text-[13px] font-medium text-muted-foreground transition-all duration-150 hover:text-foreground hover:bg-muted/45"
            @click="showAddModal = false; resetAddForm()"
          >
            Cancel
          </button>
          <button
            :disabled="submitting"
            class="rounded-xl bg-primary px-4 py-2 text-[13px] font-medium text-primary-foreground transition-all duration-150 hover:brightness-105 disabled:opacity-60"
            @click="addJob"
          >
            {{ submitting ? 'Adding...' : 'Add Job' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="selectedJob" class="fixed inset-0 z-40 bg-black/30 flex items-center justify-center p-4">
      <div class="w-full max-w-md rounded-2xl border border-border/40 bg-card p-5 shadow-md">
        <div class="flex items-center justify-between mb-4">
          <h2 class="font-sans text-[16px] font-medium text-foreground">
            Cron Job
          </h2>
          <button
            class="p-1 rounded-xl text-muted-foreground hover:text-foreground hover:bg-muted/50"
            @click="selectedJob = null"
          >
            <Icon icon="hugeicons:cancel-01" class="size-4" />
          </button>
        </div>

        <div class="space-y-2 text-[13px]">
          <p><span class="text-muted-foreground">Name:</span> {{ jobLabel(selectedJob) }}</p>
          <p><span class="text-muted-foreground">Type:</span> {{ selectedJob.job_type ?? 'shell' }}</p>
          <p><span class="text-muted-foreground">Schedule:</span> {{ formatSchedule(selectedJob) }}</p>
          <p><span class="text-muted-foreground">Command:</span> {{ selectedJob.command }}</p>
          <p><span class="text-muted-foreground">Next run:</span> {{ formatDateTime(selectedJob.next_run) }}</p>
          <p><span class="text-muted-foreground">Last run:</span> {{ formatDateTime(selectedJob.last_run) }}</p>
          <p><span class="text-muted-foreground">Status:</span> {{ selectedJob.last_status ?? 'pending' }}</p>
          <p><span class="text-muted-foreground">Auto-delete:</span> {{ selectedJob.delete_after_run ? 'yes' : 'no' }}</p>
        </div>

        <div class="mt-5 flex items-center justify-between">
          <button
            :disabled="deletingJob"
            class="px-3 py-1.5 rounded-xl bg-destructive/10 text-destructive text-[12px] font-medium border border-destructive/20 disabled:opacity-60"
            @click="deleteSelectedJob"
          >
            {{ deletingJob ? 'Deleting...' : 'Delete Job' }}
          </button>
          <button
            class="px-3 py-1.5 rounded-xl border border-border/50 text-[12px] font-medium text-muted-foreground hover:text-foreground hover:bg-muted/50"
            @click="selectedJob = null"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
